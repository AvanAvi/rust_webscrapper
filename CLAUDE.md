# proofcrawl: working notes for Claude

Self-hostable Rust crawler whose main output is evidence: one signed receipt per considered URL in a tamper-evident ledger. The full specification lives with the project owner. Build one feature (`F0`, `F1`, …) at a time, end each with the Gate Report, then stop.

## Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p convention-check          # checks the whole tree, exit 1 on violations
./scripts/lint-probe.sh                         # proves every configured lint fires
cargo audit && cargo deny check
```

Everything runs on the toolchain pinned in `rust-toolchain.toml` (1.99.1). Never override it with `RUSTUP_TOOLCHAIN`.

## Convention summary (full text: CONVENTIONS.md)

- All logic lives in methods on structs. Free functions are allowed only for `main` and test code. Traits are used for injection (`Clock`, `IdSource`, `HttpFetcher`, `LedgerStore`, `SignalParser`).
- Every variable, param, field, and const carries a Hungarian prefix for its type (`b_ i_ u_ f_ ch_ str_ by_ e_ o_ vec_ map_ set_ opt_ res_ err_ dt_ dur_ uuid_ path_ url_ tx_ rx_ mtx_ jh_ cb_`). Consts use the SCREAMING form (`U_MAX_BODY_BYTES`).
- serde and sqlx fields always carry `rename` to a clean, unprefixed name.
- Limits: ≤25 lines per method, ≤4 params besides `self`, cognitive complexity ≤8, ≤300 lines per file, one loop level per method.
- No `unwrap`/`expect`/`panic!`/`todo!` outside tests. No `unsafe` (`#![forbid(unsafe_code)]`). One `thiserror` enum per crate.
- `///` on every type, field, const, and method. `//!` at the top of every module.
- Time and randomness come only through injected `Clock`/`IdSource`.
- Never log secrets, bodies, or full query strings.

## Rules of engagement

- Only approved dependencies. Before adding one: check crates.io (current version, `rust-version`, not yanked, ≥14 days old), RustSec, and docs.rs for the exact locked version, then record it in `DEPENDENCIES.md`.
- Tests use only the local `mock-site`, never real third-party sites.
- Prefer rustls. `deny.toml` bans `openssl`, `openssl-sys`, and `native-tls`.

## Layout

`crates/`: `crawler-core`, `crawler-fetch`, `crawler-ledger`, `crawler-store`, `crawler-server`, `crawler-worker`, `crawler-export`, `crawler-cli` (binary `crawlerd`), `mock-site`, `convention-check`.
