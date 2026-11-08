+++
id = "how-page"
part = "page"
kind = "feature"
needs = ["how-parts", "how-look", "ways-shared"]
files = ["temper/crates/temper-client-view/src/pages/how.rs", "temper/crates/temper-client-view/src/tests/how.rs", "temper/crates/temper-client-view/src/pages/settings.rs", "temper/crates/temper-client-view/src/tests/settings.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-shell-terminal/src/tests/golden/settings.txt", "temper/docs/design/client/journeys/chat-draft/**"]
effort = "high"
minutes = 200
+++

# How work runs: the way, the ways' tabs, the path in lanes and a goal's budget

## Goal

Draw How work runs as `temper/docs/design/client/README.md`, 5.13, *The page*,
has it, at `Settings { project }`, in both shells: its heading naming the way the
project runs and its lede; "Ask temper to change this"; the section "The path of
a change" with the ways' tabs, the shown way's path in lanes, a stop's card and
the way's act; Budgets while a goal's budget is set; today's live rows, What
needs you and While you're away, then at its foot What the project shows and Ways
of working. The disabled Status, Funds and Telling you rows go. Its look is
`how-look`'s; its parts and lanes are `how-parts`'; its tabs, card and act are
`ways-shared`'s.

Acceptance:

1. `pages/how.rs` holds `notes/contracts.md`, C5's `head`, `ask`, `path` and
   `budgets`, drawing from `pages::how::parts(&pages::how::facts(..))` and
   `pages::how::lanes(..)` alone, no rule of their own.
2. The heading and lede are 5.13's: the current way's `name` and its `line` then
   "Ask temper to change any of it, in words; an owner accepts its proposal, and
   new work follows it."; "A way of its own" and "PROJECT runs settings of its
   own, drawn below." then that sentence; "How work runs in PROJECT." and the
   sentence alone until the read answers and after its watch ends. They stand in
   the frame's place for them (`settings::open`), the frame's kicker and tabs as
   they are; "Ask temper to change this", a primary link to `Ask { project }`,
   under the lede while `parts.ask`.
3. The path section is C3's tree: its title and "the way: NAME" or "the way: its
   own"; `ways::tabs`; the shown way's group with its line; the table "The path of
   a change", its header row (an empty header, then each column's step in 5.13's
   words, a bar's `words`), its rows "You", "Agents", the authority row of one
   cell spanning every column, "Code", each stop's button in its lane named as
   5.13 says (a person's stop's `words`, an agent's or code's card's `name`),
   binding `Act::WayStop`, current while open, its style C3's by actor; a bar's
   header and cells `HOW_BAR`; `ways::card` under the table while a stop is open;
   `ways::act` and `ways::refused`. Until the read answers, the title alone;
   ended, "The ways can't be read now.". Settings' column carries `HOW_WIDE` on
   this tab alone.
4. Budgets, while `parts.budget`: a section "Budgets" and one row (`HOW_ROW`):
   "Goals", "each given its own when it starts", and the amount or "up to N".
5. The body's order is 5.13's: the path, Budgets, What needs you, While you're
   away, What the project shows, Ways of working. No row is disabled; the Status
   and Telling you sections, Funds' row and the "Not built yet" line are gone
   with their words.
6. The text shell prints the page: the heading, the tabs a line each, the table a
   line a row, the card, the act and the rows.
7. The focused tests of `temper-client-view`, `temper-client-shell-terminal`,
   `temper-client-gallery` and `temper-client-journeys` pass, clippy clean;
   `heavy cargo check --tests` for `temper-client-world`,
   `temper-client-browser-world` and `temper-client-tui`. The entry "**Not yet:**
   5.13, *The page*" in `temper/docs/design/client/README.status.md` is gone.

## Inputs

- `temper/docs/plan/ui-p3a-how/notes/common.md`; `notes/contracts.md`, C1, C3, C4
  and C5.
- `temper/docs/design/client/README.md`: 5.13, *The page*, *Its parts*, *Its
  rows*, *A way's acts*; 3.5, *Settings* (the frame) and *Ask temper's page* (the
  tabs, card and act the page shares); 5.16 (a path's stops, read from the
  engine).
- `temper/docs/design/client/browser.md`, 2.5, *How work runs' look* (the classes
  this page sets).
- The mockups `temper/docs/design/client/web/whole-story/how.html` and
  `pair/how.html`: words, parts and order.
- The code: `temper-client-view`'s `pages/settings.rs` (`open`, `how_work_runs`,
  `section_head`, `shows`, `working`, `status`, `needs`, `away_settings`,
  `telling`, `facts`, `disabled_select`), `pages/ways.rs` (`ways-shared`'s),
  `pages/ask_page.rs` (how the ways are drawn on Ask), `look.rs`'s `HOW_*`
  (`how-look`'s), `words.rs` (`Word`, its four places, the ways' words);
  `temper-client-domain`'s `pages/how.rs` and `forge/stock_ways.rs`; the chat
  bar's budget words (`pages/chat.rs`, `spend`).
- The tests: `temper-client-view`'s `tests/settings.rs`
  (`every_tab_draws_its_head_and_tabs_in_order`,
  `disabled_sections_say_so_once_and_their_controls_are_disabled`,
  `funds_says_so_in_its_small`, `the_section_follows_what_needs_you`),
  `tests/ask.rs` (its ways fixtures), `tests/fixtures.rs`;
  `temper-client-shell-terminal`'s `tests/render.rs` (the golden `settings`,
  rewritten with `SKEIN_CODEGEN_WRITE=1`); `temper-client-gallery`'s `tests.rs`
  (the `settings*` states' holds).

## Owns and may change

Owns: `temper-client-view`'s new `src/pages/how.rs` and `src/tests/how.rs`;
`src/pages/settings.rs`'s `how_work_runs` and the How work runs arm of `open`'s
heading and lede, and the disabled rows' functions it no longer calls;
`src/tests/settings.rs`'s How work runs rows and its tests of the removed rows;
`words.rs`'s keys 4900 to 4949 and the removed rows' words; the golden
`settings.txt`; the `chat-draft` journey's recording and prints.

Reader and neighbour files, and what may change there:

- `src/pages.rs`: `pub(crate) mod how;`. `src/tests.rs`: `mod how;`.
- `settings::open`: the How work runs arm of its heading and lede and the call of
  `how::ask` under the lede, and `HOW_WIDE` on the column for that tab; nothing of
  its kicker, its tabs or its other arms (`ui-p1-rail`'s `deployment` changes
  them beside you).
- `temper-client-gallery`'s `tests.rs`: a `settings*` hold that reads a removed
  row or the old heading's place, that line alone.
- Journeys whose prints show How work runs (`chat-draft` today; any other the
  reprint names stale): reprinted, or recorded again as `notes/common.md` says.
- `temper/docs/design/client/README.status.md`: the one entry above, deleted.

Not yours: the gallery's states (`how-gallery`), the client world's stories
(`how-world`), `temper.css` and `look.rs` (`how-look`'s; a missing rule or name is
reported under `## Left`), the rail (`ui-p1-rail`'s).

Pre-approved options:

- **If `b.style` can't reach the column** once `open` returns: set `HOW_WIDE` in
  `open`'s How work runs arm instead, and say so.
- **If skein's table holds no cell spanning columns** as the authority row needs:
  give that row one cell with the line's words and a cell per column after it,
  empty, and say so.
- **If the text shell prints the table unreadably** (a line far past 110
  columns): keep the table; say what it prints, with its widest line, in the
  report.
- **If a `settings*` gallery state** now draws an empty path section for want of
  a `ways` delivery: leave it; `how-gallery` gives the `how` states theirs.
- **If the toast after "Use this"** isn't drawn on Settings: draw it as Ask
  temper's page does, in `pages/how.rs`, and say so.

## Steps

1. `pages/how.rs`: C5's four functions; the step words, lanes' names and line in
   `words.rs` from 4900, beside the ways' words.
2. `settings.rs`: `open`'s How work runs heading and lede from `how::head`, then
   `how::ask`; `how_work_runs`' body in 5.13's order; `status`, `telling`,
   `facts`, Funds' row and `disabled_select`, with their words, removed where
   nothing else calls them.
3. Tests, `src/tests/how.rs`: one table test, a row a state: reading; ended;
   Agents review current over CI and tiers (heading, lede, "the way: Agents
   review", the tabs with "now", the table's header row and each stop's lane and
   name from the read, the authority row, the act); Draft first with its goal's
   bar and Budgets from an amount; a cap's "up to N"; a budget given at start and
   Goals hidden, no Budgets; "Yours"; a stop open (its cell current, its card,
   "Open the reviewer ›"); a member (no act); Ask temper not set up (no offer);
   the text shell's lines of the table. `tests/settings.rs`: the How work runs
   row of `every_tab_draws_its_head_and_tabs_in_order`; the removed rows' tests
   deleted; `the_section_follows_what_needs_you`'s order. Break three rules by
   hand (an agent's button in Code; Budgets for `Given`; the heading's way taken
   from the shown tab, not the current one) and see their rows fail; restore.
4. Regenerate: the golden (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
   temper-client-shell-terminal --lib -E 'test(=tests::render::each_fixture_page_prints_as_its_golden_file)'`,
   or the test's own path), then the journeys (`heavy cargo run -p
   temper-client-journeys`, `again NAME` for each it names stale).
5. Check as `notes/common.md` says. Delete the status entry. Commit and push: the
   message says How work runs draws the way, the ways and the path in lanes, and
   drops the rows that did nothing.

## Report

Follow the implementer's text. Also give the rules you broke and the failures you
saw; the words and keys added and removed; the journeys reprinted or recorded
again; what the text shell prints of the table; each test's CPU time; and, under
`## Left`, anything of the look the page's tree needed that `how-look` lacks.
