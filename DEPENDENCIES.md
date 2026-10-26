# Dependencies

Every direct dependency is recorded here with the facts checked before adding it. Versions are those in `Cargo.lock`.

| Crate | Version | Used by | Why | `rust-version` | Released | Checks |
|---|---|---|---|---|---|---|
| syn | 3.0.9 | convention-check | Parses Rust source for the convention checker (spec 3.7 names `syn`). Features: `full`, `parsing`, `visit`; default features off | 1.71 | 2026-10-12 (14 days before adoption) | crates.io: latest stable, not yanked. RustSec: no advisories (no package page). API read from the 3.0.9 sources in the cargo registry |
| thiserror | 2.0.21 | convention-check | The one error enum per crate (spec 3.5) | 1.77 | 2026-09-23 | crates.io: latest stable, not yanked. RustSec: no advisories (no package page) |

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
| actions/checkout | v7.0.1 @ `3d3c42e5aac5ba805825da76410c181273ba90b1` | Published 2026-07-20 (GitHub releases API); lightweight tag resolves directly to this commit |
