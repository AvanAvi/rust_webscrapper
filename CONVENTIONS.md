# Coding convention

This is section 3 of the project specification, copied verbatim. It is enforced by `clippy.toml`, the workspace lints, and `cargo run -p convention-check`.

## 3. Coding convention (strict, enforced by tooling)

3.1 Object orientation. All logic lives in methods on structs, with traits for polymorphism and dependency injection (`Clock`, `HttpFetcher`, `LedgerStore`, `SignalParser`, and so on). Free functions are allowed only for `main` and `#[cfg(test)]` helpers. No global mutable state. Constants only as `const`. Composition over inheritance. Prefer one struct per file, file named after the type in snake_case.

3.2 Hungarian prefixes. Every variable, parameter, struct field, and constant carries a prefix for its kind, in snake_case. Containers stack at most one element prefix (`vec_str_urls`, `opt_u_limit`). Methods are verb-first snake_case with no prefix; predicates start with `is_`, `has_`, or `can_`.

| Prefix | Kind | Example |
|---|---|---|
| `b_` | bool | `b_is_allowed` |
| `i_` | signed integer | `i_delta` |
| `u_` | unsigned integer, usize | `u_body_bytes` |
| `f_` | float | `f_ratio` |
| `ch_` | char | `ch_sep` |
| `str_` | String and &str | `str_url` |
| `by_` | bytes (Vec<u8>, &[u8], Bytes, [u8; N]) | `by_body` |
| `e_` | enum value | `e_purpose` |
| `o_` | struct or trait-object instance (Arc is transparent) | `o_receipt` |
| `vec_` | Vec or slice | `vec_o_signals` |
| `map_` | map, prefix then value kind; key type goes in the doc comment | `map_o_hosts` |
| `set_` | set | `set_str_hosts` |
| `opt_` | Option | `opt_str_reason` |
| `res_` | Result | `res_o_fetch` |
| `err_` | error value | `err_fetch` |
| `dt_` | date-time | `dt_fetched_at` |
| `dur_` | Duration | `dur_timeout` |
| `uuid_` | Uuid | `uuid_job` |
| `path_` | path | `path_key_file` |
| `url_` | parsed Url | `url_final` |
| `tx_` `rx_` | channel ends | `tx_work` |
| `mtx_` | Mutex or RwLock | `mtx_cache` |
| `jh_` | JoinHandle | `jh_worker` |
| `cb_` | closure | `cb_filter` |

Constants use the SCREAMING_SNAKE form of the prefix: `U_MAX_BODY_BYTES`, `STR_USER_AGENT_PREFIX`, `DUR_CONNECT_TIMEOUT`.

3.3 Type names stay UpperCamelCase without prefixes (compiler naming lints). Enum variants are UpperCamelCase. Test functions start with `test_`.

3.4 Serialization names. Prefixed field names must not leak into JSON or SQL. Every serde field gets `#[serde(rename = "clean_name")]` and every sqlx `FromRow` field gets `#[sqlx(rename = "clean_name")]`. JSON and SQL names are plain snake_case without prefixes.

3.5 Readability limits.
- A method has at most 25 lines, at most 4 parameters besides `self`, nesting depth at most 2, and cognitive complexity at most 8.
- At most one loop level per method. A second loop becomes its own method with a verb name.
- Use early returns. Each `match` arm is at most 3 lines and calls a method when it needs more.
- A file has at most 300 lines.
- No `unwrap`, `expect`, `panic!`, or `todo!` outside tests (clippy `deny`).
- One `thiserror` error enum per crate.
- Iterator chains are allowed when they read in one glance (at most 3 adapters). Otherwise write a named method.

3.6 Comments. Every struct, enum, trait, method, constant, and field has a one-line `///` doc comment that begins with a verb ("Parses...", "Returns...") for methods, or a noun phrase for types and fields. Each module starts with one `//!` line. Use `//` only for a non-obvious reason ("why", never "what"). Example of the expected style:

```rust
/// Evaluates observed signals into a crawl decision for one purpose.
pub struct PolicyEvaluator {
    /// Purpose this crawler declares for the whole job.
    e_purpose: Purpose,
    /// Version label stored in every decision.
    str_policy_version: String,
}

impl PolicyEvaluator {
    /// Creates an evaluator for a purpose and policy version.
    pub fn new(e_purpose: Purpose, str_policy_version: String) -> Self {
        Self { e_purpose, str_policy_version }
    }

    /// Returns the decision for one URL given its observed signals.
    pub fn evaluate(&self, o_signals: &SignalSet) -> Decision {
        if o_signals.has_deny_for(self.e_purpose) {
            return self.build_deny(o_signals);
        }
        self.build_allow(o_signals)
    }
}
```

3.7 Tooling that enforces this (created in F0): `clippy.toml` with `cognitive-complexity-threshold = 8`, `too-many-arguments-threshold = 5`, `too-many-lines-threshold = 25`; workspace lints denying `unwrap_used`, `expect_used`, `panic`, `todo`, `unsafe_code`, `missing_docs`; and a `convention-check` crate (uses `syn`) that fails on: a missing or wrong prefix for annotated types (params, fields, consts, annotated lets), an unprefixed inferred `let`, a missing `rename` on serde or sqlx fields, a method with no doc comment, a free function outside `main` and tests, and a file over 300 lines.

## What the tooling can and cannot see

`convention-check` works on syntax only.

**Checked prefixes:** `let` bindings, function and method parameters, struct and variant fields, consts, statics, `for` loop bindings, closure parameters (typed and untyped), `match` arm bindings, and `if let` / `while let` / let-chain bindings. Identifiers starting with an uppercase letter inside patterns (`None`, unit variants, constants) are not bindings and are skipped.

**Enforced by review, not by tooling:**
- Nesting depth, one loop level per method, match-arm length, and iterator-chain length.
- Doc comments starting with a verb.
- Bindings inside macro invocations such as `matches!(x, Some(v))`, because macro bodies are opaque tokens.
- Enum values for enums declared outside the workspace. Only those in the fixed list in `type_classifier.rs` require `e_`; any other external enum is accepted with `o_`.
- Type aliases, which are not resolved: an alias of `Vec<u8>` is treated as its own name and expects `o_`.

**Clippy's cognitive complexity is coarser than the spec's.** Clippy adds roughly one point per branch, loop, and boolean operator, with no extra weight for nesting. A deeply nested function can therefore score at or under 8 even when it reads as complex. `scripts/lint-probe.sh`, which runs in CI, proves the lint is active and fires above 8. Nesting is still reviewed by hand.

**Deliberate exemptions:**
- `#[serde(skip)]` and `#[serde(flatten)]` fields do not need `rename`, because they produce no JSON name of their own.
- Doc comments are not required inside `#[cfg(test)]` modules or under `tests/`.
- Free functions are allowed in `tests/` files, `#[cfg(test)]` code, `#[test]` functions, and the top-level `main` of `main.rs` or `build.rs`.
- Helper functions in integration tests return `Result` and use `?`, because clippy's `allow-expect-in-tests` does not cover them.

## Proving the lints fire

`./scripts/lint-probe.sh` copies `scripts/fixtures/lint_probe.rs` into a scratch crate, together with the workspace lints, `clippy.toml`, and the toolchain file. It fails unless all nine configured lints fire: `cognitive_complexity`, `too_many_lines`, `too_many_arguments`, `unwrap_used`, `expect_used`, `panic`, `todo`, `unsafe_code`, and `missing_docs`.
