# bundle-loading: the shapes that cross tasks

The run's binding calls. The design is `skein/docs/design/ui-web.md`, 4.2 to 4.4, and `temper/docs/design/client/browser.md`, 2.2, 2.5 and 2.6; this note gives the names.

## C. Compression

### C1. The directory (`gzip-skein`)

- `skein_ui_bundle::Inputs` gains `pub gzip: bool`; the binary's `--gzip off` sets it false, default true. Any other value of `--gzip` is the usage error. (Every argument of the binary is a pair.)
- `skein_ui_bundle::types::gzipped(path: &Path) -> bool`: `.wasm`, `.js`, `.css`, `.svg`.
- Beside each such file, unless `gzip` is false: `NAME.gz`, one gzip member of `NAME`: `flate2`'s `GzBuilder` with no name and `mtime(0)`, `Compression::best()`, on `flate2`'s Rust backend. Its header's OS byte is whatever `GzBuilder` writes, fixed by `Cargo.lock`.
- The digest covers the `.gz` files too: it stays over everything in the directory (`ui-web.md`, 4.2). They are made before it is computed.
- `.gz` stays off `types::of`, so an input named `x.css.gz`, whether a stylesheet or under `--files` or `--pictures`, is `Refusal::Type`.
- The workspace's `flate2` line, `default-features = false, features = ["rust_backend"]`, exact version; `Cargo.toml`'s `[profile.dev.package.*]` gives `flate2`, `miniz_oxide`, `crc32fast` and `adler2` `opt-level = 3`, as `skein-ui-bundle` has, since `scripts/bundle.sh` and the tiers run the dev bundler.

### C2. What a request takes (`gzip-skein`)

`skein_http::server::Call::accepts_gzip(&self) -> bool`, by `skein/docs/design/http.md`, 5.2: over every `Accept-Encoding` field, a listed `gzip` or `x-gzip`, or `*` where neither is listed, with a weight above 0; case ignored; no field or a value it can't read says false.

| Field(s) | Says |
|---|---|
| none; empty; `identity`; `br`; `gzipped`; `gzip;q=0`; `gzip;q=0.000`; `gzip;q=x`; `*;q=0`; `gzip;q=0, *` | false |
| `gzip`; `GZIP`; `x-gzip`; `gzip, deflate, br`; `deflate, gzip;q=0.5`; `*`; `br;q=1, *;q=0.1`; two fields, `br` then `gzip` | true |

### C3. jig's helpers (`gzip-serve`)

```rust
// jig_protocol_engine_clients_http::bundle
pub struct File { pub kind: &'static [u8], pub bytes: Box<[u8]>, pub gzip: Option<Box<[u8]>> }
/// The file `path` is the compressed form of: `NAME` for a `NAME.gz` whose `NAME` is a `.wasm`, `.js`, `.css` or `.svg`.
#[must_use]
pub fn compressed_of(path: &[u8]) -> Option<&[u8]>
```

`kind` still answers `None` of a `.gz`. `File`'s `Debug` keeps deriving: nothing in it is private.

### C4. A server's answer

- A reader pairs them: a path `compressed_of` names attaches to that file's `gzip`, counted in the bytes and files like any other, and a `NAME.gz` with no `NAME` refuses, `a compressed form with no file`. A `.gz` that `compressed_of` doesn't name (`x.txt.gz`) refuses as a type off the closed list does, as it does today.
- temper's and jig-test-service's `Answer` gain `Gzip`, which `route` returns where `file.gzip.is_some() && head.accepts_gzip()`; `File` otherwise.
- `Answer::Gzip`'s body is the form, its fields `Content-Type` (the file's), `Cache-Control` (the file's), `Content-Encoding: gzip` and `Vary: Accept-Encoding`. `Answer::File` for a file with a form adds `Vary: Accept-Encoding`; for a file with none (a `.woff2`), neither field.
- Field order: `Content-Type`, `Content-Encoding`, `Vary`, `Cache-Control`, then the policy's.
- `/bundle/<digest>/NAME.gz` is not found: the map holds `NAME` alone.
- The page and `/robots.txt` are never compressed.

### C5. The bounds (`gzip-serve`)

`standard`'s `clients.bundle_bytes` is `32 << 20`. `clients.bundle_files` stays 128: the bundle holds about 40 files with its forms, and the status says so. jig-test's reader counts both forms too; its `BUNDLE_BYTES` is `32 << 20`, and a `_bg.wasm.gz` takes `_bg.wasm`'s cap.

## P. The page and the script

The design is `skein/docs/design/ui-web.md`, 4.3. The look's source is the request's commit, `0c1b78f3e1` or, if that hash has gone, the commit on `origin/look-requests` with the `Look-request: bundle-loading` trailer (`60102c6689` when this was written); its pages are `temper/docs/design/client/journeys/bundle-loading/`. Read them with `git show`. **Other requests on that branch are other plans'.**

### P1. The bundler's inputs (`page-skein`)

```rust
pub struct Inputs {
    // ...what it has
    /// The page's title; the application's name where `None`.
    pub title: Option<String>,
    pub placeholder: Option<Placeholder>,
}
/// What a page with a placeholder paints first (ui-web.md, 4.3).
pub struct Placeholder { pub markup: PathBuf, pub stylesheet: PathBuf }
```

- The binary: `--title WORDS`, `--placeholder PATH`, `--placeholder-stylesheet PATH`. One of the last two alone is the usage error.
- `Refusal::NoNoscriptMarker { markup: PathBuf }`: a markup without `{noscript}`. `Refusal::PlaceholderWithPage { page: PathBuf }`: an own page beside it.
- The placeholder's stylesheet takes the same name checks as a given one (`ReservedName`, `SameName`), goes in the directory by its file name, and is **not** among the loader's sheets.

### P2. The page

```html
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>TITLE</title>
<link rel="stylesheet" href="PREFIXDIGEST/PLACEHOLDER.css">
<script type="module" src="PREFIXDIGEST/boot.js"></script>
</head>
<body>
<div id="app"></div>
MARKUP, with {noscript} replaced by <noscript>WORDS</noscript>
</body>
</html>
```

With no placeholder the page is as it is today, but for `--title`, so that a bundle without either keeps its digest's inputs and its golden's page.

### P3. The loader

`skein/crates/skein-ui-bundle/loader.js`, skein's text, `include_str!`, is `boot.readable.js` from the request's commit, written short and with these differences. `boot.js` is this head, then that text:

```js
const GLUE = ["NAME.js", PLAIN, GZIPPED];
const WASM = ["NAME_bg.wasm", PLAIN, GZIPPED];
const SHEETS = [["skein-ui.css", PLAIN, GZIPPED], ["a.css", PLAIN, GZIPPED]];
```

`GZIPPED` is `PLAIN` again where there's no form. A name is written as a JavaScript string (`\` and `"` escaped; a control character in one is `Refusal::Type`).

The differences from `boot.readable.js`:

1. **Words:** every word it says comes from `#boot`'s data attributes, read once at the top:

   | Attribute | Holds | temper's words |
   |---|---|---|
   | `data-figure` | `{got}`, `{total}` | `{got} of {total} MB` |
   | `data-left` | `{left}` | ` · about {left} left` |
   | `data-seconds`, `data-minutes` | `{n}` | `{n} s`, `{n} min` |
   | `data-keeps` | | `Your browser keeps this, so next time it opens at once.` |
   | `data-waiting` | | `Waiting for the server…` |
   | `data-starting` | | `Starting…` |
   | `data-failed` | | `temper didn’t load` |
   | `data-dropped` | `{got}`, `{total}` | `The connection dropped at {got} of {total} MB.` |
   | `data-answered` | `{status}` | `The server answered {status}.` |
   | `data-silent` | | `The server didn’t answer.` |
   | `data-no-wasm` | | `This browser has no WebAssembly.` |
   | `data-no-start` | | `It didn’t start.` |

2. **The figure:** `TOTAL`, the plain sizes' sum, is the fraction's denominator as it is. `TRAVEL` is `TOTAL` or, once the module's answer says `Content-Encoding: gzip`, the third column's sum. `got` and `total` in the words are `got / TOTAL * TRAVEL` and `TRAVEL`, in MB to one decimal. Until the module's answer is in, the line is empty.
3. **`init` waits for the sheets:** `await sheets` before `glue.default({ module_or_path: answer })`.
4. **Handing over:** `boot.inert = true` beside `classList.add("out")`.

The rest is as it stands: the 250 ms is the stylesheet's; 8 s, 15 s, 3 s, 5 s; `dataset.state`, `start` and `failed`; `MutationObserver` on `#app`; the 200 ms before `remove()`.

### P4. temper's inputs (`page-temper`)

- `temper/crates/temper-client-shell-web/loading.html`: the request's `01.html` from `<div id="boot">` to its `</div>`, with: the words of P3 as attributes on `#boot`; `value="0"` on the `progress`; `#boot-line` and `#boot-note` empty; `{noscript}` in place of its `<noscript>`.
- `temper/crates/temper-client-shell-web/loading.css`: the request's `loading.css`, unchanged but for what the render check finds.
- The bundler's arguments, wherever temper's are written (`temper-client-bundle/build/steps.rs`, `scripts/bundle.sh` but its `--playground` branch, `tools/tiers/run`): `--title temper --placeholder .../loading.html --placeholder-stylesheet .../loading.css`.
- `temper/scripts/proof-live-github-managed.sh` and the gallery take neither: they load no client over a slow connection, and the page as it is today is what "given none" means.
