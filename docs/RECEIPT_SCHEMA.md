# Receipt schema (schema_version 3)

A receipt is the evidence for one considered URL: which crawler build and identity, for what
purpose, which signals the decision used, what was decided and when, and what the fetch
returned. Its canonical encoding (RFC 8785 style, `CanonicalEncoder`) is what the ledger hashes.
Every field is always present; optional values are explicit `null`. Decoding rejects missing and
unknown fields.

## Shape: server stamp around a worker submission

```
receipt = { schema_version, sequence, recorded_at, submission }
            '---- set by the server ----'          '-- worker --'
```

- `submission` (`ReceiptSubmission`) is everything a worker claims. It has no `sequence` and no
  `recorded_at`, and decoding a submission that carries either fails (unknown field), so a worker
  cannot supply them. The submission is the only part a worker could ever sign (F14).
- `ReceiptSubmission::seal(sequence, server_clock)` is the only way to make a `Receipt`: it
  takes `recorded_at` from the server's `Clock`, then runs `Receipt::validate`. `Receipt` fields
  are private.
- `seal` exists only with crawler-core's `ledger-seal` feature, which only crawler-ledger enables,
  so ledger append is the only way to seal a receipt. `scripts/check-seal-gate.sh` enforces this
  (feature ownership, a `compile_fail` doctest, and a floor on crawler-core's test count, because
  Cargo silently skips test targets whose required features are off). The gate prevents
  accidental use inside this workspace; it is not a security boundary.
- `Receipt::validate` enforces:
  - `schema_version` is 3;
  - `signal_set_id` is a lowercase SHA-256 hex digest;
  - deny: `fetch` and `fetch_failure` are both null; allow: exactly one of them is set;
  - `decision.decided_at` is strictly earlier than `fetch.started_at` whenever a fetch exists.
  Ledger append (F6) must call it on every receipt it stores.

## Version

`schema_version` is **3** (`U_RECEIPT_SCHEMA_VERSION`). It was bumped from 2 in F5.2 in one change,
before any ledger storage exists, so the ledger will only ever hold version 3. Version 2 was
never stored. The changes:

| Change | Why |
|---|---|
| `recorded_at` added, server-only, outside the submission | the spec has the server assign timestamps; worker clocks are claims |
| `submission` wrapper | separates server stamp from worker claims |
| `crawler {name, version}` added, version from the build | names the parser and collector code that produced the findings |
| `identity.product_token` added | the token robots.txt groups were matched against |
| `target.depth` added | depth and scope limits become auditable (F7) |
| `signals[]` replaced by `signal_set_id` | observations are stored once and shared; see "Signal sets" |
| `decision.matched_rule` is a closed set of rule ids | unknown ids are rejected on decode |
| `decision.decided_at` added | proves the decision preceded the fetch |
| `fetch.bytes` renamed `fetch.body_bytes` | matches `body_sha256` and `body_bytes_seen` |
| `fetch.retry_after_ms` added | 429/503 back-off is visible (F7/F10) |
| `fetch_failure` added | an allowed URL whose fetch failed no longer looks like a deny |
| `excerpt.original_bytes` renamed `excerpt.text_bytes` | it is the decoded text length, not the raw body |
| page-level signals got `content_sha256` | header evidence resolves like signal files |

`tests/receipt_schema.rs` pins a fingerprint of the receipt fields, the stored observation
fields, the set record shape, and every enum value name to this version; it fails when one
changes without the other. The frozen vector pins its own `crawler.version` (`0.1.0-frozen`,
deliberately not the build version), so a workspace version bump does not change it; production
receipts always carry the build version (`CrawlerRecord::from_build`).

## Example

The frozen test vector (`crates/crawler-core/tests/fixtures/sample_receipt.canonical.json`),
SHA-256 `393d4cb65ba44687a52f85978671f7048087cb755702fc8da5d4df631c32555b` (cross-checked with
`shasum -a 256`). Its decision comes from the real `PolicyEvaluator` (`no_reservation`), and its
one observation's excerpt, `text_bytes` and `body_bytes_seen` all describe the same 23-byte
robots.txt. Canonical bytes, one line, no trailing newline:

```json
{"recorded_at":"2026-10-26T12:34:56.265000Z","schema_version":3,"sequence":7,"submission":{"crawler":{"name":"proofcrawl","version":"0.1.0-frozen"},"decision":{"decided_at":"2026-10-26T12:34:56.000000Z","matched_rule":"no_reservation","outcome":"allow","policy_version":"v1","reasons":["robots.txt at https://example.com/robots.txt allows this URL for user-agent group \"*\" and states no Content-Signal objection to AI training."]},"fetch":{"body_bytes":13,"body_sha256":"b633a587c652d02386c4f16f8c6f6aab7352d97f16367c3c40576214372dd628","content_type":"text/html; charset=utf-8","finished_at":"2026-10-26T12:34:56.260000Z","http_status":200,"redirects":[{"status":301,"url":"http://example.com/a"}],"resolved_ips":["93.184.215.14"],"retry_after_ms":null,"started_at":"2026-10-26T12:34:56.010000Z"},"fetch_failure":null,"identity":{"contact_url":"https://crawler.example/contact","product_token":"proofcrawl","purpose":"ai_train","user_agent":"proofcrawl/0.1.0-frozen (+https://crawler.example/contact; purpose=ai-train)"},"job_id":"00000000-0000-0000-0000-000000000002","lease_epoch":3,"receipt_id":"00000000-0000-0000-0000-000000000001","signal_set_id":"676a27ce4731261023b72a020561ee5c7d02b8055a55746cee62591d032f6dc5","target":{"depth":1,"host":"example.com","url_final":"https://example.com/a","url_normalized":"http://example.com/a","url_requested":"HTTP://Example.com:80/a"},"task_id":"00000000-0000-0000-0000-000000000003","worker_id":"00000000-0000-0000-0000-000000000004"}}
```

The same receipt indented for reading (not canonical):

```json
{
  "recorded_at": "2026-10-26T12:34:56.265000Z",
  "schema_version": 3,
  "sequence": 7,
  "submission": {
    "crawler": {
      "name": "proofcrawl",
      "version": "0.1.0-frozen"
    },
    "decision": {
      "decided_at": "2026-10-26T12:34:56.000000Z",
      "matched_rule": "no_reservation",
      "outcome": "allow",
      "policy_version": "v1",
      "reasons": [
        "robots.txt at https://example.com/robots.txt allows this URL for user-agent group \"*\" and states no Content-Signal objection to AI training."
      ]
    },
    "fetch": {
      "body_bytes": 13,
      "body_sha256": "b633a587c652d02386c4f16f8c6f6aab7352d97f16367c3c40576214372dd628",
      "content_type": "text/html; charset=utf-8",
      "finished_at": "2026-10-26T12:34:56.260000Z",
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
      "retry_after_ms": null,
      "started_at": "2026-10-26T12:34:56.010000Z"
    },
    "fetch_failure": null,
    "identity": {
      "contact_url": "https://crawler.example/contact",
      "product_token": "proofcrawl",
      "purpose": "ai_train",
      "user_agent": "proofcrawl/0.1.0-frozen (+https://crawler.example/contact; purpose=ai-train)"
    },
    "job_id": "00000000-0000-0000-0000-000000000002",
    "lease_epoch": 3,
    "receipt_id": "00000000-0000-0000-0000-000000000001",
    "signal_set_id": "676a27ce4731261023b72a020561ee5c7d02b8055a55746cee62591d032f6dc5",
    "target": {
      "depth": 1,
      "host": "example.com",
      "url_final": "https://example.com/a",
      "url_normalized": "http://example.com/a",
      "url_requested": "HTTP://Example.com:80/a"
    },
    "task_id": "00000000-0000-0000-0000-000000000003",
    "worker_id": "00000000-0000-0000-0000-000000000004"
  }
}
```

## Signal sets (`signal_set_id`)

Observations are not embedded. `SignalSetArchive` stores each observation's canonical JSON keyed
by its SHA-256 (the observation hash), then a set record: the canonical JSON array of those
hashes in observation order (the decision's reasons follow that order, so it is an ordered list,
not a sorted set). `signal_set_id` is the SHA-256 of the set record. Loading re-checks every
hash, so the receipt commits to the exact observations; identical observations (one host's
robots.txt finding reused across pages) are stored once.

If storing a set fails, nothing stays referenced and no receipt may be written for that decision.

Size of a typical receipt (`tests/receipt_size.rs`: 3.3 KB robots.txt, ai.txt and tdmrep.json
answering with a 2.4 KB 404 page, four page-level observations; canonical bytes):

| | bytes | vs embedded |
|---|---|---|
| receipt with embedded observations | 12,217 | |
| receipt with `signal_set_id` | 2,479 | 80% smaller |
| stored per receipt, 100 pages of one host in one TTL (receipt + set record 470 + page-level observations 2,066 + 1/100 of host-level 7,736) | 5,092 | 58% smaller |
| stored per receipt, one page per host | 12,751 | 4% larger |

The saving is above 30% for any host crawled for more than one page, so the pointer was
implemented. A host visited for a single page costs 4% more than embedding.

## Fields

"Set by" names who produces the value.

| Field | Type | Set by | Why it exists |
|---|---|---|---|
| `schema_version` | integer | server (`seal`) | Lets a verifier pick the decoder. |
| `sequence` | integer | server, at ledger append (F6/F10) | Position in the hash chain; gaps and reordering show. |
| `recorded_at` | timestamp | server `Clock` (`seal`) | When the server accepted the receipt; never worker-supplied. |
| `submission.receipt_id` | UUID string | worker (`IdSource`) | Name of the receipt and completion idempotency key. |
| `submission.job_id` | UUID string | server (job creation), echoed by the worker | Groups receipts per job. |
| `submission.task_id` | UUID string | server (frontier), echoed | Links the receipt to its frontier task. |
| `submission.lease_epoch` | integer | server (claim), echoed | Fencing token; stale completions are refused. |
| `submission.worker_id` | UUID string | server (registration), echoed | Which worker made the observations. |
| `submission.crawler.name` | string | build constant `STR_CRAWLER_NAME` | Product name. |
| `submission.crawler.version` | string | build (`CARGO_PKG_VERSION`); only `CrawlerRecord::from_build` creates one | Parser and collector code version. |
| `submission.identity.user_agent` | string | `CrawlerIdentity` | Exact User-Agent sent. |
| `submission.identity.product_token` | string | `CrawlerIdentity` | Token matched against robots.txt groups. |
| `submission.identity.purpose` | enum string | job configuration | Purpose the policy evaluated against. |
| `submission.identity.contact_url` | string | `CrawlerIdentity` | How the site owner reaches the operator. |
| `submission.target.url_requested` | string | frontier | URL as discovered. |
| `submission.target.url_normalized` | string | `UrlNormalizer` | Deduplication key. |
| `submission.target.url_final` | string or null | fetcher | URL after redirects; null without a response. |
| `submission.target.host` | string | `UrlNormalizer` | Host signals and rate limits apply to. |
| `submission.target.depth` | integer | frontier (F7) | Link distance from the seeds (seeds are 0). |
| `submission.signal_set_id` | hex string | `SignalSetArchive` | Commits to the observations the decision used. |
| `submission.decision.outcome` | `allow` \| `deny` | `PolicyEvaluator` | The gate result. |
| `submission.decision.matched_rule` | rule id (closed set) | `PolicyEvaluator` | Highest-precedence denying rule, or `no_reservation`. |
| `submission.decision.reasons[]` | strings | `ReasonWriter` | One plain sentence per observation, in set order. |
| `submission.decision.policy_version` | string | `PolicyEvaluator` | Which documented policy decided. |
| `submission.decision.decided_at` | timestamp | worker `Clock` | Must be earlier than `fetch.started_at`. |
| `submission.fetch` | object or null | fetcher | Page fetch facts; null when denied or when the fetch failed. |
| `submission.fetch.started_at`, `.finished_at` | timestamps | fetcher `Clock` | Fetch window. |
| `submission.fetch.http_status` | integer | fetcher | Final status (4xx/5xx are fetch records, not failures). |
| `submission.fetch.content_type` | string or null | fetcher | Declared type. |
| `submission.fetch.body_bytes` | integer | fetcher | Decoded body length. |
| `submission.fetch.body_sha256` | hex string | fetcher | Hash of the decoded body. |
| `submission.fetch.redirects[]` | `{status, url}` | fetcher | Redirect chain, each hop re-vetted. |
| `submission.fetch.resolved_ips[]` | IP strings | vetting resolver | Addresses actually dialled. |
| `submission.fetch.retry_after_ms` | integer or null | fetcher | Retry-After of a 429/503 in ms, capped at one hour. |
| `submission.fetch_failure` | enum string or null | fetcher (`FetchError::fetch_failure`) | Why an allowed fetch produced no response. |

Stored observation (`SignalObservation`, one archive entry each, reached through `signal_set_id`):

| Field | Type | Why it exists |
|---|---|---|
| `kind`, `status`, `source_url`, `observed_at`, `http_status`, `final_url`, `redirects[]` | as in v2 | What was read, where, when; `observed_at` of a cached file is its fetch time (up to the 1 h TTL earlier). |
| `content_sha256` | hex string or null | Hash of the exact bytes the parser read. Signal files: the stored body. Header kinds: the canonical header text (below). Meta kinds: the page body (`fetch.body_sha256`). Null only for failures. |
| `excerpt {text, truncated, text_bytes}` | object or null | Readable start of the source; `text_bytes` is the UTF-8 length of the decoded text it was cut from. |
| `body_truncated` | bool | Whether the body was cut at the cap, so the hash covers a prefix. |
| `body_bytes_seen` | integer | Decoded bytes received before reading stopped. **A lower bound, not the real file size**: reading stops after the first chunk that crosses the cap, so for a truncated body the real size is unknown and at least this. |
| `failure` | enum string or null | Why a source was unreachable, including `evidence_not_archived`. |
| `parsed` | object or null, tagged by `format` | Typed finding for the target path. |

### Canonical header text (page-level header signals)

X-Robots-Tag and TDMRep header signals are parsed from, hashed over, and archived as one text
form per kind: only that kind's headers (`x-robots-tag`; `tdm-reservation`, `tdm-policy`), names
lowercased, values trimmed, one `name: value` line each ending in `\n`, sorted by name. The sort
is stable: lines with the same name keep their response order, because RFC 9110 section 5.3
makes that order significant (TDMRep reads the first `tdm-reservation`). Two responses that
differ only in the order of different header names give the same text and hash. Other headers
(cookies included) are never archived. If the archive refuses the text, the kind is
`unreachable` with `evidence_not_archived`.

Meta signals (`meta_robots`, `tdmrep_meta`) carry the page body hash. Page bodies are not in the
body archive yet (5 MB pages exceed its 1 MiB per-body cap); they resolve once F7/F9 store pages.

Not in the receipt, by design: `prev_hash`, `entry_hash`, signatures and `key_id`. They belong to
the ledger row and checkpoints (F6).

## Known limits

- Archive references: `SignalSetArchive::store` takes one reference per stored entry, and
  receipts must keep them (F6). Signal-file bodies and header texts are still kept only while
  the collector's cache holds them, or until evicted; F6 must take references for every hash a
  stored receipt reaches.
- The User-Agent version is the build version: `CrawlerIdentity::new` takes no version
  argument, so `identity.user_agent` always names the same version as `crawler.version`.
