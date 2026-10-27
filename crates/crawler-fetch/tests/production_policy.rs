//! End to end: the production policy never connects to the mock site on 127.0.0.1.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use crawler_core::{CrawlerIdentity, Purpose};
use crawler_fetch::{
    ClientConfig, FetchError, FetchRequest, HttpFetcher, SafeHttpClient, ScriptedHostLookup,
};
use mock_site::MockSite;
use url::Url;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Returns a production client whose resolver sends `local.mock` to 127.0.0.1.
fn production_client() -> Result<SafeHttpClient, Box<dyn std::error::Error>> {
    let o_identity = CrawlerIdentity::new(
        "proofcrawl",
        "0.1.0",
        "https://crawler.proofcrawl.dev/contact",
        Purpose::AiTrain,
    )?;
    let o_lookup =
        ScriptedHostLookup::new().with("local.mock", vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]);
    Ok(SafeHttpClient::new(
        ClientConfig::new(o_identity).with_lookup(Arc::new(o_lookup)),
    )?)
}

#[tokio::test]
async fn test_production_policy_refuses_mock_site_without_connecting() -> TestResult {
    let o_site = MockSite::start().await?;
    let o_client = production_client()?;
    let str_literal = o_site.raw_url("127.0.0.1", "/");
    let str_named = o_site.raw_url("local.mock", "/");
    for str_url in [str_literal, str_named] {
        let res_o_resp = o_client
            .fetch(FetchRequest::new(Url::parse(&str_url)?))
            .await;
        let b_blocked = matches!(&res_o_resp, Err(FetchError::BlockedAddress { str_ip, .. }) if str_ip == "127.0.0.1");
        assert!(b_blocked, "{str_url}: {res_o_resp:?}");
    }
    assert_eq!(
        o_site.raw_connections(),
        0,
        "the production policy must not open a connection"
    );
    assert!(o_site.requests().is_empty());
    Ok(())
}
