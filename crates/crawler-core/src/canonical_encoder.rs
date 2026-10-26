//! Encodes values as RFC 8785 (JSON Canonicalization Scheme) bytes.

use std::cmp::Ordering;

use serde::Serialize;
use serde_json::{Map, Number, Value};

use crate::core_error::CoreError;

/// Largest exactly interchangeable integer magnitude, 2^53-1. RFC 8785 section 3.1 requires
/// I-JSON input; RFC 7493 section 2.2 limits exact integers to [-(2^53)+1, (2^53)-1].
const U_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Produces deterministic JSON: no whitespace, object keys sorted by UTF-16 code units,
/// RFC 8785 string escaping. Receipts contain no floating-point values, so floats and
/// integers outside ±(2^53-1) are rejected instead of being formatted as doubles.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonicalEncoder;

impl CanonicalEncoder {
    /// Creates an encoder.
    pub fn new() -> Self {
        Self
    }

    /// Returns the canonical JSON bytes of any serializable value.
    pub fn encode<T: Serialize>(&self, o_value: &T) -> Result<Vec<u8>, CoreError> {
        let e_json = serde_json::to_value(o_value)?;
        let mut str_out = String::new();
        self.write_value(&e_json, &mut str_out)?;
        Ok(str_out.into_bytes())
    }

    /// Appends one JSON value.
    fn write_value(&self, e_json: &Value, str_out: &mut String) -> Result<(), CoreError> {
        match e_json {
            Value::Null => str_out.push_str("null"),
            Value::Bool(b_value) => str_out.push_str(if *b_value { "true" } else { "false" }),
            Value::Number(o_number) => self.write_number(o_number, str_out)?,
            Value::String(str_value) => self.write_string(str_value, str_out),
            Value::Array(vec_e_items) => self.write_array(vec_e_items, str_out)?,
            Value::Object(map_e_fields) => self.write_object(map_e_fields, str_out)?,
        }
        Ok(())
    }

    /// Appends an integer within the safe range, or fails for anything else.
    fn write_number(&self, o_number: &Number, str_out: &mut String) -> Result<(), CoreError> {
        let opt_u_magnitude = o_number
            .as_u64()
            .or_else(|| o_number.as_i64().map(i64::unsigned_abs));
        match opt_u_magnitude {
            Some(u_magnitude) if u_magnitude <= U_MAX_SAFE_INTEGER => {
                str_out.push_str(&o_number.to_string());
                Ok(())
            }
            _ => Err(CoreError::NonCanonicalNumber {
                str_number: o_number.to_string(),
            }),
        }
    }

    /// Appends a quoted string with RFC 8785 escaping.
    fn write_string(&self, str_value: &str, str_out: &mut String) {
        str_out.push('"');
        for ch_next in str_value.chars() {
            self.write_char(ch_next, str_out);
        }
        str_out.push('"');
    }

    /// Appends one character, escaping quotes, backslashes, and control characters.
    fn write_char(&self, ch_next: char, str_out: &mut String) {
        match ch_next {
            '"' => str_out.push_str("\\\""),
            '\\' => str_out.push_str("\\\\"),
            '\u{8}' => str_out.push_str("\\b"),
            '\t' => str_out.push_str("\\t"),
            '\n' => str_out.push_str("\\n"),
            '\u{c}' => str_out.push_str("\\f"),
            '\r' => str_out.push_str("\\r"),
            ch_ctrl if u32::from(ch_ctrl) < 0x20 => {
                str_out.push_str(&format!("\\u{:04x}", u32::from(ch_ctrl)));
            }
            _ => str_out.push(ch_next),
        }
    }

    /// Appends an array with comma-separated canonical items.
    fn write_array(&self, vec_e_items: &[Value], str_out: &mut String) -> Result<(), CoreError> {
        str_out.push('[');
        for (u_index, e_item) in vec_e_items.iter().enumerate() {
            if u_index > 0 {
                str_out.push(',');
            }
            self.write_value(e_item, str_out)?;
        }
        str_out.push(']');
        Ok(())
    }

    /// Appends an object with members sorted by the UTF-16 code units of their keys.
    fn write_object(
        &self,
        map_e_fields: &Map<String, Value>,
        str_out: &mut String,
    ) -> Result<(), CoreError> {
        let mut vec_o_members: Vec<(&String, &Value)> = map_e_fields.iter().collect();
        vec_o_members.sort_by(|o_left, o_right| self.compare_utf16(o_left.0, o_right.0));
        str_out.push('{');
        for (u_index, (str_key, e_item)) in vec_o_members.into_iter().enumerate() {
            if u_index > 0 {
                str_out.push(',');
            }
            self.write_string(str_key, str_out);
            str_out.push(':');
            self.write_value(e_item, str_out)?;
        }
        str_out.push('}');
        Ok(())
    }

    /// Orders two keys by their UTF-16 code units, as RFC 8785 requires.
    fn compare_utf16(&self, str_left: &str, str_right: &str) -> Ordering {
        str_left.encode_utf16().cmp(str_right.encode_utf16())
    }
}
