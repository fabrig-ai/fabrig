+++
id = "pg-project-delete"
part = "playground"
kind = "feature"
needs = ["pg-project-page", "pd-wire", "pg-playground-pages"]
files = [
  "temper/crates/temper-client-domain/src/pages/project.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/project_delete.rs",
  "temper/crates/temper-client-view/src/pages/project.rs",
  "temper/crates/temper-client-view/src/tests/project_delete.rs",
]
effort = "high"
minutes = 50
+++

## Goal

Delete on a project's settings, confirmed by its name, for its owners
(draft `temper/docs/design/draft/playground-reset.md`, 5, "Its
client"): the settings page (`pg-project-page`) gains a "Delete
project" section; the person types the project's name, presses "Delete
NAME", and the client sends jig's `RemoveProject` (`pd-wire`); answered
removed, the page says so and opens Chats, where `me` no longer names
the project.

Acceptance:

- on `/projects/N/settings`, for a party whose role in project N is
  owner (`me.projects[..].role == HeldRole::Owner`) and a project that
  is not the playground's (`root.demo()`'s project), a level-2 heading
  "Delete project", the paragraph `ProjectDeleteDescription`, a text
  field named "Type NAME to confirm" and a button "Delete NAME",
  disabled until the field's bytes equal the project's name exactly;
- for a party who is not an owner: the heading and "Only the project’s
  owners can delete it.", no field, no button; for the playground's
  project: no section at all (its reset deletes it: draft 6.2);
- pressing the enabled button sends one
  `c::Ask::RemoveProject { project: N }` under a fresh key of the pages'
  asker; while it is out, the section shows "Deleting NAME…" and no
  button; a second press sends nothing;
- answered `c::OwnerOutcome::ProjectRemoved { project: N }`: the page
  shows "NAME is deleted." and pushes `Address::Chats { filter: All,
  start: false }`; refused: "Not deleted: " with the refusal's words
  (the cards' refusal phrase), the button back; unknown or lost: the
  button back, nothing claimed;
- the typed name is dropped on leaving the page;
- the tests below pass, and every existing test of the packages
  changed.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 4,
  "A deleted project is gone").
- `temper/docs/design/draft/playground-reset.md`, sections 5 and 6.2.
- `temper/docs/design/client/README.md`, 3.6 (requests and
  confirmations).
- `temper/docs/plan/playground-reset-r4/tasks/pd-wire.md` (the client's
  `c::Ask::RemoveProject` and `c::OwnerOutcome::ProjectRemoved`),
  `tasks/pg-project-page.md` (the page), `tasks/pg-words.md` (the words
  `ProjectDelete`, `ProjectDeleteDescription`, `ProjectDeleteName`,
  `ProjectDeleteConfirm`, `ProjectDeleting`, `ProjectDeleteRefused`,
  `ProjectDeleteOwners`, `ProjectDeleted`).
- A text field and its act, to copy: the chats page's start words
  (`pages::Act::ChatWords`, its view binding in
  `temper-client-view/src/pages/chats.rs`).

## Owns and may change

Owned: the `files` above (`pages/project.rs` in the domain and the two
test files are new).

Neighbour files, edits allowed:

- `temper-client-domain/src/tests.rs`, `temper-client-view/src/tests.rs`:
  one `mod` line each; `temper-client-domain/src/route.rs`: the new
  acts routed to the pages, and an answered key of this page's routed
  to it, as the pages' other keyed requests are.
- `temper-client-view/src/root.rs`: the page's call passes what the
  section needs, if its signature changes.

Pre-approved:

- the page's state lives in `Pages` as `project: Option<project::Settings>`
  opened by `Went` on the settings address and dropped on any other; if
  `Pages` already has a field named `project` (it does: the current
  project), name the new one `settings`;
- the key's asker is `Owner::Pages`; if the pages route answered keys by
  a list of known keys, add this one to it;
- if `pd-wire` named the outcome or ask differently, use its names and
  say so.

## Steps

1. **Domain** (`pages/project.rs`, new; `pages.rs`):

   ```rust
   //! A project's settings page and its deletion (draft playground-reset.md, 5).
   #[derive(Debug)]
   pub struct Settings {
       pub project: u32,
       pub typed: alloc::boxed::Box<[u8]>,
       pub deleting: Option<jig_client::Key>,
       pub outcome: Option<Deletion>,
   }
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub enum Deletion {
       Deleted,
       Refused(jig_client::contract::Refusal),
       Unknown,
   }
   // pages::Act:
   DeleteWords { words: Box<[u8]> },
   DeleteProject,
   // Pages:
   pub const fn settings(&self) -> Option<&project::Settings>
   ```

   `DeleteProject` sends only when the typed bytes equal `me`'s name for
   the project, the party is its owner, it is not the playground's
   project and nothing is out. The answer: `ProjectRemoved` → `Deleted`
   and a `Request::Show` of Chats; a `Refused` answer → `Refused`; every
   other outcome → `Unknown`.
2. **View** (`temper-client-view/src/pages/project.rs`): the section of
   the acceptance after the page's heading and number, the field bound
   to `Act::DeleteWords` (as the chats page binds its words), the button
   to `Act::DeleteProject`, disabled (`State::Disabled`, as other
   disabled controls are drawn) until the typed name matches.
3. **Tests** (client domain and view unit tests):
   - `temper-client-domain/src/tests/project_delete.rs`:
     `delete_sends_remove_project_only_once_the_name_is_typed` (typed
     "widge": nothing; "widget": one `Send` whose ask is
     `c::Ask::RemoveProject { project: 2 }`; again: nothing);
     `a_member_who_is_no_owner_cannot_delete` (role `Member`: nothing
     sent);
     `the_playgrounds_project_offers_no_delete` (a running playground
     answer naming project 2: nothing sent);
     `removed_opens_chats_and_refused_says_why` (answer
     `ProjectRemoved`: `Deleted` and a `Show` of Chats; another run
     answered `Refused(NotOffered)`: `Refused`, no `Show`);
     `leaving_drops_the_typed_name`.
   - `temper-client-view/src/tests/project_delete.rs`:
     `an_owner_sees_delete_confirmed_by_name` (heading, paragraph,
     field "Type widget to confirm", button "Delete widget" disabled,
     then enabled after `DeleteWords`);
     `a_member_sees_only_who_may_delete`;
     `deleting_and_deleted_and_refused_are_said`.
   - Narrow runs per file.
4. Checks, once: `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings`; focused tests of both; `heavy cargo check --tests` for
   `temper-client-tools`, `temper-client-shell-terminal`,
   `temper-client-shell-web`, `temper-client-world`,
   `temper-client-gallery`, one at a time. Commit, push.

## Report

fab's sections; `## Interfaces`: `pages::project::{Settings,
Deletion}`, the two acts, `Pages::settings`; whether the engine's
answer was exercised (it refuses *not offered* until `pd-close`; the
unit tests script the answers).
