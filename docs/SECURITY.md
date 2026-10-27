# Security notes

Engineering notes on security-relevant choices. This is not a security audit.

## TLS stack: rustls on aws-lc-rs (C code)

- **What we use:** `reqwest` 0.13.5 with its `rustls` feature (default features off). That
  feature selects rustls 0.23 with the `aws-lc-rs` crypto provider, through
  `__rustls-aws-lc-rs` in reqwest's `Cargo.toml`.
- **The C code:** `aws-lc-rs` depends on `aws-lc-sys`, which compiles AWS-LC, Amazon's fork
  of BoringSSL, written in C and assembly. So the TLS stack is not pure Rust, even though our
  crates `#![forbid(unsafe_code)]` and no OpenSSL is linked (`deny.toml` bans `openssl`,
  `openssl-sys`, and `native-tls`).
- **Consequences:**
  - A memory-safety bug in AWS-LC could affect the crawler.
  - Builds need a C toolchain.
  - On macOS, `ld` prints a harmless unwind-info warning for one of AWS-LC's assembly symbols
    (`_sha3_keccak4_f1600`).
- **Mitigations:** keep `aws-lc-sys` current through `cargo audit` and `cargo deny check` in
  CI, and run crawls in a container (planned for F13).

### Alternative: the ring provider

`ring` is the other common rustls provider. We did not choose it, for these reasons (checked
against reqwest 0.13.5's `Cargo.toml` and ring 0.17.14's `build.rs`):

1. **reqwest 0.13.5 has no ring feature.** Its TLS features are `rustls`, which always
   selects aws-lc-rs, `rustls-no-provider`, and the `native-tls` variants. Using ring would
   mean:
   - turning on `rustls-no-provider`
   - adding `rustls` (with its `ring` feature) as a direct dependency; it is not on the
     approved list
   - installing ring as the process-wide default provider at startup
   
   That is more code and one more approval.
2. **ring does not remove C either.** Its build script compiles C files such as
   `crypto/curve25519/curve25519.c`, `crypto/fipsmodule/aes/aes_nohw.c`, and
   `crypto/fipsmodule/ec/p256.c`, plus assembly. Switching providers would reduce the amount
   of C in the TLS stack, not eliminate it.

**Revisit if** reqwest gains a direct ring feature, or a pure-Rust provider becomes mature
enough for production TLS.

## Fetching (F4)

See `crates/crawler-fetch`.

- **Address policy:** sealed. Production allows only globally routable unicast addresses,
  including embedded-IPv4 forms of IPv6.
- **Resolution:** each host is resolved once and every answer is vetted; a mixed answer
  refuses the host. IP-literal hosts are checked before any connection.
- **Redirects:** at most 5, and every hop is re-checked.
- **Limits:** connect 5 s, read 15 s, total 30 s. 5 MB cap after decompression. 20:1
  compression-ratio limit.
- **Isolation:** no proxies, no retries, no connection pool, no referer.
- **Test policy:** the loopback-only policy exists only with the `test-policy` feature.
  `scripts/check-no-test-policy.sh` and its self-test guard that crawlerd never enables it.
