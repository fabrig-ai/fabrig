# bundle-loading: status

2026-11-11. Where the design lags or departs from [bundle-loading.md](bundle-loading.md).

- **Design departs:**
  - 2, from "and nothing else": the page's head also holds its charset and viewport, as the bundler's page does (skein's `ui/web/bundle.md`, 3), and temper's server adds the sign-in ways' and kept forges' metas before `</head>` (`../protocol/sign-in.md`, 7, and `../protocol/openid-provider.md`, 7). None fetches anything. Decided by this plan's cut.
  - 2, from its sizes: the page is 2,539 bytes and its stylesheet 1,844, where this draft has 3.6 KB, and the script 2,831, where it has 2.1 KB. What travels is 4,779 bytes, where it has 2.9 KB gzipped: the stylesheet's form and the script's, 770 and 1,470, beside the page whole, because skein's `ui/web/bundle.md`, 4 compresses no page. Decided by this plan's cut.
  - 3, from "the stylesheets ... can't hold back a paint": the glue's `init` also waits for the stylesheets (skein's `ui/web/bundle.md`, 3), because a page's links held back today's starter and a client that measures an unstyled page would break where it now doesn't. Decided by this plan's cut, under the user's "take the nearest thing". The first node moves by at most the stylesheets' own download, within which the module goes on arriving.
  - 3, beyond the request's script: `#boot` takes `inert` as it hands over, so that neither a press nor an assistive tool lands on a fading placeholder. Decided by this plan's cut.
  - 4, from "The figure": until the module's answer is in, the line holds no figure, since which total it shows depends on that answer. Decided by this plan's cut.
  - 4, from "the bounds grow": `clients.bundle_bytes` grows and `clients.bundle_files` stays 128 (`../protocol/configuration/limits.md`), because the bundle holds about 40 files with its forms. Decided by this plan's cut.
  - 4, from "The workspace has no compressor today": `skein_http::gzip` is one, an encoder for a request's body that trades ratio for speed, and the bundler takes `flate2` for its best ratio (skein's `ui/web/bundle.md`, 2). Decided by this plan's cut, which has the encoder taken instead within 5% of `flate2`'s best; that comparison's figures are in no file of the repository, so the margin is unknown.
  - 7, what this design settles, by this plan's cut: both forms stay whole in memory, with no inflater (`../client/web/README.md`, 2.2); the playground keeps its own page and takes `--gzip off`, compressing it being its host's (`../playground.md`, 7).
- **Open:**
  - 7: other browsers (skein's `ui/web/README.md`, section 13); a box still starting (`../client/web/README.md`, section 3).

**Measured:** 2026-11-06, the release `web` module bundled with the placeholder:
25 plain files, 3,759,234 raw bytes and 1,359,545 bytes by `gzip -9 -c` per
file, including filename headers; before, 24 files, 3,746,557 and 1,353,371
bytes. The served module is 3,297,970 raw and 1,182,470 deterministic gzip
bytes; the directory including 14 forms is 39 files and 4,997,167 bytes.
The placeholder inputs are 3,969 raw bytes. An embedded release binary is
68,174,944 bytes. At 2 Mb/s under temper's unchanged policy, three fresh
Chromium/CDP visits paint paper in 60–72 ms, content in 304–332 ms and the
client's first node in 5,093.7–5,100.9 ms, against 1,476–1,496 ms,
15,192–15,280 ms and 15,097.9–15,180.5 ms before. The dev native server
starts listening in 781.2 ms with 63,380 KiB `VmHWM`, a loaded observation
during the release build; before, 370.4 ms and 61,860 KiB. Sizes are byte
counts of the bundler's output and `gzip -9 -c`'s. Timings come from a fresh
Chromium's paint entries and a mutation observer on `#app`, under CDP's network
emulation at 250,000 bytes/s each way with no added latency; received bytes from
CDP's encoded lengths; memory from the process's `/proc/PID/status`. The plan's
figures hold the method, its driver and the results, at `668f7cd3e5`.
temper's browser tier, whose tests load this page but for the playground's,
reads 83.069911 of its 85 CPU s over 13 tests with one red, against 61.048855
over 13 with none before (`../client/web/testing.status.md`, 3); skein's has no figure
(skein's `ui/web/bundle.status.md`, 3).
