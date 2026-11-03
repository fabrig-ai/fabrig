+++
id = "ui-web-worker-page"
part = "ui-web"
kind = "feature"
needs = ["ui-web-surface"]
files = ["skein/crates/skein-ui-web/src/platform/worker.rs", "skein/crates/skein-ui-web/src/tests/worker.rs", "skein/crates/skein-ui-web/src/tests/worker/**"]
effort = "high"
minutes = 90
+++

# skein-ui-web: the worker's platform, its pure half

## Goal

What a page holds of its world, whichever carries the messages: the world's clock as the
platform gives it (moving while the world waits idle, at the speed, never back), the timer in
the world's time, the pace and a hidden tab's pause, the messages held until the worker is
ready, the small store in the page's memory, and the world's high waters
(`skein/docs/design/ui-web.md`, 18). Portable, so the worker's page half on wasm32 and
`skein-playground`'s bridge in memory both hold it, and tested on the host.

Acceptance:

- `skein_ui_web::platform::worker` holds exactly `notes/contracts.md`, section 5 (`Page` with
  its methods, `shown`) and section 6's `Heard` enum, portable here:
  `pub enum Heard { Printed(bridge::Printed), Stopped, Failed { words: Box<[u8]> }, Clock }`
  with `Clone, Debug, PartialEq, Eq` (contracts, 8.2).
- **The messages:** `new` holds `ToWorld::Start(start)` then `ToWorld::Pace(Speed::ONE)`;
  `send` and `pace` (and `visibility`) hold theirs after; `outgoing` gives nothing until
  `ready`, then everything held, in order, once.
- **The clock** (contracts, 5, "The clock's arithmetic"): before the world's first clock,
  `start.start` and instant 0; after a `Clock` heard at `b0`, while idle and not paused,
  `now + (b - b0) × speed / 1000` (whole nanoseconds, rounded down, checked arithmetic), at
  most `next_due`, the wall moved by the same; while busy or paused, the clock's `now` and
  `wall`. A pace change or a visibility change re-bases at its browser instant. Never back: a
  later reading below one already given gives the one given, until the world passes it.
- **The pace:** `pace` sets the frame's speed and sends it unless hidden; `visibility(true)`
  sends `Speed::PAUSED`, `visibility(false)` sends the frame's speed again; a repeat sends
  nothing.
- **The timer** (contracts, 5, "`timer`").
- **The high waters:** the largest `memory` and the longest `slice` of the clocks heard.
- **The store:** records whole in insertion order, a write of an existing key replacing it in
  place; `write` always `true` (its bound is `store::Store`'s).
- **`shown`:** `navigation`'s rule for a path on the origin, and nothing else (no `https`, no
  loopback, whatever the feature).
- Step tests in `src/tests/worker.rs` pass (ui-web.md, 18, "Tested"; playground.md, 8, "The
  worker's platform"):
  - `nothing_leaves_before_the_worker_is_ready`, then the start, the pace and the inputs in
    order;
  - `the_clock_moves_while_the_world_waits_idle`: at 1×, ½× (500), 10× (10000), up to the next
    due instant and no further;
  - `the_clock_holds_while_the_world_steps` and `the_clock_holds_while_paused`;
  - `the_clock_never_goes_back`: a world's clock behind what was given holds it until the world
    passes;
  - `a_hidden_tab_pauses_and_the_shown_tab_resumes_its_speed`;
  - `the_timer_is_the_browser_instant_the_clock_reaches_the_deadline`: at 1× and 10×, a
    deadline passed is at once, none paused, none while busy, none past the next due instant;
  - `the_high_waters_are_the_largest_heard`;
  - `the_store_keeps_records_in_memory`, through `store::Store` with a bound: a write past it
    refused by the `Store`;
  - `only_a_path_on_the_origin_is_shown`.
- `heavy cargo nextest run -p skein-ui-web --lib -E 'test(/worker::/)'` passes, then the
  crate's lib tests, their counts in the report; clippy native and for wasm32 clean.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, sections 1, 5, 6
  and 8.2.
- `skein/docs/design/ui-web.md`, 10 (the navigation's targets) and 18;
  `skein/docs/design/playground.md`, 3 and 5.
- `skein-ui-web`'s `src/navigation.rs` (the rule), `src/store.rs`, `src/hidden.rs` (a pure
  half's shape).

## Owns and may change

- Owns the files in the header; `src/platform/worker.rs` exists with its module comment and
  `platform.rs` declares it (task `ui-web-surface`), and `src/tests.rs` declares
  `mod worker;`.
- `src/navigation.rs` is declared `#[cfg(any(test, target_arch = "wasm32"))]` in `lib.rs`: if
  `shown` needs its rule natively, make the module unconditional in `lib.rs` (that one line,
  listed in the report) and call its function; do not copy the rule.
- Read only: `bridge.rs`, `store.rs`, `transport.rs`.

## Steps

1. `Heard`, `Page`'s messages and store; then the clock, the pace and the timer; `shown`.
2. The step tests.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`;
   `heavy cargo clippy -p skein-ui-web --all-targets -- -D warnings` and its wasm32 run; the
   citations check.
4. Commit (why: the worker's platform's pure half, which both of a page's platforms hold);
   push your branch.

## Report

As the implementer's text says, and also: the clock's and the timer's arithmetic in a few
lines, with how rounding goes.
