//! Shared sample receipt: one real robots.txt observation, a decision from the real evaluator,
//! and a fetch that starts after the decision.

use std::net::{IpAddr, Ipv4Addr};

use crawler_core::{
    Clock, CrawlerRecord, Excerpt, FakeClock, FetchRecord, Hasher, IdSource, IdentityRecord,
    MemoryBodyArchive, PolicyEvaluator, Purpose, ReceiptSubmission, RedirectHop, RobotsParser,
    SequentialIdSource, SignalContext, SignalKind, SignalObservation, SignalParser, SignalSet,
    SignalSetArchive, SignalStatus, TargetRecord,
};
use time::OffsetDateTime;

/// Boxed error for helpers.
pub type BoxError = Box<dyn std::error::Error>;

/// Crawler version the frozen vector pins. It differs from the build version on purpose, so a
/// workspace version bump never changes the vector and the pin is visibly not the build's.
pub const STR_PINNED_CRAWLER_VERSION: &str = "0.1.0-frozen";

/// robots.txt body of the sample signal.
const STR_ROBOTS: &str = "User-agent: *\nAllow: /\n";

/// Returns the fake clock all samples use (2026-10-26T12:34:56Z).
pub fn sample_clock() -> Result<FakeClock, BoxError> {
    Ok(FakeClock::new(OffsetDateTime::from_unix_timestamp(
        1_793_018_096,
    )?))
}

/// Returns the observed robots.txt signal; hash, excerpt, and byte count all describe one body.
pub fn sample_signals(o_clock: &FakeClock) -> Result<Vec<SignalObservation>, BoxError> {
    let o_context = SignalContext::new("proofcrawl", "/a");
    Ok(vec![SignalObservation {
        e_kind: SignalKind::RobotsTxt,
        e_status: SignalStatus::Present,
        str_source_url: String::from("https://example.com/robots.txt"),
        dt_observed_at: o_clock.timestamp()?,
        opt_u_http_status: Some(200),
        opt_str_content_sha256: Some(Hasher::new().sha256_hex(STR_ROBOTS.as_bytes())),
        opt_o_excerpt: Some(Excerpt::new(STR_ROBOTS)),
        opt_str_final_url: Some(String::from("https://example.com/robots.txt")),
        vec_o_redirects: Vec::new(),
        b_body_truncated: false,
        u_body_bytes_seen: STR_ROBOTS.len() as u64,
        opt_e_failure: None,
        opt_e_parsed: Some(RobotsParser::new().parse(STR_ROBOTS.as_bytes(), &o_context)),
    }])
}

/// Returns a fetch record that starts 10 ms after the clock's current time.
fn sample_fetch(o_clock: &FakeClock) -> Result<FetchRecord, BoxError> {
    o_clock.advance(time::Duration::milliseconds(10));
    let dt_started_at = o_clock.timestamp()?;
    o_clock.advance(time::Duration::milliseconds(250));
    Ok(FetchRecord {
        dt_started_at,
        dt_finished_at: o_clock.timestamp()?,
        u_http_status: 200,
        opt_str_content_type: Some(String::from("text/html; charset=utf-8")),
        u_body_bytes: 13,
        str_body_sha256: Hasher::new().sha256_hex(b"<html></html>"),
        vec_o_redirects: vec![RedirectHop {
            str_url: String::from("http://example.com/a"),
            u_status: 301,
        }],
        vec_e_resolved_ips: vec![IpAddr::V4(Ipv4Addr::new(93, 184, 215, 14))],
        opt_u_retry_after_ms: None,
    })
}

/// Returns the pinned crawler record, decoded because only the build can construct one.
pub fn pinned_crawler() -> Result<CrawlerRecord, BoxError> {
    Ok(serde_json::from_value(serde_json::json!({
        "name": "proofcrawl",
        "version": STR_PINNED_CRAWLER_VERSION,
    }))?)
}

/// Returns the declared crawler identity.
fn sample_identity() -> IdentityRecord {
    IdentityRecord {
        str_user_agent: format!(
            "proofcrawl/{STR_PINNED_CRAWLER_VERSION} (+https://crawler.example/contact; purpose=ai-train)"
        ),
        str_product_token: String::from("proofcrawl"),
        e_purpose: Purpose::AiTrain,
        str_contact_url: String::from("https://crawler.example/contact"),
    }
}

/// Returns the target URL forms.
fn sample_target() -> TargetRecord {
    TargetRecord {
        str_url_requested: String::from("HTTP://Example.com:80/a"),
        str_url_normalized: String::from("http://example.com/a"),
        opt_str_url_final: Some(String::from("https://example.com/a")),
        str_host: String::from("example.com"),
        u_depth: 1,
    }
}

/// Returns an allowed submission: signals observed, decided by the real evaluator, then fetched.
pub fn sample_submission(o_clock: &FakeClock) -> Result<ReceiptSubmission, BoxError> {
    let vec_o_signals = sample_signals(o_clock)?;
    let o_decision = PolicyEvaluator::for_purpose(Purpose::AiTrain)
        .evaluate(&SignalSet::new(vec_o_signals.clone()));
    let o_ids = SequentialIdSource::new(1);
    let o_set = SignalSetArchive::new(std::sync::Arc::new(MemoryBodyArchive::new()));
    Ok(ReceiptSubmission {
        uuid_receipt_id: o_ids.next_id(),
        uuid_job_id: o_ids.next_id(),
        uuid_task_id: o_ids.next_id(),
        u_lease_epoch: 3,
        uuid_worker_id: o_ids.next_id(),
        o_crawler: pinned_crawler()?,
        o_identity: sample_identity(),
        o_target: sample_target(),
        str_signal_set_id: o_set.set_id(&vec_o_signals)?,
        o_decision: o_decision.to_record(o_clock.timestamp()?),
        opt_o_fetch: Some(sample_fetch(o_clock)?),
        opt_e_fetch_failure: None,
    })
}
