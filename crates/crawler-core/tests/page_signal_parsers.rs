//! TDMRep, ai.txt, X-Robots-Tag, and meta robots parsing.

use std::fs;
use std::path::PathBuf;

use crawler_core::{
    AiTxtFinding, AiTxtParser, ParsedSignal, RobotsTagFinding, RobotsTagParser, RobotsTagSource,
    SignalContext, SignalKind, SignalParser, TdmrepFinding, TdmrepParser, TdmrepSource,
};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Reads a signal fixture.
fn fixture(str_name: &str) -> Result<Vec<u8>, std::io::Error> {
    fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/signals")
            .join(str_name),
    )
}

/// Parses with a TDMRep source and returns the finding.
fn tdmrep(e_source: TdmrepSource, by_input: &[u8], str_path: &str) -> Option<TdmrepFinding> {
    match TdmrepParser::new(e_source).parse(by_input, &SignalContext::new("proofcrawl", str_path)) {
        ParsedSignal::Tdmrep(o_finding) => Some(o_finding),
        _ => None,
    }
}

/// Parses ai.txt and returns the finding.
fn ai_txt(by_input: &[u8], o_context: &SignalContext) -> Option<AiTxtFinding> {
    match AiTxtParser::new().parse(by_input, o_context) {
        ParsedSignal::AiTxt(o_finding) => Some(o_finding),
        _ => None,
    }
}

/// Parses robots directives and returns the finding.
fn robots_tag(e_source: RobotsTagSource, by_input: &[u8]) -> Option<RobotsTagFinding> {
    match RobotsTagParser::new(e_source).parse(by_input, &SignalContext::new("ProofCrawl", "/")) {
        ParsedSignal::RobotsTag(o_finding) => Some(o_finding),
        _ => None,
    }
}

#[test]
fn test_tdmrep_well_known_first_match_wins() -> TestResult {
    let by_json = fixture("tdmrep.json")?;
    let o_images =
        tdmrep(TdmrepSource::WellKnown, &by_json, "/images/cat.jpg").ok_or("wrong variant")?;
    assert_eq!(o_images.opt_b_reserved, Some(true));
    assert_eq!(
        o_images.opt_str_policy_url.as_deref(),
        Some("https://example.com/policy.json")
    );
    assert_eq!(
        o_images.opt_str_matched_location.as_deref(),
        Some("/images/*")
    );
    let o_other = tdmrep(TdmrepSource::WellKnown, &by_json, "/blog/post").ok_or("wrong variant")?;
    assert_eq!(
        (o_other.opt_b_reserved, o_other.u_protocol_errors),
        (Some(false), 0)
    );
    Ok(())
}

#[test]
fn test_tdmrep_invalid_values_are_unset_errors() -> TestResult {
    let by_json = fixture("tdmrep_invalid.json")?;
    for str_path in ["/strings/a", "/floats/a"] {
        let o_finding =
            tdmrep(TdmrepSource::WellKnown, &by_json, str_path).ok_or("wrong variant")?;
        assert_eq!(
            (o_finding.opt_b_reserved, o_finding.u_protocol_errors),
            (None, 2),
            "{str_path}"
        );
    }
    let o_rest = tdmrep(TdmrepSource::WellKnown, &by_json, "/other").ok_or("wrong variant")?;
    assert_eq!(
        (o_rest.opt_b_reserved, o_rest.u_protocol_errors),
        (Some(true), 1)
    );
    let o_broken = tdmrep(TdmrepSource::WellKnown, b"{not json", "/").ok_or("wrong variant")?;
    assert_eq!(
        (o_broken.opt_b_reserved, o_broken.u_protocol_errors),
        (None, 1)
    );
    Ok(())
}

#[test]
fn test_tdmrep_header_and_meta() -> TestResult {
    let o_header =
        tdmrep(TdmrepSource::Header, &fixture("headers.txt")?, "/").ok_or("wrong variant")?;
    assert_eq!(o_header.opt_b_reserved, Some(true));
    assert_eq!(
        o_header.opt_str_policy_url.as_deref(),
        Some("https://example.com/p.json")
    );
    let o_meta =
        tdmrep(TdmrepSource::Meta, &fixture("page_meta.html")?, "/").ok_or("wrong variant")?;
    assert_eq!(o_meta.opt_b_reserved, Some(true));
    let o_bad =
        tdmrep(TdmrepSource::Header, b"tdm-reservation: yes\n", "/").ok_or("wrong variant")?;
    assert_eq!((o_bad.opt_b_reserved, o_bad.u_protocol_errors), (None, 1));
    let o_absent =
        tdmrep(TdmrepSource::Header, b"content-type: text/html\n", "/").ok_or("wrong variant")?;
    assert_eq!(
        (o_absent.opt_b_reserved, o_absent.u_protocol_errors),
        (None, 0)
    );
    Ok(())
}

#[test]
fn test_ai_txt_extension_rules_cover_pages() -> TestResult {
    let o_context = SignalContext::new("proofcrawl", "/about");
    let o_spawning =
        ai_txt(&fixture("ai_txt_spawning_style.txt")?, &o_context).ok_or("wrong variant")?;
    assert!(o_spawning.b_reserved);
    assert_eq!(
        o_spawning.opt_str_assumed_extension.as_deref(),
        Some("html")
    );
    let o_images =
        ai_txt(&fixture("ai_txt_images_only.txt")?, &o_context).ok_or("wrong variant")?;
    assert!(!o_images.b_reserved);
    let o_jpg = ai_txt(
        &fixture("ai_txt_images_only.txt")?,
        &SignalContext::new("proofcrawl", "/cat.jpg"),
    )
    .ok_or("wrong variant")?;
    assert!(o_jpg.b_reserved);
    assert_eq!(o_jpg.opt_str_assumed_extension, None);
    Ok(())
}

#[test]
fn test_ai_txt_respects_custom_page_extensions() -> TestResult {
    let o_context = SignalContext::new("proofcrawl", "/doc").with_page_extensions(&["pdf"]);
    let o_finding =
        ai_txt(&fixture("ai_txt_spawning_style.txt")?, &o_context).ok_or("wrong variant")?;
    assert!(!o_finding.b_reserved);
    Ok(())
}

#[test]
fn test_x_robots_tag_scoping() -> TestResult {
    let o_finding =
        robots_tag(RobotsTagSource::Header, &fixture("headers.txt")?).ok_or("wrong variant")?;
    let vec_str_expected = [
        "max-snippet: 20",
        "noimageai",
        "noindex",
        "unavailable_after: 25 jun 2030 15:00:00 pst",
    ];
    assert_eq!(o_finding.vec_str_directives, vec_str_expected);
    assert!(!o_finding.b_noai, "otherbot-scoped noai must not apply");
    assert!(o_finding.b_noimageai);
    Ok(())
}

#[test]
fn test_meta_robots_for_robots_and_own_token() -> TestResult {
    let o_finding =
        robots_tag(RobotsTagSource::Meta, &fixture("page_meta.html")?).ok_or("wrong variant")?;
    assert_eq!(
        o_finding.vec_str_directives,
        ["noai", "noimageai", "noindex"]
    );
    assert!(o_finding.b_noai && o_finding.b_noimageai);
    Ok(())
}

#[test]
fn test_parser_kinds() {
    assert_eq!(
        TdmrepParser::new(TdmrepSource::Header).kind(),
        SignalKind::TdmrepHeader
    );
    assert_eq!(
        TdmrepParser::new(TdmrepSource::Meta).kind(),
        SignalKind::TdmrepMeta
    );
    assert_eq!(
        TdmrepParser::new(TdmrepSource::WellKnown).kind(),
        SignalKind::TdmrepWellKnown
    );
    assert_eq!(
        RobotsTagParser::new(RobotsTagSource::Header).kind(),
        SignalKind::XRobotsTag
    );
    assert_eq!(
        RobotsTagParser::new(RobotsTagSource::Meta).kind(),
        SignalKind::MetaRobots
    );
    assert_eq!(AiTxtParser::new().kind(), SignalKind::AiTxt);
}

/// X-Robots-Tag headers and whether `noai` must apply to the product token `ProofCrawl`.
const VEC_SCOPE_CASES: &[(&str, bool)] = &[
    ("X-Robots-Tag: noai\n", true),
    ("X-Robots-Tag: *: noai\n", true),
    ("X-Robots-Tag: googlebot: noai\n", false),
    ("X-Robots-Tag: proofcrawl: noai\n", true),
];

#[test]
fn test_x_robots_tag_unscoped() -> TestResult {
    assert_scope(VEC_SCOPE_CASES[0])
}

#[test]
fn test_x_robots_tag_star_scope() -> TestResult {
    assert_scope(VEC_SCOPE_CASES[1])
}

#[test]
fn test_x_robots_tag_other_bot_scope() -> TestResult {
    assert_scope(VEC_SCOPE_CASES[2])
}

#[test]
fn test_x_robots_tag_own_token_scope() -> TestResult {
    assert_scope(VEC_SCOPE_CASES[3])
}

/// Checks that `noai` applies, or not, for one header case.
fn assert_scope((str_header, b_applies): (&str, bool)) -> TestResult {
    let o_finding =
        robots_tag(RobotsTagSource::Header, str_header.as_bytes()).ok_or("wrong variant")?;
    assert_eq!(o_finding.b_noai, b_applies, "{str_header:?}");
    assert_eq!(
        o_finding.vec_str_directives.contains(&String::from("noai")),
        b_applies,
        "{str_header:?}"
    );
    Ok(())
}
