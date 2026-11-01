# The run's contracts

The binding calls of the playground-reset run
(`temper/docs/design/draft/playground-reset.md`), for every part and task.
They fix the shapes that cross parts: what a task builds on, it builds to
this note, and what its own half owes the other side is here too. A task
spec may add detail inside a shape; it never changes one. The surveys
the calls rest on are `notes/surveys/` beside this file. Draft sections
are cited as "draft N".

## 1. Decisions that bound the run

- **D1. A document's forges seed the store, `temper init` writes none.**
  `application.forges[]`, `identities[]` and `setup[]` stay readable,
  checked as today, and seed `Forge` records (section 2) at the store's
  first start, the way `engine.projects` seeds jig's projects: each
  under its number, the next number past the highest seeded one. At a
  later start the records are the truth; the document's forges are
  read for nothing but their check. A removed seeded forge is not
  seeded again. Reason: every engine world, service test, QA runner and
  process story names its forges in its document (s1, 7); a seed keeps
  them while the product's own path, a first run, names none (draft 3,
  "A first run's document names no forge"). Nothing else of the old
  form stays: the engine, the box and the client read forges from
  records alone.
- **D2. One store version for the run.** temper's `STORE_VERSION` moves
  from 10 to 11 once, in the task that adds section 2's families
  (`store-families`); no other task moves it. A store at 10 refuses to
  open, saying so, as every move does. jig's store version moves once,
  in jig's first store task of the deletion, and no other.
- **D3. Removed is ended, then gone.** A forge's removal ends it in one
  decision exactly as `forge_ended` ends the demo's forge today
  (`temper/docs/design/domain/forge.md`, 4.1), for any forge; the `Forge` row stays,
  `removing`, until its last kept act is answered, then is erased. The
  forge's repositories' rows stay, ended and readable, until their
  project's deletion releases them (section 5); erasure by a task's
  retention is later (jig erases no task at a horizon today: s3, 2).
- **D4. Kept acts are one family.** The demo's end's `DemoAct` mechanism
  (written in the decision, sent again at each start once the core is
  ready, erased on the saved answer: `temper/docs/design/protocol/setup.md`, 8.4)
  becomes `Kept`, for a forge's removal and the playground's reset
  alike. Its rule for which answers erase a row and which keep it is
  the `DemoAct` rule as it stands.
- **D5. The playground's start resets a playground found running at
  serve's start** (the draft's open question 1, its default): the box's
  fake forge kept no state, so the start sends `EndPlayground`'s acts
  and the person meets the welcome. Open questions 2 to 5 take their
  defaults too: names kept (D3); owners delete a project, and the
  deployment for the playground; spend kept in totals; the welcome's
  Default project reused by the next set up's first repository.

## 2. temper's store (`STORE_VERSION` 11)

New families of the forge connector's `Stored` and `Key`
(`temper-engine-domain-forge/src/boundary.rs`), encoded in
`temper-protocol-store` (`schema/store-v11.schema`, `keys.rs`,
`records.rs`). Tags 20 and 22 stay reserved for `Setting` and
`Delivery`; tags 23 and 24 are `Landing` and `Receipt`. `Demo` (17)
and `DemoAct` (19) stay until the playground part's task that removes
the demo erases them from the schema, in version 11 still (D2); their
tags are then left unused.

| Tag | Family | Key | Row |
|---|---|---|---|
| 25 | `Forge` | `Key::Forge(u16)` | `Forge` (below) |
| 26 | `Forges` | `Key::Forges`, one row | `Forges { next: u16 }`: the next forge number, never lowered; absent means 1 |
| 27 | `Playground` (replaces `Demo`, 17) | `Key::Playground`, one row | `Playground { state: PlaygroundState, forge: Option<u16>, project: Option<u32>, account: Option<u32> }`, `PlaygroundState::{Running, Resetting}`; none is not started |
| 28 | `Kept` (replaces `DemoAct`, 19) | `Key::Kept([u8; 16])`, the act's key in the forge connector's key space | `Kept { key: [u8; 16], act: KeptAct }` |

```rust
pub enum KeptAct {
    /// jig's deployment act `Cancel` (section 4): a removed forge's live change.
    Cancel { project: u32, task: u64 },
    /// jig's deployment act `RemoveProject`: the playground's project.
    RemoveProject { project: u32 },
    /// jig's deployment act `RemoveAccount`: the scripted account.
    RemoveAccount { account: u32 },
    /// The box's `remove FORGE` (section 3), until `removed FORGE`.
    BoxRemove { forge: u16 },
    /// The connector's own removal of a forge, the reset's second step,
    /// answered once that forge's own kept acts are all answered.
    RemoveForge { forge: u16 },
}
```

The `Forge` row holds what the document's three members hold today for
one forge (s1, 1: the tables "Forge and identity fields" and "Setup
entries"), less what is a credential's (a file-backed `api_token`,
`webhook_secret`, a `user` by file: records-only forges keep their
secrets in `temper-protocol-records` as today), plus:

- `first: Option<First>`, the first repository's intent (draft 3):
  `org`, `repository`, `project: SetUpProject`, `label`, as
  `temper_engine_domain_forge::First` holds them less its volatile
  `state`;
- `playground: bool`, true for the playground's fake forge alone;
- `removing: bool` (D3).

`store-families` fixes the field list and its schema record; its
domain type is `temper_engine_domain_forge::Forge`, and the engine's
`Configured` is derived from it by one total function in that crate
(`Configured::from_record` or as the task names it), so no other crate
builds `Configured` by hand once the seed task lands.

## 3. The box: group files and the pipe

- **`<data>/box/groups/N`,** one file per forge whose group the box
  runs, written by serve (the routing's box link,
  `temper-service/src/route/box_link.rs`) before it sends `start N`,
  `box/groups` made 0700 as needed, the file 0600, written whole
  (a temporary file, then a rename). Its bytes are one JSON object:

  ```json
  {"setup": ENTRY, "playground": false}
  ```

  `ENTRY` is the forge's setup entry in exactly the form of one
  `application.setup[]` entry (`temper/docs/design/protocol/setup.md`, section 8), its
  `forge` member N and `"start": "asked"`; `playground` is true for the
  playground's group, whose steps put the fake LLM first, read from the
  document's `application.demo.llm`. The box, `temper forge prepare /
  fetch / run DIRS N` and `temper fake forge DIRS N` read entry N from
  this file when the document's `setup` has no entry N, and refuse,
  naming the file, when neither has it.
- **At the box's start** each `box/groups/N` file is a group, dormant
  (asked), parsed by the same reader; with its `box/started/N` record it
  runs in its place, as a recorded asked group runs today. The
  playground's group writes no `started` record (D5).
- **New pipe lines,** beside `start`, `step`, `started`, `failed`,
  `stop`, `stopped` (`temper/docs/design/protocol/setup.md`, 7.1), with their rule of one
  ask at a time:

  | Line | From | Says |
  |---|---|---|
  | `remove FORGE` | serve, a kept `BoxRemove` | close the group's steps if they run, then remove `box/groups/FORGE`, `box/started/FORGE` and the entry's `directory`; keep the cache (Forgejo's binaries) |
  | `removed FORGE` | the box | all of it is gone; also the answer for a forge with no group and no file, so a `remove` sent again after a restart answers |

  A `start FORGE` for a forge with no group in memory reads
  `box/groups/FORGE` first; with no file it answers `failed FORGE not
  asked`, as today.
- **The box stays wiring:** it knows kinds (owned Forgejo, fake), never
  forges; the steps of a kind are `temper-forge-setup`'s and
  `temper-fake`'s, given an entry.

## 4. jig's acts and asks

In `jig-domain-people` (`boundary.rs`) unless named:

- **`Ask::RemoveProject { project: u32 }`,** a party's request, the
  project's owners'.
- **`Act::RemoveProject { project: u32 }`,** a deployment act,
  `ActKind::RemoveProject`, granted by `engine.acts`' `"remove_project"`.
- **`Act::Cancel { project: u32, task: u64, grace: Option<Duration>,
  reason: Box<[u8]> }`,** a deployment act, `ActKind::Cancel`, granted by
  `"cancel"`: the deployment's compound cancel of one live task of the
  project, whoever requested it, its delegates cancelled with it as a
  cancel cancels them (`jig/docs/design/domain/tasks.md`, section 7).
  Its answer for a task already ended is *ended*, for one erased
  *unknown*: the forge connector takes both as done (D4).
- **The project's connectors told:** `jig-domain-core`'s
  `connector::Ask::ReleaseProject { request: Token, project: u32 }`,
  sent to every connector once the deleted project's tasks are erased,
  and answered by the connector's `connector::Event::ProjectReleased {
  request: Token, project: u32 }` once its rows for the project are
  erased (temper: its `Repository` rows of that project, ended or not;
  its settings of that scope when it has any). The deletion's last step
  waits for every connector's answer; a restart sends the ask again.
- **A deleted project is gone:** from its decision on, every request
  naming it is refused as for a project that does not exist, and no
  list, `me` or page names it.
- `engine.acts`' `"close"` goes with the demo's end (draft 6.3); jig
  keeps `Act::Close` itself, which other applications may grant.

## 5. temper's client wire and acts

In `temper-client-protocol-wire` (`schema/client-wire-v1.schema`) and the
forge connector's `Ask` (`temper-engine-domain-forge/src/boundary.rs`):

- **`RemoveForge { forge: u16 }`,** of the deployment's scope, admitted as
  `SetUp` is; the owners' act.
- **`StartPlayground`** (was `StartDemo`) and **`EndPlayground`**, of the
  deployment's scope, admitted as `SetUp` is.
- **`SetUp` and `Connect` make the forge:** `SetUp` no longer names a
  forge (it sets up the box's owned Forgejo, the machine's, section 6);
  `Connect` names the forge's kind and addresses when it makes one, or
  an existing forge's number when it connects one already made (GitHub's
  "Go on", a reconnect). The parts' tasks fix the fields.
- **The `forges` subject** lists the `Forge` records but the removing
  ones; each row says whether the forge is finished (connected with its
  first repository adopted) so the welcome reads empty, unfinished or
  whole (draft 6.3) from it and the accounts.

## 6. The document's machine members

- **`application.owned`,** optional: the box's owned Forgejo, the
  machine's, as `temper init` writes it (the template's forge 3 today):
  the members of its `setup[]` entry less `forge` and `start`, with its
  `listen`; `SetUp` makes its `Forge` record from it. Null or absent:
  `SetUp` is not offered.
- **`application.demo`** keeps `llm` (with `listen`, the fake LLM's
  port) and `project`; its `forge` (a number) is replaced by `fake`, the
  members of the playground's fake `setup[]` entry less `forge` and
  `start`, with its `listen` (the fake forge's port). `StartPlayground`
  makes the playground's `Forge` record from it.
- **The listeners** (`clients`, `webhooks`) and the limits stay as they
  are. `forge.forges` is the new limit: the forges kept at most
  (default 4), sizing every per-forge table of the forge connector and
  skein's connection component in place of `forge_component.forges`
  and of the `forge.client.repositories` bound now used for forges
  (s1, 5).
