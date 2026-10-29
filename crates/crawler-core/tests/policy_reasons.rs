//! Exact wording of denied-decision reasons and crawler identity rules.

use crawler_core::{
    CoreError, CrawlerIdentity, CrawlerRecord, ParsedSignal, PolicyEvaluator, Purpose,
    RobotsParser, SignalContext, SignalKind, SignalObservation, SignalParser, SignalSet,
    SignalStatus, TdmrepParser, TdmrepSource, Timestamp,
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
        opt_str_final_url: None,
        vec_o_redirects: Vec::new(),
        b_body_truncated: false,
        u_body_bytes_seen: 0,
        opt_e_failure: None,
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
        "HTTPS://Crawler.Example/contact",
        Purpose::default(),
    )?;
    assert_eq!(
        o_identity.user_agent(),
        format!(
            "proofcrawl/{} (+https://crawler.example/contact; purpose=ai_train)",
            env!("CARGO_PKG_VERSION")
        )
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
        ("Googlebot", "https://x.example/"),
        ("GPTBot", "https://x.example/"),
        ("*", "https://x.example/"),
        ("proof crawl", "https://x.example/"),
        ("proofcrawl2", "https://x.example/"),
        ("proofcrawl", "ftp://x.example/"),
        ("proofcrawl", "https://user:pw@x.example/"),
    ];
    for (str_token, str_contact) in vec_o_bad {
        let res_identity = CrawlerIdentity::new(str_token, str_contact, Purpose::AiTrain);
        assert!(
            matches!(res_identity, Err(CoreError::InvalidIdentity { .. })),
            "{str_token} {str_contact}"
        );
    }
}

#[test]
fn test_user_agent_version_matches_crawler_version() -> TestResult {
    let o_identity = CrawlerIdentity::new(
        "proofcrawl",
        "https://crawler.example/contact",
        Purpose::AiTrain,
    )?;
    let o_crawler = CrawlerRecord::from_build();
    let str_prefix = format!("{}/{} ", o_identity.product_token(), o_crawler.version());
    assert!(
        o_identity.user_agent().starts_with(&str_prefix),
        "{} does not start with {str_prefix}",
        o_identity.user_agent()
    );
    assert_eq!(
        o_identity.to_record().str_user_agent,
        o_identity.user_agent()
    );
    Ok(())
}

#[test]
fn test_unchecked_signal_reason_says_so_plainly() -> TestResult {
    let mut o_obs = present(
        SignalKind::TdmrepWellKnown,
        "https://example.com/.well-known/tdmrep.json",
        ParsedSignal::Tdmrep(Default::default()),
    )?;
    o_obs.e_status = SignalStatus::Unreachable;
    o_obs.opt_u_http_status = Some(503);
    o_obs.opt_e_parsed = None;
    let o_decision =
        PolicyEvaluator::for_purpose(Purpose::AiTrain).evaluate(&SignalSet::new(vec![o_obs]));
    let str_reason = &o_decision.reasons()[0];
    println!("{str_reason}");
    assert_eq!(o_decision.outcome(), crawler_core::Outcome::Allow);
    assert_eq!(
        str_reason,
        "The TDMRep file at https://example.com/.well-known/tdmrep.json could not be checked because it could not be retrieved (HTTP 503). Policy v1 lets only signals that were actually observed deny, so this unchecked signal has no effect on the decision."
    );
    Ok(())
}

#[test]
fn test_unenforced_robots_directives_stay_visible() -> TestResult {
    let e_tag = crawler_core::RobotsTagParser::new(crawler_core::RobotsTagSource::Header).parse(
        b"X-Robots-Tag: noindex, noarchive",
        &SignalContext::new("proofcrawl", "/"),
    );
    let o_obs = present(SignalKind::XRobotsTag, "https://example.com/page", e_tag)?;
    let str_reason = PolicyEvaluator::for_purpose(Purpose::Search)
        .evaluate(&SignalSet::new(vec![o_obs]))
        .reasons()
        .remove(0);
    println!("{str_reason}");
    assert!(
        str_reason.contains("\"noarchive, noindex\"") && str_reason.contains("does not enforce"),
        "{str_reason}"
    );
    Ok(())
}
