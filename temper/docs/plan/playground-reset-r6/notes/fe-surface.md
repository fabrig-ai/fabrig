# The engine's surface for playground-reset-r6 (fe)

The exact shapes `fe-surface` adds to temper's engine crates, beside `notes/contracts.md`
T1, T2, T4, T5 and T6, which they implement. Every task that builds on the forge connector
(`temper-engine-domain-forge`, `forge::` below), the root (`temper-engine-domain`, `root::`)
or the store codecs (`temper-protocol-store`) reads this note. Sections 1 to 4 are the
contract's shapes as Rust; section 5 is fe's own vocabulary between the forge connector and
the root, which only fe's tasks use; section 6 is the store schema; section 7 says what each
shape does until the task that gives it its meaning.

## 1. The forge record and the machine (contracts T1, T4)

In `temper/crates/temper-engine-domain-forge/src/boundary.rs`, after `Configured`:

```rust
/// A forge the deployment keeps (draft playground-reset.md, section 3); secret-free.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ForgeRecord {
    pub number: u16,
    pub kind: Kind,
    pub mode: Mode,
    pub api: Api,
    /// The git base, as `application.forges[].git` was.
    pub git: Box<[u8]>,
    /// A fake's git program, a data path, as setup's `fake.git`.
    pub git_program: Option<Box<[u8]>>,
    pub web: Option<Box<[u8]>>,
    /// A pasted App another owns (protocol/setup.md, 17.7).
    pub read_only: bool,
    /// The bot's git identity.
    pub bot: Bot,
    /// The setup entry without `forge`, `directory` and `start`, JSON text (contracts T3).
    pub setup: Option<Box<[u8]>>,
    /// The first repository's intent, kept across a restart (draft, section 3).
    pub first: Option<First>,
    /// The playground's fake forge (draft, 6.1).
    pub playground: bool,
    pub state: ForgeState,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Api {
    pub host: Box<[u8]>,
    pub port: u16,
    pub transport: ApiTransport,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ApiTransport {
    Plaintext,
    /// `trust` is a file path, as the document's was.
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Bot {
    pub login: Box<[u8]>,
    pub name: Box<[u8]>,
    pub email: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ForgeState {
    Kept,
    Removing,
}
/// The machine's forge templates (contracts T4): records numbered 0 and `Kept`, which the
/// connector copies and numbers when it makes a forge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Machine {
    /// From `application.machine.owned`, mode `Owned`; none without one.
    pub owned: Option<ForgeRecord>,
    /// From `application.machine.github`, mode `Managed`.
    pub github: ForgeRecord,
    /// From `application.demo.forge`, mode `Fake`, `playground: true`; none without a playground.
    pub playground: Option<ForgeRecord>,
}
```

Two constructors, for the document's nulls and for neutral values in fixtures:

- `ForgeRecord::github_com() -> ForgeRecord`: number 0, `Kind::GitHub`, `Mode::Managed`,
  api `{ host: b"api.github.com", port: 443, transport: Tls { server_name:
  b"api.github.com", trust: None } }`, git `b"https://github.com"`, web
  `Some(b"https://github.com")`, `git_program: None`, `read_only: false`, bot of three empty
  byte strings, `setup: None`, `first: None`, `playground: false`, `state: Kept`. These are
  the GitHub defaults of `temper/crates/temper-service/src/startup/application.rs` (`forge`,
  its null members), which sv's parser of `application.machine.github` may use.
- `Machine::bare() -> Machine`: `{ owned: None, github: ForgeRecord::github_com(),
  playground: None }`.

`forge::Limits` gains `pub forges: u32`, the forges kept at most (`Kept` and `Removing`
records), placed after `repositories`. The engine's `small` profile is 4, every other 16
(contracts T4); fixtures take 4.

`root::Config` (`temper/crates/temper-engine-domain/src/translate.rs`) gains `pub machine:
forge::Machine`, after `forges`. Until `fe-records`, `forges`, `identities` and `demo` stay
beside it and `machine` is read by nothing; `fe-records` removes `forges` and `identities`.

`forge::DemoConfig` gains `pub project_name: Box<str>` and `pub project_policy: u32`, the
playground's project as `application.demo.project` names it (`{"name": "Playground",
"policy": 1}`, contracts T4), which the playground's start makes. Until `fe-playground-root`
nothing reads them; `adoption.project` stays and is read as today.

## 2. The connector's requests (contracts T4, T5, T6)

`forge::Ask`: `StartDemo` is renamed `StartPlayground` in its place; appended, in this order:

```rust
    MakeForge { choice: ForgeChoice },
    RemoveForge { forge: u16 },
    EndPlayground,
```

```rust
/// What `MakeForge` makes (contracts T4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ForgeChoice {
    Owned,
    GitHub,
    /// A Forgejo elsewhere: `api` is `host:port`, `git` and `web` URL bases.
    Forgejo { api: Box<[u8]>, git: Box<[u8]>, web: Option<Box<[u8]>> },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MakeAnswer { Made { forge: u16 }, Refused(MakeRefusal) }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MakeRefusal { Full, NotOffered }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RemoveAnswer { Removing, Refused(RemoveRefusal) }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RemoveRefusal { NoSuchForge, Playground }
/// The reset's answer (contracts T6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndAnswer { Reset, NotOffered }
```

`forge::Answered`: `StartDemo(StartAnswer)` renamed `StartPlayground(StartAnswer)`; appended
`MakeForge(MakeAnswer)`, `RemoveForge(RemoveAnswer)`, `EndPlayground(EndAnswer)`.
`root::ApplicationAnswer` likewise: `StartDemo` renamed `StartPlayground`; appended before
`Returning`: `MakeForge(forge::MakeAnswer)`, `RemoveForge(forge::RemoveAnswer)`,
`EndPlayground(forge::EndAnswer)`.

All four (`StartPlayground`, `MakeForge`, `RemoveForge`, `EndPlayground`) are of the
deployment's scope, as `SetUp` is (`root`'s `route/applications.rs`, `start`).

## 3. The root's requests to serve and serve's answers (contracts T5)

Both `forge::Request` and `root::Request`:

- `SetUpGroup { forge: u16, request: Token, entry: Box<[u8]> }`: `entry` is the record's
  setup entry (contracts T3), which serve writes to `box/groups/N` before `start N`.
- New, after `SetUpGroup`, in this order:

  ```rust
      /// Stop the forge's box group and remove its files (the pipe's `remove N`, contracts T3).
      RemoveGroup { forge: u16 },
      /// Erase the forge's records in the secrets store (temper-protocol-records' items).
      EraseSecrets { forge: u16 },
      /// A forge made, restored at start, or its record changed: serve adds or updates it.
      Register(ForgeRecord),
      /// A forge removed: serve drops it from its forge layer and setup host.
      Unregister { forge: u16 },
  ```

`root::Request::Register` holds `forge::ForgeRecord` (`Register(forge::ForgeRecord)`).

Serve's answers are `forge::Event` variants, which serve sends as
`root::Event::Forge(Box::new(forge::Event::GroupRemoved { forge }))`, as it sends
`BoxClosed` today (`temper-service/src/route/box_link.rs`):

```rust
    /// The box answered `removed N`.
    GroupRemoved { forge: u16 },
    /// The forge's records are erased from the secrets store, durably.
    SecretsErased { forge: u16 },
```

When the connector sends `Register` (from `fe-records`): for every restored record (`Kept`
and `Removing` alike, so serve can finish a removal's `EraseSecrets`) once restoration ends
(at `Event::Restored`, before the root's `ReadAfresh`), in number order; for a record made
(`MakeForge`'s decision); and whenever a saved record changes a member serve reads (mode,
addresses, bot, setup entry). `Unregister` goes in a removal's decision (`fe-remove`).

## 4. Store rows (contracts T2, T6)

`forge::Stored`, appended: `Forge(ForgeRecord)`, `ForgeNumbers { next: u16 }`,
`Playground(Playground)`, `KeptAct(KeptAct)`. `forge::Key`, appended: `Forge(u16)`,
`ForgeNumbers`, `Playground`, `KeptAct([u8; 16])`. `stored_key`: `Forge(row.number)`,
`ForgeNumbers`, `Playground`, `KeptAct(row.key)`.

```rust
/// The playground's row (contracts T6); no row is not started.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Playground {
    pub state: PlaygroundState,
    /// The playground's fake forge.
    pub forge: u16,
    pub project: Option<u32>,
    pub account: Option<u32>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlaygroundState { Running, Resetting(ResetStep) }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResetStep { Project, Forge, Account }
/// A removal's act, kept until answered (contracts T5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeptAct {
    pub key: [u8; 16],
    pub forge: u16,
    pub act: KeptActKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeptActKind {
    /// jig's `Act::Cancel` (contracts J8), reason `Forge removed`.
    Cancel { project: u32, task: u64 },
    /// The box link's `remove N`, done on `removed N`.
    RemoveGroup,
    /// Serve's erasure of the forge's secrets, done once durable.
    EraseSecrets,
}
```

`RecordKind` (`src/reconcile.rs`) gains `Forge`, `ForgeNumbers`, `Playground`, `KeptAct`,
appended. Restore bounds (from the tasks that restore them): `Forge` rows at most
`l.forges`; one `ForgeNumbers`; one `Playground`; `KeptAct` rows at most `l.tasks +
2 * l.forges`.

## 5. fe's own vocabulary between the connector and the root

Only fe's tasks emit or route these; other designers' code never names them.

`forge::Request`, appended:

```rust
    /// Send one kept act to its taker (root: Cancel to the core as the deployment's act;
    /// RemoveGroup and EraseSecrets to serve as `root::Request::{RemoveGroup, EraseSecrets}`).
    SendKeptAct(KeptAct),
    /// The playground's start was decided: make its project, account and adoption.
    PlaygroundStart { request: Token },
    /// The reset's act of its current step, to the core as the deployment's act.
    ResetAct(ResetAct),
```

```rust
/// A reset step the core takes (contracts T6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResetAct { RemoveProject { project: u32 }, RemoveAccount { account: u32 } }
```

`forge::Event`, appended:

```rust
    /// The core answered a kept `Cancel`; `keep` while it must be sent again.
    KeptActAnswered { key: [u8; 16], keep: bool },
    /// The core is ready after a start: send every kept act and the reset's act again.
    SendKeptActs,
    /// The playground's project was made.
    PlaygroundProject { project: u32 },
    /// The playground's scripted account was added.
    PlaygroundAccount { account: u32 },
    /// The playground's start ended, with its answer.
    PlaygroundAnswered { answer: StartAnswer },
    /// The core answered the reset's act; `keep` while it must be sent again.
    ResetActed { act: ResetAct, keep: bool },
```

`forge::Domain` gains `pub fn playground(&self) -> Option<&Playground>` (the restored or
saved row; `None` until `fe-playground-forge` keeps one).

## 6. The store schema at version 11 (contracts T2)

`temper/crates/temper-protocol-store/schema/store-v10.schema` is renamed
`store-v11.schema` (`git mv`), its first line `family temper-store 11`; the generated module
`src/generated/v10.rs` becomes `src/generated/v11.rs`, `pub mod v11`, and every `v10` path
in Rust names `v11` (`STORE_VERSION` 11). After the `ReceiptRow` record, before `enum Value`:

```
# Family 31: a forge the deployment keeps (draft playground-reset.md, section 3).
enum ForgeKind { forgejo, github }
enum ForgeMode { connected, managed, owned, fake }
record ApiTls { server_name: bytes 255 trust: option bytes 4096 }
enum ApiTransport { plaintext, tls: ApiTls }
record ForgeApi { host: bytes 255 port: u16 transport: ApiTransport }
record ForgeBot { login: bytes 255 name: bytes 255 email: bytes 254 }
record FirstExisting { project: u32 }
record FirstNew { name: bytes 1024 }
enum FirstProject { existing: FirstExisting, new: FirstNew }
enum FirstCreateStep { admin, org, repository, file }
record FirstMadeId { id: u64 }
record FirstFailedStep { step: FirstCreateStep }
enum FirstState { making, made: FirstMadeId, failed: FirstFailedStep }
record ForgeFirst { playground: bool org: bytes 1024 repository: bytes 1024 project: FirstProject state: FirstState label: bytes 64 }
enum ForgeState { kept, removing }
record Forge { number: u16 kind: ForgeKind mode: ForgeMode api: ForgeApi git: bytes 1024 git_program: option bytes 4096 web: option bytes 1024 read_only: bool bot: ForgeBot setup: option bytes 4096 first: option ForgeFirst playground: bool state: ForgeState }
# Family 32: the next forge number to give; absent is 1.
record ForgeNumbers { next: u16 }
# Family 33: the playground (draft playground-reset.md, 6.1 and 6.2).
enum ResetStep { project, forge, account }
record PlaygroundResetting { step: ResetStep }
enum PlaygroundState { running, resetting: PlaygroundResetting }
record Playground { state: PlaygroundState forge: u16 project: option u32 account: option u32 }
# Family 34: a removal's acts, kept until answered (draft playground-reset.md, section 4).
record KeptCancel { project: u32 task: u64 }
enum KeptActKind { cancel: KeptCancel, remove_group, erase_secrets }
record KeptAct { key: fixed 16 forge: u16 act: KeptActKind }
```

`enum Value` appends, in this order: `forge: Forge, forge_numbers: ForgeNumbers, playground:
Playground, kept_act: KeptAct`. The decoder's bounds take every new field's ceiling (no
`Bounds` field derives from `forge::Limits`).

Keys (`src/keys.rs`), constants beside `FLIGHT`: `FORGE: u8 = 31`, `FORGE_NUMBERS: u8 = 32`,
`PLAYGROUND: u8 = 33`, `KEPT_ACT: u8 = 34`. `Key::Forge(n)` is `tag(31).u64(n)`;
`ForgeNumbers` is `tag(32)` alone; `Playground` is `tag(33)` alone; `KeptAct(k)` is
`tag(34).bytes(k)`, decoded only at 16 bytes. Decoding refuses a forge number past `u16`,
a key of another width and trailing bytes, as the existing families do.

## 7. Until each shape has its meaning

`fe-surface` decides each new request as a refusal or a no-op:

- `MakeForge` answers `MakeAnswer::Refused(MakeRefusal::NotOffered)`; `RemoveForge`
  answers `RemoveAnswer::Refused(RemoveRefusal::NoSuchForge)`; `EndPlayground` answers
  `EndAnswer::NotOffered`; each kept as a connect's answer is (`connect::State::answers`),
  with `Request::ApplicationDone`, and any carried value answered `CarriedUnused`.
- `StartPlayground` is today's `StartDemo`, renamed: the demo's start, unchanged.
- The connector emits none of `RemoveGroup`, `EraseSecrets`, `Register`, `Unregister`,
  `SendKeptAct`, `PlaygroundStart`, `ResetAct`; `SetUpGroup`'s `entry` is empty
  (`Box::new([])`). The root routes each of the four serve requests through its now door
  (as `StopGroup`); it takes `SendKeptAct`, `PlaygroundStart` and `ResetAct` as no-ops.
- Serve (`temper-service/src/route.rs`) takes `RemoveGroup`, `EraseSecrets`, `Register`
  and `Unregister` as no-ops and ignores `SetUpGroup`'s `entry`, until sv's tasks.
- The connector takes `GroupRemoved`, `SecretsErased`, `KeptActAnswered`, `SendKeptActs`,
  `PlaygroundProject`, `PlaygroundAccount`, `PlaygroundAnswered` and `ResetActed` as no-ops,
  and restores the four new rows as no-ops (their room checks and references accept them).
