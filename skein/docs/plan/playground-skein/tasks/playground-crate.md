+++
id = "playground-crate"
part = "playground"
kind = "interface"
needs = []
files = ["skein/crates/skein-playground/**", "skein/tests/playground/**", "Cargo.toml", "Cargo.lock"]
effort = "medium"
minutes = 40
+++

# skein-playground: the crate and its world, laid down

## Goal

Make the crate `skein-playground` and its world `skein-playground-world`, empty but for their
manifests and module layout, so the tasks that fill them (the visitor, the slices, the bridge
in memory, the replay, the worker's parts) each own disjoint files from their start
(`skein/docs/design/playground.md`, 2; `notes/contracts.md`, section 8). Nothing here
behaves: no public item but the modules.

Acceptance:

- `skein/crates/skein-playground/Cargo.toml`: package `skein-playground`, description "A
  simulated world run in a browser's worker, and its visitor", the workspace's version,
  edition, licence, rust-version and publish; `[lints] workspace = true`; dependencies
  `skein-lib`, `skein-io`, `skein-http`, `skein-sim`, `skein-sim-world`, `skein-shell` and
  `skein-ui-web`, each `.workspace = true`; and under
  `[target.'cfg(target_arch = "wasm32")'.dependencies]`, `wasm-bindgen`, `js-sys` and
  `web-sys` with the features `DedicatedWorkerGlobalScope`, `WorkerGlobalScope`,
  `MessageEvent`, `Performance`.
- `skein/crates/skein-playground/clippy.toml`: a copy of
  `skein/crates/skein-sim-world/clippy.toml` (ordinary Rust, programming-model.md, 10.2), its
  first comment naming the playground's kit instead of the world harness.
- `src/lib.rs`: its module comment
  `//! skein's playground kit: a simulated world run whole in a browser's worker (playground.md).`,
  `#![forbid(unsafe_code)]`, `extern crate alloc;`, and the modules
  `pub mod visitor; pub mod slices; pub mod memory; pub mod replay;` and
  `#[cfg(target_arch = "wasm32")] pub mod worker;`, each file holding its module comment
  alone:
  - `src/visitor.rs`: `//! The visitor, a program of the world speaking HTTP to the application's listener (playground.md, 4).`
  - `src/slices.rs`: `//! The worker's loop over a world, in slices paced by the browser (playground.md, 3).`
  - `src/memory.rs`: `//! The bridge in memory, for a native test (playground.md, 5).`
  - `src/replay.rs`: `//! A session log replayed natively (playground.md, 6).`
  - `src/worker.rs`: `//! The worker's parts on wasm32: its messages in, its timer and its posts (playground.md, 7).`
- `skein/tests/playground/Cargo.toml`: package `skein-playground-world`, description "The
  playground kit's world: the visitor against skein's HTTP server, the slices and the
  replay", the workspace's fields; dependencies `skein-fail-stop`, `skein-lib`, `skein-io`,
  `skein-http`, `skein-sim`, `skein-sim-world`, `skein-shell`, `skein-world`, `skein-heap`,
  `skein-ui-web`, `skein-playground`; the `[lints.rust]` and `[lints.clippy]` tables of
  `skein/tests/world/Cargo.toml` (no `[[bin]]`); `[lib] test = false, doctest = false`.
- Its `src/lib.rs`: `//! The playground kit's world (playground.md, 8).`, and
  `pub mod scripted; pub mod site;`, the files holding their comments alone:
  `src/scripted.rs`: `//! A scripted stand-in for the visitor, for the slices' tests.`;
  `src/site.rs`: `//! A world of the visitor and skein's HTTP server, scripted.`
- Its `tests/suite/main.rs`: `//! The playground kit's world tests (playground.md, 8).`, the
  counting allocator as `skein/tests/world/tests/suite/main.rs` declares it
  (`#[global_allocator] static HEAP: skein_heap::Counting = skein_heap::Counting;`, or that
  file's own form), and `mod visitor; mod slices; mod memory; mod replay;`, each file under
  `tests/suite/` holding a module comment alone.
- The root `Cargo.toml`: `"skein/crates/skein-playground"` in `members` after
  `"skein/crates/skein-oauth-issuer"`'s neighbourhood, in the list's order (alphabetical
  among `skein/crates/`), `"skein/tests/playground"` likewise among `skein/tests/`; and
  `skein-playground = { path = "skein/crates/skein-playground" }` in
  `[workspace.dependencies]` beside `skein-sim-world`'s line. `Cargo.lock` as cargo writes
  it.
- `heavy cargo check -p skein-playground -p skein-playground-world --tests` and
  `heavy cargo clippy -p skein-playground -p skein-playground-world --all-targets -- -D warnings`
  pass; `heavy cargo clippy -p skein-playground --target wasm32-unknown-unknown -- -D warnings`
  passes; `heavy cargo nextest run -p fabrig-boundary` passes (the dependency direction reads
  the members). An unused dependency warning, if a lint raises one, takes a scoped
  `#[expect]` in `lib.rs` with the reason "used by the modules this crate's tasks fill", or,
  for the wasm32 target dependencies, the dependency moves to the worker task: say which in
  the report.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, section 8.
- `skein/docs/design/playground.md`, 2 and 8; `skein/docs/design/world.md`, 2 (the product
  world and the harness).
- As models: `skein/crates/skein-sim-world/Cargo.toml` and `clippy.toml`,
  `skein/tests/world/Cargo.toml` and `tests/suite/main.rs`.

## Owns and may change

- Owns the files in the header; in the root `Cargo.toml`, the three lines above only, and in
  `Cargo.lock` what cargo writes for the two packages.
- Other plans may add their own lines to the root `Cargo.toml`: keep to your lines, in the
  list's order.

## Steps

1. The two crates' manifests, `clippy.toml`, and module files; the root lines.
2. The checks and clippy runs above; `heavy cargo nextest run -p fabrig-boundary`.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; the citations check.
4. Commit (why: the playground kit's crate and world, laid down for its tasks); push your
   branch.

## Report

As the implementer's text says, and also: any lint the empty modules raised and what you did.
