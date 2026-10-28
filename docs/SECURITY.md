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

### TLS end-to-end test coverage (issue #8)

`cargo test -p crawler-fetch --features test-policy --locked --test tls_redirects` exercises
real TLS on loopback through SafeHttpClient and its reqwest redirect policy. The test harness
creates fresh CA keys and signed server leaves entirely in memory; no certificate, private key,
or OS trust-store modification is committed or required. The TLS listener lives in integration
test support, alongside the existing plaintext mock-site, and is not part of mock-site's library.

The scenarios cover HTTPS-to-HTTP refusal with the typed downgrade error and zero plaintext
requests; successful HTTPS-to-HTTPS redirects with recorded hops; an untrusted leaf signed by a
different generated root (`UnknownIssuer`); hostname mismatch (typed certificate-name error);
matching-name success followed by wrong-name refusal on the same SafeHttpClient; redacted errors;
and a downgraded robots.txt becoming unreachable/redirect_refused and denying the crawl.
Credentials in requested URLs are refused before TLS; the redaction test checks that path too.
The socket-dependent mutation script disables the downgrade branch temporarily, requires the
end-to-end assertion to fail, restores the source even on error, and rechecks the original test.

Extra test roots are compiled only under `test-policy`, using `tls_certs_merge` so reqwest keeps
its production platform verifier. Certificate and hostname verification remain enabled. The
feature also exposes a last-transport-error diagnostic with reqwest's URL removed, so tests can
assert the underlying rustls certificate error rather than accepting any connection failure.
This diagnostic is for sequential test assertions; it holds only the last failure per client.
Neither API exists in the default build. The default-build compile-negative test and the guard
script self-test verify root API absence/presence as well as rejecting feature-enabled binaries.

Linux behavior is verified only by CI on `ubuntu-24.04`; local macOS socket results must be
recorded separately. Adding the scenarios does not itself establish that either platform has
passed: consult the run results for the tested commit. These tests do not cover real public CAs,
HSTS, certificate pinning, OCSP/revocation behavior, TLS version/cipher policy, or plain HTTP
origin confidentiality. Receipts record observations and do not prove legality.
