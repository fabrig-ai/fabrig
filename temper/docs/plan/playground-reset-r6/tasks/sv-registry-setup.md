+++
id = "sv-registry-setup"
part = "fs"
kind = "feature"
needs = ["sv-registry", "bx-steps"]
files = ["temper/crates/temper-service/**"]
effort = "high"
minutes = 110
+++

## Goal

The setup host's half of serve's registry (`sv-registry`): a registered record is set up,
reconciled and connected as a document forge with its setup entry is today, from the
record's own setup text (`notes/contracts.md`, T1, T3, T5). `Register` adds the forge to
skein's setup host (`host::Request::Add`) and, for a record with a setup entry, makes its
local reconcile entry; `Unregister` removes both.

Acceptance:
- `Register` of a new record whose endpoint is reachable (`sv-registry`'s rule): the setup
  host holds it (`host::Request::Add`, checked first with `Host::admits`; a refusal noted
  `Note::ForgeSetupStopped { number, why: SetupStopped::Missing }` and the forge kept in
  `unserved` with the reason), with the `Given` today's `setup/settings.rs::fixed` builds
  for its kind:
  - Forgejo with a setup entry (owned, fake, or a Forgejo elsewhere, whose entry is
    `{"bot": {"login": "temper", "email": "temper@localhost"}, "mode": "managed"}`, no
    admin token: `notes/contracts.md`, T4): its `Entry` from
    `temper_forge_setup::setup::Entry::body_text(setup, forge, ..)` (bx-steps),
    `temper_forge_setup::given::values(&entry, listener, &limits)` as `forgejo()` does
    (listener: the webhooks listener's bound address, as `setup.rs` fills pending ones,
    unless the mode is managed admin-connect);
  - Forgejo with none (not made by the connector; kept for safety): the constant `Given`
    of `forgejo()`'s second half, with `bot_login`/`bot_email` from the record's `bot`;
  - GitHub: the GitHub `Given` as `fixed()` builds it, `read_only` from the record.
  A held endpoint is not added: the forge is `unserved` with the reason `b"unreachable"`.
- A record with a setup entry gets a `lifecycle::Reconcile::Local(Local { .. })` as
  `local_starts` builds one for a records-only entry: `source` by its mode (`Managed` for a
  managed body, else `Prepared`), `repositories` the fake's count, `handover: None`,
  `path` the entry's `admin_token` place, `trigger` `StartDemo` for a playground record,
  `Start` for a managed entry (it has no box group), `StartAsked` for another under the box,
  `Start` otherwise; `limits`, `webhooks`, `bot_login` and
  `identity` from the record (`sv-registry`'s identity). Registered at restore it is
  `pending: true` and starts when the lifecycle is restored, as today; registered after the
  restore (a forge made at runtime) it is `restored: true, pending: false`, its reconcile
  waiting for its group's `started` (the box link's `Out::Started`, `lifecycle::started`),
  or, for a managed entry, for the admin connect.
- An update (`Register` of a held number) replaces the host's forge (`Remove` then `Add`)
  only when its `Given` changed (GitHub's `read_only`, `web`); a run of the forge then live
  ends `Down` as the host's `Remove` ends it, and the lifecycle treats that stop as any
  host stop. The `Local` keeps its state; its `bot_login` and `identity` follow the record.
- `Unregister`: the host removes the forge (`host::Request::Remove`), its runs in
  `engine.setup.runs` and the setup layer's per-forge tables (`served`, `unserved`,
  `file_api`, `pastes`, `erases` naming it, `backs`, staged and public callers) drop it; a
  host `Stop` arriving for a run of an unregistered forge is ignored.
- The serve setup layer keeps what `fixed` reads of `settings::Engine` (the clients'
  public origin or listener, the webhooks listener, the limits) so it can build a `Given`
  after start; and `settings::Engine` gains `directories: jig_protocol_config::Directories`
  (filled by startup where it already has them), kept by the layer for `Entry::body_text`.
- A `Register` of a number the document configures (the transition's seeded records,
  `sv-registry`'s rule) changes nothing of the setup host or the lifecycle: the document's
  entry, directory and `Local` stay.
- With no `Register` sent, every existing temper-service test passes unchanged.

## Inputs

- `temper/docs/design/protocol/setup.md`, 5.1 (the host's forges), 6.3 to 6.7 (the local
  reconcile, its triggers and the first repository), 8 (the setup entry), 8.4.
- Draft `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.4.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T1, T3, T5, T7.
- `temper/docs/plan/playground-reset-r6/tasks/bx-steps.md` (the names `Entry::body_text`,
  `directory`; their report's final signatures if they differ).

## Owns and may change

- Owns, in `temper/crates/temper-service/`: `src/setup.rs`, `src/setup/settings.rs`,
  `src/setup/lifecycle.rs`, `src/setup/public.rs`, `src/forges.rs` (its `register` and
  `unregister` call the setup half), `src/settings.rs` (`Engine::directories`), the
  startup lines that fill it (`src/startup.rs`), and their tests.
- Fixtures building `settings::Engine` literally gain the field (compiler-forced:
  `src/tests/fixtures.rs` and any world that builds one; rg `settings::Engine {`).
- Pre-approved options:
  - Factor `fixed`'s per-forge body into `fn host_forge(statics: &Statics, forge:
    &settings::Forge, entry: Option<&Entry>, endpoint: (Addr, host::Transport)) ->
    Result<host::Forge, Problems>` used at start and on `Register`, so the two cannot drift
    (`Statics` is the kept subset of the settings). Default as said.
  - The constant `Given` for a Forgejo record with no entry: if
    `skein_forge_setup::given::check` refuses the record's bot as `bot_login`, keep temper's
    constants as today and say so.

## Steps

1. `src/setup/settings.rs`: `Statics` and `host_forge` (options); `fixed` uses them;
   `pub(crate) fn local(record: &ForgeRecord, entry: &Entry, statics: &Statics, under_box:
   bool, restored: bool) -> lifecycle::Local`, the one `local_starts` builds today for a
   records-only entry, used by `local_starts` and by `Register`.
2. `src/setup.rs`: `Layer::add(&mut self, forge: host::Forge)` and `Layer::remove(forge)`
   sending `host::Request::Add`/`Remove` to the component (or to `pending` before the host
   starts, as the start-time forges wait for the webhooks bind).
3. `src/forges.rs`: `register` calls the setup half after the connection half; `unregister`
   before it.
4. Tests in `src/tests/registry_setup.rs` (registered in `src/tests.rs`), on the existing
   setup fixtures (`src/tests/setup*.rs`) with `fixtures::record`:
   - `a_registered_owned_record_is_served_and_waits_for_its_group`
   - `a_registered_owned_record_reconciles_after_its_group_started` (`Out::Started` for it;
     the handover read, `TakeOver`, `Reconcile` as `tests/setup_lifecycle.rs` drives a
     records-only entry)
   - `a_record_restored_with_records_reconciles_at_start` (registered at restore,
     `AdminToken` and `Origin` records present: `Reconcile` without a group start)
   - `a_registered_fake_record_is_a_playground_and_waits_for_its_start` (`StartDemo`
     trigger: `NotSetUp` until asked)
   - `a_registered_managed_record_admin_connects` (a Forgejo elsewhere's record with T4's
     managed entry: `Source::Managed`, trigger `Start`; `Connect { How::ForgejoAdmin }` runs
     `Reconcile` with the entry's bot)
   - `a_registered_github_record_runs_a_manifest` (as `tests/setup_github.rs`)
   - `an_update_of_read_only_replaces_the_hosts_github_forge`
   - `unregister_ends_a_live_run_and_drops_the_forge_from_every_setup_table`
   - `a_held_record_is_unserved_unreachable`
5. Narrow runs, one at a time:
   `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::registry_setup::/)'`
   (9 tests). Then `heavy cargo nextest run -p temper-service`; `heavy cargo nextest run -p
   temper-one-process-world`; `heavy cargo check --workspace --tests`.
6. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets`; old forms (`python3 -I
   -B scripts/old_forms.py` in `temper/`). Commit and push.

## Report

- `Statics`' members, and whether the managed constant `Given` took the record's bot.
- Under `## Left`: the start-time paths that read `settings.setup` and `records_only`
  (they go in `sv-startup-refuse`).
