+++
id = "early-route"
part = "early"
kind = "feature"
needs = ["kept-start", "kept-started-from", "checkout-heads-engine", "early-step"]
files = ["temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/link.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/**", "temper/crates/temper-engine-domain/src/tests/**", "temper/crates/temper-engine-domain-forge/src/**", "temper/tests/engine/domain/src/**", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/tests/suite/forge_routes/early_start.rs", "temper/tests/engine/forge/**", "temper/docs/design/domain/forge.status.md", "temper/docs/design/domain/landing.status.md"]
effort = "high"
minutes = 150
+++

# early-route: the change procedure starts kept, and its producer starts early

## Goal

temper's change procedure starts kept, and its top gives the step what C5 reads and does
what it decides: the pending dependencies and their pushed heads, the producer's start from
the landing tip with those heads merged in, the dependencies marked started from, a
conflicting start cancelled, the next producer merging the landing tip first, and the
dependents stepped as their dependencies push (contract C6). One story runs it whole.

Acceptance:

- The root declares `KeptStart { connector: <the forge connector's number,
  `config.forge_connector`>, code: CHANGE_CODE }` in jig core's `kept_starts` where it makes
  the core (`temper-engine-domain/src/domain.rs`, `Core::new(config.core, ..)`).
- Each change step's `Facts.pending` as C6 has it, built beside `landing_route::kept`
  (`route/forge_route/landing_route.rs`), and passed with `StepChange` as `kept` is.
- The step that moves `early` from `None` to `Started` sends its heads' dependencies as the
  `ProcedureStep.started_from` of the decision that makes the producer.
- `workspace_item` (`temper-engine-domain-forge/src/items.rs`) gives `Start::Merged` and
  `Start::Merge` as C6 has them; the landing tip for `Merge`'s base is the forge's head of
  the change's landing branch, as an update's base is read.
- A `Preparation::Conflicted` reaching the root (`link.rs`, `decorate_answer`) for an attempt
  of a change's live producing delegate is kept by the forge for that change until the
  delegate ends (in memory: a restart's next prepare conflicts again) and steps it with
  `Heard.start_conflict`; `Decision::CancelProducer` cancels that delegate, its reason
  `its early start conflicts with <the head, short hex>`, and its end holds nothing.
- A change's branch head first read or moved (`answered`, `branch_changed`, delivery heads),
  and a change's task going on from kept, step each live change waiting at producing with no
  producer whose task's pending dependencies name it.
- The story below passes. Every story of `temper/tests/engine/domain`'s `forge_routes` and
  `temper/tests/engine/forge` passes; a story whose expectations change only because a
  dependent change now starts kept (active and kept rather than waiting, its producer
  starting on its dependency's pushed head) is updated to the new behaviour and named in
  the report (`change_tool.rs`'s `two_changes_in_one_call_are_one_batch_the_second_after_the_first`
  and `goal_changes.rs`'s `a_goal_makes_two_changes_into_its_branch_the_second_after_the_first`
  are the likely ones); one that fails otherwise is a question.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11 (every bullet), 8.2 steps 1 and 7, section 9
  (a kept change is not ready), 5.2 (the forge's reads of a branch).
- `temper/docs/design/domain/landing.md`, 4.1 (a dependency across repositories; the
  change procedure starts kept).
- `jig/docs/design/domain/connectors.md`, 6.1 and 6.2; `jig/docs/design/domain/tasks.md`, 5.7.
- `temper/docs/design/protocol/channel.md`, section 5 (`conflicted`, routed to the change).
- Status entries this task removes: `temper/docs/design/domain/forge.status.md`, the whole
  "8.11: early starts: ..." entry; `temper/docs/design/domain/landing.status.md`, 4.1's "a
  dependent made with its dependency, started kept (jig's `domain/connectors.md`, 6.1);"
  (the rest of 4.1's entry stays).
- `temper/docs/plan/reference-m6/notes/common.md`; `notes/contracts.md`, C1 to C6.

## Owns and may change

Owns the files in the header; tasks `checkout-heads`, `checkout-heads-engine` and
`early-step` changed some of them before you: build on their hunks.

- In the root's `forge_route.rs`, keep the hunks to `StepChange`'s facts, the decision's
  translation into jig's `ProcedureStep`, the producing delegate's cancel, and the
  dependents' steps; other plans change `change_delegate`, `chat_change` and
  `change_landing_words` beside you.
- Reader and neighbour files: stories of `temper/tests/engine/domain/tests/suite/**` and
  `temper/tests/engine/forge/tests/**` whose expectations change as Acceptance says, those
  expectations only.
- Options: if the forge's working set does not read a dependency's branch until something
  asks, read it as the change's own branch is read at each step (`StepStage::Branch`), and
  say so. If the world's scripted producers cannot deliver before they finish, keep the
  dependency pending with its head pushed by leaving its pull request's mergeability
  unsettled, and say so. If a change's dependency is another change's task that the forge
  has no row of yet (made in the same batch, not yet started), it is `Change` with no head.

## Steps

1. The declaration, `pending`, the marks, the starts, the conflict's route and the cancel,
   the dependents' steps.
2. Forge connector unit tests (`temper-engine-domain-forge/src/tests/`): `pending`'s kinds
   for a same-branch change, another repository's change and a task that is no change; the
   `Merged` and `Merge` items of a producer; the dependents stepped on a head's first read.
3. The story, a new file `temper/tests/engine/domain/tests/suite/forge_routes/early_start.rs`
   (its `mod` line in `forge_routes.rs`), modelled on `goal_changes.rs`'s
   `a_goal_makes_two_changes_into_its_branch_the_second_after_the_first`:
   `a_dependent_change_starts_early_on_its_dependencys_pushed_head`: G makes C2 and C3 into
   g in one call, C3 after C2; C3's task is active and kept at once and has no producer;
   C2's producer pushes a head and C2 is still pending; C3's producer is assigned
   `Merged { branch: g, heads: [C2's head] }` and C3's task marks C2 started from; C3's
   producer finishes; C3 opens and is not ready while C2 is pending; C2 lands; C3 merges into
   g; g's history holds C2's head and C3's merge.
4. Remove the status texts above. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-forge -p
   temper-engine-domain-world -p temper-engine-forge-world --all-targets -- -D warnings` (the
   world packages by their names in `cargo metadata`); the focused tests of those packages
   and of `temper-service`. Commit, push.

## Report

As the implementer's text says; name each story whose expectations changed and why, and
where the dependents' steps hook in.

## Answers

2026-11-03. **The fake forge squashes, so g's history cannot hold C2's pushed head: add a
fake-forge prerequisite, or change the story's history assertion?** Change the assertion; no
prerequisite, no change to skein. skein's fake forge is designed to squash every merge
(`skein/docs/design/fake-forge.md`, 2.2), and a goal's landing as a merge commit is the gap
`goal_changes.rs`'s ignored story already carries; neither is this task's. Keep the story
running, not ignored, and replace its last two history assertions with what the fake can show:

- g's history from main's tip to its tip is two commits, each the parent of the next: C2's
  landing (`change::State::Landed { merge, .. }` of C2) on main's tip, then C3's landing,
  which is g's tip;
- g's tip holds C2's work and C3's: its tree equals the tree of C3's pushed head (built on
  the merge of C2's pushed head), and holds the paths C2's pushed head added.

Everything before (the `Merged` start with C2's head, the started-from mark, C3 opening kept
and not ready, the landings in order) stays as it is. In `temper/docs/design/domain/forge.status.md`,
where the 8.11 entry is removed, add one line under 8.11: a dependent's landing holding its
dependency's pushed head in g's ancestry is unobserved in stories, the fake forge squashing
merges (`skein/docs/design/fake-forge.md`, 2.2). Then remove the status texts the task names
and finish the Steps.

The service fixture missing `started_from` (`temper-service/src/tests/fixtures.rs`) stays
under `## Left`, as reported.
