+++
id = "mmdebstrap-orders"
part = "orders"
kind = "feature"
needs = []
files = ["hearth/tests/mmdebstrap/tests/suite/orders.rs", "hearth/tests/mmdebstrap/tests/suite/main.rs", "hearth/tests/mmdebstrap/src/world.rs"]
effort = "high"
minutes = 30
+++

# Tests of the orders beside mmdebstrap's machines

## Goal

The capped queues beside `output::Reader` and `packages::Stanzas` go in a later task. They aren't `.room()` calls, but a machine's extra output panicked
on them, and `hearth-protocol-builder` takes one output from some of them, so an uncapped queue would lose it without a word. This task writes, in
mmdebstrap's world, the tests that hold what they kept, all passing on today's code. No production code changes.

Acceptance:

- The six tests of `## Steps`, 2 stand in `hearth/tests/mmdebstrap/tests/suite/orders.rs` and pass, each with the name given: D001's table cites them.
- Each bites, by the rule named beside it, and the report gives the failure seen.
- `generated_failures_keep_the_reference_s_tail_and_cause` was broken by `## Steps`, 3, and the report says whether it bit.
- Every test of `hearth-mmdebstrap-world`'s `suite` passes: check the count, and that it rose by the tests you added.
- Nothing outside `hearth/tests/mmdebstrap/` changes, but what `## Owns and may change` allows.

## Inputs

- `hearth/docs/plan/hearth-memory-3/notes/common.md`
- `hearth/docs/design/domain/render.md`, 5.3 and 8.5
- `hearth/docs/design/domain/testing.md`, 2.4
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "The machines and the domains"
- `skein/docs/foundation/programming-model.md`, 7.2
- `skein/testing/skein-world/src/stream.rs`: `Wire`, whose `take` panics with `one demand at a time` on a second demand out

## Owns and may change

- **Owns:** the files under `files`. `main.rs` takes one `mod orders;`. `world.rs` takes `Step`, `output_steps` and `index_steps`, and its existing drivers
  stay as they are, `UP_MAX_OUT` included: the task that uncaps the machines changes them.
- **Option, pre-approved:** if a doubled demand fails on `Wire`'s `one demand at a time` before the test's own assertion, that is the rule's reason: keep it.
- **Option, pre-approved:** if `## Steps`, 3 does not bite, no seed there admits more than `tail_lines` lines. Add a seventh test to `suite/orders.rs`,
  `the_tail_keeps_the_newest_lines_oldest_first`: drive `tail::admit` directly at `Limits { tail_lines: 2, ..LIMITS }`, over rows of distinct `E: ` lines, up to
  twice `tail_lines`. It bites by the same break. Then change `hearth/docs/development/debt/D001-room-checks-and-max-out.md`'s `tail.rs` row to name it, and
  nothing else of that file.
- **Option, pre-approved:** if a test's driver reads better sharing a generic body in `world.rs`, share it. Its signatures below stay.

## Steps

1. **`hearth/tests/mmdebstrap/src/world.rs`,** on `Queue::uncapped()`, with `read_output`'s and `read_index`'s handling of `End`:

   ```rust
   /// What one entry point left in a machine's queues, before the driver took them.
   #[derive(Debug)]
   pub struct Step<T> {
       pub above: Vec<T>,
       pub below: Vec<Down>,
   }

   /// A pipe's reader at seeded cuts. The first step is `open`'s; the last three are strays past the
   /// terminal event: `Up::End`, `Up::Failed(Fault::Reset)` and `Up::Bytes` of an `Inst` line.
   #[must_use]
   pub fn output_steps(bytes: &[u8], seed: u64, limits: &Limits, end: End) -> Vec<Step<Seen>>

   /// The same for a list's reader; its stray bytes are a `Package: ` line.
   #[must_use]
   pub fn index_steps(bytes: &[u8], seed: u64, limits: &Limits, end: End) -> Vec<Step<Found>>
   ```

   Each `below` of a step before the strays goes to `Wire::take`, in order. The strays reach the machine directly, and no `Wire`. The driver panics, as
   `read_output` does, if no terminal event comes.

2. **`hearth/tests/mmdebstrap/tests/suite/orders.rs`.** The generated tests run 16 seeds, each with `End::Whole` and with `world::end`, at `fixtures::LIMITS`:
   `generate::output` for a pipe, `generate::index` for a list.

   - `a_pipe_reader_has_one_scan_out_whatever_the_cuts`: no step's `below` holds more than one. **Break:** in `output::up`'s `Up::Bytes` arm, demand twice.
   - `a_delivery_gives_a_pipe_s_caller_at_most_one_thing`: no step's `above` holds more than one. **Break:** in the same arm, push the `seen` twice.
   - `a_pipe_ends_once_and_last`: across the steps' `above`, one `Seen::Ended` or `Seen::Failed`, and it is the last item; the three strays leave
     nothing above or below. **Break:** take out `up`'s `if reader.ended { return; }`.
   - `a_list_reader_has_one_scan_out_whatever_the_cuts`: as the pipe's. **Break:** in `packages::up`'s `Up::Bytes` arm, demand twice.
   - `a_delivery_gives_a_list_s_caller_at_most_one_stanza`: no step's `above` holds more than one `Found::Stanza`. **Break:** in `emit`, push the
     stanza twice.
   - `a_list_s_end_follows_its_last_stanza_and_comes_once`: the generated lists, as the pipe's end, and by hand a list of two stanzas with a blank line
     between them and none after the last, with a line feed at its end, over 8 seeds at `End::Whole`: the items above are `Stanza`, `Stanza`, `Ended`, and the
     step of `Up::End` holds the last two in that order. **Break:** in `packages::up`'s `Up::End` arm, push `Found::Ended` before `finish`.

3. **The tail's row.** D001 names `generated_failures_keep_the_reference_s_tail_and_cause` for `admit`'s order, and nothing has shown it bites. In `admit`,
   drop the new line where the queue is full, in place of popping the oldest, run that test, and restore it. If it fails on the tail, the row stands. If not,
   take the option above.

4. `cargo fmt`, clippy on `hearth-mmdebstrap-world`, `heavy cargo nextest run -p hearth-mmdebstrap-world`.

5. `notes/common.md`, "The end of every task". Commit and push.

## Report

The first line; each test with what bit it and the failure seen; whether the tail's test bit; the count of `suite` before and after; the tests' CPU seconds.
