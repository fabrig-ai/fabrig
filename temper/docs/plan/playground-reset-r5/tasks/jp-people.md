+++
id = "jp-people"
part = "jig-remove-project"
kind = "feature"
needs = ["jp-acts", "pg-jig-uses", "jp-row"]
files = ["jig/crates/jig-domain-people/src/**", "jig/crates/jig-protocol-store/schema/people-v*.schema", "jig/crates/jig-protocol-store/src/generated/people_v*.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "jig/crates/jig-protocol-store/src/codec/people/**", "jig/tests/people/tests/suite/project_removal.rs", "jig/tests/people/tests/suite/main.rs"]
effort = "high"
minutes = 80
+++

## Goal

Give jig's people child its part of a project's deletion (draft
`temper/docs/design/draft/playground-reset.md`, section 5): the owners'
ask `Ask::RemoveProject`, and the two events the core's deletion sends,
one when the project closes (its roster and inbox gone, so every party
ask naming it is refused as for an unknown project) and one at its end
(the retained answers naming it erased, so a retry answers as one past
its retention does).

Acceptance:

- `people::Ask::RemoveProject { project: u32 }`: `project(ask)` gives the
  project; admitted for a person holding `Role::Owner` on that project,
  otherwise refused as other owner asks are (`Refusal::Role { held }`);
  routed to the core as `Ask::Rename` is, keyed and answered as every
  ask (`Outcome::ProjectRemoved { project }`, from `jp-acts`, retained as a
  permanent answer). In the core's dispatcher
  (`jig-domain-core/src/routing/people.rs`) its arm calls
  `crate::removal::close::close(core, env, work, &mut out, request, project)`
  (`jp-row`'s entry, whose stub refuses `NotOffered`; use the signature
  `jp-row`'s code has); task `jp-close` gives it its body.
- Its store codec: the people schema's `Ask` gains `AskRemoveProject`,
  changed in place in the newest people version (`jp-acts` and
  `pg-jig-uses` made it; make no new version), translations in
  `codec/people/asks.rs`, a round-trip case registered as the asks'
  others are; `layouts.rs`' people line regenerated
  (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world
  --test drift`).
- `people::Event::ProjectClosed { project: u32 }`: the project's roster
  dropped (`domain.roles`) with `Request::Erase { key: Key::Roles(project) }`,
  and every volatile inbox entry, census result and waiting entry naming
  the project dropped. After it, an ask naming the project is answered
  exactly as one naming a project number never made (assert the two
  outcomes equal in the test), and the people child's documents and
  lists show nothing of it. A project number with no roster is a no-op.
- `people::Event::ProjectErased { project: u32 }`: every retained
  `Stored::Answer` whose ask or outcome names the project and every
  `Stored::Acted` whose act or outcome names it erased (in memory and
  `Request::Erase` of its key), by one exhaustive `names_project(&Ask |
  &Act | &Outcome, project) -> bool` over each variant's project fields
  (a task-only variant names no project). Others stay. Any read
  position still naming a task of the project is not this event's: the
  task's erasure took it.
- Every exhaustive match over `people::Ask` and `people::Event` compiles
  (S4 section 4 lists the `Ask` readers: `jig-domain-core`'s
  `acts/requests.rs`, `routing/party_controls.rs`, `roles.rs`,
  `person_task.rs`; the client worlds and kits; temper's
  `temper-engine-domain/src/store.rs` byte accounting, charging it as
  `Rename` without its name).
- Tests in `jig-people-world`'s `suite` binary, a new
  `project_removal.rs`: an owner's ask admitted and routed, a member's
  refused, a retry answered from the retained answer; `ProjectClosed`
  then an ask naming the project refused as an unknown one and the
  `Roles` row erased; `ProjectErased` erasing the answers and acts naming
  the project and keeping one naming another; a restart after each
  restoring nothing of the project.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1).
- `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`,
  section 2 (steps 1 and 4).
- `jig/docs/design/domain/people.md`, 5.1 (a party's asks), 5.3 (the
  deployment's acts), 6 (retention of answers).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  sections 1 (people's families) and 4 (the `Ask` readers, layer by layer).

## Owns and may change

- Owns: `jig-domain-people` (the ask in `boundary.rs`, its admission in
  `domain.rs`, the two events in a new `src/removed.rs` with their
  dispatch; unit tests in `src/tests/`); the people family's schema (the
  newest version, in place), its generated code, `layouts.rs`' people
  line, `codec/people/`; `jig/tests/people/tests/suite/project_removal.rs`
  (new, its `mod` line in `main.rs`).
- May change: compiler-forced arms only, over `people::Ask` and
  `people::Event`, in `jig-domain-core` (the dispatcher's arm above;
  others ignore or refuse as their neighbours do), `jig/testing/*`,
  `jig/tests/*`, `jig/examples/ops/*`, temper's `temper-engine-domain`
  (`store.rs`) and temper's worlds. Not the client's wire (`jp-wire`).
- Not yours: `jig-domain-core/src/removal/**` (call its entry, never
  edit it), `routing/deployment_acts.rs`.
- Options: if the people child's inbox entries are rebuilt from task rows
  by `Event::Waiting` so that dropping them at close is undone by a later
  `Waiting` of a winding-down task, also ignore `Waiting`/`ChatState` for a
  project with no roster and say so. If an `Answer`'s ask names its
  project only through a task, it names no project here.

## Steps

1. The ask, admission and routing; the forced arms; `heavy cargo check -p
   jig-domain-core --tests`.
2. The codec in place; `heavy cargo nextest run -p jig-protocol-store --lib
   -E 'test(/ask/)'`.
3. `removed.rs`: both events.
4. Tests: `heavy cargo nextest run -p jig-people-world --test suite -E
   'test(/project_removal/)'`.
5. `cargo fmt`; `heavy cargo clippy -p jig-domain-people -p
   jig-domain-core -p jig-protocol-store --all-targets -- -D warnings`;
   `heavy cargo check --tests` for each other package whose arms you
   touched; the focused tests of `jig-domain-people`, `jig-people-world`,
   `jig-protocol-store`, `jig-core-world`. Commit and push.

## Report

As `notes/common.md` says; name the refusal an unknown project's ask
gets (the later tasks assert it), and every file outside the owned list
you edited for forced arms.
