//! Mock-site scenarios for addresses and redirects (needs `--features test-policy`).
#![cfg(feature = "test-policy")]

#[path = "fetch_support/mock_client.rs"]
mod mock_client;

use crawler_fetch::{FetchError, HostScope};
use mock_client::{E_LOOPBACK, fetch, start};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Redirect paths on private-redirect.mock and the blocked address each one ends at.
const VEC_O_BLOCKED_REDIRECTS: [(&str, &str); 4] = [
    ("/to-literal", "0.0.0.0"),
    ("/to-name", "0.0.0.0"),
    ("/to-metadata", "169.254.169.254"),
    ("/to-mapped", "::ffff:0.0.0.0"),
];

#[tokio::test]
async fn test_ok_page_is_decoded_and_recorded() -> TestResult {
    let (o_site, o_client) = start().await?;
    let o_resp = fetch(&o_client, &o_site.url("ok.mock", "/"), HostScope::AnyHost).await??;
    assert_eq!(
        (o_resp.u_status, o_resp.header("content-encoding")),
        (200, Some("gzip"))
    );
    assert!(String::from_utf8_lossy(&o_resp.by_body).contains("Hello from the mock site."));
    assert!(o_resp.u_encoded_bytes > 0);
    assert_eq!(o_resp.vec_e_resolved_ips, vec![E_LOOPBACK]);
    assert_eq!(o_resp.header("x-robots-tag"), Some("noai"));
    Ok(())
}

#[tokio::test]
async fn test_redirects_to_blocked_addresses_never_connect() -> TestResult {
    let (o_site, o_client) = start().await?;
    for (str_path, str_ip) in VEC_O_BLOCKED_REDIRECTS {
        let str_url = o_site.url("private-redirect.mock", &format!("{str_path}?token=s3cret"));
        let res_o_resp = fetch(&o_client, &str_url, HostScope::AnyHost).await?;
        let b_blocked = matches!(&res_o_resp, Err(FetchError::BlockedAddress { str_ip: str_b, .. }) if str_b == str_ip);
        assert!(b_blocked, "{str_path}: {res_o_resp:?}");
        assert!(
            !format!("{res_o_resp:?}").contains("s3cret"),
            "query leaked"
        );
    }
    assert_eq!(
        o_site.count_path("/landed"),
        0,
        "a blocked address was connected to"
    );
    Ok(())
}

#[tokio::test]
async fn test_dns_rebinding_second_answer_is_blocked() -> TestResult {
    let (o_site, o_client) = start().await?;
    let res_o_resp = fetch(
        &o_client,
        &o_site.url("rebind.mock", "/"),
        HostScope::AnyHost,
    )
    .await?;
    let b_blocked = matches!(&res_o_resp, Err(FetchError::BlockedAddress { str_host, .. }) if str_host == "rebind.mock");
    assert!(b_blocked, "{res_o_resp:?}");
    assert_eq!((o_site.count_path("/"), o_site.count_path("/next")), (1, 0));
    Ok(())
}

#[tokio::test]
async fn test_redirect_hops_are_capped() -> TestResult {
    let (o_site, o_client) = start().await?;
    let res_o_hops = fetch(
        &o_client,
        &o_site.url("hops.mock", "/r"),
        HostScope::AnyHost,
    )
    .await?;
    assert_eq!(res_o_hops, Err(FetchError::TooManyRedirects { u_max: 5 }));
    let u_requests = o_site
        .requests()
        .iter()
        .filter(|o_r| o_r.str_host == "hops.mock")
        .count();
    assert_eq!(u_requests, 6, "the first request plus 5 followed redirects");
    Ok(())
}

#[tokio::test]
async fn test_redirect_scope_and_hop_recording() -> TestResult {
    let (o_site, o_client) = start().await?;
    let res_o_scoped = fetch(
        &o_client,
        &o_site.url("scope.mock", "/"),
        HostScope::SameHost,
    )
    .await?;
    assert!(
        matches!(res_o_scoped, Err(FetchError::RedirectOutOfScope { .. })),
        "{res_o_scoped:?}"
    );
    let o_resp = fetch(
        &o_client,
        &o_site.url("scope.mock", "/"),
        HostScope::AnyHost,
    )
    .await??;
    assert_eq!(o_resp.vec_o_redirects.len(), 1);
    assert_eq!(o_resp.vec_o_redirects[0].u_status, 302);
    assert!(o_resp.vec_o_redirects[0].str_url.contains("scope.mock"));
    assert!(o_resp.str_final_url.contains("ok.mock"));
    Ok(())
}
