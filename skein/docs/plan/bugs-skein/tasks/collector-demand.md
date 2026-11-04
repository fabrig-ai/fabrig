+++
id = "collector-demand"
part = "json"
kind = "feature"
needs = []
files = ["skein/crates/skein-json/src/collector.rs", "skein/crates/skein-json/src/tests/collector.rs"]
effort = "high"
minutes = 35
+++

# skein-json's collector keeps its hands off the caller's queue

## Goal

`skein-json`'s collector (`src/collector.rs`, near 347) takes back a demand with `below.pop()`, from the
caller's queue. `Queue::pop` takes the oldest item, so with another item queued it takes that instead, or
panics. No caller reaches this today: each enters with `below` empty. (Bug 12, re-checked: holds, latent.)

Acceptance:

- `Collector` has a private `demands: Queue<Down>`, the pattern of its `events`.
- `up` calls `json::up` into `collector.demands`. Past the budget it clears them, stops the tokenizer into a
  local throwaway queue (the terminal path only) and reports `Failed`. Otherwise the one demand goes to
  `below` before `drive`: at most one output, which capacity-1 queues rely on.
- `collector_bound` adds `Queue::<Down>::price(1)`.
- No public signature changes.
- The test below fails on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/json.md`, 3.2, 5.1 ("at most one event above and one request below each").

## Owns and may change

- Owns: `src/collector.rs`, `src/tests/collector.rs`.
- `collector_bound`'s readers are additive or `is_some` (`skein-llm/src/document.rs`,
  `skein-llm/src/openai/common.rs`, `smith-protocol-program/src/backend/memory.rs`) and need no edit.
- `skein/tests/json/tests/suite/memory.rs`'s ceilings (`COLLECTOR_PEAK_CEILING` 440, measured 349; tagged 1100,
  measured 828) should hold one `Down`. If one does not, that is a budget: stop and ask, with the measurement.
- Rejected, do not add: `Queue::pop_back`, against lib.md's first in, first out.

## Steps

1. In `src/tests/collector.rs`, modelled on `skip_count_failure_stops_before_a_large_values_end`, with
   `Limits { skip: 3, .. }` and `{"omit":"abcdefgh`, write `a_collector_leaves_the_callers_queue_alone`:
   call `collector::up` directly, since `Machine::up` asserts `below.len() <= 1`; queue a foreign `Down`
   before the delivery that crosses the budget; assert `Failed(SkippedTooLong)` and that `below` holds exactly
   the foreign item. Run on the old code and quote the panic.
2. The fix.
3. `heavy cargo nextest run -p skein-json --lib`, then the focused tests of `skein-json`, `skein-json-world`,
   `skein-llm`. `heavy cargo check -p smith-protocol-program --tests`. `cargo fmt`, `python3 -I -B
   skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-json --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code and the memory test's new peaks.

## Answers

2026-11-04. Question: the uncapped `demands` queue reaches 445 B against `COLLECTOR_PEAK_CEILING` 440; take a
capacity-1 private queue, or have the ceiling reviewed?

Take the capacity-1 queue: `demands: Queue::with_capacity(1)`, in place of "the pattern of its `events`" in the
acceptance. Everything else in the spec stands, and both ceilings stay as they are.

- This departs from programming-model.md, 6.1 (queues between components are uncapped), knowingly: write the reason
  in the commit and in a line at the field. The queue stages one demand inside one call of `up`, between
  `json::up` and the forward or the `clear`, so it is empty at every entry and `json::up` states at most one
  request (json.md, 5.1). The tokenizer's one `below.push` is under `demand`, called once per entry point, so
  the push cannot fail, which is what 6.1's rule protects. A growing queue's four-entry floor (`growth.rs`) is
  the 96 B the run measured; lib's floor stays.
- The throwaway queue of the failure path stays `Queue::uncapped()`: no exception is needed there, and no
  ceiling scenario reaches it.
- `collector_bound`'s `Queue::<Down>::price(1)` now prices what the queue holds; keep it.
- Expected peaks: 349 + 24 = 373 B (ceiling 440), 828 + 24 = 852 B (ceiling 1100), from the run's +96 B at four
  entries. Reasoned, not measured: the answer's machine builds nothing. Measure them; if either is still past its
  ceiling, stop and ask, with the measurement. Report the peaks of every untagged scenario, which the first run
  left unobserved.
- Build on `ca7882a62d` of this branch; its test stands. Finish step 3's checks, which that run did not reach:
  the focused tests of `skein-json`, `skein-json-world` and `skein-llm`, the `smith-protocol-program` check and
  clippy.
