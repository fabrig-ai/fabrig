+++
id = "sv-playground"
part = "pg"
kind = "feature"
needs = ["sv-removal", "fe-kept-root", "fe-records", "cl-wire"]
files = ["temper/crates/temper-service/**"]
effort = "high"
minutes = 70
+++

## Goal

serve's side of the playground on records (`notes/contracts.md`, T6, T8;
`temper/docs/plan/playground-reset-r6/notes/cl-wire.md`, 3): the client's
`Read::playground` answers from the engine's `Playground` row, the playground's start
still carries the scripted key, and serve's gates read the engine's ended forges (any
`Removing` record, or none) instead of the demo's mirrored state.

Acceptance:
- `Read::playground` answers `w::PlaygroundAnswer` with: `state` from
  `engine.domain.playground()` (fe-kept-root's `root::Domain::playground() ->
  Option<forge::Playground>`): `Running` → `running`, `Resetting(_)` → `resetting`, none →
  `not_started`; `project` and `forge` from the row (`project.unwrap_or(0)`, `forge`; 0 and
  0 with no row); `provider`, `owner`, `name`, `cues` from `settings::Demo` as today; no
  record while the document has no `application.demo` (as today). The placeholder `cl-wire`
  left (the demo's state, `Ended` read as `resetting`) goes.
- serve keeps the row beside the demo's settings where it mirrors the demo's state today
  (`route.rs` and `engine.rs`: `demo.state = engine.domain.demo_state()`): a new
  `settings::Demo::playground: Option<forge::Playground>` refreshed at the same two places,
  which `family::to_root` reads.
- `StartPlayground` still parks while the demo's key file is read and carries it
  (`clients.rs`); for T4's demo shape (`settings::Demo.forge == 0`), the read's `forge` is
  0 and nothing else changes. A test proves the carried key reaches the root's
  `Application` for T4's shape.
- `crate::forge::ended(engine, forge)`: for a forge the document configures (the
  transition's, until `sv-startup-refuse`), today's rule (the demo's mirrored state); for
  every other number, `engine.domain.forge_ended(forge)` (true for a `Removing` record, no
  record, or the demo's ended forge, since `fe-records`). So the rule holds whether or not
  the document's forges were seeded as records. Its 27 callers are unchanged.
  `sv-startup-refuse` drops the document's branch.
- Every existing temper-service test passes (those that read the demo's ended forge
  through the mirror now read the domain; adjust a fixture only where it set the mirror by
  hand, and list each).

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/cl-wire.md`, 3 and 4.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T5 (`forge_ended`), T6, T8.
- `temper/docs/plan/playground-reset-r6/notes/fe-surface.md`, 4 (`Playground`) and
  `temper/docs/plan/playground-reset-r6/tasks/fe-kept-root.md` (the accessor), `fe-records.md`
  (`forge_ended`).
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 8.4 (the demo's read and key).

## Owns and may change

- Owns: `temper/crates/temper-service/` (`src/family.rs`, `src/settings.rs`,
  `src/route.rs`, `src/engine.rs`, `src/forge.rs`, `src/clients.rs`, their tests).
- Fixtures building `settings::Demo` literally gain the field (compiler-forced).
- Pre-approved option: if `root::Domain::forge_ended` is not public, use the accessor
  fe's task gives (rg `fn forge_ended` in `temper-engine-domain`), and say so.

## Steps

1. `src/settings.rs`: `Demo::playground`. `src/route.rs`, `src/engine.rs`: refresh it.
2. `src/family.rs`: `Read::playground`'s answer from the row.
3. `src/forge.rs::ended`: the domain's.
4. Tests in `src/tests/family.rs` and `src/clients/demo_tests.rs`:
   `the_playground_read_answers_not_started_with_no_row`,
   `the_playground_read_answers_running_with_its_project_and_forge`,
   `the_playground_read_answers_resetting_at_each_step`,
   `a_playground_start_of_t4s_shape_carries_the_key`,
   and in `src/tests/forge.rs`: `a_removing_records_forge_is_ended_for_every_gate` (a call
   for it answers `NotConnected` at once, as for the demo's ended forge).
5. Narrow runs, one at a time:
   `heavy cargo nextest run -p temper-service --lib -E 'test(/playground|removing_records_forge/)'`
   (5 tests); then `heavy cargo nextest run -p temper-service`; `heavy cargo nextest run -p
   temper-client-world`; `heavy cargo nextest run -p temper-one-process-world` and `-p
   temper-real-loop-world` (their forges are records by now, or still the document's: both
   must pass); `heavy cargo check --workspace --tests`.
6. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets`; old forms. Commit and
   push.

## Report

- Every fixture adjusted for the mirror's removal.
