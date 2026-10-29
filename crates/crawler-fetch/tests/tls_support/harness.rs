//! Builds the real SafeHttpClient and extracts typed certificate errors from its test diagnostic.

use std::error::Error;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

use crawler_core::{CrawlerIdentity, Purpose};
use crawler_fetch::{ClientConfig, FetchLimits, SafeHttpClient, ScriptedHostLookup};
use rustls::CertificateError;

use super::certificates::{BoxError, Certificates};

pub fn client(o_roots: &Certificates) -> Result<SafeHttpClient, BoxError> {
    let o_identity = CrawlerIdentity::new(
        "proofcrawl",
        "https://crawler.example/contact",
        Purpose::AiTrain,
    )?;
    let o_lookup = ScriptedHostLookup::new()
        .with("tls.mock", vec![IpAddr::V4(Ipv4Addr::LOCALHOST)])
        .with("wrong.mock", vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]);
    let o_limits = FetchLimits {
        dur_connect: Duration::from_secs(2),
        dur_read: Duration::from_secs(2),
        dur_total: Duration::from_secs(5),
        ..FetchLimits::default()
    };
    Ok(SafeHttpClient::new(
        ClientConfig::new(o_identity)
            .with_test_policy()
            .with_test_root(o_roots.root()?)
            .with_lookup(Arc::new(o_lookup))
            .with_limits(o_limits),
    )?)
}

pub fn certificate_error(o_client: &SafeHttpClient) -> Result<CertificateError, BoxError> {
    let err_transport = o_client
        .take_test_transport_error()
        .ok_or("transport diagnostic")?;
    certificate_kind(&err_transport)
}

fn certificate_kind(err_transport: &(dyn Error + 'static)) -> Result<CertificateError, BoxError> {
    let mut opt_err_source: Option<&(dyn Error + 'static)> = Some(err_transport);
    while let Some(err_source) = opt_err_source {
        if let Some(rustls::Error::InvalidCertificate(e_certificate)) = err_source.downcast_ref() {
            return Ok(e_certificate.clone());
        }
        opt_err_source = nested_source(err_source);
    }
    Err("expected typed rustls certificate error".into())
}

fn nested_source<'a>(err_source: &'a (dyn Error + 'static)) -> Option<&'a (dyn Error + 'static)> {
    if let Some(err_io) = err_source.downcast_ref::<std::io::Error>() {
        return err_io
            .get_ref()
            .map(|err_inner| err_inner as &(dyn Error + 'static));
    }
    err_source.source()
}

#[test]
fn test_certificate_diagnostic_preserves_io_wrapped_kind() -> Result<(), BoxError> {
    let err_io = std::io::Error::other(rustls::Error::InvalidCertificate(
        CertificateError::UnknownIssuer,
    ));
    assert_eq!(certificate_kind(&err_io)?, CertificateError::UnknownIssuer);
    Ok(())
}
