//! Maximal schema samples: every optional field filled, one signal per parsed format, and both
//! fetch outcomes, so the field list shows every field with its non-null type.

use std::net::{IpAddr, Ipv6Addr};

use crawler_core::{
    AiTxtFinding, ContentSignal, CrawlerRecord, DecisionRecord, Excerpt, FakeClock, FetchFailure,
    FetchRecord, Hasher, IdentityRecord, Outcome, ParsedSignal, PolicyRule, Purpose, Receipt,
    ReceiptSubmission, RedirectHop, RobotsFinding, RobotsRule, RobotsTagFinding, RuleKind,
    SignalFailure, SignalKind, SignalObservation, SignalStatus, TargetRecord, TdmrepFinding,
    Timestamp,
};
use time::OffsetDateTime;
use uuid::Uuid;

/// Boxed error for helpers.
pub type BoxError = Box<dyn std::error::Error>;

/// Returns a rule.
pub fn rule() -> RobotsRule {
    RobotsRule {
        e_kind: RuleKind::Disallow,
        str_pattern: String::from("/private/"),
    }
}

/// Returns one finding of every parsed format, with every optional field filled.
pub fn findings() -> Vec<(SignalKind, ParsedSignal)> {
    vec![
        (SignalKind::RobotsTxt, ParsedSignal::RobotsTxt(robots())),
        (SignalKind::AiTxt, ParsedSignal::AiTxt(ai_txt())),
        (SignalKind::TdmrepWellKnown, ParsedSignal::Tdmrep(tdmrep())),
        (
            SignalKind::XRobotsTag,
            ParsedSignal::RobotsTag(robots_tag()),
        ),
    ]
}

/// Returns a robots.txt finding with every optional field filled.
pub fn robots() -> RobotsFinding {
    RobotsFinding {
        b_allowed: false,
        opt_str_matched_agent: Some(String::from("*")),
        opt_o_matched_rule: Some(rule()),
        opt_u_crawl_delay_ms: Some(1000),
        opt_o_content_signal: Some(ContentSignal {
            opt_b_search: Some(true),
            opt_b_ai_input: Some(false),
            opt_b_ai_train: Some(false),
        }),
        u_groups: 1,
        u_invalid_lines: 0,
        b_truncated: false,
    }
}

/// Returns an ai.txt finding with every optional field filled.
pub fn ai_txt() -> AiTxtFinding {
    AiTxtFinding {
        b_reserved: true,
        opt_str_matched_agent: Some(String::from("*")),
        opt_o_matched_rule: Some(rule()),
        opt_str_assumed_extension: Some(String::from("html")),
        u_invalid_lines: 0,
        b_truncated: false,
    }
}

/// Returns a TDMRep finding with every optional field filled.
pub fn tdmrep() -> TdmrepFinding {
    TdmrepFinding {
        opt_b_reserved: Some(true),
        opt_str_policy_url: Some(String::from("https://example.com/policy.json")),
        opt_str_matched_location: Some(String::from("/*")),
        u_protocol_errors: 0,
    }
}

/// Returns an X-Robots-Tag finding.
pub fn robots_tag() -> RobotsTagFinding {
    RobotsTagFinding {
        vec_str_directives: vec![String::from("noai")],
        b_noai: true,
        b_noimageai: false,
    }
}

/// Returns a present observation with every optional field filled.
fn signal(e_kind: SignalKind, e_parsed: ParsedSignal) -> Result<SignalObservation, BoxError> {
    Ok(SignalObservation {
        e_kind,
        e_status: SignalStatus::Present,
        str_source_url: String::from("https://example.com/robots.txt"),
        dt_observed_at: Timestamp::parse("2026-10-28T00:00:00.000000Z")?,
        opt_u_http_status: Some(200),
        opt_str_content_sha256: Some(Hasher::new().sha256_hex(b"x")),
        opt_o_excerpt: Some(Excerpt::new("x")),
        opt_str_final_url: Some(String::from("https://example.com/robots.txt")),
        vec_o_redirects: vec![hop()],
        b_body_truncated: false,
        u_body_bytes_seen: 1,
        opt_e_failure: Some(SignalFailure::ServerError),
        opt_e_parsed: Some(e_parsed),
    })
}

/// Returns one maximal observation per parsed format.
pub fn observations() -> Result<Vec<SignalObservation>, BoxError> {
    findings()
        .into_iter()
        .map(|(e_kind, e_parsed)| signal(e_kind, e_parsed))
        .collect()
}

/// Returns a redirect hop.
fn hop() -> RedirectHop {
    RedirectHop {
        str_url: String::from("http://example.com/"),
        u_status: 301,
    }
}

/// Returns a fetch record with every optional field filled.
fn fetch() -> Result<FetchRecord, BoxError> {
    Ok(FetchRecord {
        dt_started_at: Timestamp::parse("2026-10-28T00:00:01.000000Z")?,
        dt_finished_at: Timestamp::parse("2026-10-28T00:00:02.000000Z")?,
        u_http_status: 200,
        opt_str_content_type: Some(String::from("text/html")),
        u_body_bytes: 1,
        str_body_sha256: Hasher::new().sha256_hex(b"x"),
        vec_o_redirects: vec![hop()],
        vec_e_resolved_ips: vec![IpAddr::V6(Ipv6Addr::LOCALHOST)],
        opt_u_retry_after_ms: Some(1000),
    })
}

/// Returns the target with every optional field filled.
fn target() -> TargetRecord {
    TargetRecord {
        str_url_requested: String::from("https://example.com/a"),
        str_url_normalized: String::from("https://example.com/a"),
        opt_str_url_final: Some(String::from("https://example.com/a")),
        str_host: String::from("example.com"),
        u_depth: 0,
    }
}

/// Returns an allow decision made before the fetch.
fn decision() -> Result<DecisionRecord, BoxError> {
    Ok(DecisionRecord {
        e_outcome: Outcome::Allow,
        e_matched_rule: PolicyRule::NoReservation,
        vec_str_reasons: vec![String::from("x")],
        str_policy_version: String::from("v1"),
        dt_decided_at: Timestamp::parse("2026-10-28T00:00:00.500000Z")?,
    })
}

/// Returns an allowed submission with a fetch, or with a fetch failure when `b_failed`.
fn submission(b_failed: bool) -> Result<ReceiptSubmission, BoxError> {
    Ok(ReceiptSubmission {
        uuid_receipt_id: Uuid::nil(),
        uuid_job_id: Uuid::nil(),
        uuid_task_id: Uuid::nil(),
        u_lease_epoch: 1,
        uuid_worker_id: Uuid::nil(),
        o_crawler: CrawlerRecord::from_build(),
        o_identity: IdentityRecord {
            str_user_agent: String::from("proofcrawl/0.1.0"),
            str_product_token: String::from("proofcrawl"),
            e_purpose: Purpose::Search,
            str_contact_url: String::from("https://crawler.example/contact"),
        },
        o_target: target(),
        str_signal_set_id: Hasher::new().sha256_hex(b"set"),
        o_decision: decision()?,
        opt_o_fetch: (!b_failed).then(fetch).transpose()?,
        opt_e_fetch_failure: b_failed.then_some(FetchFailure::Timeout),
    })
}

/// Returns two sealed receipts: one fetched, one whose fetch failed.
pub fn receipts() -> Result<Vec<Receipt>, BoxError> {
    let o_clock = FakeClock::new(OffsetDateTime::from_unix_timestamp(1_793_145_603)?);
    Ok(vec![
        submission(false)?.seal(1, &o_clock)?,
        submission(true)?.seal(2, &o_clock)?,
    ])
}
