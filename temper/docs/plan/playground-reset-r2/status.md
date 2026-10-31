# Where the code stands, against playground-reset.md's section 2

2026-10-31, at this plan's base (`main` at `c30cf6a03` with fab's
AGENTS.md and the replay's inputs). From five read-only surveys of the
tree; earlier runs' branches and the landed result of the run this
replays were deliberately not read.

## Forges are the document's

- `application.forges` is a required, nonempty list with explicit `u16`
  numbers (`temper-service/src/startup/application.rs:355,770`); the
  identities list must match it one to one (`:815`); `application.setup`
  entries are parsed by `temper-forge-setup`'s `Setup::read`
  (`setup.rs:93`); `application.demo` by `temper-fake/src/demo.rs:82` and
  the service's `startup/application/demo.rs:26`.
- `temper init` asks nothing: it writes a fixed template naming forge 1
  (the fake), 2 (GitHub) and 3 (owned Forgejo, asked), identities 1-3,
  the demo, and `engine.acts` granting `close` and `remove_account`
  (`temper-init/template/temper.json`), with five machine ports
  (clients, owned, fake forge, fake LLM, webhooks).
- About forty service modules read the configured forge list (survey b,
  2): startup checks, connections and secrets loading, the setup host,
  workers' identities, request translation, URLs, sign-in metadata. The
  child commands (`temper forge ... DIRS N`, `temper fake forge|llm`)
  reread the document.
- No forge record, number allocator or `forge.forges` limit exists;
  `forge.repositories` and its kin are already deployment counts, but
  several tables are sized by the configured list's length.
- The forge connector's families stop at 24 (`Demo` 17, `DemoAct` 19),
  store version 10 (`temper-protocol-store`).

## The box knows groups from the document

- `temper-box` gets every step at construction (`Steps { always, asked }`
  from `temper-forge-setup::steps` and `temper-fake::steps`); its pipe
  knows `start N` and `stop N`; `box/started/N` are empty records for
  known groups only. No `box/groups/N`, no runtime group, no `remove`.
- Non-asked owned and fake forges run in `always`, with no stoppable
  group. `temper box --fake-backends` exists and assumes forges 2 and 4.

## The set up holds its intent in memory

- `SetUp { forge, org, repository, project, playground }` and `Connect {
  forge, how }` name a configured forge; the first repository's intent
  is `Domain.first` and the setup host's `Local.first`, both volatile;
  a restart loses it (setup.md 6.7). The welcome's Default project and
  its adoption are made by the client.

## The playground ends, never leaves

- `StartDemo` saves `Demo { forge, Running }` and asks the box for the
  demo's group; it adopts into the configured project 1 under the
  caller's sign-in (setup.md 8.4's project making and deployment acts
  are designed, not built).
- Another configured forge's connect ends it for good: `DemoState::Ended`,
  `DemoAct` rows closing its chats ("Ended with the playground") and
  removing its account, kept and resent at start. `forge_ended` (the
  demo's forge, ended) is checked in about forty places across the
  connector and the service; there is no general ended or removed forge.
- The client judges the demo's model and forge halves and removes the
  scripted account itself.

## jig deletes nothing

- Deployment acts: `MakeProject`, `Close`, `AddKey`, `RemoveAccount`,
  `Adopt`. No `RemoveProject`, no general deployment cancel (`Close` is
  a person's conversation only), no party request to delete a project.
- No whole-task erasure: retention's erase (store.md 11.1) is not built;
  `forget` drops an ended task's owner rows only. `Write::DropTranscript`
  exists in `jig-protocol-store` but no decision emits it, and neither
  the core's nor temper's root `Write` carries it.
- No durable intent resumed at start in jig (temper's `DemoAct` is the
  only precedent); no connector notice of a project's end; spend is
  kept in project-scoped ledgers; notes and accounts' `Use::Project`
  name projects too.
- The conformance world (`jig-conformance`, `jig-conformance-world`)
  sweeps cuts after every commit; its referee holds selected facts, not
  every row.

## The client

- temper has no project settings page. The welcome decides only "over
  or not" (no empty/unfinished/whole); Forges lists configured slots;
  "Connect a forge elsewhere" tells the person to edit the document.
- The client wire (family version 11, `client-wire-v1.schema`) has
  `StartDemo`, `DemoAnswer`, `SetUp`/`Connect` by number, no removal.
