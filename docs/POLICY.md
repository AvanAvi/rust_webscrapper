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

## Signal semantics by purpose

The crawler declares one purpose per job: `search`, `ai_input`, `ai_train`, or `archive`.
The default is `ai_train`, the strictest. Each observed signal gets a verdict. **Deny wins:**
if any signal denies, the URL is denied and the page is not fetched. The decision records:

- one plain sentence per consulted signal, in observation order
- the matched rule, which is the highest-precedence denying rule (or `no_reservation`)

**Only signals that were present can deny.** The one exception is an unreachable
robots.txt, which RFC 9309 requires to be treated as a full disallow.

| Signal (source) | search | ai_input | ai_train | archive | Rule id |
|---|---|---|---|---|---|
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

1. `robots_txt.unreachable`
2. `robots_txt.disallow`
3. `robots_txt.content_signal`
4. `tdmrep.reservation`
5. `ai_txt.disallow`
6. `robots_tag.noai`
7. `robots_tag.noimageai`

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
