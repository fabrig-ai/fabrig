+++
id = "jp-client"
part = "project-delete"
kind = "feature"
needs = ["jig-ask"]
files = [
  "jig/crates/jig-client-view/src/sections/project/owner.rs",
  "jig/crates/jig-client-view/src/page/project/owner_form.rs",
  "jig/crates/jig-client-view/src/page/project/delete.rs",
  "jig/crates/jig-client-view/src/owner_project_words.rs",
  "jig/crates/jig-client-view/src/tests/owner_project.rs",
  "jig/crates/jig-client-view/src/tests/domain/owner_project.rs",
  "jig/tests/client/tests/golden/**",
  "jig/tests/client/tests/stories/projects.rs",
  "jig/testing/jig-client-scripted-engine/src/requests/projects.rs",
]
effort = "high"
minutes = 80
+++

## Goal

The reference client's Delete on a project's settings page
(playground-reset.md, 5, "Its client: Delete on the project's settings,
confirmed by its name, for its owners"): offered when the project's
document offers `ActKind::RemoveProject`, enabled only once the person
has typed the project's name exactly, sending `Ask::RemoveProject {
project }`; on `ProjectRemoved` the client goes home (the inbox, no
project) with a notice naming the deleted project; a refusal shows on
the page. temper's own page is the client part's (D5), not this task's.

Acceptance:

- View (`src/tests/owner_project.rs`):
  `delete_is_offered_to_an_owner_with_remove_project` (a Danger-styled
  "Delete project" region with a name field and a disabled Delete
  button); `delete_is_absent_without_the_offer` (the act not offered, or
  `Refused`/`Bounded`: no region); the page passes `cards_access::check`.
- Domain (`src/tests/domain/owner_project.rs`, `spine` style):
  `delete_stays_disabled_until_the_name_matches` (a prefix, a different
  case and a trailing space keep it disabled; the exact name enables it);
  `delete_sends_remove_project_once` (one `Ask::RemoveProject { project }`
  sent, a second press while pending sends nothing);
  `project_removed_goes_home_with_a_notice` (the answer navigates to
  `Address::Inbox { project: None }` and keeps a notice "Project NAME
  deleted"); `a_refused_delete_shows_on_the_page` (*unknown* and *not
  offered* shown with `words.refusal`, the field kept).
- Story (`jig/tests/client/tests/stories/projects.rs`, the scripted
  engine): `an_owner_deletes_a_project_by_its_name`: the owner opens the
  project's settings, types its name, deletes it, lands on the inbox, and
  the project is no longer in the frame's projects.
- The printed-page goldens (`jig/tests/client/tests/golden/*.txt`)
  regenerated where an owner's project page now shows Delete
  (`SKEIN_CODEGEN_WRITE=1`), and no other golden changed.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1
  (`Ask::RemoveProject`, `Outcome::ProjectRemoved`, a project gone
  refused *unknown*).
- `temper/docs/design/draft/playground-reset.md`, section 5 ("Its client").
- `jig/docs/design/client/view/` (the project page and confirmations: rg
  `ProjectSection` and `dangerous`), `jig/docs/design/client/domain/root.md`
  (intents and their answers).

## Owns and may change

Owns the files in the header. May change, as `notes/scope.md` says:
`jig-client-view/src/{action.rs,binding.rs,page.rs,ui.rs,page/conversations.rs,page/project.rs,page/routing.rs,intents.rs,ui/client/outcome.rs,notices.rs,notices/**,owner_words.rs,phrases/**,words.rs}`
for the new action, meaning, intent purpose, notice and words and their
arms (the `jig-ask` arms there were placeholders: replace them);
`jig/testing/jig-client-scripted-engine/src/{requests.rs,payload.rs}`
so the scripted engine answers `RemoveProject` with `ProjectRemoved` and
drops the project from the documents it serves; every exhaustive match
the new items force. No change to `jig-client-domain`'s contract or the
wire (`jig-ask` made them).

Not yours: temper's client (`temper-client-*`, D5's).

## Steps

1. Words (`owner_project_words.rs`): new `ProjectPhrase`s after the
   last id (9029): `DeleteProject` "Delete project", `DeleteNameField`
   "Type the project's name to delete it", `DeleteWhat` "Its
   conversations and tasks are ended and erased, with its notes and
   settings. Its spend stays in the totals.", `Deleted` "Project {name}
   deleted" (a text hole, registered as the other holes are), `ALL` and
   `COUNT` extended, registered in `owner_words.rs`.
2. `page/project/delete.rs` (new): the Delete state on the project page:
   `pub struct Delete { typed: Box<str>, pending: Option<Token>, refused:
   Option<Refusal> }`; `fn offered(doc: &OwnerProject) -> bool`
   (`ActKind::RemoveProject` with `Admission::Allowed`, as
   `pool_forms` reads its offer); `fn ready(doc, state) -> bool` (typed ==
   `doc.name`, byte for byte); `fn send(domain, doc, out)` asking
   `contract::Ask::RemoveProject { project: doc.number }` through
   `domain.intents.ask` with `Origin::Page { handle, purpose:
   IntentPurpose::RemoveProject }` (new purpose); `fn answered(...)`: on
   `ProjectRemoved`, `routing::navigate` to `navigation.home` and keep
   the `Deleted` notice, as `IntentPurpose::BeginConversation`'s arm
   navigates; otherwise `refused` set.
3. `sections/project/owner.rs`'s `forms`: before `review_form`, when
   `offered`, a region in `JigStyle::Danger` with `DeleteWhat`, a
   `Role::TextField` bound to a new `JigMeaning::ProjectDeleteName {
   project }` (decoded from `Act::Input { value }` in `binding.rs`, as
   `ProjectServiceName` is), and a `Role::Button` "Delete project" bound
   to a new `Act::DeleteProject { project }`, `State::Disabled` unless
   `ready`, and the pending progress or the refusal beneath.
4. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-client-view --lib -E 'test(/owner_project/)'`;
   - `heavy cargo nextest run -p jig-client-world --test stories -E 'test(/deletes_a_project/)'`
     (binary name from `jig/tests/client/Cargo.toml`);
   - `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-world --test suite -E 'test(/text_pages/)'`,
     then the same without the variable;
   - `heavy cargo nextest run -p jig-client-view -p jig-client-scripted-engine -p jig-client-world -p jig-client-browser-world`;
   - `heavy cargo check --tests -p temper-client-view -p temper-client-tools`
     (temper reads jig-client-view; its focused tests if it needed an arm);
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
5. Commit and push.

## Report

The standard report, plus the goldens that changed and why; how the
page behaves if the project leaves the frame before the answer arrives.
