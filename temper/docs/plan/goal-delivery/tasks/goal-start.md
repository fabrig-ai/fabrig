+++
id = "goal-start"
part = "goal-workspace"
kind = "feature"
needs = ["charter-delivery", "run-branch"]
files = [
  "temper/crates/temper-engine-domain-forge/src/items.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/goal_start.rs",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-smith/src/start.rs",
  "temper/crates/temper-engine-smith/src/tests.rs",
  "temper/tests/worker/checkout/src/merges.rs",
  "temper/tests/worker/checkout/tests/suite/merges.rs",
  "temper/docs/design/domain/worker.status.md",
]
effort = "high"
minutes = 110
+++

# A goal's run starts from g, pushes to its run branch, and is told it delivers

## Goal

A goal's run under `modify`, holding its goal branch, starts from g's tip with whatever its run branch holds that g lacks merged in, and pushes to that run branch, so it takes r's writer slot and never g's. The workspace tells the charter, so jig grants `deliver`. Nothing has built this start before. Every other run's workspace and grants stay as they are, and a test holds that.

Acceptance:

- **The start,** as `notes/contracts.md`'s table has it, one test row each, in `temper-engine-domain-forge/src/tests/goal_start.rs`:
  - with `modify`, holding confirmed g, r no head: `Start::Base(g)`, push r, landing none, delivers;
  - with `modify`, holding confirmed g, r at a head: `Start::Merge { branch: g, base: head }`, push r, landing g, delivers;
  - the same with saved work: `Start::Saved(s)`, push r, landing none, delivers;
  - without `modify`, holding confirmed g: `Start::Branch(g)`, no push, not delivers, as today;
  - with `modify`, holding g drifted or not yet confirmed: as today, `Start::Base(default)`, push r, not delivers;
  - with `modify`, holding no g: as today, not delivers.
- **`Seed.modify`:** `run_workspace` gives it true only where the run's charter's `modify` is set, `may_push` allows r and the task holds r (`unheld_run_branch` false). So a goal that can't push r inspects g as today, with no merge in progress and no write. A test holds an authority without kind 2.
- **The grant:** `charter_with_history` sets `charter.delivers` from the repositories (`notes/contracts.md`). In `temper-engine-smith/src/tests.rs`, beside `a_report_run_cannot_modify_without_delivery`: with `delivers` on a repository, a Report run with `policy.modify` has `grants.deliver` with `checks_must_pass`, `grants.tools.modify`, `conventions.checks` of `.temper/pre-pr`, `outcome.change` none and `outcome.report` some. With `delivers` and an Open run, none.
- **Inspect alone holds:** a test named `a_chats_an_advisors_and_a_reviewers_runs_inspect_alone`, in `forge_routes/b2_tree.rs` beside `a_goal_without_modify_cannot_modify_but_its_change_producer_can`, in a world whose charters have `modify: true`: the three runs' charters have `grants.deliver` none and `grants.tools.modify` false.
- **The checkout:** three stories in `temper/tests/worker/checkout/tests/suite/merges.rs`, on `Start::Merge { branch: main, base: the run branch's head }`, push the run branch, landing `main`:
  - `a_goals_later_run_holding_its_run_branch_in_g_starts_on_gs_tip`: no merge commit and no conflict, its tree is g's;
  - `a_goals_later_run_merges_its_run_branch_into_gs_tip_and_fast_forwards_it`: a clean merge is committed before the run, and a delivery fast-forwards the run branch;
  - `a_goals_later_run_keeps_a_conflict_with_its_run_branch_in_progress`: its start names the file, and a delivery after settling it fast-forwards the run branch.

## Inputs

- `temper/docs/design/domain/forge.md`, 11.1 (*What a goal writes goes by its run branch*).
- `temper/docs/design/domain/worker.md`, 4.1 and 5 (*A merge in progress*, *A goal's later run*, *Push commits what the run checked*).
- `temper/docs/design/domain/agent.md`, 4.2 and 4.3.
- `jig/docs/design/domain/hosts.md`, 8.
- `temper/docs/plan/goal-delivery/notes/common.md` and `notes/contracts.md`.

## Owns and may change

- **Owns:**
  - `Seed`, `Item`, `workspace_item` and `task_workspace_item` in `temper-engine-domain-forge/src/items.rs`, and the new `src/tests/goal_start.rs`.
  - `ForgeRepository` in `temper-engine-domain/src/boundary.rs`; `run_workspace` and `workspace_repository` in `route/forge_route.rs`, those functions alone (`model-catalogue-2` writes elsewhere in `route/`).
  - `charter_with_history` in `temper-engine-smith/src/start.rs`, and its tests.
  - `temper/tests/worker/checkout/src/merges.rs` and its suite.
- **May change:**
  - **Literals:** each `Seed { .. }` and `ForgeRepository { .. }` the compiler names takes the new field, `temper-service`'s tests and `temper/crates/temper-engine-domain-forge/src/tests*` among them, each only for that. `temper-service/src/translate.rs` doesn't translate `delivers`.
  - `temper/crates/temper-engine-domain-forge/src/tests.rs`: the `mod goal_start;` line.
  - `temper/docs/design/domain/worker.status.md`: remove the **Not yet** entry for section 5 that names `temper/docs/plan/goal-delivery/`, once the stories pass. This departs from the implementer's text.
  - **Readers of a goal's workspace:** `World::configured`'s charter has `modify: true`, so a goal holding g there now starts `Base(g)` with push r and r's writer. A test asserting otherwise for a goal that isn't about delivering takes `charter.policy.modify = false` in its own world, as `goal_changes.rs::inspecting_goal` does; or, where the assertion's point is only g's slot or g's hold, it stays as it is. Name each in the report.
- **Not here:** `goal_delivery.rs`, which `goal-walks` lifts. The worker's production code: the design says it needs none. If a story shows otherwise, a fix in `temper-worker-domain-checkout/src/hold.rs` that keeps every existing story green is pre-approved, and say so in the report. A wire or schema change is a question.
- **Options:**
  - A contained start's `deliver` with no new edit: the worker's `resume_contained` treats it as a saved start from g, so it pushes g's tip to r. That is a fast-forward and harmless: keep it, and say what the story saw.
  - g's or r's head is `branch_head`'s: use it, without a new read.

## Steps

1. Tests first, in the order of the acceptance. See each fail for the rule's reason.
2. `items.rs`: the new `Seed` and `Item` fields, the selection and its landing. `workspace_item`'s `landing` for a non-inherited item comes from the selection: return it beside `CheckoutSelection`.
3. `forge_route.rs`: `Seed.modify` in `run_workspace`, reading the charter by `context.executor` as `chat_change` reads one (`domain.core.charter`); `ForgeRepository.delivers` in `workspace_repository`, false with no push.
4. `start.rs`: `charter.delivers`, set where `conventions` is.
5. The checkout's stories, on `merges.rs`'s world, as its `Case::Contained` and `Case::Resumed` are set up.
6. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-forge -p temper-engine-smith -p temper-worker-checkout-world --all-targets -- -D warnings`.
7. **A change of meaning runs its readers' tests:** what a goal's workspace holds changed, so run all of `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-smith -p temper-engine-domain-world -p temper-worker-checkout-world -p temper-service`.
8. Commit and push.

## Report

Each rule you broke by hand and its failure; each reader you changed and how; what a contained start's empty delivery did; and any production change in the worker.
