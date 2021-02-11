pub mod filter {
    use crate::{DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT};
    use crate::domain::domain::VirtualHost;

    pub fn filter_vhosts(vhosts: &Vec<VirtualHost>, include_custom_domains: bool, ignore_list: &Vec<&str>) -> Vec<VirtualHost> {
        let mut results: Vec<VirtualHost> = Vec::new();

        for vhost in vhosts {
            if vhost_add_permitted(vhost, &results, include_custom_domains, &ignore_list) {
                debug!("+ add vhost '{}'", vhost.to_string());
                results.push(vhost.to_owned());
            }
        }

        return results
    }

    fn vhost_add_permitted(vhost: &VirtualHost, buffer: &Vec<VirtualHost>,
                           include_custom_ports: bool, ignore_list: &Vec<&str>) -> bool {
        let mut permitted = false;

        if vhost_not_in_ignore_list(&vhost.domain, &ignore_list) {
            if include_custom_ports {
                if !vec_contains_same_domain_with_port(buffer, &vhost.domain, vhost.port) {
                    permitted = true;
                }

            } else {
                if vhost_has_standard_port(vhost.port) {
                    if !vec_contains_same_domain_with_port(buffer, &vhost.domain, vhost.port) {
                        permitted = true;
                    }
                }
            }
        }

        permitted
    }

    fn vhost_not_in_ignore_list(domain: &String, ignore_list: &Vec<&str>) -> bool {
        !ignore_list.contains(&domain.as_str())
    }

    fn vhost_has_standard_port(port: i32) -> bool {
        port == DEFAULT_HTTP_PORT || port == DEFAULT_HTTPS_PORT
    }

    fn vec_contains_same_domain_with_port(vhosts: &Vec<VirtualHost>,
                                              domain: &String, port: i32) -> bool {
        vhosts.iter()
              .find(|vhost| &vhost.domain == domain && vhost.port == port).is_some()
    }
}
