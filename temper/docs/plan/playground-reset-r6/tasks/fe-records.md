+++
id = "fe-records"
part = "fs"
kind = "feature"
needs = ["fe-surface"]
files = [
  "temper/crates/temper-engine-domain-forge/**",
  "temper/crates/temper-engine-domain/**",
  "temper/tests/engine/**",
]
effort = "high"
minutes = 110
+++

## Goal

The forge connector keeps forges as records in the store (draft
`temper/docs/design/draft/playground-reset.md`, section 3; `notes/contracts.md` T1, T4),
beside the document's configured forges while those still exist: it restores `Forge` and
`ForgeNumbers` rows, projects `Configured` from each `Kept` record, gives a record's forge
the git identity of its number, sends `Register` for every restored record, and treats a
`Removing` record, or a number with no record that the document does not name, as ended.
The machine's templates reach the connector. Additive: every deployment and world that
configures forges through the document runs as today (`notes/sv-progress.md`, contract
request 1, its step (a)); `fe-config-drop` removes the document's path later.

Acceptance:
- `forge::Domain::new(l, seed, config, identities, forges, machine: Machine, under_box)`:
  `machine` added after `forges` and kept for `fe-make`; the root passes `config.machine`.
- A restored `Forge` row is held; `configured(n)`, `forges()`, `offer(n)` and every reader
  of the configured list read the union of the document's `Configured` forges and the
  projections of `Kept` records (a record and a document forge of one number: the record's
  projection wins).
- `ForgeRecord::configured(&self) -> Configured` (pub) is the projection: `admin_connect`
  = `kind == Forgejo && mode == Managed`; `bot` = `bot.login` when `setup` is `Some`, else
  empty; `owned` = for `Mode::Owned`, `Some(Owned { forgejo: pin::VERSION, runner:
  Some(pin::RUNNER_VERSION), password: b"forge-N/person_password" })`
  (`skein_forge::forgejo::pin`; the password path relative to the data directory, see
  `notes/fe-progress.md`, contract request 1), else `None`; `git` = `git` when it fits
  `PULL_ADDRESS_BYTES`, else empty; `web`, `read_only`, `number`, `kind`, `mode` copied;
  `asked` = `setup.is_some()`.
- A record's forge (`Kept` or `Removing`) has the git identity `GitIdentity { forge,
  credential: forge as u32 }` unless the document's identities name that forge.
- `forge_ended(n)` is true when record `n` is `Removing`; when no record `n` exists, `n`
  was given (below `ForgeNumbers`' next number, which is never past a number given) and
  the document names `n` neither among its forges nor its identities (a removed forge's);
  or (as today) for the demo's ended forge. A number never given and never configured is
  not ended (worlds and tests that use forge numbers without configuring them run as
  today). A removed forge's repository row restores and reconciles, is in no list, and
  stays readable by its tag.
- Once restoration ends (`Event::Restored`), before `RestartDone { Restored }`, the
  connector sends `Request::Register(record)` for every restored record, `Kept` and
  `Removing`, in number order; the root routes each to serve (`root::Request::Register`).
- Restore fits: at most `l.forges` `Forge` rows (`RestoreSet::Rows(RecordKind::Forge)`),
  one `ForgeNumbers` (kept as `next_number`, absent meaning 1, for `fe-make`);
  `l.forges == 0` is invalid in `valid`.
- `forge::Event::ReleaseProject { project: u32 }` and `forge::Request::ProjectReleased {
  project: u32 }` exist (fe's vocabulary for contracts J5, documented as
  `notes/fe-surface.md` section 5 documents its own); the connector answers every
  `ReleaseProject` with `ProjectReleased` at once, erasing nothing yet (`fe-make` gives it
  its erasures; `fe-kept-root` routes it). The root takes `ProjectReleased` as a no-op arm.
- Every existing test of the owned packages passes unchanged in meaning; the new tests
  below pass.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T1, T2, T4,
  T5 (`Register`), J5; `notes/fe-surface.md` (all); `notes/sv-progress.md`, contract
  request 1.
- `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/design/domain/forge.md`, 4.1 (the configured forges, the ended forge).
- `temper/docs/design/domain/root.md`, 9.1 (restoration and reconciliation).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/`, `temper/crates/temper-engine-domain/`,
  `temper/tests/engine/domain/` (`temper-engine-domain-world`), `temper/tests/engine/forge/`,
  `temper/tests/engine/protocol-forge/`.
- May change: any file the compiler names because `forge::Domain::new` gained `machine`
  (none is expected outside the owned crates).
- Pre-approved options:
  - Tables: default, `records: Map<u16, ForgeRecord>` (sized `l.forges`) beside the
    existing `configured: Map<u16, Configured>`, which holds the document's forges and the
    records' projections, kept in step by one helper that every record change goes
    through (`fe-make` and `fe-remove` call it); `configured`'s capacity becomes
    `l.client.repositories + l.forges`. If one map of records with the projection computed
    on read is simpler, take it and say so.
  - A restored record's long git base is projected as an empty `git`, as temper-service's
    `configured` did; it is not refused.
  - Reconciliation (`reconcile.rs`): `Need::Forge(n)` resolves for a record `n` (any state)
    or a document forge or identity; `Need::Repository(r)` resolves as today, and also when
    the repository row exists and its forge is a removed forge's by the rule above.
  - `Register` door: route each through `super::now` with an assert, as `StopGroup` is,
    and reserve its room as the root reserves the connector's other immediate requests
    (find how `"demo box group door reserved"` is guaranteed); if restoration can send more
    `Register`s at once than the door holds (up to `l.forges`), raise the door's reservation
    by `l.forges` where the root sizes it, and say so.
  - The demo is unchanged: `DemoConfig.adoption.forge` keeps naming the demo's forge.

## Steps

1. `src/boundary.rs` (forge): `ForgeRecord::configured`; the two J5 variants.
2. `src/domain.rs` (forge): `Domain::new`'s `machine`; restore `Stored::Forge` and
   `Stored::ForgeNumbers`; the record helper; `identities` readers (`items.rs`,
   `adoption.rs`, `reconcile.rs`) fall back to the number rule for records; `restored()`
   sends `Register`; `ReleaseProject` answered at once. `forge_ended` (in `src/demo.rs`
   today; move it to `domain.rs` if clearer) gains the record rules; `retire_ended` runs
   whenever some forge is ended, not only the demo's.
3. `src/restore.rs`, `src/reconcile.rs`, `src/limits.rs` (forge): the fits and references
   above; `reconcile::capacity` adds `l.forges` (a record's Writer).
4. Root (`temper/crates/temper-engine-domain/`): `domain.rs` passes `&config.machine` (or a
   clone) at both `forge::Domain::new` sites; `route/forge_route.rs`'s `ProjectReleased`
   arm (comment: "fe-kept-root answers the core").
5. Forge unit tests: a fixture in `src/tests/fixtures.rs`, `pub(crate) fn
   restore_records(d: &mut Domain, records: &[ForgeRecord])`, restoring each row through
   `Event::Restore` then `Event::Restored`. New tests in a new `src/tests/records.rs`:
   `a_kept_record_projects_its_configured_forge` (each mode; Owned's relative password;
   `asked` from the setup entry; a long git base projects empty);
   `records_list_beside_the_documents_forges_and_win_a_shared_number`;
   `removing_and_removed_forges_are_ended_and_their_repositories_stay_readable_by_tag`;
   `restoration_registers_every_record_in_number_order_before_restored`;
   `more_forge_rows_than_the_limit_do_not_fit`;
   `a_records_identity_is_its_number_unless_the_document_names_one`;
   `release_project_is_answered_at_once`.
6. Engine world: a new module `temper/tests/engine/domain/src/forges.rs` (`pub mod forges`
   in `lib.rs`), for this task's and later tasks' stories:

   ```rust
   /// A record for world fixtures: api `forge-N.test:443` over TLS, git and web
   /// `https://forge-N.test`, bot `temper`/`Temper`/`temper@localhost`, a setup entry
   /// `{"bot":{"login":"temper","email":"temper@localhost"}}` for Owned and Fake, none
   /// otherwise; `Kept`, not the playground's, no first repository.
   pub fn record(number: u16, kind: forge::Kind, mode: forge::Mode) -> forge::ForgeRecord;
   /// The record whose projection is `configured` (kind, mode, git, web, read-only; a setup
   /// entry when it was asked or named a bot).
   pub fn from_configured(configured: &forge::Configured) -> forge::ForgeRecord;
   /// Each record's `Forge` row and one `ForgeNumbers` past the highest number.
   pub fn rows(records: &[forge::ForgeRecord]) -> Box<[temper_engine_domain::Record]>;
   ```

   One story in `tests/suite/reconcile.rs` (or a new `tests/suite/records.rs`):
   `a_seeded_forge_record_is_listed_and_registered_after_a_restart` (seed a record with
   `Store::seed`, crash, cold start: `forges()` lists it and `immediate` holds its
   `Register`).
7. Run, one at a time:
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib`
   - `heavy cargo nextest run -p temper-engine-domain --lib`
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite`
   - `heavy cargo nextest run -p temper-engine-forge-world`
   - `heavy cargo nextest run -p temper-engine-protocol-forge-world`
   - `heavy cargo nextest run -p temper-service --lib` (it reads the connector's rows)
   - `heavy cargo check --workspace --all-targets`
   - `cargo fmt --all`; `heavy cargo clippy -p temper-engine-domain-forge -p
     temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
8. Commit, push your branch.

## Report

- The tables you chose and how the door's room for `Register` is guaranteed.
