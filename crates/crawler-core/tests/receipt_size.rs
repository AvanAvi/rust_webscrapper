//! Receipt size with embedded observations versus with `signal_set_id`, for a typical page:
//! a 3.3 KB robots.txt, ai.txt and tdmrep.json answering with a 2.4 KB HTML 404 page, and four
//! page-level observations. Run with `--nocapture` to see the numbers; the test keeps the
//! pointer design honest by requiring a saving above 30 percent once observation storage is
//! counted, for a 100-page crawl of one host inside one cache TTL.

#[path = "receipt_support/sample.rs"]
mod sample;

use std::sync::Arc;

use crawler_core::{
    CanonicalEncoder, Clock, Excerpt, Hasher, MemoryBodyArchive, ParsedSignal, PolicyEvaluator,
    Purpose, RobotsParser, RobotsTagParser, RobotsTagSource, SignalContext, SignalKind,
    SignalObservation, SignalParser, SignalSet, SignalSetArchive, SignalStatus,
};
use sample::{BoxError, sample_clock, sample_submission};
use serde_json::Value;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Origin of the typical site.
const STR_ORIGIN: &str = "https://www.example-news.com";
/// Pages crawled on the host inside one cache TTL.
const U_PAGES: usize = 100;

/// Returns an observation of `by_body` (excerpt and hash) at `str_url`.
fn observe(
    o_kind: (SignalKind, SignalStatus, u16),
    str_url: &str,
    by_body: &[u8],
    opt_e_parsed: Option<ParsedSignal>,
) -> Result<SignalObservation, BoxError> {
    Ok(SignalObservation {
        e_kind: o_kind.0,
        e_status: o_kind.1,
        str_source_url: str_url.to_owned(),
        dt_observed_at: sample_clock()?.timestamp()?,
        opt_u_http_status: Some(o_kind.2),
        opt_str_content_sha256: Some(Hasher::new().sha256_hex(by_body)),
        opt_o_excerpt: (!by_body.is_empty())
            .then(|| Excerpt::new(&String::from_utf8_lossy(by_body))),
        opt_str_final_url: Some(str_url.to_owned()),
        vec_o_redirects: Vec::new(),
        b_body_truncated: false,
        u_body_bytes_seen: by_body.len() as u64,
        opt_e_failure: None,
        opt_e_parsed,
    })
}

/// Returns the robots.txt observation: 120 Disallow lines, about 3.3 KB.
fn robots_signal(o_context: &SignalContext) -> Result<SignalObservation, BoxError> {
    let mut str_robots = String::from("User-agent: *\n");
    (0..120).for_each(|u_i| str_robots.push_str(&format!("Disallow: /section-{u_i}/p/\n")));
    let e_robots = RobotsParser::new().parse(str_robots.as_bytes(), o_context);
    let o_kind = (SignalKind::RobotsTxt, SignalStatus::Present, 200);
    let str_url = format!("{STR_ORIGIN}/robots.txt");
    observe(o_kind, &str_url, str_robots.as_bytes(), Some(e_robots))
}

/// Returns an absent signal file answered by a 2.4 KB HTML 404 page.
fn not_found_signal(e_kind: SignalKind, str_path: &str) -> Result<SignalObservation, BoxError> {
    let str_404 = format!(
        "<!doctype html><html><head><style>{}</style></head><body><h1>404</h1></body></html>",
        "body{margin:0;font-family:sans-serif}".repeat(62)
    );
    let str_url = format!("{STR_ORIGIN}{str_path}");
    observe(
        (e_kind, SignalStatus::Absent, 404),
        &str_url,
        str_404.as_bytes(),
        None,
    )
}

/// Returns the three host-level observations (robots.txt present, ai.txt and tdmrep.json 404).
fn host_signals(o_context: &SignalContext) -> Result<Vec<SignalObservation>, BoxError> {
    Ok(vec![
        robots_signal(o_context)?,
        not_found_signal(SignalKind::AiTxt, "/ai.txt")?,
        not_found_signal(SignalKind::TdmrepWellKnown, "/.well-known/tdmrep.json")?,
    ])
}

/// Returns the four page-level observations of one page (robots meta present, others absent).
fn page_signals(
    o_context: &SignalContext,
    str_page: &str,
) -> Result<Vec<SignalObservation>, BoxError> {
    let by_meta: &[u8] =
        b"<meta name=\"robots\" content=\"index, follow, max-image-preview:large\">";
    let e_meta = RobotsTagParser::new(RobotsTagSource::Meta).parse(by_meta, o_context);
    let o_absent = |e_kind: SignalKind| (e_kind, SignalStatus::Absent, 200);
    let mut vec_o_obs = [SignalKind::XRobotsTag, SignalKind::TdmrepHeader]
        .into_iter()
        .map(|e_kind| observe(o_absent(e_kind), str_page, b"", None))
        .collect::<Result<Vec<_>, BoxError>>()?;
    let o_present = (SignalKind::MetaRobots, SignalStatus::Present, 200);
    vec_o_obs.push(observe(o_present, str_page, by_meta, Some(e_meta))?);
    vec_o_obs.push(observe(
        o_absent(SignalKind::TdmrepMeta),
        str_page,
        by_meta,
        None,
    )?);
    Ok(vec_o_obs)
}

/// Returns the canonical size of a value.
fn size<T: serde::Serialize>(o_value: &T) -> Result<usize, BoxError> {
    Ok(CanonicalEncoder::new().encode(o_value)?.len())
}

/// Returns (receipt with pointer, receipt with embedded signals, set record) sizes in bytes.
fn receipt_sizes(vec_o_signals: &[SignalObservation]) -> Result<(usize, usize, usize), BoxError> {
    let o_clock = sample_clock()?;
    let mut o_submission = sample_submission(&o_clock)?;
    let o_set = SignalSetArchive::new(Arc::new(MemoryBodyArchive::new()));
    o_submission.str_signal_set_id = o_set.set_id(vec_o_signals)?;
    o_submission.o_decision = PolicyEvaluator::for_purpose(Purpose::AiTrain)
        .evaluate(&SignalSet::new(vec_o_signals.to_vec()))
        .to_record(o_submission.o_decision.dt_decided_at.clone());
    let o_receipt = o_submission.seal(7, &o_clock)?;
    let mut e_embedded: Value = serde_json::to_value(&o_receipt)?;
    if let Some(map_e_sub) = e_embedded["submission"].as_object_mut() {
        map_e_sub.remove("signal_set_id");
        map_e_sub.insert(
            String::from("signals"),
            serde_json::to_value(vec_o_signals)?,
        );
    }
    let u_record = size(&vec![Hasher::new().sha256_hex(b""); vec_o_signals.len()])?;
    Ok((size(&o_receipt)?, size(&e_embedded)?, u_record))
}

/// Returns the saving of `u_size` against `u_embedded`, in percent.
fn saving(u_size: usize, u_embedded: usize) -> f64 {
    (1.0 - u_size as f64 / u_embedded as f64) * 100.0
}

/// Returns (embedded receipt, pointer receipt, set record, host-level obs, page-level obs) sizes.
fn measure() -> Result<[usize; 5], BoxError> {
    let str_page = format!("{STR_ORIGIN}/articles/2026/10/some-story");
    let o_context = SignalContext::new("proofcrawl", "/articles/2026/10/some-story");
    let vec_o_host = host_signals(&o_context)?;
    let vec_o_page = page_signals(&o_context, &str_page)?;
    let vec_o_all: Vec<SignalObservation> = vec_o_host.iter().chain(&vec_o_page).cloned().collect();
    let (u_pointer, u_embedded, u_record) = receipt_sizes(&vec_o_all)?;
    let u_host: usize = vec_o_host
        .iter()
        .map(size)
        .sum::<Result<usize, BoxError>>()?;
    let u_page: usize = vec_o_page
        .iter()
        .map(size)
        .sum::<Result<usize, BoxError>>()?;
    Ok([u_embedded, u_pointer, u_record, u_host, u_page])
}

#[test]
fn test_pointer_saves_more_than_thirty_percent() -> TestResult {
    let [u_embedded, u_pointer, u_record, u_host, u_page] = measure()?;
    let u_per_page = u_pointer + u_record + u_page;
    let (u_total_100, u_total_1) = (u_per_page + u_host / U_PAGES, u_per_page + u_host);
    println!(
        "embedded receipt {u_embedded} B; pointer receipt {u_pointer} B ({:.0}% smaller)",
        saving(u_pointer, u_embedded)
    );
    println!(
        "observations: host-level {u_host} B shared, page-level {u_page} B, set record {u_record} B"
    );
    println!(
        "stored per receipt, {U_PAGES} pages/host: {u_total_100} B ({:.0}% smaller)",
        saving(u_total_100, u_embedded)
    );
    println!(
        "stored per receipt, 1 page/host: {u_total_1} B ({:.0}% smaller)",
        saving(u_total_1, u_embedded)
    );
    assert!(saving(u_pointer, u_embedded) > 30.0 && saving(u_total_100, u_embedded) > 30.0);
    Ok(())
}
