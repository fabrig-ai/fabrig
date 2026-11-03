+++
id = "playground-worker"
part = "worker"
kind = "feature"
needs = ["playground-visitor"]
files = [
  "temper/crates/temper-playground/Cargo.toml",
  "temper/crates/temper-playground/src/lib.rs",
  "temper/crates/temper-playground/src/worker.rs",
  "temper/crates/temper-playground/src/programs.rs",
  "temper/crates/temper-playground/src/tests/**",
  "Cargo.lock",
]
effort = "high"
minutes = 60
+++

# `temper-playground` on wasm32 is the worker's module

## Goal

"On wasm32 its start, the worker's module's entry, makes the world from the start message's
story, seed and start, and hands it to skein's slices" (`temper/docs/design/playground.md`, 3).
Give `temper-playground` the worker's entry over skein's `worker::run` (`notes/contracts.md`,
C4), on the composition in slices that `playground-visitor` made (C1, C2).

Acceptance:

- C4 exactly: `worker::start` (wasm32), its private `make`, `programs::install` (every target),
  the crate a `cdylib` and an `rlib`, `wasm-bindgen` for wasm32 only.
- `heavy cargo build --target wasm32-unknown-unknown -p temper-playground` writes
  `temper_playground.wasm`, and `heavy cargo clippy --target wasm32-unknown-unknown -p
  temper-playground -- -D warnings` passes.
- A native test (`src/tests/world.rs` or a new `src/tests/worker.rs`) of what `make` does on every
  target, through a portable function `make` calls (the implementer names it, e.g.
  `story::start_slices(start: &bridge::Start, browser: Time) -> Option<Slices<..>>`): a start
  naming `a` gives slices whose first slices post the printed link; a start naming an unknown
  story gives none (the wasm32 `make` panics on it with the story's name in its words).
- `programs::install` then `programs::with` nests as `with` alone does: a test shows a scoped
  world's parameters win inside `with` and the installed ones return after it.
- Natively nothing else changes: `heavy cargo nextest run -p temper-playground -p
  temper-playground-world` passes with the trunk's counts plus yours.

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md`;
  `temper/docs/plan/playground-browser/notes/contracts.md`, C2 and C4.
- `temper/docs/design/playground.md`, 3 and 4; `skein/docs/design/playground.md`, 5 (the
  worker says it is ready, the start message) and 7 (the worker's module).
- `skein/crates/skein-playground/src/worker.rs` (`run`), `skein/crates/skein-ui-web/src/bridge.rs`
  (`Start`); `temper/crates/temper-client-shell-web/Cargo.toml` for the `cdylib` and wasm32
  dependency form temper uses.

## Owns and may change

- The files in `files`; `Cargo.toml` gains `[lib] crate-type = ["cdylib", "rlib"]` and
  `[target.'cfg(target_arch = "wasm32")'.dependencies] wasm-bindgen.workspace = true`.
- `src/story.rs` and `src/stories.rs` only to add the portable function `make` calls, if you put
  it there; their C1 and C2 items keep their signatures.
- Pre-approved options:
  - If a `cdylib` crate type makes a native test binary or a reader's build fail (a duplicate
    symbol, a link error), keep `cdylib` and report the error and its fix; if no fix is local,
    stop and ask.
  - If wasm32 clippy fails in a crate of another tree at your base, do not chase it: report the
    first error with file and line, and meet the rest of your acceptance (plan `wasm32-temper`
    measures the same build).
  - If `programs::with`'s reset would clobber installed parameters, make `with` restore what was
    there before it (installed or none) rather than none, and say so.

## Steps

1. `programs::install` and `with`'s restore; their test.
2. The portable start of slices from a `bridge::Start`, and its test.
3. `worker.rs` on wasm32, the crate types and the wasm32 dependency; the wasm32 build and clippy.
4. `cargo fmt`, the old-form count, the native clippy and tests; commit per step; push.

## Report

As the implementer's text says, and also: the wasm32 module's size in bytes in the dev profile,
and whether wasm32 clippy passed at your base and at your head.
