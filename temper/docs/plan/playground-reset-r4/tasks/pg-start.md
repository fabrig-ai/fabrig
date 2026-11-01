+++
id = "pg-start"
part = "playground"
kind = "feature"
needs = ["pg-config", "pg-wire", "fr-make-forge"]
files = [
  "temper/crates/temper-engine-domain-forge/src/playground.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/playground.rs",
  "temper/crates/temper-engine-domain/src/route/playground.rs",
  "temper/tests/engine/domain/tests/suite/playground.rs",
]
effort = "high"
minutes = 70
+++

## Goal

`StartPlayground`'s body (draft
`temper/docs/design/draft/playground-reset.md`, 6.1): the `Playground`
row, running, naming what it makes; the playground's fake forge's
record, made through `make::make` (`fr-make-forge`) from
`DemoConfig.forge` (`pg-config`), or the kept playground record when
one is seeded (D1); the box asked for its group; the playground's
project, made; the scripted account, the deployment's, its uses naming
the playground's project alone (jig's `domain/accounts.md`, 4.2); and,
once the fake forge connects, `playground/widget` adopted into that
project. Answered `running` once adopted. The playground is offered
while no real forge is kept, and refused *not offered* once one is
(6.3). `StartDemo` and the demo's end stay as they are until
`pg-demo-out-engine`.

Decided here (the playground designer's, `notes/progress-playground.md`,
Decisions): each act is the person's own request, under their sign-in
and a key derived from the start's, as the demo's start makes them
today (`temper-engine-domain/src/route/demo.rs`, its module comment);
the project is made at each start, named `Playground`, policy 1
(`temper/docs/design/protocol/setup.md`, 8.4: the project the start
makes, "the stock policy 1 unless changed").

Acceptance:

- the connector's `StartPlayground`, with no `DemoConfig`: *not
  offered*; with a kept `Forge` record that is neither `playground` nor
  `removing`: *not offered*; with the `Playground` row `Resetting`:
  *not offered*;
- with no `Playground` row: in its decision, saves `Playground {
  state: Running, forge: None, project: None, account: None }`, then
  takes the kept record that is `playground` and not `removing`, or
  makes one with `make::make(d, env, config.forge.clone(), out)` (no
  `config.forge`, or a refused make: `Failed(Group { why })` naming the
  refusal, the row erased again), and saves the row with `forge:
  Some(n)`; under the box, with the forge not connected, `SetUpGroup {
  forge: n, request }` as the demo's start asks it; then
  `Request::PlaygroundStart { request }` to the root;
- with the row `Running`: the same start goes on from the first step
  not done (the row's `None`s), making nothing twice; one start at a
  time, the others answered with it, as the demo's starts are;
- the root's steps, each the person's request: `people::Ask::MakeProject
  { name: "Playground", policy: 1 }` (refused: `Failed(Project)`);
  the account as the demo's start adds it (`AddKey` carried, holder
  `Deployment`; an existing deployment account of the scripted
  provider reused), then `people::AccountChange::SetUses { account,
  uses: Some([Use::Project(project)]) }` (refused: `Failed(Account)`);
  each step's result saved in the row through
  `forge::Event::PlaygroundMade`; once connected (within
  `DemoConfig.deadline`, else `Failed(NotConnected)`), the adoption of
  `DemoConfig.adoption` with its `forge` and `project` the row's
  (refused: `Failed(Adoption)`); then `Running`;
- the scripted account serves the playground's project and no other:
  a task of another project finds it in no account's uses (the
  world's test);
- the tests below pass, and every existing test of the packages
  changed.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 1,
  D1 and D5; 2, the `Playground` row; 5).
- `temper/docs/design/draft/playground-reset.md`, 6.1 and 6.3.
- `temper/docs/design/protocol/setup.md`, 8.4 (the demo's start, which
  this one follows step by step).
- `jig/docs/design/domain/accounts.md`, 4.2 (uses).
- The demo's start, the model: `temper-engine-domain-forge/src/demo.rs`
  (`start`, `answered`, `restart`) and
  `temper-engine-domain/src/route/demo.rs` (`start`, `begin`,
  `progress`, `reply`, `adopted`, `connection`, `done`).
- `temper/docs/plan/playground-reset-r4/tasks/fr-make-forge.md`
  (`make::make`, `Refusal`, `kind_busy`), `tasks/pg-config.md`
  (`DemoConfig.forge`), `tasks/store-families.md` (`Playground`,
  `Domain::playground()`), `tasks/forge-asks.md` (`Ask::StartPlayground`,
  its stub in `asks.rs`), `tasks/pg-wire.md` (`StartFailure::Project`).

## Owns and may change

Owned: the `files` above, all new.

Neighbour files, edits allowed:

- `temper-engine-domain-forge`: `boundary.rs` (appended:
  `Request::PlaygroundStart { request: Token }`, `Event::PlaygroundMade
  { project: Option<u32>, account: Option<u32> }`,
  `Event::PlaygroundAnswered { answer: StartAnswer }`, each with a doc
  line); `asks.rs` (the `StartPlayground` arm calls
  `playground::start`); `domain.rs` (one field `pub(crate) playground_start:
  crate::playground::State` and its `new`, the two events' dispatch
  arms, and `application_answer` reading the start's answers); `lib.rs`
  (`mod playground;`); `tests.rs` (`mod playground;`).
- `temper-engine-domain`: `route.rs` (`pub(crate) mod playground;`,
  the connection arm beside `demo::connection`, the progress call
  beside `demo::progress`), `route/forge_route.rs` (the
  `PlaygroundStart` arm beside `DemoStart`'s), `route/applications.rs`
  (the `StartPlayground` context captured as `StartDemo`'s is),
  `route/held.rs` (the playground route's replies beside the demo's),
  `domain.rs` (the route's fields beside `demo_start`'s),
  `deadline.rs` (the start's deadline beside the demo's),
  `admission.rs` (the new events' arm).
- `temper/tests/engine/domain/tests/suite/main.rs`: `mod playground;`.
- Compiler-forced arms for the new variants anywhere in temper.

Pre-approved:

- if the root's reply token scheme (`REPLY = 0` for the demo) leaves no
  room, the playground route takes another fixed token (`u64::MAX - 1`
  or the next free), saying which;
- if a step's reply cannot tell its account (`AccountKeyKept` names a
  source), find the deployment's account of the scripted provider as
  `route/demo.rs`' `list_end_acts` finds it;
- if `make::make` needs the decision's `env`, take it through the
  dispatch as the asks' other bodies do.

## Steps

1. **Connector** (`playground.rs`, module comment citing draft 6.1 and
   contracts 2):

   ```rust
   #[derive(Debug)]
   pub(crate) struct State { /* pending starts and their answers, as demo's */ }
   pub(crate) fn start(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, out: &mut Queue<Request>);
   pub(crate) fn made(d: &mut Domain, project: Option<u32>, account: Option<u32>, out: &mut Queue<Request>);
   pub(crate) fn answered(d: &mut Domain, answer: StartAnswer, out: &mut Queue<Request>);
   /// Whether a kept forge other than the playground's stands (draft 6.3).
   pub(crate) fn real_forge_kept(d: &Domain) -> bool;
   ```

   `made` saves the row with the step's field set; `answered` answers
   every pending start (`ApplicationDone`) as `demo::answered` does.
2. **Root** (`route/playground.rs`): `Context { sign_in, key, carried }`,
   `Start { request, phase, until }`, `Phase::{Waiting, Project,
   Account, Uses, Connecting, Adoption}`; `start`, `progress`, `reply`,
   `adopted`, `connection`, `done`, as the demo's route, with the steps
   of the acceptance; keys `key(original, purpose)` over `"temper
   playground start\0"`. A person's ask goes as
   `Work::People(people::Event::Ask { reply_to, sign_in, key, ask,
   models: Box::new([]) })`; the carried key as the demo's `core::Event::Key`;
   the adoption through `applications::start`.
3. **Tests:**
   - connector unit tests (`tests/playground.rs`, the crate's fixtures):
     `no_demo_config_is_not_offered`,
     `a_kept_real_forge_withdraws_the_playground`,
     `a_resetting_playground_is_not_offered`,
     `a_first_start_saves_the_row_and_makes_its_forge` (saves in order:
     `Playground` running, `Forge`, `Forges`, `Playground` with the
     forge; `ForgeKept`; under the box `SetUpGroup`; `PlaygroundStart`),
     `a_seeded_playground_forge_is_taken_not_made`,
     `a_refused_make_fails_the_start_and_leaves_no_row`,
     `made_steps_are_saved_in_the_row`.
   - engine world (`temper-engine-domain-world`,
     `tests/suite/playground.rs`, its config built as `suite/demo.rs`
     builds it, with `DemoConfig.forge` a fake record of number 0 and
     no configured forge 1; read only of `demo.rs`):
     `start_playground_makes_its_project_account_and_adoption` (one
     start: the project `Playground` exists, the deployment account of
     provider 1 has uses `[Project(p)]`, `widget` adopted into `p`, the
     answer `Running`, the row `{Running, Some(n), Some(p), Some(a)}`);
     `the_scripted_account_serves_no_other_project` (a task of the
     seeded project 1 finds no account of the provider serving it, as
     the core's coverage reads it);
     `a_second_start_makes_nothing_again`;
     `a_refused_project_fails_the_start_naming_it` (a project
     `Playground` made first by the person: `Failed(Project)`);
     `a_kept_real_forge_refuses_the_start` (a world with a configured
     real forge: `NotOffered`).
   - Narrow runs: `-p temper-engine-domain-forge --lib -E
     'test(playground::)'`; `-p temper-engine-domain-world --test suite
     -E 'test(playground::)'`.
4. `cargo fmt`; clippy on `temper-engine-domain-forge`,
   `temper-engine-domain`, `temper-engine-domain-world`; focused tests of
   the three and of `temper-service`; `heavy cargo check --tests` for
   `temper`, `temper-system-world`, `temper-engine-forge-world`,
   `temper-engine-protocol-forge-world`, `temper-one-process-world`,
   `temper-client-world`, one at a time. Commit, push.

## Report

fab's sections; `## Interfaces`: the new variants, `playground::State`,
`start`, `made`, `answered`, `real_forge_kept`, the root route's public
items; the reply token taken; the grants the start needs in
`engine.acts` (none, if every act is the person's request: say so for
`bx-init`).
