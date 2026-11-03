+++
id = "switcher-view"
part = "switcher"
kind = "feature"
needs = ["switcher-acts", "skein-form-escape"]
files = [
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/**",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# The banner's switcher: Rename and a new project, drawn and bound

## Goal

The banner's switcher offers Rename on the open project's row to its owners, the rename's form in
place (Save, Cancel, Escape), and the new project's form at the list's end, bound to
`switcher-acts`' `pages::Act::Switcher` (`temper/docs/design/client/README.md`, 5.3, "Projects in
the banner"). The switcher's look (`details.temper-projects`, its classes, the stylesheet) is the
look plan's (`look-welcome-chats`, `notes/common.md`): this task draws the nodes, names and acts
the design gives, in the banner the trunk has.

Acceptance:

- The open project's row offers "Rename" while `me`'s role on it is owner, and not otherwise;
  pressed, `Act::Switcher(switcher::Act::Rename)`.
- While `Pages::switcher().renaming()` names the open project, its row is a `Role::Form` named
  "Rename PROJECT" holding the name's field (its value the switcher's rename words, focused once
  as the form opens), "Save" and "Cancel"; the field's input gives `RenameWords`, its submit and
  "Save" `Save`, and "Cancel" `Cancel`; "Cancel" names `Shortcut::ESCAPE`, so the page's outline
  holds (skein's `ui.md`, 6.1 and 14). While a rename is pending, "Save" is disabled; a refused
  rename's words show under the field.
- The list ends with a `Role::Form` named "New project", its field's placeholder "+ New project",
  its value the switcher's new words, and "Create"; input gives `NewWords`, submit and "Create"
  `Create`; disabled while pending; a refusal's words under the field.
- The tests below pass; the status entry below is gone.

## Inputs

- `temper/docs/plan/first-run-projects/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.3, "Projects in the banner"; 5.1, *a banner* (read
  only: the look's).
- `skein/docs/design/ui.md`, 6.1 ("Escape", inside a form) and section 14 (the outline's rules).
- `temper-client-domain`'s `pages/switcher.rs` (built by `switcher-acts`), read only.

## Owns and may change

- `temper-client-view/src/frame.rs`:
  - The banner's projects. **If the trunk's banner draws the look plan's switcher** (a
    `Role::Disclosure` of projects inside the menu, with a rename control on the open row and a
    new project's form, from `look-welcome-chats`): bind its existing nodes to the acts above,
    adding only the nodes it lacks (the rename's form and its Cancel, the refusal's words), in
    its order and styles. **If it still draws one button per project** (`project_buttons`): keep
    those buttons as they are, add after the open project's button its "Rename" (owners only)
    or, while renaming, the rename's form in its place, and after the last button the new
    project's form, with no new style. Say in the report which banner you found.
  - `Bindings::decode`: the switcher's field inputs and submits map as `ChatWords`' do
    (`Input` gives the words, `Submit` gives `Save` or `Create`), presses as now.
  - The banner reads the role from `me` (`MeProject.role == HeldRole::Owner`), as the context line
    reads it (`pages/chat.rs`, `context`).
- `temper-client-view/src/words.rs`: the words "Rename", "Rename PROJECT" (a hole for the
  project's name), "Save", "Cancel" (reuse a word that already says it), "New project",
  "+ New project", "Create", in the phrasebook's form, both words sets; a refusal's words from the
  phrases jig's words give for a refused ask, as the chats' start shows its refusal.
- `temper-client-view/src/tests/frame.rs`, in its fixtures' form: `an_owner_is_offered_rename_on_the_open_project`,
  `a_member_is_offered_no_rename`, `the_rename_form_binds_its_field_save_and_cancel`
  (its Cancel names Escape; the tree passes skein-ui's outline), `the_new_project_form_ends_the_list`,
  `a_pending_rename_disables_save`, `a_refused_new_project_shows_why`. The domain's state for
  these comes from driving `pages::Act::Switcher` through the fixture root, or from a fixture
  that sets it, as the existing frame tests drive their acts.
- Goldens: the terminal shell's goldens that print the banner
  (`temper-client-shell-terminal/src/tests/golden/`, `first_run.txt` and others) gain the new
  project's form and, for an owner, "Rename": regenerate them with `SKEIN_CODEGEN_WRITE=1` on
  `render.rs`'s `each_fixture_page_prints_as_its_golden_file` (`notes/common.md`), keeping only
  the banner's lines. A journey's recorded print (`temper/docs/design/client/journeys/`) is the
  look plan's: if `temper/tools/journeys`' test now fails on the banner, list it under `## Left`
  with the failing journey, and do not reprint it.
- Status, as `notes/common.md` says: `temper/docs/design/client/README.status.md`, **Not yet**:
  remove `5.3, Projects in the banner: the switcher's Rename and new project: no such acts in
  \`temper-client-domain\`'s \`pages.rs\`, and the banner draws one button per project
  (\`temper-client-view\`'s \`frame.rs\`, \`project_buttons\`).` If the banner you found draws
  the open project alone (the look's), remove the whole sentence; if it still draws every
  project, keep `5.3, Projects in the banner: the banner draws one button per project
  (\`temper-client-view\`'s \`frame.rs\`, \`project_buttons\`), not the open one alone with its
  switcher.` in its place.

## Steps

1. The banner and its bindings; `heavy cargo nextest run -p temper-client-view --lib -E
   'test(frame)'`.
2. The goldens; `heavy cargo nextest run -p temper-client-shell-terminal --lib -E 'test(render)'`.
3. The status edit; the citations check.
4. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-client-view -p
   temper-client-shell-terminal --all-targets -- -D warnings`; the focused tests of
   `temper-client-view` and its direct dependents (`temper-client-shell-web`,
   `temper-client-shell-terminal`, `temper-client-world`); `heavy cargo nextest run -p
   temper-client-journeys` once, reporting what it finds. Commit the code and tests, then the status text; push.

## Report

As the implementer's text says, which banner you found, and the goldens and journeys that moved.
