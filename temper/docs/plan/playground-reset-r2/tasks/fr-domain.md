+++
id = "fr-domain"
part = "forge-records"
kind = "feature"
needs = ["fr-ask"]
files = [
  "temper/crates/temper-engine-domain-forge/src/records.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/domain/**",
  "temper/crates/temper-engine-domain-forge/src/connect.rs",
  "temper/crates/temper-engine-domain-forge/src/adoption.rs",
  "temper/crates/temper-engine-domain-forge/src/restore.rs",
  "temper/crates/temper-engine-domain-forge/src/reconcile.rs",
  "temper/crates/temper-engine-domain-forge/src/kept.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/**",
  "temper/crates/temper-engine-domain/src/**",
  "temper/tests/engine/domain/**",
  "temper/tests/engine/forge/**",
]
effort = "high"
minutes = 90
+++

## Goal

The forge connector keeps, numbers and restores forge records
(`temper/docs/design/draft/playground-reset.md, 3`): a `SetUp` or
`Connect` with `Target::New` makes the forge's record in its decision,
numbered next and never reused; a seed keeps a record under its given
number; the first repository's intent is kept in the record, restored
at the next start and cleared once the first repository is adopted
(`temper/docs/design/protocol/setup.md, 6.7`, now kept).

Acceptance, unit tests in `temper-engine-domain-forge`
(`src/tests/records.rs`, beside fr-ask's):

- `a_new_owned_forge_is_numbered_kept_and_set_up`: on an empty store
  under the box, `SetUp { target: New(owned) }` answers
  `SettingUp { forge: 1 }`, saves `Stored::Forge` (number 1, kind
  Forgejo, mode Owned, `first` the intent) and `ForgeNumbers { next: 2
  }`, and pushes `Request::Made { forge: 1 }` before `Request::SetUp {
  forge: 1, .. }`; the forges' rows list forge 1.
- `numbers_are_never_given_twice`: make 1 and 2, erase 2's record
  (`kept::erase_forge`), restore the saved rows into a fresh domain; the
  next made forge is 3.
- `a_refused_request_makes_nothing`: a New SetUp refused (not under the
  box; busy; an invalid name) and a New Connect refused (the kind `how`
  needs differs from the projection's) save no row and push no `Made`.
- `connect_new_github_makes_a_github`: `Connect { target: New(github),
  how: GitHubManifest { .. } }` answers with the new number and the
  Connect request names it.
- `the_forges_limit_refuses_full`: with `forge.forges` = 2 and two kept
  forges, a New SetUp is refused `Full`; a definition longer than
  `definition_bytes` is refused `Full` too.
- `a_seed_is_kept_once_under_its_number`: `Event::Seed { number: 4 }`
  saves forge 4 and `next` 5; the same seed again, or a seed of a
  number below `next`, saves nothing.
- `the_first_intent_is_kept_restored_as_making_and_cleared_at_adoption`.

Engine world (`temper-engine-domain-world`): today's
`tests/qa_all/qa_first_repository.rs`
`first_repository_row_is_volatile_across_restart` asserts the intent
is lost; it becomes `first_repository_intent_is_kept_across_restart`:
after the restart the forge's row shows the first repository as
making, with its names and project.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 2
  and 5.
- `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/design/protocol/setup.md`, 6.7;
  `temper/docs/design/domain/forge.md`, 4.1;
  `temper/docs/design/protocol/credentials.md`, section 2 (why a
  number is never reused: it keys the forge's secrets).
- `tasks/forge-families.md` (the held rows and `kept.rs`'s makers) and
  `tasks/fr-ask.md` (the surface this task fills).

## Owns and may change

Owns, after `fr-ask` (D4's `rm-` and `pg-` tasks own these next):
`temper-engine-domain-forge`'s `src/records.rs`, `src/domain.rs`,
`src/connect.rs`, `src/adoption.rs`, its tests, and the restore and
reconcile paths for the `Forge` rows (`src/restore.rs`,
`src/reconcile.rs`, the restore arms in `domain.rs`), `src/kept.rs`
(a companion maker if one is missing, below);
`temper-engine-domain`'s `src/**` where the root passes these through;
the engine worlds' tests and fixtures.

May change: `src/boundary.rs` of the forge crate only for a doc
comment or a derive the new code needs (fr-ask fixed the shapes; a
shape change is a question); anything `notes/scope.md` lists as
forced.

## Steps

1. **`records::make_forge`** (fr-ask's stub), in order, all checks
   before any write, so a refused request makes nothing:
   - the definition at most `limits.definition_bytes`, and fewer than
     `limits.forges` records held that are not `removing`; else `Full`;
   - `kept::give_number` (its `None`, numbers exhausted: `Full`; never a
     wrap, `notes/scope.md`);
   - `kept::save_forge(ForgeRecord { number, kind: configured.kind,
     mode: configured.mode, removing: false, definition, first })`;
   - insert `configured` with `number` set into the `configured` map,
     and the identity `(number, u32::from(number))`;
   - push `Request::Made { forge: number }`, then return the number.
   Give `make_forge` a checking twin the decisions call first
   (`records::room(d, &new) -> Result<(), Full>`), so the decision
   checks the request's own conditions and room before it makes.
2. **SetUp with `Target::New(new)`** (`domain.rs`, near today's SetUp
   decision, `domain.rs:1178`): the conditions today's offer and
   decision check on a configured owned forge, checked on
   `new.configured` instead (mode `Owned`, `asked`, under the box, no
   other forge setting up, names and project valid; `Invalid` when the
   mode is not owned); then `make_forge` with `first: Some(FirstIntent
   { playground, org, repository, project, label })`, the label today's
   (`temper-host`); then on as `Target::Kept(n)` goes, answering
   `SettingUp { forge: n }`.
3. **Connect with `Target::New(new)`** (`connect.rs`, `start`): the
   kind and mode `how` needs (`GitHubApp`/`GitHubManifest`: GitHub;
   `ForgejoAdmin`: Forgejo, mode Managed or Connected) must be the
   projection's, else `NoSuchForge`; the deployment-owner and carried
   checks as today; then `make_forge`, `first` from `ForgejoAdmin`'s
   `first` (`FirstIntent` from `FirstAsk`), then on as with the kept
   forge, the answer naming it. `Target::Kept(n)` stays as today (GitHub's
   Go on, `playground-reset.md, 6.4`).
4. **`Event::Seed { number, new }`:** ignored when `number` is 0, has a
   record, or is below `forge_next()`, or when no room (`room`); else
   the record saved under `number`, `ForgeNumbers { next: number + 1 }`
   saved when that is above the held `next` (a companion maker in
   `kept.rs` or `records.rs`, `notes/scope.md`), `configured` and the
   identity installed. A seed pushes no `Made`: the service builds the
   seeded forges with the restored ones. `number` at `u16::MAX` keeps
   `next` at `u16::MAX` and every later `give_number` is refused.
5. **The intent kept:** a restored `ForgeRecord` with `first: Some(i)`
   inserts `First { playground, org, repository, project, label,
   state: FirstState::Making }` into the `first` map (a made-but-not-
   adopted repository is made again by the setup host, fr-setup). In
   the decision that commits an adoption (`adoption.rs`) of a
   repository on forge `n` whose record has `first: Some(_)`, save the
   record with `first: None`. Today's volatile state changes
   (`first_made`, `first_failed`, `connect.rs:214`'s removal) stay; the
   record's intent is cleared only by the adoption.
6. **Restore and reconcile:** restored `Forge` rows are held as
   forge-families made them; the `configured` map is filled by the
   service's `configure` calls, not by restore. Where `reconcile.rs`
   resolves a restored reference against configured forges
   (`reconcile.rs:291`), resolve it against held records or
   configured forges, so a store restored before `configure` reconciles.
7. **The root:** nothing beyond fr-ask's pass-throughs, unless a test
   needs a helper; an engine-world fixture that needs a forge seeds a
   `Record::Forge { row: Stored::Forge(..) }` by `Store::seed`
   (`temper/tests/engine/domain/src/store.rs:259`) or steps
   `forge::Event::Seed`, then calls `configure_forge`.
8. Tests (Goal). Then `cargo fmt`; `heavy cargo clippy -p
   temper-engine-domain-forge -p temper-engine-domain --all-targets --
   -D warnings`; one at a time: `heavy cargo nextest run -p
   temper-engine-domain-forge -p temper-engine-domain`, `-p
   temper-engine-domain-world -p temper-engine-forge-world`, `-p
   temper-service` (a direct dependent; its tests run unchanged,
   nothing there sends `Target::New` yet); `python3 -I -B
   scripts/old_forms.py` from `temper/`.
9. Commit (making, seed, intent, tests) and push.

## Report

The standard report, plus: where the volatile `first` entry and the
record's intent can disagree; what adoption clears the intent (the
names it matches); whether `reconcile.rs` needed the records path.
