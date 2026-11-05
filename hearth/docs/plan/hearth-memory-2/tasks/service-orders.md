+++
id = "service-orders"
part = "orders"
kind = "feature"
needs = []
files = ["hearth/tests/builder/tests/suite/orders.rs", "hearth/tests/builder/tests/suite/main.rs", "hearth/tests/builder/src/simulated.rs"]
effort = "high"
minutes = 60
+++

# Tests of the orders beside the service's room checks

## Goal

A reading of `iterate` found no room check that alone keeps an order: `hearth/docs/design/domain/render.md`, 8.5 states what keeps each. That was a reading,
and nothing ran. This task runs it. One test shows that what a call prints and stores doesn't depend on how much room the service's queues have, and
another holds the one order that test can't see. Then each order is broken by hand, to learn which test holds it. No production code changes.

Acceptance:

- The two tests of `## Steps`, 2 and 3, stand in `hearth/tests/builder/tests/suite/orders.rs` and pass.
- Each row of `## Steps`, 4 was broken once by hand and restored, and the commit message gives, row by row, the test that failed and the failure seen.
- `hearth-builder-world`'s focused suite passes: check the count, and that it rose by two.

## Inputs

- `hearth/docs/plan/hearth-memory-2/notes/common.md`
- `hearth/docs/design/domain/render.md`, 8.5
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "What each room check keeps", "The service"
- `hearth/docs/design/domain/testing.md`, 2.6
- `skein/docs/foundation/programming-model.md`, 7.2

## Owns and may change

- **Owns:** the files under `files`: `main.rs` takes one `mod orders;`, `simulated.rs` takes one function.
- **Departure from `notes/common.md`:** these run the whole builder, so the two tests may cost 0.3 CPU s between them, not 0.05 each. Past that, drop the
  32 column first, then `bounds::queued()`, and say so with the measurement.
- **Don't touch** `hearth/tests/builder/tests/suite/domain.rs` or `properties.rs`, which another draft is changing.
- **Option, pre-approved:** if a scenario needs `fixtures::small_limits()` and not `bounds::limits()`, give it those, with `queue` set the same way.
- **Option, pre-approved:** a line of `ERROR` that states a duration differs by virtual time, not by order. Compare what's left with durations masked,
  and name the line in the report.
- **A difference between the columns is a finding,** and the most useful thing this task can return: an order only room keeps. Don't mask it: report
  `Done:` with what you have pushed if the rest holds, naming the scenario, the queue sizes and the first difference, under `## Left`.
- **Row 4:** the survey found no one-line break. If you find none either, say so and leave that row's bite out.

## Steps

1. **`hearth/tests/builder/src/simulated.rs`:**

   ```rust
   /// The call at the named limits, unmetered.
   #[must_use]
   pub fn with_limits(scenario: &Scenario<'_>, seed: u64, limits: hearth::Limits) -> Outcome<Proc, Shared> {
       build(scenario, seed, Config::calm(), &Options { limits, ..Options::default() }).run()
   }
   ```

2. **`what_a_call_prints_and_stores_does_not_depend_on_its_queues`,** in `suite/orders.rs` (`#[path = "../../src/fixtures/bounds.rs"] mod bounds;`, as in
   `suite/memory.rs`). For each of `fixtures::thin()`, `bounds::queued()`, `fixtures::resolving(fixtures::THIN)` and the build scenario of
   `building_holds_no_more_than_their_ceiling`, run `simulated::with_limits` at `bounds::SEED` and `bounds::limits()` with `limits.queue` at 3, 32 and
   4,096. All three runs give the same `contents(fixtures::OUTPUT)`, `contents(fixtures::ERROR)`, `fixtures::client_exits` and `fixtures::store_files`.

3. **`a_build_prints_its_layers_then_its_failure_then_its_reply`,** also in `suite/orders.rs`: the build scenario at `bounds::limits()`. Its output's lines
   are its three `to build` lines, in the plan's order, then the `failed` line and nothing after it, and the client exits 69 only after them.

4. **Bites.** Each is one change, undone after. The first five names are `hearth/docs/design/domain/render.md`, 8.5's; the sixth is the loops'.

   | # | Order | Break | Should fail |
   |---|---|---|---|
   | 1 | Resumed work before the next event | `protocol_up`'s `told` loop stops checking `service.protocol.work_pending()` | step 2 |
   | 2 | One record at a time | `protocol_down` stops checking `service.protocol.takes()` | step 2 |
   | 3 | A build's report comes whole | `hearth-domain-builder`'s `building::holds` returns `false` | step 3 |
   | 4 | An answer follows its resources' close | `hearth-protocol-builder`'s `resume` lets render hear before `child.done()` | step 2 |
   | 5 | A store step's answer before the next step | `hearth-protocol-builder`'s `up` takes `!above.is_empty()` as true for every store event, as a stale event in `calls` would make it | step 2, the `resolving` row |
   | 6 | `file_told` before `told` | swap the two loops in `protocol_up` | possibly nothing |

   Row 6 tests the claim that this isn't an order: run step 2 and `hearth-builder-world`'s `suite` module `protocol`. If nothing fails, say so. If something
   does, it is an order, and the commit names the test.

5. `cargo fmt`, clippy on `hearth-builder-world`, `heavy cargo nextest run -p hearth-builder-world`.
6. `notes/common.md`, "The end of every task". Commit, with the table of bites, and push.

## Report

The first line; the table of bites; the CPU seconds of the two tests; the suite's count before and after; anything under `## Left`.

## Answers

2026-11-05 (fabrig/hearth, the plan's owner, through agents/loop). **Extend, don't accept.** Rows 1 (resumed work before the next event) and 5 (a store step's answer before the next step) are orders `hearth/docs/design/domain/render.md`, 8.5 names, and the round exists to pin them before the room checks go. Keep commit 0c9daff810 and its tests.

- For each of rows 1 and 5, make a scenario or queue setting in which the one-line mutation makes a test fail: a resolving step with an answer in flight when a stale store event arrives, work pending at the moment an event comes, a low queue limit. Changing the spec's load, seed or scenarios is allowed. The order stays one fixed, small scenario each, cheap in CPU, in `orders.rs`.
- If a row still won't bite after that bounded try, about 45 minutes, don't widen further: leave the row out of the tests, say what you tried, and record it in D001 as an order held only by the design, with what the service round must recheck before removing that check. That's a recorded outcome, not a pass.
- Row 6 stays as it is: no order was found, and that's the finding.
- No production change. Files may extend to any fixture this needs under `hearth/tests/builder/`.
