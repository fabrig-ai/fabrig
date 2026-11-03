+++
id = "ui-web-worker-web"
part = "ui-web"
kind = "feature"
needs = ["ui-web-worker-page", "ui-web-log"]
files = ["skein/crates/skein-ui-web/src/platform/worker/web.rs", "skein/crates/skein-ui-web/src/platform/worker/web/**", "skein/crates/skein-ui-web/src/platform/worker.rs", "skein/crates/skein-ui-web/src/platform.rs", "skein/crates/skein-ui-web/Cargo.toml"]
effort = "high"
minutes = 90
+++

# skein-ui-web: the worker's platform, its page half

## Goal

`platform::Worker<R>`, the page's platform for a world in a Web Worker: it starts the worker
from the bundle, holds the page's messages until the worker says it is ready, carries the
transport's duties over the bridge, raises the world's reports in the inbox as fetch's own,
keeps the session log, gives the world's clock and the timer in the world's time, pauses the
world while the tab is hidden, and ends the worker on a reset, a new story or a failure
(`skein/docs/design/ui-web.md`, 18; `skein/docs/design/playground.md`, 5 to 7). wasm32 only:
its logic is `Page`'s and `Log`'s, tested natively by their tasks.

Acceptance:

- `skein_ui_web::platform::Worker<R>` and `platform::worker_address` exactly as
  `notes/contracts.md`, section 6 gives them, in `src/platform/worker/web.rs`, declared in
  `worker.rs` under `#[cfg(target_arch = "wasm32")] mod web;` and re-exported from
  `platform.rs` beside `Browser` (`pub use worker::{Worker, worker_address};` under the same
  cfg). `Heard` stays `worker.rs`'s, portable.
- **Starting:** `new Worker(address, {type: "module"})`; its `onmessage` handler in before
  anything is posted. An empty message (`bridge::READY`) is the worker ready: `Page::ready`,
  then everything `Page::outgoing` gives, each `ToWorld` encoded, one `postMessage` each, its
  buffer transferred. Every later message decodes as `ToPage` (a `Malformed` one is a broken
  contract: the platform raises `Heard::Failed` with words saying so and ends the worker),
  goes to `Page::heard` and `Log::heard` at the browser's monotonic instant, then raises:
  `ToPage::Report(r)` through `fetch`; `Printed`, `Stopped`, `Failed` through `heard`;
  `Clock` through `heard` as `Heard::Clock` after the clock moved, and the timer set again;
  `Stamp` raises nothing.
- **The worker's `onerror`** (a module that failed to load, or trapped with no words) raises
  `Heard::Failed` with the event's message as words and ends the worker.
- **`Platform for Worker<R>`:** `send`, `stream`, `abandon`, `read_on` each become one
  `bridge::Input`, kept by `Log::sent` and sent through `Page::send` (posted at once when
  ready); `read_all`, `write`, `remove` are `Page`'s; `navigate` shows a target `shown`
  accepts in the address bar with `history.replaceState` and follows nothing else.
- **The clock and the timer:** `clocks` is `Page::clocks` at the browser's monotonic instant
  (`platform::web`'s `clocks` reading, its first half); `timer(deadline)` keeps the deadline
  and sets one browser timeout at `Page::timer`'s instant, cleared and set again at each
  clock heard, each pace and visibility change, raising `due()` when it fires (as
  `timer::web::Timer` does for the browser).
- **The hidden tab:** `visibility` is `Page::visibility`; the shell wires the document's
  `visibilitychange` to it (the platform registers nothing of its own there).
- **`end`:** the worker terminated, its handlers dropped, nothing more raised.
- **`worker_address`:** the page's module script whose `src` ends in `/boot.js`, that
  directory with `boot-worker.js` (`skein_ui_bundle::WORKER_STARTER`'s value, written here as
  a literal: `skein-ui-web` does not depend on the bundler); `None` when no such script.
- `Cargo.toml`'s `web-sys` features gain exactly what the half uses (`Worker`,
  `WorkerOptions`, `WorkerType`, `MessageEvent`, `ErrorEvent`, `HtmlScriptElement`, and the
  like), listed in the report.
- `heavy cargo clippy -p skein-ui-web --target wasm32-unknown-unknown -- -D warnings` and the
  native `heavy cargo clippy -p skein-ui-web --all-targets -- -D warnings` are clean;
  `heavy cargo nextest run -p skein-ui-web --lib` still passes; the wasm32 clippy of
  `skein-ui-example-shell-web`, `jig-client-test-shell-web` and `temper-client-shell-web`
  still passes (the feature list changed).

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, sections 1, 4, 5
  and 6.
- `skein/docs/design/ui-web.md`, 5.1 (the parts), 10 and 18; `skein/docs/design/playground.md`,
  5 ("The worker says it is ready"), 6 and 7.
- Models: `skein-ui-web`'s `src/fetch/web.rs` (callbacks raising in the inbox),
  `src/timer/web.rs`, `src/hidden/web.rs`, `src/platform/web.rs`.

## Owns and may change

- Owns the files in the header: in `worker.rs` and `platform.rs`, the module and re-export
  lines only; in `Cargo.toml`, the `web-sys` feature list only.
- Read only: `Page`, `Heard`, `Log`, `bridge` (a change any of them needs is a question).
- Pre-approved: if transferring the buffer needs a `js_sys::Uint8Array` copy first, copy it
  and transfer the copy's buffer; if `postMessage` with a transfer list is not in the pinned
  `web-sys`, post without the transfer and say so.

## Steps

1. Starting, the ready message and the posts; the reports raised; the log kept.
2. The platform's duties and store; the clock, the timer, the hidden tab, `end`;
   `worker_address`.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; the clippy runs above; the
   citations check.
4. Commit (why: a page runs its world in a worker through the platform its shell holds);
   push your branch.

## Report

As the implementer's text says, and also: the `web-sys` features added, and how a message is
posted (copy, transfer).
