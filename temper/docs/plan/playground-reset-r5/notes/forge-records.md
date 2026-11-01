# Part note: `forge-records` and `remove-forge` (playground-reset-r5)

The shapes inside parts `forge-records` (`fr-*`) and `remove-forge`
(`rf-*`) that several of their tasks share. It adds to
`notes/contracts.md` and never changes it. Draft:
`temper/docs/design/draft/playground-reset.md`, sections 3, 4 and 6.4.
Removal's own steps are in `notes/remove-forge.md`.

## 0. How the part moves: transition, then cut-over

The document's forges are read by about 30 production functions and a few
hundred test sites. So the part moves in three stages, each green:

1. **Records beside the document.** Every layer gains the record path
   while the document's forges still work: the connector keeps both its
   configured forges (`Domain::new`'s `forges`) and its records in the one
   `configured` map; the service configures a forge at runtime from a
   record as well as at start from the document; the box runs groups from
   group files as well as from the document's steps. The parser accepts a
   document whose `application.forges`, `identities` and `setup` are empty
   or absent. While the document names forges, `make_forge` never takes a
   configured number (section 3).
2. **Migrations.** Each crate's and world's fixtures move from document
   or typed forges to records (section 7), one task per crate or world.
3. **Cut-over** (`fr-out-shell`, then `fr-out-service`, which is
   `notes/common.md`'s `forge-doc-out`): the document's forge members, the
   typed start-time forges, `Domain::new`'s `forges` and `identities`,
   `Config.forges` and every reader of them go whole. After it the
   document names no forge.

No transitional *reader* survives stage 3; nothing imports the document's
forges into records.

## 1. The record (`temper_engine_domain_forge::boundary`, task `fr-family`)

```rust
/// A forge's record, kept by the forge connector (draft, section 3; contracts, 2).
pub struct ForgeRecord {
    pub forge: u16,
    pub kind: Kind,
    pub mode: Mode,
    /// Today's `Configured` fields, with the same meaning.
    pub admin_connect: bool,
    pub bot: Box<[u8]>,
    pub owned: Option<Owned>,
    pub git: Box<[u8]>,
    pub web: Option<Box<[u8]>>,
    pub read_only: bool,
    pub asked: bool,
    /// The document entries, as the service's parsers read them; the domain never reads them.
    pub entries: Entries,
    /// The first repository's intent, kept until it is adopted (draft, 3).
    pub first: Option<FirstIntent>,
    /// Set by `remove_forge` (notes/remove-forge.md); `None` while kept.
    pub removal: Option<Removal>,
}
/// One forge's members of today's document, JSON, without their numbers.
pub struct Entries {
    /// An `application.forges[]` entry without `number` (and without `identity`).
    pub forge: Box<[u8]>,
    /// An `application.identities[]` entry without `number` and `forge`.
    pub identity: Box<[u8]>,
    /// An `application.setup[]` entry without `forge` and `directory`, if the forge has one.
    pub setup: Option<Box<[u8]>>,
}
/// Today's volatile `First`, its state left out: what a restart keeps.
pub struct FirstIntent {
    pub playground: bool,
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub project: SetUpProject,
    pub label: Box<[u8]>,
}
/// What the service filled for a forge to make, from the person's choice and
/// the machine's members: the record's fields but its number, first and removal.
pub struct Template {
    pub kind: Kind,
    pub mode: Mode,
    pub admin_connect: bool,
    pub bot: Box<[u8]>,
    pub owned: Option<Owned>,
    pub git: Box<[u8]>,
    pub web: Option<Box<[u8]>>,
    pub read_only: bool,
    pub asked: bool,
    pub entries: Entries,
}
/// `make_forge`'s input (contracts, 3): the choice and what it is made of.
pub enum NewForge {
    /// The playground's fake forge, from the template the service gave at start
    /// (`Domain::configure_playground`): the engine makes it without a client act.
    Playground,
    /// Owned, GitHub or a Forgejo elsewhere, filled by the service's translation
    /// of the client's act (its kind and mode name the choice).
    Filled(Template),
}
/// A `SetUp` or `Connect` names a forge not yet kept, or a kept one to go on with.
pub enum ForgeChoice {
    New(NewForge),
    Kept(u16),
}
pub enum MakeRefusal {
    /// `forge.forges` records are kept (removing ones count).
    Full,
    /// The machine's one port of this mode is a kept forge's: one owned and one
    /// fake forge at most, removing ones not counted.
    Taken,
    /// No playground template: the machine has no fake forge (`NewForge::Playground`).
    NotOffered,
}
```

- **The choice by kind and mode:** Playground `(Forgejo, Fake)`, Owned
  `(Forgejo, Owned)`, GitHub `(GitHub, Connected)`, a Forgejo elsewhere
  `(Forgejo, Managed)` with `admin_connect: true`. A connected Forgejo
  (bot token pasted) has no client act today and gets none.
- `ForgeRecord::new(forge: u16, template: Template) -> ForgeRecord`
  (`first: None`, `removal: None`) and `ForgeRecord::configured(&self) ->
  Configured` (the connector's in-memory row, `number: forge`). The
  record's identity is number `forge` (one identity per forge): its
  `GitIdentity` is `{ forge, credential: u32::from(forge) }`.
- `Stored::Forge(ForgeRecord)`, `Key::Forge(u16)`; `Stored::ForgeNext(u16)`,
  `Key::ForgeNext`. Codec (`temper-protocol-store`): family bytes 31
  and 32 (`keys::FORGE`, `keys::FORGE_NEXT`), `Key::Forge` the byte then
  the number widened to eight bytes, `Key::ForgeNext` the byte alone.
  Schema records `ForgeRecord`, `ForgeEntries` (`forge: bytes 2048
  identity: bytes 1024 setup: option bytes 4096`), `FirstIntent`,
  `Removal` (notes/remove-forge.md), `ForgeNext { next: u16 }`.
  `STORE_VERSION` stays 10 (temper's `protocol/store.md`, 4: in place
  until the first release); schema, generated code and goldens change in
  place.
- **Bounds:** `forge_component::Limits`-free: the connector's
  `Limits.forges: u32` bounds the `Forge` family; `ForgeNext` is one row.
  The entries' byte bounds are the schema's above
  (`ENTRY_FORGE_BYTES = 2048`, `ENTRY_IDENTITY_BYTES = 1024`,
  `ENTRY_SETUP_BYTES = 4096` in the domain's `limits.rs`); the service
  refuses a fill past them.

## 2. The connector's requests and entries

- `Request::ForgeKept { record: ForgeRecord }` (forge domain, then the
  root's `Request::ForgeKept { record }`): the service configures the
  forge from it (section 5). Emitted after `make_forge`'s decision, for
  each restored record without a removal in `Restored`'s handling before
  `RestartDone { Restored }`, and whenever a record is saved again
  (its first intent made or cleared). The service treats a second
  `ForgeKept` for a configured number as an update of its first intent
  alone.
- `Domain::configure_playground(&mut self, template: Option<Template>)`:
  before the first input, as `configure_demo` is; the root passes
  `Config.playground` to it.
- `Domain::make_forge(&mut self, new: NewForge, out: &mut Queue<Request>)
  -> Result<u16, MakeRefusal>` (`fr-domain`): in the caller's decision,
  takes `ForgeNext` (no row: 1, or past the largest configured number while
  the document names forges), saves `Stored::Forge` and
  `Stored::ForgeNext(n + 1)`, inserts its `Configured` and connection state
  (`NotSetUp` for owned and fake, `NotConnected` otherwise), emits
  `ForgeKept`, and for a box kind (owned, fake) under the box nothing more:
  the box is asked by the `SetUp` (owned) or the playground's start
  (`SetUpGroup`) as today. Numbers are never reused, so `ForgeNext`
  never falls, removal or not.
- `Ask::SetUp { forge: ForgeChoice, .. }` and `Ask::Connect { forge:
  ForgeChoice, how }`: `New` makes the forge in the ask's decision, then
  goes on as today with its number; `Kept(n)` is today's `forge: n`.
  Refusals: `SetUpRefusal::Make(MakeRefusal)` and
  `ForgeRefusal::Make(MakeRefusal)` beside today's.
- **The first intent kept:** `SetUp` (and an admin `Connect`'s `first`)
  saves the record with `first: Some(intent)` in its decision; at restart
  `Domain.first` is rebuilt from it with `FirstState::Making`, and the
  service, told by `ForgeKept`, makes the repository at the forge's next
  connect (setup.md, 6.7, as amended by the draft, 3). The record is saved
  with `first: None` in the decision that adopts that forge's repository
  named by the intent (or removes the forge). The engine domain world's
  `first_repository_row_is_volatile_across_restart` turns into its
  opposite, `first_repository_intent_survives_a_restart` (`fr-domain`).

## 3. The service's machine member and its fills (task `fr-machine`)

`application.machine`, the machine's facts for forges made at runtime
(contracts, 5); `temper init` writes it. The clients' and webhooks'
listeners stay where they are (`application.clients.listener`,
`application.webhooks.listener`).

```json
"machine": {
  "bot": {"login": "temper", "name": "Temper", "email": "temper@localhost"},
  "github": {"api": null, "transport": null, "git": null, "web": null},
  "owned": {
    "listen": {"socket": "$owned"},
    "forgejo": "fetched",
    "runner": {"binary": "fetched", "labels": ["temper-host:host"], "capacity": 1, "path": "/usr/local/bin:/usr/bin:/bin"},
    "person": {"login": "$login", "email": "$email"}
  },
  "fake": {
    "listen": {"socket": "$forge"},
    "git": "demo/bin/git",
    "owner": "playground",
    "repositories": [{"name": "widget", "landing": "main"}]
  }
}
```

- Every member but `bot` may be `null`: no owned Forgejo here, no
  playground. `github`'s members are today's forge entry's (defaults
  `api.github.com:443`, TLS, `https://github.com`); `--fake-backends`
  rewrites them as it rewrites forge 2 today.
- **A box forge's directory is `forge-N`** under data, for owned and fake
  alike (no member): the setup entry's `directory` is filled from the
  number wherever the entry is read with its number (the group file, the
  service's parse).
- `temper_service::machine` (new module): `parse(member) -> Machine`;
  `playground(&Machine) -> Option<Template>`; `fill(&Machine, choice:
  &wire NewForge) -> Result<Template, Problem>` (owned from `owned`; GitHub
  from `github` with the person's `read_only`; a Forgejo elsewhere from the
  person's address: `http://` only on loopback, plaintext; `https://`
  TLS on the machine's roots at port 443 or the URL's; git and API base
  the address; `bot` the machine's; setup entry `{"bot": .., "mode":
  "managed"}`); `parse_record(&ForgeRecord, &Machine) -> Result<Parsed,
  Problems>` with `Parsed { forge: settings::Forge, identity:
  settings::Identity-without-token, setup: Option<settings::Setup> }`,
  through today's entry parsers given the number; and its inverse for
  fixtures, `record_of(&settings::Forge, &settings::Identity,
  Option<&settings::Setup>) -> ForgeRecord`, the test kits' one builder
  (section 7).
- Tokens never come from entries: a record's forge has
  `api_token: None`, `webhook_secret: None`, and its credentials are its
  secret records (credentials.md, 2.1), as records-only forms are today.

## 4. The group file (`<data>/box/groups/N`, task `fr-group`)

Written by serve (through the shell) before `start N`, mode 0600, `box`
and `box/groups` 0700; read by the box at its start and at `start N`, and
by the forge's commands (`temper forge prepare|run|reconcile N`, `temper
fake forge`). Codec: `temper_forge_setup::group`.

```json
{"forge": 4, "kind": "owned", "setup": { "forge": 4, "directory": "forge-4", "bot": {..}, "start": "asked", "mode": {"owned": {..}} }}
{"forge": 1, "kind": "fake", "llm": true, "setup": { "forge": 1, "directory": "forge-1", "bot": {..}, "mode": {"fake": {..}} }}
```

- `setup` is the whole `application.setup[]` entry as today's
  `temper_forge_setup::setup` parser reads it (number and directory
  filled); its mode's `listen` is the group's listener, which the box
  holds from its start (the machine's `owned.listen`, `fake.listen`).
- `llm` (fake only): the playground's group, its first step the fake LLM
  (today's demo group: fake llm, forge prepare, fake forge, label
  `Playground`, not recorded). A fake group without `llm` is today's
  non-demo fake forge.
- `temper_forge_setup::group::{GroupFile, GroupKind, encode, decode}`:
  `GroupFile { forge: u16, kind: GroupKind, entry: setup::Entry }`,
  `GroupKind::{Owned, Fake { llm: bool }}`; `decode(bytes, &Directories)
  -> Result<GroupFile, Problem>` refuses a number disagreeing with the
  entry's, an owned kind with a non-owned mode, and trailing bytes;
  bounded by `GROUP_BYTES = 8192`.
- Steps by kind: `temper_forge_setup::step::group(&GroupFile, &Context)
  -> Group` for owned (today's `steps` for one owned entry, `recorded:
  true`); `temper_fake::steps::group(&GroupFile, &Context) -> Group` for
  fake (today's demo group, or a fake forge's two steps, `recorded:
  false`). `Context` is what `steps` takes today (program, directories).

## 5. The service's runtime forges (`fr-svc-forges`, `fr-svc-route`)

- **Tables sized from `forge.forges`:** the forge component's forges, the
  setup host's, tokens, record rows, the workers' forge table, the
  connector's `connection_limits` all take `forge.forges` (new limit,
  `temper-service/src/limits.rs` `Forges.forges`, name `forge.forges`,
  small profile 4) in place of the configured count, and
  `forge_component.forges` goes (it equals `forge.forges`).
- **Configure at runtime:** `Engine::keep_forge(parsed: Parsed, runtime:
  ForgeRuntime)` adds the forge to every table: skein's
  `skein_forge_connection::Request::Add`, `skein_forge_setup::Request::Add`
  (tasks `fr-sk-connection`, `fr-sk-setup`), tokens, rows (credential rows
  of a number not yet kept are held, not refused, until the restart's
  `Restored`, then reported as today's unconfigured rows), workers.
- **The shell's part, drained as handover reads are:**
  `Service::take_forge_runtime() -> Option<ForgeAsk { forge, api:
  jig_protocol_config::Address, transport: startup::Transport }>` and
  `Service::forge_runtime(forge, Result<ForgeRuntime, Box<[u8]>>)` (the
  shell's `forges()` resolution, one forge); `Service::take_group_write()
  -> Option<GroupWrite { forge, bytes }>` and `Service::group_written(forge,
  Result<(), Box<[u8]>>)`. A `SetUp` or `SetUpGroup` for a box kind writes
  the group file (again, if written before), then sends `start N`.
- **The client's choice:** `family::to_root` translates wire
  `ForgeChoice::New` through `machine::fill` into `NewForge::Filled`; a
  fill refused answers `set_up_refused { invalid }` / `connect_refused`
  as an invalid request is answered.

## 6. The wire (`temper-client-protocol-wire`, task `fr-wire`)

```
record NewGitHub { read_only: bool }
record NewForgejo { address: bytes 2048 }
enum NewForge { owned, github: NewGitHub, forgejo: NewForgejo }
record KeptForge { forge: u16 }
record NewForgeChoice { forge: NewForge }
enum ForgeChoice { new: NewForgeChoice, kept: KeptForge }
record SetUp { forge: ForgeChoice org: bytes 1024 repository: bytes 1024 project: SetUpProject playground: bool }
record Connect { forge: ForgeChoice how: How }
record RemoveForge { forge: u16 }
enum Request { ..., remove_forge: RemoveForge }             # after pg-wire's
enum MakeRefusal { full, taken, not_offered }
record MakeRefused { why: MakeRefusal }
record Removing { forge: u16 }
enum RemoveRefusal { no_such_forge, role }
record RemoveRefused { why: RemoveRefusal }
enum Outcome { ..., make_refused: MakeRefused, removing: Removing, remove_refused: RemoveRefused }
```

A schema enum's variant holds a record, never an enum (skein's
`enum_payloads_must_be_records`), so `new` holds `NewForgeChoice`, whose
`forge` is the `NewForge`: in Rust, `ForgeChoice::New(NewForgeChoice {
forge: NewForge::Owned })`. Elsewhere in these notes a wire
`ForgeChoice::New(choice)` means `choice.forge`.

`Outcome` gains, after pg-wire's `reset`: `make_refused: MakeRefused`
(a `SetUp` or `Connect` whose forge could not be made), `removing:
Removing` (`RemoveForge` decided: the forge is ended and its removal kept),
`remove_refused: RemoveRefused`. A `SetUp` or `Connect` that made a forge
answers as today (`setting_up`, `connecting`, `link`); the new forge's
number reaches the client in the Forges watch (its row appears), and the
`setting_up` / `connecting` outcomes gain nothing. `ForgeRow` is
unchanged: a removing forge has no row.

## 7. Fixtures: the one builder

- Domain-level worlds (no service): a record is
  `ForgeRecord::new(n, template)` with `Entries` empty (`b"{}"`), restored
  through the world's store or its `Event::Restore { record:
  Stored::Forge(..) }` before `Restored`, with a `Stored::ForgeNext` row
  past it.
- Service tests and worlds that run the service: `machine::record_of`
  from the typed fixture they have today, its rows committed into the
  fixture store (`fixtures::commit`) before the restart, and its secret
  rows (`ApiToken`, `User`, `GitToken`...) as records-only fixtures seed
  them today.
- Process tests (the `temper` binary): the forge made by the act a person
  sends (`temper ctl forges set-up --new owned`, `connect --new github
  ...`, task `fr-ctl`), or, for a seeded GitHub App, its record seeded with
  its secrets by `temper-github-app`'s store seeding (`fr-mig-temper-gh`).
