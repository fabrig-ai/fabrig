+++
id = "fe-playground-forge"
part = "pg"
kind = "feature"
needs = ["fe-remove"]
files = ["temper/crates/temper-engine-domain-forge/**"]
effort = "high"
minutes = 110
+++

## Goal

The forge connector's half of the playground on records (draft
`temper/docs/design/draft/playground-reset.md`, 6.1 to 6.3; `notes/contracts.md` T6;
`notes/fe-surface.md` sections 4 and 5): `StartPlayground` makes the `Playground` row and
the fake forge's record and asks the root for the rest; `EndPlayground` resets it step by
step from the row, each step resumed at every start; a start that finds the row `Running`
resets it. The demo's end on another forge's connect goes. The demo's types stay until
`fe-demo-gone`; nothing here calls them any more.

Acceptance:
- `StartPlayground`, decided once for all callers waiting (as the demo's start shares one
  operation, bounded by `l.applications`):
  - refused `NotOffered` when `machine.playground` is `None`, while any `Kept` record that is
    not the playground's exists, or while the row reads `Resetting`;
  - with no row: in one decision the playground's record made from `machine.playground`
    (numbered as `MakeForge` numbers, `playground: true`, `Mode::Fake`, `Kept`; `Full` from
    numbering answers `NotOffered`), `ForgeNumbers` and `Playground { state: Running, forge,
    project: None, account: None }` saved, `Register(record)` sent, and under the box
    `SetUpGroup { forge, request, entry: record.setup }` when the forge needs its group (as
    the demo's `needs_group` decides);
  - with a `Running` row: the group asked again if it needs it;
  - then `Request::PlaygroundStart { request }`; `PlaygroundProject { project }` and
    `PlaygroundAccount { account }` save the row with the number; `PlaygroundAnswered {
    answer }` answers every waiting start (`Answered::StartPlayground`), kept in memory as
    the demo's answers are.
- `EndPlayground`: `NotOffered` with no row (kept in memory); otherwise, in one decision, a
  `Running` row saved `Resetting(Project)` and the step begun; every caller waits for the
  row's erasure and is answered `EndAnswer::Reset` then. The steps, in order, each begun
  after the last one's end:
  - `Project`: with `project: Some(p)`, `Request::ResetAct(ResetAct::RemoveProject {
    project: p })`, ended by `ResetActed { act, keep: false }`; with none, ended at once;
  - `Forge`: `fe-remove`'s `remove` of the playground's record (no refusal), ended in the
    decision that erases its `Forge` row; a record already gone ends it at once;
  - `Account`: with `account: Some(a)`, `ResetAct(ResetAct::RemoveAccount { account: a })`,
    ended by `ResetActed { keep: false }`; with none, at once;
  - then the row erased and the waiting ends answered `Reset`.
  Each step's start saves the row at that step (`Resetting(step)`).
- At every start (`SendKeptActs`): a restored `Running` row is saved `Resetting(Project)`
  and its reset begins (draft section 9's default); a restored `Resetting(step)` row sends
  that step's act again (or goes on, for `Forge`, when the record is gone).
- `forge_ended` reads the playground's forge as any other (no demo special case for it);
  the playground's record lists in `forges()` while `Kept`.
- Gone from the connector's behaviour: `demo::connected` (the demo's end on another
  forge's connect), `demo::restart`'s group ask for a `Demo` row, `DemoState::Ended`'s
  effects; `Domain::playground()` returns the row.
- Unit tests below pass; tests of the demo's end that no longer hold are deleted (list
  them in the report).

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T4, T6;
  `notes/fe-surface.md` sections 1, 4, 5.
- `temper/docs/design/draft/playground-reset.md`, 6.1 to 6.3 and section 9's first default.
- `temper/docs/design/protocol/setup.md`, 8.4 (the demo's start, which this replaces).
- `temper/crates/temper-engine-domain-forge/src/demo.rs` (the start's sharing and group
  ask, to reuse).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/`.
- May change: nothing else.
- Pre-approved options:
  - Module: default, a new `src/playground.rs`; `src/demo.rs` keeps only what the root's
    old demo code still calls (`fe-demo-gone` deletes it).
  - Making the record: reuse `fe-make`'s numbering and `keep_record`; do not duplicate
    them. While the connector was built with document forges (the transition of
    `notes/sv-progress.md`, contract request 1), the playground's record takes the first
    number above every document forge and the counter moves past it, so it never shares a
    document forge's number; `MakeForge` stays refused then (`fe-make`).
  - The `Forge` step's end: default, `removal::answered` tells the playground module when
    it erases a record whose number is the row's forge. If a check at each answer is
    simpler, take it.
  - Many `EndPlayground` callers: bound them by `l.applications` as starts are;
    `ApplicationAgain { Busy }` past it.

## Steps

1. `src/playground.rs`: `start`, `answered`, `end`, `step`, `resume` (from
   `SendKeptActs`), `restore` of `Stored::Playground`; the step's arms for
   `Ask::StartPlayground`, `Ask::EndPlayground`, `PlaygroundProject`, `PlaygroundAccount`,
   `PlaygroundAnswered`, `ResetActed`; `application_answer` for both answers.
2. Remove the calls to `demo::connected` and `demo::restart` (`domain.rs`'s
   `Connection` arm and `read_afresh`).
3. Tests in a new `src/tests/playground.rs`:
   `a_start_makes_the_row_and_the_fake_forge_in_one_decision`;
   `a_start_is_not_offered_beside_a_real_forge_or_while_resetting`;
   `starts_share_one_answer`;
   `the_reset_removes_project_forge_and_account_in_order_then_the_row`;
   `a_reset_without_project_or_account_skips_those_steps`;
   `each_step_is_resumed_at_start_from_the_row`;
   `a_start_that_finds_the_playground_running_resets_it`;
   `another_forge_connecting_no_longer_ends_the_playground`.
4. Run, one at a time: `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
   `heavy cargo nextest run -p temper-engine-domain --lib`; `heavy cargo check --workspace
   --all-targets`; `cargo fmt --all`; `heavy cargo clippy -p temper-engine-domain-forge
   --all-targets -- -D warnings`; `python3 -I -B scripts/old_forms.py` from `temper/`.
5. Commit, push your branch.

## Report

- The demo tests deleted, and any reader of the demo's state that no longer means anything
  (for `fe-demo-gone`).

## Answers

### 2026-11-01: Rerun this task after fe-remove publishes its removal implementation (recommended), or reassign that missing implementation to this task? The partial playground work is committed and pushed. (playground-reset-r6/fe-playground-forge-1, foreman playground-reset-r6/fe-playground-forge/foreman-1)

Rerun fe-playground-forge after fe-remove publishes its removal implementation; removal remains fe-remove’s work.
The task spec declares `needs = ["fe-remove"]`.
Its acceptance requires: “`Forge`: `fe-remove`'s `remove` of the playground's record (no refusal)”.
The pre-approved option specifies: “`removal::answered` tells the playground module when it erases a record whose number is the row's forge.”
The next run integrates the completed dependency, combines removal handlers and replay with playground progression, resolves conflicts, and completes all required checks.
The partial pushed work is not accepted as done: the report explicitly says acceptance is incomplete and tests remain unverified.

### 2026-11-01: The required fe-remove implementation is still unpublished. Rerun after fe-remove publishes (recommended, following the recorded answer), or supply the completed dependency commit if it was published under another ref? (playground-reset-r6/fe-playground-forge-2, foreman playground-reset-r6/fe-playground-forge/foreman-2)

Rerun fe-playground-forge after fe-remove publishes its removal implementation; removal remains fe-remove’s work.
The recorded answer explicitly requires: “Rerun fe-playground-forge after fe-remove publishes its removal implementation”.
The spec declares `needs = ["fe-remove"]`.
Acceptance requires: “`Forge`: `fe-remove`'s `remove` of the playground's record (no refusal)”.
The next run integrates the completed dependency, combines removal handlers and replay with playground progression, and completes all required checks.
The partial pushed work is not accepted as done: the report explicitly says acceptance remains blocked and checks remain unverified.
The requested report file could not be written because this environment permits only filesystem reads.

### 2026-11-01: Rerun after fe-remove publishes its completed removal implementation (recommended, as the two recorded answers require), or supply its completed commit if published under another ref? The partial playground work remains committed and pushed. (playground-reset-r6/fe-playground-forge-3, foreman playground-reset-r6/fe-playground-forge/foreman-3)

Rerun fe-playground-forge after fe-remove publishes its removal implementation; removal remains fe-remove’s work.
The recorded answer requires: “Rerun fe-playground-forge after fe-remove publishes its removal implementation”.
The spec declares `needs = ["fe-remove"]`.
Acceptance requires: “`Forge`: `fe-remove`'s `remove` of the playground's record (no refusal)”.
The next run integrates the completed dependency, combines removal handlers and replay with playground progression, and completes all required checks.
The partial pushed work is not accepted as done: the report explicitly says acceptance remains blocked and checks remain unverified.
The requested report file could not be written because this environment permits only filesystem reads.

### 2026-11-01: Rerun after fe-remove publishes its completed removal implementation (recommended, following the three recorded answers), or supply its completed commit if published under another ref? The partial playground commit remains pushed. (playground-reset-r6/fe-playground-forge-4, foreman playground-reset-r6/fe-playground-forge/foreman-4)

Rerun fe-playground-forge after fe-remove publishes its removal implementation; removal remains fe-remove’s work.
The recorded answer requires: “Rerun fe-playground-forge after fe-remove publishes its removal implementation”.
The spec declares `needs = ["fe-remove"]`.
Acceptance requires: “`Forge`: `fe-remove`'s `remove` of the playground's record (no refusal)”.
The next run integrates the completed dependency, combines removal handlers and replay with playground progression, and completes all required checks.
The partial pushed work is not accepted as done: the report explicitly says acceptance remains blocked and checks remain unverified.
The requested report file could not be written because this environment permits only filesystem reads.

### 2026-11-01: four runs found fe-remove's removal unpublished; needs judgement. (playground-reset-r6/fe-playground-forge-4)

Keep the task as written; start it again only once fe-remove has landed on the trunk.
fe-remove was itself blocked on `sv-skein-retire`, and its branch went stale; its answer of
today has it merge the trunk and build the removal, which its run 5 is doing. Until
`playground-reset-r6/fe-remove` is merged into `playground-reset-r6/integration`, a run
of this task can only find `removal.rs` missing, as runs 1 to 4 did.

First step of the next run: `git fetch origin`, then merge
`origin/playground-reset-r6/integration` into this branch (a merge, no rebase, no
force-push). Then check that `temper/crates/temper-engine-domain-forge/src/removal.rs`
exists with the `remove` that `src/playground.rs` calls. If it does not, stop with that
one line as the report; do not ask this question again. After the merge, combine the
staged playground handlers in `domain.rs` with fe-remove's kept-act handlers and replay,
using the pre-approved default (`removal::answered` tells the playground module). Then run
every check in Steps 4 and 5. Commit 8edae6bfef is the starting point, not accepted work.
