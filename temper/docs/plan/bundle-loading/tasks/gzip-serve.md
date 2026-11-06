+++
id = "gzip-serve"
part = "compression-serve"
kind = "feature"
needs = ["gzip-skein"]
files = [
  "jig/crates/jig-protocol-engine-clients-http/src/bundle.rs",
  "jig/crates/jig-protocol-engine-clients-http/src/bundle/**",
  "jig/testing/jig-test/src/bundle.rs",
  "jig/testing/jig-test-service/src/server/bundle.rs",
  "jig/testing/jig-test-service/src/tests/**",
  "temper/crates/temper-service/src/clients/bundle.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/tests/bundle.rs",
  "temper/crates/temper-service/src/tests/fixtures/bundle.rs",
  "temper/crates/temper-service/src/tests/forge_sign_in.rs",
  "temper/crates/temper-service/src/tests/startup.rs",
  "temper/crates/temper/src/bundle.rs",
  "temper/crates/temper/tests/suite/startup.rs",
  "temper/docs/help/reference/limits.md",
  "temper/scripts/bundle.sh",
]
effort = "high"
minutes = 75
+++

## Goal

jig's helpers and temper's server read both forms and answer by what the request takes, within bounds that hold both. With this, `temper serve` sends the module gzipped, which is worth having before any page changes.

A departure from the usual rule that a branch stays in its tree: this one crosses jig and temper, because `File`'s new field runs through both, and jig-test's reader and temper's are copies which refuse a bundle's `.gz` until they learn it.

Acceptance:

- A real bundle from the bundler is read, from a directory and through `embedded`, with each form on its file; a bundle without forms still reads.
- temper's server: a client asking gets the form with the right fields, a client not asking gets `NAME`, each under the page's policy; `NAME.gz` by name is not found.
- Both forms count against `clients.bundle_files` and `clients.bundle_bytes`; `standard`'s bytes are 32 MiB; `docs/help/reference/limits.md` says so.
- A `NAME.gz` with no `NAME` refuses the start; so does `x.txt.gz`.
- `scripts/bundle.sh --playground` makes its bundle with `--gzip off`.

## Inputs

- `temper/docs/plan/bundle-loading/notes/common.md`, and `notes/contracts.md`, C3, C4 and C5; `Call::accepts_gzip` of C2 is on the trunk.
- `skein/docs/design/ui-web.md`, 4.4. `jig/docs/design/protocol/service.md`, 11.4. `temper/docs/design/client/browser.md`, 2.2 and 2.6. `temper/docs/design/protocol/configuration.md`, section 7. `temper/docs/design/playground.md`, 7.

## Owns and may change

- Owns: `files` above.
- Status, as `notes/common.md` says: the **Not yet** lines of this plan in `jig/docs/design/protocol/service.status.md`, `temper/docs/design/client/browser.status.md`, `temper/docs/design/protocol/configuration.status.md` and `temper/docs/design/playground.status.md`.
- Readers: `File { kind, bytes }` literals take `gzip: None` wherever the compiler says (`rg 'File \{' -g '*.rs' jig temper`: `temper-service/src/tests/forge_sign_in.rs`, its `tests/fixtures/bundle.rs`, `jig-test-service/src/tests/fixtures/bundle.rs`, the two readers). `Answer`'s new arm where the compiler asks, in `temper-service/src/clients/` and its `playground_tests.rs`.
- Generated: `temper/docs/help/reference/limits.md`, by its own tool (`rg 'limits.md' temper/crates temper/tools`).
- Options: if `Map` has no `get_mut`, build the map after pairing, in the same function. If a test pins `16 << 20` for `standard`, update it (`temper-service/src/tests/startup.rs` does); a test's own `Clients { ... }` stays as it is.
- Not yours: `temper/tests/client-browser/**`; the page, the script and `temper-client-bundle/` (`--gzip` isn't passed there, so its default holds).

## Steps

Tests first (`notes/common.md`):

1. `jig-protocol-engine-clients-http/src/bundle/tests.rs`: `compressed_of` takes `a/x.js.gz` to `a/x.js`, and `.wasm`, `.css`, `.svg` the same way; takes nothing from `x.txt.gz`, `x.gz`, `x.js`, `x.js.gz.gz`. `kind` of `x.js.gz` is `None`.
2. `temper-service/src/tests/bundle.rs`, with the fixture's `boot.js` given a `gzip` of made-up bytes, since no inflater is here:
   - `route`'s table gains rows: an `Accept-Encoding: gzip` on `boot.js`, `Answer::Gzip`; `gzip;q=0`, `Answer::File`; on `robots`-less `module.wasm` with no form, `Answer::File`; `boot.js.gz`, `Answer::NotFound`.
   - `a_client_asking_gets_the_compressed_answer`: the form's bytes, the C4 fields, the page's policy (as the test beside it asserts it).
   - `a_client_not_asking_gets_the_plain_one`: the plain bytes, `Vary`, no `Content-Encoding`.
   - `a_file_with_no_form_says_neither_field`.
3. `temper/crates/temper/tests/suite/startup.rs`, in the tests beside `BUNDLE_LIMITS`, for both `read` and `embedded`:
   - `a_bundle_reads_each_form_onto_its_file`;
   - `the_compressed_forms_count_against_the_bounds`: one below each bound refuses, naming `past clients.bundle_files` and `past clients.bundle_bytes`;
   - `a_compressed_form_with_no_file_refuses_the_start`, `a compressed form with no file`;
   - `a_gz_of_another_type_refuses_the_start`.
4. `jig-test-service`: a test beside its bundle's tests, the same asking and not asking.

Then the code:

5. jig's `bundle.rs`: C3.
6. `jig-test/src/bundle.rs`, `jig-test-service/src/server/bundle.rs`: C4, C5.
7. `temper/crates/temper/src/bundle.rs`: pair in `read` and `embedded_read`, by one function. `embedded`'s order is the directory's, so `x.css` comes before `x.css.gz`; don't rely on that: gather forms and attach them at the end.
8. `temper-service/src/clients/bundle.rs`: `Answer::Gzip`, `route`, `content`, `serve_call`'s fields in C4's order (`List::with_capacity` grows to what it holds), `chunk`.
9. `limits.rs`: C5, and its comment. The help reference.
10. `temper/scripts/bundle.sh`: `--gzip off` on the playground's bundler line only. The tier's playground server reads its files as they are.
11. Check on a real bundle: make one with the dev bundler (`heavy cargo build -p skein-ui-bundle`) from any module `heavy cargo build --target wasm32-unknown-unknown -p temper-client-shell-web` gives, start `temper serve` naming it, and `curl -H 'Accept-Encoding: gzip' -D-` and plain, comparing `gunzip`'s bytes with the plain answer's. Say so in your report.
12. **A change of meaning runs its readers' tests:** once: `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and temper's; `heavy cargo clippy -p jig-protocol-engine-clients-http -p jig-test -p jig-test-service -p temper-service -p temper --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-engine-clients-http -p jig-test-service -p temper-service`, then `-p temper --test suite -E 'test(/bundle|startup/)'`; then `heavy cargo check --tests` naming `jig-client-browser-world` and `temper-client-browser-world`.
13. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit, push.

## Report

What ran; each rule broken by hand and what failed; `curl`'s answers from step 11; how many files a real temper bundle holds, against 128.
