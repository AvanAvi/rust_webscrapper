//! Exercises the actual reqwest redirect policy and platform TLS verifier over local sockets.
#![cfg(feature = "test-policy")]

#[path = "tls_support/certificates.rs"]
mod certificates;
#[path = "tls_support/harness.rs"]
mod harness;
#[path = "tls_support/tls_site.rs"]
mod tls_site;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use certificates::{BoxError, Certificates};
use crawler_core::{
    MemoryBodyArchive, Outcome, PolicyEvaluator, PolicyRule, Purpose, SequentialIdSource,
    SignalFailure, SignalSet, SignalStatus, SystemClock,
};
use crawler_fetch::{CollectorConfig, FetchError, FetchRequest, HttpFetcher, SignalCollector};
use harness::{certificate_error, client};
use rustls::CertificateError;
use tls_site::TlsSite;

type TestResult = Result<(), BoxError>;

#[tokio::test]
async fn test_https_to_http_redirect_is_refused_without_plaintext_request() -> TestResult {
    let o_roots = Certificates::new()?;
    let o_site = TlsSite::start(o_roots.server("tls.mock")?).await?;
    let o_client = client(&o_roots)?;
    let err_fetch = o_client
        .fetch(FetchRequest::new(o_site.url("tls.mock", "/down")?))
        .await
        .err()
        .ok_or("downgrade must fail")?;
    assert!(
        matches!(err_fetch, FetchError::RedirectDowngrade { .. }),
        "{err_fetch:?}"
    );
    assert_eq!(o_site.o_requests.load(Ordering::SeqCst), 1);
    assert!(
        o_site.o_plain.requests().is_empty(),
        "plaintext request escaped"
    );
    Ok(())
}

#[tokio::test]
async fn test_https_to_https_redirect_is_followed() -> TestResult {
    let o_roots = Certificates::new()?;
    let o_site = TlsSite::start(o_roots.server("tls.mock")?).await?;
    let o_response = client(&o_roots)?
        .fetch(FetchRequest::new(o_site.url("tls.mock", "/up")?))
        .await?;
    assert_eq!(o_response.u_status, 200);
    assert_eq!(o_response.by_body, b"ok");
    assert_eq!(
        o_response.str_final_url,
        o_site.url("tls.mock", "/landed")?.as_str()
    );
    assert_eq!(o_response.vec_o_redirects.len(), 1);
    assert_eq!(o_response.vec_o_redirects[0].u_status, 302);
    assert_eq!(
        o_response.vec_o_redirects[0].str_url,
        o_site.url("tls.mock", "/up")?.as_str()
    );
    assert_eq!(o_site.o_requests.load(Ordering::SeqCst), 2);
    assert!(o_site.o_plain.requests().is_empty());
    Ok(())
}

#[tokio::test]
async fn test_untrusted_certificate_is_refused() -> TestResult {
    let o_trusted = Certificates::new()?;
    let o_other = Certificates::new()?;
    let o_site = TlsSite::start(o_other.server("tls.mock")?).await?;
    let o_client = client(&o_trusted)?;
    let err_fetch = o_client
        .fetch(FetchRequest::new(o_site.url("tls.mock", "/landed")?))
        .await
        .err()
        .ok_or("different root must fail")?;
    assert!(
        matches!(err_fetch, FetchError::Connect { .. }),
        "{err_fetch:?}"
    );
    assert_eq!(
        certificate_error(&o_client)?,
        CertificateError::UnknownIssuer
    );
    assert_eq!(o_site.o_requests.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn test_hostname_mismatch_is_refused() -> TestResult {
    let o_roots = Certificates::new()?;
    let o_site = TlsSite::start(o_roots.server("tls.mock")?).await?;
    let o_client = client(&o_roots)?;
    let err_fetch = o_client
        .fetch(FetchRequest::new(o_site.url("wrong.mock", "/landed")?))
        .await
        .err()
        .ok_or("wrong name must fail")?;
    assert!(
        matches!(err_fetch, FetchError::Connect { .. }),
        "{err_fetch:?}"
    );
    assert_name_error(&o_client)?;
    assert_eq!(o_site.o_requests.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn test_safe_http_client_keeps_hostname_verification_enabled() -> TestResult {
    let o_roots = Certificates::new()?;
    let o_site = TlsSite::start(o_roots.server("tls.mock")?).await?;
    let o_client = client(&o_roots)?;
    let o_response = o_client
        .fetch(FetchRequest::new(o_site.url("tls.mock", "/landed")?))
        .await?;
    assert_eq!(
        o_response.u_status, 200,
        "matching certificate must work first"
    );
    assert!(
        o_client
            .fetch(FetchRequest::new(o_site.url("wrong.mock", "/landed")?))
            .await
            .is_err()
    );
    assert_name_error(&o_client)?;
    assert_eq!(o_site.o_requests.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn test_tls_error_text_has_no_query_or_credentials() -> TestResult {
    let o_roots = Certificates::new()?;
    let o_other = Certificates::new()?;
    let o_site = TlsSite::start(o_other.server("tls.mock")?).await?;
    let o_client = client(&o_roots)?;
    let err_fetch = failed_fetch(
        &o_client,
        o_site.url("tls.mock", "/landed?private_query=secret_query")?,
    )
    .await?;
    assert_eq!(
        certificate_error(&o_client)?,
        CertificateError::UnknownIssuer
    );
    assert_redacted(&err_fetch);
    check_name_and_redirect_redaction(&o_other, &o_site).await?;
    let o_client = client(&o_other)?;
    let url_credentials = credentials_url(&o_site)?;
    let err_fetch = failed_fetch(&o_client, url_credentials).await?;
    assert_eq!(err_fetch, FetchError::CredentialsInUrl);
    assert_redacted(&err_fetch);
    Ok(())
}

async fn check_name_and_redirect_redaction(o_roots: &Certificates, o_site: &TlsSite) -> TestResult {
    let o_client = client(o_roots)?;
    let err_fetch = failed_fetch(
        &o_client,
        o_site.url("wrong.mock", "/landed?private_query=secret_query")?,
    )
    .await?;
    assert_name_error(&o_client)?;
    assert_redacted(&err_fetch);
    let err_fetch = failed_fetch(&o_client, o_site.url("tls.mock", "/down")?).await?;
    assert!(matches!(err_fetch, FetchError::RedirectDowngrade { .. }));
    assert_redacted(&err_fetch);
    Ok(())
}

async fn failed_fetch(
    o_client: &crawler_fetch::SafeHttpClient,
    url_target: url::Url,
) -> Result<FetchError, BoxError> {
    o_client
        .fetch(FetchRequest::new(url_target))
        .await
        .err()
        .ok_or_else(|| "expected fetch failure".into())
}

fn credentials_url(o_site: &TlsSite) -> Result<url::Url, BoxError> {
    let mut url_credentials = o_site.url("tls.mock", "/landed?private_query=secret_query")?;
    url_credentials
        .set_username("secret_user")
        .map_err(|()| "username")?;
    url_credentials
        .set_password(Some("secret_password"))
        .map_err(|()| "password")?;
    Ok(url_credentials)
}

fn assert_name_error(o_client: &crawler_fetch::SafeHttpClient) -> TestResult {
    let e_certificate = certificate_error(o_client)?;
    assert!(
        matches!(
            e_certificate,
            CertificateError::NotValidForName | CertificateError::NotValidForNameContext { .. }
        ),
        "{e_certificate:?}"
    );
    Ok(())
}

fn assert_redacted(err_fetch: &FetchError) {
    let str_text = format!("{err_fetch} {err_fetch:?}");
    for str_secret in [
        "private_query",
        "secret_query",
        "secret_user",
        "secret_password",
        "?",
    ] {
        assert!(!str_text.contains(str_secret), "secret leaked");
    }
}

#[tokio::test]
async fn test_downgraded_robots_redirect_denies() -> TestResult {
    let o_roots = Certificates::new()?;
    let o_site = TlsSite::start(o_roots.server("tls.mock")?).await?;
    let o_collector = SignalCollector::with_ids(
        Arc::new(client(&o_roots)?),
        Arc::new(MemoryBodyArchive::new()),
        (
            Arc::new(SystemClock::new()),
            Arc::new(SequentialIdSource::new(1)),
        ),
        CollectorConfig::new("proofcrawl"),
    );
    let vec_o_observations = o_collector
        .collect(&o_site.url("tls.mock", "/page")?)
        .await?;
    let o_robots = &vec_o_observations[0];
    assert_eq!(o_robots.e_status, SignalStatus::Unreachable);
    assert_eq!(o_robots.opt_e_failure, Some(SignalFailure::RedirectRefused));
    let o_decision = PolicyEvaluator::for_purpose(Purpose::AiTrain)
        .evaluate(&SignalSet::new(vec_o_observations));
    assert_eq!(o_decision.outcome(), Outcome::Deny);
    assert_eq!(o_decision.rule(), PolicyRule::RobotsUnreachable);
    assert!(o_site.o_plain.requests().is_empty());
    Ok(())
}
