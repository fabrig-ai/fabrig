# debt-5: the plan

From `inputs/request.md` (fabrig/debt, 2026-11-04): four debt parts, one per tree. `notes/common.md`
holds the standing rules every task names.

## Parts and tasks

1. **temper-items** (temper D007, D008, D009, D010), slice 1.
   - `tasks/temper-docs-guide.md`, medium, ~20 minutes, needs nothing.
   - `tasks/temper-pipeline-names.md`, docs, medium, ~35 minutes, needs nothing.
   - `tasks/temper-account-counts.md`, medium, ~35 minutes, needs nothing.
   - `tasks/temper-fixtures-kit.md`, migration, medium, ~60 minutes, needs nothing.

2. **smith-lifecycles** (`inputs/smith-lifecycles-review.md`, findings 1 to 10; `skein/docs/foundation/programming-model.md, 5.4`),
   slice 2. `notes/smith-review.md` holds the check against the code. No finding is skipped.
   - `tasks/smith-domain-test-helpers.md`, medium, ~25 minutes, needs nothing: finding 2.
   - `tasks/smith-domain-machines.md`, medium, ~30 minutes, needs the last: findings 7, 8, 9.
   - `tasks/smith-agent-test-helpers.md`, medium, ~20 minutes, needs nothing: finding 2.
   - `tasks/smith-agent-machines.md`, medium, ~50 minutes, needs the last: findings 1, 3, 4, 9, 10.
   - `tasks/smith-host-io.md`, medium, ~40 minutes, needs nothing: findings 5, 6, 9, 10.

3. **skein-forge-root** (skein D010; `skein/docs/design/forge.md, 4.4`), slice 3. `notes/forge-root.md`
   holds the layout, the calls and what runs. The cut wrote 4.4 and its **Not yet**.
   - `tasks/forge-limits.md`, medium, ~25 minutes, needs nothing.
   - `tasks/forge-neutral-modules.md`, medium, ~35 minutes, needs the last.
   - `tasks/forge-decimal.md`, migration, medium, ~15 minutes, needs the last: the measure reads 0.
   - `tasks/forge-plan-failure.md`, medium, ~15 minutes, needs the last.
   - `tasks/forge-users.md`, medium, ~25 minutes, needs the last.
   - `tasks/connection-twins.md`, medium, ~20 minutes, needs `forge-plan-failure`.
   - `tasks/forge-root-close.md`, docs, medium, ~15 minutes, needs `forge-users` and `connection-twins`: the item goes.

4. **jig-core-steps** (jig D001; `jig/docs/design/domain/root.md, 3`), slice 4. `notes/core-steps.md`
   holds what each root does with each arm, why the due set, and what must not change. The cut wrote
   3's new paragraph and its **Not yet**. **D001 stays open:** see the question below.
   - `tasks/core-due-activation.md`, high, ~100 minutes, needs nothing: `Now::Activate` goes.
   - `tasks/core-prepare-agent.md`, high, ~80 minutes, needs the last: `Now::PrepareAgent` and
     `Event::PreparedAgent` go; D001's text is brought up to date.

## The question

`d001-scope` in `plan.toml` is the user's. D001's Paid-when cannot hold as written: of its five arms, only
two can leave the core without a design change; the others do real work in temper, and the design holds
them. The default is being built, and it is a part of every other answer but (c), which keeps
`Now::Activate`. The part is in the plan and no task waits for the answer.

## Critical path

`core-due-activation` → `core-prepare-agent` (~180 implementer-minutes). Beside it:
`forge-limits` → `forge-neutral-modules` → `forge-decimal` → `forge-plan-failure` → `forge-users` →
`forge-root-close` (~130), with `connection-twins` beside `forge-users`; `smith-agent-test-helpers` →
`smith-agent-machines` (~70); `temper-fixtures-kit` (~60); `smith-domain-test-helpers` →
`smith-domain-machines` (~55). The parts read none of each other's code. `temper-pipeline-names`
and `temper-account-counts` share one file: the first leaves `temper-client-domain/src/limits.rs` to the
second.

## Overlaps

Against the trunks of `fabrig-scale` and `qa-out`, 2026-11-04:

- **skein and smith:** none. `fabrig-scale` touches no path under `skein/`, and none of smith's five tasks'
  files. All `bugs-skein` branches are merged, so the moves start from fixed code, and no task edits a
  fake forge.
- **`temper-account-counts`:** `temper-client-shell-web/src/limits.rs`, `temper-client-tools/src/limits.rs`,
  `temper-client-shell-terminal/src/limits.rs` and `temper/tests/client/src/fixtures.rs`, changed by
  `fabrig-scale/client-capacities`. One literal in each; the item names these files, so they cannot be
  avoided.
- **`temper-fixtures-kit`:** `temper/tests/real-loop/src/fixtures.rs` and
  `temper/crates/temper/tests/support/configuration.rs`, one line each on `fabrig-scale`'s trunk; and one
  table line of `temper/docs/design/domain/testing.md`, which `qa-out/acceptance` also changes.
- **`temper-pipeline-names`:** comment lines of `temper-service/src/limits.rs`,
  `temper/crates/temper/tests/support/owned.rs`, `temper/tests/engine/domain/src/forge_routes.rs`,
  `temper/tests/engine/domain/tests/suite/walking.rs` and `temper/tests/system/src/composed.rs`, and the
  date line and one clause of `temper/docs/design/client/README.status.md`.
- **`core-due-activation`, `core-prepare-agent`:** `jig-domain-core/src/routing/mod.rs`: one line
  on `fabrig-scale`'s trunk, in the `AnswerPayload` arm, beside the `PreparedAgent` arm.

## Calls made

- **temper D008** reads "a commit as history" as all 55 commit hashes in temper's comments, and "a ruling"
  also as `ruling 116` and `ruling 235`. Two of the item's open promises are kept already:
  `forge_component.installations` has its row in `limits/names.rs`, and `not_installed` is in
  `client/README.status.md`, so no status entry is added.
- **temper D009:** `AccountLimits` goes. `uses` becomes `Limits::row_uses`, for what it bounds; no jig change.
- **temper D010** is one kit, `temper-fixtures`, and `tier_task_show` goes to the engine world's library,
  which moves `temper-client-tools` to that world's `[dependencies]`.
- **smith:** every finding stands; 3, 5, 9 and 10's sketches are corrected in `notes/smith-review.md`. Finding
  9's rule stays in the commits: `programming-model.md`, 5.4 and `smith/AGENTS.md` do not change.
  `smith-host-io` does not wait for finding 2's helpers, which are in other crates.
- **skein D010:** `notes/forge-root.md` holds them: `Problem` keeps `String`, the one public name
  that changes; `Bounds`, "order" and the field walkers stay; `skein-forge-setup`'s like copy stays, the item
  not naming that crate; D005 and D006 stay open. About -50 non-test lines, not the item's 1.5k.
- **jig D001:** `notes/core-steps.md` holds them: the due set in place of the item's `Work`, and the one
  behaviour that changes on purpose. The cut's check added one thing to the survey: a `Person` task must
  stay out of `due`, or `Core::quiescent` never holds.
