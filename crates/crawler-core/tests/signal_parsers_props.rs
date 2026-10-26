//! Every parser survives arbitrary bytes, and every finding encodes canonically (no floats).

use crawler_core::{
    AiTxtParser, CanonicalEncoder, ParsedSignal, RobotsParser, RobotsTagParser, RobotsTagSource,
    SignalContext, SignalParser, TdmrepParser, TdmrepSource,
};
use proptest::prelude::*;

/// Returns every parser configuration.
fn all_parsers() -> Vec<Box<dyn SignalParser>> {
    vec![
        Box::new(RobotsParser::new()),
        Box::new(AiTxtParser::new()),
        Box::new(TdmrepParser::new(TdmrepSource::Header)),
        Box::new(TdmrepParser::new(TdmrepSource::Meta)),
        Box::new(TdmrepParser::new(TdmrepSource::WellKnown)),
        Box::new(RobotsTagParser::new(RobotsTagSource::Header)),
        Box::new(RobotsTagParser::new(RobotsTagSource::Meta)),
    ]
}

/// Checks that a finding round-trips through JSON and has a canonical encoding.
fn check_encodable(e_parsed: &ParsedSignal) -> Result<(), TestCaseError> {
    let by_canonical = CanonicalEncoder::new()
        .encode(e_parsed)
        .map_err(|err_e| TestCaseError::fail(err_e.to_string()))?;
    let e_decoded: ParsedSignal = serde_json::from_slice(&by_canonical)
        .map_err(|err_e| TestCaseError::fail(err_e.to_string()))?;
    prop_assert_eq!(&e_decoded, e_parsed);
    Ok(())
}

/// Generates robots-like text from a vocabulary of record keys, values, and junk.
fn robots_like() -> impl Strategy<Value = String> {
    let o_line = (
        "(User-agent|Allow|Disallow|Crawl-delay|Content-Signal|Sitemap|[A-Za-z-]{0,8})",
        "(:| : )?",
        "(\\*|proofcrawl|/|/[a-z*$%0-9]{0,12}|[0-9.]{0,6}|(search|ai-train|ai-input)=(yes|no|x)(, ?)?|[ -~]{0,20})",
        "(\n|\r\n|\r| # c\n)",
    );
    prop::collection::vec(o_line, 0..40).prop_map(|vec_o_lines| {
        vec_o_lines
            .into_iter()
            .map(|(str_k, str_s, str_v, str_e)| format!("{str_k}{str_s}{str_v}{str_e}"))
            .collect()
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn test_arbitrary_bytes_never_panic(by_input in prop::collection::vec(any::<u8>(), 0..4096), str_path in "/[ -~]{0,40}", str_token in "[A-Za-z*-]{0,12}") {
        let o_context = SignalContext::new(&str_token, &str_path);
        for o_parser in all_parsers() {
            check_encodable(&o_parser.parse(&by_input, &o_context))?;
        }
    }

    #[test]
    fn test_robots_like_text_never_panics(str_text in robots_like(), str_path in "/[a-z/.]{0,20}") {
        let o_context = SignalContext::new("proofcrawl", &str_path);
        for o_parser in all_parsers() {
            check_encodable(&o_parser.parse(str_text.as_bytes(), &o_context))?;
        }
    }

    #[test]
    fn test_json_like_text_never_panics(str_text in r#"\[(\{"location": ?"[/*a-z$]{0,8}", ?"tdm-reservation": ?(0|1|1\.0|-1|"1"|null|true|1e2)(, ?"tdm-policy": ?"[a-z:/.]{0,10}")?\},? ?){0,6}\]?"#, str_path in "/[a-z/]{0,12}") {
        let e_parsed = TdmrepParser::new(TdmrepSource::WellKnown).parse(str_text.as_bytes(), &SignalContext::new("proofcrawl", &str_path));
        check_encodable(&e_parsed)?;
    }
}
