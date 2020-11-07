#[cfg(test)]
mod site_tests {
    use crate::{DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT};
    use crate::domain::domain::VirtualHost;
    use crate::site::site::get_sites_from_vhosts;

    #[test]
    fn without_www_domains_result_should_not_contain_domains_with_www_lol() {
        let vhost1 = VirtualHost { domain: "cronbox.ru".to_string(), port: DEFAULT_HTTPS_PORT };
        let vhost2 = VirtualHost { domain: "www.google.com".to_string(), port: DEFAULT_HTTPS_PORT };
        let vhost3 = VirtualHost { domain: "tinyops.ru".to_string(), port: DEFAULT_HTTPS_PORT };

        let vhosts = vec![vhost1.clone(), vhost2.clone(), vhost3.clone()];

        let results = get_sites_from_vhosts(vhosts, false);

        assert_eq!(results.len(), 2);

        let site_with_www_found = results.iter().find(|site| site.url == "https://www.google.com");

        assert!(site_with_www_found.is_none())
    }

    #[test]
    fn with_www_domains_results_should_contain_domains_with_www() {
        let vhost1 = VirtualHost { domain: "cronbox.ru".to_string(), port: DEFAULT_HTTPS_PORT };
        let vhost2 = VirtualHost { domain: "www.google.com".to_string(), port: DEFAULT_HTTPS_PORT };
        let vhost3 = VirtualHost { domain: "tinyops.ru".to_string(), port: DEFAULT_HTTPS_PORT };

        let vhosts = vec![vhost1.clone(), vhost2.clone(), vhost3.clone()];

        let results = get_sites_from_vhosts(vhosts, true);

        assert_eq!(results.len(), 3);

        let site_with_www_found = results.iter().find(|site| site.url == "https://www.google.com");

        assert!(site_with_www_found.is_some())
    }

    #[test]
    fn vhost_with_https_port_should_contain_https_prefix_for_url() {
        let vhost1 = VirtualHost { domain: "dfov.ru".to_string(), port: DEFAULT_HTTPS_PORT };
        let vhosts = vec![vhost1.clone()];

        let results = get_sites_from_vhosts(vhosts, false);

        assert_eq!(results.len(), 1);

        let site_found = results.iter().find(|site| site.url == "https://dfov.ru");
        assert!(site_found.is_some())
    }

    #[test]
    fn vhost_with_standard_http_port_should_contain_http_prefix_for_url() {
        let vhost1 = VirtualHost { domain: "tinyops.ru".to_string(), port: DEFAULT_HTTP_PORT };
        let vhosts = vec![vhost1.clone()];

        let results = get_sites_from_vhosts(vhosts, false);

        assert_eq!(results.len(), 1);

        let site_found = results.iter().find(|site| site.url == "http://tinyops.ru");
        assert!(site_found.is_some())
    }

    #[test]
    fn vhost_with_non_standard_port_should_contain_http_prefix_for_url() {
        let domain = "cronbox.ru";
        let custom_port = 2345;
        let vhost1 = VirtualHost { domain: domain.to_string(), port: custom_port };
        let vhosts = vec![vhost1.clone()];

        let results = get_sites_from_vhosts(vhosts, false);

        assert_eq!(results.len(), 1);

        let expected_url = format!("http://{}:{}", domain, custom_port);

        let site_found = results.iter().find(|site| site.url == expected_url);
        assert!(site_found.is_some())
    }

    #[test]
    fn site_name_without_https_should_contain_http_postfix() {
        let vhost1 = VirtualHost { domain: "tinyops.ru".to_string(), port: DEFAULT_HTTP_PORT };
        let vhosts = vec![vhost1.clone()];

        let results = get_sites_from_vhosts(vhosts, false);

        assert_eq!(results.len(), 1);

        let site_found = results.iter().find(|site| site.name == "tinyops.ru_http");
        assert!(site_found.is_some())
    }
}