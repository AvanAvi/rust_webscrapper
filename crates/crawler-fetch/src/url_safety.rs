//! Validates a URL before any connection: scheme, credentials, host, and IP-literal addresses.

use std::net::IpAddr;
use std::sync::Arc;

use url::{Host, Url};

use crate::address_policy::AddressPolicy;
use crate::fetch_error::FetchError;

/// Checks URLs; IP-literal hosts are checked here because they never reach the resolver.
#[derive(Debug, Clone)]
pub struct UrlSafety {
    /// Policy applied to IP-literal hosts.
    o_policy: Arc<dyn AddressPolicy>,
}

impl UrlSafety {
    /// Creates a validator for a policy.
    pub fn new(o_policy: Arc<dyn AddressPolicy>) -> Self {
        Self { o_policy }
    }

    /// Returns `Ok` when the URL may be requested.
    pub fn check(&self, url_target: &Url) -> Result<(), FetchError> {
        if !matches!(url_target.scheme(), "http" | "https") {
            return Err(FetchError::UnsupportedScheme {
                str_scheme: url_target.scheme().to_owned(),
            });
        }
        if !url_target.username().is_empty() || url_target.password().is_some() {
            return Err(FetchError::CredentialsInUrl);
        }
        match url_target.host() {
            None => Err(FetchError::InvalidUrl {
                str_reason: String::from("missing host"),
            }),
            Some(Host::Domain(_)) => Ok(()),
            Some(Host::Ipv4(o_v4)) => self.check_ip(IpAddr::V4(o_v4)),
            Some(Host::Ipv6(o_v6)) => self.check_ip(IpAddr::V6(o_v6)),
        }
    }

    /// Returns a URL without user info, query, or fragment, for errors and logs.
    pub fn redact(&self, url_target: &Url) -> String {
        let str_host = url_target.host_str().unwrap_or("?");
        let str_port = url_target
            .port()
            .map_or_else(String::new, |u_port| format!(":{u_port}"));
        format!(
            "{}://{str_host}{str_port}{}",
            url_target.scheme(),
            url_target.path()
        )
    }

    /// Checks one literal address against the policy.
    fn check_ip(&self, e_ip: IpAddr) -> Result<(), FetchError> {
        self.o_policy
            .check(e_ip)
            .map_err(|o_rejection| FetchError::BlockedAddress {
                str_host: e_ip.to_string(),
                str_ip: e_ip.to_string(),
                str_range: o_rejection.str_range.to_owned(),
            })
    }
}
