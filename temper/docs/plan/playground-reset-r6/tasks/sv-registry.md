+++
id = "sv-registry"
part = "fs"
kind = "feature"
needs = ["sv-startup-machine", "sv-skein-connection", "sv-skein-setup"]
files = ["temper/crates/temper-service/**", "temper/crates/temper-protocol-records/src/forge.rs"]
effort = "high"
minutes = 120
+++

## Goal

serve keeps its forges as a registry that changes at runtime: the root's
`Request::Register(ForgeRecord)` adds or updates a forge, `Request::Unregister { forge }`
removes it (`notes/contracts.md`, T5), and every reader of serve's forges reads the
registry. The document's forges still seed it at start (the transition of
`notes/sv-progress.md`, request 1), so nothing a deployment does today changes; a record
registered at restore is in the registry before the forge records load. This task is the
forge connection's half; the setup host's half is `sv-registry-setup`.

Acceptance:
- `engine.workers.forges` (today a `Box<[settings::Forge]>` fixed at start) is a registry
  that `Register`/`Unregister` change; every reader listed below reads it as today.
- `Register(record)` of a number not held: the forge enters the registry (its
  `settings::Forge` built from the record, below), the forge connection holds it
  (`skein_forge_connection::Request::Add`, no access: its access comes from its records),
  the records loader keeps its rows, its git identity's room exists, and for GitHub the
  tokens' table has its entry. Of a number held: its record's fields replace the held ones
  (`mode`, `web`, `read_only`, `git`, `git_program`, the bot); the connection is not
  touched (kind, API and transport of a record never change; an update that changes them
  is ignored and noted, `Note::ForgeRecordChanged { number }`).
- `Unregister { forge }`: the connection removes it (`Request::Remove`), the registry, the
  records loader's rows, the git identity's values, the tokens' entry and the setup
  lifecycle's per-forge maps (`adoptions`, `reconcile`, `deleting`, `installations`,
  `deleted`) drop it. Nothing of it is read afterwards; a root request naming it is
  answered as for a forge never configured.
- A record whose API host is an IP literal with `ApiTransport::Plaintext` is reachable at
  once (`skein_io::Addr::Ip`); any other (a host name, or TLS) is held: added with
  `forge::HELD_ADDRESS` and kept in the layer's held map, so it connects nothing and is
  named `Note::ForgeNotConnected { Unreachable { host } }` exactly as an unresolved
  document host is today. `sv-resolve` resolves them.
- At start, the root's restore registers each record before the forge records load
  (`forge/records.rs`'s `Phase::Load`), and the loader's `Phase::Connect` walks the
  registry: a registered GitHub record connects from its records as a GitHub forge does
  today; a registered Forgejo record with a setup entry (owned, fake, and a Forgejo
  elsewhere, whose entry is managed: `notes/contracts.md`, T4) is left to the setup
  lifecycle, as a records-only entry is today (`sv-registry-setup` gives it its entry); a
  registered Forgejo record with none connects from its records (`records::access` with
  `Kind::Forgejo`). Each is named `ForgeNotConnected` when its records are missing or
  refused.
- With no `Register` sent (every deployment until fe's records path lands), every existing
  temper-service test passes unchanged.

## Inputs

- `temper/docs/design/protocol/forge.md`, 2.2 to 2.6 (connections, records at start);
  `temper/docs/design/protocol/credentials.md`, 2.1 and 4 (records, git identities).
- Draft `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T1, T5 (`Register`,
  `Unregister`), T7.
- `temper/docs/plan/playground-reset-r6/notes/sv-progress.md`, Contract requests 1 and 2.
- skein's runtime forges: `skein_forge_connection::Request::{Add, Remove}` and
  `Component::admits` (`sv-skein-connection`).

## Owns and may change

- Owns: `temper/crates/temper-service/` for this task: new module `src/forges.rs`, and the
  edits below in `forge.rs`, `forge/records.rs`, `forge/tokens.rs`, `values.rs`,
  `workers.rs`, `engine.rs`, `system.rs`, `route.rs`, `setup/lifecycle.rs` (the per-forge
  maps' removal only), and their tests.
- `temper/crates/temper-protocol-records/src/forge.rs`: `Rows::drop_forge(&mut self, forge:
  u16)` (drop a forge's values), with a unit test, if the service needs it there.
- Readers that change only their argument (a slice of the registry in place of the boxed
  slice): `translate.rs::items`, `assignment.rs::down`, `family.rs::{to_root, panel,
  document}`, `clients.rs::{advance, iterate, ...}`, `setup/routes.rs`, `setup/lifecycle.rs`
  (`configured`, `advance`, `restored`, `take_read`, `take_removal`), `route.rs`.
- Not in this task: the setup host's forges (`setup/settings.rs`, `sv-registry-setup`),
  the document's parsing (`startup/`), the binary.
- Pre-approved options:
  - Where the registry lives. Default: `Workers.forges` becomes `List<settings::Forge>`
    with capacity `limits.forge_component.forges` (every reader keeps reading
    `engine.workers.forges`, as a slice), and `src/forges.rs` holds the records beside it
    in `engine.records: Map<u16, temper_engine_domain_forge::ForgeRecord>` (capacity
    `forge_component.forges`) and the functions `register(engine, record)` and
    `unregister(engine, forge)`. If moving the list out of `Workers` into `engine.forges`
    reads better, do it and say so.
  - The record's `settings::Forge` (one function, `forges::settings(record:
    &ForgeRecord) -> settings::Forge`): `mode`, `number`, `kind` (`Kind::Forgejo` /
    `GitHub`), `api` (`Address::Addr(Ip)` for an IP literal host, else
    `Address::Host { name, port }`), `transport` (`Plaintext`, or `Tls { server_name,
    trust }` with `trust` the record's path), `git`, `git_program`, `web`, `app_read_only`
    = `read_only`, `api_token: None`, `webhook_secret: None`, `user: 0`, `identity: forge
    as u32` (T1). Its git identity is `settings::Identity { number: forge as u32, forge,
    token: empty, login: bot.login, author: { bot.name, bot.email } }`, kept where
    `Tokens` (GitHub) and the setup lifecycle (`sv-registry-setup`) read it.
  - The identity values' table (`values.identities`) and `Tokens` are sized by the
    document's identities today; size both by `limits.forge_component.forges` (the
    document's forges never exceed it: the connection component refuses more at start).
  - A number registered that the document also configures (transition only: fe-records
    may seed the document's forges as records and register them at each start,
    `notes/sv-progress.md`, request 5) is the document's forge: only the record's `mode` is
    taken, silently, and nothing else of serve changes (no `Add`, no tokens or identity
    entry: the document's are kept). Default as said; fe's `MakeForge` never makes such a
    number while the document names forges (request 1).
  - New `Note` variants are allowed (`Note::ForgeRecordChanged { number: u16 }`); the
    binary's note printer (`temper/crates/temper/src/host.rs::note_line`) gains its line,
    compiler-forced.

## Steps

1. `src/forges.rs` (`//! The deployment's forges at runtime (draft playground-reset.md,
   section 3; notes contracts T5).`):
   - `pub(crate) fn settings(record: &ForgeRecord) -> settings::Forge` and `fn
     identity(record: &ForgeRecord) -> settings::Identity` as above.
   - `pub(crate) fn register(engine: &mut Engine, record: ForgeRecord)`: new or update as
     the acceptance says. New: `engine.forge.add(&settings, endpoint)` (below), the records
     loader's allowlist (`engine.forge_records.rows.register(number)`), `Tokens::add` for
     GitHub, the record kept in `engine.records`.
   - `pub(crate) fn unregister(engine: &mut Engine, forge: u16)`: the reverse, as the
     acceptance says; the identity's values: `values.identities` gains `remove(number)`.
2. `forge.rs`: `Layer::add(&mut self, forge: &settings::Forge, endpoint: Endpoint)` builds
   the `skein_forge_connection::settings::Forge` as `settings()` builds one today (`access:
   None`, `log_origins` empty), checks it with `Component::admits` (a refusal noted,
   `Note::ForgeNotConnected { number, why: NotConnected::Unreachable { host } }` with the
   API's host text, and nothing added), sends `Request::Add`; `Endpoint` is the address and
   transport (`Reachable(SocketAddr)` or `Held { host }`; `sv-resolve` adds the resolved
   form). `Layer::remove(forge)` sends `Request::Remove` and drops its `held` entry.
   `Layer::new`'s `held` map gets capacity `limits.forges`.
3. `route.rs`: `root::Request::Register(record)` → `crate::forges::register`;
   `root::Request::Unregister { forge }` → `crate::forges::unregister` (fe-surface left
   them as no-ops).
4. `forge/records.rs`: `Rows` keeps a `Map<u16, ()>` of registered numbers in place of
   the fixed `configured` list (`register`, `unregister`: the latter drops the forge's
   values); `Loader::new` seeds it from the document's forges; `Phase::Connect` walks the
   registry by index as today: a registered GitHub record, or a Forgejo record with no
   setup entry, connects from its records (`records::access(kind, &records::origin(kind,
   &forge.api), rows, ..)`), a missing or refused one noted `ForgeNotConnected`; a Forgejo
   record with a setup entry is skipped, as `engine.setup.records_only(number)` is today. Restored
   records are registered before `Phase::Load` begins: check it holds (the root's restore
   runs before the store's durable mark, `engine.rs::iterate`); if it does not, make
   `Phase::Load` wait for `engine.domain.ready()` and say so.
5. `forge/tokens.rs`: `Tokens::new` sized by `forge_component.forges`; `Tokens::add(forge,
   identity: &settings::Identity, installations: u32)` and `Tokens::remove(forge)`.
6. Tests in `src/tests/registry.rs` (registered in `src/tests.rs`), on `fixtures::started()`
   and the existing forge fixtures, each driving `route::request` with `Register` /
   `Unregister` and a `ForgeRecord` fixture (new `fixtures::record(number, kind, mode,
   api)`):
   - `a_registered_forgejo_record_is_held_by_the_connection_and_read_by_the_documents`
     (`engine.forge.component.start(n)` is `Some`; `family::document` and
     `translate::items` find its `git`)
   - `a_registered_record_with_a_host_name_is_held_and_named_unreachable`
   - `a_registered_github_record_has_its_tokens_entry_and_identity_room`
   - `an_update_replaces_the_mode_and_web_and_keeps_the_connection`
   - `an_update_changing_the_api_is_ignored_and_noted`
   - `unregister_drops_the_forge_from_every_table` (a call for it afterwards answers
     `Unavailable` from the component; its records' rows gone; `values.identities` has no
     value for `forge as u32`)
   - `records_restored_before_the_load_connect_a_forgejo_record_from_its_records`
     (records seeded as `tests/forge.rs` seeds them; the loader's `Connect` phase sends a
     `Connect` with the records' access)
   - `the_document_forges_seed_the_registry_unchanged` (the fixture engine's forges are
     the registry's, in order)
7. Narrow runs, one at a time:
   `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::registry::/)'` (8
   tests). Then temper-service whole (`heavy cargo nextest run -p temper-service`), and the
   packages that read serve's forges through it: `heavy cargo nextest run -p
   temper-one-process-world`, `-p temper-real-loop-world`, `-p temper-system-world`; and
   `heavy cargo check --workspace --tests` once (the binary and other readers).
8. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper-protocol-records
   --all-targets`; `python3 -I -B scripts/old_forms.py` from `temper/` (never raise a
   count; `--write` if you removed old forms). Commit and push.

## Report

- Where the registry lives (option taken) and each reader's file.
- Whether restored records were registered before `Phase::Load` without a wait.
- Under `## Left`: readers of the document's forges that remain (they go in
  `sv-startup-refuse`).
