# The run's binding shapes (playground-reset-r6)

The lead's calls for the shapes that cross tasks, crates and trees. Every task builds to them;
each section says which draft section it settles
(`temper/docs/design/draft/playground-reset.md`). A name here is exact; a shape left
"inside" is the implementer's.

## J. jig: deleting a project (draft section 5)

### J1. The deployment's act and its grant

- `jig_domain_people::Act::RemoveProject { project: u32 }`, a new variant appended after
  `Adopt`; `ActKind::RemoveProject` appended to `ActKind` and to `ActKind::ALL`, last.
- The configuration's grant word is `remove_project` (`jig-protocol-config`'s `acts`
  parser), beside `close`, `add_key`, `remove_account`, `adopt` and `make_project`.
- Its subject: a live project (not deleting, not deleted). Refused `Unknown` for a project
  that does not exist or is being deleted; its answer once made is
  `Outcome::ProjectRemoved { project: u32 }`, a new `Outcome` variant. Saved and replayed
  as every act's answer is (people.md, 5.3); a resend after the deletion ended answers the
  saved answer while it is retained, and `Unknown` past it, which the connector takes as
  done.

### J2. The owner's request

- `jig_domain_people::Ask::RemoveProject { project: u32 }`, a party's request of the
  project's scope, admitted for the project's `Owner` role alone (as `Rename`); answered
  `Outcome::ProjectRemoved { project }`.
- Its offer in the owner's project document: `jig_domain_core::acts::Act::RemoveProject`,
  offered to the project's owners; the client's contract and wire carry it as they carry
  `Rename`'s.

### J3. What one decision does (closed at once)

In the decision that admits `RemoveProject` (act or request), jig writes the project's
**deletion row** and the project is *deleting*: it leaves every list and document; every
later request naming it (a task, a message, an adoption, a settings change, a watch) is
refused `Unknown`, as for a project that does not exist; each of its live tasks is
cancelled as a conversation's close cancels it, with its grace (the deployment's compound,
`Party::Deployment { project }`, reason `Project deleted`). The answer is given then: a
deletion is answered at its start, not at its end.

### J4. The deletion row and its stages (kept until whole)

- One row per deleting project, in the people family (people v13 changes in place: schema, generated code, goldens and the layouts
  fingerprint, as `jig/docs/design/protocol/store.md`, 5 says before the first release), keyed by the project's number: `{ project: u32, stage: Stage }` with
  `Stage::{Ending, Erasing, Releasing}`. The row's exact record is jig's
  (`jig-protocol-store`), inside.
- *Ending:* waits until every task of the project has ended (its grace honoured).
- *Erasing:* each ended task erased whole at once, as retention would erase it at the
  horizon: its ended row, its stub, history, activity, milestones, updates, results,
  ended-delegate rows and edges, its indexes (ended-by-party, ended-by-project), its
  proposals and placed holds, its transcript's file (`DropTranscript`), every party's
  inbox entry, read position and result reference naming it.
- *Releasing:* the project's own rows: its policy copy and name, its roles, its live
  ledgers of the current period (spend already settled stays in the parties' and the
  deployment's totals, never per project), its notes of every scope, its account uses
  (`Use::Project(project)` dropped from each account's uses; an account whose uses become
  empty keeps `Some([])`), and each connector told (J5). Then the row is erased; its name
  is free again; its number is never reused.
- At each start, before the core opens to requests, jig restores every deletion row and
  resumes it at its stage, so no start serves a half-deleted project.
- Settings of a project's scope: none exist in code today (every setting is the
  deployment's, scope 0); nothing is erased for them, and the report says so.

### J5. The connector told

- `jig_domain_core::Ask::ReleaseProject { project: u32 }`, a new variant of the core's ask
  to a connector (`routing/vocabulary.rs`), sent to every configured connector in the
  *Releasing* stage, and again at each start while the row stands there.
- The connector answers `jig_domain_core::Event::ProjectReleased { project: u32,
  connector: u16 }`, a new core event, once its own rows for the project are erased
  durably. Answering a project it holds nothing for is immediate. jig erases the deletion
  row once every connector has answered.
- temper's root routes `ReleaseProject` to its forge connector (section T of this note);
  until temper's task does, the jig task adds temper's arm answering `ProjectReleased` at
  once (common.md, "Across trees").

### J6. Whole-task erasure, the mechanism

- A tasks-child request erasing one ended task whole (J4's *Erasing* list) and a core write
  `jig_domain_core::Write::DropTranscript { task: u64 }` reaching the store's existing
  `Write::DropTranscript` (`jig-protocol-store/src/boundary.rs`). temper's root routes core
  writes to its store: the jig task adds that arm (common.md, "Across trees").

### J7. jig's client

- jig's client wire, link and engine-clients carry `Ask::RemoveProject { project }`, its
  outcome and its offer as they carry `Rename`'s (client-v19 changes in place, as `jig/docs/design/protocol/client.md`
  says before the first release).
- `jig-client-view`'s project settings (`/projects/N/settings`) offers **Delete project**
  to an owner when offered: a confirmation that asks the person to type the project's name,
  its send disabled until the typed text equals the project's current name; on its answer
  the page leaves for the projects list. temper's client has no project settings page:
  nothing in temper's client changes for J.

### J8. Two more deployment acts temper's removal and playground need

Order: `Act::{…, Adopt, RemoveProject, Cancel}`; `ActKind::{…, MakeProject, RemoveProject,
Cancel}`, `ALL` in that order. A party's retained `Answer` naming a deleted project is kept
as a connector's `Acted` is (J4).

- `Act::Cancel { project: u32, task: u64, reason: Box<[u8]> }`, `ActKind::Cancel`, grant
  word `cancel`: the deployment's compound cancel of one live task of the project, whoever
  requested it, with its delegates cancelled first (as `Close`'s compound, without `Close`'s
  "a person's conversation" rule); answered as a compound request. Refused `Ended` for a task
  that has ended, which the connector takes as done. temper's `RemoveForge` uses it for each
  change on the forge's repositories.
- `Act::AddKey` gains `uses: Option<Box<[jig_domain_people::Use]>>` (people's own `Use`, which the core
  translates to accounts'), the uses the new
  account is made with (`None` as today): the playground's scripted account is made with
  `Some([people::Use::Project(playground's project)])`, serving that project alone.

## T. temper (draft sections 3, 4, 6)

### T1. Forge numbers and the forge record

- A forge's number is a `u16` from 1, given by the forge connector when the forge is made,
  the next after the highest ever given, never reused. The counter is a store row (T2,
  `ForgeNumbers`); 65535 given, the next make is refused `Full`.
- The record, in `temper_engine_domain_forge` (boundary), secret-free:

  ```rust
  /// A forge the deployment keeps (draft playground-reset.md, section 3).
  pub struct ForgeRecord {
      pub number: u16,
      pub kind: Kind,                    // Forgejo | GitHub, as today
      pub mode: Mode,                    // Connected | Managed | Owned | Fake, as today
      pub api: Api,
      pub git: Box<[u8]>,                // git base, as `application.forges[].git` was
      pub git_program: Option<Box<[u8]>>,// a fake's git program (data path), as setup's `fake.git`
      pub web: Option<Box<[u8]>>,
      pub read_only: bool,               // a pasted App another owns (setup.md, 17.7)
      pub bot: Bot,                      // the bot's git identity
      pub setup: Option<Box<[u8]>>,      // the setup entry, T3's spelling; None for none
      pub first: Option<First>,          // the first repository's intent, the existing `First`
      pub playground: bool,              // the playground's fake forge
      pub state: ForgeState,
  }
  pub struct Api { pub host: Box<[u8]>, pub port: u16, pub transport: ApiTransport }
  pub enum ApiTransport { Plaintext, Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> } }
  pub struct Bot { pub login: Box<[u8]>, pub name: Box<[u8]>, pub email: Box<[u8]> }
  pub enum ForgeState { Kept, Removing }
  ```

  `trust` is a file path, as the document's was. The worker's git identity for a forge is
  credential number `forge as u32` (`GitIdentity { forge, credential: forge as u32 }`).
- `Configured` stays the domain's projection that its readers use, now built from each
  `Kept` record (`admin_connect`, `bot`, `owned`, `asked` derived as today from the record's
  mode and setup entry; `asked` is true for every record with a setup entry).

### T2. temper's store, version 11

`temper-protocol-store`'s `STORE_VERSION` becomes 11, once for the run (later tasks change
the v11 schema in place). Families, after the existing ones:

| Byte | Family | Key | Row |
|---|---|---|---|
| 31 | `Forge` | `Forge(u16)`: the forge's number | the `ForgeRecord` (T1) |
| 32 | `ForgeNumbers` | `ForgeNumbers` (one row) | `{ next: u16 }`, the next number to give; absent is 1 |
| 33 | `Playground` | `Playground` (one row) | `{ state: PlaygroundState, forge: u16, project: Option<u32>, account: Option<u32> }` (T6) |
| 34 | `KeptAct` | `KeptAct([u8; 16])`: the act's key, the forge connector's key space | `{ key: [u8; 16], forge: u16, act: KeptActKind }` (T5) |

`Demo` (17) and `DemoAct` (19) go with the demo's end (T6's task removes them; their bytes
stay unused). Bounds: `Forge` and `ForgeNumbers` by `forge.forges` (T4); `KeptAct` by
`tasks` plus `forge.forges` times 2.

### T3. The setup entry and the box's groups

- A record's `setup` is the setup entry as `application.setup[]` spelled it (setup.md,
  section 8) **without** `forge`, `directory` and `start`: `{"bot": {...}, "mode": ...}`,
  JSON text, at most 4096 bytes. Its directory is `forge-N` under the data directory, for
  every kind (a fake's too).
- `<data>/box/groups/N`, 0600 in `<data>/box/groups/` 0700: the group's settings, the
  record's entry with `"forge": N` and `"directory": "forge-N"` added, so it parses as one
  `application.setup[]` entry did (`temper_forge_setup::setup`'s entry parser). serve writes
  it (whole, by a temporary file renamed into place) before it asks `start N`.
- The box builds a group's steps by its mode: owned, `forge fetch` (when fetched), `forge
  prepare`, `forge run N forgejo`, `forge run N runner` (as today's asked owned group); fake,
  `fake llm`, `forge prepare N`, `fake forge N` (as today's playground group, labelled
  `Playground`); managed, none (it is reconciled in serve). Every group is asked and
  recorded: at a box start, each group whose `box/started/N` and `box/groups/N` both exist
  joins the start chain before serve; a `box/started/N` with no `box/groups/N` is removed.
  `temper forge ... N` and `temper fake forge N` read `box/groups/N`, never the document.
- The pipe gains a request and a reply, beside `start N`, `stop N`, `step`, `started N`,
  `stopped N`, `failed N WHY` (setup.md, 7.1):
  - `remove N\n`: the box stops group N if it runs (as `stop N`), then removes, in this
    order, `box/started/N`, the directory `forge-N` and everything in it, and
    `box/groups/N`; it keeps the cache (Forgejo's binaries). Answered `removed N\n`. A
    `remove N` for a group with none of the three answers `removed N` at once; a restart
    mid-removal leaves `box/groups/N` without its marker, a dormant group the next `remove
    N` ends.
  - `start N` for a group with no `box/groups/N` answers `failed N no settings`.

### T4. The document after the change

- `application.forges`, `application.identities` and `application.setup` leave the
  document: a document naming one is refused, naming the member.
- `application.machine`, new, the machine's:
  - `owned`: the owned Forgejo here, a setup entry's body `{"bot", "mode": {"owned": ...}}`
    as `temper init` wrote forge 3's (its listener the box's owned port), or null for none;
  - `github`: `{"api", "transport", "git", "web"}`, each null for github.com's, as
    `application.forges[]`' GitHub entry spelled them.
  - `git_program`: null, or a data path: the git program of every forge the connector makes
    but the owned one and the playground's (whose setup entry's `git` is its own). serve puts
    it in `Machine.github.git_program`; a Forgejo elsewhere's record takes the same
    (`fe-make-git`). `temper init` writes null; `temper box --fake-backends` sets it.
- `application.demo` keeps its name: its `llm` as today; `forge`, now the playground's fake
  forge's entry body `{"bot", "mode": {"fake": ...}}` (as setup entry 1 was, its listener the
  box's fake forge port, its `git` `demo/bin/git`); `project`, now `{"name": "Playground",
  "policy": 1}`, the project the playground's start makes. `null` for no playground.
- `engine.acts`: `[{"connector": 1, "may": ["make_project", "add_key", "adopt", "cancel",
  "remove_project", "remove_account"]}]` in `temper init`'s document; `close` goes.
- The machine's templates reach the forge connector in the root's `Config` (replacing
  `forges: Box<[Configured]>` and `demo`'s forge): `temper_engine_domain_forge::Machine {
  owned: Option<ForgeRecord>, github: ForgeRecord, playground: Option<ForgeRecord> }`,
  each a record with `number` 0 and `state` `Kept`, which the connector copies and numbers
  when it makes a forge: `owned` from `application.machine.owned` (mode `Owned`), `github`
  from `application.machine.github` (mode `Managed` until a `Connect` with a pasted App sets
  `Connected`), `playground` from `application.demo.forge` (mode `Fake`). A Forgejo
  elsewhere is made from the request's addresses, mode `Managed`, with a managed
  setup entry and no admin token (`{"bot": {"login": "temper", "email":
  "temper@localhost"}, "mode": "managed"}`), so its row offers the admin connect (`admin`)
  and the setup host has its settings.
- The request that makes a forge: `forge::Ask::MakeForge { choice: ForgeChoice }`,
  `ForgeChoice::{Owned, GitHub, Forgejo { api: Box<[u8]>, git: Box<[u8]>, web:
  Option<Box<[u8]>> }}`, deployment scope, admitted for owners as `SetUp` is; answered
  `MakeAnswer::{Made { forge: u16 }, Refused(MakeRefusal::{Full, NotOffered})}`
  (`NotOffered` for `Owned` with no `machine.owned`, or not under the box). In its decision:
  the record saved, `ForgeNumbers` moved on, `Register` sent. `SetUp` and `Connect` then
  name that number as today; `SetUp`'s decision writes `first` into the record (the intent
  kept), and `FirstMade`/`FirstFailed` update it.
- Limits: `forge.forges`, new, a u32, the forges kept at most (`Kept` and `Removing`
  records); 4 in the `small` profile, 16 in the others. `forge.repositories` and its kin are
  the deployment's, unchanged. Nothing is sized by a count of configured forges.
- `temper init` writes no forge: a first run's store holds none.

### T5. Removing a forge

- The request: `forge::Ask::RemoveForge { forge: u16 }`, deployment scope, admitted for the
  deployment's owners as `SetUp` is; answered `RemoveAnswer::{Removing, Refused(
  RemoveRefusal)}`, `RemoveRefusal::{NoSuchForge, Playground}` (the playground's forge is
  removed only by the reset). Kept in memory as `SetUp`'s answer is.
- In its decision: the record's `state` becomes `Removing`, and the forge is ended at once
  (generalising `forge_ended`: a `Removing` record, or none, is ended); its repositories
  leave their projects; its writer row and its flights are erased; and the decision writes
  its kept acts, each a `KeptAct` row (T2):
  - `KeptActKind::Cancel { project: u32, task: u64 }`, one per live change task on its
    repositories: jig's `Act::Cancel` (J8), reason `Forge removed`;
  - `KeptActKind::RemoveGroup`, under the box when the record has a setup entry: the box
    link's `remove N` (T3), done on `removed N`;
  - `KeptActKind::EraseSecrets`: serve erases the forge's records in jig's secrets store
    (temper-protocol-records' items for the forge), done once durable.
- Each kept act is sent in the decision and again at each start once the core is ready,
  until answered, as `DemoAct`'s are today; on its answer its row is erased. When the last
  row of a `Removing` forge is erased, the `Forge` row is erased. Its repository rows stay
  while a task names them (draft section 9's default); a repository row whose forge has no
  record is a removed forge's: in no list, readable by its tag.
- The root's requests to serve, beside `StopGroup` and `SetUpGroup` (each a variant of both
  `temper_engine_domain_forge::Request` and `temper_engine_domain::Request`):
  - `SetUpGroup { forge: u16, request: Token, entry: Box<[u8]> }`: `entry` is the record's
    setup entry; serve writes `box/groups/N` (T3) before `start N`;
  - `RemoveGroup { forge: u16 }` → serve's box link sends `remove N`; serve answers the root
    `Event::GroupRemoved { forge: u16 }` on `removed N`;
  - `EraseSecrets { forge: u16 }` → serve answers `Event::SecretsErased { forge: u16 }`;
  - `Register(ForgeRecord)`: a forge made, or restored at start, or its record changed:
    serve adds or updates it in its forge layer (skein's connection component, T7) and its
    setup host; `Unregister { forge: u16 }`: removed from both, at the removal's decision.
- The client: Remove on the forge's row in Forges, confirmed; its words name what goes (the
  forge's records, its box group and directory for an owned one) and what stays on the forge
  (a managed Forgejo's bot and hooks; a GitHub App, linked to its settings to delete it).

### T6. The playground

- `forge::Ask::StartPlayground` replaces `StartDemo`; `forge::Ask::EndPlayground`, new;
  both deployment scope, admitted as `SetUp` is.
- `PlaygroundState::{Running, Resetting(ResetStep)}`, `ResetStep::{Project, Forge,
  Account}`; no row is *not started*.
- Start: refused `NotOffered` while any `Kept` record that is not the playground's exists,
  or while the row reads resetting. Otherwise, in one decision: the row `Running` (when
  none); the fake forge's record made from `application.demo.forge` (T4), `playground:
  true`, mode `Fake`; `Register` and `SetUpGroup` for it; then as the deployment's acts,
  each keyed from the start's key and kept until answered: `MakeProject` of
  `application.demo.project` (its number written to the row), `AddKey` of the scripted key
  with `uses: Some([Use::Project(project)])` (its account written to the row), and, once the
  fake forge connects, `Adopt` of `playground/widget` into the project. Answered `Running`
  once all are done; `StartAnswer` keeps its other values.
- End: in one decision the row reads `Resetting(Project)`; then, in order, each step after
  the last one's answer, resumed at each start from the row: `Project`, jig's
  `Act::RemoveProject { project }` (J1), done on `ProjectRemoved` or `Unknown`; `Forge`, the
  connector's own removal of the playground's forge (T5's decision, without the request's
  refusal), done when its `Forge` row is erased; `Account`, `Act::RemoveAccount { account
  }`, done on its answer or `Unknown`; then the row is erased. `EndAnswer::{Reset,
  NotOffered}`: `Reset` once the row is erased, `NotOffered` with no row; kept in memory.
- A start of the engine that finds the row `Running` resets it, as `EndPlayground` does
  (draft section 9's default).
- What goes: `DemoState::Ended`, the demo's end on another forge's connect, `DemoAct` and
  its end's acts, the client's model and forge halves, the chats closed "Ended with the
  playground", and the `close` grant.

### T7. skein: forges at runtime

- `skein-forge-connection`'s component takes forges added and removed after its start:
  `Request::Add { forge: u16, settings: <one forge's settings, as its start's list holds
  them> }` and `Request::Remove { forge: u16 }` (its boundary), each answered by its
  existing events; a `Connect` for a forge not added stays ignored, as today.
- `skein-forge-setup`'s host likewise: `Add { forge, settings }` and `Remove { forge }` for
  its per-forge settings.
- Neither names temper; both keep their start-time lists as the initial set.

### T8. The client wire, version 12

`temper-client-protocol-wire`'s family `temper-client-wire` moves to 12, once for the run, in
the client's last task with the journeys recorded again; until then the shapes below are
built at version 11, renamed variants keeping their tags and new ones appended:

- `Request::start_playground` replaces `start_demo`; new `end_playground`;
  new `remove_forge { forge: u16 }`; new `make_forge { choice: ForgeChoice }`,
  `ForgeChoice { owned, github, forgejo: { api: text 259, git: text 1024, web: option text
  1024 } }`, answered `Outcome::made { forge: u16 }` or a refusal (`full`, `not_offered`);
  then `set_up`/`connect` name that number as today.
- `Read::playground` replaces `Read::demo`; `PlaygroundAnswer` replaces `DemoAnswer`, its
  `state` `PlaygroundState { not_started, running, resetting }`, its other members as
  `DemoAnswer`'s.
- `Outcome` gains `removing`, `reset`, and the refusals `playground` (`remove_forge` of the
  playground's forge) and `not_offered` where it lacks one.
- `ForgeRow` keeps its members; the engine lists every `Kept` record, `Removing` ones never.

## G. Generated files under docs/

A task may commit what a generator rewrites under `docs/`, never a hand edit there: the ctl
reference pages `temper/docs/help/reference/ctl-*.md` (`SKEIN_CODEGEN_WRITE=1`, checked by
`every_ctl_group_page_is_what_the_table_renders`) and a journey's reprinted pages
`temper/docs/design/client/journeys/*/NN.html` (`heavy cargo run -p temper-client-journeys`),
never a recording. The journey `welcome-set-up`, whose walk changes with forges made at
runtime, is deleted by the client's last task, to be walked again by a person after the run.
