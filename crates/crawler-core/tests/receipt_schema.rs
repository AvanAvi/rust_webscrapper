//! Schema fingerprint: a golden hash of everything a receipt commits to. Four sections:
//! the receipt's key paths with their JSON types, the stored signal observation's key paths (one
//! per parsed format), the signal set record as `SignalSetArchive` writes it, and every enum
//! value name. Renaming or retyping a field, adding or removing one, or renaming an enum value
//! changes the fingerprint; the test then fails until `U_RECEIPT_SCHEMA_VERSION` is bumped and
//! the golden pair below is updated together.

#[path = "schema_support/enum_values.rs"]
mod enum_values;
#[path = "schema_support/maximal.rs"]
mod maximal;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crawler_core::{
    BodyArchive, CanonicalEncoder, Hasher, MemoryBodyArchive, SignalSetArchive,
    U_RECEIPT_SCHEMA_VERSION,
};
use maximal::BoxError;
use serde::Serialize;
use serde_json::Value;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Key paths and the JSON types seen at each.
type FieldMap = BTreeMap<String, BTreeSet<&'static str>>;

/// Schema version and SHA-256 of the fingerprint text it was frozen with.
const O_GOLDEN_FINGERPRINT: (u32, &str) = (
    3,
    "5febadfb426ce8b6cca895277f3b6f4d308722554d6652f9c8c0485c82b29c16",
);

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

/// Returns one section: a title line and one `path: types` line per key path of the values.
fn section<T: Serialize>(str_title: &str, vec_o_values: &[T]) -> Result<String, BoxError> {
    let mut o_fields = FieldMap::new();
    for o_value in vec_o_values {
        let e_root: Value = serde_json::from_slice(&CanonicalEncoder::new().encode(o_value)?)?;
        walk("$", &e_root, &mut o_fields);
    }
    let str_lines: String = o_fields
        .iter()
        .map(|(str_path, set_types)| {
            let vec_str_types: Vec<&str> = set_types.iter().copied().collect();
            format!("{str_path}: {}\n", vec_str_types.join("|"))
        })
        .collect();
    Ok(format!("[{str_title}]\n{str_lines}"))
}

/// Returns the set record exactly as `SignalSetArchive` stores it.
fn set_record() -> Result<Value, BoxError> {
    let o_archive = Arc::new(MemoryBodyArchive::new());
    let str_set_id = SignalSetArchive::new(o_archive.clone()).store(&maximal::observations()?)?;
    let by_record = o_archive.load(&str_set_id).ok_or("set record stored")?;
    Ok(serde_json::from_slice(&by_record)?)
}

/// Returns the full fingerprint text.
fn fingerprint_text() -> Result<String, BoxError> {
    Ok([
        section("receipt", &maximal::receipts()?)?,
        section("signal_observation", &maximal::observations()?)?,
        section("signal_set_record", &[set_record()?])?,
        format!("[enums]\n{}", enum_values::enum_lines()?),
    ]
    .concat())
}

#[test]
fn test_receipt_schema_matches_its_version() -> TestResult {
    let str_text = fingerprint_text()?;
    let str_hash = Hasher::new().sha256_hex(str_text.as_bytes());
    assert_eq!(
        (U_RECEIPT_SCHEMA_VERSION, str_hash.as_str()),
        O_GOLDEN_FINGERPRINT,
        "\nThe receipt schema changed without a schema_version bump and fingerprint update.\n\
         Bump U_RECEIPT_SCHEMA_VERSION in crates/crawler-core/src/receipt.rs, document the change \
         in docs/RECEIPT_SCHEMA.md, then set O_GOLDEN_FINGERPRINT to ({}, \"{str_hash}\").\n\
         Current fingerprint text:\n{str_text}",
        U_RECEIPT_SCHEMA_VERSION + u32::from(U_RECEIPT_SCHEMA_VERSION == O_GOLDEN_FINGERPRINT.0),
    );
    Ok(())
}

#[test]
fn test_maximal_samples_fill_every_optional_field() -> TestResult {
    let str_text = fingerprint_text()?;
    let vec_str_only_null: Vec<&str> = str_text
        .lines()
        .filter(|str_line| str_line.ends_with(": null"))
        .collect();
    assert!(
        vec_str_only_null.is_empty(),
        "fields only ever null hide their type: {vec_str_only_null:?}"
    );
    Ok(())
}
