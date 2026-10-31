# The contracts between this plan's parts

The lead's, fixed before the parts were cut: each part builds on these
names and shapes, and a task that needs one changed asks. Section
numbers without a document are the draft's
(`temper/docs/design/draft/playground-reset.md`). Inside a part, the
shapes not fixed here are its tasks'.

## 1. jig: the deployment's new acts and project deletion (section 5)

In `jig-domain-people` (`src/boundary.rs`):

- `Act::RemoveProject { project: u32 }`: the deployment's act
  (`jig/docs/design/domain/people.md`, 5.3), its subject a live project.
- `Act::Cancel { project: u32, task: u64, grace: Option<Duration>,
  reason: Box<[u8]> }`: the deployment's compound cancel of one live
  task of the project and its delegates (`jig/docs/design/domain/tasks.md`,
  the compound request), any requester, answered as `Close` is. A forge's
  removal cancels each change on its repositories with it (section 4).
- `ActKind::RemoveProject` and `ActKind::Cancel`, in `ActKind::ALL`; their
  names in `engine.acts` (`jig-protocol-config/src/acts.rs`):
  `"remove_project"` and `"cancel"`.
- `Ask::RemoveProject { project: u32 }`: the project's owners' request,
  a party's, offered as `ActKind`'s `RemoveProject` in the client's
  `OwnerProject.acts` (`jig-client-domain`), and named the same on the
  client contract (`jig_client_domain::contract::Ask::RemoveProject {
  project }`).
- `Outcome::ProjectRemoved { project: u32 }`: the answer to both, given
  once the decision that closes the project commits (the project closed
  and its deletion kept); the deletion goes on after the answer and ends
  whole across restarts (section 5, "Kept until whole"). A project gone
  or being deleted is refused as unknown.
- Connectors told: `jig_domain_core::connector::Ask::ProjectRemoved {
  project: u32 }`, sent by the core to every connector once the
  project's tasks are erased, in the decision that erases its own rows.
  One way: the connector erases what it keeps for the project in the
  same decision, and answers nothing. temper's forge connector erases
  its repository rows of the project (and its settings of the
  project's scope, kinds 2, 4 and 5, where they exist).
- The transcript's drop: `jig_domain_core::routing::vocabulary::Write::
  DropTranscript { task: u64 }`, a write of the core's decisions, which
  an application's root carries to `jig_protocol_store::Write::
  DropTranscript { task }` (it exists there today). temper's root
  `Write::DropTranscript { task }` coalesces on a root key of its own,
  `temper_engine_domain::Key::Transcript { task }`, beside `Key::Wires`,
  and never on a task row's key.
- Spend outlives the project: its funding ledgers are not erased
  (section 9's default). The referee's "no row names the project"
  excludes them, and nothing else.
- A retained answer that named the project answers unknown on replay
  once the project is gone: a project number below the deployment's
  `projects` high-water mark and not live is gone.

## 2. temper: the forge connector's new families (sections 3, 4, 6)

In `temper-engine-domain-forge/src/boundary.rs`, added beside today's
families (`Demo` and `DemoAct` stay until the `playground` part removes
them):

```rust
/// A forge kept in the store (playground-reset.md, 3).
pub struct ForgeRecord {
    pub number: u16,
    pub kind: Kind,
    pub mode: Mode,
    /// Ended at once by its removal; erased once its kept acts are answered (4).
    pub removing: bool,
    /// The forge's members as the document had them, without its numbers (below).
    pub definition: Box<[u8]>,
    /// The first repository's intent, kept until it is made (3; setup.md, 6.7).
    pub first: Option<FirstIntent>,
}
pub struct FirstIntent {
    pub playground: bool,
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub project: SetUpProject,
    pub label: Box<[u8]>,
}
/// The playground's row (6.1, 6.2): none is no playground.
pub struct Playground {
    pub state: PlaygroundState,
    pub forge: Option<u16>,
    pub project: Option<u32>,
    pub account: Option<u32>,
}
pub enum PlaygroundState {
    Running,
    Resetting(ResetStep),
}
/// The reset's acts, in order (6.2).
pub enum ResetStep {
    Project,
    Forge,
    Account,
}
/// An act a forge's removal keeps until answered (4, "Kept until whole").
pub struct KeptAct {
    pub key: [u8; 16],
    pub forge: u16,
    pub act: KeptActKind,
}
pub enum KeptActKind {
    /// jig's `Act::Cancel` of a change on one of the forge's repositories.
    Cancel { project: u32, task: u64 },
    /// The box's `remove N` (contract 3).
    RemoveGroup,
    /// The forge's secrets, each item `temper-protocol-records` numbers for it.
    EraseSecrets,
}
```

| Byte | `Stored` | `Key` |
|---|---|---|
| 31 | `Forge(ForgeRecord)` | `Forge(u16)` |
| 32 | `ForgeNumbers { next: u16 }` | `ForgeNumbers` |
| 33 | `Playground(Playground)` | `Playground` |
| 34 | `KeptAct(KeptAct)` | `KeptAct([u8; 16])` |

- **Numbers:** the forge connector gives the next forge `ForgeNumbers.next`
  (1 when the row is none) and saves `next + 1` in the same decision; a
  number is never given twice, removed forges' included. Forge numbers
  are 1 to `u16::MAX - 1`; `next == u16::MAX` is exhausted: no number is
  given, and a restored forge numbered `u16::MAX` is refused by the
  restore fit.
- **The definition** is JSON, one object, the members the document gave
  the forge: `{"forge": F, "identity": I, "setup": S}`, where `F` is the
  forge's `application.forges` entry, `I` its `identities` entry and `S`
  its `application.setup` entry or `null` (GitHub, connected), each as
  `temper/docs/design/protocol/configuration.md` and `setup.md` 8 have
  them, less the members that number the forge or the identity
  (`number`, `forge`, `identity`), which the record's `number` gives
  (the identity's number is the forge's). temper-service checks it with
  the readers that check the document's members today; the domain
  carries it as bytes, bounded by a new count limit
  `forge.definition_bytes`. A new forge's definition is composed by
  temper-service, in the request's translation, from the request and
  the machine's document (contract 4), by stock definitions per kind
  (owned Forgejo, the playground's fake, GitHub, managed and connected
  Forgejo: today's template entries, moved into code); the domain's
  decision numbers it.
- **The playground's reset** is the `Playground` row's own steps: each
  step's act is sent under a key derived from the step, and the row
  moves to the next step on its answer; the row is erased after
  `Account`. The `Forge` step is the connector's own removal of the
  playground's forge (section 4), done when that forge's record is gone.
- **A forge's removal** keeps one `KeptAct` per act in the decision that
  ends the forge; the forge's record is erased once no `KeptAct` names
  it.

## 3. The box's groups (sections 3 and 4)

- **`<data>/box/groups/N`**, 0600 in a 0700 directory, written by serve
  (atomically: a temporary file in the same directory, then renamed)
  before it asks `start N`: one JSON object, `{"setup": S,
  "playground": P}`, `S` the forge's `application.setup` entry as the
  document had it, numbered (`"forge": N`), its ports the machine's
  (the document's `application.ports`, contract 4), `P` true for the
  playground's group, whose steps add the fake LLM's.
- **The box builds a group's steps by its kind** from that file, owned
  Forgejo's or the fakes', by `temper-forge-setup` alone: the fake's
  steps are command lines (`temper fake forge DIRS N`, `temper fake llm
  DIRS`), so the box still names no `temper-fake`.
- **The child commands** (`temper forge fetch|prepare|run|reconcile DIRS
  N`, `temper fake forge DIRS N`, `temper fake llm DIRS`) read forge N's
  setup entry from `<data>/box/groups/N` (the playground's LLM members
  from the machine's document, contract 4), never from
  `application.setup`.
- **The pipe:** `start N` as today, the group read from its file when
  the box does not hold it yet; `remove N`, new: the box stops the
  group if it runs, then removes `groups/N`, `started/N` and the forge's
  directory (the setup entry's `directory`), keeping the cache, and
  answers `removed N`; a group or file already gone is removed all the
  same and answered `removed N`, so a kept `remove` sent again after a
  restart ends. A failure answers `failed N WHY`, as a start's.
- **At the box's start,** each `groups/N` whose `started/N` exists
  starts as a recorded group does today; one without stays dormant
  until asked. The box's own listeners stay its own: a group's removal
  closes no listener the box holds.

## 4. The document keeps the machine's (section 3)

- `application.forges`, `identities`, `setup` and `demo` leave the
  document; a first run's document names no forge.
- **`application.ports`**, new, the machine's ports `temper init`
  chooses and the box holds from its start: `{"owned": P, "forge": P,
  "llm": P}`, the owned Forgejo's, the fake forge's and the fake LLM's
  (each a u16, loopback). The webhooks' and clients' listeners stay
  where they are.
- **`application.playground`**, the scripted model's members of
  today's `application.demo.llm` (`provider`, `key`, `asked`, `cued`,
  `otherwise`; its `listen` is `ports.llm`) and `project`'s `name` and
  `policy`, or null for none. The scripted provider stays in
  `engine.llm` (6.1).
- **Limits:** `forge.forges`, the forges kept at most (default 4, the
  forge component's `forges`), and `forge.definition_bytes`.

## 5. The forge connector's requests (sections 4, 6), and the client's wire

In `temper-engine-domain-forge/src/boundary.rs`'s `Ask` and `Answered`,
and on the client's wire (`temper-client-protocol-wire`), in place:

- `RemoveForge { forge: u16 }`, of the deployment's scope, its owners':
  answered `Removing` at its decision, `NotOffered` for the playground's
  forge while the playground runs (only its reset removes it), `Unknown`
  for a forge not kept or already removing.
- `StartPlayground` (today `StartDemo`), answered as `StartDemo` is
  today; refused `NotOffered` while a real forge is kept (6.3).
- `EndPlayground`, of the deployment's scope: answered `Resetting` at
  its decision (the row reads resetting), `NotOffered` with no
  playground running.
- The read `Playground` (today `Demo`): none, or running (its forge,
  project, provider and the scripted model's cues, as `DemoAnswer`
  carries them today), or resetting.
- `SetUp` and `Connect` make a forge's record: `SetUp { org, repository,
  project, playground }` names no forge and makes an owned Forgejo's;
  `Connect { forge: Option<u16>, how }` makes one of the kind `how`
  gives when `forge` is none, and goes on with a kept one (GitHub's Go
  on, 6.4) when it names it. Each answer names the forge's number.
- The forges' rows (`forges` watch) list the kept records, a removing
  one not at all, each with whether it is finished (connected with its
  first repository adopted) for the welcome's states (6.3).
