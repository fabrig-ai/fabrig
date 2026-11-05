+++
id = "config-orders"
part = "orders"
kind = "feature"
needs = []
files = ["hearth/tests/config/tests/suite/orders.rs", "hearth/tests/config/tests/suite/main.rs", "hearth/tests/config/src/world.rs"]
effort = "high"
minutes = 25
+++

# Tests of the orders beside the JSON decoder

## Goal

The capped queues beside `hearth_protocol_config::json::Decoder` go in a later task. A machine's extra output panicked on them, and `Decode::resume` in `hearth-protocol-builder` takes one
output from each per call, so on an uncapped queue a second would wait unseen. This task writes, in the config world, the tests that hold what they kept, all passing on today's code. No
production code changes.

Acceptance:

- The three tests of `## Steps`, 2 stand in `hearth/tests/config/tests/suite/orders.rs` and pass, each with the name given: D001's table cites them.
- Each bites, by the rule named beside it, and the report gives the failure seen.
- Every test of `hearth-config-world` passes: check the count, and that it rose by three.
- Nothing outside `hearth/tests/config/` changes.

## Inputs

- `hearth/docs/plan/hearth-memory-4/notes/common.md`
- `hearth/docs/design/domain/render.md`, 8.5
- `hearth/docs/design/domain/testing.md`, 2.4
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "The machines and the domains"
- `skein/docs/foundation/programming-model.md`, 7.2
- `skein/testing/skein-world/src/stream.rs`: `Wire`, whose `take` panics with `one demand at a time` on a second demand out
- `hearth/tests/mmdebstrap/tests/suite/orders.rs` and `Step` in `hearth/tests/mmdebstrap/src/world.rs`: the same tests for round 3's machines

## Owns and may change

- **Owns:** the files under `files`. `main.rs` takes one `mod orders;`. `world.rs` takes `Step` and `steps`, and `run` and `check` stay as they are, their queues and their assertions included: the
  task that uncaps the machine changes them.
- **Don't use `generate::mutate`:** another draft (`debt-21`) moves it out of this world. `generate::document` stays.
- **Option, pre-approved:** if a doubled demand fails on `Wire`'s `one demand at a time` before the test's own assertion, that is the rule's reason: keep it.
- **Option, pre-approved:** if, on today's code, a step that holds an outcome holds something below, drop that one assertion from the third test, and say so in the report, with the document and
  the seed. The rest of the test stands.

## Steps

1. **`hearth/tests/config/src/world.rs`,** on `Queue::uncapped()`, with `run`'s use of `Wire`:

   ```rust
   /// What one entry point left in a decoder's queues, before the driver took them.
   #[derive(Debug)]
   pub struct Step {
       pub above: Vec<Event>,
       pub below: Vec<Down>,
   }

   /// A decoder over `document` at seeded cuts, from `Decode` to its outcome, then `Close`. The first step is `Decode`'s; the last four are
   /// `Close`'s and three strays past it: `Up::End`, `Up::Failed(StreamFault::Reset)` and `Up::Bytes` of one byte.
   #[must_use]
   pub fn steps(document: &[u8], limits: &Limits, seed: u64) -> Vec<Step>
   ```

   Each `below` of a step before the strays goes to `Wire::take`, in order. The strays reach the decoder directly, and no `Wire`. The driver panics, as `run` does, if no outcome comes.

2. **`hearth/tests/config/tests/suite/orders.rs`,** with `//! Orders beside the decoder (hearth/docs/design/domain/render.md, 8.5).` One helper runs the checks, at
   `Limits { document: 4096, depth: 16, keys: 32, settings: 16 }`: 16 seeds of `generate::document`, each decoded, and 4 seeds each of these, each failed:

   ```rust
   // A duplicate key, a fraction, trailing bytes, a cut and something other than an object.
   const REFUSED: &[&[u8]] = &[br#"{"a":1,"a":2}"#, br#"{"a":1.2}"#, br#"{"a":1} x"#, br#"{"a":[1,"#, b"[]"];
   ```

   - `a_decoder_has_one_read_out_whatever_the_cuts`: no step's `below` holds more than one. **Break:** in `json.rs`'s `next`, after `tk::down`, push a second
     `Down::Demand { read: Read::Fill(1), room: 0 }` below.
   - `a_delivery_gives_a_decoder_s_caller_at_most_one_event`: no step's `above` holds more than one. **Break:** in `drive`'s `Done` arm, push the `Decoded` twice.
   - `a_decoder_s_outcome_comes_once_and_closed_last`: across the steps' `above`, the items are one outcome and then one `Closed`: `Decoded` for a generated document, `Failed` for each refused one; the
     step that holds the outcome holds nothing below; and the three strays leave nothing above or below. **Break:** at the top of `json::up`, when `decoder.state == Waiting::Nothing`, push
     `Event::Closed` and return.

3. `cargo fmt`, clippy on `hearth-config-world`, `heavy cargo nextest run -p hearth-config-world`.

4. `notes/common.md`, "The end of every task". Commit and push.

## Report

The first line; each test with what bit it and the failure seen; the count of `hearth-config-world` before and after; the tests' CPU seconds.
