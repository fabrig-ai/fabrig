+++
id = "playground-memory"
part = "memory"
kind = "feature"
needs = ["playground-slices", "playground-visitor", "ui-web-worker-page", "ui-web-log"]
files = ["skein/crates/skein-playground/src/memory.rs", "skein/crates/skein-playground/src/memory/**", "skein/tests/playground/tests/suite/memory.rs", "skein/tests/playground/tests/suite/memory/**", "skein/tests/playground/src/native.rs"]
effort = "high"
minutes = 90
+++

# skein-playground: the bridge in memory

## Goal

The bridge in memory: a `Platform` that hands the transport's duties to the slices in the
same thread, as the same messages, and raises their reports as the worker's platform does,
so an application's native test drives its client's host against its world through the
visitor, as its page does in a browser, with no worker and no browser
(`skein/docs/design/playground.md`, 5, "Natively, the bridge is in memory"). This is the
plan's meeting point: a page's exchange answered by skein's HTTP server in the world, both
real sides, natively.

Acceptance:

- `skein_playground::memory::Memory<P, M, R>` exactly as `notes/contracts.md`, section 8.2
  gives it, holding a `platform::worker::Page`, the `Slices`, a `log::Log` (its build `None`)
  and the inbox.
- **The platform's duties** (`skein_ui_web::platform::Platform`): `send`, `stream`,
  `abandon`, `read_on` each become one `bridge::Input`, kept by `Log::sent`, handed to the
  slices through `Page::send` and `Page::outgoing` (the worker is ready at once: `new` calls
  `Page::ready`); the store calls are `Page`'s; `navigate` follows nothing (natively there is
  no address bar) and is kept, readable by a test through `shown`, the last target
  `worker::shown` accepted.
- **A slice** (`Memory::slice(browser)`): what `Page::outgoing` holds goes to the slices first
  (`ToWorld::Input` to `Slices::input`, `ToWorld::Pace` to `Slices::pace`, `ToWorld::Start`
  ignored: the world was made from it by the caller), then one `Slices::slice(browser, ..)`;
  each post goes to `Page::heard` and `Log::heard`, then raises in the inbox as
  `ui-web-worker-web` raises it (contracts, section 6: `Report` through `fetch`, `Printed`,
  `Stopped`, `Failed`, `Clock` through `heard`; `Stamp` nothing). It returns the slices'
  `Wake`.
- `pace`, `clocks`, `timer` and `log` are `Page`'s and `Log`'s.
- **The world tests** in `tests/suite/memory.rs`, over the site world of
  `skein/tests/playground/src/site.rs` (its process enum and scripted server, built with
  `skein_sim_world::World` instead of the harness's; if site's public items do not serve,
  compose the same processes in `src/native.rs`, declared in `src/lib.rs` with one line), the
  test standing for a client's shell: an inbox of its own report type, the transport
  (`skein_ui_web::transport::carry`) over the `Memory`, and browser instants it chooses:
  - `an_exchange_through_the_bridge_in_memory_is_answered`: an exchange carried by the
    transport, slices until `Wake::Message`, the `Answered` report raised in the inbox with
    the server's status and body;
  - `the_stream_is_read_on_through_the_platform`: opened, a piece, `read_on` through the trait,
    the next piece, closed;
  - `the_clock_is_the_worlds`: `clocks` after slices is the world's clock, and moves while the
    world waits idle at the paced speed;
  - `the_log_kept_natively_reads_back`: `skein_ui_web::log::read(&memory.log())` gives each
    input sent, stamped as the slices stamped it, its build `None`, its end the last clock;
  - `a_navigation_is_not_followed`.
- `heavy cargo nextest run -p skein-playground-world -E 'test(/memory::/)'` passes, its count
  in the report; clippy over both packages and the wasm32 clippy of `skein-playground` are
  clean; within the default profile's budget.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, sections 1, 4, 5,
  6 and 8.
- `skein/docs/design/playground.md`, 5 and 6; `skein/docs/design/ui-web.md`, 5.1, 10 and 18.
- The trunk's `skein_playground::slices`, `skein_playground::visitor`,
  `skein_ui_web::platform::worker` and `skein_ui_web::log`, and
  `skein/tests/playground/src/site.rs`, as their tasks left them.

## Owns and may change

- Owns the files in the header; `skein/tests/playground/src/lib.rs` may gain
  `pub mod native;` alone.
- Read only: `slices.rs`, `visitor.rs`, `site.rs`, `scripted.rs`, `skein-ui-web` (a change any
  needs is a question).
- Pre-approved: if `Memory` reads clearer holding the inbox's two wrapping functions in a
  private struct, do so; if a test needs the world, reach it through `slices()`.

## Steps

1. `Memory` and its `Platform` impl; `slice`.
2. The world tests, the exchange's first.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; the clippy runs; the citations
   check.
4. Commit (why: a native test drives a client against its world through the visitor, as its
   page does); push your branch.

## Report

As the implementer's text says, and also: the shape of the test's stand-in shell in a few
lines, which an application's native test copies.
