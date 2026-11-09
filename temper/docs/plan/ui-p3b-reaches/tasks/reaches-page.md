+++
id = "reaches-page"
part = "page"
kind = "feature"
needs = ["reaches-model"]
files = ["temper/crates/temper-client-view/src/pages/how.rs", "temper/crates/temper-client-view/src/pages/settings.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/forge/bindings.rs", "temper/crates/temper-client-view/src/tests/how.rs", "temper/crates/temper-client-view/src/tests/settings.rs", "temper/crates/temper-client-view/src/tests/frame.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/crates/temper-client-shell-terminal/src/tests/golden/settings.txt", "temper/crates/temper-client-domain/src/forge/classes.rs"]
effort = "high"
minutes = 180
+++

## Goal

Draw How work runs' What reaches you: the project's classes in three
columns of words, from `pages::how::reaches`, after the path and before
Roles, with the question route and "Away until" as its rows; and remove
the sections it takes the place of, *What needs you* and *While you're
away, agents may decide* with its checks, so that a class changes by
proposal alone (`temper/docs/design/client/README.md`, 5.13, *What
reaches you* and *Its rows*; 3.5, *Away*'s *On How work runs*).

Acceptance:

- How work runs' main holds, in order, the path's section, What reaches
  you, Roles (while drawn), Budgets (while drawn), What the project shows
  and Ways of working (while Goals is shown): no "What needs you" and no
  "While you’re away, agents may decide" heading, no check of kind 8
  anywhere on the page.
- The section's tree is `notes/contracts.md`, section 2's: its heading,
  its head's right ("on PROJECT", or "…; each project has its own" by
  `Parts::several`), while `Parts::reaches` the columns' grid with one
  group per column that has a line, each its chip, its lines and its
  small line, in the design's words (contracts, section 2's table); then
  the rows "Questions a goal can answer from its plan" and, while
  `Domain::away_shown`, "Away until", each drawing and acting as today.
- "Limits" (the Away toast's act and Home's) focuses What reaches you's
  heading, as it focused the old section's (`Pages::away_focus`).
- The look: `temper.css` under `/* ==== ui: how ==== */` takes
  `whole-story.css`' `.grid3`, `.box` and `.who` rules as `browser.md`,
  2.5, *How work runs' look*, *What reaches you*, names them, on the five
  slots of contracts, section 2, by day and by night (night's tokens, as
  the block's other rules take them).
- The checks' code leaves with them: `temper-client-domain`'s
  `forge::classes` (`asking`, `turned`, `Asking`) and its test
  `asking_and_turning_rows` (`src/tests/away.rs`, and its call), the
  view's `WaysRow::Classes`, and the words no shell draws any more
  (`AwaySettingsTitle`, `ClassesAsking`, `ClassesAskingDetail`,
  `ClassesLocked`, `ClassesDecideLine`, `SettingsNeeds`, each where
  nothing else names it). Stop asking's "Apply", the proposal card's
  kind 8 lines and `forge::ways`' kind 8 read stay as they are.
- The view's tests:
  - `tests/how.rs`' headings test expects the new order;
  - a table test of the columns, in place of `tests/settings.rs`'
    `the_section_follows_what_needs_you` and
    `you_approves_section_asks_even_when_away_without_a_duration_select`,
    on `f::root_with_classes`, in both shells, a row each: Agents review's
    stock (three columns, "While you’re away, they are decided…", "Away
    until" drawn); You approve's (no "Agents decide, then tell you"
    group, the six movable kinds' lines, "They ask even while you’re
    away.", no "Away until"); contracts, section 3's table (the set's
    line first in "Asks you", the "all but changes to vsr’s protocol
    core" line); a table with an item with words (its line "… under
    “WORDS”"); every movable kind decided (no "Asks you" group); the
    full table of 20 items of 120 bytes the old test drew (every line
    drawn, nothing cut); kind 8 unread (`how(None)`: the heading, its
    head's right and the question route's row, no group); and `me` with
    two projects (the head's right "each project has its own");
  - the Away until tests (`defaults_with_no_row`,
    `the_select_writes_until`, `it_works_while_off`,
    `an_expired_until_still_reads_eight`,
    `away_controls_are_numbered_in_the_terminal`) keep what they check,
    finding the select under What reaches you;
    `limits_focuses_the_section` and `tests/frame.rs`' focus test find
    the "What reaches you" heading.
- The terminal's `settings.txt` golden regenerated
  (`SKEIN_CODEGEN_WRITE=1`, `render.rs`), its diff only this section's.
- The gallery's `settings-away` and `settings-away-you-approve` leave
  (their samples, mockup mapping, `HOLDS` rows,
  `settings_away_draws_the_default_choices`, `names.txt` lines, and
  `worlds.rs`' two branches keyed on them); kind 8 answers none for every
  remaining sample, `settings_bytes`' kind 8 branch left for
  `reaches-gallery`. Every other gallery expectation still holds.
- Delete the **Not yet** entries `5.13, *What reaches you* and *Its
  rows*, and 3.5, *Away*'s *On How work runs*` from
  `temper/docs/design/client/README.status.md`, and `2.5, *How work
  runs' look*, *What reaches you*, and 7.2` from
  `temper/docs/design/client/browser.status.md`.

## Inputs

- `temper/docs/plan/ui-p3b-reaches/notes/common.md`; `notes/contracts.md`,
  sections 1 and 2.
- `temper/docs/design/client/README.md`, 5.13, *The page*'s *What
  reaches you*, *Its parts*, *Its rows*, *Its acts* and *The terminal*;
  3.5, *Away*, *On How work runs*; `browser.md`, 2.5, *How work runs'
  look*.
- The look: `temper/docs/design/client/web/whole-story/how.html`, the
  section "What reaches you", and `whole-story.css` lines 83 to 125
  (`.shead`, `.who`, `.grid3`, `.box`).
- Patterns: `pages/how.rs`' `roles` (a section with its head's right
  words, read from `how::parts`); `pages/settings.rs`' `needs` and
  `away_settings`, whose rows move; `shared::object` (`shared/reach.rs`)
  for the subjects' plurals, `cards/proposal.rs`' `comma_join`;
  `pages/ways.rs`' `legend` for who acts' marks; `temper.css`'s
  `.temper-ways-legend > .temper-ways-kind-agent::before` and the
  `--star` variable.
- Fixtures: `tests/settings.rs`' `away_settings`, `how`, `how_view`,
  `section_nodes`, `setting_row`; `f::root_with_classes`.

## Owns and may change

- **Owns:** the files in this spec's header, and How work runs' drawing
  wherever it stands in `temper-client-view`.
- **Anchors** (`ui-n3a-only` writes the same files): words after
  `HowRoleInstructions` in the enum and in `ALL`, their English after
  `HowPath`'s, their ids anywhere in the id table; look constants after
  `HOW_ROW` and their names in the reserved slots 681 to 685 (contracts,
  section 2); CSS rules after `.temper-how-role-more`'s, inside the `how`
  block's own `@media (max-width: 760px)` for any phone rule.
- **Readers and neighbours:**
  - `temper-client-domain`'s `src/forge.rs` (the `mod classes` line) and
    `src/tests/away.rs` (the removed test and its call): deletions only;
  - `temper-client-view`'s `cards/proposal.rs`: `comma_join` made
    `pub(crate)` if the small line reuses it, nothing else;
  - the gallery (`temper/tools/gallery/src/samples.rs`, `worlds.rs`,
    `tests.rs`, `names.txt`): the removals above, and an expectation your
    drawing's correct change trips, repaired keeping what it checks;
  - `temper/tests/client/tests/suite/settings.rs` and any client world
    test that reads the moved rows: call sites only, if one breaks.
- **Options, pre-approved:**
  - in the terminal a named group prints its name: if the chip's text
    would print the column's words twice there, draw the chip in the
    browser alone (the view choosing by its shell, `browser.md`, 2.5)
    and say so;
  - if `Domain::away_shown` hides "Away until" where the old section drew
    it, keep `away_shown`'s rule: the row follows the switch;
  - if a removed word is still named by a test of another package, drop
    that assertion's word with the section it named, and say so.

## Steps

1. The section in `pages/how.rs`, `how_work_runs`' new order, the rows
   moved from `pages/settings.rs`, the old sections and the checks'
   code removed; words, look slots, CSS.
2. The view's tests as above. Break the rule once by hand (draw the
   "Agents decide" group with no line) and see the You approve row fail;
   restore.
3. The golden, the gallery's removals, the status entries.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-view -p temper-client-domain -p
   temper-client-shell-terminal -p temper-client-shell-web -p
   temper-client-gallery -p temper-client-world --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-client-view -p
   temper-client-domain -p temper-client-shell-terminal -p
   temper-client-gallery`, and `heavy cargo nextest run -p
   temper-client-world -E 'test(/settings::|away::/)'`, then `heavy cargo
   nextest run -p temper-client-world --profile fuzzy -E
   'test(/^fuzzy::how::/)'`. Commit and push.

## Report

As the implementer's text asks; list the words and look slots added and
removed, and every expectation outside your files you changed.
