+++
id = "ancestry-lacks"
part = "ancestry"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/tests/engine/forge/tests/suite/simulation.rs", "temper/tests/engine/forge/tests/suite/changes.rs", "temper/tests/engine/forge/src/changes.rs", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 90
+++

## Goal

A change first in its queue whose head is far behind its base takes its
base, or in conflict goes to its merge job and then a resolution, however
many commits behind it is. Any commit an ancestry range lists, or a range
the forge refuses as too large to list, now proves the head lacks the
base's tip, where today a nonempty page with `more: true` and a `TooLarge`
read as unknown, and the change waited at its turn until its stall held it
(`temper/docs/design/domain/forge.md`, 5.3).

Acceptance:

- The forge world holds the rule in one new table test (Steps, 1), its rows
  written and seen failing first: a head behind by one page, by more than a
  page, and by a range refused as too large, each clean and in conflict.
- `ancestry_status` in `temper-engine-domain-forge/src/domain.rs` reads a
  `Read::Commits` result as `notes/contracts.md`, A, *Containment*, has it,
  for both its readers (a change's `Compare` stage and a catch-up's
  `CatchUp` stage). Nothing else of either stage changes.
- The two existing ancestry tables that held the old reading keep only the
  rows that are still unknown, and the moved rows join the tables of the
  outcome they now have (Steps, 3).
- The **Not yet** entry 5.3, 8.4 (*Stalls*), 8.7 at the end of
  `temper/docs/design/domain/forge.status.md` ("a nonempty ancestry page
  that says more follow, …") is removed, its lines alone
  (`notes/common.md`, *Your status entry*).

## Inputs

- `temper/docs/design/domain/forge.md`, 5.3, the bullet *Containment needs
  only ancestry*; 8.4, the bullet *Stalls*, its last sentence; 8.7, rows 2
  and 6 of the requirements' tables; 8.2, step 1, a catch-up.
- `temper/docs/plan/landing-behind-main/notes/contracts.md`, A.
- `temper/docs/plan/landing-behind-main/notes/common.md`.
- `temper/docs/plan/landing-behind-main/inputs/request.md`, A.
- The forge world's tests as they stand:
  `simulation.rs`'s `goal_landing_updates_a_moved_base_with_a_fresh_ancestry_range`,
  `goal_landing_refuses_to_merge_without_complete_ancestry`,
  `a_conflicting_update_is_resolved_from_a_merge_in_progress`; `changes.rs`'s
  `a_catch_up_starts_when_the_full_comparison_is_too_large` and
  `a_catch_up_retries_uncertain_ancestry_without_a_restart`.
- The change domain's merge-job tests,
  `temper-engine-domain-forge-change/src/tests/merge.rs`
  (`a_conflict_with_a_merge_job_goes_to_the_job_before_a_resolution`,
  `a_merge_job_that_left_conflicts_makes_a_resolution_of_the_same_base`):
  what a conflict does after the update, which this task doesn't change.

## Owns and may change

- Owns the files in `files`: `ancestry_status` alone in `domain.rs`.
- `temper/tests/engine/forge/src/changes.rs`: add one pub helper that steps
  a change with `merge_job: true`, as `change_step` does with `false`
  (`change_step_merge_job(world, entry, heard, gates) -> change::Decision`,
  or a `merge_job` parameter on the innermost helper with the existing
  helpers passing `false`: your choice). Existing helpers keep their
  signatures.
- Pre-approved: if a test outside `files` that reads ancestry (the engine
  domain world's `forge_routes`, the connector's `src/tests/change.rs`)
  asserted a wait on a nonempty page with `more: true` or on `TooLarge`,
  change that assertion to the new reading (the change takes its base) and
  name it in the report; its other steps stay.
- Pre-approved: if the world can't say a merge job's end with the head
  unmoved through the helper above in a few lines, end the conflict rows
  at `Delegate::Merge`, and name in the report the change domain's test
  that holds the resolution after it.
- Not yours: `ancestry_status`'s callers, the change domain
  (`temper-engine-domain-forge-change`), skein's Forgejo plan (the task
  `forgejo-commits-oldest` owns it), any figure, `forge.md`.

## Steps

1. **The table, failing first,** in `simulation.rs`, beside
   `goal_landing_updates_a_moved_base_with_a_fresh_ancestry_range`, whose
   setup it takes (`World::with_goal_room(seed, 16, 4096)`, a short goal
   history by `prepare_goal_history`, `queue_goal_landing` at the goal's
   head, so the change is `First`, CI passed, its rules asking up to date):

   ```rust
   #[test]
   fn a_first_change_behind_its_base_takes_it_or_goes_to_its_merge_job() {
       // rows: (name, commits main moves by, scripted ancestry, conflict)
   }
   ```

   | row | main moves by | the step's ancestry read | then |
   |---|---|---|---|
   | `one_page` | 1 commit | the fake's: one commit, `more: false` | `Effect::Update { base, head }` |
   | `past_a_page` | 17 commits, the world's 16 rows and one | the fake's: 16 commits, `more: true` | `Effect::Update { base, head }` |
   | `too_large` | 1 commit | `world.script_ancestry(Some(Err(Error::TooLarge)))` for this step, cleared after | `Effect::Update { base, head }` |
   | each of the three `_in_conflict` | as its row, the last moved commit writing the head's file with other content | as its row | `Effect::Update`, then, stepped with the merge job named, `Delegate::Merge { base, from: Some(head) }`, then the job ending with the head unmoved, `Delegate::Resolve { base }` |

   Every row also asserts the step read
   `Read::Commits { before: head, after: base, page: 1 }`
   (`world.take_calls()`), and `past_a_page` that the fake's range there
   holds more than a page. Run it: `one_page` and its conflict row pass;
   `past_a_page` and `too_large`, both ways, wait (`Decision::Wait`). Note
   each failure for the report.

2. **The fix:** `ancestry_status` as `notes/contracts.md`, A, has it, its
   comment citing `forge.md`, 5.3. Run the table: every row passes.

3. **The readers that held the old reading** (a change of meaning):
   - `goal_landing_refuses_to_merge_without_complete_ancestry`: drop its
     `Err(Error::TooLarge)` row, now the new table's `too_large`. It keeps
     `Refused`, `InvalidAnswer` and the empty page with `more: true`.
   - `a_catch_up_retries_uncertain_ancestry_without_a_restart`: drop its
     `Err(Error::TooLarge)` row and its one-commit page with `more: true`.
     It keeps the other three.
   - `a_catch_up_starts_when_the_full_comparison_is_too_large`: make it a
     loop over the ancestry it meets: the fake's (as now), a scripted
     `Err(Error::TooLarge)`, and a scripted one-commit page with
     `more: true`; each starts the merge job at once
     (`Delegate::Merge { base: commit, from: None }`), spends no
     resolution and reads the catch-up range, as now.
   Break the rule once by hand (give `TooLarge` back to `Unknown`; then a
   nonempty page with `more: true`), see `too_large` and then
   `past_a_page` fail for that reason, restore, and name both in the
   report.

4. **Narrow runs, then the readers.** This changes what an ancestry read
   means, so it runs the tests of the packages that read it:
   - `heavy cargo nextest run -p temper-engine-forge-world --test suite -E 'test(/^simulation::/) | test(/^changes::/)'`
   - `heavy cargo nextest run -p temper-engine-forge-world --test suite`
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib`
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/forge_routes::/)'`
   Check each run's count; one that selected nothing proves nothing.

5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-domain-forge -p temper-engine-forge-world
   --all-targets -- -D warnings`; remove the status entry; commit saying
   why (a head far behind its base takes it rather than waiting for its
   stall); push.

## Report

As the implementer's text has it, and: each row's failure before the fix;
the rule broken for each bite and the failure seen; each reader test you
edited and how; whether the conflict rows reach `Delegate::Resolve` in the
world or stop at the merge job, and why; the counts of the runs in step 4.
