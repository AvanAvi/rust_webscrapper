//! Answers questions about item attributes such as docs, derives, and cfg flags.

use syn::{Attribute, Meta};

/// Reads attribute lists without interpreting their meaning further.
#[derive(Debug, Clone, Copy, Default)]
pub struct AttrInspector;

impl AttrInspector {
    /// Creates an inspector.
    pub fn new() -> Self {
        Self
    }

    /// Returns whether the attributes include a doc comment.
    pub fn has_doc(&self, vec_o_attrs: &[Attribute]) -> bool {
        vec_o_attrs
            .iter()
            .any(|o_attr| o_attr.path().is_ident("doc"))
    }

    /// Returns whether the attributes include a `cfg(...)` mentioning `test`.
    pub fn is_cfg_test(&self, vec_o_attrs: &[Attribute]) -> bool {
        self.has_tool_word(vec_o_attrs, "cfg", "test")
    }

    /// Returns whether the attributes mark a test function, such as `#[test]` or `#[tokio::test]`.
    pub fn is_test_fn(&self, vec_o_attrs: &[Attribute]) -> bool {
        vec_o_attrs
            .iter()
            .filter_map(|o_attr| o_attr.path().segments.last())
            .any(|o_seg| o_seg.ident == "test")
    }

    /// Returns whether a `derive` attribute names the given trait.
    pub fn derives(&self, vec_o_attrs: &[Attribute], str_trait: &str) -> bool {
        self.has_tool_word(vec_o_attrs, "derive", str_trait)
    }

    /// Returns whether a serde field is renamed, skipped, or flattened.
    pub fn has_serde_rename(&self, vec_o_attrs: &[Attribute]) -> bool {
        ["rename", "skip", "flatten"]
            .iter()
            .any(|str_word| self.has_tool_word(vec_o_attrs, "serde", str_word))
    }

    /// Returns whether serde never sees the fields: `transparent`, or `into` plus `from`/`try_from`.
    pub fn is_serde_proxied(&self, vec_o_attrs: &[Attribute]) -> bool {
        let b_into = self.has_tool_word(vec_o_attrs, "serde", "into");
        let b_from = ["from", "try_from"]
            .iter()
            .any(|str_word| self.has_tool_word(vec_o_attrs, "serde", str_word));
        self.has_tool_word(vec_o_attrs, "serde", "transparent") || (b_into && b_from)
    }

    /// Returns whether a `tool(...)` attribute contains a given word.
    pub fn has_tool_word(&self, vec_o_attrs: &[Attribute], str_tool: &str, str_word: &str) -> bool {
        vec_o_attrs
            .iter()
            .filter(|o_attr| o_attr.path().is_ident(str_tool))
            .any(|o_attr| self.has_word(o_attr, str_word))
    }

    /// Returns whether a list attribute's tokens contain a whole word.
    fn has_word(&self, o_attr: &Attribute, str_word: &str) -> bool {
        let Meta::List(o_list) = &o_attr.meta else {
            return false;
        };
        let str_tokens = o_list.tokens.to_string();
        str_tokens
            .split(|ch_c: char| !(ch_c.is_alphanumeric() || ch_c == '_'))
            .any(|str_part| str_part == str_word)
    }
}
