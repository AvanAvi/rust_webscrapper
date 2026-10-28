# Signal policy v1

> **Status: NEEDS LEGAL REVIEW.** Policy version `v1`, implemented by `PolicyEvaluator` in
> `crawler-core` (F3). Every
> interpretation below is an engineering reading of a published or de-facto format, not legal
> advice. A receipt records what was observed and how it was interpreted; it does not prove
> that a crawl was lawful.

## Interpretations needing legal review

### ai.txt: assumed page extensions (F2)

- **Source gap:** Spawning publishes no grammar for ai.txt. Its documentation gives only the
  file's location (`/ai.txt` or `/.well-known/ai.txt`) and purpose (media-type opt-outs for
  AI training).
- **Interpretation:** the file is read with robots.txt syntax. Groups use `User-Agent` lines,
  rules use `Allow`/`Disallow`, `*` and `$` are wildcards, the longest rule wins, and allow
  wins a tie.
- **Extension fallback:** if the URL path matches no rule, the path is retried with each
  assumed page extension appended. The defaults are `html` and `htm`, set in `SignalContext`.
  So `Disallow: *.html` reserves `/about`. The receipt records the extension used in
  `parsed.assumed_extension`.
- **Effect:** a disallow rule that applies is a reservation. An absent ai.txt is recorded as
  `absent` and is never a reservation.
- **Why it needs review:** the fallback treats an extensionless URL as an HTML page before
  fetching it. That may over- or under-reserve compared with what a rightsholder intended.

### X-Robots-Tag scope (F2.1)

- An unscoped value such as `X-Robots-Tag: noai` applies to us.
- A value scoped to `*` (`X-Robots-Tag: *: noai`) also applies to us. That scope is not
  standard, but it is read as "all crawlers".
- A value scoped to another crawler (`googlebot: noai`) does not apply. A value scoped to our
  product token, compared case-insensitively, does.
- Values whose directive itself contains a colon are not treated as scoped: `max-snippet`,
  `max-image-preview`, `max-video-preview`, and `unavailable_after`.
- **One header value with several scopes (F3):** a `bot:` prefix scopes the comma-separated
  directives after it, up to the next prefix. So `googlebot: noindex, proofcrawl: noai` gives
  `noai` to us and `noindex` only to googlebot. Directives before the first prefix apply to
  every crawler. Google's documentation shows only one scope per header, so this reading of a
  combined header is ours. Before F3 the whole value was scoped to the first bot, which
  silently dropped our `noai`.

### Other parser readings (F2)

- **Content-Signal:** syntax follows Cloudflare's Content Signals Policy announcement
  (2025-09-24): `key=yes|no` pairs for `search`, `ai-input`, and `ai-train` inside a
  user-agent group. An omitted key means no preference. When lines or matching groups
  conflict, `no` wins.
- **TDMRep (W3C CG final report, 2024-05-10):** only `1` and `0` are valid reservation
  values. Anything else is a protocol error and leaves the reservation unset. In
  `tdmrep.json` the first rule whose `location` matches decides.
- **Crawl-delay:** not part of RFC 9309. It is honoured as a minimum interval, stored as
  whole milliseconds, and capped at one day.

## Signal-file fetching and caching (F5)

Section references are to RFC 9309, read from the RFC text at rfc-editor.org (2026-10-28).

### Status mapping

| Result of fetching a signal file | Status recorded | Effect for robots.txt |
|---|---|---|
| 2xx | `present`, parsed | Its rules apply (§2.3.1.1) |
| 4xx except 429 | `absent` | "Unavailable": no rules, any resource may be crawled (§2.3.1.3) |
| 429 | `unreachable`, failure `rate_limited` | Full disallow (our reading; see below) |
| 5xx | `unreachable`, failure `server_error` | "Unreachable": complete disallow (§2.3.1.4) |
| Timeout, refused address, DNS failure, connection error, unsafe body | `unreachable`, typed failure | Complete disallow (§2.3.1.4: "server or network errors") |
| More than 5 redirects, or a refused redirect | `unreachable`, failure `redirect_refused` | Complete disallow (our reading; see below) |

ai.txt and tdmrep.json use the same status mapping. Only a present file can deny, so their
absent and unreachable results have no effect (owner decision, F3).

### Unreachable robots.txt policy

- **While unreachable:** the URL is denied for every purpose (`robots_txt.unreachable`).
  The decision reason names the HTTP status or the failure, for example
  "no HTTP response: timed out".
- **Retry with exponential backoff:** a failed fetch is cached for
  `delay(n) = min(60 s × 2^(n-1), 1 h) × (1 ± 20 %)`, capped at 1 h again after jitter.
  Here n is the number of consecutive failures; the jitter comes from the injected `IdSource`.
  That gives about 60 s, 120 s, 240 s, …, up to 1 h. Concurrent callers share the cached
  failure, so a failing host gets one request per backoff period, not one per URL.
- **Recovery:** the first success resets the streak and is cached for the TTL.
- **Not adopted:** §2.3.1.4 allows treating a long-unreachable file (the example is 30 days)
  as unavailable, which would mean allowing everything. Policy v1 never does that.

### Cache TTL

- **Successful fetches** (present or absent) are reused for **1 hour**. §2.4 says a cached
  robots.txt SHOULD NOT be used for more than 24 hours, and the collector clamps any
  configured TTL to 24 h. One hour keeps observations fresh while costing three requests per
  host per hour at most.
- **Failures** are cached for the backoff delay only, never for the full TTL.
- **Bounded in hosts:** at most 10,000 hosts are cached. At the limit, the least recently used
  host with no fetch in flight is dropped (its archived bodies lose the cache's reference). If
  every cached host has a fetch in flight, the new host is fetched without caching, and that
  fetch's archive reference is released at once.
- **Bounded in bytes (F5.1):** the cached files may account for at most 128 MiB: each file's
  archived body plus an estimate of its parsed form, computed from lengths (allocator slack is
  not counted). Without this bound, 10,000 hosts x 3 files x 1 MiB could pin about 30 GiB.
  Over the bound the least recently used idle hosts are dropped; the host just stored is kept,
  so the overshoot is at most one host (3 MiB of bodies plus its parsed forms).

### Size limits

Signal files are fetched with a 1 MiB cap. Longer bodies are truncated, not rejected; the
receipt's excerpt shows `truncated`. The robots parser reads the first 500 KiB, the minimum
§2.5 requires. Pages keep the 5 MB reject cap.

**What the hash covers (F5.1).** A signal's `content_sha256` is the SHA-256 of the **stored
bytes**: the decoded body as far as it was kept, which is also exactly what the parser read.
For a truncated file it is not the hash of the file the server holds. Two fields say so:

- `body_truncated`: whether the body was cut at the 1 MiB cap.
- `body_bytes_seen`: decoded bytes actually received before reading stopped. Reading stops
  after the first chunk that crosses the cap, so for a truncated body this is a lower bound of
  the real size, not the real size.

Re-parsing the archived bytes gives the same finding as the live decision.

### Evidence archiving and the fail-closed rule (F5.1)

Every answered signal-file body (2xx, 4xx, 5xx alike) is stored in the body archive before it
is used. The in-memory archive has three limits:

| Limit | Value | Why |
|---|---|---|
| Per body | 1 MiB | Equals the signal fetch cap, twice the 500 KiB RFC 9309 §2.5 asks parsers to handle, so the stored bytes always cover what was parsed. |
| Total bytes | 256 MiB | Twice the cache's 128 MiB budget: bodies the cache pins can never fill the archive, and the other half absorbs bodies of in-flight and uncached fetches. |
| Entries | 65,536 | Above the 30,000 bodies a full 10,000-host cache can reference, so small bodies cannot exhaust it; bookkeeping (hash key plus entry) stays near 13 MiB. |

At a limit the oldest body nobody references is evicted. If every body is referenced, the
store fails. **The collector then fails closed:** the signal is recorded as `unreachable`
with failure `evidence_not_archived`, with no hash and no parsed content, and the policy
denies the URL for **every purpose** under rule `evidence.not_archived`, whatever the signal
kind. This includes ai.txt and TDMRep, which otherwise never deny when unreachable, and a
robots.txt 404, which otherwise allows. No allow decision rests on a body that cannot be
re-checked. The failure is cached for the backoff period only, like any other failure.

Known gap until F6: a body is kept while the cache references it. Once the cache drops it,
the body is evictable. Receipts must hold their own reference once the ledger exists.

**Page-level evidence (F5.2).** X-Robots-Tag and TDMRep header signals are parsed from the
canonical text of their headers (lowercased names, trimmed values, one line each, sorted by
name, same-name lines in response order), and that text is archived; if the archive refuses
it, the header signal is `evidence_not_archived` and denies, like a signal file. Meta signals
record the page body hash; page bodies are not archived until F7/F9. See
`docs/RECEIPT_SCHEMA.md`.

### Interpretations needing legal review (F5)

- **Cross-host redirects of robots.txt are followed** (owner's default). §2.3.1.2 says
  crawlers SHOULD follow at least five redirects "even across authorities", and that the
  rules found apply "in the context of the initial authority".
  - The collector follows up to 5 hops, and every hop passes the address checks.
  - The redirect chain and the final URL are recorded in the observation.
  - The rules are cached under the original host.
- **More than 5 redirects counts as unreachable** (complete disallow). §2.3.1.2 says crawlers
  MAY assume the file is unavailable, which would mean allow-all; we chose the stricter
  reading.
- **429 counts as unreachable**, although §2.3.1.3 gives 400-499 as the example range for
  "unavailable". A rate-limit response says nothing about the site's rules, so treating it as
  allow-all would be wrong.
- **A refused address or an unsafe body counts as unreachable**, which denies for robots.txt.
- **Duplicate TDMRep headers: the first in response order wins** (F5.2). The canonical header
  text keeps same-name lines in response order (RFC 9110 §5.3 makes it significant), and the
  TDMRep parser reads the first `tdm-reservation`. Sorting the values instead would let `0`
  win over `1` and could hide a reservation.
- **A body that cannot be archived denies every purpose** (F5.1), even when the signal itself
  would allow (a robots.txt 404, an ai.txt with no matching rule). This is stricter than the
  signals require; it exists so that no allow decision rests on evidence that cannot be
  re-checked. It only triggers when the archive is full of referenced bodies.

## Signal semantics by purpose

The crawler declares one purpose per job: `search`, `ai_input`, `ai_train`, or `archive`.
The default is `ai_train`, the strictest. Each observed signal gets a verdict. **Deny wins:**
if any signal denies, the URL is denied and the page is not fetched. The decision records:

- one plain sentence per consulted signal, in observation order
- the matched rule, which is the highest-precedence denying rule (or `no_reservation`)

**Only signals that were present can deny.** There are two exceptions:

- An unreachable robots.txt, which RFC 9309 requires to be treated as a full disallow.
- A signal of any kind whose body could not be archived (`evidence_not_archived`), which
  denies every purpose (see "Evidence archiving and the fail-closed rule").

| Signal (source) | search | ai_input | ai_train | archive | Rule id |
|---|---|---|---|---|---|
| Any signal file whose body could not be archived | deny | deny | deny | deny | `evidence.not_archived` |
| robots.txt rule disallows the path | deny | deny | deny | deny | `robots_txt.disallow` |
| robots.txt 5xx, timeout, or no response | deny | deny | deny | deny | `robots_txt.unreachable` |
| robots.txt 4xx (absent) | no rules | no rules | no rules | no rules | none |
| Content-Signal `search=no` | deny | none | none | none | `robots_txt.content_signal` |
| Content-Signal `ai-input=no` | none | deny | none | none | `robots_txt.content_signal` |
| Content-Signal `ai-train=no` | none | none | deny | none | `robots_txt.content_signal` |
| Content-Signal line or key missing, or `=yes` | none | none | none | none | none |
| TDMRep `tdm-reservation: 1` (header, meta, or tdmrep.json) | none | deny | deny | none | `tdmrep.reservation` |
| TDMRep `0`, invalid value, unset, absent, or unreachable | none | none | none | none | none |
| ai.txt disallow applies | none | deny | deny | none | `ai_txt.disallow` |
| ai.txt absent or unreachable | none | none | none | none | none |
| X-Robots-Tag or robots meta `noai` | none | deny | deny | none | `robots_tag.noai` |
| X-Robots-Tag or robots meta `noimageai` | none | deny | deny | none | `robots_tag.noimageai` |
| Other robots directives (`noindex`, `nofollow`, `noarchive`, `none`, `max-*`, …) | none | none | none | none | recorded only |

Precedence of the matched rule, when several signals deny:

1. `evidence.not_archived`
2. `robots_txt.unreachable`
3. `robots_txt.disallow`
4. `robots_txt.content_signal`
5. `tdmrep.reservation`
6. `ai_txt.disallow`
7. `robots_tag.noai`
8. `robots_tag.noimageai`

Every denying signal still appears in the reasons. Archive has no Content-Signal key, so no
Content-Signal value affects it.

### Mappings needing legal review

- **TDMRep → AI training and AI input only** (owner's default). A TDM reservation (EU DSM
  Directive art. 4) is read as covering text-and-data mining for AI training and AI input,
  not search indexing or archiving. Whether search snippets or archives count as TDM is a
  legal question.
- **ai.txt → AI training and AI input only.** Spawning describes ai.txt as covering
  commercial AI training. We extend it to AI input as the safer reading.
- **`noimageai` denies AI uses of whole pages,** as the spec requires, even though it names
  images. This is a deliberately strict reading.
- **`noindex` and `noarchive` are recorded but not enforced** (owner decision, 2026-10-27).
  They stay visible: the parsed finding lists every directive, and the decision reason names
  the directives v1 keeps but does not enforce.
- **Unreachable ai.txt or TDMRep sources do not deny** (owner decision, 2026-10-27). Only
  robots.txt has an RFC rule for unreachability. The decision reason says plainly that the
  signal could not be checked.
- **Page-level signals** (X-Robots-Tag, meta robots, TDMRep header and meta) are known only
  after the page is fetched. The evaluator applies them whenever they are in the signal set;
  how the pipeline handles them after a fetch is decided in F7.
