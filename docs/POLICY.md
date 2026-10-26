# Signal policy v1

> **Status: stub; NEEDS LEGAL REVIEW.** F3 (policy evaluator) completes this document. Every
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

To be written in F3.
