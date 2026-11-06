# bundle-loading: the plan

The client's bundle goes compressed, and a placeholder shows while it loads. Compression first, since it is worth having by itself; then the page and the script, with the look request `bundle-loading`.

## Parts and tasks

| Task | Part | Needs | What |
|---|---|---|---|
| `before` | figures-before | | a release bundle's sizes, raw and gzipped; a person's wait at 2 Mb/s, a start's time and memory, the tier's figures, before anything lands; the driver the proof re-uses |
| `gzip-skein` | compression-skein | | the bundler's `NAME.gz`, `Call::accepts_gzip`, the example's server |
| `gzip-serve` | compression-serve | `gzip-skein` | jig's helpers, temper's reader and server, the bounds, the playground's `--gzip off` |
| `page-skein` | page-skein | `gzip-skein` | the bundler's page with a placeholder, the loader, the example's fixture, two browser stories |
| `page-temper` | page-temper | `page-skein`, `gzip-serve`, `before` | the look request's landing, the proof at 2 Mb/s on a release bundle, the figures after, the account |

The shapes that cross them are `notes/contracts.md`. The design, skein's `ui-web.md`, 4.2 to 4.4 and 15, `client/browser.md`, 2.2, 2.5, 2.6, 4.3 and 5, `protocol/configuration.md`, 7, and their neighbours in `http.md`, 5.2, `examples.md`, 9.3, `protocol/service.md`, 11.4 and `playground.md`, 7, was written by this cut, with each document's **Not yet** lines; the tasks close those. The draft's status has nothing lagging.

## Critical path

`gzip-skein` (60), `page-skein` (75), `page-temper` (120): about 4 h 15 of implementer time, with `gzip-serve` (75) and `before` (45) beside it.

## Overlaps

- `temper/docs/design/client/browser.md` and `browser.status.md`: `chat-reading-c/reading-close` and `model-check/trial-close` change them too. This cut's edits are small lines in 2.2, 2.5, 2.6, 4.3 and 5, and a line in the status's head.
- `temper/docs/design/client/journeys.md`, `mockups/STATUS.md`: `chat-reading-c/reading-close`, at the very end of `page-temper`, one line.
- `temper/crates/temper-client-shell-web/temper.css`: `chat-reading-c/reading-browser` and others; `page-temper` only takes one selector out of existing rules, and adds nothing.
- `skein/docs/design/ui-web.status.md`: `chat-reading-c/page-reading`, already merged in its trunk: a line each.
- `temper/tests/client-browser/src/lib.rs`, which `chat-reading-c/reading-browser` and `sign-in-new-tab/browser-story` are changing: kept off. `sign-in-new-tab` and `model-check` touch nothing of this plan's code, by the request.

## Decisions

A format or protocol change needs no `Question:` (the request). Made here:

- **Both forms whole in memory,** the draft's section 7 settled: no inflater in the service, a client not taking gzip still answered, and memory bounded by `clients.bundle_bytes` (`temper/docs/design/client/browser.md`, 2.2). The other way, the plain form alone inflated, would add a mechanism for a rare client.
- **`clients.bundle_bytes` 32 MiB; `clients.bundle_files` stays 128,** since about 40 files fit. A departure from the draft's "the bounds grow", in its status.
- **The playground** keeps its own page and takes `--gzip off`, the draft's section 7 settled: compressing it is its host's (`temper/docs/design/playground.md`, 7), and the tier's download figure would double otherwise (`browser_playground.rs` sums the directory). It's `story_a`'s to take the placeholder later.
- **A compressed form's bytes are in the digest,** so "the digest of everything in it" keeps meaning that.
- **What didn't hold:** the draft's section 4 says the workspace has no compressor. It has `skein_http::gzip`, an encoder for LLM request bodies that trades ratio for speed. `gzip-skein` compares it with `flate2`'s best on the real module and takes it only within 5%; otherwise the bundler takes `flate2`, as the draft says.
- **`init` waits for the stylesheets,** beyond the draft: today's links hold back the starter, so no client has ever measured a page without its sheets, and the browser tier reads layout. It costs the stylesheets' own download, during which the module goes on arriving. In the draft's status.
- **The figure before the module answers** is an empty line, since which total to show depends on that answer. It lasts under 250 ms on anything but a stalled server, where 15 s then says it's waiting.
- **`inert` at handing over,** beyond the request's script, so that neither a press nor an assistive tool lands on a fading placeholder.
- **`{noscript}` is a marker in the markup,** so temper's noscript words stay in one place, `--noscript`, and the bundler still reads no HTML.

## Measured

The release `web` bundle has 25 plain files: 3,759,234 raw bytes and
1,359,545 bytes when each is measured with `gzip -9 -c`, including its
filename header. Before, 24 files totalled 3,746,557 and 1,353,371 bytes.
The module is 3,297,970 raw bytes; its served deterministic gzip form is
1,182,470 bytes. Including all 14 stored forms, the directory is 39 files
and 4,997,167 bytes. The placeholder's markup and stylesheet cost 3,969
raw bytes. The embedded release binary is 68,174,944 bytes and serves the
page with no bundle directory configured.

At 2 Mb/s, three fresh release visits paint the paper in 60–72 ms and
show content in 304–332 ms; the client's first node arrives in
5,093.7–5,100.9 ms. Before, two visits painted blank paper in 1,476–1,496 ms
and first showed content in 15,192–15,280 ms, with the first node in
15,097.9–15,180.5 ms. A cached return paints in 76 ms and reaches the client
in 170.5 ms with no visible placeholder. These are Chromium/CDP measurements
on the same driver, viewport, throttle and unchanged server policy as
`figures.md`, **Method**; its **After** keeps every size, byte count,
startup measurement, render comparison and tier figure.
