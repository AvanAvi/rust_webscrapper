# proofcrawl: working notes for Claude

Self-hostable Rust crawler whose main output is evidence: one signed receipt per considered URL in a tamper-evident ledger. The full specification lives with the project owner. Build one feature (`F0`, `F1`, …) at a time, end each with the Gate Report, then stop.

## Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p convention-check          # checks the whole tree, exit 1 on violations
./scripts/lint-probe.sh                         # proves every configured lint fires
cargo test -p crawler-fetch --features test-policy --locked   # mock-site scenarios (loopback policy)
./scripts/check-no-test-policy.sh               # crawlerd never enables test-policy
./scripts/check-no-test-policy-selftest.sh      # proves that check fails on a binary enabling it
./scripts/check-fetch-deps.sh                   # axum never enters crawler-fetch's normal deps
cargo audit && cargo deny check
```

Everything runs on the toolchain pinned in `rust-toolchain.toml` (1.99.1). Never override it with `RUSTUP_TOOLCHAIN`.

Local Postgres for F6 onward: see `docs/DEVELOPMENT.md` (`scripts/dev-db.sh`; any reachable Postgres via `DATABASE_URL` works).

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
- Commit messages contain only the owner's message text: no Co-Authored-By trailer, no Claude/Anthropic mention, author and committer = the configured git identity (attribution disabled in the gitignored `.claude/settings.local.json`).
- Prefer rustls. `deny.toml` bans `openssl`, `openssl-sys`, and `native-tls`.

## Feature notes

### F5 (SignalCollector)

- Signal-file bodies (robots.txt, ai.txt, `/.well-known/tdmrep.json`) must be **stored keyed
  by their content hash**, not only hashed. A receipt's `signals[].content_sha256` must
  resolve to the exact bytes that were parsed, so it can be re-checked and re-parsed later.
  Store each distinct body once (content-addressed), cap its size like the parsers do, and
  keep it as long as any receipt references it.

## Policy notes

`docs/POLICY.md` records every interpretation that needs legal review. Update it whenever a
parser or the policy evaluator makes a judgement call.

## Layout

`crates/`: `crawler-core`, `crawler-fetch`, `crawler-ledger`, `crawler-store`, `crawler-server`, `crawler-worker`, `crawler-export`, `crawler-cli` (binary `crawlerd`), `mock-site`, `convention-check`.
