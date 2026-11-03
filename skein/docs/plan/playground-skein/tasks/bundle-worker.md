+++
id = "bundle-worker"
part = "bundle"
kind = "feature"
needs = []
files = ["skein/crates/skein-ui-bundle/src/lib.rs", "skein/crates/skein-ui-bundle/src/page.rs", "skein/crates/skein-ui-bundle/src/main.rs", "skein/crates/skein-ui-bundle/src/tests.rs", "skein/crates/skein-ui-bundle/src/tests/**"]
effort = "high"
minutes = 60
+++

# skein-ui-bundle: a bundle's worker module

## Goal

The bundler takes a worker module beside the page's and writes its starter, its glue, its
module and its snippets into the same digest directory, so a page starts it by its address
under the directory (`skein/docs/design/playground.md`, 7, "The bundle" and "The worker's
files"). A bundle without a worker is byte for byte what it is today.

Acceptance:

- `skein_ui_bundle` gains exactly `notes/contracts.md`, section 7: `Worker { name, glue }`,
  `bundle_with(inputs, glue, worker, out)` and `WORKER_STARTER`; `bundle(inputs, glue, out)`
  keeps its signature and is `bundle_with(inputs, glue, None, out)`.
- With a worker named `world`, the digest directory holds, beside today's files:
  `boot-worker.js`, whose bytes are exactly `import init from "./world.js";\ninit();\n` (the
  page's starter's form, `page::starter`), `world.js` and `world_bg.wasm` from the worker's
  glue, and its snippets under `snippets/`. A snippet at a path the page's glue holds with the
  same bytes is written once; with other bytes, `Refusal::SameName` naming the snippet's path.
  A worker named as the page's module (`inputs.name`), or a worker name that is not one
  segment as `--name`'s check says, refuses: `SameName` naming `<name>.js` for the first; the
  CLI's usage for the second.
- The digest covers the worker's files as every other file of the directory; the page
  (`index.html`) is unchanged by a worker (the page starts the worker itself).
- The binary gains `--worker NAME=PATH`, at most once: it generates the worker's glue with
  `glue(PATH, NAME)` and calls `bundle_with`. Its usage line names it.
- Step tests in `src/tests.rs` (or a `src/tests/worker.rs` it declares) pass:
  - `a_worker_is_bundled_beside_the_page`: the files above, the starter's exact bytes, the
    page's `index.html` equal to the same inputs' bundle without a worker;
  - `a_worker_changes_the_digest`: with and without a worker, two digests; the same inputs with
    the same worker, one digest and the same bytes;
  - `a_bundle_without_a_worker_is_unchanged`: `bundle` and `bundle_with(.., None, ..)` give the
    same directory, byte for byte;
  - `a_shared_snippet_is_kept_once` and `a_clashing_snippet_is_refused`;
  - `a_worker_named_as_the_page_is_refused`;
  - the CLI's argument test (`main.rs`'s `the_arguments_make_inputs`) extended for `--worker`:
    accepted once, refused twice and without `=`.
- `heavy cargo nextest run -p skein-ui-bundle` passes, its count in the report;
  `heavy cargo clippy -p skein-ui-bundle --all-targets -- -D warnings` is clean.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, section 7.
- `skein/docs/design/playground.md`, 7; `skein/docs/design/ui-web.md`, 4.2 and 4.3 (the
  bundle, the page and the starter).

## Owns and may change

- Owns the files in the header.
- Readers, read only, which must keep building unchanged: `skein/tests/ui-web/src/lib.rs`,
  `skein/tests/ui-web/tests/suite/serve.rs`, `temper/tools/gallery/src/lib.rs` (each calls
  `bundle` and builds `Inputs` and `Glue` literals). Check them:
  `heavy cargo check -p skein-ui-web-tests -p temper-client-gallery --tests`.
- Pre-approved: if `bundle`'s body is clearer split into private helpers to serve both entry
  points, split it; if the worker's `Glue` needs a field the page's does not, it is a
  question, not a new field.

## Steps

1. `Worker`, `WORKER_STARTER`, `bundle_with` and the worker's files; `bundle` delegating.
2. The CLI's `--worker`.
3. The step tests; run them alone, then the crate's.
4. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; the clippy run; the readers'
   check; the citations check.
5. Commit (why: a playground's bundle carries its world's worker module); push your branch.

## Report

As the implementer's text says, and also: the files a bundle with a worker holds, as listed by
your first test.
