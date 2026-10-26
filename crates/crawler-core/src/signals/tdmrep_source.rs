//! Lists where a TDMRep signal can be published.

/// Publication method of a TDMRep signal (W3C TDMRep CG final report, 2024-05-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TdmrepSource {
    /// `tdm-reservation` / `tdm-policy` HTTP response headers.
    Header,
    /// `<meta name="tdm-reservation">` / `<meta name="tdm-policy">` elements.
    Meta,
    /// `/.well-known/tdmrep.json` rule file.
    WellKnown,
}
