+++
id = "run-branch"
part = "run-change"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/held_branch.rs",
  "temper/crates/temper-engine-smith/src/calls.rs",
  "temper/crates/temper-engine-smith/src/tests.rs",
  "temper/docs/help/change.md",
]
effort = "high"
minutes = 90
+++

# `"run"` beside `"mine"`, and a change that goes on beside its requester's run

## Goal

`change` takes `"branch": "run"`: its requester's run branch, with no producing step, its head the branch's tip as read at opening, held by the goal, one at a time, and closing deletes nothing. It is `"mine"` on another branch. And such a change steps while its requester's run sits in that branch's writer slot, where today a taken slot stops every change.

Acceptance:

- **The call:** `{"words":"...","branch":"run","landing":"goal"}` decodes to `ChangeBranch::Run` and makes a change on `temper/<root>/r<task>`, with the parameters `notes/contracts.md` names. Any other value is still `Problem::Range(b"branch")`.
- **Its refusals:** the four words in `notes/contracts.md`, each with a test. `"mine"`'s words, and its refusal with `"landing":"goal"`, are unchanged. `"run"` with `"landing":"goal"` is allowed: it isn't a branch into itself.
- **Its life is `"mine"`'s:** no producer, no hold taken or handed, its pull request r into g, nothing deleted as it closes. A test reads that on the change's row and on the goal's hold.
- **The slot** (`notes/contracts.md`; `forge.md`, 8.5): with `Writer::Run { task: holder, .. }` in the slot of a branch its requester holds, the change's `Facts.writer_taken` is false and no `BranchMoved` is raised for a head its read finds. A head that moves under that writer is taken by `take_pushed_head`, checked again.
- **What stays:** a delegate's run in the slot, a landing into the branch and an update entry stop the change as today. A backward move or another identity's move is still drift. A handed branch, whose holder is the change itself, is untouched. Each has a test or an existing one you name.
- **A branch nothing has reached:** a run change whose branch has no head holds at opening as `Hold::BranchMissing` (`forge.md`, 8.1). A test holds it, and a cancel ends it.
- `temper/docs/help/change.md` names `run` beside `mine`, in one clause, keeping the page's bound.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1 (*A change on its requester's held branch*), 8.5 and 11.
- `temper/docs/design/domain/agent.md`, 4.3.1 (`branch`).
- `jig/docs/design/domain/connectors.md`, 11.
- `temper/docs/plan/goal-delivery/notes/common.md` and `notes/contracts.md`.

## Owns and may change

- **Owns:**
  - `ChangeBranch` in `temper-engine-domain/src/boundary.rs`.
  - `change_branch` in `route/forge_route.rs`, and its tests in `route/forge_route/tests.rs`.
  - `change_facts` and what you add beside `change_branch_hold` in `temper-engine-domain-forge/src/domain.rs`, and `src/tests/held_branch.rs`.
  - The `change` decoding and `PLAN_CHANGE`'s `branch` member in `temper-engine-smith/src/calls.rs`, and its tests in `src/tests.rs`.
- **May change:**
  - `temper/docs/help/change.md`: the `branch` clause. Departs from the implementer's text, since a tool's value is its help's.
  - `temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs`: tests of the call.
  - **Figures the schema's growth moves:** `PLAN_CHANGE` grows by some tens of bytes. A test holding a stock charter's room (`temper-service/src/tests/stock_brief.rs`, `temper-engine-smith/src/tests/room.rs`, `temper-engine-domain/src/tests/room_checks.rs`) may change its figure only, with the figure in `temper/docs/design/domain/agent.md`, 4.3 ("at least 361 bytes") if that one moved. Say so in the report. `model-catalogue-2/stock-models` writes `stock*.rs` too, so keep to the figure.
- **Not here:** `items.rs` and `run_workspace`, which `goal-start` changes. So on this trunk a goal's run doesn't yet write r: a positive test seeds r's forge hold itself (`forge::Event::Hold { entry: 0, task, resource, from: None }`), and the slot's writer with `Event::Claim`.
- **Options:**
  - `change_branch`'s body differs only by letter and words: one function taking the choice. The words of `"mine"` stay byte for byte.
  - An early-start dependent doesn't see r's head, because the fresh read of the change doesn't reach `d.heads`: have that read record it (`owned: false`) and wake dependents as `branch_changed` does, and say so. `goal-walks`'s story proves it.

## Steps

1. Tests first: the decode in `temper-engine-smith/src/tests.rs`; the slot in `held_branch.rs`, on a row whose branch is a run branch, the writer a `Writer::Run` of the holder and then of a delegate; the refusals in `route/forge_route/tests.rs`, beside the `"mine"` test near line 3168. See each fail.
2. `ChangeBranch::Run`, the decoding and the schema: `"enum":["mine","run"]`, and to the description `run: your own run branch, what you delivered, landed as it stands, with no coder;`.
3. `change_branch`.
4. The slot: a small function beside `change_on_held_branch`, used by `change_facts` for both `writer_taken` and the `branch_moved` guard.
5. The help's clause, then `heavy cargo nextest run -p temper-docs` if its bound test lives there (`rg` for it).
6. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-forge -p temper-engine-smith --all-targets -- -D warnings`.
7. **A change of meaning runs its readers' tests:** what a taken slot means changed, so run all of `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain-forge-change -p temper-engine-domain -p temper-engine-smith -p temper-engine-domain-world`, and `heavy cargo check -p temper-service --tests`.
8. Commit and push.

## Report

Each rule you broke by hand and its failure; the figures that moved; whether you took the early-start option; and any reader you had to change.

## Answers

2026-11-06. **Regenerate `temper/tests/playground/tests/fixtures/story-a.log` here, or have its owner refresh it first?** Neither: leave it, and list it under `## Left`. The run's acceptance is met, and the log is outside this task. The schema's growth is the request's item 3, and the log's format doesn't change, so nothing is stopped by `notes/common.md`'s rule on goldens, and this answer and your report tell its owner. The trunk's end check and its repair take the visit once the merges meet, regenerating with its generator (`SKEIN_CODEGEN_WRITE` on `story_a_walks_from_the_welcome_to_the_csv_change_landed`), never by hand.

What's left for this task: the pushed commit stands. The fail-fast stop left the figures this spec gives you unobserved, and `goal-start` runs `temper-service`. So run `heavy cargo nextest run -p temper-service --lib -E 'test(/^tests::stock::brief::/)'` (the file compiles as that module; check the count). If it fails on a room figure the schema's 88 bytes moved, change that figure only, with what's derived from it at `temper-service/src/limits.rs`, near line 397 (the comment and `brief_bytes`, which is charter 3's room less 1,473), and `temper/docs/design/domain/agent.md`, 4.3 ("at least 361 bytes") if that one moved, then commit and push. `limits.rs` is named here for those two alone. Say in the report which figures moved. Don't rerun the wider suites or chase other replay failures: name them under `## Left` as unobserved.
