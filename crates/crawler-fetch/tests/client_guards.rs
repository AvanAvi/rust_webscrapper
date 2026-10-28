//! Client construction guards, per-hop redirect checks, and the fake fetcher (default build).

use std::sync::Arc;

use crawler_core::{CrawlerIdentity, Hasher, Purpose, Timestamp};
use crawler_fetch::{
    ClientConfig, FakeFetcher, FetchError, FetchLog, FetchRequest, FetchResponse, HostScope,
    HttpFetcher, ProductionAddressPolicy, RedirectGuard, SafeHttpClient, UrlSafety,
};
use url::Url;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Returns an identity with the given contact URL.
fn identity(str_contact: &str) -> Result<CrawlerIdentity, crawler_core::CoreError> {
    CrawlerIdentity::new("proofcrawl", "0.1.0", str_contact, Purpose::AiTrain)
}

/// Returns a redirect guard over the production policy.
fn guard(e_scope: HostScope) -> RedirectGuard {
    let o_safety = UrlSafety::new(Arc::new(ProductionAddressPolicy::new()));
    RedirectGuard::new(o_safety, e_scope, 5, Arc::new(FetchLog::new()))
}

#[cfg(not(feature = "test-policy"))]
#[test]
fn test_default_build_cannot_construct_the_test_policy() -> TestResult {
    use crawler_fetch::PolicyChoice;
    assert!(!PolicyChoice::is_test_policy_compiled());
    assert_eq!(
        PolicyChoice::LoopbackForTests.build().err(),
        Some(FetchError::TestPolicyUnavailable)
    );
    let o_config =
        ClientConfig::new(identity("https://crawler.proofcrawl.dev/contact")?).with_test_policy();
    assert_eq!(
        SafeHttpClient::new(o_config).err(),
        Some(FetchError::TestPolicyUnavailable)
    );
    Ok(())
}

#[cfg(not(feature = "test-policy"))]
#[test]
fn test_default_build_refuses_placeholder_contact() -> TestResult {
    for str_contact in [
        "https://crawler.example/contact",
        "https://example.com/bot",
        "https://www.example.org/",
    ] {
        let res_client = SafeHttpClient::new(ClientConfig::new(identity(str_contact)?));
        assert!(
            matches!(res_client, Err(FetchError::PlaceholderContact { .. })),
            "{str_contact}"
        );
    }
    Ok(())
}

#[test]
fn test_production_client_builds_with_real_contact() -> TestResult {
    let o_client = SafeHttpClient::new(ClientConfig::new(identity(
        "https://crawler.proofcrawl.dev/contact",
    )?))?;
    assert!(
        o_client
            .check_url(&Url::parse("http://10.0.0.1/")?)
            .is_err()
    );
    Ok(())
}

#[test]
fn test_https_to_http_downgrade_is_refused_and_upgrade_allowed() -> TestResult {
    let (url_https, url_http) = (
        Url::parse("https://a.org/x")?,
        Url::parse("http://a.org/y")?,
    );
    let o_any = guard(HostScope::AnyHost);
    let res_down = o_any.check_hop(&url_https, &url_https, &url_http, 1);
    assert!(
        matches!(res_down, Err(FetchError::RedirectDowngrade { .. })),
        "{res_down:?}"
    );
    assert_eq!(o_any.check_hop(&url_http, &url_http, &url_https, 1), Ok(()));
    Ok(())
}

#[test]
fn test_hop_count_is_rechecked() -> TestResult {
    let url_http = Url::parse("http://a.org/y")?;
    let res_hop = guard(HostScope::AnyHost).check_hop(&url_http, &url_http, &url_http, 6);
    assert_eq!(res_hop, Err(FetchError::TooManyRedirects { u_max: 5 }));
    Ok(())
}

#[test]
fn test_hop_address_and_scope_are_rechecked() -> TestResult {
    let url_http = Url::parse("http://a.org/y")?;
    let url_meta = Url::parse("http://169.254.169.254/latest")?;
    let res_meta = guard(HostScope::AnyHost).check_hop(&url_http, &url_http, &url_meta, 1);
    assert!(
        matches!(res_meta, Err(FetchError::BlockedAddress { .. })),
        "{res_meta:?}"
    );
    let url_other = Url::parse("http://b.org/")?;
    let res_scope = guard(HostScope::SameHost).check_hop(&url_http, &url_http, &url_other, 1);
    assert!(
        matches!(res_scope, Err(FetchError::RedirectOutOfScope { .. })),
        "{res_scope:?}"
    );
    Ok(())
}

#[test]
fn test_error_text_never_contains_query_or_credentials() -> TestResult {
    let url_from = Url::parse("https://a.org/x")?;
    let url_to = Url::parse("http://user:pw@b.org/path?token=s3cret#frag")?;
    let err_hop = guard(HostScope::AnyHost)
        .check_hop(&url_from, &url_from, &url_to, 1)
        .err()
        .ok_or("expected refusal")?;
    let str_text = format!("{err_hop} {err_hop:?}");
    for str_secret in ["s3cret", "token", "user", "pw@", "frag"] {
        assert!(
            !str_text.contains(str_secret),
            "leaked {str_secret}: {str_text}"
        );
    }
    Ok(())
}

/// Returns a small canned response.
fn canned() -> Result<FetchResponse, Box<dyn std::error::Error>> {
    let dt_now = Timestamp::parse("2026-10-27T00:00:00.000000Z")?;
    Ok(FetchResponse {
        u_status: 200,
        str_final_url: String::from("https://a.org/"),
        vec_o_headers: vec![(String::from("content-type"), String::from("text/html"))],
        by_body: b"hi".to_vec(),
        u_encoded_bytes: 2,
        vec_o_redirects: Vec::new(),
        vec_e_resolved_ips: Vec::new(),
        b_truncated: false,
        opt_dur_retry_after: None,
        dt_started_at: dt_now.clone(),
        dt_finished_at: dt_now,
    })
}

#[tokio::test]
async fn test_fake_fetcher_returns_canned_results() -> TestResult {
    let o_canned = canned()?;
    let o_fake = FakeFetcher::new().with("https://a.org/", Ok(o_canned.clone()));
    assert_eq!(
        o_fake
            .fetch(FetchRequest::new(Url::parse("https://a.org/")?))
            .await,
        Ok(o_canned.clone())
    );
    assert!(
        o_fake
            .fetch(FetchRequest::new(Url::parse("https://b.org/")?))
            .await
            .is_err()
    );
    let o_record = o_canned.to_record(&Hasher::new());
    assert_eq!(
        (o_record.u_bytes, o_record.opt_str_content_type.as_deref()),
        (2, Some("text/html"))
    );
    Ok(())
}
