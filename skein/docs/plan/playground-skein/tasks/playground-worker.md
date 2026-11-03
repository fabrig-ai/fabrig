+++
id = "playground-worker"
part = "playground"
kind = "feature"
needs = ["playground-slices"]
files = ["skein/crates/skein-playground/src/worker.rs", "skein/crates/skein-playground/src/worker/**", "skein/crates/skein-playground/Cargo.toml"]
effort = "high"
minutes = 75
+++

# skein-playground: the worker's parts on wasm32

## Goal

The worker module's loop, which an application's worker start hands its world to: it says it
is ready, waits for the start message, makes the world through the application's function,
then runs a slice after each message and on its one timer, posting each of the world's
messages encoded with its buffer transferred, its memory and its slices measured; a panic
posts the world's failure before the module traps (`skein/docs/design/playground.md`, 3, 5
and 7). wasm32 only; the application's browser tier runs it.

Acceptance:

- `skein_playground::worker::run` exactly as `notes/contracts.md`, section 8.4 gives it, in
  `src/worker.rs` (and private files under `src/worker/`), the module `#[cfg(target_arch =
  "wasm32")]` as `playground-crate` declared it.
- **Starting:** the panic hook first (below); the `DedicatedWorkerGlobalScope`'s `onmessage`
  handler in; then one empty message posted (`bridge::READY`, playground.md, 5, "The worker
  says it is ready"). The first message must decode as `ToWorld::Start`: `make(&start,
  browser)` makes the `Slices`, `browser` the worker's `performance.now()` as a
  `skein_lib::Time`. Any other first message, or a `Malformed` one, is a broken contract: a
  panic, which the hook reports.
- **Each later message:** `ToWorld::Input` to `Slices::input`, `ToWorld::Pace` to
  `Slices::pace` at the browser instant, a second `Start` a broken contract; then a slice at
  once (playground.md, 5, "A slice's posts, in order").
- **A slice:** `Slices::slice(browser, &mut posts)`; its duration in browser time measured
  around it; the closing `Clock`'s `memory` set to the module's linear memory in bytes
  (`core::arch::wasm32::memory_size::<0>()` pages × 65 536, checked) and its `slice` to the
  duration; each post encoded (`ToPage::encode`) and posted, one `postMessage` each, its
  buffer transferred.
- **The one timer:** after each slice, from its `Wake`: `Now`, a timeout of 0; `At(b)`, a
  timeout for `b` less the browser's now, rounded up to the millisecond; `Message`, none.
  A message clears the timeout and runs its slice; the timeout's callback runs a slice.
- **The panic hook** (`std::panic::set_hook`): it posts `ToPage::Failed { words }` with the
  panic's message and location, then lets the module trap (playground.md, 7, "A panic is
  fail-stop"). It holds nothing of the loop's, so a panic inside a slice can still post.
- No `unsafe`; the loop's state in one `Rc<RefCell<..>>` whose second borrow panics,
  fail-stop, as the reference shell keeps its own (ui-web.md, 5.3).
- `Cargo.toml`'s wasm32 `web-sys` features gain what the parts use beyond those
  `playground-crate` listed, each listed in the report.
- `heavy cargo clippy -p skein-playground --target wasm32-unknown-unknown -- -D warnings` and
  the native clippy over `skein-playground` are clean; `heavy cargo nextest run -p
  skein-playground-world` still passes.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, sections 1 and 8.
- `skein/docs/design/playground.md`, 3, 5 and 7; `skein/docs/design/ui-web.md`, 4.1 (the
  toolchain) and 5 (a panic is fail-stop).
- The trunk's `slices.rs` as its task left it; models: `skein-ui-web`'s `src/timer/web.rs` and
  `src/fetch/web.rs` (closures over the glue).

## Owns and may change

- Owns the files in the header; in `Cargo.toml`, the wasm32 dependencies' lines only.
- Read only: `slices.rs`, `skein-ui-web`.
- Pre-approved: if the pinned `web-sys` lacks `post_message_with_transfer` on the worker's
  scope, post without the transfer and say so; if `make` needs the browser instant otherwise
  than as a `Time` from `performance.now()`, it is a question.

## Steps

1. The hook, the handler, the ready message, the start.
2. A slice, its measures and posts; the timer.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; the clippy runs; the citations
   check.
4. Commit (why: a worker module runs an application's world in slices on the browser's
   clock); push your branch.

## Report

As the implementer's text says, and also: the `web-sys` features added, and how the timer and
the transfer are done.
