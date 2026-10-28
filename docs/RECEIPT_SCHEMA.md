# Receipt schema (schema_version 2)

A receipt is the evidence for one considered URL: who crawled, for what purpose, which signals
were seen, what was decided and why, and what the fetch returned. Its canonical encoding
(RFC 8785 style, see `CanonicalEncoder`) is what the ledger hashes. Every field is always
present; optional values are explicit `null`. Decoding rejects missing and unknown fields.

## Version

`schema_version` is **2** (`U_RECEIPT_SCHEMA_VERSION`). It was bumped from 1 in F5.1 because:

- every signal gained two required fields, `body_truncated` and `body_bytes_seen`, so a v1
  decoder rejects v2 receipts and a v2 decoder rejects v1 receipts;
- the failure value `archive_unavailable` became `evidence_not_archived`, and the rule id
  `evidence.not_archived` was added to `decision.matched_rule`.

Any change to the canonical body changes receipt hashes, so the version is a single integer
with no minor part. No v1 receipt was ever stored (there is no ledger before F6), so nothing
needs migrating. The test `receipt_schema.rs` pins a fingerprint of the field list to this
version and fails when one changes without the other.

## Example

The frozen test vector (`crates/crawler-core/tests/fixtures/sample_receipt.canonical.json`),
SHA-256 `474210b41b81694868d4d1fb034ca7df8d8e9509e8b93a1283eab08c74b16ff1`. Canonical bytes,
one line, no trailing newline:

```json
{"decision":{"matched_rule":"robots.allow","outcome":"allow","policy_version":"v1","reasons":["robots.txt allows /a for *"]},"fetch":{"body_sha256":"b633a587c652d02386c4f16f8c6f6aab7352d97f16367c3c40576214372dd628","bytes":1234,"content_type":"text/html; charset=utf-8","finished_at":"2026-10-26T12:34:56.250000Z","http_status":200,"redirects":[{"status":301,"url":"http://example.com/a"}],"resolved_ips":["93.184.215.14"],"started_at":"2026-10-26T12:34:56.000000Z"},"identity":{"contact_url":"https://crawler.example/contact","purpose":"ai_train","user_agent":"proofcrawl/0.1 (+https://crawler.example/contact)"},"job_id":"00000000-0000-0000-0000-000000000002","lease_epoch":3,"receipt_id":"00000000-0000-0000-0000-000000000001","schema_version":2,"sequence":7,"signals":[{"body_bytes_seen":23,"body_truncated":false,"content_sha256":"16ceb5ee3e0dc13aa9adf31a3ebbe45a1d965b8c2b9f72eaf84e5911e140ed95","excerpt":{"original_bytes":22,"text":"User-agent: *\nAllow: /","truncated":false},"failure":null,"final_url":"https://example.com/robots.txt","http_status":200,"kind":"robots_txt","observed_at":"2026-10-26T12:34:56.000000Z","parsed":{"allowed":true,"content_signal":null,"crawl_delay_ms":null,"format":"robots_txt","groups":1,"invalid_lines":0,"matched_agent":"*","matched_rule":{"kind":"allow","pattern":"/"},"truncated":false},"redirects":[],"source_url":"https://example.com/robots.txt","status":"present"}],"target":{"host":"example.com","url_final":"https://example.com/a","url_normalized":"http://example.com/a","url_requested":"HTTP://Example.com:80/a"},"task_id":"00000000-0000-0000-0000-000000000003","worker_id":"00000000-0000-0000-0000-000000000004"}
```

The same receipt indented for reading (not canonical):

```json
{
  "decision": {
    "matched_rule": "robots.allow",
    "outcome": "allow",
    "policy_version": "v1",
    "reasons": [
      "robots.txt allows /a for *"
    ]
  },
  "fetch": {
    "body_sha256": "b633a587c652d02386c4f16f8c6f6aab7352d97f16367c3c40576214372dd628",
    "bytes": 1234,
    "content_type": "text/html; charset=utf-8",
    "finished_at": "2026-10-26T12:34:56.250000Z",
    "http_status": 200,
    "redirects": [
      {
        "status": 301,
        "url": "http://example.com/a"
      }
    ],
    "resolved_ips": [
      "93.184.215.14"
    ],
    "started_at": "2026-10-26T12:34:56.000000Z"
  },
  "identity": {
    "contact_url": "https://crawler.example/contact",
    "purpose": "ai_train",
    "user_agent": "proofcrawl/0.1 (+https://crawler.example/contact)"
  },
  "job_id": "00000000-0000-0000-0000-000000000002",
  "lease_epoch": 3,
  "receipt_id": "00000000-0000-0000-0000-000000000001",
  "schema_version": 2,
  "sequence": 7,
  "signals": [
    {
      "body_bytes_seen": 23,
      "body_truncated": false,
      "content_sha256": "16ceb5ee3e0dc13aa9adf31a3ebbe45a1d965b8c2b9f72eaf84e5911e140ed95",
      "excerpt": {
        "original_bytes": 22,
        "text": "User-agent: *\nAllow: /",
        "truncated": false
      },
      "failure": null,
      "final_url": "https://example.com/robots.txt",
      "http_status": 200,
      "kind": "robots_txt",
      "observed_at": "2026-10-26T12:34:56.000000Z",
      "parsed": {
        "allowed": true,
        "content_signal": null,
        "crawl_delay_ms": null,
        "format": "robots_txt",
        "groups": 1,
        "invalid_lines": 0,
        "matched_agent": "*",
        "matched_rule": {
          "kind": "allow",
          "pattern": "/"
        },
        "truncated": false
      },
      "redirects": [],
      "source_url": "https://example.com/robots.txt",
      "status": "present"
    }
  ],
  "target": {
    "host": "example.com",
    "url_final": "https://example.com/a",
    "url_normalized": "http://example.com/a",
    "url_requested": "HTTP://Example.com:80/a"
  },
  "task_id": "00000000-0000-0000-0000-000000000003",
  "worker_id": "00000000-0000-0000-0000-000000000004"
}
```

## Fields

"Set by" names the component that produces the value today (F5) and, where it moves, the
component that will own it once the server exists (F10).

| Field | Type | Set by | Why it exists |
|---|---|---|---|
| `schema_version` | integer | crawler-core constant | Lets a verifier pick the decoder; changes whenever the canonical body changes. |
| `receipt_id` | UUID string | worker (`IdSource`) | Stable name for the receipt; also the idempotency key for completion. |
| `sequence` | integer | ledger, assigned by the server at append (F6/F10) | Position in the hash chain; gap-free, so deletions and reordering show. |
| `job_id` | UUID string | server (job creation) | Groups receipts by crawl job for export and verification. |
| `task_id` | UUID string | server (frontier) | Links the receipt to the frontier task it completes. |
| `lease_epoch` | integer | server (claim) | Fencing token: proves which lease produced the receipt; stale completions are refused. |
| `worker_id` | UUID string | server (worker registration) | Which worker made the observations; provenance if a worker misbehaves. |
| `identity.user_agent` | string | `CrawlerIdentity` | The exact User-Agent sent, so the site's logs can be matched. |
| `identity.purpose` | `search` \| `ai_input` \| `ai_train` \| `archive` | job configuration | The declared purpose the policy evaluated against. |
| `identity.contact_url` | string | `CrawlerIdentity` | How the site owner reaches the operator. |
| `target.url_requested` | string | frontier | The URL as discovered, before normalization. |
| `target.url_normalized` | string | `UrlNormalizer` | Deduplication key; one receipt per normalized URL. |
| `target.url_final` | string or null | fetcher | URL after redirects; null when no fetch happened. |
| `target.host` | string | `UrlNormalizer` | Host the signals and rate limits apply to. |
| `signals[]` | array | `SignalCollector` | Every signal consulted, in observation order, including absent and unreachable ones. |
| `signals[].kind` | enum string | collector | Which signal: `robots_txt`, `ai_txt`, `tdmrep_well_known`, `tdmrep_header`, `tdmrep_meta`, `x_robots_tag`, `meta_robots`. |
| `signals[].status` | `present` \| `absent` \| `unreachable` | collector (status mapping in POLICY.md) | Absent and unreachable have different policy meaning. |
| `signals[].source_url` | string | collector | Where the signal was read. |
| `signals[].observed_at` | timestamp | collector (`Clock`) | When the source was fetched; for a cached file this is the original fetch time, up to the 1 h TTL earlier. |
| `signals[].http_status` | integer or null | collector | Final status; null when there was no response. |
| `signals[].content_sha256` | hex string or null | collector + body archive | Archive key of the **stored** bytes the parser read; resolves to them while referenced. Null for page-level signals and failures. |
| `signals[].excerpt` | object or null | collector | Human-readable start of the source (at most 2048 bytes). |
| `signals[].excerpt.text` | string | collector | The excerpt itself (lossy UTF-8). |
| `signals[].excerpt.truncated` | bool | collector | Whether the excerpt is shorter than the decoded text. |
| `signals[].excerpt.original_bytes` | integer | collector | UTF-8 length of the decoded text the excerpt was cut from (not the raw body length). |
| `signals[].final_url` | string or null | collector | Final URL after redirects; rules still apply to the original host (RFC 9309 §2.3.1.2). |
| `signals[].redirects[]` | array of `{status, url}` | collector | Every redirect followed, each one re-vetted. |
| `signals[].body_truncated` | bool | collector (F5.1) | Whether the body was cut at the 1 MiB cap, so `content_sha256` covers a prefix. |
| `signals[].body_bytes_seen` | integer | collector (F5.1) | Decoded bytes received before reading stopped; a lower bound of the real size when truncated. |
| `signals[].failure` | enum string or null | collector | Why a source was unreachable (`server_error`, `rate_limited`, `timeout`, `blocked_address`, `dns_failure`, `connection_failed`, `redirect_refused`, `bad_body`, `evidence_not_archived`). |
| `signals[].parsed` | object or null, tagged by `format` | parsers | Typed finding for the target path (`robots_txt`, `ai_txt`, `tdmrep`, `robots_tag`); null unless present. |
| `decision.outcome` | `allow` \| `deny` | `PolicyEvaluator` | The gate result; a deny means no page request. |
| `decision.matched_rule` | string | `PolicyEvaluator` | Highest-precedence denying rule id, or `no_reservation`. |
| `decision.reasons[]` | array of strings | `ReasonWriter` | One plain sentence per consulted signal. |
| `decision.policy_version` | string | `PolicyEvaluator` | Which documented policy produced the decision. |
| `fetch` | object or null | fetcher | Page fetch facts; null when denied. |
| `fetch.started_at`, `fetch.finished_at` | timestamps | fetcher (`Clock`) | Fetch window. |
| `fetch.http_status` | integer | fetcher | Final page status. |
| `fetch.content_type` | string or null | fetcher | Declared type of the body. |
| `fetch.bytes` | integer | fetcher | Decoded body length. |
| `fetch.body_sha256` | hex string | fetcher | Hash of the decoded page body. |
| `fetch.redirects[]` | array of `{status, url}` | fetcher | Redirect chain of the page. |
| `fetch.resolved_ips[]` | array of IP strings | fetcher (vetting resolver) | Addresses actually dialled; evidence for the SSRF guard. |

Not in the body, by design: `prev_hash`, `entry_hash`, signatures and `key_id`. They belong to
the ledger row and checkpoints (F6); putting them in the body would make the hash circular.

## Review against F6-F11: proposed changes (not applied)

Each item says which feature needs it. Applying them is one more bump, to **3**, before the
first ledger row is written, so the ledger never holds two schemas.

**Add**

1. `recorded_at` (timestamp, server). F6/F10: the spec says the server assigns timestamps;
   today every timestamp comes from the worker's clock. Worker times stay as observations.
2. `crawler {name, version}` (worker build). Ties findings to the parser and collector code
   that produced them, so a re-parse can use the same version. `identity.user_agent` carries a
   version only as free text.
3. `identity.product_token` (string). The token the robots groups were matched against; today
   it must be dug out of the User-Agent.
4. `fetch_failure` (enum string or null). F7: "deny and error outcomes also get receipts". An
   allowed URL whose page fetch fails (timeout, blocked address, body too large) has no place
   for the error now; `fetch` is null and the receipt looks like a deny.
5. `fetch.retry_after_ms` (integer or null). F7 per-host rate limiting and F10 admission act
   on 429/503 Retry-After; the receipt should show why a host was backed off.
6. `decision.decided_at` (timestamp). The gate-before-fetch claim (decision before any page
   request) is provable only if the decision time is recorded next to `fetch.started_at`.
7. `content_sha256` for page-level signals. X-Robots-Tag and meta values live only in the
   excerpt today. Archive the header block (and use the page body hash for meta signals) so
   page-level evidence resolves like signal files. Needed for F9's verification report.
8. `target.depth` (integer). F7 depth limits and scope rules are otherwise not auditable.

**Rename or redefine**

9. `excerpt.original_bytes` to `excerpt.text_bytes`: it measures the lossy UTF-8 text, not the
   raw body, which `body_bytes_seen` now covers.
10. `fetch.bytes` to `fetch.body_bytes`, matching `body_bytes_seen` and `body_sha256`.
11. `decision.matched_rule`: decode as a closed enum instead of a free string. The frozen test
    vector uses `robots.allow`, which the evaluator can never produce; fix the vector in the
    same bump (and its excerpt, whose `original_bytes` 22 disagrees with `body_bytes_seen` 23).

**Retention (behaviour, no field change)**

12. Archive references. `content_sha256` and `fetch.body_sha256` are the archive keys, so no
    extra reference field is needed. But today a body is kept only while the collector's cache
    references it; F6 must take one archive reference per stored receipt, so every hash in the
    ledger keeps resolving.

**Remove**

13. Nothing. Each field is used by a feature above or by verification.

**Test coverage**

14. Extend the fingerprint to enum value sets (`kind`, `status`, `failure`, `outcome`,
    `purpose`, rule ids, `format`). Renaming a value changes meaning without changing the field
    list, and today only the frozen vector would notice, and only for values it contains.
