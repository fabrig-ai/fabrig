+++
id = "pg-project-page"
part = "playground"
kind = "feature"
needs = ["pg-words"]
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/project_settings.rs",
  "temper/crates/temper-client-view/src/pages/project.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/tests/project_settings.rs",
]
effort = "high"
minutes = 45
+++

## Goal

Give temper's client a project's settings page, `/projects/<project>/settings`,
the page a project's Delete goes on (draft
`temper/docs/design/draft/playground-reset.md`, 5, "Its client"; the
client design leaves a project's settings page later:
`temper/docs/design/client/README.md`, section 11, "Pages the mockups do
not show"). This task makes the page with the project's name and
number, reached from the navigation beside "Adoption"; its Delete is
`pg-project-delete`'s.

Acceptance:

- `Address::ProjectSettings { project }` prints as
  `/projects/<project>/settings` and parses back, a trailing slash
  allowed as for the others; a project number that is not a decimal
  `u32` is `Place::Missing`, as `/projects/x/adopt` is;
- the page draws one `main` with a level-1 heading "`NAME` · settings"
  and the paragraph "Project `N`", from `me`'s projects; for a project
  `me` does not list (none, or deleted), the heading is "Settings" and
  the paragraph "This project is gone.";
- the navigation, where it draws "Adoption" for the current project,
  draws a "Settings" link to that project's settings page after it,
  `Current` while that page is shown;
- the tests below pass, and every existing test of the packages
  changed and of their dependents.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md`.
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `temper/docs/design/client/README.md`, 3.3 (routing), 3.5 (pages),
  5.3 (navigation); `temper/docs/design/client/browser.md`, 2.3 (paths).
- The words `ProjectSettings`, `ProjectSettingsTitle`,
  `ProjectSettingsNumber` and `ProjectGone`, added by `pg-words`
  (`temper/crates/temper-client-view/src/words.rs`).

## Owns and may change

Owned: the `files` above (the two test files and
`temper-client-view/src/pages/project.rs` are new).

Neighbour files, edits allowed:

- `temper/crates/temper-client-domain/src/tests.rs` and
  `temper/crates/temper-client-view/src/tests.rs`: one `mod
  project_settings;` line each, in alphabetical place;
  `temper/crates/temper-client-view/src/pages.rs`: one `pub(crate) mod
  project;` line.
- Every file of the temper tree with an exhaustive `match` on
  `temper_client_domain::Address` (`departure.rs`, `route.rs`,
  `forge.rs`, `forge/picker.rs`, `pages/connect_model.rs` in the domain;
  `choice.rs`, `pages/connect_model.rs`, `forge/connect.rs` in the view;
  `temper/tools/gallery/src/samples.rs`;
  `temper/tests/client/src/front/account_acts.rs`;
  `temper/tests/one-process/src/m15/pages.rs`, `src/stories.rs`): the
  compiler-forced arm, which does what the arm of `Address::Accounts`
  does there (a page with no forge state). In the gallery's
  `samples.rs` path match the arm is `("project-settings", None)`; add
  no gallery sample.

Pre-approved:

- the page keeps `Pages::project` as it is: opening
  `/projects/N/settings` for a project `me` lists also makes it the
  current project, as opening a chat of it does, if the code makes
  that one line (default: set `pages.project` when `me` lists it);
  otherwise leave the current project and say so;
- if a shell (terminal, web, CLI) matches `Address` exhaustively in
  non-test code, its forced arm follows `Accounts`'; default: none does.

## Steps

1. **Address** (`temper-client-domain/src/address.rs`):

   ```rust
   pub enum Address {
       // ...after Adopt:
       /// A project's settings (draft playground-reset.md, 5).
       ProjectSettings { project: u32 },
   }
   ```

   `print`: `(b"projects", Some(project), b"settings")`; `parse`: the
   arm `(b"projects", _, b"settings")` beside `adopt`'s, through
   `project_decimal`. The module doc's path list gains
   `/projects/<project>/settings`.
2. **Pages** (`temper-client-domain/src/pages.rs`): the `Went` arms
   (the address is kept; no page state; the pre-approved current
   project); every other `Address` match gets `Accounts`' arm.
3. **View page** (`temper-client-view/src/pages/project.rs`, new):

   ```rust
   //! A project's settings page (draft playground-reset.md, 5).
   pub(crate) fn settings(
       b: &mut skein_ui::Builder,
       root: &temper_client_domain::Domain,
       w: &crate::words::Words,
       bindings: &mut crate::frame::Bindings,
       project: u32,
   )
   ```

   It opens `Role::Main` with `look::MAIN`, finds the project in
   `root.pages().me(root.client())`'s `projects`, and draws the heading
   and paragraph of the acceptance (the number through
   `skein_lib::Decimal`). `bindings` is unused here but is the
   signature `pg-project-delete` draws its Delete with; keep it (an
   `_bindings` name is fine).
   `root.rs`: the page's branch beside `Adopt`'s, and the address joins
   the arm that draws `choice::banner` and `frame::demo_banner`.
   `frame.rs`: the "Settings" link after "Adoption" (`action(...,
   Role::Link, ProjectSettings, Act::Go { address:
   Address::ProjectSettings { project } }, current, ...)`), and the new
   address in each of `frame.rs`' `Address` matches as `Accounts`' arm
   but for the link's own `current`. `demo_banner`'s address match gives
   the page the banner (`Adopt`'s arm).
4. **Tests** (temper's client domain and view unit tests,
   `temper/docs/design/domain/testing.md`'s client tier):
   - `temper-client-domain/src/tests/project_settings.rs`:
     `project_settings_prints_and_parses_back` (numbers 1 and
     `u32::MAX`, with and without a trailing slash, and the query
     ignored); `a_project_settings_path_without_a_number_is_missing`
     (`/projects/x/settings`, `/projects//settings`,
     `/projects/4294967296/settings`).
   - `temper-client-view/src/tests/project_settings.rs`, building a
     root with `me` listing project 2 named "widget" through the
     existing fixtures (`tests/fixtures.rs`' `root_at_with_forges` or
     the closest that delivers `me` with named projects; read only, do
     not change `fixtures.rs`):
     `the_settings_page_names_its_project` (heading "widget ·
     settings", paragraph "Project 2");
     `a_project_me_does_not_list_is_gone` (address for project 9:
     heading "Settings", "This project is gone.");
     `the_navigation_links_the_current_projects_settings` (on a chats
     page of project 2: a link "Settings" whose action decodes to
     `Act::Go { address: Address::ProjectSettings { project: 2 } }`;
     on the settings page it is `State::Current`).
5. Checks, once: `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-client-domain`, `-p
   temper-client-view`; then `heavy cargo check --tests` for each of
   `temper-client-tools temper-client-shell-cli
   temper-client-shell-terminal temper-client-shell-web
   temper-client-world temper-client-gallery temper-one-process-world`,
   one at a time. Commit, push.

## Report

fab's sections; `## Interfaces`: `Address::ProjectSettings`,
`pages::project::settings`'s signature; each neighbour file given a
forced arm.
