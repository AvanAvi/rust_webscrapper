# proofcrawl

This is a temporary README. I will rewrite it when the first end-to-end crawl works.

I'm building a self-hostable crawler in Rust whose main output is evidence.
A crawler with the filing habits of an accountant who trusts nobody.

The goal is a signed receipt in a tamper-evident log for every URL considered, including denied URLs. The ledger and signing are still planned.

This is for teams that train or fine-tune models on web data, dataset vendors, and anyone who may later need to answer: "What did your crawler see, and why did it fetch that page?"

## What a receipt records

The receipt format records the crawler's declared identity and purpose, the signals it observed, the allow or deny decision, the rule and reasons, and any fetch facts.
Signals include robots.txt, Content-Signal, TDMRep, ai.txt, X-Robots-Tag, and meta robots.
Header and HTML signals are available only after a page fetch.

A receipt records observations. It does not prove legality.
proofcrawl cannot read or interpret terms of service.
The policy interpretations still need legal review. See [the policy](docs/POLICY.md) and [receipt schema](docs/RECEIPT_SCHEMA.md).

## Where it stands

Built and merged:

- Rust workspace, lint gates and a convention checker.
- Receipt types, canonical JSON encoding and hashing, and receipt schema version 3.
- Signal parsers and the policy evaluator.
- An SSRF-safe HTTP client and a local mock site for tests.
- A signal collector with a bounded in-memory evidence archive.

There is no working crawl pipeline yet. The `crawlerd` binary is a stub.
At the moment, the paperwork is ahead of the legs.

Planned, not built:

- A Postgres ledger, hash chain, signed checkpoints and a verifier.
- A frontier, a single-process crawl pipeline and a working CLI.
- WARC and manifest exports, an API server, separate workers and metrics.

The first end-to-end goal is one process that crawls a site and writes a receipt per URL.
It should survive being killed mid-crawl, with a verify command to check that the log was not altered.
Workers on several machines come later.

## How it will work (the plan)

1. Submit a crawl with a declared identity and purpose.
2. Check the site's signals for each URL.
3. Decide whether to allow or deny it and start its receipt.
4. Fetch only if allowed, then record fetch facts and page-level signals.
5. Store the completed receipt and evidence, including receipts for denied URLs.
6. Repeat for URLs in the frontier.

Workers will claim URLs under time-limited leases. A dead worker's unfinished URLs will return to the queue.
The queue gets the glamorous job of remembering what I forgot.

## Stack

- Rust: the workspace language.
- tokio: the async runtime.
- reqwest on rustls: HTTP and TLS.
- scraper: HTML parsing for meta robots and TDMRep meta tags.
- serde: receipt serialization and deserialization.
- sha2: SHA-256 hashes for receipts and bodies.
- axum: the local mock server today, the planned API server later.
- Postgres (planned): storage for jobs, the frontier and ledger entries.
- Ed25519 signing (planned).
- WARC (planned): receipt-linked exports.

## What you can run today

Use the toolchain pinned in `rust-toolchain.toml`. Builds also need a C toolchain.

```sh
cargo fmt --all --check
cargo run -p convention-check
cargo test -p crawler-core --locked
```

Fetch and TLS socket tests use the local mock site and need local networking.
See [CLAUDE.md](CLAUDE.md) for the full check commands.

## Conventions and workflow

The code follows strictly object oriented Rust, with traits for injection.
Checks enforce Hungarian prefixes and short methods, and CI runs on every PR.
See [CONVENTIONS.md](CONVENTIONS.md) for the full rules.
Work uses short-lived feature branches and pull requests; CI must pass before merge.

License: MIT. See [LICENSE](LICENSE).
