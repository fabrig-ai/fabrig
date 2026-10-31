+++
id = "jp-people"
part = "project-delete"
kind = "feature"
needs = ["jig-ask"]
files = [
  "jig/crates/jig-domain-people/src/**",
  "jig/tests/people/tests/suite/drop_project.rs",
]
effort = "high"
minutes = 75
+++

## Goal

Give the people child what a project's deletion asks of it
(playground-reset.md, 5): a task's traces forgotten when the task is
erased (its chat, every party's read position of it, its waiting
entries), the project's roster dropped when its own rows go, and retained
answers that named the gone project answering *unknown* on replay
(`notes/contracts.md`, 1), with the start no longer asserting a roster a
retained answer names.

Acceptance (`jig-people-world`, new `tests/suite/drop_project.rs`):

- `forgetting_a_task_drops_every_partys_read_and_its_chat`: two parties
  read conversation 9 (`ReadConversation` rows) and it waits on a role of
  its project: after `Event::ForgetTask { task: 9 }`, both
  `ReadConversation` keys are erased, the chat's totals, replies, needs,
  waiting entries, unread and recent items of task 9 are gone from every
  inbox, and nothing of task 10 changed. No `EndedRead` erase is emitted
  (the core erases those rows from its read of the task's rows).
- `dropping_a_project_drops_its_roster`: after `Event::DropProject {
  project: 3 }`, `Key::Roles(3)` is erased, `has_project(3)` is false,
  every waiting entry for a role of project 3 is gone and project 4's
  roster is untouched.
- `a_retained_answer_naming_a_gone_project_answers_unknown`: a party's
  retained `Rename` on project 3 (outcome `Renamed`), and a deployment's
  retained `MakeProject` whose outcome is `ProjectMade { project: 3 }`:
  after the drop, sending each key again answers
  `Outcome::Refused(Refusal::Unknown)`; the answers stay stored (not
  erased) and are replayed as before for project 4.
- `a_retained_project_removed_replays_as_itself`: a retained answer
  `ProjectRemoved { project: 3 }` (to an `Ask::RemoveProject` and to an
  `Act::RemoveProject`) replays as `ProjectRemoved { project: 3 }` after
  the drop: the one answer still true of a gone project, so a kept act
  sent again after a restart ends.
- `a_start_with_an_answer_naming_a_gone_roster_opens`: a store holding a
  retained `RolesSet { project: 3 }` and no `Roles(3)` row starts and
  opens (no assertion; the answer's roster link is skipped), and replays
  it as unknown.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1
  (retained answers; `ProjectRemoved`).
- `temper/docs/design/draft/playground-reset.md`, section 5 ("What
  outlives it": inboxes and read positions drop what named its tasks).
- `jig/docs/design/domain/people.md`, section 6 (the inbox and read
  positions) and 5.3 (retained acts).
- `jig/crates/jig-domain-people/src/inbox.rs`'s `forget_chat` and
  `forget_event`, `replace_waiting`, `prune_unheld_roles` (what exists).

## Owns and may change

Owns `jig-domain-people/src/**` (after `jig-ask`, which owned it) and the
new test file. May change, as `notes/scope.md` says: the world's
`tests/suite/main.rs` (`mod` line); the core's exhaustive matches on
`people::Event` or `people::Request` the variants force (none are
expected: the core builds events and maps requests in
`routing/people.rs`, whose arms you may add as no-ops if forced).

Not yours: when the core sends these (`jp-delete`); the core's fence on
new requests (`jp-close`).

## Steps

1. `boundary.rs`, `Event`:

   ```rust
   /// An erased task's traces: its chat, every party's read of it, its
   /// waiting entries (temper/docs/design/draft/playground-reset.md, 5).
   ForgetTask { task: u64 },
   /// A deleted project's roster, once its tasks are erased (playground-reset.md, 5).
   DropProject { project: u32 },
   ```

2. `ForgetTask`: `forget_chat` for the task, then the task's `needs`,
   `replies`, `waiting` entries (`replace_waiting` with no entries does
   this), `unread`, `result_census` and `recent` items naming it, removed.
   Unlike `forget_event`, it erases no `EndedRead` and does not depend on
   `has_project`.
3. `DropProject`: `roles.remove(project)` and `Request::Erase { key:
   Key::Roles(project) }`; every `waiting` entry under `Whom::Role {
   project, .. }` removed; any other map keyed or valued by the project
   (rg `project` in `domain.rs` and `inbox.rs`) cleared of it.
4. Replay (`domain.rs`'s `admit_ask` replay, `domain/acts.rs`'s `admit`
   replay): a helper `fn named_project(ask_or_act, outcome) -> Option<u32>`
   giving the project a retained ask, act or outcome names (the ask's
   `project()` when it is not `DEPLOYMENT`; `Act::Close`, `Cancel`,
   `Adopt` and `RemoveProject`'s `project`; an outcome's `project`, as
   `ProjectMade`, `RolesSet`, `Renamed`, `PolicyChanged`, `PoolSet`,
   `Adopted`, `Compound` carry it). If it names a project not in `roles`
   and the outcome is not `ProjectRemoved`, the replay answers
   `Outcome::Refused(Refusal::Unknown)`; the stored answer is kept as is.
5. Start: `domain/start.rs`'s `start_link` and the `restored` path in
   `domain.rs`: a `RolesSet` answer whose roster is missing links nothing
   and fails nothing.
6. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-people-world --test suite -E 'test(/drop_project/)'`
     (binary name from `jig/tests/people/Cargo.toml`);
   - `heavy cargo nextest run -p jig-domain-people -p jig-people-world`;
   - `heavy cargo nextest run -p jig-domain-core -p jig-core-world -p jig-conformance-world`;
   - `heavy cargo check --tests -p temper-engine-domain`, and its focused
     tests if you edited it;
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
7. Commit and push.

## Report

The standard report, plus: every map you cleared for a task or a
project; the outcomes `named_project` reads a project from.
