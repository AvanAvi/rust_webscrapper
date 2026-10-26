//! Proves the checker flags each rule on bad input and accepts clean input.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use convention_check::{ConventionChecker, ParsedSource, Violation, ViolationKind};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Expected violation count per kind for `bad_conventions.rs`.
const VEC_BAD_COUNTS: &[(ViolationKind, usize)] = &[
    (ViolationKind::MissingPrefix, 10),
    (ViolationKind::WrongPrefix, 5),
    (ViolationKind::MissingRename, 5),
    (ViolationKind::MissingDoc, 4),
    (ViolationKind::FreeFunction, 1),
    (ViolationKind::TestName, 1),
    (ViolationKind::NestingTooDeep, 2),
    (ViolationKind::NestedLoop, 1),
];

/// Offenders in `bad_conventions.rs` that must each be named by a violation.
const VEC_BAD_OFFENDERS: &[(ViolationKind, &str)] = &[
    (ViolationKind::MissingPrefix, "max_items"),
    (ViolationKind::MissingPrefix, "`count`"),
    (ViolationKind::MissingPrefix, "`value`"),
    (ViolationKind::MissingPrefix, "`doubled`"),
    (ViolationKind::MissingPrefix, "`name`"),
    (ViolationKind::MissingPrefix, "`n`"),
    (ViolationKind::MissingPrefix, "`size`"),
    (ViolationKind::MissingPrefix, "`limit`"),
    (ViolationKind::MissingPrefix, "`found`"),
    (ViolationKind::MissingPrefix, "`x`"),
    (ViolationKind::WrongPrefix, "b_flag"),
    (ViolationKind::WrongPrefix, "STR_LIMIT"),
    (ViolationKind::WrongPrefix, "str_total"),
    (ViolationKind::WrongPrefix, "u_id"),
    (ViolationKind::WrongPrefix, "str_count"),
    (ViolationKind::MissingRename, "u_undocumented"),
    (ViolationKind::MissingRename, "sqlx field `u_id`"),
    (ViolationKind::MissingRename, "serde field `str_text`"),
    (ViolationKind::MissingDoc, "Undocumented"),
    (ViolationKind::MissingDoc, "total"),
    (ViolationKind::FreeFunction, "helper"),
    (ViolationKind::TestName, "checks_total"),
    (ViolationKind::NestingTooDeep, "deep_if"),
    (ViolationKind::NestingTooDeep, "deep_closure"),
    (ViolationKind::NestedLoop, "nested_loop"),
];

/// Checks a fixture's text reported under a non-test path, so every rule applies.
fn check_fixture(str_name: &str) -> Result<Vec<Violation>, Box<dyn std::error::Error>> {
    let path_fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(str_name);
    let str_text = fs::read_to_string(&path_fixture)?;
    Ok(check_text(&format!("src/{str_name}"), &str_text)?)
}

/// Checks one in-memory source reported under the given path.
fn check_text(
    str_path: &str,
    str_text: &str,
) -> Result<Vec<Violation>, convention_check::CheckError> {
    let o_source = ParsedSource::from_text(PathBuf::from(str_path), str_text)?;
    Ok(ConventionChecker::new().check_sources(&[o_source]))
}

/// Counts violations per kind.
fn count_kinds(vec_o_violations: &[Violation]) -> BTreeMap<ViolationKind, usize> {
    let mut map_u_counts = BTreeMap::new();
    for o_violation in vec_o_violations {
        *map_u_counts.entry(o_violation.kind()).or_insert(0) += 1;
    }
    map_u_counts
}

/// Returns whether any violation of a kind mentions a name.
fn has_violation(vec_o_violations: &[Violation], e_kind: ViolationKind, str_name: &str) -> bool {
    vec_o_violations.iter().any(|o_v| {
        o_v.kind() == e_kind
            && (o_v.detail().contains(str_name) || o_v.location().ends_with(str_name))
    })
}

#[test]
fn test_bad_fixture_reports_exact_counts_per_kind() -> TestResult {
    let vec_o_violations = check_fixture("bad_conventions.rs")?;
    let map_u_expected: BTreeMap<_, _> = VEC_BAD_COUNTS.iter().copied().collect();
    assert_eq!(
        count_kinds(&vec_o_violations),
        map_u_expected,
        "{vec_o_violations:#?}"
    );
    Ok(())
}

#[test]
fn test_bad_fixture_names_each_offender() -> TestResult {
    let vec_o_violations = check_fixture("bad_conventions.rs")?;
    for (e_kind, str_name) in VEC_BAD_OFFENDERS.iter().copied() {
        let b_found = has_violation(&vec_o_violations, e_kind, str_name);
        assert!(b_found, "missing {e_kind:?} for {str_name}");
    }
    Ok(())
}

#[test]
fn test_good_fixture_is_clean() -> TestResult {
    let vec_o_violations = check_fixture("good_conventions.rs")?;
    assert!(
        vec_o_violations.is_empty(),
        "unexpected: {vec_o_violations:#?}"
    );
    Ok(())
}

#[test]
fn test_file_over_limit_is_reported() -> TestResult {
    let str_text = "//! Long file.\n".to_owned() + &"// filler\n".repeat(300);
    let vec_o_violations = check_text("src/long.rs", &str_text)?;
    let map_u_expected = BTreeMap::from([(ViolationKind::FileTooLong, 1)]);
    assert_eq!(count_kinds(&vec_o_violations), map_u_expected);
    Ok(())
}

#[test]
fn test_file_at_limit_is_accepted() -> TestResult {
    let str_text = "//! Long file.\n".to_owned() + &"// filler\n".repeat(299);
    assert!(check_text("src/long.rs", &str_text)?.is_empty());
    Ok(())
}

#[test]
fn test_main_allowed_only_in_entry_files() -> TestResult {
    let str_text = "//! Entry.\nfn main() {}\n";
    assert!(check_text("src/main.rs", str_text)?.is_empty());
    let vec_o_violations = check_text("src/lib.rs", str_text)?;
    assert!(has_violation(
        &vec_o_violations,
        ViolationKind::FreeFunction,
        "main"
    ));
    Ok(())
}

#[test]
fn test_integration_test_files_allow_helpers() -> TestResult {
    let str_text =
        "//! Tests.\nfn helper() -> u32 { 1 }\n#[test]\nfn test_one() { let u_x = helper(); }\n";
    assert!(check_text("tests/helpers.rs", str_text)?.is_empty());
    Ok(())
}

#[test]
fn test_test_name_enforced_in_test_files() -> TestResult {
    let str_text = "//! Tests.\n#[test]\nfn one() {}\n";
    let vec_o_violations = check_text("tests/naming.rs", str_text)?;
    assert!(has_violation(
        &vec_o_violations,
        ViolationKind::TestName,
        "one"
    ));
    Ok(())
}

#[test]
fn test_workspace_tree_is_clean() -> TestResult {
    let path_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let o_checker = ConventionChecker::new();
    let vec_path_files = o_checker.collect_files(&path_root)?;
    let b_any_fixture = vec_path_files.iter().any(|path_f| {
        path_f
            .components()
            .any(|e_part| e_part.as_os_str() == "fixtures")
    });
    assert!(!b_any_fixture, "fixture directories must be skipped");
    let vec_o_violations = o_checker.check_files(&vec_path_files)?;
    assert!(
        vec_o_violations.is_empty(),
        "workspace violations: {vec_o_violations:#?}"
    );
    Ok(())
}

#[test]
fn test_nested_function_bodies_reset_depth() -> TestResult {
    let str_text = "//! Depth.\n/// T.\npub struct T;\nimpl T {\n    /// M.\n    pub fn m(&self) {\n        if true {\n            if true {\n                /// I.\n                struct I;\n                impl I {\n                    /// N.\n                    fn n(&self) {\n                        if true {}\n                    }\n                }\n            }\n        }\n    }\n}\n";
    let vec_o_violations = check_text("src/depth.rs", str_text)?;
    assert!(vec_o_violations.is_empty(), "{vec_o_violations:#?}");
    Ok(())
}

#[test]
fn test_loop_inside_closure_inside_loop_is_nested() -> TestResult {
    let str_text = "//! Loops.\n/// T.\npub struct T;\nimpl T {\n    /// M.\n    pub fn m(&self, vec_u_items: &[u32]) {\n        for u_a in vec_u_items {\n            vec_u_items.iter().for_each(|u_b| loop {});\n        }\n    }\n}\n";
    let vec_o_violations = check_text("src/loops.rs", str_text)?;
    let map_u_counts = count_kinds(&vec_o_violations);
    assert_eq!(
        map_u_counts.get(&ViolationKind::NestedLoop),
        Some(&1),
        "{vec_o_violations:#?}"
    );
    assert_eq!(
        map_u_counts.get(&ViolationKind::NestingTooDeep),
        Some(&1),
        "{vec_o_violations:#?}"
    );
    Ok(())
}
