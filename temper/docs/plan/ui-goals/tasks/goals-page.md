+++
id = "goals-page"
part = "goals-page"
kind = "feature"
needs = ["goals-row"]
files = [
  "temper/crates/temper-client-view/src/pages/goals.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/goals.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/goals.txt",
  "temper/crates/temper-client-shell-terminal/src/tests/render.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/fixtures/goals.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/fixtures.rs",
  "temper/crates/temper-client-shell-web/src/limits.rs",
  "temper/crates/temper-client-tools/src/limits.rs",
]
effort = "high"
minutes = 90
+++

# The Goals page

## Goal

`goals::main` draws the mock's `goalsPage`: its intro, its filters and its rows, in both shells.

Acceptance:

- `.temper-goals`, in order:
  1. `.temper-goals-intro`: the kicker "PROJECT · goals"; `<h1 class=temper-goals-headline>`: "5 goals, 3 under way."; "1 goal, 1
     under way." in the singular; "No goals yet."; "More than N goals, M under way." while `unread`, singular as the count asks;
     then the lede: "A goal turns a draft into landed work, its own way: W.", W the `ways`' names in lower case, joined by ", ". M is
     rows in state needs, running, planning and paused. With no ways, "A goal turns a draft into landed work."
  2. `.temper-goals-section` > `.temper-goals-section-head` > `.temper-goals-filters`: "All", "Needs you", "Under way", "Ended", the
     chosen one pressed, each a move (`Act::GoalsFilter`). **No "Start one from a draft":** `ui-goal-page` adds it.
  3. `.temper-goals-rows` of `row`s, live then the ended page, those `goals::shown` holds; or `<p class=temper-goals-empty>None.</p>`.
     With no goals at all and the filter All, the same "None.".
  4. "More goals", `.temper-goals-more`, a quiet button after the rows while `more`, `Act::MoreGoals`.
- Before `me` or the watch delivers, the page draws its intro's kicker and lede alone, with no heading's number.
- The headline's numbers are of every row, whatever the filter.
- **The terminal:** a page of its own, its heading, filters as numbered buttons, a row a numbered link with its state and headline; a
  golden `goals.txt` beside `golden/task.txt`, by the test `each_fixture_page_prints_as_its_golden_file`.
- No spend is drawn.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *Goals*.
- The mock's `goalsPage` (`temper/docs/design/client/greenfield/app.js`).
- `temper/crates/temper-client-view/src/pages/chats.rs`, `list`; `temper/crates/temper-client-shell-terminal/src/tests/render.rs`.

## Owns and may change

Owns, as `files` lists them. `temper-client-shell-terminal/src/limits.rs` and `temper-client-tools/src/tests/limits.rs` move with the
presets if a preset moves.

Options, each pre-approved:

- **Presets:** a page's own count is `Goals` 4 handles and 1 watch beside the baseline's 5 and 4, under every preset's 17, 16 and 16.
  Don't grow a preset unless a test shows the need; then all three together, and say why.
- A row in `Role::List`, `.temper-goals-rows`, if the link's semantics then read better; your call.
- The golden is regenerated: `SKEIN_CODEGEN_WRITE=1 heavy cargo test -p temper-client-shell-terminal each_fixture_page`.

## Steps

1. **`main`,** from `goals-address`' stub. Words as `notes/common.md`.
2. **Tests,** `tests/goals.rs`:
   - `the_headline_counts_every_row_in_every_form`: none, one, several, more.
   - `the_lede_names_the_projects_ways`.
   - `the_filters_press_and_move`: each one's address.
   - `each_filter_shows_its_states`: Needs; Under way; Ended.
   - `a_filter_no_goal_matches_says_none`.
   - `more_goals_offers_and_reads`.
   - `the_page_draws_no_spend_and_no_draft_link`.
3. **The golden:** a hand-built `goals` tree, as `fixtures/task.rs`.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, with the page's handles and watches at their largest.
