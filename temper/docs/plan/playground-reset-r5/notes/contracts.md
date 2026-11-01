# The run's binding shapes (playground-reset-r5)

The lead's calls for what crosses parts, cut from
`temper/docs/design/draft/playground-reset.md` (the draft) and the
surveys under `temper/docs/plan/playground-reset-r5/notes/surveys/`. A
part's own note (`notes/<part>.md`) may add shapes inside its part; it
never changes one here. Names are Rust paths at the trunk's tip; a field
list marked *at least* may grow inside its owner's task, never shrink.

## 1. jig: the deployment's new acts (part `jig-remove-project`)

In `jig-domain-people` (`boundary.rs`, beside `Act::Close`), granted by
`engine.acts` as every deployment act is (jig's `domain/people.md`, 5.3):

| Act | Grant word | Subject | Becomes | Answer |
|---|---|---|---|---|
| `Act::Cancel { project: u32, task: u64, grace: Option<Duration>, reason: Box<[u8]> }` | `cancel` | a live task of the named project, any requester (a change, a chat, a procedure); the grace and reason bounded as `Close`'s | the deployment's compound (jig's `domain/tasks.md`, the compound request): each live delegate cancelled, then the task with the grace, each with the reason | as `Close`'s: a compound's outcome; *ended* for a task ended since, *unknown* past retention |
| `Act::RemoveProject { project: u32 }` | `remove_project` | a live project of the deployment | the project's deletion (draft, section 5) begun in this decision: closed at once, then its tasks cancelled, ended, erased, then its rows, resumed at each start until whole | `Outcome::ProjectRemoved { project }` once the closing decision is durable; the rest of the deletion is jig's own, and needs no answer |

- `ActKind::{Cancel, RemoveProject}` join `ActKind::ALL`; the grant
  parser (`jig-protocol-config`, `acts.rs`) reads `"cancel"` and
  `"remove_project"`.
- The owner's ask, `Ask::RemoveProject { project }`, of the project's
  owners, answers the same `Outcome::ProjectRemoved { project }`. Its
  client wire and jig's client page (Delete, confirmed by the project's
  name) are the part's later tasks.
- **A project removed is gone** from its closing decision: every ask,
  act, adoption, message, task or settings change naming it is refused
  as one naming an unknown project; it leaves every list; its name is
  free; its number is never reused.
- **The application is told** in the closing commit: the core's request
  to the root gains `ProjectRemoved { project }` (beside today's
  `ProjectMade`), delivered as `ProjectMade` is. temper's root routes it
  to the forge connector (part `remove-forge` owns temper's handling:
  the project's `Repository` rows leave it, its kinds' `Setting` rows of
  its scope are erased). A root that has nothing to do with it ignores it
  explicitly.
- **Whole-task erasure at once:** jig gains one entry that erases an
  ended task whole, as its retention would at the horizon (jig's
  `protocol/store.md`, 11.1), its transcript with it; the deletion uses
  it, and nothing else of this run does.

## 2. temper: the forge connector's store families (parts `forge-records`, `remove-forge`, `playground`)

In `temper_engine_domain_forge::Stored` / `Key` and their codecs in
`temper-protocol-store` (temper's `protocol/store.md`, section 2). The
families 23 to 30 stay reserved as that section has them.

| Byte | Family | Key | Holds | Owner part |
|---|---|---|---|---|
| 17 | `Playground` (was `Demo`) | `Key::Playground` (no field: one row) | the playground's state, `Running` or `Resetting`, and what it made: its forge's number, its project's and its account's, each once made | `playground` |
| 19 | `Kept` (was `DemoAct`) | `Key::Kept([u8; 16])` | a deployment's act the forge connector asked and has not heard answered: `KeptAct::{Cancel { project, task }, RemoveProject { project }, RemoveAccount { account }}` (*at least*), sent again at each start until answered | `remove-forge` makes the rename and adds `Cancel`; `playground` adds `RemoveProject`'s use |
| 31 | `Forge` | `Key::Forge(u16)` | a forge's record, `ForgeRecord` (below) | `forge-records` |
| 32 | `ForgeNext` | `Key::ForgeNext` (no field: one row) | the next forge number, `u16`, never lowered: numbers are never reused | `forge-records` |

- **`ForgeRecord`** (`temper_engine_domain_forge::boundary`), *at least*:
  `forge: u16`; `kind: Kind` and `mode: Mode` (today's `Kind`, `Mode`,
  `Mode::Fake` among them); what the connector's `Configured` holds
  today (bot login, git and web addresses, read-only, owned details,
  admin connect, asked); the forge's document entries as the service
  parses them (its `application.forges`, `application.identities` and
  `application.setup` members for that forge, as bytes the service's
  parsers read: the domain never reads them); `first: Option<FirstIntent>`,
  the first repository's intent (today's volatile `First`, its
  organisation, repository, project, label and playground flag); and
  `removal: Option<Removal>`, set by `RemoveForge` (part `remove-forge`
  defines `Removal`; part `forge-records` writes `None`).
- `Key::Forge(u16)` encodes as family byte 31 then the number widened to
  eight bytes; `Key::ForgeNext` and `Key::Playground` as the family byte
  alone. `STORE_VERSION` follows section 4 of temper's `protocol/store.md`.

## 3. temper: the connector's entries for the other parts

On `temper_engine_domain_forge::Domain` (names binding, bodies the
owner's):

- `make_forge(&mut self, new: NewForge, out) -> Result<u16, MakeRefusal>`
  (`forge-records`): numbers it from `ForgeNext`, saves its record and the
  bumped `ForgeNext` in one decision, and asks the box for its group
  (section 4) when its kind runs on the box. `NewForge` names the choice
  (`Playground`, `Owned`, `GitHub`, `Forgejo` connected or managed) and
  what the person gave; the service fills the document entries from the
  machine's members (section 5).
- `remove_forge(&mut self, forge: u16, out) -> Result<(), RemoveRefusal>`
  (`remove-forge`): in one decision ends the forge (as `domain/forge.md`
  4.1 ends the playground's today, for any forge), sets its record's
  `removal`, and keeps the rest (its changes cancelled by `KeptAct::Cancel`
  rows, its box group removed, its secrets, writer and flights erased,
  its repositories' rows once no kept task names them, then its record)
  until whole, across restarts. The playground's reset calls it.
- Client acts (`temper-client-protocol-wire`, the forge family):
  `RemoveForge { forge }` (`remove-forge`); `StartPlayground` and
  `EndPlayground`, replacing `StartDemo` (`playground`); `SetUp` and
  `Connect` naming a `NewForge` choice for a forge not yet kept, or a kept
  forge's number to go on with (`forge-records`).

## 4. The box (part `forge-records`; `remove` by `remove-forge`)

- **Group files:** before `start FORGE`, serve writes
  `<data>/box/groups/FORGE`, mode 0600 (`box` and `box/groups` 0700): the
  group's kind (`owned` or `fake`, the playground's being `fake` with the
  fake LLM first) and the forge's setup entry and listener, as JSON. Its
  codec is one module of `temper-forge-setup` (`group`), read by the box
  and written by the service; the box builds the group's steps by its
  kind from it (today's `temper-forge-setup` and `temper-fake` steps).
- **At the box's start** it runs each group whose `started` record
  exists, from its group file; a group file without a record stays
  dormant until asked. The document names no forge.
- **The pipe** gains, beside `start`/`stop` (temper's `protocol/setup.md`,
  7.1): `remove FORGE` from serve, answered `removed FORGE` once the box
  stopped the group (if running) and removed `<data>/box/groups/FORGE`,
  `<data>/box/started/FORGE` and the forge's directory, keeping the cache;
  a forge with no group file answers `removed FORGE` at once; `busy` and
  `stopping` as `stop`'s.

## 5. The document (part `forge-records`)

- `application.forges`, `application.identities`, `application.setup`'s
  forge entries and `application.demo` leave `temper.json`; `temper init`
  names no forge. The machine's members stay or move under one member
  (the clients' and webhooks' listeners, the owned Forgejo's listener and
  directory pattern, the fakes' listeners): `forge-records`' note names
  them.
- `forge.forges`: the forges kept at most (a count, `Limits`); the other
  forge limits are the deployment's counts, not sums over forges.
- `engine.acts` in `temper init`'s template grants connector 1 `cancel`,
  `remove_project`, `remove_account`, and what the playground's start
  needs (part `playground`); `close` goes with the demo's end.

## 6. Order between parts

- `forge-records` first (draft 7.1); `remove-forge` on it (7.2, with
  6.4's start over); `jig-remove-project` from the start, in parallel
  (7.3); `playground` on all three (7.4).
- A task names in `needs` only the tasks whose merged code it builds on.
