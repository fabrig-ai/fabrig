+++
id = "jig-acts"
part = "project-delete"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-people/src/boundary.rs",
  "jig/crates/jig-domain-people/src/domain/acts.rs",
  "jig/crates/jig-domain-people/src/domain/start/fit.rs",
  "jig/crates/jig-domain-core/src/routing/deployment_acts.rs",
  "jig/crates/jig-domain-core/src/routing/vocabulary.rs",
  "jig/crates/jig-domain-core/src/connector.rs",
  "jig/crates/jig-protocol-config/src/acts.rs",
  "jig/crates/jig-protocol-config/src/tests/acts.rs",
  "jig/crates/jig-protocol-store/schema/people-v13.schema",
  "jig/crates/jig-protocol-store/src/generated/people_v13.rs",
  "jig/crates/jig-protocol-store/src/codec/people/acts.rs",
  "jig/tests/core/tests/suite/deployment_acts.rs",
  "jig/tests/people/tests/suite/acts.rs",
]
effort = "high"
minutes = 75
+++

## Goal

Add jig's two new deployment acts and the vocabulary every later
project-deletion and forge-removal task builds on: `Act::RemoveProject`,
`Act::Cancel`, their `ActKind`s and `engine.acts` names,
`Outcome::ProjectRemoved`, the connector notice
`connector::Ask::ProjectRemoved` and the core write
`Write::DropTranscript`. `Act::Cancel` is built whole; `RemoveProject`
is admitted and refused *not offered* until `jp-delete` builds it.

Acceptance:

- `engine.acts` accepts `"cancel"` and `"remove_project"`, and refuses an
  unknown name as today (`jig-protocol-config`'s acts tests).
- A granted `Act::Cancel { project, task, grace, reason }` on a live task
  of the project, whatever its requester (a person's conversation, a
  procedure task a conversation delegated, a deployment's task), cancels
  it and its delegates as `Close` does, answered as a compound request;
  on an ended task it is refused *ended*; on a task of another project,
  or none, *unknown*; ungranted, *not offered* (new cases in
  `jig/tests/core/tests/suite/deployment_acts.rs`, beside `Close`'s).
- A granted `Act::RemoveProject { project }` is refused *not offered*,
  not saved (as an ungranted act is), so `jp-delete` can build it with no
  stored answer in the way.
- The act and its outcome encode and decode to themselves in the people
  store family (`jig-protocol-store`'s codec tests), the schema changed in
  place.
- `Write::DropTranscript { task }` and `connector::Ask::ProjectRemoved {
  project }` exist; no core decision emits them yet; temper's root
  carries `DropTranscript` to its store's
  `jig_protocol_store::Write::DropTranscript { task }` and routes
  `ProjectRemoved` to nothing yet (a no-op arm, commented with
  `notes/contracts.md` section 1's owner: the forge-remove part).
- Every package of jig and temper builds; their tests pass unchanged in
  meaning.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1:
  the names and shapes, verbatim.
- `jig/docs/design/domain/people.md`, 5.3 (the deployment's acts: the
  table, keys, grants, restart).
- `jig/docs/design/domain/tasks.md`, section 7, the compound request.
- `jig/docs/design/protocol/configuration.md`, section 3, the `acts`
  member (around line 108).
- `jig/docs/design/protocol/store.md`, 5 and 5.1 (schemas change in place
  until the first release).
- `temper/docs/design/draft/playground-reset.md`, sections 4 and 5 (cite
  at type level).

## Owns and may change

Owns the files in the header. May change, each only as said:

- Every exhaustive match on `people::Act`, `people::ActKind`,
  `people::Outcome`, `connector::Ask` or the core's `Write`, in jig and
  temper (rg each; survey: `jig-domain-people/src/domain/acts.rs:31,55,257`,
  `domain/start/fit.rs:229`, `jig-domain-core/src/routing/deployment_acts.rs:21,56`,
  `temper/crates/temper-engine-domain/src/store.rs:779` (retained-act
  payload sizing), `temper-engine-domain/src/route.rs:264-278` (core writes),
  `temper-engine-domain/src/store.rs:552` (the root's `Write`),
  `temper-service/src/store/route.rs:102` (the store's writes),
  `temper-engine-domain/src/route/forge_route.rs:3590` onward (connector
  asks), `temper-engine-domain/src/route/demo.rs:341` (outcomes),
  `temper/crates/temper-client-tools/src/jig/render/outcome.rs`,
  `temper/crates/temper-client-view/src/cards.rs:153`,
  `temper/crates/temper-client-tools/src/jig/render/document/act_answers.rs:261`,
  jig's client and engine-clients outcome translators): an arm for each
  new variant, doing for it what the arm does for its kin (`Close` for
  `Cancel`, `MakeProject` for `RemoveProject`, `ProjectMade` for
  `ProjectRemoved`); a renderer's words for `ProjectRemoved` are
  "Project N deleted" in the style of its `ProjectMade` line.
- temper's root `Write` (`temper-engine-domain/src/store.rs`): a new
  variant `DropTranscript { task: u64 }`, carried by
  `temper-service/src/store/route.rs` to `jig_protocol_store::Write::
  DropTranscript { task }`, and its decision accounting
  (`temper-engine-domain/src/decision.rs:297`) charging it as an `Erase`.
- If the client wire's `Outcome` (`jig-client-protocol-wire`'s current
  schema) must carry `ProjectRemoved` for the build, add it there in place
  with its generated code and goldens; otherwise leave the client wire to
  `jig-ask`.
- The people store schema is changed in place
  (`people-v13.schema`), regenerated with `skein-codegen` as its header
  says; if the store's format check or goldens require a version bump,
  follow them and say so.

Not yours: `Ask::RemoveProject` (the party's request) and the client
contract's `ActKind`: task `jig-ask` adds them.

## Steps

1. `jig-domain-people/src/boundary.rs`:

   ```rust
   pub enum Act {
       // ... today's five ...
       /// The project's deletion, in the deployment's name
       /// (temper/docs/design/draft/playground-reset.md, 5).
       RemoveProject { project: u32 },
       /// The deployment's compound cancel of one live task and its delegates
       /// (playground-reset.md, 4; tasks.md, the compound request).
       Cancel { project: u32, task: u64, grace: Option<Duration>, reason: Box<[u8]> },
   }
   pub enum ActKind { Close, AddKey, RemoveAccount, Adopt, MakeProject, RemoveProject, Cancel }
   // ActKind::ALL gains both, in that order.
   pub enum Outcome { /* ... */ ProjectRemoved { project: u32 }, /* ... */ }
   ```

   `domain/acts.rs`: `valid` bounds `Cancel`'s reason as `Close`'s;
   `kind` maps both; `valid_outcome`: `Cancel` takes `Close`'s outcomes,
   `RemoveProject` takes `ProjectRemoved` naming its project. Owned
   memory (people.md 5.3) is unchanged: neither carries more than
   `Close` does.

2. `jig-protocol-config/src/acts.rs`: `b"remove_project" =>
   ActKind::RemoveProject`, `b"cancel" => ActKind::Cancel`; tests in
   `src/tests/acts.rs`.

3. `jig-domain-core/src/routing/deployment_acts.rs`:
   - `Cancel`: a `cancel(core, work, request, project, task, grace,
     reason)` beside `close`, sharing its body but for the two
     conversation checks (`flags.conversation` and a person requester),
     which `Cancel` does not make: delegates cancelled with no grace,
     then the task with `grace`, as one compound by
     `tasks::Party::Deployment { project }`. A task already gone answers
     as `Close`'s `closed_subject` does.
   - `RemoveProject`: `refused(work, request, people::Refusal::NotOffered)`
     without saving, with a comment naming `jp-delete`. If the admission
     path saves every refusal of a granted act, refuse it in admission
     (`jig-domain-people/src/domain/acts.rs`) instead, as an ungranted act
     is, and say so.

4. `jig-domain-core/src/routing/vocabulary.rs`: `Write::DropTranscript {
   task: u64 }`; `jig-domain-core/src/connector.rs`: `Ask::ProjectRemoved {
   project: u32 }`, each with a doc line citing the draft's section 5.

5. Store codec: people schema's `Act` and `Outcome` in place;
   `codec/people/acts.rs` both ways; codec tests for each new variant.

6. Tests: the acceptance's cases, in `jig-core-world`
   (`jig/tests/core/tests/suite/deployment_acts.rs`):
   `cancel_ends_a_live_task_of_any_requester_with_its_delegates`,
   `cancel_on_an_ended_task_is_refused_ended`,
   `cancel_on_another_projects_task_is_unknown`,
   `cancel_ungranted_is_not_offered`,
   `remove_project_is_not_offered_and_not_saved`; and
   `jig-people-world`'s acts suite where it enumerates `ActKind::ALL`.
   Then, one at a time:
   - `heavy cargo nextest run -p jig-protocol-config --lib`;
   - `heavy cargo nextest run -p jig-protocol-store`;
   - `heavy cargo nextest run -p jig-domain-people -p jig-domain-core`;
   - `heavy cargo nextest run -p jig-core-world -p jig-people-world`;
   - `heavy cargo check --tests -p temper-engine-domain -p temper-service
     -p temper-client-tools -p temper-client-view -p jig-engine-clients-world
     -p jig-client-world`, then the focused tests of every one you edited;
   - `cargo fmt`; clippy over every package you edited;
   - `python3 -I -B scripts/old_forms.py` from `jig/` and `temper/`.
7. Commit and push.

## Report

The standard report, plus: every match arm you added outside the owned
files; whether `RemoveProject`'s refusal is in routing or admission;
whether a store or wire version moved.

## Answers

2026-10-31.

- **May this task add a root `Key` variant so `Write::DropTranscript`
  coalesces on its own key?** Yes (option A). In
  `temper-engine-domain/src/store.rs`, `Key` gains `Transcript { task:
  u64 }`, a root-only key as `Key::Wires` is (doc line citing
  `playground-reset.md`, 5, and jig's `protocol/store.md`, 8.5), and
  `Write::key` maps `DropTranscript { task }` to it. No task-row key is
  reused: a drop and an erase of the same task in one decision keep
  their own places. Its consumers follow `Key::Wires` in each:
  - `Decision::write` (`decision.rs`) refuses `Write::Erase(Key::
    Transcript { .. })` as it refuses `Erase(Key::Wires)`, and refuses
    `DropTranscript { task: 0 }`; the consumed-write placeholder at
    `decision.rs:555` needs nothing (it is never submitted again);
  - `Range::contains` answers false; `record_bytes`' key arm and the
    retained-act sizing answer `Some(0)` where `Key::Wires` does;
  - `temper-service`: `store/route.rs` carries `DropTranscript { task }`
    to `store::Write::DropTranscript { task }` and answers `None` for
    `Erase(Key::Transcript { .. })` as for `Erase(Key::Wires)`;
    `store/ranges.rs`' cursor and `clients.rs:712` answer `None` as for
    `Key::Wires`.
  The root `Key` is not stored (the store encodes the drop as its own
  `Own::Transcript` key), so no store version moves. Name each arm in
  the report. The rest of the task stands; the refusal of
  `RemoveProject` in people admission, as the run found, is as step 3
  allows.
