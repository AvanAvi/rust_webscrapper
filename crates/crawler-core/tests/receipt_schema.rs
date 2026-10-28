//! Schema fingerprint: a golden hash of the receipt's canonical field list (every key path with
//! its JSON types, one signal per parsed format). Changing a field name, adding or removing a
//! field, or changing a field's JSON type changes the fingerprint; the test then fails until
//! `U_RECEIPT_SCHEMA_VERSION` is bumped and the golden pair below is updated together.
//! Enum value sets (for example the failure names) are not part of the fingerprint.

use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv6Addr};

use crawler_core::{
    AiTxtFinding, CanonicalEncoder, ContentSignal, DecisionRecord, Excerpt, FetchRecord, Hasher,
    IdentityRecord, Outcome, ParsedSignal, Purpose, Receipt, RedirectHop, RobotsFinding,
    RobotsRule, RobotsTagFinding, RuleKind, SignalFailure, SignalKind, SignalObservation,
    SignalStatus, TargetRecord, TdmrepFinding, Timestamp, U_RECEIPT_SCHEMA_VERSION,
};
use serde_json::Value;
use uuid::Uuid;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Boxed error for helpers.
type BoxError = Box<dyn std::error::Error>;
/// Key paths and the JSON types seen at each.
type FieldMap = BTreeMap<String, BTreeSet<&'static str>>;

/// Schema version and SHA-256 of the canonical field list it was frozen with.
const O_GOLDEN_FINGERPRINT: (u32, &str) = (
    2,
    "8e1f00ebc5b696e4996c31a9b9f660d3b98c93df3dcfc3fd73f864448d7e4859",
);

/// Returns a rule.
fn rule() -> RobotsRule {
    RobotsRule {
        e_kind: RuleKind::Disallow,
        str_pattern: String::from("/private/"),
    }
}

/// Returns one finding of every parsed format, with every optional field filled.
fn findings() -> Vec<(SignalKind, ParsedSignal)> {
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
fn robots() -> RobotsFinding {
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
fn ai_txt() -> AiTxtFinding {
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
fn tdmrep() -> TdmrepFinding {
    TdmrepFinding {
        opt_b_reserved: Some(true),
        opt_str_policy_url: Some(String::from("https://example.com/policy.json")),
        opt_str_matched_location: Some(String::from("/*")),
        u_protocol_errors: 0,
    }
}

/// Returns an X-Robots-Tag finding.
fn robots_tag() -> RobotsTagFinding {
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
        dt_started_at: Timestamp::parse("2026-10-28T00:00:00.000000Z")?,
        dt_finished_at: Timestamp::parse("2026-10-28T00:00:01.000000Z")?,
        u_http_status: 200,
        opt_str_content_type: Some(String::from("text/html")),
        u_bytes: 1,
        str_body_sha256: Hasher::new().sha256_hex(b"x"),
        vec_o_redirects: vec![hop()],
        vec_e_resolved_ips: vec![IpAddr::V6(Ipv6Addr::LOCALHOST)],
    })
}

/// Returns the maximal receipt.
fn maximal_receipt() -> Result<Receipt, BoxError> {
    let vec_o_signals = findings()
        .into_iter()
        .map(|(e_kind, e_parsed)| signal(e_kind, e_parsed))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Receipt {
        u_schema_version: U_RECEIPT_SCHEMA_VERSION,
        uuid_receipt_id: Uuid::nil(),
        u_sequence: 1,
        uuid_job_id: Uuid::nil(),
        uuid_task_id: Uuid::nil(),
        u_lease_epoch: 1,
        uuid_worker_id: Uuid::nil(),
        o_identity: IdentityRecord {
            str_user_agent: String::from("proofcrawl/0.1"),
            e_purpose: Purpose::Search,
            str_contact_url: String::from("https://crawler.example/contact"),
        },
        o_target: target(),
        vec_o_signals,
        o_decision: decision(),
        opt_o_fetch: Some(fetch()?),
    })
}

/// Returns the target with every optional field filled.
fn target() -> TargetRecord {
    TargetRecord {
        str_url_requested: String::from("https://example.com/a"),
        str_url_normalized: String::from("https://example.com/a"),
        opt_str_url_final: Some(String::from("https://example.com/a")),
        str_host: String::from("example.com"),
    }
}

/// Returns a decision.
fn decision() -> DecisionRecord {
    DecisionRecord {
        e_outcome: Outcome::Deny,
        str_matched_rule: String::from("robots_txt.disallow"),
        vec_str_reasons: vec![String::from("x")],
        str_policy_version: String::from("v1"),
    }
}

/// Returns the JSON type name of a value.
fn type_name(e_value: &Value) -> &'static str {
    match e_value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Records a value's type at its path and walks into it; tagged objects add `<format>`.
fn walk(str_path: &str, e_value: &Value, o_fields: &mut FieldMap) {
    o_fields
        .entry(str_path.to_owned())
        .or_default()
        .insert(type_name(e_value));
    match e_value {
        Value::Array(vec_e_items) => {
            let str_item = format!("{str_path}[]");
            vec_e_items
                .iter()
                .for_each(|e_item| walk(&str_item, e_item, o_fields));
        }
        Value::Object(map_e_fields) => walk_object(str_path, map_e_fields, o_fields),
        _ => {}
    }
}

/// Walks the fields of an object, tagging the path with its `format` value when present.
fn walk_object(
    str_path: &str,
    map_e_fields: &serde_json::Map<String, Value>,
    o_fields: &mut FieldMap,
) {
    let str_tag = match map_e_fields.get("format") {
        Some(Value::String(str_format)) => format!("<{str_format}>"),
        _ => String::new(),
    };
    for (str_key, e_child) in map_e_fields {
        walk(&format!("{str_path}{str_tag}.{str_key}"), e_child, o_fields);
    }
}

/// Returns the canonical field list: one `path: types` line per key path, sorted.
fn field_list() -> Result<String, BoxError> {
    let by_json = maximal_receipt()?.canonical_bytes(&CanonicalEncoder::new())?;
    let e_root: Value = serde_json::from_slice(&by_json)?;
    let mut o_fields = FieldMap::new();
    walk("$", &e_root, &mut o_fields);
    Ok(o_fields
        .iter()
        .map(|(str_path, set_types)| {
            let vec_str_types: Vec<&str> = set_types.iter().copied().collect();
            format!("{str_path}: {}\n", vec_str_types.join("|"))
        })
        .collect())
}

#[test]
fn test_receipt_schema_matches_its_version() -> TestResult {
    let str_fields = field_list()?;
    let str_hash = Hasher::new().sha256_hex(str_fields.as_bytes());
    assert_eq!(
        (U_RECEIPT_SCHEMA_VERSION, str_hash.as_str()),
        O_GOLDEN_FINGERPRINT,
        "\nThe receipt schema changed without a schema_version bump and fingerprint update.\n\
         Bump U_RECEIPT_SCHEMA_VERSION in crates/crawler-core/src/receipt.rs, document the change \
         in docs/RECEIPT_SCHEMA.md, then set O_GOLDEN_FINGERPRINT to ({}, \"{str_hash}\").\n\
         Current field list:\n{str_fields}",
        U_RECEIPT_SCHEMA_VERSION + u32::from(U_RECEIPT_SCHEMA_VERSION == O_GOLDEN_FINGERPRINT.0),
    );
    Ok(())
}

#[test]
fn test_maximal_sample_fills_every_optional_field() -> TestResult {
    let str_fields = field_list()?;
    let vec_str_only_null: Vec<&str> = str_fields
        .lines()
        .filter(|str_line| str_line.ends_with(": null"))
        .collect();
    assert!(
        vec_str_only_null.is_empty(),
        "fields only ever null hide their type: {vec_str_only_null:?}"
    );
    Ok(())
}
