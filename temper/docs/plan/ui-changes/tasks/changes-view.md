+++
id = "changes-view"
part = "changes-view"
kind = "feature"
needs = ["changes-wire", "landing-log"]
files = [
  "temper/crates/temper-engine-domain-forge/src/views.rs",
  "temper/crates/temper-engine-domain-forge/src/views/**",
  "temper/crates/temper-engine-domain-forge/src/tests/views.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/limits.rs",
  "temper/crates/temper-engine-domain/src/route/panels.rs",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/limits/names.rs",
  "temper/tests/engine/forge/tests/suite/changes.rs",
  "temper/tests/engine/domain/tests/suite/panels.rs",
]
effort = "high"
minutes = 110
+++

# The watch `changes`

## Goal

The engine serves `Changes { project }`: a project's queues, in order, the branches' red and its last
landings, whole, in the decision that asked, and marked as they change.

Acceptance:

- `Document::Changes` is what `notes/contracts.md` says, made as *The rows* below says.
- A `Watch::Changes` is served through `temper-service`, for a party with a role in the project, as
  `Watch::Landings` is; a scope naming another project is `Gone`, as there.
- A world lands a change, closes it and still reads it in `landed`, with its title.
- The marks below each have a step test that bites.
- `forge.changes_rows` (32) and `forge.changes_landed` (8) are limits, as `notes/contracts.md` says.

## Inputs

- `temper/docs/plan/ui-changes/notes/common.md`; `temper/docs/plan/ui-changes/notes/contracts.md`, *The view*,
  *The wire* and *The limits*.
- `temper/docs/design/client/README.md`, 4.6, *A project's changes*.
- `temper/docs/design/client/forge.md`, section 3.
- `temper/docs/design/domain/forge.md`, section 9, for what a queue is.
- **The model:** `landings_subject`, `landings_view` and `Document::Landings` in `views.rs`;
  `public/landings.rs`, `public_queues`, for the order.

## Owns and may change

Owns `files`. May also change:

- `temper/crates/temper-engine-domain-forge/src/domain.rs`, `verification.rs`, `landing_log.rs`: the marks,
  those lines alone.
- `temper/crates/temper-engine-domain-forge/src/public/landings.rs`, `public.rs`: to share the ordering (a
  helper taking the candidates) and `repository_address` (made `pub(crate)`), no change of what they give.
- `temper/crates/temper-engine-domain-forge/src/panel.rs`: nothing but a visibility.
- `temper/tests/engine/domain/src/oracle.rs`, `temper/crates/temper-engine-domain/src/tests/documents.rs`,
  `temper/tools/gallery/src/*`: a `Document` arm the compiler asks, nothing more.
- `temper/tests/engine/forge/tests/suite/main.rs`: its `mod`.
- The limits' names: `temper/docs/help/reference/limits.md`, `temper/docs/examples/reference/*`,
  `temper/crates/temper-service/src/tests/startup.rs` (a literal).
- **Design status, its closed line alone:** `temper/docs/design/domain/forge.status.md` (9). The client's
  status files are `changes-page`'s, which runs beside this task.

Pre-approved:

- `temper-engine-domain/src/decision.rs` holds a `ForgeDocument`'s `owned_bytes` within `result_bytes` or
  `transcript_bytes`. If the worst `ChangesView` under `small` passes it, add `changes_rows` and
  `changes_landed` to the check `valid` makes against those limits, as the smallest change that keeps it true,
  and give the figures in the report.
- A change of meaning: `save_change` and `save_queue_change` emit one more subject. Tests that assert a
  `Marked` exactly (`rg 'Marked \{' temper/crates/temper-engine-domain-forge/src/tests`) change with it.

## Steps

1. **`views.rs`:** `changes_subject`, the types, the `document` arm (before the `landing.rows` lookup,
   beside the list's), `changes_view`, `owned_bytes`; `lib.rs`'s re-exports.
2. **The rows,** in `fn changes_view(&self, project: u32, now: Wall, limits: &Limits) -> Option<ChangesView>`:
   - **Which queues:** the `(repository, base)` of each change in the project (`repositories[..].project`)
     that is `panel::ready`, or owns the queue (`change.owns_turn` and not `terminal()`), and each branch
     that is red. By `repository_address`, then branch.
   - **A queue's rows:** those that own the queue and are not `panel::ready` (at `Updating`, `Checking`,
     `Gating`, `Landing`), by their `ready_since`, then `queue_ready`'s candidates in `panel::queue_before`'s
     order, as `public_queues` selects them. `forge.md`, 9 has the first keep its turn through those steps,
     so the row must not vanish while it updates. Title by `crate::text::fit(&row.title,
     LOGGED_TITLE_BYTES, "…".as_bytes())`. `ready_since` from the state, or `change.ready_since`.
     `checks_since`, `change.since`, on the queue's first row alone.
   - **`changes_rows`** bound the rows across the queues, in order; the rest count in `more`. A queue past
     it stays, with `changes` empty. At most 32 queues.
   - **`red`:** a `Verification` of the repository, one of its `changes` based on the branch, whose tier has
     a verdict `Owns` or `Suspect` and no `repaired` (`landing_red`'s rule, shared, not copied); or a live
     change into the branch with `base_repair`.
   - **`landed`:** every project repository's `landing_log`, each landing's each change, by `at`, newest
     first, then by repository and commit for a steady order; `changes_landed` kept, the rest in
     `landed_more`.
   - **`landing`:** the newest logged landing's branch; else the home repository's
     `settings.default_branch`; else the first repository's; else empty.
3. **Marks:** `changes_subject(project)` joins `Request::Marked`:
   - in `save_queue_change`, when `before != after`: a change enters, leaves or moves;
   - in `save_change`, when `row.change.owns_turn` or `row.base_repair`: the first's step, a repair's;
   - in `landing_log::landed`: each merge;
   - in `verification.rs`, as its row is saved or erased: a red begins or ends.

   One helper, `changes_mark(d, repository) -> Option<u64>`, by `repositories[..].project`. Keep within
   `MARKED`.
4. **`route/panels.rs`:** the arm of `document @ (..)`. **`family.rs`:** `Watch::Changes` to
   `forge::changes_subject(subject.project())`, in place of `changes-wire`'s refusal; the arm of `document`,
   building `w::ChangesPanel`, times by `as_nanos`. **Limits:** `limits.rs`, `valid`, `names.rs`.
5. **Tests.** `src/tests/views.rs`, by its `document(d, subject)` form:
   - `changes_document_lists_a_queue_in_order`: priority, then time; a change past the window ahead.
   - `changes_document_keeps_the_first_while_it_updates`: at `Updating`, it still leads, with `checks_since`.
   - `changes_document_says_a_red_with_nothing_queued`: a queue, no rows, `red`, and `landing`.
   - `changes_document_cuts_and_counts`: `changes_rows` and `changes_landed` past, `more`, `landed_more`.
   - `changes_document_lists_landed_newest_first_across_repositories`.
   - `changes_document_keeps_to_its_project`: another project's change and scope.
   - `changes_marks_*`, one for each mark of step 3.
   - `owned_bytes`: its arm charges every field, in the file's form.

   `temper-service/src/tests/family.rs`, beside `LandingsView`: `changes_document_encodes_and_decodes`, and
   the watch's subject. `tests/engine/domain/tests/suite/panels.rs`, beside its `Landings`: the watch through
   the root. `temper/tests/engine/forge/tests/suite/changes.rs`: `a_landed_change_keeps_its_title_after_its_close`:
   the story, on `world.document(PROJECT, top::changes_subject(PROJECT))`, with
   `temper/tests/engine/forge/tests/suite/landing.rs` as the form: queued, the document lists it with its
   title; landed; closed, its `ChangeRow` gone, and `landed` still says it.
6. **Check:** the narrow tests, then `heavy cargo nextest run -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-service --lib`, `-p temper-engine-forge-world`, `-p temper-engine-domain-world
   --test suite -E 'test(panels::)'`, `-p temper-docs`; since `Marked` says more, a change of meaning, also `-p
   temper-engine-domain-world --test suite -E 'test(forge_routes::)'`. `heavy cargo check -p temper-client-gallery -p temper-fake
   --tests`; fmt, old forms, clippy; the citations.
7. Commit and push.

## Report

As the implementer's text has it, with the worst `owned_bytes` under `small`, and each test's rule broken.
If `queue_first` lets the next ready change take `First` while the first is at `Updating` (it reads
`queue_ready`, which leaves `Updating` out), say so with a test's evidence: it departs from
`temper/docs/design/domain/forge.md`, 9. Don't change it here: the document then has two rows that own the
queue, and only the queue's first carries `checks_since`.
