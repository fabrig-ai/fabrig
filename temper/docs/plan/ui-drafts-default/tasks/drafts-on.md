+++
id = "drafts-on"
part = "drafts-on"
kind = "feature"
needs = ["configuring-offers"]
files = [
  "temper/crates/temper-client-domain/src/forge/ways.rs",
  "temper/crates/temper-client-domain/src/tests/forge_ways.rs",
  "temper/crates/temper-engine-domain-forge/src/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/settings.rs",
  "temper/crates/temper-service/src/stock/kinds.rs",
  "temper/crates/temper-client-tools/src/settings.rs",
  "temper/crates/temper-settings-form/src/fields/jig.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/**",
  "temper/tests/client/tests/suite/settings_tools.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/help/settings.md",
  "temper/docs/help/reference/settings-fields.md",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 150
+++

# A new project has `drafts` on, and a project holding a goal shows Goals

## Goal

Kind 7's stock has `drafts` true, everywhere that stock is written, and the setting stays so that a project can turn it off.
A fresh project then offers "Start goal…" with Goals hidden, and Home hides its Goals section while To begin shows, so a
project holding a goal shows Goals in its bar whatever the setting says: the setting hides an offer and not a person's own
work. `temper/docs/plan/ui-drafts-default/plan.md` has the reading behind that.

Acceptance:

- **The stock:** `Ways::STOCK`, `project_stock(7)`, the tools' `stock(7)` and the kinds' `value_bytes` literal all have `drafts`
  true and every other switch false, `way` 0. Each value but the literal's is pinned by a row that bites.
- **Off stays off:** a project whose kind 7 has `drafts` false offers no "+ Draft" on a chat that is no configuring chat, no
  Drafts filter, no draft lede, no "Start goal…", no rail's "Goal".
- **The bar:** with Goals hidden and To begin showing, a project holding a goal has "Goals" in the rail in both shells and in the
  browser's tab bar (then "Home", "Needs you", "Goals", "Chats"). A project holding none, Goals hidden, has neither.
- **Words:** `temper/docs/help/settings.md` and `temper help settings-fields` say the new stock.
- **Prints:** goldens and every journey print are regenerated; `temper-client-journeys` exits 0 and its tests pass. `chat-draft`
  walks with no toggle, as below.
- **Stories:** any browser or process story that reads the old default is changed and compiled, not run
  (`heavy cargo check -p temper-client-browser-world --tests`). None is known: the fake box's walks name their kind 7 outright.
- **This changes what a shared item means,** so the focused suites of every reader run: `temper-client-domain`,
  `temper-client-view`, `temper-client-tools`, `temper-client-world`, `temper-engine-domain-forge`, `temper-engine-domain`,
  `temper-engine-domain-world`, `temper-service`, `temper-settings-form`, `temper-client-shell-terminal`,
  `temper-client-shell-web`, `temper-client-gallery`, `temper-client-journeys`, and `temper`'s with `--lib`.

## Inputs

- `temper/docs/plan/ui-drafts-default/notes/common.md`, and the request's `Recording flakes` line.
- `temper/docs/plan/ui-drafts-default/plan.md`, *Can a goal started there be found?*
- `temper/docs/design/client/README.md`, 3.5, *A chat's document* (*Offered*); 5.3 (the rail's entries and the tab bar); 5.13
  (*What the settings change*, and the paragraph ending "a value of none reads as the stock").
- `temper/docs/design/protocol/configuration.md`, 13.4: the kind 7 row, and *A project's kinds 6 and 7 have no row in the stock*.
- `temper/docs/design/client/journeys.md`, 8, 8.1, 8.2, and section 9's `chat-draft` row.

## Owns and may change

Owns, as `files` lists them. Departures from the implementer's text, for this task:

- `temper/docs/help/settings.md` is yours, for the one sentence below.
- `temper/docs/design/client/journeys/**` is yours: `chat-draft`'s `recording`, `setup` and `README.md` as below, and every
  print the tool writes.
- Remove these **Not yet** entries (the note's departure): in `temper/docs/design/client/README.status.md`, "kind 7's stock:
  `Ways::STOCK`" and "Goals in the bar of a project holding a goal"; in `temper/docs/design/protocol/configuration.status.md`,
  "kind 7's stock has `drafts` false".

May also change, for the test or call site alone: `temper/tests/client/tests/suite/**` and `temper/tests/client/src/**` (an
expected page), `temper/tools/gallery/src/samples.rs`, `worlds.rs` (a sample that means the off state naming it),
`temper/crates/temper-client-domain/src/tests/**`, `temper/crates/temper-client-shell-web/tests/suite/**`.

`temper/crates/temper-client-view/src/pages/settings.rs` needs no change: until kind 7 is read its controls draw the stock,
disabled, as 5.13 now says; a test that expected Off there changes.

Options, each pre-approved:

- **A fixture reading the stock:** by default it keeps `Ways::STOCK`, a fresh project, and the expected words change with it;
  a test whose point is the off state names `drafts: false`. If one fixture's new default would change more than a dozen
  unrelated expectations in a file, give that fixture `drafts: false` and say so.
- **`home.rs`'s fixture:** `StatusFixture.goals` feeds both `Ways.goals` and the goal deliveries. To hold a goal with Goals hidden,
  add a field `shown: bool` that `home_root` reads, each literal taking `shown: goals`. If another shape is smaller, take it.
- **`chat-draft`:** if `again` can't walk the edited recording for a reason other than a moving mark, keep the walk as it was, add
  `toggle checkbox "Chats can write drafts"` to `setup` before its `reload`, so that the page starts Off and the recorded switch
  holds, leave `README.md`'s marks as they are, and say so in the report.
- **Another journey named stale by the reprint:** `again NAME`, and say so.

## Steps

1. **The stock.** `drafts: true` in:
   - `temper/crates/temper-client-domain/src/forge/ways.rs`, `Ways::STOCK`;
   - `temper/crates/temper-engine-domain-forge/src/settings.rs`, `project_stock`;
   - `temper/crates/temper-service/src/stock/kinds.rs`, kind 7's `value_bytes` literal;
   - `temper/crates/temper-client-tools/src/settings.rs`, `stock`: `skein_bools: [true, false, false, false]`.

2. **The bar.** `temper/crates/temper-client-view/src/frame.rs`:

   ```rust
   fn goals_shown(root: &Domain, project: u32) -> bool {
       let shown = match root.forge().ways(project) {
           Some(ways) => ways.goals,
           None => false,
       };
       shown || goals_current(root) || root.has_goals(project)
   }
   ```

3. **Words.**
   - `temper/docs/help/settings.md`: "The stock hides all four and uses 0." becomes "The stock offers drafts, hides the other
     three and uses 0."
   - `temper/crates/temper-settings-form/src/fields/jig.rs`, the `project-ways` form's line: "With none stored, all switches are
     false and way is 0." becomes "With none stored, drafts is true, the other switches false and way 0."
   - `temper/docs/help/reference/settings-fields.md`: that line, as `the_page_is_what_fields_renders` prints the page.

4. **Rows,** in existing tests, each new or changed row breaking when its value is flipped by hand:
   - `temper/crates/temper-client-domain/src/tests/forge_ways.rs`, `no_value_is_the_stock`: pin
     `Ways { drafts: true, goals: false, changes: false, away: false, way: 0 }`.
   - `temper/crates/temper-engine-domain-forge/src/tests/settings.rs`, `a_projects_kinds_are_judged`: pin `project_stock(7)`'s value
     to the same.
   - `temper/tests/client/tests/suite/settings_tools.rs`, `settings_show_prints_a_projects_ways_and_questions`: the stock's row
     prints `"drafts":true`.
   - `temper/crates/temper-client-view/src/tests/rail.rs`:
     - `drafts_offered_reads_drafts`: `(None, false)`, `(Some(Ways::STOCK), true)`, `(Some(Ways { drafts: false, ..Ways::STOCK }),
       false)`, `(Some(Ways { drafts: false, goals: true, ..Ways::STOCK }), false)`;
     - `goal_shows_while_drafts_are_offered`: `None`, `Some(Ways::STOCK)`, `Some(Ways { drafts: false, ..Ways::STOCK })`;
     - `the_rail_shows_what_kind_seven_shows`: its `drafts: true` row becomes `drafts: false`.
   - `temper/crates/temper-client-view/src/tests/home.rs`, `to_begin_keeps_chats`: keep what it checks, with the fixture's Goals hidden,
     and add: `styled(view.tree(), look::RAIL_NAV)`'s children hold "Goals", and in the browser view the tab bar is "Home", "Needs
     you", "Goals", "Chats". `rail.rs`'s `the_tab_bar_by_goals` holds the other side, no goal and no entry. Removing
     `|| root.has_goals(project)` must fail it.

5. **The fallout.** Run `temper-client-view` and `temper-client-domain`, and change what reads the old default, by the option
   above. Known: `rail.rs` (the rows at `the_rails_new_chat_is_a_link_and_a_menu` and others with `Ways::STOCK`), `chats.rs` (the
   plain lede), `new_chat.rs` (`the_bar_offers_a_draft_when_the_setting_offers_it`'s `plain`), `settings.rs`, `reference.rs`,
   `configure.rs`, `chat.rs`, and `temper/tools/gallery/src/tests.rs`. `settings-all-hidden` names its switches outright and
   holds.

6. **Goldens.** `temper-client-shell-terminal`: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-shell-terminal
   --lib -E 'test(golden)'`, then read the diff: `settings.txt`'s switch and any page whose words the default moved.

7. **The journeys.** A recording's kind 7 answer holds no value and the client takes the stock (`configuration.md`, 13.4), so
   a reprint carries the default, and `chat-draft`'s recorded switch, which turns `drafts` on, no longer has a control.
   - `chat-draft`'s `recording`: delete the three lines that make its first steps, the key `mark`, the `mark` before the toggle
     and `act toggle ... "Chats can write drafts"` (the first `act` line and the two `mark` lines before it). The walk is the
     recording's steps (`journeys.md`, 8.1), so `again` no longer walks them.
   - `chat-draft`'s `setup`: key marks shift by one: `mark 3`, `4`, `5`, `9` become `mark 2`, `3`, `4`, `8`.
   - `heavy cargo run -p temper-client-journeys -- again chat-draft`, retried as the request's `Recording flakes` line says.
   - `chat-draft`'s `README.md`: the walk loses "turn on ... and mark it again"; "How work runs with drafts On, marked" is `01.html`
     and "the same page before opening + New chat" `02.html`; the rest move down by two; "all 22 prints" becomes 20.
   - `heavy cargo run -p temper-client-journeys`: the reprint of the others. Read the diff by its words alone (`git diff --stat`, then
     `grep` for "Goal", "Draft", "write drafts"): what changes is the rail's "Goal", the Chats lede, the Drafts filter, "+ Draft"
     on a new chat and "Chats can write drafts" On. Anything else is a red.

8. Narrow tests after each change, the note's closing checks, then fmt, clippy and the focused suites the acceptance names, through
   `heavy`. Commit and push.

## Report

As the implementer's text has it, and: the fixture option taken in each file, the rules broken and the failures seen, which
journeys were recorded again and which reprinted, retries, and any story changed, compiled and not run.
