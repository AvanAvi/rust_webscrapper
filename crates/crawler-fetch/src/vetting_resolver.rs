//! Resolves each host once, filters the answer through the address policy, and hands only
//! vetted addresses to the connector, which never performs a lookup of its own.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use crate::address_policy::AddressPolicy;
use crate::fetch_error::FetchError;
use crate::fetch_log::FetchLog;
use crate::host_lookup::HostLookup;

/// reqwest resolver that enforces the address policy on every resolution.
#[derive(Debug, Clone)]
pub struct VettingResolver {
    /// Policy deciding which addresses may be used.
    o_policy: Arc<dyn AddressPolicy>,
    /// Underlying name lookup.
    o_lookup: Arc<dyn HostLookup>,
    /// Per-fetch log receiving vetted addresses and refusals.
    o_log: Arc<FetchLog>,
}

impl VettingResolver {
    /// Creates a resolver for one fetch.
    pub fn new(
        o_policy: Arc<dyn AddressPolicy>,
        o_lookup: Arc<dyn HostLookup>,
        o_log: Arc<FetchLog>,
    ) -> Self {
        Self {
            o_policy,
            o_lookup,
            o_log,
        }
    }

    /// Looks a host up and returns the vetted addresses, or a refusal.
    async fn vet(self, str_host: String) -> Result<Vec<IpAddr>, FetchError> {
        let vec_e_answer = self
            .o_lookup
            .lookup(&str_host)
            .await
            .map_err(|str_reason| FetchError::DnsFailure {
                str_host: str_host.clone(),
                str_reason,
            })?;
        self.filter(str_host, &vec_e_answer)
    }

    /// Refuses the host if any answer is blocked (a mixed answer suggests rebinding), else returns all.
    fn filter(&self, str_host: String, vec_e_answer: &[IpAddr]) -> Result<Vec<IpAddr>, FetchError> {
        let opt_o_blocked = vec_e_answer
            .iter()
            .find_map(|e_ip| self.o_policy.check(*e_ip).err());
        if let Some(o_rejection) = opt_o_blocked {
            let str_ip = o_rejection.e_ip.to_string();
            return Err(FetchError::BlockedAddress {
                str_host,
                str_ip,
                str_range: o_rejection.str_range.to_owned(),
            });
        }
        if vec_e_answer.is_empty() {
            return Err(FetchError::DnsFailure {
                str_host,
                str_reason: String::from("no addresses"),
            });
        }
        Ok(vec_e_answer.to_vec())
    }
}

impl Resolve for VettingResolver {
    /// Resolves, vets, and records; any blocked address in the answer refuses the host.
    fn resolve(&self, o_name: Name) -> Resolving {
        let o_resolver = self.clone();
        let str_host = o_name.as_str().to_owned();
        Box::pin(async move {
            let o_log = Arc::clone(&o_resolver.o_log);
            match o_resolver.vet(str_host).await {
                Ok(vec_e_ips) => {
                    o_log.record_ips(&vec_e_ips);
                    let o_addrs: Addrs =
                        Box::new(vec_e_ips.into_iter().map(|e_ip| SocketAddr::new(e_ip, 0)));
                    Ok(o_addrs)
                }
                Err(err_refusal) => {
                    o_log.refuse(err_refusal.clone());
                    Err(err_refusal.into())
                }
            }
        })
    }
}
