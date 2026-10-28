//! End to end: mock-site signal files through the collector and the policy evaluator
//! (needs `--features test-policy`).
#![cfg(feature = "test-policy")]

#[path = "fetch_support/collector_setup.rs"]
mod collector_setup;

use collector_setup::{BoxError, Harness, harness};
use crawler_core::{
    Decision, Outcome, ParsedSignal, PolicyEvaluator, PolicyRule, Purpose, SignalKind,
    SignalObservation, SignalSet,
};
use crawler_fetch::{FetchRequest, HttpFetcher};
use url::Url;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Decision plus the observations it was made from.
type Decided = (Decision, Vec<SignalObservation>);

/// Collects the signals of a page and evaluates them for a purpose.
async fn decide(o_h: &Harness, url_page: Url, e_purpose: Purpose) -> Result<Decided, BoxError> {
    let vec_o_obs = o_h.o_collector.collect(&url_page).await?;
    let o_set = SignalSet::new(vec_o_obs.clone());
    Ok((
        PolicyEvaluator::for_purpose(e_purpose).evaluate(&o_set),
        vec_o_obs,
    ))
}

/// Checks outcome, rule, and that reason `u_reason` contains `str_phrase`.
fn expect(o_d: &Decision, o_want: (Outcome, PolicyRule), u_reason: usize, str_phrase: &str) {
    assert_eq!((o_d.outcome(), o_d.rule()), o_want, "{:?}", o_d.reasons());
    assert!(
        o_d.reasons()[u_reason].contains(str_phrase),
        "{:?}",
        o_d.reasons()
    );
}

/// Deny under a rule.
fn deny(e_rule: PolicyRule) -> (Outcome, PolicyRule) {
    (Outcome::Deny, e_rule)
}

/// Allow with no reservation.
const O_ALLOW: (Outcome, PolicyRule) = (Outcome::Allow, PolicyRule::NoReservation);

#[tokio::test]
async fn test_robots_200_disallow_and_content_signal() -> TestResult {
    let o_h = harness(100).await?;
    let (o_private, _) =
        decide(&o_h, o_h.url("sig-ok.mock", "/private/a")?, Purpose::Search).await?;
    expect(
        &o_private,
        deny(PolicyRule::RobotsDisallow),
        0,
        "(rule \"Disallow: /private/\")",
    );
    let (o_train, _) = decide(&o_h, o_h.url("sig-ok.mock", "/page")?, Purpose::AiTrain).await?;
    expect(
        &o_train,
        deny(PolicyRule::ContentSignalNo),
        0,
        "sets Content-Signal ai-train=no",
    );
    let (o_search, _) = decide(&o_h, o_h.url("sig-ok.mock", "/page")?, Purpose::Search).await?;
    expect(
        &o_search,
        O_ALLOW,
        0,
        "states no Content-Signal objection to search indexing",
    );
    Ok(())
}

#[tokio::test]
async fn test_robots_404_allows() -> TestResult {
    let o_h = harness(100).await?;
    let (o_404, _) = decide(&o_h, o_h.url("sig-404.mock", "/x")?, Purpose::AiTrain).await?;
    expect(
        &o_404,
        O_ALLOW,
        0,
        "was not found (HTTP 404), so no robots.txt rules apply.",
    );
    Ok(())
}

#[tokio::test]
async fn test_robots_500_and_timeout_deny() -> TestResult {
    let o_h = harness(100).await?;
    let (o_500, _) = decide(&o_h, o_h.url("sig-500.mock", "/x")?, Purpose::Search).await?;
    expect(
        &o_500,
        deny(PolicyRule::RobotsUnreachable),
        0,
        "could not be retrieved (HTTP 500)",
    );
    let (o_slow, _) = decide(
        &o_h,
        o_h.raw_url("sig-timeout.mock", "/x")?,
        Purpose::Search,
    )
    .await?;
    expect(
        &o_slow,
        deny(PolicyRule::RobotsUnreachable),
        0,
        "(no HTTP response: timed out)",
    );
    Ok(())
}

#[tokio::test]
async fn test_cross_host_redirect_applies_target_rules_to_the_initial_host() -> TestResult {
    let o_h = harness(100).await?;
    let url_page = o_h.url("sig-redirect.mock", "/from-target/x")?;
    let (o_decision, vec_o_obs) = decide(&o_h, url_page, Purpose::Search).await?;
    expect(
        &o_decision,
        deny(PolicyRule::RobotsDisallow),
        0,
        "Disallow: /from-target/",
    );
    let vec_o_hops = &vec_o_obs[0].vec_o_redirects;
    assert_eq!((vec_o_hops.len(), vec_o_hops[0].u_status), (1, 302));
    assert!(vec_o_hops[0].str_url.contains("sig-redirect.mock"));
    let opt_str_final = vec_o_obs[0].opt_str_final_url.as_deref();
    assert!(opt_str_final.is_some_and(|str_u| str_u.contains("sig-target.mock")));
    assert_eq!(o_h.count("sig-target.mock", "/robots.txt"), 1);
    Ok(())
}

#[tokio::test]
async fn test_oversized_robots_is_truncated_and_late_rules_ignored() -> TestResult {
    let o_h = harness(100).await?;
    let (o_early, vec_o_obs) =
        decide(&o_h, o_h.url("sig-big.mock", "/early/x")?, Purpose::Search).await?;
    expect(
        &o_early,
        deny(PolicyRule::RobotsDisallow),
        0,
        "Disallow: /early/",
    );
    let (o_late, _) = decide(&o_h, o_h.url("sig-big.mock", "/late/x")?, Purpose::Search).await?;
    expect(&o_late, O_ALLOW, 0, "allows this URL");
    assert!(
        vec_o_obs[0]
            .opt_o_excerpt
            .as_ref()
            .is_some_and(|o_e| o_e.is_truncated())
    );
    let Some(ParsedSignal::RobotsTxt(o_found)) = &vec_o_obs[0].opt_e_parsed else {
        return Err("robots finding expected".into());
    };
    assert!(o_found.b_truncated, "the parser saw a cut file");
    Ok(())
}

#[tokio::test]
async fn test_ai_txt_present_reserves_and_absent_does_not() -> TestResult {
    let o_h = harness(100).await?;
    let (o_present, _) = decide(&o_h, o_h.url("sig-ai.mock", "/about")?, Purpose::AiTrain).await?;
    expect(
        &o_present,
        deny(PolicyRule::AiTxtDisallow),
        1,
        "assuming the page is .html",
    );
    let (o_absent, _) = decide(&o_h, o_h.url("sig-404.mock", "/about")?, Purpose::AiTrain).await?;
    expect(
        &o_absent,
        O_ALLOW,
        1,
        "a missing signal is never treated as a reservation",
    );
    Ok(())
}

#[tokio::test]
async fn test_tdmrep_valid_reserves() -> TestResult {
    let o_h = harness(100).await?;
    let (o_valid, _) = decide(
        &o_h,
        o_h.url("sig-ok.mock", "/reserved/x")?,
        Purpose::AiTrain,
    )
    .await?;
    assert!(
        o_valid.reasons()[2].contains("reserves text-and-data-mining rights (tdm-reservation: 1)")
    );
    let b_tdm_denies = o_valid
        .verdicts()
        .iter()
        .any(|o_v| o_v.denying_rule() == Some(PolicyRule::TdmrepReserved));
    assert!(b_tdm_denies, "{:?}", o_valid.reasons());
    Ok(())
}

#[tokio::test]
async fn test_tdmrep_invalid_and_float_are_protocol_errors() -> TestResult {
    let o_h = harness(100).await?;
    for str_host in ["sig-tdm-invalid.mock", "sig-tdm-float.mock"] {
        let (o_decision, vec_o_obs) =
            decide(&o_h, o_h.url(str_host, "/x")?, Purpose::AiTrain).await?;
        expect(&o_decision, O_ALLOW, 2, "does not reserve this URL");
        let Some(ParsedSignal::Tdmrep(o_found)) = &vec_o_obs[2].opt_e_parsed else {
            return Err("tdmrep finding expected".into());
        };
        assert_eq!(
            (o_found.opt_b_reserved, o_found.u_protocol_errors),
            (None, 1),
            "{str_host}"
        );
    }
    Ok(())
}

/// Returns the X-Robots-Tag finding among page observations.
fn robots_tag(vec_o_page: &[SignalObservation]) -> Result<Vec<String>, BoxError> {
    let o_tag = vec_o_page
        .iter()
        .find(|o_o| o_o.e_kind == SignalKind::XRobotsTag)
        .ok_or("observation")?;
    match &o_tag.opt_e_parsed {
        Some(ParsedSignal::RobotsTag(o_found)) => Ok(o_found.vec_str_directives.clone()),
        _ => Err("robots tag finding expected".into()),
    }
}

#[tokio::test]
async fn test_page_level_signals_are_merged_and_decide() -> TestResult {
    let o_h = harness(100).await?;
    let o_page = o_h
        .o_client
        .fetch(FetchRequest::new(o_h.url("sig-ok.mock", "/page")?))
        .await?;
    let vec_o_page = o_h.o_collector.page_observations(&o_page)?;
    assert_eq!(vec_o_page.len(), 4);
    assert_eq!(
        robots_tag(&vec_o_page)?,
        ["noai", "noindex"],
        "two headers merged"
    );
    let o_decision =
        PolicyEvaluator::for_purpose(Purpose::AiTrain).evaluate(&SignalSet::new(vec_o_page));
    expect(
        &o_decision,
        deny(PolicyRule::RobotsTagNoai),
        0,
        "contains \"noai\", which denies AI training",
    );
    Ok(())
}

#[tokio::test]
async fn test_observations_use_the_injected_clock_and_archived_bodies() -> TestResult {
    use crawler_core::{BodyArchive, Clock};
    let o_h = harness(100).await?;
    o_h.advance(std::time::Duration::from_secs(5));
    let (_, vec_o_obs) = decide(&o_h, o_h.url("sig-ok.mock", "/page")?, Purpose::Search).await?;
    assert_eq!(
        vec_o_obs[0].dt_observed_at,
        o_h.o_clock.timestamp()?,
        "observed_at is clock time"
    );
    let str_sha = vec_o_obs[0]
        .opt_str_content_sha256
        .clone()
        .ok_or("robots hash")?;
    let by_body = o_h.o_archive.load(&str_sha).ok_or("archived")?;
    assert!(String::from_utf8_lossy(&by_body).contains("Disallow: /private/"));
    Ok(())
}
