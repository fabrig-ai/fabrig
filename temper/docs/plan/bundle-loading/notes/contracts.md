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
