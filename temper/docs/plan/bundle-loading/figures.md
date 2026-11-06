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
