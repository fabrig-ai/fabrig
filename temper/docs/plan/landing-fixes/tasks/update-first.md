+++
id = "update-first"
part = "update-first"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge-change/src/domain.rs",
  "temper/crates/temper-engine-domain-forge-change/src/tests.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/update_first.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 75
+++

# A head already behind takes its base before its CI

## Goal

The request's item 1. A change whose head already lacks its base's tip, under rules that ask up to date, sits in `Checking` waiting for CI (5:26 in the run reported) on a head its turn can only replace. Now it takes the update first, and a CI conclusion already read stays as it was.

Acceptance, in `checking` (`temper-engine-domain-forge-change/src/domain.rs`), at a head equal to the branch:

- `contains_base` `Failed`, CI read at this head `Unknown` or `Pending`, `clean` empty, and `updates + 1 < limits.updates`: `Effect::Update { head, base: base_tip }`, state `Updating`, `updates` up by one. This comes before the eager-gate branch: no gate runs on a head about to be replaced.
- `Passed` at this head, behind: `Gating`, as today, no effect. The update waits for its turn.
- `Failed` at this head, behind: as today, `repair_due` takes the base first.
- `contains_base` `Unknown` or `Pending`: as today, no update on a guess.
- `facts.ci.head != head`: as today, it waits. CI not read at this head is no conclusion.
- `clean` not empty (this head came from an update): it waits for its CI, though the base moved again.
- `updates + 1 >= limits.updates`: it waits for its CI, no hold: the last update stays for its turn.
- Under rules asking no update the root gives `contains_base` `Passed` (`temper-engine-domain-forge/src/domain.rs`, `contains_base: if requires.up_to_date`), so nothing changes there. A story proves it.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.2, step 3 and the paragraph after step 7; section 10, **Lazy** and **By its landing branch's rule**; 5.3 for the fresh reads.
- `temper/docs/plan/landing-fixes/notes/common.md`.

## Owns and may change

Owns the files in the header. In `forge_routes.rs`, the one `mod update_first;` line.

Reader and neighbour files:

- `temper/tests/engine/domain/tests/suite/forge_routes.rs`'s stories that move the base under a pending CI, `a_conflicting_update_is_resolved_from_a_merge_in_progress` and `an_approval_carries_over_a_clean_update_and_is_asked_again_after_a_repair` among them, and `forge_routes/{ci_poll,repair_green,catch_up,goal_landing,held_change,repairs}.rs`: if one now sees its update earlier, make it conclude CI before moving the base, keeping what it asserts. An assertion that can't be kept is a report under `## Left`, not a new rule.
- `temper/crates/temper/tests/suite/forge_land.rs` and `temper/crates/temper/tests/support/forge_land/**` are read-only unless the same edit applies. `model-catalogue-1/stock-seven` also changes them.

Departs from the implementer's text: delete from `temper/docs/design/domain/forge.status.md` the `Not yet` entry beginning `8.2, step 3, and 10:`.

Options, each pre-approved:

- If a change on its requester's held branch (`facts.requester_branch`, 8.1) misbehaves under the rule in `goal_landing` or `held_change`, leave those changes out of it with that one condition, and say so in the report so the design can follow.
- If the fake forge can't hold a base move before the first CI read, move the base between the producer's push and its `observe_push`.

## Steps

1. **Reproduce.** In `temper-engine-domain-forge-change/src/tests.rs`, beside `a_clean_update_preserves_the_turn_and_carries_a_review`, add `a_head_already_behind_takes_its_base_before_its_ci`, then one test per acceptance line, using `change(State::Checking { head: HEAD })` and `facts()`. The first fails today with no effect.
2. **Fix.**

   ```rust
   Status::Unknown | Status::Pending => {
       if behind_unchecked(&change, facts, limits) {
           let ready_since = change.ready_since.unwrap_or(now);
           return update_due(change, head, facts.base_tip, ready_since, now, limits);
       }
       // the eager-gate and settling branches, as they are
   }

   /// A head already behind takes its base before CI concludes on it, once for
   /// a head a push gave it, and never with the change's last update
   /// (temper/docs/design/domain/forge.md, 8.2, step 3).
   fn behind_unchecked(change: &Change, facts: &Facts, limits: &Limits) -> bool
   ```

   `Checking -> Updating -> Checking` is the path `repair_due` already takes, so `updating`, `conflict_due` and the panel need nothing.
3. **Stories,** `forge_routes/update_first.rs`, using `early_start.rs`'s `goal_world`, `make`, `push`, `observe_push` and `ci_poll.rs`'s way of setting a status (a local helper; `up_to_date` rules as `early_start.rs` writes them):
   - `a_change_opened_behind_its_base_is_updated_before_its_ci_concludes`: the base moves after the producer's push; with no CI concluded at the first head the row goes to `Updating`, then `Checking` at a new head that contains the base; `updates == 1`; green at the new head, it lands at that head with no second update.
   - `a_green_head_behind_its_base_keeps_its_result_until_its_turn`: green at the head before the base is seen to move: `updates == 0` through gating, and the update comes when it is first.
   - `a_head_behind_under_rules_asking_no_update_is_not_updated`: `light_rules.rs`'s rules: `updates == 0`, and it lands at its head.
4. Break the rule once (drop the `behind_unchecked` branch) and see the first story fail on `updates`; then drop `clean.is_empty()` and see its unit test fail. Restore.
5. fmt, `old_forms.py`, clippy of `temper-engine-domain-forge-change` and `temper-engine-domain-world`, then the focused tests of `temper-engine-domain-forge-change`, `temper-engine-domain-forge`, `temper-engine-domain` and `temper-engine-domain-world`. This changes when a change updates, so also run `temper`'s `forge_land` tests. Commit and push.

## Report

The implementer's report, plus: the CI a story saw on the replaced head, whether any story needed its order changed and which, and whether the held-branch option was taken.
