+++
id = "rail"
part = "rail"
kind = "feature"
needs = ["addresses", "rail-parts"]
files = ["temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/tests/rail.rs", "temper/crates/temper-client-view/src/tests/frame.rs", "temper/crates/temper-client-view/src/tests/person_menu.rs", "temper/crates/temper-client-shell-web/temper.css"]
effort = "high"
minutes = 220
+++

# The rail as the whole story draws it, from `rail::parts`

## Goal

Draw the rail, the phone's tab bar, the switcher and the rail's foot as
`temper/docs/design/client/README.md`, 5.3, now has them, every part decided by
`temper_client_domain::pages::rail::parts`: Home, Needs you, Work, Ask temper, a
hairline, How work runs; the new box, "+ New draft or change" with its menu or
"+ New change" alone; Deployment at the foot; "All projects" first in the
switcher; "How work runs for PROJECT" and "Deployment" in its menu; the tab bar
Home, Needs you, Work and Ask. Both shells draw it, the terminal as numbered
lines.

Acceptance:

1. On a project's page the rail's children, in order, are the brand link, the
   unfold disclosure where it is drawn today, the switcher, the Away chip
   (browser), the navigation (`RAIL_NAV`), the new box (`RAIL_NEW`), the foot
   (`RAIL_FOOT`) and the night switch (browser), and the navigation holds "Home",
   "Needs you" (with its count), "Work", "Ask temper" while offered, the hairline
   `div.temper-rail-sep`, and "How work runs", each to 5.3's address and current
   where 5.3 says. No entry hides by kind 7.
2. The new box is a skein-ui menu named "New draft or change", its items "Draft",
   "Change" and "Ask temper" with 5.3's lines and addresses, while `parts.new` is
   `DraftOrChange`; a link named "New change" to `NewChat { draft: false }` while
   it is `Change`; nothing while `Hidden`. "Ask temper" in the menu only while
   `parts.ask`.
3. The foot holds, in order, the Away switch while shown, the playground's "Set up
   temper" while it shows today, the link's state, "Behind" where it goes today,
   "Deployment" (to `Accounts`, current on Deployment's pages, carrying the forges'
   and accounts' dots), the person's menu or line, and the lost sign-out's alert.
   The person's menu holds "Accounts" and "Sign out", as today.
4. The switcher's menu holds "All projects" first while `parts.all_projects`, its
   small words "N projects", pressing it going to `Inbox` with no project (5.3,
   *All projects*); the switcher's summary names "All projects" and that row is
   the pressed one while `rail::across`; then the projects; then "Rename PROJECT",
   "New project", "How work runs for PROJECT" (to `Settings { project }`) and
   "Deployment" (to `Accounts`).
5. The tab bar is "Home", "Needs you", "Work" and "Ask" (while `parts.ask`); while
   To begin shows, "Home", "Ask" and "Deployment"; the rail while To begin shows,
   "Home" and "Ask temper" alone, no new box, the foot as above.
6. The terminal prints the same parts as numbered lines, the menu's items as it
   prints a menu today.
7. `temper.css`: the rail's new rules under a new marker `/* ==== ui: rail ==== */`,
   placed just before `/* ==== ui: switcher ==== */`: the hairline, the new box's
   button filling the dashed box with its "+ " drawn by style, "Deployment" as the
   foot's rows; `.temper-rail-new-all`'s rules gone; the tab bar four columns, and
   three while it holds no fourth link (browser.md, 2.5).
8. The focused tests of `temper-client-view` and `temper-client-gallery` pass, and
   `temper-client-shell-web`'s, clippy clean; the README.status.md entry "**Not
   yet:** 5.3, *The rail*, *The rail's parts* and *Projects in the rail*" is gone.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`; `contracts.md`, C3 and C6.
- `temper/docs/design/client/README.md`, 5.3: *The rail* (all its items), *While To
  begin shows*, *The rail's parts*, the paragraph on the phone's tab bar,
  *Projects in the rail*, *All projects*.
- `temper/docs/design/client/browser.md`, 2.5: *The project's frame* (the tab bar's
  four columns) and *The rail as the whole story draws it*.
- The look: each page's `nav.rail` in `temper/docs/design/client/web/whole-story/`
  and `pair/` and `team/` beside it, and `whole-story.css`'s `.rail`, `.nav`,
  `.nav .sep`, `.new`, `.foot`.
- The code: `frame.rs`, `rail`, `entries`, `shown`, `goals_shown`,
  `goals_current`, `settings_link`, `person`, `tabbar`, `switcher`;
  `temper_client_domain::pages::rail` (rail-parts); `temper.css`'s rail rules
  (around "#app > .temper-rail", the tab bar's column rules and the marker list).

## Owns and may change

Owns: `temper-client-view`'s `frame.rs` and its tests `src/tests/rail.rs`,
`src/tests/frame.rs` and `src/tests/person_menu.rs`; `temper.css`'s rail and tab
bar rules.

Reader and neighbour files, and what may change there:

- `temper-client-view`'s `words.rs`: the rail's new words, keys from 4600 to 4649
  (C6), beside `RailNewAll`'s lines; the rail-only words that go (`RailNewMenu`,
  `RailNewChat`, `RailNewChatDescription`, `RailNewGoal`,
  `RailNewGoalDescription`, `RailNewAll`, `SettingsFor`) removed in their four
  places. Reuse a word that says the same ("How work runs" is
  `SettingsHowTab`'s; "Ask temper" `AskTitle`'s).
- `look.rs`: `RAIL_SEP` and any other new style in slots 646 to 649 (C6);
  `RAIL_NEW_ALL`'s slot renamed `temper-reserved-N` where its rules go, its
  constant removed: a name never moves in `NAMES`.
- Other tests of `temper-client-view` that assert the rail or the tab bar, their
  rail lines alone: `tests/home.rs` (`goals_takes_chats_place`,
  `to_begin_cuts_the_rail_on_home_and_ask_alone`,
  `settings_ask_link_opens_the_delivered_asking_chat_in_both_shells`),
  `tests/ask.rs` (`settings_is_current_in_the_rail_and_phone_tabs`,
  `asking_chat_picker_applies_to_its_next_run`'s rail "Settings"),
  `tests/settings.rs` (`start_goes_to_ask_and_go_on_to_the_open_chat`;
  `each_flow_draws_its_path_in_the_browser_and_none_in_the_terminal`'s rail-current
  lines alone, not its path's, which `deployment` changes),
  `tests/changes.rs` (`changes_is_current_after_chats_in_the_rail_and_absent_from_the_tabbar`,
  now Work current),
  `tests/chat.rs` (`goals_navigation` and its callers;
  `only_a_delivered_template_five_executor_changes_chat_path_and_navigation`),
  `tests/builders.rs` (`the_tab_has_no_act`'s rail lines), `tests/forges.rs`
  (`the_navigation_has_forges_with_its_dot_and_words`), `tests/look.rs`
  (`RAIL_NAMES`, the token table). Keep what each checks for the page.
- `temper/tools/gallery`: its samples' `open_menu` (`rail-new-menu` opens "New
  draft or change"), and `src/tests.rs`'s rows and tests that assert the rail or
  the tab bar (`rail-new-menu`, `rail-person-menu`, `rail-away-*`, `night-home`,
  `night-ask`, `home-begin-ask-left`, `ask-first-run`, `ask-typing`,
  `ask-sending`, `ask-refused`, the print test's rail lines,
  `the_rail_menu_sample_opens_the_menu`). Rows alone; new states are
  `gallery`'s. `each_settings_switch_off_keeps_its_value_and_changes_its_navigation`
  is `work-page`'s, beside you (the switches now hide Work's filters): leave it,
  and list it under `## Left` if it fails on your branch.
- `temper-client-shell-web`'s `tests/suite/switcher.rs` and `main.rs` marker
  checks, if the new marker moves what they read.
- `temper-client-shell-terminal`'s `src/tests/render.rs`
  `the_first_run_prints_settings_and_the_playground_banner`'s rail line ("]
  Settings" becomes "] Deployment"). Its goldens are `regenerate`'s: leave them
  stale and list them under `## Left`.

Not yours, left stale and listed under `## Left`: the client world's, browser's,
real loop's, one process's and `temper`'s process stories that press "New chat",
"Settings", "Goals" or "Chats" in the rail (`readers`); the terminal goldens and
the journeys (`regenerate`). Team: `parts.team` is never true yet (C3), and Team
has no page: draw nothing for it, and leave no code for it.

Pre-approved options:

- **If skein's menu button can show only its name:** name it "New draft or
  change" and draw the "+ " by `::before`, as the night switch draws its icon.
- **If "All projects"'s count words need a plural rule:** use the phrasebook's
  plural as the projects' other counts do; it shows from two.
- **If the tab bar's To begin form must say "Deployment" where it said
  "Settings":** it does (5.3); its link goes to `Accounts`.
- **If a gallery row's oracle compares the whole rail list:** rewrite the list to
  5.3's, keeping the row's other checks.

## Steps

1. **Read the parts** once per build: `let parts = rail::parts(&rail::facts(root,
   project, times.now))`, in `rail` and `tabbar`, and draw from it alone; delete
   `goals_shown`, `shown`'s kind 7 reading and `Shown.changes`.
2. **The navigation** in 5.3's order, the hairline a `Role::Slot` with
   `RAIL_SEP` and no name; current by 5.3: Work on `Work`, `Goals`, `Changes`,
   `Chats`, `NewChat`, a `Chat` but an Ask temper chat's, and `Task`; How work runs
   on `Settings`, `Repositories`, `Adopt`; every match on `Address` exhaustive.
3. **The new box** by `parts.new`, as acceptance 2.
4. **The foot:** `settings_link` becomes the Deployment link, with its dots and its
   current set (Accounts, Forges, Models, Builders, Members, Limits, ConnectForge,
   ConnectModel).
5. **The switcher:** "All projects" first, as acceptance 4, a `Role::Button` bound
   to `PageAct::Go`; its small words as a project row's repository words are drawn
   in each shell; "How work runs for PROJECT" and "Deployment" after the forms.
6. **The tab bar** by the parts, as acceptance 5.
7. **`temper.css`**, as acceptance 7. Remove replaced rules where they stand.
8. **Tests.** In `tests/rail.rs`, rewrite the rail's order-and-current table
   (`rail_and_tabbar_keep_the_order_addresses_current_states_and_count`) to 5.3, a
   row per address group; the new box's tests (`the_rails_new_chat_is_a_link_and_a_menu`,
   `its_items_go_where_they_say`, `goal_shows_while_drafts_are_offered`,
   `all_ways_goes_to_settings`) become one table over `DraftOrChange` and `Change`
   with the items, their lines and addresses; the kind 7 tests
   (`the_rail_shows_what_kind_seven_shows`, `until_read_the_optional_entries_are_hidden`,
   `a_page_keeps_its_entry_while_on_it`, `the_tab_bar_by_goals`) become one row each
   that kind 7 changes nothing in the rail; the switcher's panel test gains "All
   projects" and the two links; the terminal's numbered rail test its new lines.
   Break two rules by hand (Work not current on a `Task`; "All projects" shown with
   one project) and see their rows fail; restore.
9. **Readers** as Owns says, then **check** as `notes/common.md` says, for
   `temper-client-view`, `temper-client-gallery` and `temper-client-shell-web`
   (`--lib` and its suite), and `heavy cargo check --tests` for
   `temper-client-shell-terminal`, `temper-client-world`,
   `temper-client-browser-world` and `temper`. Remove the status entry. Commit and
   push: the message says the rail draws from one function of its parts and holds
   the whole story's entries.

## Report

Follow the implementer's text. Also give the rules you broke and the failures you
saw; the words added and removed with their keys; the style slots taken; and,
under `## Left`, each stale golden and each story in another package that presses
an old entry, by file and line.

## Rebase (2026-11-08, step merge, foreman ui-p1-rail/rail/foreman-1)

combine rail and Deployment vocabulary counts in words.rs
Resolve `temper/crates/temper-client-view/src/words.rs` against integration commit `bf66c99c4d` (“Move deployment tabs from project Settings into Deployment's frame”).
The conflict is in shared vocabulary totals: both tasks changed `PHRASES`, `SEGMENTS`, `ENGLISH_BYTES` and `ALL`’s array length.
Preserve both tasks’ variants, IDs, English phrases and `ALL` entries, including rail’s removals and Deployment’s updated wording.
For the refused tips combined, set `PHRASES = 2133`, `SEGMENTS = 3762`, `ENGLISH_BYTES = 52250`, and `ALL: [Word; 1451]`.
Keep rail keys in 4600–4649 and Deployment keys in 4750–4799, as contracts C6 requires.
Verify the combined totals through the existing `english_declarations()` assertion in `src/tests/accounts.rs`, then complete the spec’s focused checks.
The other paths in the refusal auto-merged; preserve their combined changes.
