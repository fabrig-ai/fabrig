# bundle-loading: the plan

The client's bundle goes compressed, and a placeholder shows while it loads. Compression first, since it is worth having by itself; then the page and the script, with the look request `bundle-loading`.

## Parts and tasks

| Task | Part | Needs | What |
|---|---|---|---|
| `before` | figures-before | | a release bundle's sizes, raw and gzipped; a person's wait at 2 Mb/s, a start's time and memory, the tier's figures, before anything lands; the driver the proof re-uses |
| `gzip-skein` | compression-skein | | the bundler's `NAME.gz`, `Call::accepts_gzip`, the example's server |
| `gzip-serve` | compression-serve | `gzip-skein` | jig's helpers, temper's reader and server, the bounds, the playground's `--gzip off` |

## Critical path

`gzip-skein`, `gzip-serve`. The rest of the plan follows on this page's next push.

## Overlaps

None of these three touches a file another draft is changing, other than `temper/docs/design/client/browser.md` and `browser.status.md`, which this cut edits in sections no other draft does.

## Decisions

Format and protocol changes need no `Question:` (the request). So far:

- **Both forms whole in memory,** the draft's section 7 settled: no inflater in the service, and memory is bounded by `clients.bundle_bytes` (`temper/docs/design/client/browser.md`, 2.2).
- **`clients.bundle_bytes` 32 MiB; `clients.bundle_files` stays 128,** since about 40 files fit.
- **The playground** keeps its own page and takes `--gzip off`, the draft's section 7 settled: compressing it is its host's (`temper/docs/design/playground.md`, 7).
- **A compressed form's bytes are in the digest,** so "the digest of everything in it" keeps meaning that.
- **What didn't hold:** the draft's section 4 says the workspace has no compressor. It has `skein_http::gzip`, an encoder for LLM request bodies that trades ratio for speed. `gzip-skein` compares it with `flate2`'s best on the real module and takes it only within 5%; otherwise the bundler takes `flate2`, as the draft says.
