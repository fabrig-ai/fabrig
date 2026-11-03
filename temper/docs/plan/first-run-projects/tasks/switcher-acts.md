+++
id = "switcher-acts"
part = "switcher"
kind = "feature"
needs = ["first-chat"]
files = [
  "temper/crates/temper-client-domain/src/pages/switcher.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/acts.rs",
  "temper/crates/temper-client-domain/src/tests/pages/switcher.rs",
  "temper/crates/temper-client-domain/src/tests/pages.rs",
]
effort = "high"
minutes = 75
+++

# The banner's switcher: Rename and a new project, the domain's acts

## Goal

The banner's switcher renames the open project in place and makes a new one
(`temper/docs/design/client/README.md`, 5.3, "Projects in the banner"). This task builds its
state and acts in `temper-client-domain`: the rename's form, Rename sending jig's `Rename`, the
new project's form sending jig's `MakeProject` on policy 1 and opening the project it made. The view's
part is `switcher-view`'s.

Acceptance:

- `pages::Act::Switcher(switcher::Act)` drives a `switcher::Switcher` held by `Pages` and read by
  the view through `Pages::switcher()`.
- Rename on the open project, while `me`'s role on it is `HeldRole::Owner`, opens its form with the
  project's name as its words; Save sends one `c::Ask::Rename { project, name }`; answered
  renamed, the form closes; refused, it stays with the refusal.
- The new project's words, always held (its form is always in the list); Create sends one
  `c::Ask::MakeProject { name, policy: 1 }`; answered `ProjectMade { project }`, the words empty
  and the project is opened (chosen once `me` lists it, `Address::Chats { filter: All, start:
  false }` shown); refused, the words stay with the refusal.
- Cancel closes the rename's form and sends nothing; an empty name, or Rename's name unchanged,
  sends nothing; while a request is out of either form, Save and Create do nothing; choosing
  another project or leaving for a page of no project closes the rename's form.
- The tests below pass.

## Inputs

- `temper/docs/plan/first-run-projects/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.3, "Projects in the banner" (all of it), and 3.6 (one act,
  one request).
- jig's `jig/docs/design/domain/people.md`, 5.1 (`Rename` by a project's owners; `MakeProject` by a
  deployment's owner, a name no other project holds) and `jig/crates/jig-client-domain/src/contract/asks.rs`
  (`Ask::Rename`, `Ask::MakeProject`), `contract/outcome.rs` (`Renamed`, `ProjectMade`), read only.
- The patterns to follow: `pages/chats.rs` `send` and its answer (a keyed ask from the pages, its
  pending key, its refusal), `forge/start.rs` `StartIn::ask` (a `MakeProject` from the client), and
  `pages.rs` `choose_project` / `Act::OpenProject` (opening a project).

## Owns and may change

- New `temper-client-domain/src/pages/switcher.rs`, its module doc citing
  `temper/docs/design/client/README.md, 5.3`:

  ```rust
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum Act {
      /// Open the rename's form for the open project.
      Rename,
      /// The rename's field.
      RenameWords { words: Box<[u8]> },
      /// The rename's "Save", or its field's Enter.
      Save,
      /// The rename's "Cancel", or Escape.
      Cancel,
      /// The new project's field.
      NewWords { words: Box<[u8]> },
      /// The new project's "Create", or its field's Enter.
      Create,
  }
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum Form {
      Rename,
      New,
  }
  #[derive(Clone, Debug, Default)]
  pub struct Switcher { /* the rename's project and words, the new project's words, the key out,
                           the refusal and its form, a project to open */ }
  impl Switcher {
      /// The project whose rename's form is open.
      pub fn renaming(&self) -> Option<u32>;
      pub fn rename_words(&self) -> &[u8];
      pub fn new_words(&self) -> &[u8];
      pub fn pending(&self) -> Option<Form>;
      pub fn refusal(&self) -> Option<(Form, &/* the refused ask's type the chats' start keeps */)>;
  }
  ```

  Words are bounded by jig's project name bound (`jig_client::contract::Text` at the bound
  `people::NAME_BYTES` names, or the client contract's equivalent); keys are `Key { asker:
  Owner::Pages.asker(), number }` from `Pages`' serial, as the chats' start draws them. The answer
  arrives through `pages.rs`' `told`; match it by the switcher's key.
- `temper-client-domain/src/pages.rs`: `pub mod switcher;` (or `pub(crate)`, as `choice` is), the
  `Act::Switcher(switcher::Act)` variant, a `switcher` field on `Pages`, `pub fn switcher(&self) ->
  &switcher::Switcher`, the dispatch, the answer in `told`, and the close on a project change
  (`choose_project` / `project_changed`). Opening the made project: if `me` does not list it when
  the answer arrives, keep it and choose it at the `me` update that lists it; say which you did.
  Every exhaustive match over `pages::Act` gains its arm (compiler-forced: `acts.rs`, the pages'
  matches in `pages/chat.rs` and `pages/task.rs`; arms only). If `acts.rs` lists which acts need a
  confirmation or a sign-in, the switcher's acts need neither.
- Tests: new `temper-client-domain/src/tests/pages/switcher.rs`, registered in `tests/pages.rs`
  beside `chats`, in that directory's fixtures' form (a `me` with an owned project and a
  member's project):
  - `rename_sends_jigs_rename_with_the_new_name`,
  - `rename_is_not_opened_on_a_project_the_person_does_not_own`,
  - `an_empty_or_unchanged_name_sends_nothing`,
  - `cancel_closes_the_form_and_sends_nothing`,
  - `a_renamed_answer_closes_the_form`, `a_refused_rename_keeps_the_form_and_its_refusal`,
  - `a_new_project_sends_make_project_on_policy_one`, `an_empty_new_name_sends_nothing`,
    `a_refused_new_project_keeps_its_words_and_refusal`,
  - `a_made_project_is_opened_on_its_chats_page` (the answer before and after `me` lists it),
  - `one_request_at_a_time`, `choosing_another_project_closes_the_form`.

## Steps

1. The module, the act and its dispatch; `heavy cargo nextest run -p temper-client-domain --lib -E
   'test(switcher)'`.
2. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-client-domain --all-targets --
   -D warnings`; the focused tests of `temper-client-domain` and its direct dependents
   (`temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`,
   `temper-client-world`; `heavy cargo check -p temper-client-view --tests` first if the new arm
   reaches the view's matches, whose arms you may add). Commit; push.

The README's status entry for 5.3 closes with `switcher-view`, not here.

## Report

As the implementer's text says, and how the made project is opened when `me` lags.
