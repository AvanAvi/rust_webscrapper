//! Exact wording of denied-decision reasons and crawler identity rules.

use crawler_core::{
    CoreError, CrawlerIdentity, ParsedSignal, PolicyEvaluator, Purpose, RobotsParser,
    SignalContext, SignalKind, SignalObservation, SignalParser, SignalSet, SignalStatus,
    TdmrepParser, TdmrepSource, Timestamp,
};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Builds a present observation from a source URL and parser output.
fn present(
    e_kind: SignalKind,
    str_url: &str,
    e_parsed: ParsedSignal,
) -> Result<SignalObservation, Box<dyn std::error::Error>> {
    Ok(SignalObservation {
        e_kind,
        e_status: SignalStatus::Present,
        str_source_url: str_url.to_owned(),
        dt_observed_at: Timestamp::parse("2026-10-27T00:00:00.000000Z")?,
        opt_u_http_status: Some(200),
        opt_str_content_sha256: None,
        opt_o_excerpt: None,
        opt_e_parsed: Some(e_parsed),
    })
}

#[test]
fn test_reason_sentences_read_plainly() -> TestResult {
    let o_context = SignalContext::new("proofcrawl", "/images/cat.jpg");
    let str_json = r#"[{"location":"/images/*","tdm-reservation":1}]"#;
    let e_tdm = TdmrepParser::new(TdmrepSource::WellKnown).parse(str_json.as_bytes(), &o_context);
    let e_robots = RobotsParser::new().parse(b"User-agent: *\nDisallow: /images/\n", &o_context);
    let vec_o_obs = vec![
        present(
            SignalKind::TdmrepWellKnown,
            "https://example.com/.well-known/tdmrep.json",
            e_tdm,
        )?,
        present(
            SignalKind::RobotsTxt,
            "https://example.com/robots.txt",
            e_robots,
        )?,
    ];
    let vec_str_reasons = PolicyEvaluator::for_purpose(Purpose::AiTrain)
        .evaluate(&SignalSet::new(vec_o_obs))
        .reasons();
    assert_eq!(
        vec_str_reasons,
        [
            "The TDMRep file at https://example.com/.well-known/tdmrep.json reserves text-and-data-mining rights (tdm-reservation: 1) for location \"/images/*\", which denies AI training.",
            "robots.txt at https://example.com/robots.txt disallows this URL for user-agent group \"*\" (rule \"Disallow: /images/\"), so it may not be crawled for any purpose.",
        ]
    );
    Ok(())
}

#[test]
fn test_identity_declares_token_contact_and_purpose() -> TestResult {
    let o_identity = CrawlerIdentity::new(
        "proofcrawl",
        "0.1.0",
        "HTTPS://Crawler.Example/contact",
        Purpose::default(),
    )?;
    assert_eq!(
        o_identity.user_agent(),
        "proofcrawl/0.1.0 (+https://crawler.example/contact; purpose=ai_train)"
    );
    let o_record = o_identity.to_record();
    assert_eq!(
        (o_record.e_purpose, o_record.str_contact_url.as_str()),
        (Purpose::AiTrain, "https://crawler.example/contact")
    );
    Ok(())
}

#[test]
fn test_identity_rejects_spoofing_and_bad_values() {
    let vec_o_bad = [
        ("Googlebot", "1.0", "https://x.example/"),
        ("GPTBot", "1.0", "https://x.example/"),
        ("*", "1.0", "https://x.example/"),
        ("proof crawl", "1.0", "https://x.example/"),
        ("proofcrawl2", "1.0", "https://x.example/"),
        ("proofcrawl", "", "https://x.example/"),
        ("proofcrawl", "1.0", "ftp://x.example/"),
        ("proofcrawl", "1.0", "https://user:pw@x.example/"),
    ];
    for (str_token, str_version, str_contact) in vec_o_bad {
        let res_identity =
            CrawlerIdentity::new(str_token, str_version, str_contact, Purpose::AiTrain);
        assert!(
            matches!(res_identity, Err(CoreError::InvalidIdentity { .. })),
            "{str_token} {str_version} {str_contact}"
        );
    }
}
