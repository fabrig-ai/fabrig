# Bundle loading figures

## Method

Measured on 2026-11-06, before this plan's implementation, at base
`5b178bcaefc81ef03a50f2f6817aa2335b3164ca`.

The application module was built with
`cargo build --target wasm32-unknown-unknown --profile web -p temper-client-shell-web`.
The bundler and native server were built with `cargo build -p skein-ui-bundle`
and `cargo build -p temper`, with `TEMPER_BUNDLE=none`; all three builds used
the required build wrapper. The module's release profile is `web`
(`temper/docs/design/client/browser.md`, 2.6); the native server is a dev build.
The bundler took the `commands` arguments from
`temper/crates/temper-client-bundle/build/steps.rs`: the module, name
`temper_client_shell_web`, prefix `/bundle/`, `temper.css`, `task-pages.css`,
the shell's `files/`, and noscript text. Its output is kept outside the checkout.

Each raw size is `wc -c FILE`; each compressed size is
`gzip -9 -c FILE | wc -c`, including gzip's filename header. The gzip column
is a size measurement, not what the server sent. Every file is counted,
including licenses, fonts and files not requested by this particular page.

The deployment follows `temper/tests/client-browser/src/configuration.rs`'s
`demo`: `temper init --box`, replace its four listeners with reserved loopback
ports in both configuration documents, and name the bundle directory in
`application.clients.bundle`. Start `temper serve --config DIR --data DIR
--cache DIR` with closed stdin. The visit is `/`, signed out, without spending
a sign-in link. The policy is the server's unchanged policy
(`temper/docs/design/client/browser.md`, 2.2), captured in the response log:

```text
default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'
```

Run [the driver](driver/load.py) with `python3 -I -B`, using
`SKEIN_TEST_CHROMIUM`'s Chromium 147.0.7727.101, new headless, CDP pipe, and a
new temporary profile per invocation. The viewport is 1440 × 900 at scale 1,
set by `Emulation.setDeviceMetricsOverride`. Network emulation is 250,000
bytes/s in each direction (2 Mb/s), zero added latency, online; cache is
enabled. Background networking is disabled. The two invocations are:

```text
python3 -I -B temper/docs/plan/bundle-loading/driver/load.py URL 1440 OUT1 --mark first-paint --mark first-text --mark first-node
python3 -I -B temper/docs/plan/bundle-loading/driver/load.py URL 1440 OUT2 --cached --mark first-paint --mark first-text --mark first-node
```

`--cached` navigates away and back in the same Chromium process, preserving
its profile and HTTP cache. It measures a second navigation, not a browser
restart. Each invocation writes `load.json`, Chromium stderr and the named
PNGs; stdout is the same single JSON document. The driver answers JavaScript
dialogs and records runtime exceptions and security log entries.

Times are milliseconds from the browser's `NavigationStart`. Paint times
come from the page's paint performance entries. Every 50 ms, a
`Runtime.evaluate` reads `document.body.innerText`; its first nonempty result
is the readable-text proxy, with polling and renderer scheduling uncertainty.
It can precede the next contentful paint. A mutation observer records the
first child of `#app` and its `performance.now()` time. Screenshots are taken
when the driver observes the named mark, so their capture can follow its
timestamp; that observation time is also logged. The first-text screenshot
can already show the next client state.

Bytes at a mark are accumulated CDP `Network.responseReceived.encodedDataLength`
and `Network.dataReceived.encodedDataLength`, reconciled to
`Network.loadingFinished.encodedDataLength` at completion, up to that mark's
browser timestamp. These are received HTTP response bytes, including headers,
excluding request bytes and transport overhead. Cached bodies contribute zero.
They are event-granular, not a packet capture or an estimate from 2 Mb/s.
Each response's URL, status, headers and content encoding, each completion's
encoded length, and the byte deltas are retained in the JSON.

Server startup is wall time from process launch to reading its
`listening clients` stderr line. `VmHWM` comes from that process's
`/proc/PID/status`, at the listening line and again after the browser loads,
before its clean stop. It includes the demo deployment as well as the bundle;
there is no subtraction of an empty-server baseline and no memory figure for
child processes.

The browser tier runs once with
`TIERS_WRAPPER=heavy python3 -I -B tools/tiers/run --cpu OUT temper browser`,
with `TEMPER_BUNDLE=none`. The runner's CPU collector sums user and system
CPU of tests and their descendants, excluding prerequisite builds; its wall
span is reported separately. The tier builds its usual dev module with
`loopback-navigation`, plus its playground bundle, rather than using the
release bundle above (`temper/docs/design/client/browser.md`, 4.3).

## Before

### Release bundle sizes

The digest directory is `668da015606d08db`; rows below it omit that prefix.
The raw compiler output is processed by the bundler; the module row measures
the resulting served module. Byte counts are decimal bytes.

| File | Raw bytes | `gzip -9` bytes |
|---|---:|---:|
| `index.html` (bundle root) | 580 | 318 |
| `boot.js` | 57 | 83 |
| `skein-ui.css` | 6,372 | 1,242 |
| `temper.css` | 249,567 | 37,710 |
| `task-pages.css` | 15,658 | 3,817 |
| `temper_client_shell_web.js` | 54,578 | 9,083 |
| `temper_client_shell_web_bg.wasm` | 3,292,877 | 1,180,424 |
| `files/brand.svg` | 231 | 194 |
| `files/chat.svg` | 159 | 163 |
| `files/doc.svg` | 183 | 175 |
| `files/menu.svg` | 199 | 185 |
| `files/pen.svg` | 156 | 161 |
| `files/robots.txt` | 26 | 57 |
| `files/send.svg` | 191 | 169 |
| `files/split.svg` | 185 | 174 |
| `files/fonts/LICENSE-ibm-plex.txt` | 4,348 | 1,958 |
| `files/fonts/LICENSE.txt` | 4,764 | 1,534 |
| `files/fonts/README.txt` | 1,618 | 765 |
| `files/fonts/dejavu-sans-700-latin.woff2` | 25,432 | 25,483 |
| `files/fonts/ibm-plex-mono-latin-400-normal.woff2` | 14,812 | 14,872 |
| `files/fonts/ibm-plex-mono-latin-500-normal.woff2` | 14,988 | 15,048 |
| `files/fonts/ibm-plex-sans-latin-400-normal.woff2` | 19,156 | 19,216 |
| `files/fonts/ibm-plex-sans-latin-500-normal.woff2` | 20,064 | 20,124 |
| `files/fonts/ibm-plex-sans-latin-600-normal.woff2` | 20,356 | 20,416 |
| **Total: 24 files** | **3,746,557** | **1,353,371** |

### Waiting at 2 Mb/s

| Moment | Fresh 1: ms / bytes received | Fresh 2: ms / bytes received | Cached after fresh 2: ms / bytes received |
|---|---:|---:|---:|
| First paint | 1,496 / 329,977 | 1,476 / 329,977 | 292 / 479 |
| First contentful paint | 15,192 / 3,625,050 | 15,280 / 3,625,793 | 700 / 1,470 |
| First readable text (poll) | 15,203 / 3,625,281 | 15,253 / 3,625,281 | 667.7 / 958 |
| First child of `#app` | 15,097.9 / 3,624,530 | 15,180.5 / 3,624,530 | 643.8 / 479 |

The first-paint screenshots are blank off-white (`rgb(252, 251, 248)`),
without a mark, words or progress. The first nonempty text in all three loads
is “temper”, “Inbox”, “Chats”, “Forges”, “Accounts”, “Connecting”. The first
child is `header.temper-banner`. At screenshot capture, the client has
advanced to the signed-out message: “Signed out. To sign in, open a sign-in
link temper serve printed.” The fresh first-paint screenshots were captured
at 1,654 and 1,580 ms; the cached one at 559 ms.

All bundle responses are `200` without `Content-Encoding`; no compressed
answer is served. The cached visit's bundle responses are disk-cache hits;
its total received response bytes are 1,470. No runtime exception or security
log entry was observed in any measured load. The signed-out stream returns
`401`, then its fetch is cancelled (`net::ERR_ABORTED`); this is recorded,
not treated as a policy violation. The extra exploratory fresh run, before
disabling Chromium background networking, is excluded from the tables.

### Server start

| Measurement | Before |
|---|---:|
| Launch to `listening clients` | 370.4 ms |
| `VmHWM` at listening | 61,860 KiB |
| `VmHWM` after browser loads, before stop | 65,468 KiB |
| Clean stop exit status | 0 |

### Browser tier

| Run | Test CPU seconds | Test wall seconds | CPU budget | Result |
|---|---:|---:|---:|---|
| `browser-trim` baseline, lower reported figure | 58 | unknown | 85 | Within budget |
| `browser-trim` baseline, upper reported figure | 79 | unknown | 85 | Within budget |
| Before, this run | 61.048855 | 59.021640 | 85 | 13/13 passed; within budget |

The baseline's **58** and **79** CPU-second figures are quoted by the
[plan request](inputs/request.md) from `browser-trim`'s report, under load.
Its wall times are unknown here; that source report was not available in
this checkout. This run used 45.915981 user and 15.132874 system CPU seconds,
leaving 23.951145 CPU seconds below the budget. There were zero starved,
leaked, incomplete or unobserved tests. Nextest's own summary wall time was
59.041 seconds; the table uses the collector's test span.

The host had 16 logical CPUs. At command start, the 1/5/15-minute load
averages were 43.49 / 24.28 / 17.73. These figures describe a loaded run;
no idle-machine normalization is applied. The builder and retained raw logs
are named in the task report.

## After

### A person's first paint

| Visit | Before: ms / response bytes | After: ms / response bytes | First paint after |
|---|---:|---:|---|
| Fresh 1 | 1,496 / 329,977 | 60.0 / 4,441 | Paper alone |
| Fresh 2 | 1,476 / 329,977 | 60.0 / 7,044 | Paper alone |
| Fresh 3 | not measured | 72.0 / 7,608 | Paper alone |
| Cached return | 292 / 479 | 76.0 / 479 | Paper alone |

The paper is deliberately blank for the placeholder's 250 ms CSS delay.
Before, it was `rgb(252, 251, 248)` until the client; after, it is
`rgb(247, 247, 245)` until the mark and progress appear. The first contentful
paint is now 304–332 ms, against 15,192–15,280 ms before. A cached return
shows paper and then the client: every sampled mark has opacity zero until
handover, and the placeholder is made inert and removed without appearing.

### Waiting at 2 Mb/s

| Moment | Fresh 1: ms / bytes | Fresh 2: ms / bytes | Fresh 3: ms / bytes | Cached after fresh 1: ms / bytes |
|---|---:|---:|---:|---:|
| First paint | 60.0 / 4,441 | 60.0 / 7,044 | 72.0 / 7,608 | 76.0 / 479 |
| First contentful paint | 332.0 / 66,176 | 312.0 / 66,176 | 304.0 / 61,666 | 216.0 / 1,470 |
| First readable text (poll) | 267.5 / 48,176 | 247.0 / 49,676 | 213.5 / 39,166 | 171.1 / 479 |
| First child of `#app` | 5,099.1 / 1,246,959 | 5,100.9 / 1,246,959 | 5,093.7 / 1,246,959 | 170.5 / 479 |

The same `driver/load.py`, Chromium, 1440 × 900 viewport, 250,000 bytes/s
network emulation, deployment and unaltered policy as **Method** were used.
The driver additionally records the placeholder's computed opacity and title;
the metrics and byte accounting are unchanged. All four marks were captured
in this run, including first-contentful-paint. The readable-text proxy can
see a line before its CSS animation makes it visible; use contentful paint
for the first visible content. The first node remains `header.temper-banner`.
The page's title is `temper` throughout loading. No runtime exception or
security log entry occurred in any of the three fresh visits or the cached
return. Earlier exploratory captures preceded the spacing correction and
are excluded from these figures.

The module's body is 1,182,470 compressed bytes. Each fresh CDP completion
is 1,183,044 encoded bytes: the form plus 574 response-header bytes. The
cached module's encoded length is zero. The cached navigation receives
1,470 bytes in total. The loader's displayed total is **1.2 MB**, from
1,234,535 compressed bytes for the module, glue and three client sheets;
it excludes boot, the placeholder and fonts. Its fraction counts decoded
bytes, as the contract requires.

### Release bundle sizes

The release `web` module and bundler were built by **Method**. The bundler
also received `--title temper`, `--placeholder` with the shell's
`loading.html`, and `--placeholder-stylesheet` with its `loading.css`.
The digest is `2e04861d1feb95a2`. The comparison column again uses
`gzip -9 -c FILE`, including its filename header. The last column is the
bundler's deterministic form actually served; a dash means no stored form.

| File | Raw bytes | `gzip -9` bytes | Stored `.gz` bytes |
|---|---:|---:|---:|
| `boot.js` | 2,831 | 1,478 | 1,470 |
| `files/brand.svg` | 231 | 194 | 184 |
| `files/chat.svg` | 159 | 163 | 154 |
| `files/doc.svg` | 183 | 175 | 167 |
| `files/fonts/LICENSE-ibm-plex.txt` | 4,348 | 1,958 | — |
| `files/fonts/LICENSE.txt` | 4,764 | 1,534 | — |
| `files/fonts/README.txt` | 1,618 | 765 | — |
| `files/fonts/dejavu-sans-700-latin.woff2` | 25,432 | 25,483 | — |
| `files/fonts/ibm-plex-mono-latin-400-normal.woff2` | 14,812 | 14,872 | — |
| `files/fonts/ibm-plex-mono-latin-500-normal.woff2` | 14,988 | 15,048 | — |
| `files/fonts/ibm-plex-sans-latin-400-normal.woff2` | 19,156 | 19,216 | — |
| `files/fonts/ibm-plex-sans-latin-500-normal.woff2` | 20,064 | 20,124 | — |
| `files/fonts/ibm-plex-sans-latin-600-normal.woff2` | 20,356 | 20,416 | — |
| `files/menu.svg` | 199 | 185 | 176 |
| `files/pen.svg` | 156 | 161 | 153 |
| `files/robots.txt` | 26 | 57 | — |
| `files/send.svg` | 191 | 169 | 160 |
| `files/split.svg` | 185 | 174 | 164 |
| `loading.css` | 1,844 | 782 | 770 |
| `skein-ui.css` | 6,372 | 1,242 | 1,229 |
| `task-pages.css` | 15,658 | 3,817 | 3,808 |
| `temper.css` | 249,808 | 37,724 | 37,736 |
| `temper_client_shell_web.js` | 55,344 | 9,184 | 9,292 |
| `temper_client_shell_web_bg.wasm` | 3,297,970 | 1,183,313 | 1,182,470 |
| `index.html (bundle root)` | 2,539 | 1,311 | — |
| **Total: 25 plain files** | **3,759,234** | **1,359,545** | **1,237,933** |

Before's 24 plain files totalled 3,746,557 raw and 1,353,371 `gzip -9`
bytes. After's 25 plain files total 3,759,234 raw and 1,359,545 `gzip -9`
bytes. The directory, counting the 14 stored forms too, is 39 files and
4,997,167 bytes. The markup and placeholder stylesheet together are
3,969 raw bytes, within 6 KiB; the generated page and that stylesheet
are 4,383 raw bytes. The page is served without compression.

A separate `TEMPER_BUNDLE=embed` release build produces a **68,174,944-byte**
native binary. Copied alone into a directory, it serves the same placeholder,
title, stylesheet and loader through both `temper serve` and normal
`temper box`, with no bundle directory configured. The server adds its
existing empty `temper-sign-in` metadata to the generated head.

Both module answers were captured with `curl -D`: plain is 3,297,970 bytes;
`Accept-Encoding: gzip` is exactly the stored 1,182,470-byte form, and
inflating it equals the plain body byte for byte. Both retain the immutable
cache policy, all security policy fields and `Vary: Accept-Encoding`; only
the compressed answer has `Content-Encoding: gzip`. Header order follows
`notes/contracts.md`, C4. The page is uncompressed under the same policy.

### Render check

`driver/render.py` extracts the four request pages and stylesheet from
`0c1b78f3e1` without landing them. It captures each element's complete
computed style and box, then PNGs, at 1440, 1000 and 420 × 900. CDP sets
the viewport, including 420. `driver/compare.py` retains every difference;
none is hidden by a tolerance or ignored property. Screenshots and raw
snapshots are retained outside the checkout, as the task report specifies.

The real release page reaches downloading and its estimate, the eight-second
note (at 0.5 Mb/s, since the release load completes inside eight seconds
at 2 Mb/s), waiting after 15 seconds with no byte, every failure, scripts
off, retry, handover and reduced motion. A forwarding proxy preserves the
server's policy and compressed bytes while pausing or closing a module
answer. Silent answers close before any client bytes; 404 uses a bundle
copy with both the module and its form removed. CDP deletes WebAssembly
before page scripts, or disables scripts, or requests reduced motion.
`Starting…` uses a throwaway glue copy whose `init` waits after consuming
the real answer; failed init consumes it and throws. These deliberately
held states make no production change. The reference failure and the real
failure are also captured hovered and keyboard-focused, after the retry
button's intro animation.

The one visual correction to the requested stylesheet is
`letter-spacing: normal` on `#boot`: the client stylesheet otherwise
changes it to `-0.18px` during download. With this reset, the mark, bar,
fixed-word lines and button have the requested geometry. Live figures,
estimates and the bar's fill differ because the release is smaller and
compression is served. Computed styles also retain the client's 17 unused
custom properties, light color scheme and box-sizing reset; the covered
body's typography/background differ and the fixed container's computed
content width follows box sizing, while its physical box is unchanged.
These do not change the placeholder's drawing. The request pages' titles
name their examples; real loading tabs say `temper`.

### Server start

The same dev native server and startup/memory method as **Method** are
used. The start occurred during the separate release build; it is a loaded
observation, without normalization.

| Measurement | Before | After |
|---|---:|---:|
| Launch to `listening clients` | 370.4 ms | 781.2 ms |
| `VmHWM` at listening | 61,860 KiB | 63,380 KiB |
| `VmHWM` after browser loads, before stop | 65,468 KiB | 66,660 KiB |
| Clean stop exit status | 0 | 0 |

### Browser tier

| Run | Test CPU seconds | Test wall seconds | CPU budget | Result |
|---|---:|---:|---:|---|
| `browser-trim` baseline, lower reported figure | 58 | unknown | 85 | Within budget |
| `browser-trim` baseline, upper reported figure | 79 | unknown | 85 | Within budget |
| Before | 61.048855 | 59.021640 | 85 | 13/13 passed |
| After, full temper tier | 83.069911 | 50.645617 | 85 | 12 passed, 1 failed; CPU within budget |
| After, failed public test alone | 5.570423 | 11.866217 | 85 | Same test failed alone; CPU within budget |
| After, skein tier | unknown | unknown | 145 | Unobserved: stopped for the repeated failure |

The same CPU collector and tier command as **Method** were used, with a
fresh output directory per invocation. The full run used 60.563673 user
and 22.506238 system CPU seconds. There were zero starved, leaked,
incomplete or unobserved selected tests in either temper invocation.
The full tier has only 1.930089 CPU seconds of headroom; it is **red for
its assertion**, despite staying below the CPU limit.

`browser_public::a_visitor_reads_a_public_project_and_loses_it_when_made_private`
fails waiting for the shell command that runs
`temper ctl ... project amend Demo --visibility public --key public`
to exit. The page has reached New chat/Ready and the placeholder is gone.
The full run expires the command's ten-second deadline; the one allowed
isolated run expires the same deadline at the same step. Its thread uses
about 109 ms CPU and waits about 17.6 ms runnable, so this is not a
wall-hard-limit or starvation warning. Neither the public browser story
nor the ctl path is changed by this task. Under the run's repeated-failure
rule, the task stops for a scope/repair decision; **browser-tier acceptance
is not complete**, and the skein tier has not been run.

The tier builds and loads the compressed ordinary bundle under temper's
policy. Its separate static playground keeps `--gzip off` and its own page,
as the plan's contract requires. The script path was separately built with
`bundle.sh --loopback`; `curl` verified title, markup, stylesheet and loader
through both serve and normal box. No harness wait or browser-story change
was made.
