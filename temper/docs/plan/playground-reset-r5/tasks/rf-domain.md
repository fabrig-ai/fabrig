+++
id = "rf-domain"
part = "remove-forge"
kind = "feature"
needs = ["rf-family"]
files = ["temper/crates/temper-engine-domain-forge/src/**"]
effort = "high"
minutes = 120
+++

## Goal

The forge connector removes a forge whole (draft
`temper/docs/design/draft/playground-reset.md`, section 4; 6.4's start
over): `Domain::remove_forge` ends it in one decision and keeps the rest
until whole, across restarts, as `notes/remove-forge.md`, sections 2, 3
and 5 say; `Ask::RemoveForge` is its owners' act; a project removed by
jig takes its repositories out of it.

Acceptance:

- `Domain::remove_forge(&mut self, forge: u16, out: &mut Queue<Request>)
  -> Result<(), RemoveRefusal>`: the note's section 2, steps 1 to 5, in
  the caller's decision. `Ask::RemoveForge { forge }` admits it for the
  deployment's owners (role checked as `connect::start` checks it,
  refused `Role` otherwise: add `RemoveRefusal::Role`), answers
  `Answered::RemoveForge(Removing)` held in memory as the connect answers
  are, or `Refused(NoSuchForge)`.
- **Ended for any forge:** `Domain::forge_ended(forge)` is true for a
  record with a removal, besides today's ended demo forge, and every gate
  survey S3, section 2 lists reads it (the gates call `forge_ended`
  already; `retire_ended`'s guard, today `DemoState::Ended`, runs for
  either). The ended forge leaves `Domain::forges()`.
- **Kept until whole:** `Event::GroupRemoved` and `Event::SecretsErased`
  save the record's flags; `Kept` rows of `KeptAct::Cancel` are saved
  before sent (`Request::SendKept`) and erased on `Event::KeptAnswered {
  keep: false }`; a repository row of a removing forge is erased in the
  decision that erases the last row naming it (or at once if none does);
  the record is erased when the note's condition holds; at restart, a
  record with a removal is restored ended (never `ForgeKept`), and its
  `RemoveGroup` / `EraseSecrets` emitted again while its flag is false
  (with the restart's other resends).
- **Removal frees a box port:** `make_forge`'s `Taken` no longer counts a
  record with a removal (it already does not, by `fr-domain`); a new owned
  forge made while the old one's group is not yet removed is admitted.
- **Project removed:** `Event::ProjectRemoved { project }` marks the
  project gone in the connector (a `removed_projects` set bounded by
  `Limits.repositories`, dropped as its repositories go): its repositories
  leave readable/writable/home/context lists and adoption choices, and
  their rows are erased by the same rule as a removing forge's.
- Tests (library tests, a new `src/tests/removal.rs`):
  `remove_forge_ends_the_forge_in_its_decision_and_asks_the_rest`,
  `remove_forge_keeps_a_cancel_per_live_change_on_its_repositories`,
  `remove_forge_erases_its_writer_and_flights_at_once`,
  `a_repository_named_by_a_task_stays_until_the_task_is_erased`,
  `the_record_is_erased_once_group_secrets_cancels_and_repositories_are_done`,
  `a_restart_mid_removal_resends_what_is_unanswered_and_never_configures_it`,
  `a_removed_number_is_never_made_again`,
  `remove_forge_by_a_non_owner_is_refused_role`,
  `a_removed_project_takes_its_repositories_out_of_every_list`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (sections 2
  and 3), `temper/docs/plan/playground-reset-r5/notes/remove-forge.md`,
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 1 and 2), `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`,
  section 7 (`ProjectRemoved` is jig's only notice).
- `temper/docs/design/domain/forge.md`, 4.1 (the playground's forge's
  end, which this makes any forge's).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  section 2 (every ended-forge gate).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/src/**`.
- Not yours: the root's sending of kept acts and routing of the events
  (`rf-root`), the service's (`rf-service`).
- Options: if a live change's project cannot be read from its `Change`
  row, read it from its `Names`' adoption row and say so. If the
  repositories' erasure check at each erase costs a scan of every task's
  rows, keep a per-repository count of naming rows instead, restored at
  start.

## Steps

1. `remove_forge` and the ask; ended for any forge.
2. Kept until whole; restart.
3. `ProjectRemoved`.
4. Tests. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge
   --lib -E 'test(/removal/)'`.
5. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge
   --all-targets -- -D warnings`; focused tests of
   `temper-engine-domain-forge` and `temper-engine-domain` (`forge_ended`
   changes meaning: run the root's too) and `heavy cargo nextest run -p
   temper-engine-domain-world --test suite`. Old-forms from `temper/`.
   Commit and push.

## Report

As `notes/common.md` says; how the repositories' erasure finds the rows
naming them.
