+++
id = "pg-start"
part = "playground"
kind = "feature"
needs = ["rm-domain", "fr-service", "bx-serve"]
files = [
  "temper/crates/temper-engine-domain-forge/src/playground.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/playground.rs",
  "temper/crates/temper-engine-domain/src/route/playground.rs",
  "temper/crates/temper-engine-domain/src/tests/playground.rs",
  "temper/tests/engine/domain/tests/suite/playground_start.rs",
]
effort = "high"
minutes = 90
+++

## Goal

"Try the playground" makes the playground as ordinary records (draft
6.1): `StartPlayground` (today `StartDemo`), the deployment's, keeps a
`Playground` row, running, naming what it makes, and makes, in order:
the fake forge's record (the playground's stock definition), its box
group, the playground's project (`application.playground.project`'s
name and policy), the scripted account (the deployment's, its uses the
playground's project alone) and, once the fake forge connects,
`playground/widget` adopted into that project. It is refused *not
offered* while a real forge is kept (6.3). The client's read
`playground` answers from the row. The demo's end on another forge's
connect is not reached any more; its code and the `Demo`/`DemoAct`
families stay until `pg-gone` removes them.

Acceptance:

- `temper-engine-domain-forge`, `src/tests/playground.rs`:
  - `a_start_keeps_the_row_and_makes_the_fake_forge`: on an empty
    store, `StartPlayground` saves `Playground { state: Running, forge:
    Some(n), project: None, account: None }` with `n` the next number,
    a `Forge` record of mode `Fake` with the request's definition, a
    `Request::Made`, under the box a `SetUpGroup` for `n`, then
    `PlaygroundStart`; the parts made later are saved into the row
    (`Event::PlaygroundMade`).
  - `a_start_with_a_real_forge_kept_is_not_offered` (a real record,
    finished or not, removing or not).
  - `a_second_start_while_running_makes_nothing_again`: concurrent and
    later starts coalesce; no second record.
  - `a_start_while_resetting_is_not_offered` (a row `Resetting`).
- `temper-engine-domain`, `src/tests/playground.rs`:
  `the_start_makes_its_project_account_uses_and_adoption_in_order`:
  `people::Ask::MakeProject { name, policy }`, then the key's
  `AccountChange::AddKey`, then `AccountChange::SetUses { account,
  uses: Some([Use::Project(p)]) }`, then, once connected,
  `forge::Ask::Adopt` into `p`, each under the start's sign-in and a key
  derived from the start's; answered `Running`; an existing deployment
  account of the provider is reused (its uses set) and no key added.
- `temper-engine-domain-world`, `tests/suite/playground_start.rs`
  (replacing the start half of `suite/demo.rs`):
  `a_first_start_answers_running_with_its_own_project`,
  `a_start_whose_account_or_adoption_fails_answers_failed_and_keeps_the_row`,
  `a_start_whose_forge_never_connects_answers_not_connected`.
- `temper-service`: the wire `start_playground` reaches the engine as
  `forge::Ask::StartPlayground { new }` composed by fr-'s stock
  (`forge::stock::playground` + `stock::new_forge`); the read
  `playground` answers running from the row (project, forge, provider,
  owner, name, cues) and resetting for a resetting row, none without a
  row (`src/tests/family.rs`, `src/clients/demo_tests.rs` follow); the
  start seed bridge (D2's playground seed, forge 1) is gone.
- Every package depending on the changed crates builds and passes,
  but for the demo-end tests that can no longer configure a demo
  (deleted, each named in the report).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2 (the
  `Playground` row), 4 (`application.playground`) and 5
  (`StartPlayground`, the read `Playground`).
- `temper/docs/design/draft/playground-reset.md`, 6.1 and 6.3.
- `temper/docs/design/protocol/setup.md`, 8.4 (the demo's start: each
  act the person's request under their sign-in, keys derived from the
  start's, the deadline).
- `jig/docs/design/domain/accounts.md`, 4.2 (an account's uses).
- `temper/docs/design/domain/forge.md`, 4.1.
- The code: `temper-engine-domain-forge/src/demo.rs` (`start`,
  `answered`, `restart`, `needs_group`), `temper-engine-domain/src/route/demo.rs`
  (the start half: `Context`, `Start`, `Phase`, `key`, `start`, `begin`,
  `progress`, `reply`, `adopted`, `connection`, `done`),
  `route/applications.rs`, and the service's playground settings,
  `family.rs`'s `StartPlayground` arm and `clients.rs`'s parked starts
  (`demo_read_answer`).

Assumed, from D2's tasks as merged: `forge::NewForge`, `Request::Made`,
root `Request::ForgeMade`, `src/records.rs`'s `make_forge(d, new,
first, out) -> Result<u16, Refusal>`, the service's `forge::stock::
{playground, new_forge}` and `settings::Machine`, the playground seed
bridge (search for it by `stock::playground` in `startup/` and
`forge/registry.rs`), the parsed playground settings (fr-'s rename of
`settings::Demo`). From D3: bx-serve's `SetUpGroup` arm calling
`box_groups::start_group(.., playground, ..)` with the registry's
`setup_member` (a playground record gives `playground: true`: if
bx-serve takes that from the record's mode, nothing to do; else pass it
there, a call site). From rm-domain: `forge_ended`, the record's
`removing`.

## Owns and may change

Owns the new modules and tests in the header.

May change, as the work needs:

- The forge crate: `boundary.rs` (rename `Ask::StartDemo` and
  `Answered::StartDemo` in place to `StartPlayground`; the new config,
  event and request below, placed where the demo's are; `DemoStart`
  renamed `PlaygroundStart`), `domain.rs` (dispatch, fields), `demo.rs`
  (its start half moves to `playground.rs`; its end half stays),
  `lib.rs`, `connect.rs` (its `unreachable!` admissions).
- The root: `boundary.rs` (`Config.demo` gives way to
  `Config.playground: Option<forge::PlaygroundConfig>`;
  `ApplicationAnswer::StartDemo` renamed `StartPlayground`),
  `domain.rs`, `route.rs`, `route/forge_route.rs` (the forge request
  arms of the start), `route/applications.rs`, `route/demo.rs` (its
  start half moves to `route/playground.rs`), `restart.rs`,
  `deadline.rs`, `admission.rs`, `decision.rs`, `store.rs` (the
  `MakeProject` arm that says the demo never makes a project).
- `temper-service`: `settings.rs` (the playground settings' domain
  config), `startup/` (removing the playground seed bridge; reading the
  playground's project name and policy), `family.rs` and `clients.rs`,
  `clients/*.rs` (the start's translation and the read), `engine.rs`
  and `route.rs` (the demo state mirror, read from the row now),
  `forge/registry.rs` and `forge/stock.rs` (call sites only),
  `system.rs` (the shell's handover-read routing), their tests.
- `temper-fake/src/demo.rs` and the binary's `src/setup.rs` only where
  a renamed config type forces it.
- Tests: `temper-engine-domain-forge/src/tests/demo.rs`,
  `temper/tests/engine/domain/tests/suite/demo.rs` (the start half
  moves to `playground_start.rs`; end tests that cannot configure a
  demo are deleted), the root's and service's tests that build a
  `StartDemo` or `Config.demo`. `notes/scope.md` applies.

Runs beside `rm-project` (forge crate `project_removed.rs`, one event
after `ForgetAdoption`, the root's connector-ask arm) and `rm-service`
(service `forge/removal.rs`, `route.rs`'s two removal arms,
`family.rs`'s `RemoveForge` arm, the consumers' `remove`): keep off
those lines.

## Steps

1. **Vocabulary** (forge crate):

   ```rust
   /// The playground's configuration (playground-reset.md, 6.1); the service composes it.
   pub struct PlaygroundConfig {
       pub provider: u32,
       pub deadline: skein_lib::Duration,
       pub project: PlaygroundProject,      // { name: Box<[u8]>, policy: u32 }
       pub repository: PlaygroundRepository, // { host, owner, name, landing }: playground/widget
   }
   Ask::StartPlayground { new: NewForge }
   Answered::StartPlayground(StartAnswer)     // StartAnswer as today
   Event::PlaygroundMade(PlaygroundPart)      // the root made a part: save it into the row
   pub enum PlaygroundPart { Project(u32), Account(u32) }
   Event::PlaygroundAnswered { answer: StartAnswer } // today's DemoAnswered, renamed
   Request::PlaygroundStart { request: Token }       // today's DemoStart, renamed
   impl Domain { pub fn configure_playground(&mut self, c: Option<PlaygroundConfig>); pub fn playground_config(&self) -> Option<&PlaygroundConfig>; }
   ```

   `PlaygroundConfig::within` bounds its bytes as `DemoConfig::within`.
2. **The decision** (`src/playground.rs`): `start(d, env, request,
   token, new, out)`: refuse `NotOffered` with no config, a row
   `Resetting`, or any record whose number is not the row's forge (a
   real forge kept, 6.3); coalesce as today's `demo::start`; with no
   row, `make_forge` (refused *limit*: answer `Failed(Group { why:
   b"limit" })`), save the row naming the forge, ask the group under the
   box, push `PlaygroundStart`. With a row `Running`: ask the group if
   it is down (as `needs_group`), push `PlaygroundStart`. `made(d,
   part, out)` saves the row with the part. `restart` as today's demo's
   (asks a running row's dormant group); `pg-reset` changes it.
   `forge_ended`'s demo clause stays (reads `DemoState`, now never set).
3. **The root** (`src/route/playground.rs`, from route/demo.rs's start
   half, keyed `b"temper playground start\0"`): phases `Waiting,
   Project, Account, Uses, Connecting, Adoption`, each skipped when the
   row already names its part (a retried start after a failure):
   - `Project`: `core::Event` for the sign-in's `people::Ask::
     MakeProject { name, policy }` (the person's request, as the client
     sends it); `ProjectMade { project }` pushes `PlaygroundMade(
     Project(p))`;
   - `Account`: today's `begin` (reuse the deployment's account of the
     provider, else `AddKey` with the carried key); the account's
     number pushes `PlaygroundMade(Account(a))`;
   - `Uses`: `AccountChange::SetUses { account, uses: Some(Box::new(
     [people::Use::Project(p)])) }` under the sign-in;
   - `Connecting`, `Adoption`: as today, into `p`, on the row's forge,
     the adoption built from `PlaygroundConfig.repository` (`home:
     true`, `prefix: b"temper/"`, `role: Owned`, `ci: true`, no checks,
     as `settings::Demo::domain` builds it today).
   Answers as today's: `Running`, `Failed(Account | Adoption |
   NotConnected | Group)`. The row stays on a failure.
4. **The service:** the playground settings give `PlaygroundConfig`
   (the repository's host from the playground's stock definition read
   with the machine, as fr-'s registry reads it); `family.rs` and
   `clients.rs` build `Ask::StartPlayground { new }` with
   `stock::new_forge(stock::playground(machine), machine)`; the seed
   bridge goes; the read answers from `engine.domain`'s
   `forge.playground()` row and the config's cues (running: the row's
   project and forge, the config's provider, repository owner and name,
   cues; resetting; none).
5. **Tests** as the acceptance names. Then, one at a time:
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
   - `heavy cargo nextest run -p temper-engine-domain --lib`;
   - `heavy cargo nextest run -p temper-engine-domain-world`;
   - `heavy cargo nextest run -p temper-service`;
   - `heavy cargo nextest run -p temper-engine-forge-world -p temper-system-world -p temper-client-world -p temper-one-process-world -p temper-fake -p temper`
     (focused; the readers of the start and of the read);
   - `cargo fmt`; `heavy cargo clippy` over every package changed,
     `--all-targets -- -D warnings`;
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
6. Commit (vocabulary and renames; the decision; the root; the service;
   tests) and push.

## Report

The standard report, plus: each test deleted and why; how the account's
number is learned after `AddKey` (the outcome names a source); whether
`SetUses` on a deployment account is admitted for the deployment's
owner as is; the definition's host as read; anything of the demo's end
you could not leave in place.
