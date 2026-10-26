//! RFC 8785 conformance: published vectors, key ordering, escaping, and number limits.

use crawler_core::{CanonicalEncoder, CoreError};
use serde_json::{Value, json};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// RFC 8785 section 3.2.4: canonical bytes of the section 3.2.2 sample, in hex.
const STR_RFC_SAMPLE_HEX: &str = "7b226c69746572616c73223a5b6e756c6c2c74727565\
2c66616c73655d2c226e756d62657273223a5b3333333333333333332e333333333333332c3165\
2b33302c342e352c302e3030322c31652d32375d2c22737472696e67223a22e282ac245c753030\
30665c6e4127425c225c5c5c5c5c222f227d";

/// The numbers member of the RFC sample; floats are out of scope for receipts.
const STR_RFC_NUMBERS_MEMBER: &str = "\"numbers\":[333333333.3333333,1e+30,4.5,0.002,1e-27],";

/// Encodes a JSON value canonically as text.
fn encode(e_value: &Value) -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::from_utf8(CanonicalEncoder::new().encode(e_value)?)?)
}

/// Decodes a hex string into bytes.
fn decode_hex(str_hex: &str) -> Vec<u8> {
    (0..str_hex.len())
        .step_by(2)
        .filter_map(|u_at| u8::from_str_radix(&str_hex[u_at..u_at + 2], 16).ok())
        .collect()
}

#[test]
fn test_rfc_sample_bytes_match_without_float_member() -> TestResult {
    let str_rfc = String::from_utf8(decode_hex(STR_RFC_SAMPLE_HEX))?;
    assert!(
        str_rfc.contains(STR_RFC_NUMBERS_MEMBER),
        "vector decoded wrongly: {str_rfc}"
    );
    let str_expected = str_rfc.replace(STR_RFC_NUMBERS_MEMBER, "");
    let e_input = json!({
        "string": "\u{20ac}$\u{f}\nA'B\"\\\\\"/",
        "literals": [null, true, false]
    });
    assert_eq!(encode(&e_input)?, str_expected);
    Ok(())
}

#[test]
fn test_rfc_sample_floats_are_rejected() {
    let e_input = json!({"numbers": [333333333.333_333_3, 1e30, 4.5, 0.002, 1e-27]});
    let res_bytes = CanonicalEncoder::new().encode(&e_input);
    assert!(matches!(
        res_bytes,
        Err(CoreError::NonCanonicalNumber { .. })
    ));
}

#[test]
fn test_rfc_sorting_vector() -> TestResult {
    let e_input = json!({
        "\u{20ac}": "Euro Sign",
        "\r": "Carriage Return",
        "\u{fb33}": "Hebrew Letter Dalet With Dagesh",
        "1": "One",
        "\u{1f600}": "Emoji: Grinning Face",
        "\u{80}": "Control",
        "\u{f6}": "Latin Small Letter O With Diaeresis"
    });
    let str_expected = "{\"\\r\":\"Carriage Return\",\"1\":\"One\",\"\u{80}\":\"Control\",\
\"\u{f6}\":\"Latin Small Letter O With Diaeresis\",\"\u{20ac}\":\"Euro Sign\",\
\"\u{1f600}\":\"Emoji: Grinning Face\",\"\u{fb33}\":\"Hebrew Letter Dalet With Dagesh\"}";
    assert_eq!(encode(&e_input)?, str_expected);
    Ok(())
}

#[test]
fn test_non_bmp_keys_sort_by_utf16_not_code_point() -> TestResult {
    let e_input =
        json!({"\u{ffff}": 5, "\u{fb33}": 4, "\u{e000}": 3, "\u{1f600}": 2, "\u{10000}": 1});
    let str_expected =
        "{\"\u{10000}\":1,\"\u{1f600}\":2,\"\u{e000}\":3,\"\u{fb33}\":4,\"\u{ffff}\":5}";
    assert_eq!(encode(&e_input)?, str_expected);
    Ok(())
}

#[test]
fn test_key_sort_edge_cases() -> TestResult {
    let e_input = json!({"b": 0, "aa": 0, "a\u{0}": 0, "a": 0, "A": 0, "9": 0, "10": 0, "": 0});
    let str_expected = "{\"\":0,\"10\":0,\"9\":0,\"A\":0,\"a\":0,\"a\\u0000\":0,\"aa\":0,\"b\":0}";
    assert_eq!(encode(&e_input)?, str_expected);
    Ok(())
}

#[test]
fn test_every_control_character_is_escaped_lowercase() -> TestResult {
    let str_controls: String = (0u8..0x20).map(char::from).collect();
    let mut str_expected = String::from("\"");
    for u_code in 0u8..0x20 {
        let str_escape = match u_code {
            0x08 => String::from("\\b"),
            0x09 => String::from("\\t"),
            0x0a => String::from("\\n"),
            0x0c => String::from("\\f"),
            0x0d => String::from("\\r"),
            _ => format!("\\u{u_code:04x}"),
        };
        str_expected.push_str(&str_escape);
    }
    str_expected.push('"');
    assert_eq!(encode(&json!(str_controls))?, str_expected);
    Ok(())
}

#[test]
fn test_non_control_characters_are_literal() -> TestResult {
    let e_input = json!("/\u{7f}\u{e9}\u{2028}\u{1f600}<>&'");
    assert_eq!(encode(&e_input)?, "\"/\u{7f}\u{e9}\u{2028}\u{1f600}<>&'\"");
    Ok(())
}

#[test]
fn test_nested_structures_have_no_whitespace() -> TestResult {
    let e_input = json!({"b": [{}, [], [null]], "a": {"d": 1, "c": -2}});
    assert_eq!(
        encode(&e_input)?,
        "{\"a\":{\"c\":-2,\"d\":1},\"b\":[{},[],[null]]}"
    );
    Ok(())
}

#[test]
fn test_integer_limits_follow_i_json() -> TestResult {
    assert_eq!(
        encode(&json!([
            9_007_199_254_740_991_u64,
            -9_007_199_254_740_991_i64
        ]))?,
        "[9007199254740991,-9007199254740991]"
    );
    for e_bad in [
        json!(9_007_199_254_740_992_u64),
        json!(-9_007_199_254_740_992_i64),
        json!(u64::MAX),
        json!(1.0),
    ] {
        let res_bytes = CanonicalEncoder::new().encode(&e_bad);
        assert!(
            matches!(res_bytes, Err(CoreError::NonCanonicalNumber { .. })),
            "{e_bad}"
        );
    }
    Ok(())
}
