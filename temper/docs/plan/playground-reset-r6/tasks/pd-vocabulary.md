+++
id = "pd-vocabulary"
part = "pd"
kind = "interface"
needs = ["pd-acts"]
files = [
  "jig/crates/jig-domain-core/src/routing/vocabulary.rs",
  "jig/crates/jig-domain-core/src/acts.rs",
  "jig/crates/jig-domain-core/src/reads/vocabulary.rs",
  "jig/crates/jig-domain-core/src/stopping.rs",
  "jig/crates/jig-domain-core/src/routing/mod.rs",
  "jig/crates/jig-domain-core/src/routing/deployment_acts.rs",
  "jig/crates/jig-domain-core/src/routing/key_routes.rs",
  "jig/crates/jig-domain-tasks/src/boundary.rs",
  "jig/crates/jig-domain-tasks/src/domain.rs",
  "jig/crates/jig-domain-accounts/src/boundary.rs",
  "jig/crates/jig-domain-accounts/src/domain.rs",
  "jig/crates/jig-domain-accounts/src/changes.rs",
  "jig/crates/jig-domain-accounts/src/keys.rs",
  "jig/testing/jig-test-domain/src/**",
  "jig/testing/jig-test-store/src/**",
  "jig/testing/jig-test-store-files/src/**",
  "temper/crates/temper-engine-domain/src/route.rs",
  "temper/crates/temper-engine-domain/src/route/asks.rs",
  "temper/crates/temper-engine-domain/src/store.rs",
  "temper/crates/temper-engine-domain/src/decision.rs",
  "temper/crates/temper-service/src/store/route.rs",
]
effort = "medium"
minutes = 100
+++

## Goal

Add the core's and its children's vocabulary for a project's deletion, so that the child,
store, core and client tasks after it each write one crate: the core's ask to a connector and
its answer (contracts J5), the core's transcript drop (J6) routed by every root to the store,
the owner's offer kind (J2), the tasks and accounts children's events, `AddKey`'s uses threaded
to the accounts child (J8), and the two store read scopes the core's erasure pages through.
Bodies stay empty: nothing new is decided here.

Acceptance:
- `jig_domain_core::Ask::ReleaseProject { project: u32 }`,
  `jig_domain_core::Event::ProjectReleased { project: u32, connector: u16 }` and
  `jig_domain_core::Write::DropTranscript { task: u64 }` exist (appended last).
  `ProjectReleased` is not new work, passes the stopping gate, and is routed to a decision
  that changes nothing (`Request::Decided` only).
- temper's root answers `Ask::ReleaseProject` at once with `Event::ProjectReleased { project,
  connector }`, as it answers `ProjectGoal` for a connector without a home (route/asks.rs),
  and routes `Write::DropTranscript` to the store's `store::Write::DropTranscript { task }`;
  jig-test-domain's and the ops example's roots do the same (the ops example may drop the
  write if its store keeps no transcripts; say so).
- `jig_domain_core::acts::Act::RemoveProject`, appended last, not yet offered anywhere.
- `jig_domain_core::reads::Scope` (the store query identity, `reads/vocabulary.rs`) gains
  `ProjectRows { project: u32 }` and `TaskRows { task: u64 }`; no party read admits them; the
  store's `intervals::scope` answers `None` for both until `pd-store-reads`.
- `jig_domain_tasks::Event::Erase { record: Box<TaskRecord> }` and
  `jig_domain_tasks::Event::ForgetProject { project: u32 }`;
  `jig_domain_accounts::Event::ForgetProject { project: u32 }`; each accepted and doing nothing
  (bodies: `pd-tasks-erase`, `pd-accounts-uses`); none is new work.
- `jig_domain_accounts::Change::AddKey` gains `uses: Option<Box<[Use]>>`; the core passes the
  deployment act's `uses` (people `Use` translated to accounts `Use`, as `SetUses` is) and
  `None` for a party's key; `keys::add` takes the value and ignores it (`pd-accounts-uses`).
- jig and temper build; the named tests pass.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J2, J5, J6, J8.
- `jig/docs/design/domain/engine.md`, section 4 (the core's vocabulary).
- `jig/docs/design/protocol/store.md`, 8.5 (a transcript dropped whole).

## Owns and may change

Owned: the `files` above (in each, only what this spec names plus compiler-forced arms).

Reader files, compiler-forced arms only (common.md):
- `jig/crates/jig-domain-core`: other matches on `Ask`, `Event`, `Write`, `acts::Act`,
  `reads::Scope`, `tasks::Event`, `accounts::Event`, `accounts::Change` (e.g. `acts/objects.rs`,
  `acts/standing.rs`, `documents/offers.rs`, `reads.rs`, `routing/tasks.rs`,
  `routing/children.rs`). `acts::Act::RemoveProject` is judged as `Rename` where acts are
  judged, and offered nowhere.
- `jig/crates/jig-protocol-store`: `intervals.rs` and `reads.rs` arms for the new scopes
  (`None` / not served); `src/tests` only if a match there is exhaustive.
- `jig/crates/jig-protocol-engine-clients/src/translate/task.rs`: `act()` maps
  `acts::Act::RemoveProject` to `Err` with its existing problem form (the wire carries it from
  `pd-client-wire`).
- `jig/examples/ops/domain/src/{route.rs,store.rs}`, `jig/testing/jig-test-service/src/**`,
  `jig/tests/client-protocol/src/engine.rs`, and the worlds with exhaustive root `Write`
  matches: `jig/tests/core/src/{watches.rs,observations.rs,application_observations.rs}`,
  `jig/tests/one-process/src/referee.rs`, `jig/tests/channel/src/world/store.rs`,
  `jig/tests/ops/src/conformance/mod.rs` (a dropped transcript changes no row they track).
- temper: `temper/crates/temper-service/src/clients.rs`,
  `temper/crates/temper-service/src/clients/translation.rs`, `temper/tests/client/src/translate.rs`
  (`ProjectReleased` is not the client kit's: panic as its neighbours do),
  `temper/tests/one-process/src/referee/{durable.rs,restart.rs}`,
  `temper/tests/engine/domain/src/{store.rs,escalation_referee.rs,roles_referee.rs}`.
  The report lists each temper file.

Options, pre-approved:
- temper's root `Write` gains `DropTranscript { task: u64 }`. `Write::key()` must give it a key
  no other write shares: default, a new root `Key::Transcript(u64)` used only for coalescing
  (its other arms compiler-forced; a row is never saved under it). If `Write::key()` can
  return no key for it without touching the coalescer's callers, do that instead and say so.
  `owned_bytes` 0; `decision.rs` admits it as an erase is admitted. Same choice in
  jig-test-domain's root `Write`/`Key`.
- `jig-test-store-files`' typed mirror applies a dropped transcript to nothing (transcripts are
  not mirrored rows); if it mirrors transcript turns, remove that task's turns.

## Steps

1. Core vocabulary (`routing/vocabulary.rs`): the three variants, doc comments citing
   `draft playground-reset.md, 5` and contracts J5/J6 in the design's words ("Erase every row
   the connector keeps for this project, then answer `ProjectReleased`"; "the task's
   transcript file and its places, dropped whole"). `new_work`: `ProjectReleased` false.
   `routing/mod.rs`: `ProjectReleased` routes to `Requests::Out([Request::Decided])`.
2. `acts.rs`: `Act::RemoveProject` (doc "Delete the project").
3. `reads/vocabulary.rs`: the two scopes, documented "every ended task row and decided person
   proposal of one project, in key order" and "every row under one task's key prefixes, in
   key order" (`pd-store-reads` defines them precisely).
4. Children: the events in `jig-domain-tasks/src/boundary.rs` and
   `jig-domain-accounts/src/boundary.rs`, arms in their `step` doing nothing;
   `Change::AddKey { provider, join, carried, uses }`; `changes.rs` passes `uses` to
   `keys::add` (new last parameter), unused there.
5. Core threading: `routing/deployment_acts.rs` `route` passes the act's `uses` into the
   accounts change (translate each `people::Use` to `accounts::Use`); `key_routes::admitted`
   builds party `AddKey` changes with `uses: None`.
6. Roots: temper `route/asks.rs` (`ReleaseProject` answered at once), temper `route.rs` and
   `store.rs` (`Write::DropTranscript`), temper-service `store/route.rs`
   (`engine::Write::DropTranscript { task } => store::Write::DropTranscript { task }`);
   jig-test-domain `routes.rs` (`route_core`, `route_ask`) and `lib.rs` (`Write`),
   jig-test-store `route.rs`, jig-test-store-files; the ops example.
7. Tests:
   - jig-test-domain, in its world `jig/tests/core` (new `tests/suite/deletion_vocabulary.rs`,
     registered in `main.rs`): `a_release_ask_is_answered_at_once_by_the_test_root` (route a
     `core::Request::Ask { connector: 1, ask: Ask::ReleaseProject { project: 1 } }` through
     the test root and see `Event::ProjectReleased { project: 1, connector: 1 }` come back),
     `a_dropped_transcript_reaches_the_store` (a decision with `Write::DropTranscript` for a
     task with turns; the store's transcript of that task reads nothing afterwards).
   - temper (`temper/crates/temper-engine-domain/src/tests/`, beside `projection_settled.rs`,
     new `project_released.rs`): `a_release_ask_is_answered_at_once`.
   - Commands, one at a time:
     `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/deletion_vocabulary/)'`,
     `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/project_released/)'`,
     `heavy cargo nextest run -p jig-domain-core --lib`,
     `heavy cargo nextest run -p jig-accounts-world --test suite`,
     `heavy cargo nextest run -p jig-tasks-world --test suite`.
8. `cargo fmt`; `heavy cargo clippy --all-targets` on every changed package, temper's
   included; old forms; commit and push.

## Report

Every file outside `files` you changed, and which option you took for the drop's key.
