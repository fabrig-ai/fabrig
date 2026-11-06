+++
id = "focused-cuts"
part = "focused-cuts"
kind = "feature"
needs = []
files = [
  "jig/tests/core/tests/suite/deletion.rs",
  "jig/tests/core/tests/suite/faults.rs",
  "jig/tests/core/tests/suite/decision_reads.rs",
  "jig/tests/core/tests/suite/main.rs",
  "jig/tests/core/tests/suite/fuzzy/deletion.rs",
  "jig/tests/core/tests/suite/fuzzy/decision_reads.rs",
  "jig/tests/core/tests/suite/fuzzy/faults.rs",
]
effort = "medium"
minutes = 60
+++

# The core world's sweeps move to fuzzy, and focused keeps cuts chosen for why

## Goal

`jig/docs/design/domain/testing.md`, 5 and 9 draw the line: focused crashes after a few chosen commits, fuzzy sweeps every commit. Five focused tests of `jig/tests/core` sweep. Each sweep moves under `fuzzy::` whole, keeping its name, and focused keeps a few cuts named for why they're chosen (testing.md, 10: "the commits around an effect, a claim and an answer").

**`plan.md`'s table is the map:** `jig/docs/plan/jig-world-cost/plan.md`, *Sweeps*. It says why no existing fuzzy test holds these: read at the cut, conformance's sweeps cut durably only, and none compares a run with its uncut self.

Acceptance:

1. Each row of the table holds: the focused test by its new name, with the cuts named; the fuzzy test by its old name, making every check it makes today, over every commit it sweeps today. Each focused test's old name is gone from focused.
2. Both suites pass for `jig-core-world`, and `--profile fuzzy` selects the five moved tests.
3. Sweep and chosen cuts run **one body**, so a focused cut makes the same checks as the sweep's.
4. Each chosen commit is found by what it holds, not by a bare number, and the test asserts it holds that: after `Cut::Durable(n)` the named row is in `world.store.rows`, and after `Cut::Submitted(n)` it isn't. Where a later row changes what the check can see, assert the nearest thing that's true and say so.
5. `jig/tests/store`'s cuts are untouched: testing.md, 9 places them in focused.

## Inputs

- `jig/docs/design/domain/testing.md`, 5 (*Crashing at every commit*), 9 and 10.
- `jig/docs/design/domain/people.md`, 5.2 and 8, for the deletion's steps.
- `jig/docs/plan/jig-world-cost/plan.md`, *Sweeps*.
- `jig/docs/plan/jig-world-cost/notes/common.md`.

## Owns and may change

- **Owns:** the spec's `files`. **Departure:** none. This task changes no design: 5, 9 and 10 stay true.
- **Readers, read at the cut:** the private helpers of `deletion.rs`, `faults.rs` and `decision_reads.rs` are read only there. They become `pub(crate)` where the fuzzy files need them, reached as `crate::deletion::..`. `suite/deletion.rs`'s `finish` is `pub(super)` and read elsewhere, so it stays. `jig/tests/core/tests/suite/main.rs` gains `mod decision_reads;` and `mod deletion;` inside `mod fuzzy`, and nothing else.
- **Options:**
  - `World::commits` lists the root's commits only: a fixture's commit (`jig-core-kit/src/world/store.rs`) isn't in it, so an index isn't always a number. Count back from the end: the commit `i` from the end of `commits` is number `store.applied - i`, so long as no fixture's commit follows it. Assert the row, as acceptance 4 says, and a miscount shows.
  - A chosen commit the code doesn't have as described (a row in an earlier commit than this spec expects, say): choose the nearest commit that shows the same thing, name it for what it holds, and put it in the report.
  - `decision_reads`'s `cut` number becomes a small local enum named for the moment. Its arms are yours.
  - The moved tests keep their names, since those still say what they check.

## Steps

1. **`deletion.rs`:**

   ```rust
   /// The uncut run's first and last commit of the deletion, and its final rows.
   pub(crate) fn uncut_deletion() -> (u64, u64, BTreeMap<root::Key, root::Record>);
   /// Cut one commit of the deletion, restart, finish, and end as the uncut run did.
   pub(crate) fn a_cut_deletion_ends_the_same(first: u64, cut: Cut, expected: &BTreeMap<root::Key, root::Record>);
   ```

   The focused test is `a_deletion_cut_around_its_decision_and_last_step_ends_the_same`, with its four cuts as the table names them. The middle commit is the one halfway between `first` and `last`: it's chosen for being between two family steps, so it has no one row to name, and acceptance 4 asks only `reached_cut` of it. `fuzzy/deletion.rs` holds `a_deletion_cut_after_any_commit_ends_the_same`, with today's loop.
2. **`faults.rs`:** a helper naming a commit by what it holds, and the shared bodies.

   ```rust
   /// The first commit of `world` saving a row `chosen` picks.
   fn commit_saving(world: &World, chosen: impl Fn(&root::Record) -> bool) -> u64;
   pub(crate) fn effect_scenario(world: &mut World);
   pub(crate) fn an_effect_cut_recovers(cut: Cut);
   pub(crate) fn a_failed_commit_stops(number: u64);
   pub(crate) fn a_startup_cut_restarts(engine: bool, cut: Cut);
   ```

   The focused tests are `an_effect_and_its_terminal_survive_both_crash_cuts`, `a_failed_effect_or_terminal_commit_stops_before_any_later_output` and `the_sign_in_chat_and_claim_commits_restart_on_either_side_of_durability`, with cuts as the table names. Keep the `engine` world's `broken()` handling where it is today. The three sweeps go to `fuzzy/faults.rs` with their names and loops.
3. **`decision_reads.rs`:** the loop's body becomes `pub(crate) fn a_decision_across_a_restart(proposal: bool, restart: Restart)`. The focused test is `decisions_keep_their_offered_fence_and_room_across_a_restart_at_the_decision_and_the_turn`. `fuzzy/decision_reads.rs` holds `decisions_keep_their_offered_fence_and_room_across_each_restart_cut`, all eight runs.
4. No break-it-once for what only moved. For `commit_saving`'s choice, point one test at the wrong row once, see acceptance 4's assertion fail, restore it, and report that.
5. Narrow runs by name in each profile (`jig/docs/plan/jig-world-cost/notes/common.md`, *Running*), then the implementer's closing steps for `jig-core-world`. Commit and push.

## Report

The implementer's report, plus: each focused test's cuts as they landed (the commit, what it holds, and why it's chosen), any row of the table that differs and why, so `plan.md` can be corrected, and the check you broke and what it showed.
