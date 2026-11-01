+++
id = "fr-surface"
part = "forge-records"
kind = "interface"
needs = ["forge-asks", "jig-acts", "fr-limits"]
files = [
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/kept.rs",
  "temper/crates/temper-engine-domain-forge/src/release.rs",
  "temper/crates/temper-engine-domain-forge/src/removal.rs",
  "temper/crates/temper-engine-domain-forge/src/make.rs",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/admission.rs",
  "temper/crates/temper-engine-domain/src/restart.rs",
  "temper/crates/temper-engine-domain/src/route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/kept.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/forge.rs",
  "temper/crates/temper-service/src/forge/table.rs",
]
effort = "high"
minutes = 50
+++

## Goal

The vocabulary the forge-records and forge-removal bodies build on,
wired end to end with stub bodies, so that those bodies land in their
own new module files and run in parallel: the connector tells the
service which forges it keeps and which it removed; it sends kept acts
and hears their answers; it hears jig's `ReleaseProject` and answers
`ProjectReleased`; the box link's `removed` reaches it. No behaviour
changes: every stub does nothing, or answers as the code does today.

Acceptance:

- The new `Request` and `Event` variants below exist in
  `temper-engine-domain-forge`, the root and the service route each to
  a stub in a new module file named below, and every package that
  matches them builds its tests.
- The stubs: `kept::send_all` and `kept::answered` and
  `kept::box_answered` do nothing; `release::release` answers
  `Request::ProjectReleased` at once (what temper's root does today
  since `jig-acts`, now through the connector); `removal` and `make`
  hold only their module comment and `pub(crate) struct State` (empty)
  for the bodies to fill; the service's `forge::table::kept` and
  `removed` do nothing.
- The root sends `forge::Event::SendKept` where it sends
  `SendDemoActs` today (once the core is ready at each start).
- jig's `connector::Ask::ReleaseProject` reaches the connector as
  `Event::ReleaseProject`, and its `ProjectReleased` goes back to the
  core as `jig-acts` sends it today; the test `jig-acts` wrote for it
  still passes.
- Existing tests pass unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 1,
  D3 and D4, 2's `KeptAct`, 3's pipe lines, 4's `ReleaseProject`).
- The draft, `temper/docs/design/draft/playground-reset.md`, section 4.
- The demo's kept acts as they stand, the model for `SendKept` and
  `KeptAnswered`: `temper-engine-domain-forge/src/demo.rs` (`acts`,
  `send_acts`, `act_answered`), `temper-engine-domain/src/route/demo.rs`
  (`list_end_acts`, `send_end_act`, `end_acted`),
  `temper-engine-domain/src/restart.rs` (`SendDemoActs`).
- `tasks/jig-acts.md` and its merged report (the root's
  `ReleaseProject` arm in `route/forge_route.rs`).
- `notes/progress-box.md`'s contract problem: a `BoxRemove` answered
  `failed` stays kept.

## Owns and may change

Owned: the files in the header. `kept.rs`, `release.rs`, `removal.rs`,
`make.rs` (connector), `route/kept.rs` (root) and `forge/table.rs`
(service) are new; this task writes only their stubs and module
comments, and later tasks own them (`kept-acts`, `rm-release-project`,
`rm-remove-forge`, `fr-make-forge`, `fr-forge-added`).

Neighbours: every `match` on the changed enums anywhere in temper (the
engine worlds, `temper/tests/**`, service tests): the new arm only,
doing what its stub does.

Pre-approved options:

- If the root's restart sends `SendDemoActs` from more than one place,
  send `SendKept` beside each.
- If `Request::ForgeKept`'s `Box<Forge>` makes a `Request` queue's
  element size check fail, keep the box and say so; never inline it.

## Steps

1. **The connector's vocabulary** (`temper-engine-domain-forge/src/boundary.rs`),
   names binding, each with its doc line:

   ```rust
   // Request (connector to root):
   /// The service holds this forge from now on (draft 3): at each start
   /// for every kept record, and when a forge is made.
   ForgeKept { forge: Box<crate::Forge> },
   /// The forge is removed: the service drops it at once (draft 4).
   ForgeRemoved { forge: u16 },
   /// Send one kept act (D4): a jig deployment act, or the box's remove.
   SendKept(Kept),
   /// The connector's rows of the project are erased (contracts, 4).
   ProjectReleased { request: Token, project: u32 },

   // Event (root to connector):
   /// The core is ready at a start: send every kept act again (D4).
   SendKept,
   /// A kept jig act's saved answer: `keep` false erases its row (D4's rule).
   KeptAnswered { key: [u8; 16], keep: bool },
   /// The box's answer to `remove FORGE`: `removed` true for `removed`,
   /// false for a `failed` line, which keeps the act (contracts, 3).
   BoxRemoved { forge: u16, removed: bool },
   /// jig's `ReleaseProject` (contracts, 4).
   ReleaseProject { request: Token, project: u32 },
   ```

   `domain.rs`'s `step` sends `SendKept` to `kept::send_all`,
   `KeptAnswered` to `kept::answered`, `BoxRemoved` to
   `kept::box_answered`, `ReleaseProject` to `release::release`.
   `Domain` gains `pub(crate) removal: crate::removal::State` and
   `pub(crate) make: crate::make::State` (empty structs with
   `new(l: &Limits)`), so their tasks add fields without touching
   `domain.rs`. `lib.rs`: the four `mod` lines.

2. **The root** (`temper-engine-domain`). `boundary.rs`'s `Request`
   gains `ForgeKept { forge: Box<forge::Forge> }`, `ForgeRemoved { forge:
   u16 }`, `BoxRemove { forge: u16 }` (to the box link), each documented.
   `route/forge_route.rs`'s output translation: `ForgeKept` and
   `ForgeRemoved` to the root's `Request` of the same name (through
   `super::now`, as `SetUp` goes); `SendKept(row)` to
   `route::kept::send(domain, row)`; `ProjectReleased` to the core as
   `jig-acts`' arm answers it today. `route/kept.rs`:
   `pub(crate) fn send(domain: &mut Domain, row: forge::Kept)` (stub:
   nothing) and `pub(crate) fn acted(domain, env, decision, to, outcome)`
   (stub, unreachable until `kept-acts` sends acts: `unreachable!` with
   the reason). The `ReleaseProject` arm `jig-acts` wrote becomes
   `Work::Forge(forge::Event::ReleaseProject { request, project })`.
   `restart.rs`: `Work::Forge(forge::Event::SendKept)` after
   `SendDemoActs`. `admission.rs`: the new events beside
   `SendDemoActs`'s arm.

3. **The service** (`temper-service`). `route.rs`: `ForgeKept` to
   `crate::forge::table::kept(engine, *forge)`, `ForgeRemoved` to
   `crate::forge::table::removed(engine, forge)`, `BoxRemove` to
   `crate::forge::table::box_remove(engine, forge)`; all three stubs in
   the new `src/forge/table.rs` (`pub(crate) mod table;` in `forge.rs`).

4. **Tests.** `temper-engine-domain-forge`: in a new
   `src/tests/surface.rs`: `release_answers_project_released_at_once`,
   `kept_events_change_nothing_yet`. `temper-engine-domain`: in
   `src/route/forge_route/tests.rs`,
   `release_project_reaches_the_connector_and_is_answered`. Narrow runs
   by those names.

5. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-service --all-targets -- -D warnings`;
   their focused tests; `heavy cargo check --tests` for
   `temper`, `temper-engine-domain-world`, `temper-engine-forge-world`,
   `temper-engine-protocol-forge-world`, `temper-system-world`,
   `temper-one-process-world`, `temper-conformance-world`,
   `temper-client-world`, `temper-real-loop-world`. Commit and push.

## Report

fab's sections, and `## Interfaces`: every variant and stub function
added, with its path and signature, so `kept-acts`, `rm-*` and `fr-*`
can be checked against it.
