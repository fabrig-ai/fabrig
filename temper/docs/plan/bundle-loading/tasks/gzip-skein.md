+++
id = "gzip-skein"
part = "compression-skein"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui-bundle/**",
  "skein/crates/skein-http/src/server.rs",
  "skein/crates/skein-http/src/tests.rs",
  "skein/examples/ui/server/src/**",
  "skein/tests/ui-web/**",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 60
+++

## Goal

Compression's generic half, in skein, which names no temper: the bundler writes a compressed form beside each `.wasm`, `.js`, `.css` and `.svg`; `Call` says whether a request takes gzip; the example's server answers by it, as any server may copy.

Acceptance:

- A bundle holds `NAME.gz` beside each such file and beside no other; each inflates to `NAME`; two runs give the same directory, byte for byte.
- An input named `.gz` refuses the bundle: nothing is compressed twice. `--gzip off` writes `NAME` alone.
- The example's server answers a client asking for gzip with the form and the right fields; one not asking gets `NAME`; decoded, the bytes are the same; `NAME.gz` by name is not found.
- `Call::accepts_gzip` holds the table in the contract.
- The bundler's time on a module the size of temper's debug one (11 MB) is said in your report.

## Inputs

- `temper/docs/plan/bundle-loading/notes/common.md`, and `notes/contracts.md`, C1, C2 and C4.
- `skein/docs/design/ui-web.md`, 4.2 and 4.4. `skein/docs/design/http.md`, 5.2. `skein/docs/design/examples.md`, 9.3.

## Owns and may change

- Owns: `skein-ui-bundle`, `Call` in `skein-http`'s `server.rs` with its test, the example's server, `skein/tests/ui-web`, and the workspace's `flate2` lines in `Cargo.toml` and `Cargo.lock`.
- Status, as `notes/common.md` says: in `skein/docs/design/http.status.md`, `ui-web.status.md` and `examples.status.md`, the **Not yet** lines of this plan. `ui-web.status.md`'s line goes whole.
- Readers, a literal's new field only: `temper/tools/gallery/src/lib.rs` takes `gzip: false` (its prints are opened as files, so no form is read, and it says so in a comment); `skein/tests/ui-web/src/lib.rs` and `tests/suite/serve.rs` take `gzip: true`, the default in `src/main.rs`'s arguments and in `src/tests.rs`'s `inputs`.
- Golden: `skein/crates/skein-ui-bundle/src/tests/golden/index.html` names a digest, which changes. Regenerate it as its test says.
- Options: if `skein_http::gzip::encode` comes within 5% of `flate2`'s best on the real module (step 8), take it, add no dependency, change `ui-web.md`, 4.2's line naming the compressor and C1's `Cargo.toml` work falls away, and say so; otherwise `flate2`, as C1 says. If `flate2`'s `GzBuilder` won't give stable bytes, write the member by hand over `miniz_oxide` and say so. If the example's `Bundle::new` signature is awkward with a second map, change it; its tests are yours. Where another crate of skein calls `Bundle::new`, change the call site only.
- Not yours: `jig/**`, `temper/**` but the gallery line above. A bundle with forms is refused by jig's and temper's readers until `gzip-serve` merges; that is expected, and their browser tiers aren't yours to run.

## Steps

Tests first (`notes/common.md`):

1. `skein-ui-bundle/src/tests.rs`, with `flate2` as a dev dependency read through its `read::GzDecoder`:
   - `the_bundler_writes_a_compressed_form_beside_each_text`: each `.wasm`, `.js`, `.css`, `.svg` of `inputs(...)`'s bundle has its `.gz`, inflating to it; `files/about.txt` has none.
   - `a_compressed_form_is_the_same_bytes_each_time`: two runs, trees equal; bytes 4 to 8 of a `.gz`, its time, are zero, and its flags byte names no file.
   - `the_digest_covers_the_compressed_forms`: `gzip` true and false give different digests.
   - `a_gz_input_refuses_the_bundle`: a stylesheet, a file and a picture so named, each `Refusal::Type`.
   - `gzip_off_writes_the_plain_files_alone`.
   - `main.rs`'s `the_arguments_make_inputs`: `--gzip off` in the complete arguments, and `--gzip on` and `--gzip` alone in its usage table.
2. `skein-http/src/tests.rs`, beside its `call` helper: a table test of C2, named `a_request_takes_gzip_by_its_accept_encoding`.
3. `skein/tests/ui-web/tests/suite/serve.rs`, on a real bundle through `routes::route`, with `flate2` as a dev dependency:
   - `a_client_asking_gets_the_compressed_answer`: for `notes.js` and `notes_bg.wasm`, with `Accept-Encoding: gzip`, the body is the `.gz` file, inflating to the plain bytes; `Content-Encoding: gzip`, `Vary: Accept-Encoding`, `NAME`'s type, the immutable `Cache-Control`.
   - `a_client_not_asking_gets_the_plain_one`: no field, `identity`, `gzip;q=0`: the plain bytes, `Vary: Accept-Encoding`, no `Content-Encoding`.
   - `a_compressed_form_has_no_address`: `GET /bundle/<digest>/notes.js.gz` is 404; the page, asked with gzip, stays plain.
   - `skein_ui_web_tests::get` takes no headers; add a helper beside it that does.

Then the code:

4. `skein-ui-bundle`: `types::gzipped`; `Inputs.gzip`; in `bundle_with`, once every file is in `files` and before `digest::of`, insert each `.gz`. They must not pass through `insert`'s closed-list check; a private path for them is the way. `--gzip`.
5. `skein-http`: `Call::accepts_gzip`, under the strict subset: no `std`, no indexing.
6. The example's server: `bundle.rs` keeps each form apart from the files, pairing a `.gz` with its file as C4 says; `routes.rs` answers. Its bound on bytes counts both. Its `BUNDLE_BYTES` and `FILES` grow only if a bundle of its own needs it.
7. `Cargo.toml`, `Cargo.lock`: as C1. Check it's nowhere but the tool: `cargo tree -p skein-ui-example-server -e normal` holds no `flate2`.
8. Time the bundler on an 11 MB `.wasm` (`heavy cargo build -p skein-ui-bundle`, then run it on a debug module you've built). If it takes above 20 s in the dev profile, find out why: the `opt-level` lines are what should cover it. On the same module, in a throwaway, compare `flate2`'s best with `skein_http::gzip::encode` (`skein/crates/skein-http/src/gzip.rs`), the workspace's encoder for request bodies, which trades ratio for speed, as the draft didn't know when it said the workspace has no compressor.
9. Narrow tests; then, once: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-bundle -p skein-http -p skein-ui-example-server -p skein-ui-web-tests --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-ui-bundle -p skein-ui-example-server -p skein-ui-web-tests`, and `-p skein-http`. Check the gallery compiles: `heavy cargo check -p temper-client-gallery --tests`, and other dependents of `Inputs` by `rg skein_ui_bundle`.
10. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit, push.

## Report

What ran; each rule broken by hand and what failed; the bundler's time; `flate2`'s version; the size of a real module's `.gz` if you built one.
