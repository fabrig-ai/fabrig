+++
id = "printer-orders"
part = "orders"
kind = "feature"
needs = []
files = ["hearth/crates/hearth-client-protocol-records/src/tests/printer.rs", "hearth/tests/builder/src/io.rs", "hearth/tests/builder/src/io/stream.rs", "hearth/tests/builder/tests/suite/orders.rs"]
effort = "high"
minutes = 30
+++

# Tests of the orders beside the printer

## Goal

`Client`'s two capped queues, in `hearth-protocol-builder/src/client.rs`, go in a later task. An extra output of the printer panics on them today; uncapped, it would wait and go out. This task writes the tests that hold what they kept, all passing on today's code. No production code changes.

Acceptance:

- The three tests of `## Steps` stand and pass, each with the name given: D001's table cites them.
- Each bites, by the rule named beside it, and the report gives the failure seen.
- Every test of `hearth-client-protocol-records` passes, and its count rose by two. `hearth-builder-world`'s focused suite passes, and its count rose by one: check the counts.
- `orders::what_a_call_prints_and_stores_does_not_depend_on_its_queues` passes, unchanged.
- Nothing outside `files` changes.

## Inputs

- `hearth/docs/plan/hearth-memory-5/notes/common.md`
- `hearth/docs/design/domain/render.md`, 8.5: "One write at a time", "A record is answered once" and "One record at a time"
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "The machines and the domains": the rows of `Client`'s `requests` and `events`
- `skein/docs/foundation/programming-model.md`, 7.2
- `hearth/tests/mmdebstrap/tests/suite/orders.rs`: the same tests for round 3's machines

## Owns and may change

- **Owns:** the files under `files`.
- **`Machine` in `src/tests/printer.rs` stays as it is,** its queues and its assertions included: the task that uncaps the machine changes them. The new tests don't go through it.
- **Option, pre-approved:** if a scenario of the protocol world leaves a client's output with room unspent on the good path, so that the new check in `Below::stream` fails there, clear the entry where that scenario ends the stream, and say so in the report with the scenario. If the check at `Down::Finish` is what fails, drop that one check, and say so in the same way. The rest stands.
- **Option, pre-approved:** if 2 seeds of the world test cost more than 0.05 CPU s, keep one seed.

## Steps

1. **`hearth/crates/hearth-client-protocol-records/src/tests/printer.rs`.** Beside `Machine`, a driver that owns `Queue::uncapped()` queues, on `List` as `Machine::print` is:

   ```rust
   /// What one entry point left in a printer's queues, before the driver took them.
   #[derive(Debug)]
   struct Step {
       above: Box<[Event]>,
       below: Box<[Down]>,
   }

   /// What the driver tells the printer around one record.
   #[derive(Clone, Copy, Debug)]
   enum Told {
       /// `Up::Room` for each demand.
       Room,
       /// `Up::Failed(Fault::Other)` before the record.
       FaultBefore,
       /// `Up::Failed(Fault::Other)` in place of the `Room` after this many pieces.
       FaultAfter(u32),
   }

   /// One record, from `Print` until its answer: the first step is `Print`'s, each next one a `Room`'s or a fault's.
   fn steps(form: Form, limits: Limits, record: Record, told: Told) -> Box<[Step]>
   ```

   It stops at the first step that holds an event. It panics if a step holds no event and no demand, and past the record's length and two steps.

   ```rust
   const CHUNKS: [u32; 6] = [1, 2, 3, 16, 64, 65_536];
   ```

   - `a_printer_has_one_demand_out_whatever_the_chunk`: at each of `CHUNKS`, for `Record::Spec(fixtures::x1_prod())` in `Form::Text` and `terminal()` in `Form::JsonLines`, with `Told::Room`. Across the steps, in order: a `Down::Demand` has `Read::Nothing` and a room from 1 to the chunk; no demand comes while another is out or before the room the last one was answered with is spent; a `Down::Send` spends that room and stays within it; each `Room` gives a piece; no `Down::Finish`. The pieces add up to `form::print(form, &record, None, &limits)`. **Break:** in `printer.rs`'s `room`, push the next `Down::Demand` twice.
   - `a_record_is_answered_once_whatever_the_chunk`: across the steps' `above`, one event, the one expected, in the last step; and the last step's `below` holds no demand. At each of `CHUNKS` for `Record::Spec(fixtures::x1_prod())` in `Form::Text`: `Told::Room` gives `Event::Printed`, and `Told::FaultBefore` and `Told::FaultAfter(0)` give `Event::Failed(Fault::Other)`, as `Told::FaultAfter(1)` does at each chunk where the record takes more than one piece. `Limits { record: 1, chunk }` gives `Event::Refused(Refusal::TooLong)`, with no bytes. `terminal()` in `Form::Text` gives `Event::Printed`, with no bytes. **Break:** in both of `room`'s `Printed` returns, push `Event::Printed` twice.

2. **`hearth/tests/builder/src/io.rs` and `io/stream.rs`.** `Below` gains `granted: BTreeMap<Token, u32>`, the room answered on each client output and not yet spent. In `Below::stream`'s branch for `PROTOCOL_OUTPUT` and `PROTOCOL_ERROR`:
   - `Down::Demand` with room above 0: `assert!(self.granted.insert(token, room).is_none(), "one demand out on a client's output")`, then the `Up::Room` as now. With room 0, it removes the entry.
   - `Down::Send(bytes)`: it removes the entry, `.expect("a piece answers a demand")`, and asserts the bytes fit, `"a piece within its room"`, then writes them as now.
   - `Down::Finish`: `assert!(!self.granted.contains_key(&token), "a stream finishes after its last piece")`.

3. **`hearth/tests/builder/tests/suite/orders.rs`,** with `protocol` added to its imports:

   ```rust
   #[test]
   fn a_client_s_output_has_one_demand_out_whatever_the_chunk() {
       let mut limits = hearth::limits();
       limits.protocol.printer.chunk = 16;
       for seed in 0..2 {
           let run = protocol::with_limits(&fixtures::thin(), seed, &limits);
           assert_eq!(run.stdout, fixtures::THIN_TEXT.as_bytes(), "seed {seed}: the records whole, at 16 bytes a piece");
           assert_eq!(run.code, Some(0), "seed {seed}: done");
       }
   }
   ```

   Its rule is `Below::stream`'s check, which holds in every protocol world run. **Break:** in `room`, push the next `Down::Demand` before the `Down::Send`. That stays within `requests`' two, so the test fails on `one demand out on a client's output` today, not on a full queue.

4. `cargo fmt`, clippy on `hearth-client-protocol-records` and `hearth-builder-world`, `heavy cargo nextest run -p hearth-client-protocol-records`, `heavy cargo nextest run -p hearth-builder-world`.

5. `notes/common.md`, "The end of every task". Commit and push.

## Report

The first line; each test with what bit it and the failure seen; each package's count before and after; the new tests' CPU seconds; any option taken.
