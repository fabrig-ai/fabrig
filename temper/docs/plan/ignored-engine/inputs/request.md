# ignored-engine: twelve ignored engine stories run again

testing-strategy 8 says an ignored test is a finding its owner runs until it's fixed. Twelve of temper's ignored tests sit in the engine-domain world, the forge protocol's GitHub joint and one client story. Some wait on work that has since landed. Others wait on a small capability of the world or joint they run in. This plan brings each of them back into the suite that selects it.

## What's wanted

**Each test below loses its `#[ignore]` and passes in focused or fuzzy, whichever selects it.** For each test:

- If it passes as it stands, it stays un-ignored.
- If it fails, fix the cause the reason names, by the design it cites: the product where the product is wrong, the world, joint or fake where the test lacks a capability.
- A test whose design has moved is rewritten to the design as it stands now, never deleted.
- If the cause turns out to be a feature that isn't built and is larger than a task, the test stays ignored. Its reason is updated to name the missing design section, and the plan's notes say so.

**Restore**, waiting on `restore-request`'s Restore admission and ForgetReplaced. Both are on main since, by `jig/docs/design/domain/engine.status.md` (17.10) and `temper/docs/design/protocol/configuration.status.md`:

- `temper/tests/engine/domain/tests/suite/settings/kinds.rs`:
  - `a_landing_rule_and_the_forges_charter_changed_then_restored`
  - `a_restore_leaves_out_a_deleted_projects_rule`
  - `forgetting_erases_the_forges_replaced_rows`
- `temper/tests/client/tests/suite/settings_tools.rs`: `each_kind_through_ctl_proposed_shown_accepted_and_restored` (`tools.md`, 11.2).

The status entries that say "whether they pass unignored is unknown" are updated to match.

**The engine-domain world's changes to existing records** (`temper/tests/engine/domain/tests/suite/change_panel.rs`):

- `own_record_change_refreshes_a_watched_panel`
- `priority_moving_ahead_marks_the_waiting_panel`
- `queue_exit_and_reentry_refresh_the_panel_behind`

All three need the world's `Change` step to commit a mutation, a priority change, and a queue exit and re-entry to records that already exist. Today it only creates records. These are one capability of the world.

**Forge routes** (`temper/tests/engine/domain/tests/suite/forge_routes/`):

- `goal_changes.rs`: `a_goal_makes_two_changes_into_its_branch_the_second_after_the_first`. The fake forge squashes Style merges, but a goal's landing needs a merge commit (C5).
- `goal_landing_held.rs`: `a_goals_landing_held_past_its_repairs_lands_after_a_change_into_its_branch`. The restart oracle changes `Deployment.messages` after the released goal landing.
- `landing.rs`: `a_change_created_with_a_dependency_joins_before_production`. It waits on jig building a declared procedure with a pending dependency that starts kept (`connectors.md`, 6.1). Check whether jig does this on main now.

**The GitHub joint** (`temper/tests/engine/protocol-forge/tests/suite/github_land.rs`):

- `restart_after_merge_before_read_never_merges_twice`, which needs a cut after an applied merge and before its confirming read.
- `ruleset_refuses_merge_without_landing`, which needs a ruleset knob.

## Limits

- No test moves to another suite, and no budget, timeout or filter changes.
- No other ignored test is touched.
- `walk = false`. Effort medium. Three or four tasks, by the groups above.

## Sources

- The twelve tests and their `#[ignore]` reasons.
- `jig/docs/design/domain/engine.md`, 17.10, and its status.
- `temper/docs/design/protocol/configuration.status.md`.
- `temper/docs/design/client/tools.md`, 11.2.
- The engine-domain world: `temper/tests/engine/domain/`.
- The forge protocol world: `temper/tests/engine/protocol-forge/`.
- `jig/docs/design/domain/connectors.md`, 6.1.
- `skein/docs/foundation/testing-strategy.md`, 8.

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands.** Never `tools/tiers/run`, nor a browser, process or live tier or one of their stories. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made, and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
