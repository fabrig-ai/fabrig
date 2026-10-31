+++
id = "cl-project"
part = "client"
kind = "feature"
needs = ["cl-surface", "jig-ask"]
files = [
  "temper/crates/temper-client-domain/src/pages/project.rs",
  "temper/crates/temper-client-domain/src/tests/pages/project.rs",
  "temper/crates/temper-client-view/src/pages/project.rs",
  "temper/crates/temper-client-view/src/pages/project_words.rs",
  "temper/crates/temper-client-view/src/tests/project.rs",
]
effort = "high"
minutes = 70
+++

## Goal

temper's project settings page (`/projects/N/settings`, made empty by
`cl-surface`) with Delete for the project's owners, confirmed by typing
the project's name (draft 5, "Its client"): the page reads the project,
offers Delete when the owner document offers `ActKind::RemoveProject`,
sends jig's `Ask::RemoveProject { project }` once the typed name matches
and is confirmed, and leaves for the chats once the project is gone.
Until `jp-delete` the engine refuses it *not offered*; the page shows
that refusal in jig's words.

Acceptance:

- `temper-client-domain`, new `src/tests/pages/project.rs`:
  `the_settings_page_reads_its_project`,
  `delete_is_offered_only_when_the_owner_document_offers_it`,
  `delete_waits_for_the_projects_exact_name`,
  `a_confirmed_delete_sends_remove_project_once`,
  `a_project_gone_from_me_leaves_for_the_chats`,
  `a_refused_delete_keeps_jigs_refusal`.
- `temper-client-view`, new `src/tests/project.rs`:
  `an_owner_sees_delete_confirmed_by_the_name`,
  `another_party_sees_the_name_and_no_delete`,
  `delete_is_disabled_until_the_name_matches`,
  `a_sent_delete_shows_deleting_then_its_refusal`.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 1
  (`RemoveProject`, its answer, a project gone answers unknown).
- `temper/docs/plan/playground-reset-r2/tasks/jig-ask.md` (the client
  contract's ask and offered act, as merged) and `cl-surface.md`, step 7
  and 9 (the page's stubs and acts).
- `temper/docs/design/draft/playground-reset.md`, 5.
- `jig/docs/design/client/domain/contract.md`, the owner's project
  document and its reads (`rg -n OwnerProject` there).
- `jig/crates/jig-client-view/src/page/project.rs` and
  `jig/crates/jig-client-view/src/page/opening.rs` (how jig's reference
  UI reads a project: `contract::Read::Project { project }`, answered
  `ReadValue::OwnerProject` for an owner); read, never change.

## Owns and may change

Owns the files in the header. May add its test modules' `mod` lines
(`temper/crates/temper-client-domain/src/tests/pages.rs` or its
`tests.rs`, `temper/crates/temper-client-view/src/tests.rs`) and
`mod project_words;` beside `project` in
`temper/crates/temper-client-view/src/pages.rs` (the lines only, each in
its alphabetical place). Forced edits per scope.md. Not yours:
`D/pages.rs` (`cl-play-domain` holds it): `cl-surface`'s hooks there
(`went`, the act and told routing) call your functions as they are; if a
hook lacks a parameter you need, add it at the hook's call in D/pages.rs
(the argument only, a forced edit) and say so in the report.

## Steps

D = `temper/crates/temper-client-domain/src`, V =
`temper/crates/temper-client-view/src`.

1. **The page's state (D/pages/project.rs).** `Project { project,
   read: Option<jig_client::Key or Handle, as pages.rs's reads use>,
   document: Option<Box<c::OwnerProject>>, name: Option<Box<[u8]>>
   (a non-owner's `ReadValue::Project`), typed: Box<[u8]>, confirming:
   bool, delete: Option<Deleting> }`, `Deleting { key, progress,
   refusal: Option<c::Refusal or EntranceRefusal> }`. Read the project
   when the page is made (follow how D/pages.rs's other pages read one
   value; re-read on a link gap as they do).
2. **Delete.** `pub fn offered(page: &Project) -> bool`: the owner
   document's `acts` holds `ActKind::RemoveProject`. `DeleteProject`
   opens the confirmation (`confirming`), `DeleteWords { words }` sets
   `typed` (bounded by the field limit), `CancelDelete` closes it,
   `ConfirmDelete` sends `jig_client::Ask::Jig(Box::new(c::Ask::
   RemoveProject { project }))` once, only when offered and `typed`
   equals the document's name bytes exactly, under the name
   `remove-project-N` (`crate::key::named` with the pages' key space, as
   other named page requests are).
3. **Its end.** A done answer, or the project gone from `Me`'s projects
   (the frame's document), drops the page's state and pushes
   `Request::Replace { address: Address::Chats { filter: All, start:
   false } }`; a selected project that is gone is deselected as the
   pages deselect one today. A refusal is kept and shown; an unknown
   outcome (contracts.md 1: a project gone answers unknown) reads as
   gone if `Me` no longer lists it, else as a refusal.
4. **The drawing (V/pages/project.rs, V/pages/project_words.rs).** A
   `Main` headed "PROJECT · settings"; for an owner offered Delete, a
   group "Delete PROJECT": "Deleting PROJECT ends its chats and tasks and
   erases them; its repositories leave it. What it spent stays in the
   totals." then, after "Delete project…" (`DeleteProject`), an input
   "Type PROJECT to delete it" (`DeleteWords`), a button "Delete
   project" (`ConfirmDelete`, disabled until the name matches) and
   "Keep it" (`CancelDelete`); sending: "Deleting PROJECT…"; a refusal in
   jig's refusal words as the view words others (`jig_client::words`,
   `Words::say_word`). Another party: the name alone, and "Only the
   project's owners can delete it." Words in `project_words.rs` after
   `V/accounts/words.rs`'s pattern, under a phrase owner no other table
   uses. Bindings: the page acts via the pages' bindings (as
   `V/pages/chats.rs` binds `pages::Act`s).
5. **Checks:** the tests above; `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings`; one `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-web -p
   temper-client-shell-terminal -p temper-client-journeys`.
6. Old forms from `temper/`; commit and push.

## Report

- The client contract's answer to a done `RemoveProject` as `jig-ask`
  built it, and which end signal the page relies on.
- What the code taught for `client/README.md` (temper had no project
  settings page).
