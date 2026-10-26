# Dependencies

Every direct dependency is recorded here with the facts checked before adding it. Versions are those in `Cargo.lock`.

| Crate | Version | Used by | Why | `rust-version` | Released | Checks |
|---|---|---|---|---|---|---|
| syn | 3.0.9 | convention-check | Parses Rust source for the convention checker (spec 3.7 names `syn`). Features: `full`, `parsing`, `visit`; default features off | 1.71 | 2026-10-12 (14 days before adoption) | crates.io: latest stable, not yanked. RustSec: no advisories (no package page). API read from the 3.0.9 sources in the cargo registry |
| thiserror | 2.0.21 | convention-check | The one error enum per crate (spec 3.5) | 1.77 | 2026-09-23 | crates.io: latest stable, not yanked. RustSec: no advisories (no package page) |
| url | 2.5.8 | crawler-core | URL parsing (WHATWG), IDNA, dot-segment resolution | 1.63 | 2026-01-05 | Not yanked. RustSec: none for `url`. Pulls `idna` 1.1.0, past the RUSTSEC-2024-0421 fix (>=1.0.0) |
| serde | 1.0.229 (`derive`) | crawler-core | Receipt (de)serialization | 1.56 | 2026-07-18 | Not yanked. RustSec: none |
| serde_json | 1.0.154 | crawler-core | JSON values feeding the canonical encoder | 1.71 | 2026-10-11 | Not yanked. RustSec: none |
| sha2 | 0.11.1 | crawler-core | SHA-256 for receipts and bodies | 1.88 | 2026-10-12 | Not yanked. RustSec: only RUSTSEC-2021-0100 (0.9.7 only) |
| uuid | =1.28.0 (`v4`, `serde`) | crawler-core | IDs. Pinned to 1.28.0 because 1.29.0 (2026-10-23) was under 14 days old | 1.89.0 | 2026-10-10 | Not yanked. RustSec: none |
| time | 0.3.55 (`serde`, `formatting`, `parsing`, `std`, `macros`) | crawler-core | UTC instants and the fixed `Timestamp` format. Pulls `time-macros` 0.2.32 | 1.88.0 | 2026-08-01 | Not yanked. RustSec: RUSTSEC-2026-0009 (<0.3.47) and RUSTSEC-2020-0071 (0.1/0.2) do not apply |
| proptest | 1.12.0 (dev) | crawler-core | Property tests for URL normalization | 1.88 | 2026-10-10 | Not yanked. RustSec: none |

Transitive dependencies pulled in by these: `proc-macro2` 1.0.107, `quote` 1.0.47 (2026-07-19), `unicode-ident` 1.0.27 (2026-10-12, license `(MIT OR Apache-2.0) AND Unicode-3.0`), and `thiserror-impl` 2.0.21.

The first lockfile was resolved against a stale local index and picked `quote` 1.0.35 and `unicode-ident` 1.0.12. That `unicode-ident` carries `Unicode-DFS-2016`, which `cargo deny` rejected. Both were updated in F0.1 after checking they are not yanked and are at least 14 days old.

## Supply-chain status

`cargo audit` and `cargo deny check` both pass with no ignored advisories and no license exceptions. The license allow-list holds only the licenses actually present: MIT, Apache-2.0, and Unicode-3.0.

## Toolchain and CI tools

| Item | Version | Notes |
|---|---|---|
| Rust toolchain | 1.99.1 | Latest stable per blog.rust-lang.org (released 2026-10-15). Edition 2024 is the newest edition. |
| cargo-audit | 0.22.2 | Released 2026-06-05, `rust-version` 1.88 |
| cargo-deny | 0.20.2 | Released 2026-07-09, `rust-version` 1.88 |
| actions/cache | v6.1.0 @ `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` | Published 2026-06-26 (GitHub releases API); lightweight tag resolves directly to this commit; inputs `path`/`key` and output `cache-hit` read from `action.yml` at that SHA |
| CI runner image | `ubuntu-24.04` | The image `ubuntu-latest` points to, per actions/runner-images (checked 2026-10-27). `ubuntu-26.04` is also listed; adopting it is a separate change |
| actions/checkout | v7.0.1 @ `3d3c42e5aac5ba805825da76410c181273ba90b1` | Published 2026-07-20 (GitHub releases API); lightweight tag resolves directly to this commit |

## Canonical JSON

No RFC 8785 crate is on the approved list, so `CanonicalEncoder` implements it directly. It is tested against the RFC 8785 section 3.2.3 sorting vector and the section 3.2.4 byte vector (minus its float member). The integer limit of ±(2^53-1) cites RFC 7493 (I-JSON) section 2.2; RFC 8785 section 3.1 requires I-JSON input. Floats are rejected rather than formatted.
