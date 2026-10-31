# Where the code stands, against the draft's section 2

`temper/docs/design/draft/playground-reset.md` (the draft) against the
trunk's base, `114f60ff86` (fabrig `main` at `c30cf6a03` plus this run's
inputs), surveyed read-only on 2026-10-31. Earlier work: the draft it
replaces, `playground-apart.md`, is not in the tree; no branch on origin
was used. The branches and the landing of an earlier run of this same
change (`pgkit/*`, and `main` after it) are this replay's own answer and
were deliberately not read. Paths are from the repository root.

## Forges are the document's

- `temper.json` names forges in three joined lists: `application.forges`
  (endpoints, kind), `application.identities` (each forge's git identity)
  and `application.setup` (each forge's setup entry). Parsed by
  `temper-service`'s `startup/application.rs` (`Forge`, `Identity`,
  `parse`, `resolve`, `configured`), checked against the setup by
  `temper-forge-setup`'s `check.rs`, joined by number throughout. Numbers
  are the document's, explicit and checked unique; nothing allocates one at
  runtime.
- `temper init`'s template (`temper/crates/temper-init/template/temper.json`)
  writes forge 1 (the playground's fake, setup entry `demo/forge`), 2
  (GitHub) and 3 (owned Forgejo, `"start": "asked"`), their identities and
  setup entries, `application.demo` (scripts, provider 3, forge 1, project
  1) and `engine.acts` granting `close` and `remove_account`.
- Readers of the forges' configuration span `temper-service` (startup,
  settings, forge connection layer, setup host settings, workers' routing,
  assignment, tokens, records loader, family translation, bundle
  sign-in metas), `temper` (lib's endpoint resolution, setup check,
  box_command, forge reconcile), `temper-forge-setup`, `temper-fake`
  (steps, demo, fake backends' document rewrite), and the engine
  (`temper-engine-domain-forge`'s `Configured` map, built once at
  `Domain::new`).
- skein's components take their forges once: `skein-forge-connection`'s
  `Settings::forges` at `Component::new`, and `skein-forge-setup`'s host
  `Settings::forges` at its construction. Neither can add or remove a
  forge at runtime.
- Limits: `forge.forges` does not exist; `forge.repositories` and
  `forge.client.repositories` are counts already (small profile 2; the
  template overrides to 3); `forge_component.forges` sizes skein's
  component (small 4). `engine.memory_bytes` is a metered budget
  (272c945cc), so memory no longer argues for forges in the document.
- The box (`temper-box`) is given its steps at construction, flattened by
  `layout::flatten`; asked groups (owned `start: asked`, and the
  playground's, unrecorded) start on the pipe's `start N`, stop on `stop
  N`. Records are `box/started/N`. There is no `remove N`, no
  `box/groups/N`, and stopping keeps the record. The box may name only
  `temper-forge-setup`'s steps types (a boundary test).

## The playground is the demo, and it only ends

- The forge connector (`temper-engine-domain-forge`, `demo.rs`) keeps one
  `Demo { forge, state: Running | Ended }` row (store family 17) and
  `DemoAct` rows (family 19) for the end's acts. `StartDemo` writes
  running, asks the box group, then the root (`temper-engine-domain`,
  `route/demo.rs`) adds the scripted account (as the caller's keyed
  `AccountChange::AddKey`) and adopts `playground/widget` into the
  configured project 1. Another configured forge connecting ends it for
  good: `Ended`, the forge ended (filtered out of every list by
  `forge_ended`, calls answered `NotConnected` by `temper-service`), its
  chats closed by jig's `Act::Close` "Ended with the playground", its
  account removed by `Act::RemoveAccount`, each kept as `DemoAct` and sent
  again at each start.
- The client (`temper-client-domain`'s `forge/demo.rs`) judges the demo's
  model and forge halves apart, removes the scripted account itself at the
  model half's end, and draws a two-half banner and a setup strip
  (`temper-client-view`'s `frame.rs`). The welcome's decision
  (`pages/welcome.rs`) is a bool, not empty/unfinished/whole; "Set up
  temper" is a link to `/welcome?at=forge`.
- No account is limited to the playground's project: the scripted
  account is the deployment's with default uses, and the stock policy
  allows every model (`temper-service`'s `stock.rs`).

## A set up writes as it goes and loses its intent

- `SetUp { forge, org, repository, project, playground }` and `Connect {
  forge, how }` name an existing configured forge. The first repository's
  intent is held in memory twice (`First` in the connector, `Intent` in
  the service's setup lifecycle) and lost at a restart (`setup.md`, 6.7).
  The welcome's Default project is made by the client (`MakeProject`) and
  the first repository adopted by the client after connect.
- No test walks a set up cut short.

## Nothing is ever removed

- jig has no `RemoveProject`, no deletion of a project's rows, and no
  whole-task erasure: retention's horizon erasure is "Not yet"
  (`jig/docs/design/protocol/store.status.md`). What exists: `forget` of
  a task forgotten once read (`jig-domain-core/src/forget.rs`, partial:
  not its activity, history, terminals or transcript), the store's
  `Write::DropTranscript { task }`, and the deployment's acts
  (`MakeProject`, `Close`, `AddKey`, `RemoveAccount`, `Adopt`) with keyed,
  retained answers (`jig-domain-people`, `domain/acts.rs`).
- Project rows: `people::Stored::Policy` (name and policy copy) and
  `Roles`; tasks' rows name projects in `EndedByProject`, ledgers'
  `Funder::{Pool, Recurring, Period}`, `PlacedHold`, proposals; notes'
  `Scope::{Project, Goal, Resources}`; accounts' `Use::Project`. Project
  numbers are a high-water mark (`Deployment.projects`), never reused.
- The connector hears no project's end: there is no core-to-connector
  hand-off releasing a project's adoptions (`ForgetAdoption` is a
  rollback), and temper's settings of a project's scope (`Setting` rows,
  kind 2 built, 4 and 5 designed) have no erasure.
- The deployment's compound only cancels; temper already cancels a
  change's task in the deployment's name through `Work::Tasks(Control {
  by: Party::Deployment { project }, .. })` (`landing_route.rs`).

## Tests that pin today's behaviour

- Demo: `temper-engine-domain-forge`'s `src/tests/demo.rs`, the engine
  world's `temper/tests/engine/domain/tests/suite/demo.rs`, the process
  tier's `a_second_forge_ends_the_playground_for_good_closing_its_chat_and_account`
  and `qa_first_run.rs`, the client world's `demo_start.rs` and
  `welcome.rs` (driven by a front that scripts the demo, not the engine),
  client view and domain unit tests, and the journeys
  `temper/docs/design/client/journeys/welcome-playground` and
  `welcome-set-up` (reprinted by `temper-client-journeys`'s test).
- Forges in the document: `temper-service`'s startup tests, the owned
  example `temper/docs/examples/owned.json` and its readers, the process
  tier's support writers that edit `application.forges`.
