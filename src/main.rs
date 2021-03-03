#[macro_use]
extern crate log;
extern crate log4rs;
extern crate serde_json;
extern crate wildmatch;

use std::env;
use std::path::Path;

use clap::{App, Arg, ArgMatches};
use serde_json::json;

use crate::apache::apache::get_apache_vhosts;
use crate::domain::domain::{Site, VirtualHost};
use crate::filter::filter::filter_vhosts;
use crate::logging::logging::get_logging_config;
use crate::nginx::nginx::get_nginx_vhosts;
use crate::site::site::get_sites_from_vhosts;

mod logging;

mod main_tests;

mod webserver;
mod webserver_tests;
mod nginx;

mod domain;

mod apache;

mod site;

mod filter;
mod filter_tests;
mod site_tests;
mod nginx_tests;
mod apache_tests;
mod test_utils;
mod test_samples;

const DEFAULT_HTTP_PORT: i32 = 80;
const DEFAULT_HTTPS_PORT: i32 = 443;

const INCLUDE_DOMAINS_WITH_WWW: &str = "include-www";
const INCLUDE_CUSTOM_PORTS_OPTION: &str = "include-custom-ports";

const WWW_SEARCH_PATTERN: &str = "www.";

const WORKDIR: &str = "/etc/zabbix";

const WORK_DIR_ARGUMENT: &str = "work-dir";
const WORK_DIR_SHORT_ARGUMENT: &str = "d";

const NGINX_VHOSTS_PATH: &str = "/etc/nginx/conf.d";
const APACHE_VHOSTS_PATH: &str = "/etc/httpd/conf.d";

const NGINX_VHOSTS_PATH_ARGUMENT: &str = "nginx-vhosts-path";
const NGINX_VHOSTS_PATH_SHORT_ARGUMENT: &str = "n";
const APACHE_VHOSTS_PATH_ARGUMENT: &str = "apache-vhosts-path";
const APACHE_VHOSTS_PATH_SHORT_ARGUMENT: &str = "a";

const USE_DATA_PROPERTY_ARGUMENT: &str = "use-data-property";

const LOG_LEVEL_ARGUMENT: &str = "log-level";
const LOG_LEVEL_DEFAULT_VALUE: &str = "info";

const IGNORE_LIST_ARGUMENT: &str = "ignore-list";
const IGNORE_LIST_SHORT_ARGUMENT: &str = "i";

const DETECT_302_REDIRECTS_ARGUMENT: &str = "redirect-302";

const EXCLUDE_HTTP: &str = "exclude-http";

const ERROR_EXIT_CODE: i32 = 1;

fn main() {
    let matches = App::new("Site Discovery Flea")
        .version("1.3.3")
        .author("Eugene Lebedev <duke.tougu@gmail.com>")
        .about("Discover site configs for nginx and apache. \
                                            Then generate urls and show output in \
                                            Zabbix Low Level Discovery format")
        .arg(
            Arg::with_name(WORK_DIR_ARGUMENT)
                .short(WORK_DIR_SHORT_ARGUMENT)
                .help("set working directory")
                .long(WORK_DIR_ARGUMENT)
                .takes_value(true).required(false)
        )
        .arg(
            Arg::with_name(INCLUDE_DOMAINS_WITH_WWW)
                .long(INCLUDE_DOMAINS_WITH_WWW)
                .help("include domains with www")
        )
        .arg(
            Arg::with_name(INCLUDE_CUSTOM_PORTS_OPTION)
                .long(INCLUDE_CUSTOM_PORTS_OPTION)
                .help("include domains with custom ports")
        )
        .arg(
            Arg::with_name(NGINX_VHOSTS_PATH_ARGUMENT)
                .short(NGINX_VHOSTS_PATH_SHORT_ARGUMENT)
                .help("set nginx vhosts root path")
                .long(NGINX_VHOSTS_PATH_ARGUMENT)
                .takes_value(true).required(false)
        )
        .arg(
            Arg::with_name(APACHE_VHOSTS_PATH_ARGUMENT)
                .short(APACHE_VHOSTS_PATH_SHORT_ARGUMENT)
                .help("set apache vhosts root path")
                .long(APACHE_VHOSTS_PATH_ARGUMENT)
                .takes_value(true).required(false)
        )
        .arg(
            Arg::with_name(USE_DATA_PROPERTY_ARGUMENT)
                .help("use low level discovery format with 'data' property. example: { \"data\": [] }")
                .long(USE_DATA_PROPERTY_ARGUMENT)
                .takes_value(false).required(false)
        )
        .arg(
            Arg::with_name(LOG_LEVEL_ARGUMENT)
                .help("set logging level. possible values: debug, info, error, warn, trace")
                .long(LOG_LEVEL_ARGUMENT)
                .case_insensitive(true)
                .takes_value(true).required(false)
                .default_value(LOG_LEVEL_DEFAULT_VALUE)
        )
        .arg(
            Arg::with_name(IGNORE_LIST_ARGUMENT)
                .short(IGNORE_LIST_SHORT_ARGUMENT)
                .help("set site ignore list")
                .long(IGNORE_LIST_ARGUMENT)
                .takes_value(true).required(false)
        )
        .arg(
            Arg::with_name(DETECT_302_REDIRECTS_ARGUMENT)
                .long(DETECT_302_REDIRECTS_ARGUMENT)
                .help("detect http code 302 as redirects")
        )
        .arg(
            Arg::with_name(EXCLUDE_HTTP)
                .long(EXCLUDE_HTTP)
                .help("exclude all http domains")
        )
        .get_matches();

    let working_directory: &Path = get_argument_path_value(
        &matches, WORK_DIR_ARGUMENT, WORK_DIR_SHORT_ARGUMENT, WORKDIR);

    debug!("working directory '{}'", &working_directory.display());

    env::set_current_dir(&working_directory).expect("unable to set working directory");

    let logging_level: &str = if matches.is_present(LOG_LEVEL_ARGUMENT) {
        matches.value_of(LOG_LEVEL_ARGUMENT).unwrap()
    } else { LOG_LEVEL_DEFAULT_VALUE };

    let logging_config = get_logging_config(logging_level);
    log4rs::init_config(logging_config).unwrap();

    let include_domains_with_www = matches.occurrences_of(INCLUDE_DOMAINS_WITH_WWW) > 0;
    let include_custom_domains = matches.occurrences_of(INCLUDE_CUSTOM_PORTS_OPTION) > 0;

    let ignore_list: Vec<&str> = if matches.is_present(IGNORE_LIST_ARGUMENT) {
        matches.value_of(IGNORE_LIST_ARGUMENT).unwrap().split(",").collect()
    } else { Vec::new() };

    let detect_302_redirects = matches.occurrences_of(DETECT_302_REDIRECTS_ARGUMENT) > 0;

    let exclude_http = matches.occurrences_of(EXCLUDE_HTTP) > 0;

    debug!("ignore list '{:?}'", &ignore_list);

    info!("[~] collect virtual hosts..");
    info!("- include domains with custom ports: {}", include_custom_domains);
    let mut vhosts: Vec<VirtualHost> = Vec::new();

    let nginx_vhosts_path: &Path = get_nginx_vhosts_path(&matches);
    debug!("- nginx vhosts root: '{}'", nginx_vhosts_path.display());

    let nginx_vhosts = get_nginx_vhosts(nginx_vhosts_path, detect_302_redirects);
    let mut filtered_nginx_vhosts: Vec<VirtualHost> = filter_vhosts(&nginx_vhosts, include_custom_domains, &ignore_list);
    vhosts.append(&mut filtered_nginx_vhosts);

    let apache_vhosts_path: &Path = get_apache_vhosts_path(&matches);
    debug!("apache vhosts root: '{}'", apache_vhosts_path.display());

    let apache_vhosts = get_apache_vhosts(apache_vhosts_path);
    let mut filtered_apache_vhosts: Vec<VirtualHost> = filter_vhosts(&apache_vhosts, include_custom_domains, &ignore_list);
    vhosts.append(&mut filtered_apache_vhosts);

    let sites: Vec<Site> = get_sites_from_vhosts(vhosts, include_domains_with_www, exclude_http);

    let json;

    if matches.is_present(USE_DATA_PROPERTY_ARGUMENT) {
        json = get_low_level_discovery_json_with_data_property(sites);
    } else {
        json = get_low_level_discovery_json(sites);
    };

    println!("{}", json);
}

fn get_argument_path_value<'a>(matches: &'a ArgMatches, long_argument: &str,
                               short_argument: &str, default_path: &'a str) -> &'a Path {
    let mut path: &Path = Path::new(default_path);

    if matches.is_present(long_argument) {
        let vhosts_path_value = matches.value_of(long_argument).unwrap_or(default_path);
        path = Path::new(vhosts_path_value)

    } else {
        if matches.is_present(short_argument) {
            let vhosts_path_value = matches.value_of(short_argument).unwrap_or(default_path);
            path = Path::new(vhosts_path_value)
        }
    }

    return path;
}

fn get_nginx_vhosts_path<'a>(matches: &'a ArgMatches) -> &'a Path {
    get_argument_path_value(&matches, NGINX_VHOSTS_PATH_ARGUMENT,
        NGINX_VHOSTS_PATH_SHORT_ARGUMENT, NGINX_VHOSTS_PATH)
}

fn get_apache_vhosts_path<'a>(matches: &'a ArgMatches) -> &'a Path {
    get_argument_path_value(&matches, APACHE_VHOSTS_PATH_ARGUMENT,
                            APACHE_VHOSTS_PATH_SHORT_ARGUMENT, APACHE_VHOSTS_PATH)
}

fn get_low_level_discovery_json(sites: Vec<Site>) -> String {
    let json_structure = json!(sites);
    let json = serde_json::to_string(&json_structure).unwrap();
    return json;
}

fn get_low_level_discovery_json_with_data_property(sites: Vec<Site>) -> String {
    let json_structure = json!({"data": sites});
    let json = serde_json::to_string(&json_structure).unwrap();
    return json;
}
