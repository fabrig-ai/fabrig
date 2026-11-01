+++
id = "fr-domain"
part = "forge-records"
kind = "feature"
needs = ["fr-family"]
files = ["temper/crates/temper-engine-domain-forge/src/**"]
effort = "high"
minutes = 120
+++

## Goal

The forge connector keeps forges as records (draft
`temper/docs/design/draft/playground-reset.md`, section 3): `make_forge`
numbers and saves one, restored records become configured forges, a
`SetUp` or `Connect` naming a new forge makes it in its decision, and the
first repository's intent is kept in the record across a restart. The
document's configured forges (`Domain::new`'s `forges`) keep working
beside the records until the cut-over (`notes/forge-records.md`,
section 0).

Acceptance:

- `Domain::make_forge(&mut self, new: NewForge, out: &mut Queue<Request>)
  -> Result<u16, MakeRefusal>` as `notes/forge-records.md`, section 2
  says: refused `Full` at `Limits.forges` records (removing ones count),
  `Taken` while a record or configured forge of mode `Owned` (for an owned
  template) or `Fake` (for a fake one) is kept without a removal,
  `NotOffered` for `NewForge::Playground` with no template
  (`configure_playground(None)`). Its number: `ForgeNext`'s, or with no
  row 1, raised past every configured (document) number; saved with
  `Stored::Forge` and `Stored::ForgeNext(n + 1)` through `Request::Save`;
  inserted into `configured` (`ForgeRecord::configured`), its git identity
  (`GitIdentity { forge, credential: u32::from(forge) }`) into the
  identity table, its connection `NotSetUp` (owned, fake) or
  `NotConnected`; `Request::ForgeKept { record }` and
  `Request::MarkedForges` emitted.
- **Restore:** at `Event::Restored`, each kept record without a removal
  joins `configured`, identities and connections as above, its
  `first: Some(intent)` rebuilt into `Domain.first` with
  `FirstState::Making`, and `ForgeKept` emitted, before
  `RestartDone { Restored }`. A record with a removal stays out of
  `configured` (`rf-domain` gives it its meaning). A record whose number
  is also a configured (document) forge's is a restore misfit, as an
  unknown row is.
- **Asks:** `Ask::SetUp { forge: ForgeChoice, .. }` and `Ask::Connect {
  forge: ForgeChoice, how }`. `Kept(n)` is today's path unchanged. `New`:
  the ask's own checks (names, project, busy, role) first; then
  `make_forge` in the same decision, its refusal answered
  `SetUpRefusal::Make(r)` / `ForgeRefusal::Make(r)` (new variants); then
  today's path on the new number (an owned record's `asked` makes its
  offer `Offered` under the box). `New(NewForge::Playground)` from a
  client ask is refused `Invalid` (the playground is made by its start
  alone).
- **The first intent kept:** a `SetUp`, and an admin `Connect` with a
  `first`, on a record forge saves the record with `first: Some(intent)`
  in its decision (and emits `ForgeKept`); the decision that adopts the
  intent's repository on that forge (`adoption.rs`'s finish), saves it
  with `first: None`. Document forges keep today's volatile intent.
- **Sizing:** `configured`, `first`, connection and identity tables have
  room for the document's forges plus `Limits.forges` records.
- `Domain::forges()` lists record forges as configured ones (number
  order), so the Forges watch shows a made forge at once.
- `Ask::StartDemo` and the demo are untouched (`fr-demo` moves the demo's
  forge to a record).
- Tests (the crate's library tests, a new `src/tests/records.rs` on
  `src/tests/fixtures.rs`): `make_forge_numbers_from_one_saves_both_rows_and_tells_the_service`,
  `make_forge_numbers_past_the_documents_forges_and_never_reuses_a_number`,
  `make_forge_refuses_full_taken_and_a_playground_without_template`,
  `restored_records_are_configured_and_told_before_restored_is_done`,
  `a_set_up_naming_a_new_owned_forge_makes_it_and_asks_the_box`,
  `a_refused_set_up_makes_no_forge`,
  `a_connect_naming_a_new_github_forge_makes_it_and_connects`,
  `first_repository_intent_survives_a_restart`,
  `the_adopted_first_repository_clears_the_kept_intent`. Every existing
  test of the crate passes.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (sections 2
  and 3), `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 0 to 2).
- `temper/docs/design/domain/forge.md`, 4.1 (the forges' rows, offers);
  `temper/docs/design/protocol/setup.md`, 6.7 (the first repository).
- Surveys `temper/docs/plan/playground-reset-r5/notes/surveys/s1b-connector-store-acts.md`
  (sections 1, 3 and 4: every `configured` reader in the crate) and
  `s5-setup-welcome.md`, section 1 (the first intent's readers).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/src/**` (one writer of
  the crate for this wave).
- May change, forced only: the construction of `Ask::SetUp` and
  `Ask::Connect` (`forge: ForgeChoice::Kept(n)`) and the arms over
  `SetUpRefusal` / `ForgeRefusal` in `temper-engine-domain` (root tests
  and `route/applications.rs`), `temper-service/src/family.rs` (the new
  refusals answered as `set_up_refused { invalid }` /
  `connect_refused { not_taken }` with a comment `// fr-svc-route answers
  make_refused`) and the worlds that build those asks
  (`temper/tests/client`, `temper/tests/engine/domain`,
  `temper/tests/system`). `rg 'Ask::SetUp \{|Ask::Connect \{'` finds them.
- Options: if a `Connect`'s new GitHub forge cannot be made before the
  ask's role and carried-key checks without reordering `connect::start`,
  make it after them and say so. If the record's identity number clashes
  with a document identity's number, refuse the restore as a misfit; the
  cut-over removes the case.

## Steps

1. `make_forge`, numbering, tables' room.
2. Restore into `configured`; `ForgeKept` at `Restored`.
3. `ForgeChoice` in `SetUp` and `Connect`; the refusals; forced edits
   outside.
4. The kept intent.
5. Tests. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge
   --lib -E 'test(/records/)'`.
6. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain --all-targets -- -D warnings`; focused tests of
   `temper-engine-domain-forge` and `temper-engine-domain`; `heavy cargo
   check --tests` for `temper-service`, `temper-client-world`,
   `temper-engine-domain-world`, `temper-system-world`. Old-forms from
   `temper/`. Commit and push.

## Report

As `notes/common.md` says; every file outside the crate you edited.
