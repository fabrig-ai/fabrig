+++
id = "pg-start"
part = "playground"
kind = "feature"
needs = ["rf-domain", "rf-root", "pg-jig-uses"]
files = ["temper/crates/temper-engine-domain-forge/src/**", "temper/crates/temper-engine-domain/src/**", "temper/crates/temper-protocol-store/**", "temper/tests/engine/domain/**", "temper/tests/codecs/tests/suite/fuzzy/store.rs"]
effort = "high"
minutes = 120
+++

## Goal

The playground made when chosen, on the engine's side (draft
`temper/docs/design/draft/playground-reset.md`, 6.1 and 6.3's offer):
`StartPlayground` keeps a `Playground` row naming what it makes, makes
the fake forge's record by `make_forge(NewForge::Playground)`, asks the
box for its group, makes the playground's project, the scripted account
the deployment's with its uses naming that project alone, and once the
forge connects adopts `playground/widget` into it, each by the
deployment's keyed acts. The demo's end goes with it: no `Ended` state,
no end on another forge's connect, no "Ended with the playground"
close. The reset is `pg-reset`'s.

Acceptance:

- `temper_engine_domain_forge`: `Playground` and `PlaygroundState` as
  `notes/playground.md`, section 1, replace `Demo` and `DemoState`
  (`Stored::Playground`, `Key::Playground`; codec record `Playground`
  family 17 in `temper-protocol-store`, schema, generated code, goldens,
  round trips). `DemoConfig` becomes `PlaygroundConfig { provider: u32,
  owner: Box<[u8]>, name: Box<[u8]>, landing: Box<[u8]>, policy: u32,
  deadline: Duration }` (`configure_playground_config`, beside
  `fr-domain`'s `configure_playground(template)`); `Ask::StartDemo` and
  `Answered::StartDemo` become `Ask::StartPlayground` and
  `Answered::StartPlayground`; `Ask::EndPlayground` exists, answered
  `StartAnswer::NotOffered` until `pg-reset` (a comment says so).
  `KeptAct::Close` and its codec form go.
- The offer (`notes/playground.md`, section 3): a start is answered *not
  offered* with no `PlaygroundConfig` or template, while any record whose
  mode is not `Mode::Fake` is kept (removing or not), or while the row
  reads `Resetting`.
- The start, in the connector's decision: with no row,
  `make_forge(NewForge::Playground, out)` and the row saved `Running`
  with `maker` (the start's admitted person, which the root passes in the
  ask's context) and `forge: Some(n)`; `SetUpGroup { forge: n }` under
  the box as today; then the root's steps. The steps (`notes/playground.md`,
  sections 2 and 3), in the root (`temper-engine-domain/src/route/demo.rs`
  becomes `route/playground.rs`), each the deployment's
  `core::Event::Act` with connector `config.forge_connector` and the
  note's key, never the person's admission: `MakeProject { maker, name:
  "Playground", policy }`, `AddKey { provider, join: None, digest }` (the
  key carried as the start carries it today), `SetUses { account, uses:
  Some([Use::Project(project)]) }`, and once the forge is connected
  `Adopt { project, adoption }` with the adoption built from the config
  (home, prefix `temper/`, `Role::Owned`, CI on, no checks, as today's
  `settings::Demo::domain` builds it). Each answer's number is told to the
  connector (`forge::Event::PlaygroundMade { project | account }`), which
  saves it into the row in that decision. A step whose field is already
  `Some` is skipped (a repeated start finishes what is not done, as
  today's). The start answers `Running` once adopted; the failures and the
  deadline as today's (`Account`, `Adoption`, `NotConnected`, `Group`),
  a refused `MakeProject` answering `Failed(Adoption)` and a refused
  `SetUses` `Failed(Account)` (no new failure, so no wire change).
- Accessors the service reads (names binding, for `pg-svc`): on the
  forge connector's `Domain`, `playground(&self) -> Option<&Playground>`,
  `playground_offered(&self) -> bool` (the offer above) and
  `playground_config(&self) -> Option<&PlaygroundConfig>`; the root's
  `Domain` exposes the same three as it exposes `demo_state` and `demo`
  today.
- What goes: `demo::connected` (the end on another forge's connect),
  `ListDemoActs`/`DemoActs`, `list_end_acts`, the end's `Close`,
  `DemoState::Ended` and every gate reading it (`forge_ended` reads only
  `rf-domain`'s removal now), `retire_ended`'s demo guard, the root's
  `demo_*` names (`demo_start`, `demo_contexts`, `demo_reply`, ...,
  renamed `playground_*`).
- At a start (`restart`), a `Running` row asks its group again as today
  (`pg-reset` turns it into a reset).
- Tests, in `temper-engine-domain-world`'s `suite` binary,
  `tests/suite/demo.rs` becomes `tests/suite/playground.rs`:
  `a_start_makes_the_forge_record_the_project_the_account_and_its_uses_then_adopts`,
  `a_start_saves_each_number_into_the_row_in_the_decision_that_hears_it`,
  `a_repeated_start_skips_what_is_made_and_answers_running`,
  `a_start_is_not_offered_once_a_real_forge_is_kept`,
  `a_start_fails_when_the_project_or_account_is_refused_or_the_key_is_missing`,
  `a_start_waits_for_connection_until_the_deadline`,
  `another_forge_connecting_ends_nothing`, and today's group-failure and
  busy-box cases moved; the end's cases deleted. The forge crate's
  library tests (`src/tests/demo.rs` → `src/tests/playground.rs`) as
  their counterparts; the store codec's `Playground` round trip and key
  cases (`src/tests/records.rs`, `keys.rs`). The world's configuration
  grants connector 1 `make_project`, `add_key`, `set_uses`, `adopt`,
  `remove_project`, `remove_account`, `cancel`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (sections 2
  and 3), `temper/docs/plan/playground-reset-r5/notes/playground.md`
  (sections 1 to 3, 7 and 8: binding),
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 1 and 2: `make_forge`, `NewForge::Playground`,
  `configure_playground`), `temper/docs/plan/playground-reset-r5/notes/remove-forge.md`
  (sections 2 to 4: `forge_ended`, `Kept`).
- The draft, 6.1 and 6.3. `temper/docs/design/protocol/setup.md`, 8.4
  (today's demo start, which this replaces); jig's
  `jig/docs/design/domain/people.md`, 5.3 (the deployment's acts).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  sections 1 and 2 (every reader, file:line) and section 5's forge-lib,
  engine-domain-world and protocol-store lines.

## Owns and may change

- Owns: `temper-engine-domain-forge`, `temper-engine-domain`,
  `temper-protocol-store`, the engine-domain world
  (`temper/tests/engine/domain`), the codecs world's `fuzzy/store.rs`.
- May change, call sites only (the renamed types, the new events and
  asks, no new behaviour): `temper-service` (`engine.rs`'s demo mirror
  reads the `Playground` row; `family.rs` routes wire `start_demo` and
  `start_playground` to `Ask::StartPlayground`; the read answers from
  the row; `settings.rs`'s `Demo::domain` builds `PlaygroundConfig`; the
  service's tests' fixtures), the other engine worlds' exhaustive arms
  (`temper/tests/engine/forge`, `protocol-forge`, `system`,
  `conformance`, `one-process`, `real-loop`), `temper/tools/gallery`.
  The service's real bridge, the template's grants and
  `application.playground` are `pg-svc`'s.
- Options: if `Act::MakeProject`'s maker must be a deployment owner and
  the start's person is not one, answer `Failed(Project)` and say so; if
  jig answers `Adopt` before the forge's repository is listed, keep
  today's wait for the connection, then adopt; if the playground's
  policy number has no stock policy that allows the scripted model alone,
  use the configured project policy today's demo project has and say so
  under `## Design` (the isolation then rests on the uses alone).

## Steps

1. The row and its codec; the config; the asks.
2. The connector's start and offer; the root's steps; what goes.
3. Forge-lib tests and the world's `playground.rs`; narrow runs: `heavy
   cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/playground/)'`, `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/playground/)'`,
   `heavy cargo nextest run -p temper-protocol-store --lib`.
4. Call sites in the service and the worlds (`heavy cargo check -p PKG
   --tests` each).
5. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-protocol-store -p
   temper-engine-domain-world -p temper-service --all-targets -- -D
   warnings`; the focused tests of those, `temper-codecs-world`,
   `temper-engine-forge-world`, `temper-system-world`. Old-forms from
   `temper/`. Commit and push.

## Report

As `notes/common.md` says; the policy you used for the playground's
project and why.
