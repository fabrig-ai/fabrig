+++
id = "settings-frame"
part = "settings-frame"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/departure.rs",
  "temper/crates/temper-client-domain/src/pages/connect_model.rs",
  "temper/crates/temper-client-domain/src/pages/switcher.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-domain/src/tests/pages/project_pages.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/accounts.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/forge/forges.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/tests/accounts.rs",
  "temper/crates/temper-client-view/src/tests/forges.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-view/src/tests.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 170
+++

# Settings' frame, its tabs and Repositories

## Goal

Every tab of a project's settings is a project page in the rail's frame, "Settings" current, with its kicker, heading, lede
and tabs. Repositories is whole. `Forges` and `Accounts` draw inside the frame while a project is open, and "Sign out" is at the
`Accounts` tab's foot. How work runs here holds only its "Configure temper" row: `how-work-runs` fills it.

Acceptance:

- `Address::Repositories { project }`, printed and parsed at `/projects/<project>/settings/repositories`. It opens its project as `Settings` does
  (`address_project`); an unknown project says what `Settings` says.
- **The frame:** `pages::settings::open`, by `temper/docs/design/client/README.md`, 3.5, *Settings*: a `.col`; `.kicker` "PROJECT · settings";
  the table's heading (level 1) and lede; a navigation named "Settings", `.tabs`, links, the current `State::Current`, in the order How work runs,
  Repositories, Forges, Accounts, "Forges" and "Accounts" with the rail's dots (`forge_mark`, `account_mark`). No "Builders": `ui-settings-tabs` adds it.
- **How work runs** (`Address::Settings`): under the tabs, `.settings`, its first section with no title: one `.setting`, "Configure temper", `small` "Tell
  it how you work, in your own words or by pointing at a document. It proposes the settings; nothing changes until you accept.", and at the right, justified
  to its end, a `.btn.small` primary link: "Start", to `Configure { project }`; or with a configuring chat of the person's open in the project (`open_configuring`,
  on the trunk), "Go on", to that `Chat { task }`. `settings_controls` goes.
- **Repositories:** by 3.5, *Repositories*. The rows are the project's: `listed::rows` and each row's `repositories` where `project` matches, and the playground's project
  from the playground's read, as `frame.rs`'s `project_row` reads `listed::held`. With the watch refused, ended or not delivered: the head and no rows, no band.
  `.lines` rows are no links.
- **`Forges` and `Accounts`:** whenever `rail_project` is `Some`, inside the frame, their level-1 heading and `PAGEBAR` gone, their rows under the tabs as they are today:
  `ui-settings-tabs` restyles them. With `None`, byte for byte as today.
- **Sign out,** in the frame alone, the last of the `Accounts` tab: a `.setting` row, "Signed in as NAME", its login (`me.party.login`) in `small`, a `.btn.small` "Sign out"; while
  `sign_out_lost`, "Sign out did not reach the engine." as an alert there. The rail's foot keeps its own line of that (5.3, 6.6).
- **"Settings" is current** on `Settings`, `Repositories`, `Forges`, `Accounts`, `ConnectForge`, `ConnectModel`, `Adopt`, `Configure` and a configuring `Chat`, in the rail and the tab bar. The banner's
  `Forges` and `Accounts` keep theirs, with no project.
- **Choosing a project:** `Repositories` keeps its tab, as `Settings` does (`Act::Project`, `pages.rs`). `Forges` and `Accounts` stay, and only the words naming the project change. The switcher's own
  `Address` arms: `Repositories` as `Settings`.
- **Going back:** `Configure`'s path and back link still go to `Settings { project }`.
- **The look,** under `/* ==== ui: settings ==== */`, from `temper/docs/design/client/greenfield/mock.css`, exact: its `.col`, `.intro`, `.kicker`, `.headline`, `.lede`, `.tabs`, `.settings`, `.setting`, `.section`, `.section-head`, `.section-title`, `.lines`, `.when`, `.band`, `.quiet-band`, `.btn`, `.btn.primary`, `.btn.small`, on
  `SETTINGS_*` tokens and `temper-settings*` names, and the rules for 760 px and under: a gap of 32, tabs wrapping, `.setting` one column. Where the trunk's `--gf-*` tokens have a value, use them, as the
  other `ui:` blocks do.
- **The gallery:** `settings` drops its old parts for the frame's and "Configure temper"; new `repositories`, `repositories-none`, `forges-in-settings`, `accounts-in-settings` samples, each with its `Holds` row, and the page-heading test's new heading. `names.txt` sorted.
- **The terminal** keeps printing: `each_fixture_page_prints_as_its_golden_file` passes; the tabs print as links; a new golden `settings.txt`, a fixture of How work runs, in `tests/render.rs`.
- The tests below pass and bite. `README.status.md` and `browser.status.md` lose what this closes.

## Inputs

- `temper/docs/plan/ui-settings/notes/common.md`; `temper/docs/plan/ui-settings/notes/contracts.md`, *The address and the frame*.
- `temper/docs/design/client/README.md`, 3.5's table and its *Settings*; 5.3.
- `temper/docs/design/client/browser.md`, 2.3, 2.5 *Settings' look*, 7.2.
- The mock: `temper/docs/design/client/greenfield/settings.html`; `app.js`, `settingsPage`; `chats.html`'s band; `mock.css`.

## Owns and may change

Owns `files`. May also change:

- **`Address`'s arms,** compiler-forced, in any file of `temper/`: only the new arm, beside `Settings`'s.
- **`bindings`:** `pages::accounts::main` now takes the pages' `Bindings` as well as its own, `forge::forges::main` its `project`, and `root.rs` passes `rail_project`; the terminal's and the web shell's calls follow.
- **The bound of bindings or nodes** (`temper-client-view/src/limits.rs`; `temper-client-shell-web/src/limits.rs`; `temper-client-tools/src/limits.rs`; `temper-client-shell-terminal/src/limits.rs`), if a page's count trips: follow its formula.
- **Tests that walk the old page:** any test in `temper/tests/client/**`, `temper/tests/client-browser/**`, `temper/crates/temper/tests/**` or `temper/tools/journeys/**` that presses a link on `Settings` (`Forges`, `Accounts`, `Adopt a repository`) or `Sign out` there, or reads the old level-1 heading on `Forges` and `Accounts`: reach it the new way, and name them in the report. Journeys' prints regenerate, by their own tool.
- **Design status:** `temper/docs/design/client/README.status.md`, `browser.status.md`, for the lines this closes.

Pre-approved:

- **The trunk moves:** rail and entries may have changed under `ui-needs`, `ui-goals`, `ui-changes` by your base (`Address::Goals`, `Address::Changes`): `Repositories` follows `Settings` in every such match, and "Settings" stays current as above.
- If a token number you take is taken on a catch-up, move yours.
- If `.lede`, `.kicker` etc. of another plan already exist on the trunk under that plan's token, don't share them: `SETTINGS_*` stays its own, as `notes/common.md` says.
- If "To begin" or Home's links name `Settings` for a thing that moved (`Forges`, Accounts): leave them, the addresses hold.

## Steps

1. Domain: `Address`, its arms, its tests (`tests/address.rs`: `/projects/3/settings/repositories` both ways; a neighbour path `/projects/3/settings/other` reads as today). `pages.rs`: `address_project`, `Act::Project`, the arm making `chats` stays `Settings`' and `Configure`'s.
2. View: `pages/settings.rs` by `notes/contracts.md`; `pages.rs`'s `mod`; `pages/home.rs` loses `settings` and `settings_controls`; `root.rs`'s dispatch; `frame.rs`'s `settings_link`; `look.rs`, `words.rs` (each heading, lede, tab, row). Reuse `frame::forges_link` and `accounts_link` for the marks where they fit; if they can't draw a tab's look, draw the link in `settings.rs` and call `forge_mark`, `account_mark` as `pub(crate)`.
3. `forges.rs`, `accounts.rs`: the frame when `project` is `Some`.
4. Stylesheet; render the three widths against the mock (`notes/common.md`, *Render proofs*) as a check for yourself; the proof of record is `settings-proof`'s.
5. Tests, `tests/settings.rs`, a new module of `tests.rs`:
   - `every_tab_draws_its_head_and_tabs_in_order`: the four tabs' kicker, heading, lede and current tab.
   - `start_goes_to_configure_and_go_on_to_the_open_chat`.
   - `repositories_lists_the_projects_and_no_others`, `repositories_says_none_with_a_quiet_band`, `the_playgrounds_project_shows_its_repository`, `a_repository_row_is_no_link`.
   - `forges_and_accounts_are_in_the_frame_with_a_project_and_as_they_were_without`.
   - `sign_out_is_at_the_foot_of_accounts`, with its login; `sign_out_lost_moves_with_it`.
   - In `tests/rail.rs`: `settings_is_current_on_every_page_of_settings`.
   - Domain, `tests/pages/project_pages.rs`: `choosing_a_project_keeps_the_repositories_tab`, `choosing_a_project_stays_on_forges_and_accounts`.
6. `cargo fmt`; old forms; clippy for `temper-client-domain`, `temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`, `temper-client-gallery`; nextest of those and `temper-client-world`; `heavy cargo check -p temper --tests` and `heavy cargo check -p temper-client-browser-world --tests`, whose stories press "Settings" then "Sign out" (`browser_public.rs`, `browser_attacks.rs`, `browser_sign_in.rs`, `browser_accounts.rs`): "Sign out" is now on the `Accounts` tab, so they press "Settings", "Accounts", then it. Commit and push.

## Report

As the implementer's text says, with the old tests you changed and how they reach a page now.
