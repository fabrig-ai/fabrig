+++
id = "jp-acts"
part = "jig-remove-project"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-people/src/**", "jig/crates/jig-domain-core/src/routing/deployment_acts.rs", "jig/crates/jig-domain-core/src/routing/vocabulary.rs", "jig/crates/jig-protocol-config/src/acts.rs", "jig/crates/jig-protocol-config/src/tests/acts.rs", "jig/crates/jig-protocol-store/schema/**", "jig/crates/jig-protocol-store/src/codec/people/**", "jig/crates/jig-protocol-store/src/generated/**", "jig/tests/core/tests/suite/deployment_acts.rs"]
effort = "high"
minutes = 90
+++

## Goal

Add the deployment's two acts the draft needs to jig (draft
`temper/docs/design/draft/playground-reset.md`, sections 4 and 5):
`Act::Cancel`, whole, so a connector can cancel any live task of a
project as the deployment's compound; and `Act::RemoveProject`'s surface
(vocabulary, grant, codec, its outcome, and the core's
`Request::ProjectRemoved` to the root), its decision refused *not offered*
until a later task of this part gives it its body.

Acceptance:

- `people::Act::Cancel { project: u32, task: u64, grace: Option<Duration>, reason: Box<[u8]> }`
  is admitted as every deployment act is (grant, first owner, keyed
  answer) and becomes the deployment's compound: each live delegate of the
  task cancelled (no grace), then the task with the grace, each with the
  reason, answered as `Close` is. Its subject is any live task of the named
  project, whatever its requester and flags; an ended or closing task is
  refused as `Close` refuses one (`refusals::ended`), a task of another
  project or none as unknown.
- `people::Act::RemoveProject { project: u32 }` and `ActKind::{Cancel,
  RemoveProject}` exist, in `ActKind::ALL`; `engine.acts` reads `"cancel"`
  and `"remove_project"`; both acts round-trip through the people codec
  (a new people schema version, as the last act added made one).
- `people::Outcome::ProjectRemoved { project: u32 }` exists and
  round-trips as a retained answer. The act's decision answers
  `Refused(NotOffered)` for now, unsaved, as an ungranted act is, so no
  answer is retained.
- `jig_domain_core::Request::ProjectRemoved { project: u32 }` exists
  beside `ProjectMade`, with the same commit relationship (held until the
  decision commits); nothing sends it yet. Every exhaustive match over
  `Request`, `people::Act`, `people::ActKind` and `people::Outcome` in jig,
  temper and their test kits compiles, the new arms doing nothing
  (`ProjectRemoved` ignored by every root: jig's test kits, the ops
  example, temper's root and service) or, for byte accounting, charging
  as the nearest existing variant does.
- Tests: in `jig-core-world`'s `suite` binary, `deployment_acts.rs`, cases
  for `Cancel` (a change task and a delegate cancelled; ended refused;
  another project's task refused; ungranted refused *not offered*; a
  restart answers from the saved answer), and `RemoveProject` refused
  *not offered*; the codec's round trips in `jig-protocol-store`'s library
  tests; the grant words in `jig-protocol-config`'s `tests/acts.rs`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1).
- `jig/docs/design/domain/people.md`, 5.3 (the deployment's acts);
  `jig/docs/design/domain/tasks.md`, the compound request (section 7).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  sections 3, 4 and 7: the act's path file by file (`RemoveAccount` as
  the worked example), every reader.
- The precedent for a new act and people schema version: `git log
  --oneline -- jig/crates/jig-protocol-store/schema/people-v13.schema`
  and its first commit's `--stat`.

## Owns and may change

- Owns: `jig-domain-people`'s `boundary.rs` (`Act`, `ActKind`,
  `ActKind::ALL`, `Outcome`) and `domain/acts.rs` (kind, admission,
  subject checks, outcome save), with their unit tests;
  `jig-domain-core/src/routing/deployment_acts.rs` (`Cancel` routed
  through a `cancel` beside `close`, sharing its compound construction:
  factor `close`'s body so both call it, `close` keeping its conversation
  and person checks); `jig-domain-core/src/routing/vocabulary.rs`
  (`Request::ProjectRemoved`); `jig-protocol-config/src/acts.rs` and its
  tests; `jig-protocol-store`'s people schema, generated code and
  `codec/people/` translations; `jig/tests/core/tests/suite/deployment_acts.rs`.
- May change, compiler-forced arms only (no other edit): every exhaustive
  match over the four enums named above, in jig's crates, `jig/testing/*`,
  `jig/examples/ops/*`, `jig/tests/*`, and temper's
  (`temper/crates/temper-engine-domain/src/store.rs`, `route.rs`,
  `route/demo.rs`, `decision.rs`, `boundary.rs`, `domain.rs`;
  `temper/crates/temper-service/src/route.rs`, `clients/translation.rs`;
  temper's worlds and `temper/tools/gallery`), as survey S4 sections 4
  and 7 list them. temper's root maps `Request::ProjectRemoved` to
  nothing: a later task (part `remove-forge`) routes it.
- Options: if the compound's continuation (`PersonTaskRoute::Compound`)
  needs the task's conversation flag, carry what it needs and say so; if
  the people schema's generator is a script, run it as its header says and
  name it in the report; if adding `Outcome::ProjectRemoved` to the
  retained-answer schema would need a second schema change, make both in
  the one new version.

## Steps

1. Vocabulary in `jig-domain-people`, admission (`domain/acts.rs`: `kind`,
   subject validation, `save`), then the grant words.
2. `deployment_acts::route`: `Cancel` → `cancel(core, work, request,
   project, task, grace, reason)` sharing the compound with `close`;
   `RemoveProject` → refused *not offered* (`refused(work, request,
   people::Refusal::NotOffered)` or the unsaved path ungranted acts take:
   read `domain/acts.rs` to pick the one that leaves no retained answer).
3. `Request::ProjectRemoved` and every forced arm.
4. Codec: new people schema version with `ActCancel`, `ActRemoveProject`
   and the outcome; translations; round-trip cases registered as the
   others are (`src/tests/codec/cases.rs`).
5. Tests above; narrow runs, e.g. `heavy cargo nextest run -p
   jig-core-world --test suite -E 'test(/deployment_acts/)'`, `heavy cargo
   nextest run -p jig-protocol-store --lib`, `heavy cargo nextest run -p
   jig-protocol-config --lib`.
6. `cargo fmt`; `heavy cargo clippy -p jig-domain-people -p
   jig-domain-core -p jig-protocol-config -p jig-protocol-store
   --all-targets -- -D warnings`; then `heavy cargo check --tests` for
   each temper and jig package whose arms you touched (name each with
   `-p`); the focused tests of `jig-domain-people`, `jig-domain-core`,
   `jig-people-world`, `jig-core-world`, `temper-engine-domain`. Commit and
   push.

## Report

As `notes/common.md` says; name the schema version, the generator run,
and every file outside the owned list you edited for forced arms.
