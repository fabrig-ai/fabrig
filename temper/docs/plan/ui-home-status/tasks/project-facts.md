+++
id = "project-facts"
part = "project-facts"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/status.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/status.rs",
  "temper/crates/temper-engine-domain-forge/src/landing_log.rs",
  "temper/crates/temper-engine-domain-forge/src/views.rs",
  "temper/crates/temper-engine-domain-forge/src/views/changes.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/limits.rs",
  "temper/crates/temper-engine-domain-forge/src/reconcile.rs",
  "temper/crates/temper-engine-domain-forge/src/restore.rs",
  "temper/crates/temper-engine-domain-forge/src/adoption.rs",
  "temper/crates/temper-engine-domain-forge/src/verification.rs",
  "temper/crates/temper-protocol-store/schema/store-v11.schema",
  "temper/crates/temper-protocol-store/src/generated/**",
  "temper/crates/temper-protocol-store/golden/**",
  "temper/crates/temper-protocol-store/src/keys.rs",
  "temper/crates/temper-protocol-store/src/records.rs",
  "temper/crates/temper-protocol-store/src/records/status.rs",
  "temper/crates/temper-protocol-store/src/tests/**",
  "temper/tests/codecs/tests/suite/fuzzy/store.rs",
  "temper/tests/codecs/tests/support/store.rs",
]
effort = "high"
minutes = 90
+++

# The forge's `ProjectFacts` and `Looked`, the look and the document

## Goal

The connector keeps what changed since a person last looked: `temper/docs/design/domain/forge.md`, 5.6, whole, at the forge's boundary and in the store. The root and the service come in `status-read`.

Acceptance:

- `notes/contracts.md`, *The store* and *The forge*, hold exactly.
- **A landing counts once a change,** in the decision that first logs it: `landing_log::landed`, where `changed`, for a change on any branch of the project, a goal's branch among them. Both of `landed`'s call sites reach it.
- **A look:** `Event::Looked` does each case of 5.6's list: no row, past the bound, under `SITTING`, ahead of the clock, a new sitting.
- **`running`** is stored and restored, and always none here: `status-run` sets it. `brief` too is never written here but by a test's restored row.
- **`Domain::document`** answers `Document::Status(StatusView)` for `status_subject(project)` in the project's scope, for the asking party, with `queued`, `red` and `branch` computed at the read, per 5.6.
- **`queued` equals the changes document's count** for the same project: the rows it lists plus `more`.
- **Marks:** the subject is marked wherever `changes_mark` is, and at a look.
- **Erased** in `adoption.rs`'s project release, beside `dismissed::erase_project`, marked when any went.
- **Restored and refused** as `Dismissed` is: `restore.rs`'s bounds, `reconcile.rs`'s kinds; `ProjectFacts` at most `judge_projects`, `Looked` at most `l.dismissed()`'s product.
- `temper/docs/design/protocol/store.status.md` and `domain/forge.status.md` lose what this closes: 2's families; 5.6 stays *Not yet* only for what `status-read` adds.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md` (*`seen` is the count at `previous`*, *Reuse what `ui-changes` landed*), `notes/contracts.md`.
- `temper/docs/design/domain/forge.md`, 5.5 and 5.6; `temper/docs/design/protocol/store.md`, sections 2 and 3; `temper/docs/design/client/README.md`, 4.6 (`changes`, `status`).
- The models: `temper-engine-domain-forge/src/dismissed.rs`, and commit `84a75b751a`, which added `LandingLog` whole. Read its `--stat`: its readers are this task's.

## Owns and may change

Owns, as `files` lists them. Status lines as above.

May also change:

- **Compiler-forced arms,** each only for that: `temper/crates/temper-engine-domain/src/route/panels.rs` (`Document::Status` beside `Document::Changes`), `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`, `temper/crates/temper-fake/src/check.rs`, `temper-fake/src/tests/check.rs`, `temper/tests/engine/forge/src/world.rs`, `temper/tests/engine/protocol-forge/src/protocol.rs`, and `temper-service/src/family.rs`'s `document`, whose `Document::Status` arm answers `None` until `status-read`.
- **Tests that pin a `Marked` list:** `tests/panel.rs`, `tests/change.rs`, `tests/views.rs`, `tests/landing_log.rs`, `tests/verification.rs`, `temper/tests/engine/domain/tests/suite/change_panel.rs`, `panels.rs`, `temper/tests/engine/forge/tests/suite/changes.rs`: the status subject beside the changes subject, nothing else.
- **`views/changes.rs`:** `changes_red(row)` may become `branch_red(repository, base)`, with its one caller moved.

Options, each pre-approved:

- **Where the mark goes:** `changes_mark` returns both subjects, or each site pushes a second `Marked`. Take whichever edits the fewest tests, and mind `MARKED`'s room in `save_queue_change`; say which.
- **A store byte:** 36 and 37; if taken when you start, the next free, in `keys.rs` and the design's table.
- **A new limit is not needed:** both bounds are products of limits held. If `reconcile.rs`'s budget or `temper-service/src/limits.rs` asks for a line, add it, and regenerate `temper/docs/help/reference/limits.md` if it moves.
- **Clock behind:** `now` before `latest` is under `SITTING`, whatever the difference; `Wall`'s arithmetic stays checked.

## Steps

1. **The store:** schema, `SKEIN_CODEGEN_WRITE=1 TEMPER_BUNDLE=none heavy cargo nextest run -p temper-protocol-store`, then unset; `keys.rs`, encoders and decoders in key order; `records.rs`, `records/status.rs`.
2. **`status.rs`:** the types, `landed`, `looked`, `erase_project`, `status_view`, per the contract.
   ```rust
   pub(crate) fn landed(d: &mut Domain, now: Wall, repository: Repository, out: &mut Queue<Request>);
   pub(crate) fn looked(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, party: u64, project: u32, out: &mut Queue<Request>);
   pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>);
   ```
3. **Wiring:** `domain.rs`'s fields, restore and `RecordKind`; `lib.rs`'s `key_of` and `row_bytes`; `reconcile.rs`; `restore.rs`.
4. **Tests,** `src/tests/status.rs`, shaped as `tests/dismissed.rs`:
   `a_landing_counts_once_and_at_either_site`; `a_goal_branch_landing_counts`; `a_first_look_has_no_previous_and_nothing_new`; `a_look_within_a_sitting_moves_latest_alone`, at 29:59; `a_look_at_thirty_minutes_starts_a_sitting`; `a_change_landed_between_sittings_reads_as_new`, the rule of *`seen` is the count at `previous`*: `landed` less `seen` is 1 after the second sitting's first look; `a_clock_behind_stays_in_the_sitting`; `past_the_bound_a_look_is_too_large_and_writes_nothing`; `the_document_is_the_asking_partys_own`; `queued_is_what_the_changes_document_counts`; `red_is_the_home_landing_branchs_alone`; `no_home_has_no_branch_and_no_red`; `erased_with_its_project`; `restored_rows_past_their_bound_are_refused`.
   In `temper-protocol-store`'s `tests/keys.rs` and `tests/records.rs`, and the codecs' fuzzy `store.rs`: both families, as `LandingLog`'s.
5. **Check,** the implementer's text, with `temper-engine-domain`, `temper-service`, `temper-fake` as direct dependents (`heavy cargo check -p ... --tests`), `python3 -I -B tools/citations/citations.py --base origin/main`, then push.

A change of meaning: `Marked` lists grow, so run `temper-engine-forge-world`'s and `temper-engine-domain-world`'s suites that name `changes_subject`.

## Report

As `notes/common.md`, Your report, with where the mark went.
