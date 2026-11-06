# bundle-loading: status

2026-11-06. Where the design lags or departs from [bundle-loading.md](bundle-loading.md).

- **Design departs:**
  - 3, from this draft's "the stylesheets ... can't hold back a paint": the glue's `init` also waits for the stylesheets (skein's `ui-web.md`, 4.3), because a page's links held back today's starter and a client that measures an unstyled page would break where it now doesn't. Decided by this plan's cut, under the user's "take the nearest thing". The first node moves by at most the stylesheets' own download, within which the module goes on arriving.
  - 4, from "the bounds grow": `clients.bundle_bytes` grows and `clients.bundle_files` stays 128 (`../protocol/configuration.md`, section 7), because the bundle holds about 40 files with its forms. Decided by this plan's cut.
  - 4, from "The workspace has no compressor today": `skein_http::gzip` is one, an encoder for a request's body that trades ratio for speed. The bundler still takes `flate2` for its best ratio, unless the plan's measure finds them within 5% (skein's `ui-web.md`, 4.2). Decided by this plan's cut.
- **Open:**
  - 7: other browsers (skein's `ui-web.md`, section 16); a box still starting (`../client/browser.md`, section 5).

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
counts of the bundler's output; timings, received bytes, screenshots and
memory use the same driver and method in
`temper/docs/plan/bundle-loading/figures.md`, **Method** and **After**.
