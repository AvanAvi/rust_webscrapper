//! Decision-matrix data: scenarios and expected cells, shared by `policy_matrix.rs`.

use crawler_core::{Outcome, Purpose, SignalKind, SignalStatus};

/// Scenario: name, kind, status, HTTP status, and source bytes.
pub type Scenario = (
    &'static str,
    SignalKind,
    SignalStatus,
    Option<u16>,
    &'static str,
);
/// Expected outcome and matched rule for one purpose.
pub type Cell = (Outcome, &'static str);

/// Purposes in column order.
pub const VEC_E_PURPOSES: [Purpose; 4] = [
    Purpose::Search,
    Purpose::AiInput,
    Purpose::AiTrain,
    Purpose::Archive,
];
/// Allowed with no reservation.
pub const O_ALLOW: Cell = (Outcome::Allow, "no_reservation");

/// Every scenario; each consults exactly one signal.
pub const VEC_SCENARIOS: &[Scenario] = &[
    (
        "robots_disallow",
        SignalKind::RobotsTxt,
        SignalStatus::Present,
        Some(200),
        "User-agent: *\nDisallow: /\n",
    ),
    (
        "robots_5xx",
        SignalKind::RobotsTxt,
        SignalStatus::Unreachable,
        Some(503),
        "",
    ),
    (
        "robots_4xx",
        SignalKind::RobotsTxt,
        SignalStatus::Absent,
        Some(404),
        "",
    ),
    (
        "content_signal_search_no",
        SignalKind::RobotsTxt,
        SignalStatus::Present,
        Some(200),
        "User-agent: *\nContent-Signal: search=no\nAllow: /\n",
    ),
    (
        "content_signal_ai_input_no",
        SignalKind::RobotsTxt,
        SignalStatus::Present,
        Some(200),
        "User-agent: *\nContent-Signal: ai-input=no\nAllow: /\n",
    ),
    (
        "content_signal_ai_train_no",
        SignalKind::RobotsTxt,
        SignalStatus::Present,
        Some(200),
        "User-agent: *\nContent-Signal: ai-train=no\nAllow: /\n",
    ),
    (
        "ai_txt_disallow",
        SignalKind::AiTxt,
        SignalStatus::Present,
        Some(200),
        "User-Agent: *\nDisallow: /\n",
    ),
    (
        "ai_txt_absent",
        SignalKind::AiTxt,
        SignalStatus::Absent,
        Some(404),
        "",
    ),
    (
        "tdmrep_well_known_1",
        SignalKind::TdmrepWellKnown,
        SignalStatus::Present,
        Some(200),
        r#"[{"location":"/*","tdm-reservation":1}]"#,
    ),
    (
        "tdmrep_well_known_absent",
        SignalKind::TdmrepWellKnown,
        SignalStatus::Absent,
        Some(404),
        "",
    ),
    (
        "tdmrep_header_1",
        SignalKind::TdmrepHeader,
        SignalStatus::Present,
        Some(200),
        "tdm-reservation: 1\n",
    ),
    (
        "tdmrep_meta_1",
        SignalKind::TdmrepMeta,
        SignalStatus::Present,
        Some(200),
        r#"<meta name="tdm-reservation" content="1">"#,
    ),
    (
        "x_robots_tag_noai",
        SignalKind::XRobotsTag,
        SignalStatus::Present,
        Some(200),
        "X-Robots-Tag: noai\n",
    ),
    (
        "x_robots_tag_noimageai",
        SignalKind::XRobotsTag,
        SignalStatus::Present,
        Some(200),
        "X-Robots-Tag: noimageai\n",
    ),
    (
        "meta_robots_noai",
        SignalKind::MetaRobots,
        SignalStatus::Present,
        Some(200),
        r#"<meta name="robots" content="noai">"#,
    ),
    (
        "meta_robots_noimageai",
        SignalKind::MetaRobots,
        SignalStatus::Present,
        Some(200),
        r#"<meta name="robots" content="noimageai">"#,
    ),
];

/// Expected cells per scenario, columns Search, AiInput, AiTrain, Archive.
pub const VEC_EXPECTED: &[(&str, [Cell; 4])] = &[
    (
        "robots_disallow",
        [(Outcome::Deny, "robots_txt.disallow"); 4],
    ),
    ("robots_5xx", [(Outcome::Deny, "robots_txt.unreachable"); 4]),
    ("robots_4xx", [O_ALLOW; 4]),
    (
        "content_signal_search_no",
        [
            (Outcome::Deny, "robots_txt.content_signal"),
            O_ALLOW,
            O_ALLOW,
            O_ALLOW,
        ],
    ),
    (
        "content_signal_ai_input_no",
        [
            O_ALLOW,
            (Outcome::Deny, "robots_txt.content_signal"),
            O_ALLOW,
            O_ALLOW,
        ],
    ),
    (
        "content_signal_ai_train_no",
        [
            O_ALLOW,
            O_ALLOW,
            (Outcome::Deny, "robots_txt.content_signal"),
            O_ALLOW,
        ],
    ),
    (
        "ai_txt_disallow",
        [
            O_ALLOW,
            (Outcome::Deny, "ai_txt.disallow"),
            (Outcome::Deny, "ai_txt.disallow"),
            O_ALLOW,
        ],
    ),
    ("ai_txt_absent", [O_ALLOW; 4]),
    (
        "tdmrep_well_known_1",
        [
            O_ALLOW,
            (Outcome::Deny, "tdmrep.reservation"),
            (Outcome::Deny, "tdmrep.reservation"),
            O_ALLOW,
        ],
    ),
    ("tdmrep_well_known_absent", [O_ALLOW; 4]),
    (
        "tdmrep_header_1",
        [
            O_ALLOW,
            (Outcome::Deny, "tdmrep.reservation"),
            (Outcome::Deny, "tdmrep.reservation"),
            O_ALLOW,
        ],
    ),
    (
        "tdmrep_meta_1",
        [
            O_ALLOW,
            (Outcome::Deny, "tdmrep.reservation"),
            (Outcome::Deny, "tdmrep.reservation"),
            O_ALLOW,
        ],
    ),
    (
        "x_robots_tag_noai",
        [
            O_ALLOW,
            (Outcome::Deny, "robots_tag.noai"),
            (Outcome::Deny, "robots_tag.noai"),
            O_ALLOW,
        ],
    ),
    (
        "x_robots_tag_noimageai",
        [
            O_ALLOW,
            (Outcome::Deny, "robots_tag.noimageai"),
            (Outcome::Deny, "robots_tag.noimageai"),
            O_ALLOW,
        ],
    ),
    (
        "meta_robots_noai",
        [
            O_ALLOW,
            (Outcome::Deny, "robots_tag.noai"),
            (Outcome::Deny, "robots_tag.noai"),
            O_ALLOW,
        ],
    ),
    (
        "meta_robots_noimageai",
        [
            O_ALLOW,
            (Outcome::Deny, "robots_tag.noimageai"),
            (Outcome::Deny, "robots_tag.noimageai"),
            O_ALLOW,
        ],
    ),
];
