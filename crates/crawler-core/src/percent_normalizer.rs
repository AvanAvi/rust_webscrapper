//! Normalizes percent-encoding in a URL component per RFC 3986 section 6.2.2.

/// Uppercases percent-escape hex digits and decodes escapes of unreserved characters.
#[derive(Debug, Clone, Copy, Default)]
pub struct PercentNormalizer;

impl PercentNormalizer {
    /// Creates a normalizer.
    pub fn new() -> Self {
        Self
    }

    /// Returns the component with normalized percent-encoding; other text is unchanged.
    pub fn normalize(&self, str_input: &str) -> String {
        let mut str_out = String::with_capacity(str_input.len());
        let mut u_index = 0;
        while u_index < str_input.len() {
            u_index += self.copy_unit(str_input, u_index, &mut str_out);
        }
        str_out
    }

    /// Copies one escape triplet or one character at `u_index`; returns bytes consumed.
    fn copy_unit(&self, str_input: &str, u_index: usize, str_out: &mut String) -> usize {
        let Some(u_value) = self.decode_triplet(str_input.as_bytes(), u_index) else {
            return self.copy_char(str_input, u_index, str_out);
        };
        if self.is_unreserved(u_value) {
            str_out.push(char::from(u_value));
        } else {
            str_out.push_str(&format!("%{u_value:02X}"));
        }
        3
    }

    /// Returns the byte encoded by a `%XX` triplet at `u_index`, if one is there.
    fn decode_triplet(&self, by_input: &[u8], u_index: usize) -> Option<u8> {
        if by_input.get(u_index) != Some(&b'%') {
            return None;
        }
        let u_high = self.hex_value(*by_input.get(u_index + 1)?)?;
        let u_low = self.hex_value(*by_input.get(u_index + 2)?)?;
        Some(u_high * 16 + u_low)
    }

    /// Returns the value of one hex digit, if it is one.
    fn hex_value(&self, u_byte: u8) -> Option<u8> {
        char::from(u_byte)
            .to_digit(16)
            .and_then(|u_digit| u8::try_from(u_digit).ok())
    }

    /// Copies the whole character starting at `u_index`; returns its length in bytes.
    fn copy_char(&self, str_input: &str, u_index: usize, str_out: &mut String) -> usize {
        let Some(ch_next) = str_input
            .get(u_index..)
            .and_then(|str_rest| str_rest.chars().next())
        else {
            return 1;
        };
        str_out.push(ch_next);
        ch_next.len_utf8()
    }

    /// Returns whether a byte is an RFC 3986 unreserved character.
    fn is_unreserved(&self, u_byte: u8) -> bool {
        u_byte.is_ascii_alphanumeric() || matches!(u_byte, b'-' | b'.' | b'_' | b'~')
    }
}
