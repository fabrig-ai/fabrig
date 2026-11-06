+++
id = "update-worlds"
part = "worlds"
kind = "feature"
needs = ["update-class", "update-let-go"]
files = ["temper/tests/engine/forge/tests/suite/update_lost.rs", "temper/tests/engine/forge/tests/suite/main.rs", "temper/tests/engine/forge/src/world.rs", "temper/docs/design/domain/landing.status.md"]
effort = "high"
minutes = 90
+++

# The connector's world shows the class holds

## Goal

`jig/docs/design/domain/connectors.md`, 15 asks the connector's world to show a class holds, with copies late, past their deadline and after a retry, and targets another hand reached. The world holds both styles, under a queue and as a turn's prepare, and says in stories what a late copy costs.

Acceptance, in a new `temper/tests/engine/forge/tests/suite/update_lost.rs`, the first five expected to pass on arrival, as the code they pin is in your `needs`. Each is bitten once by putting kind 11's class back, or by undoing the let-go in `change_outcome`, and restored:

1. `a_lost_update_is_found_under_a_queue`: a copy landing within its lifetime: `writes` one update, the change `Checking` at the new head, nothing held.
2. `a_lost_update_is_made_again_and_lands_under_a_queue`: the first copy past its lifetime, the second answered: two updates, landed, no decision `Hold` in the run. Past the copy's landing, the landed head stands.
3. `a_lost_prepare_is_found_and_the_landing_goes_on` and `a_lost_prepare_is_made_again_and_the_landing_goes_on`: `Write::Update { style: Rebase }`, on the `landings::first_landing` shape: `landing::Phase::Landed`, no one asked.
4. `an_update_lost_for_good_holds_at_its_count_and_a_release_lands_it`: every write lost: `change::Hold::Updates`, `updates * write_attempts` writes, the account's words on the row. Writes answered again and released: landed.
5. `an_update_whose_base_went_on_while_lost_takes_the_new_tip`: `updates` 2, no hold, landed, the head holding both tips.
6. `a_copy_landing_after_the_change_moved_on_holds_and_lands_nothing_unchecked`, over two rows: a forward move, a merge copy's; a rewrite of the branch dropping its last commit, a rebase copy's over a later push, by an outside hand as `notes/common.md` says, the actor temper's. Each: `change::Hold::BranchMoved` once the slot is free, no `Write::Merge` at the head it had, and a release takes the head as it is, checks it and lands there. Expected to pass: this is what `forge.md`, 10 says a copy costs, and it is the plan's check of it.
7. `temper/tests/engine/forge/tests/suite/main.rs` names the module. The bullet naming `update-worlds` is gone from `temper/docs/design/domain/forge.status.md`, and the `5.2, 16 (update-worlds)` sentence from `temper/docs/design/domain/landing.status.md`.

## Inputs

- `temper/docs/plan/update-recovers/notes/common.md`, `temper/docs/plan/update-recovers/notes/contracts.md`
- `jig/docs/design/domain/connectors.md`, 4.3, 15
- `temper/docs/design/domain/forge.md`, 8.5, 10, 18
- `temper/docs/design/domain/landing.md`, 5.2, 16
- `temper/tests/engine/forge/tests/suite/landing.rs`, `temper/tests/engine/forge/src/landings.rs`, `temper/tests/engine/forge/tests/suite/simulation.rs`, `a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice`

## Owns and may change

- Owns the new file. `temper/tests/engine/forge/src/world.rs` gains `land_writes_late(after)`, over the fake's `landing`, `land_min` and `land_max` as `skein/tests/forge/client/src/world.rs` does, without its lifetime assertion, and `lose_writes` or the like if a story needs a write lost with no copy. Add them at the end of the `impl`.
- `temper/tests/engine/forge/src/landings.rs`: `pub` on what you reuse, no other edit.

**Options:**

- **If a story here fails because the machine, the top or the client does otherwise than the design says,** the design's words win if the fix is one line in a task's named function: fix it and say so. Otherwise the story stays, marked by a `Question:` in your report, with what was seen, and the rest pushed.
- **If story 6's rewrite can't be made with `push_as`,** use `external` with the fake's writes, or `forge::advance` on a branch reset by the fake's own means, whichever exists. Don't change `skein-fake-forge-domain`.
- **If a story runs over the suite's budget,** take the brief's questions in order: does it belong here; can it be faster. No limit moves.

## Steps

1. Add the world's helpers and write the stories. Run: `heavy cargo nextest run -p temper-engine-forge-world --test suite -E 'test(/update_lost/)'`. Check the binary's name, and the count.
2. Bite each as *Goal* says. Note the failure seen.
3. Close the status entries. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p temper-engine-forge-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-forge-world`, then its `--profile fuzzy`. Commit, small and saying why; push.

## Report

As your text says, and, for the run after landing, what each late copy cost in the world: writes, heads moved, holds.
