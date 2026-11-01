+++
id = "store-families"
part = "forge-records"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-protocol-store/**",
  "temper/crates/temper-engine-domain-forge/src/record.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/limits.rs",
  "temper/crates/temper-engine-domain-forge/src/restore.rs",
  "temper/crates/temper-engine-domain-forge/src/reconcile.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/records.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs",
]
effort = "high"
minutes = 60
+++

## Goal

temper's store moves to `STORE_VERSION` 11 with the four families of the
run's contracts, section 2: `Forge` (25), `Forges` (26), `Playground`
(27) and `Kept` (28), their domain types, keys, records, schema and
golden bytes, and restore arms that keep the rows and give them back,
with no behaviour: nothing writes these rows yet, and no decision reads
them. `Demo` (17) and `DemoAct` (19) stay as they are.

Acceptance:

- `temper_protocol_store::STORE_VERSION == 11`; the schema is
  `schema/store-v11.schema` (`family temper-store 11`), the generated
  module `v11` (`src/generated/v11.rs`); `v10` and its schema are gone.
  A store at 10 refuses to open as every move refuses (the existing
  version check; no new code).
- Each new row round-trips through `records::encode` / `records::row`
  under its key; each new key encodes, decodes and orders after every
  family below it; a disagreeing key is refused.
- The forge connector restores each new row (`Event::Restore`) and gives
  it back through the accessors below; restore fit refuses a `Forge` or
  `Kept` row past its table, as it refuses a `DemoAct` row.
- `Configured::from_record` gives, for a record carrying what the
  document carries today, the `Configured` that
  `temper-service`'s `startup::application::configured` builds now
  (`temper/crates/temper-service/src/startup/application.rs`, `configured`).
- The drift test passes with the regenerated code and goldens; every
  package that depends on the two crates still builds its tests.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 1,
  D2 and D3, and 2, whole).
- The draft, `temper/docs/design/draft/playground-reset.md`, sections 3 and 4.
- `temper/docs/design/protocol/store.md`, section 2 (the families) and
  section 4 (the version).
- `temper/docs/design/protocol/setup.md`, section 8 (the setup entry's
  members, which `record::Setup` mirrors).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s1-forges-document.md`,
  sections 1 ("Forge and identity fields", "Setup entries") and 6
  ("Adding a family", and its `Flight` example's file list, which is
  this task's shape).

## Owns and may change

Owned: `temper/crates/temper-protocol-store/` whole (schema, generated
code, `keys.rs`, `records.rs` and its modules, goldens, its tests); in
`temper/crates/temper-engine-domain-forge/src/`: the new `record.rs`,
`boundary.rs` (the new `Stored`/`Key` variants and the `Playground`,
`PlaygroundState`, `Kept`, `KeptAct` types), `lib.rs` (`stored_key`,
`stored_bytes`, the module and its re-exports), `limits.rs`
(`kept_bound`), `restore.rs`, `reconcile.rs`, `domain.rs` (four fields,
their restore arms, accessors; nothing else), `tests.rs` (one module
line), the new `tests/records.rs`, `tests/restore_fit.rs`.

Neighbours, and the edits allowed there:

- `temper/crates/temper-fake/src/check.rs`: `v10` → `v11` and the new
  `Stored` arms, beside `Demo`/`DemoAct`'s.
- `temper/tests/codecs/`: `drift.rs`'s store version `"v10"` → `"v11"`;
  `store_queue.rs`, `store_protection.rs`: the version asserts 10 → 11
  and the `v10` module path → `v11` (rename the test whose name says
  `v10` to say `v11`); a fuzzy store generator that enumerates the
  families (`tests/suite/fuzzy/store.rs`) gains the four, else leave it.
- Any file whose `match` on `Stored`, `Key` or `RecordKind` the compiler
  stops: the new arm only, doing what `DemoAct`'s arm beside it does
  (no link, no behaviour). Tests asserting `STORE_VERSION == 10`
  elsewhere: the literal only.

Pre-approved options:

- A schema bound below names one field's bound; if the generator or a
  golden needs a different spelling (a one-field record for a list of
  byte strings, as `Segment` is), use it and say so in the report; keep
  the field names and their order.
- If a fixture enumerating key families by index
  (`temper-protocol-store/src/tests/fixtures.rs`) needs renumbering to
  add the four, renumber it.
- If `stored_bytes`' sum for `Forge` would need a helper, write it in
  `record.rs`.

Not here: any save or erase of the new rows, the seed, numbering, the
domain's forges from records (`configured` stays the constructor's), any
reader of `Playground` or `Kept`, `temper/docs/`.

## Steps

1. **Domain types, `temper-engine-domain-forge`** (new `pub mod record`
   in `src/record.rs`, cited at module level to the draft, 3, and the
   contracts, 2; `pub use record::Forge;` at the crate root). Each type
   `#[derive(Clone, PartialEq, Eq, Debug)]`. The names are binding:

   ```rust
   /// One forge the connector keeps (draft 3; contracts, 2), what the
   /// document's `forges[]`, `identities[]` and `setup[]` held for it,
   /// less its secrets.
   pub struct Forge {
       pub number: u16,
       pub kind: crate::Kind,
       pub api: Address,
       pub transport: Transport,
       /// The base of each remote (forges[].git).
       pub git: Box<[u8]>,
       pub git_program: Option<Box<[u8]>>,
       pub web: Option<Box<[u8]>>,
       /// forges[].app_read_only.
       pub read_only: bool,
       pub identity: Identity,
       /// The file form's names, none for a records-only forge.
       pub files: Option<Files>,
       /// The forge's setup entry, none for a connected forge.
       pub setup: Option<Setup>,
       /// The first repository's intent, kept until it is made.
       pub first: Option<FirstIntent>,
       /// The playground's fake forge alone.
       pub playground: bool,
       /// Ended, its kept acts not all answered (D3).
       pub removing: bool,
   }
   pub enum Address { Socket(core::net::SocketAddr), Host { name: Box<[u8]>, port: u16 } }
   pub enum Transport { Plaintext, Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> } }
   /// identities[]: the git identity's number and its login and author,
   /// each empty where the document's is null (the bot's defaults).
   pub struct Identity { pub number: u32, pub login: Box<[u8]>, pub name: Box<[u8]>, pub email: Box<[u8]> }
   /// The file form's names (never the files' bytes): forges[].api_token,
   /// webhook_secret, user, and identities[].token.
   pub struct Files {
       pub api_token: Box<[u8]>,
       pub webhook_secret: Option<Box<[u8]>>,
       pub user: FileUser,
       pub git_token: Option<Box<[u8]>>,
   }
   pub enum FileUser { Id(u64), File(Box<[u8]>) }
   /// One setup[] entry less its `forge` (protocol/setup.md, 8), as
   /// `temper_forge_setup::setup::Entry` holds it, paths as resolved.
   pub struct Setup {
       pub directory: Box<[u8]>,
       pub bot: Account,
       pub orgs: Box<[Box<[u8]>]>,
       pub mode: SetupMode,
       /// `"start": "asked"`.
       pub asked: bool,
   }
   pub struct Account { pub login: Box<[u8]>, pub email: Box<[u8]> }
   pub enum SetupMode { Managed { admin_token: Option<Box<[u8]>> }, Owned(OwnedEntry), Fake(FakeEntry) }
   pub struct OwnedEntry {
       pub public: Option<Origin>,
       pub fetch: Fetch,
       pub forgejo: Box<[u8]>,
       pub runner: Option<Runner>,
       pub listen: core::net::SocketAddr,
       pub person: Account,
   }
   pub struct Fetch { pub forgejo: bool, pub runner: bool, pub mirror: Option<Mirror> }
   pub struct Mirror { pub forgejo: Origin, pub runner: Origin }
   pub struct Origin { pub host: Box<[u8]>, pub port: u16 }
   pub struct Runner { pub binary: Box<[u8]>, pub labels: Box<[Box<[u8]>]>, pub capacity: u32, pub path: Box<[u8]> }
   pub struct FakeEntry {
       pub listen: core::net::SocketAddr,
       pub git: Box<[u8]>,
       pub owner: Box<[u8]>,
       pub repositories: Box<[FakeRepository]>,
   }
   pub struct FakeRepository { pub name: Box<[u8]>, pub landing: Box<[u8]> }
   /// `crate::First` less its volatile state, and less `playground`,
   /// which the forge's own flag holds.
   pub struct FirstIntent { pub org: Box<[u8]>, pub repository: Box<[u8]>, pub project: crate::SetUpProject, pub label: Box<[u8]> }
   ```

   Also in `record.rs`:
   - `impl Forge { pub fn fits(&self) -> bool }`: every field within the
     schema's bounds (step 3; the constants it needs are `record.rs`'s
     `pub const`s). Whoever makes a row checks it first (later tasks).
   - `impl crate::Configured { pub fn from_record(forge: &Forge) -> Self }`,
     total: the body of `startup::application::configured` for one forge
     (`temper/crates/temper-service/src/startup/application.rs`, from
     `pub(crate) fn configured`): `mode` Connected without a setup entry,
     else Managed / Owned / Fake by `SetupMode`; `admin_connect` for
     Managed with no `admin_token`; `bot` the entry's bot login, else
     empty; `owned` for Owned, `forgejo` and `runner` from
     `skein_forge::forgejo::pin`, `password` the entry's directory
     joined with `/person_password` (`temper-forge-setup`'s
     `place::File::PersonPassword`); `asked` the entry's `asked` or
     `playground` (today's demo forge is asked); `git` empty when longer
     than `PULL_ADDRESS_BYTES`; `web`, `read_only`, `number`, `kind`.
     Nothing calls it yet but the tests.

2. **Rows and keys, `boundary.rs`**, as the contracts' section 2 gives
   them: `Stored::Forge(record::Forge)`, `Stored::Forges { next: u16 }`
   (an inline variant: `boundary::Forges` is the subject's type and
   stays), `Stored::Playground(Playground)`, `Stored::Kept(Kept)`;
   `Key::Forge(u16)`, `Key::Forges`, `Key::Playground`,
   `Key::Kept([u8; 16])`; `pub struct Playground { pub state:
   PlaygroundState, pub forge: Option<u16>, pub project: Option<u32>,
   pub account: Option<u32> }`, `pub enum PlaygroundState { Running,
   Resetting }`, `pub struct Kept { pub key: [u8; 16], pub act: KeptAct }`
   and `KeptAct` exactly as the contracts write it (with their doc
   comments); `Clone, Copy, PartialEq, Eq, Debug` on the last four.
   `RecordKind` gains `Forge`, `Forges`, `Playground`, `Kept`.
   `lib.rs`: `stored_key` (`Forge(row) => Key::Forge(row.number)`, the
   singletons to their unit keys, `Kept(row) => Key::Kept(row.key)`);
   `stored_bytes`: `Forge` the sum of its byte fields' lengths, the
   other three `Some(0)`.

3. **Schema, `temper-protocol-store`.** `git mv` the schema to
   `schema/store-v11.schema`, first line `family temper-store 11`, the
   module path in `lib.rs` to `generated/v11.rs`, `pub mod v11`, every
   `v10` use in the crate and its neighbours to `v11`,
   `STORE_VERSION = 11`. Append after the receipts, before `Value`:

   ```
   # Families 25 to 28: forges kept in the store, the playground and kept
   # acts (temper/docs/design/draft/playground-reset.md, 3, 4 and 6).
   enum ForgeKind { forgejo, github }
   record ForgeIp4 { ip: fixed 4 port: u16 }
   record ForgeIp6 { ip: fixed 16 port: u16 }
   enum ForgeSocket { ip4: ForgeIp4, ip6: ForgeIp6 }
   record ForgeHost { name: bytes 255 port: u16 }
   enum ForgeAddress { socket: ForgeSocket, host: ForgeHost }
   record ForgeTls { server_name: bytes 255 trust: option bytes 4096 }
   enum ForgeTransport { plaintext, tls: ForgeTls }
   record ForgeIdentity { number: u32 login: bytes 255 name: bytes 255 email: bytes 255 }
   record ForgeUserId { id: u64 }
   record ForgeUserFile { file: bytes 4096 }
   enum ForgeUser { id: ForgeUserId, file: ForgeUserFile }
   record ForgeFiles { api_token: bytes 4096 webhook_secret: option bytes 4096 user: ForgeUser git_token: option bytes 4096 }
   record SetupAccount { login: bytes 255 email: bytes 255 }
   record SetupName { name: bytes 4096 }
   record SetupOrigin { host: bytes 255 port: u16 }
   record SetupMirror { forgejo: SetupOrigin runner: SetupOrigin }
   record SetupFetch { forgejo: bool runner: bool mirror: option SetupMirror }
   record SetupRunner { binary: bytes 4096 labels: list 8 SetupName capacity: u32 path: bytes 4096 }
   record SetupOwned { public: option SetupOrigin fetch: SetupFetch forgejo: bytes 4096 runner: option SetupRunner listen: ForgeSocket person: SetupAccount }
   record SetupRepository { name: bytes 4096 landing: bytes 4096 }
   record SetupFake { listen: ForgeSocket git: bytes 4096 owner: bytes 4096 repositories: list 64 SetupRepository }
   record SetupManaged { admin_token: option bytes 4096 }
   enum SetupMode { managed: SetupManaged, owned: SetupOwned, fake: SetupFake }
   record ForgeSetup { directory: bytes 4096 bot: SetupAccount orgs: list 1024 SetupName mode: SetupMode asked: bool }
   record FirstExisting { project: u32 }
   record FirstNew { name: bytes 1024 }
   enum FirstProject { existing: FirstExisting, new: FirstNew }
   record FirstIntent { org: bytes 1024 repository: bytes 1024 project: FirstProject label: bytes 64 }
   record Forge { number: u16 kind: ForgeKind api: ForgeAddress transport: ForgeTransport git: bytes 4096 git_program: option bytes 4096 web: option bytes 4096 read_only: bool identity: ForgeIdentity files: option ForgeFiles setup: option ForgeSetup first: option FirstIntent playground: bool removing: bool }
   record Forges { next: u16 }
   enum PlaygroundState { running, resetting }
   record Playground { state: PlaygroundState forge: option u16 project: option u32 account: option u32 }
   record KeptCancel { project: u32 task: u64 }
   record KeptProject { project: u32 }
   record KeptAccount { account: u32 }
   record KeptForge { forge: u16 }
   enum KeptAct { cancel: KeptCancel, remove_project: KeptProject, remove_account: KeptAccount, box_remove: KeptForge, remove_forge: KeptForge }
   record Kept { key: fixed 16 act: KeptAct }
   ```

   and `Value` gains, at its end, `forge: boxed Forge, forges: Forges,
   playground: Playground, kept: Kept`. A `SocketAddrV6`'s flow and
   scope are not kept (decoded as 0). The new fields take the schema's
   ceilings (no `records::limits` entry) unless a test shows a bound
   must follow a forge limit.

   `keys.rs`: `pub const FORGE: u8 = 25; FORGES: u8 = 26;
   PLAYGROUND: u8 = 27; KEPT: u8 = 28;` with a doc line each;
   `Key::Forge(n)` writes `tag(FORGE).u64(n)`, `Key::Forges` and
   `Key::Playground` their tag alone, `Key::Kept(key)` `tag(KEPT).bytes(key)`
   as `DemoAct`; decode refuses a forge number past `u16` and a kept key
   not 16 bytes. `keys::largest` still bounds every key.
   `records.rs`: the encode and decode arms, total, in a new
   `records/forge.rs` module.
   Regenerate the code and goldens by the drift test's write mode
   (`temper/tests/codecs/tests/suite/drift.rs`, its header:
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world
   --test suite -E 'test(drift::)'`); never by hand.

4. **Restore, `domain.rs`, `restore.rs`, `reconcile.rs`, `limits.rs`.**
   - `limits.rs`: `pub const fn kept_bound(l: &Limits) -> Option<u32>`:
     `demo_act_bound(l)` plus two per forge (a `BoxRemove` and a
     `RemoveForge`) plus two (`RemoveProject`, `RemoveAccount`), the
     forges counted by `l.client.repositories`, the bound that sizes
     `configured` today (`domain.rs`, `Domain::new`); `valid` refuses
     limits where it overflows. Re-export it beside `demo_act_bound`.
   - `Domain` gains `forge_records: Map<u16, crate::Forge>` (capacity
     `l.client.repositories`), `next_forge: Option<u16>`,
     `playground: Option<crate::Playground>`,
     `kept: Map<[u8; 16], crate::KeptAct>` (capacity `kept_bound`).
     `restore` inserts each row; `restore::row` checks room for `Forge`
     and `Kept` as for `DemoAct` (`Rows(RecordKind::Forge)`,
     `Rows(RecordKind::Kept)`), the singletons `Ok(())`;
     `reconcile`'s `references` takes no link for any of the four (the
     `DemoAct` arm's comment says why); the position helper near the
     end of `domain.rs` returns `None` for them, as for `DemoAct`.
   - Accessors, `pub`, on `Domain`, each with a one-line doc:
     `forge_record(&self, number: u16) -> Option<&crate::Forge>`,
     `forge_records(&self) -> impl Iterator<Item = &crate::Forge>` (in
     number order), `next_forge(&self) -> u16` (the row's `next`, 1 when
     absent: contracts, 2), `playground(&self) ->
     Option<&crate::Playground>`, `kept(&self) -> impl Iterator<Item =
     (&[u8; 16], &crate::KeptAct)>`.

5. **Tests** (the codec's unit tests and the connector's unit tests;
   the codecs world for drift):
   - `temper-protocol-store/src/tests/records.rs`:
     `forge_rows_round_trip_with_their_keys`: a smallest `Forge`
     (GitHub, `Host` address, TLS, no files, setup, first) and a full one
     (owned setup with public origin, mirror, runner with two labels,
     files with a `FileUser::File`, a `FirstIntent` with
     `SetUpProject::New`, `playground` and `removing` true, an IPv6
     socket), each through `encode` then `row` under `Key::Forge`, equal;
     the full one under `Key::Forge(number + 1)` is refused.
     `fake_and_managed_setups_round_trip`: a fake entry with two
     repositories, a managed entry with and without `admin_token`.
     `singleton_and_kept_rows_round_trip`: `Forges { next: u16::MAX }`;
     `Playground` in each state with every option none and every option
     at its maximum; a `Kept` per `KeptAct` variant at the maxima, key
     `[255; 16]`.
   - `tests/keys.rs`: `forge_keys_follow_receipts_in_order`: the four
     keys encode, decode to themselves, and sort after the largest
     `Receipt` key and before nothing else; `Key::Forge` orders by
     number; a 15-byte kept key is refused. `tests/fixtures.rs`' random
     keys draw the four.
   - `tests/sizes.rs`: the version assert to 11, and `Forge`'s and
     `Kept`'s encoded bounds are `Some`.
   - `temper-engine-domain-forge/src/tests/records.rs` (new, `mod
     records;` in `tests.rs`): `restored_rows_are_given_back`: restore a
     `Forge`, `Forges { next: 4 }`, a `Playground` and two `Kept` rows,
     then the accessors return them, `next_forge() == 4`;
     `next_forge_is_one_without_its_row`;
     `configured_from_record_follows_each_mode`: records for a GitHub
     forge (no setup), a fake forge with `playground` true, an owned one
     asked with a runner and a managed one without an admin token, each
     against the `Configured` literal today's service builds for the
     template's forges (`temper/crates/temper-init/template/temper.json`);
     `fits_refuses_a_field_past_its_bound`.
   - `tests/restore_fit.rs`: its table of row bounds gains
     `(K::Forge, CLIENT.repositories)` and `(K::Kept,
     kept_bound(&LIMITS).unwrap())`.
   - Narrow runs: `heavy cargo nextest run -p temper-protocol-store
     --lib`; `heavy cargo nextest run -p temper-engine-domain-forge --lib
     -E 'test(records::) | test(restore_fit::)'`; the drift test and
     `-E 'test(store_)'` in `temper-codecs-world`'s `suite`; `fuzzy` for
     the store codec's fuzzy test alone if it gained the families.

6. `cargo fmt`; `heavy cargo clippy -p temper-protocol-store -p
   temper-engine-domain-forge -p temper-fake --all-targets -- -D
   warnings`; the focused tests of those packages; `heavy cargo check
   --tests` for each direct dependent: `temper-engine-domain`,
   `temper-service`, `temper`, `temper-codecs-world`,
   `temper-engine-domain-world`, `temper-engine-forge-world`,
   `temper-engine-protocol-forge-world`, `temper-one-process-world`,
   `temper-real-loop-world`, `temper-system-world`,
   `temper-conformance-world`, `temper-client-world`,
   `temper-worker-domain`, `temper-engine-smith`, `temper-github-app`.
   Commit (the rename, then the families, then the tests, or as you
   find clearest) and push.

## Report

fab's sections, and `## Interfaces`: every public item added or changed
(path and signature), the schema's new records as written, the key
tags, the accessors and `kept_bound`'s formula. Say which compiler-forced
arms you added outside the owned files, and any bound you changed from
step 3.
