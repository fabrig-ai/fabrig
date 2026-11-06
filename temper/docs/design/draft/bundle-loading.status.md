# bundle-loading: status

2026-11-06. Where the design lags or departs from [bundle-loading.md](bundle-loading.md).

- **Design departs:**
  - 3, from this draft's "the stylesheets ... can't hold back a paint": the glue's `init` also waits for the stylesheets (skein's `ui-web.md`, 4.3), because a page's links held back today's starter and a client that measures an unstyled page would break where it now doesn't. Decided by this plan's cut, under the user's "take the nearest thing". The first node moves by at most the stylesheets' own download, within which the module goes on arriving.
  - 4, from "the bounds grow": `clients.bundle_bytes` grows and `clients.bundle_files` stays 128 (`../protocol/configuration.md`, section 7), because the bundle holds about 40 files with its forms. Decided by this plan's cut.
  - 4, from "The workspace has no compressor today": `skein_http::gzip` is one, an encoder for a request's body that trades ratio for speed. The bundler still takes `flate2` for its best ratio, unless the plan's measure finds them within 5% (skein's `ui-web.md`, 4.2). Decided by this plan's cut.
- **Open:**
  - 7: other browsers (skein's `ui-web.md`, section 16); a box still starting (`../client/browser.md`, section 5).
