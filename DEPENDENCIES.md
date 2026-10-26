# Dependencies

Every direct dependency is recorded here with the facts checked before adding it. Versions are those in `Cargo.lock`.

| Crate | Version | Used by | Why | `rust-version` | Released | Checks |
|---|---|---|---|---|---|---|
| syn | 3.0.9 | convention-check | Parses Rust source for the convention checker (spec 3.7 names `syn`). Features: `full`, `parsing`, `visit`; default features off | 1.71 | 2026-10-12 (14 days before adoption) | crates.io: latest stable, not yanked. RustSec: no advisories (no package page). API read from the 3.0.9 sources in the cargo registry |
| thiserror | 2.0.21 | convention-check | The one error enum per crate (spec 3.5) | 1.77 | 2026-09-23 | crates.io: latest stable, not yanked. RustSec: no advisories (no package page) |

Transitive dependencies pulled in by these: `proc-macro2`, `quote`, `unicode-ident`, and `thiserror-impl`.

## Toolchain and CI tools

| Item | Version | Notes |
|---|---|---|
| Rust toolchain | 1.99.1 | Latest stable per blog.rust-lang.org (released 2026-10-15). Edition 2024 is the newest edition. |
| cargo-audit (CI only) | 0.22.2 | Released 2026-06-05, `rust-version` 1.88 |
| cargo-deny (CI only) | 0.20.2 | Released 2026-07-09, `rust-version` 1.88 |
| actions/checkout | v7 | Latest major tag |
