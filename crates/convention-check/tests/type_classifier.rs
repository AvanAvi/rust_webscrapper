//! Table-driven tests for the type-to-prefix mapping.

use std::collections::HashSet;

use convention_check::{TypeClassifier, TypeScope};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Returns the prefix the classifier picks for a type written as source text.
fn classify(str_ty: &str) -> Result<Option<&'static str>, syn::Error> {
    let e_ty: syn::Type = syn::parse_str(str_ty)?;
    let set_str_enums = HashSet::from([String::from("Purpose")]);
    Ok(TypeClassifier::new(set_str_enums).expected_for_type(&e_ty, &TypeScope::new()))
}

/// Types written as source text and the prefix each must map to.
const VEC_CASES: &[(&str, &str)] = &[
    ("bool", "b_"),
    ("i64", "i_"),
    ("usize", "u_"),
    ("f64", "f_"),
    ("char", "ch_"),
    ("&str", "str_"),
    ("&'static str", "str_"),
    ("String", "str_"),
    ("Cow<'static, str>", "str_"),
    ("Vec<u8>", "by_"),
    ("&[u8]", "by_"),
    ("[u8; 32]", "by_"),
    ("bytes::Bytes", "by_"),
    ("Vec<String>", "vec_"),
    ("&[Receipt]", "vec_"),
    ("HashMap<String, u32>", "map_"),
    ("serde_json::Map<String, Value>", "map_"),
    ("Timestamp", "dt_"),
    ("BTreeSet<String>", "set_"),
    ("Option<u32>", "opt_"),
    ("Result<u32, Error>", "res_"),
    ("std::io::Error", "err_"),
    ("Box<dyn std::error::Error>", "err_"),
    ("time::OffsetDateTime", "dt_"),
    ("Duration", "dur_"),
    ("Uuid", "uuid_"),
    ("&Path", "path_"),
    ("PathBuf", "path_"),
    ("url::Url", "url_"),
    ("mpsc::Sender<u8>", "tx_"),
    ("Receiver<u8>", "rx_"),
    ("Arc<Mutex<u32>>", "mtx_"),
    ("JoinHandle<()>", "jh_"),
    ("Box<dyn Fn(u8) -> u8>", "cb_"),
    ("impl FnMut()", "cb_"),
    ("fn(u8) -> u8", "cb_"),
    ("Purpose", "e_"),
    ("std::cmp::Ordering", "e_"),
    ("Arc<Receipt>", "o_"),
    ("&dyn LedgerStore", "o_"),
    ("Receipt", "o_"),
];

#[test]
fn test_known_types_map_to_expected_prefix() -> TestResult {
    for (str_ty, str_expected) in VEC_CASES.iter().copied() {
        assert_eq!(classify(str_ty)?, Some(str_expected), "type `{str_ty}`");
    }
    Ok(())
}

#[test]
fn test_unknown_shapes_accept_any_prefix() -> TestResult {
    for str_ty in ["(u8, String)", "impl Iterator<Item = u8>", "_"] {
        assert_eq!(classify(str_ty)?, None, "type `{str_ty}`");
    }
    Ok(())
}
