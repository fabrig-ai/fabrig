+++
id = "away-client"
part = "client"
kind = "feature"
needs = ["decided-client", "classes-kind", "classes-value", "decide-forge"]
files = [
  "temper/crates/temper-client-domain/src/forge/ways.rs",
  "temper/crates/temper-client-domain/src/forge/classes.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/pages/away.rs",
  "temper/crates/temper-client-domain/src/tests/forge_ways.rs",
  "temper/crates/temper-client-domain/src/tests/away.rs",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/forge/bindings.rs",
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/shared/reach.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-client-view/src/tests/start_goal.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/src/tests/fixtures.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/**",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/worlds/ways.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 150
+++

# Away by the project's classes, and Settings' section as their view

## Goal

The client reads kind 8, a project's classes, as it reads kind 7; the Away switch shows where the classes hold an item Away may decide, or while the party's row is on; Settings' *While you're away, agents may decide* becomes a view of the table whose marks an owner changes by one `ChangeSettings` of kind 8; a proposal's kind 8 item reads in words; and a goal's asking choice is worded by the classes (`temper/docs/design/client/README.md`, 3.5, *Away*; 5.13; 5.14). Runs after `decided-client`, which writes the same view files and makes `shared::object`.

Acceptance:

1. **Kind 8's read,** in `temper/crates/temper-client-domain/src/forge/ways.rs`: a third control, `Control::Classes`, its slot read for the open project as kind 7's is (as the project opens, again when the `settings` document's line for kind 8 of that project changes and when the link comes back, the same backoff), `SettingRead { kind: 8, scope: project, number: 1 }`. The value decodes by the kit's `classes_from` and is kept only where `Classes::check` passes; a read with no value is `Classes::stock(Way::AgentsReview)` (`temper/docs/design/domain/forge.md`, 5.10); a record that fails to decode or check, or a lasting refusal, leaves it `None`. `Asked::Classes(Classes)` encodes by `classes_record`; `Act::Set { project, asked: Asked::Classes(..) }` sends one item of kind 8 through the existing set path (one out at a time, a `stale` sent once more, the value asked shown meanwhile). `Side::classes(project) -> Option<&Classes>` answers the open project's table.
2. **The proposal's items:** the proposal read's whitelist takes kind 8 beside 1, 2, 6 and 7, and `proposal_values` decodes kind 8 into `Proposed::Classes { project, classes }` (a removal or reset as the stock).
3. **Away's showing,** `Domain::away_shown(project, wall)` (`temper/crates/temper-client-domain/src/domain.rs`): `Side::classes(project)` holds an item Away may decide (`Classes::away_may_decide`), or `away_on(project, wall)`. Neither a goal nor kind 7's `away` member shows it any more (that member goes with `ways-away`). `pages/away.rs`'s `act` sends `SetAway { project, on, until }` as `decide-forge` left it; its doc comments say the classes decide.
4. **The section's items,** a pure domain module `temper/crates/temper-client-domain/src/forge/classes.rs` (sketch): `asking(classes)` lists the "asks you" items in 5.13's order (the movable kinds in `Kind::ALL`'s order, each by its own item or, with none, as `AskAway`; then the table's other items at `Ask` or `AskAway`, in their order), and `turned(classes, asking)` the table with that item turned between `Ask` and `AskAway`, a kind with no item of its own given one at `Ask`, added at the end.
5. **Settings' section** (`temper/crates/temper-client-view/src/pages/settings.rs`, `away_settings`), as 5.13's *While you're away, agents may decide* draws it: shown while `asking` holds any item (the stock's until kind 8 is read, disabled); the small "Agents decide, then tell you: …" line while an item is at `Decide`; the `.setting` "What asks you" with its small words and a check an item, named by `shared::object` (from `decided-client`) with its first letter capitalised, `State::Pressed` for `Ask`, bound to `Act::Set { asked: Asked::Classes(turned(..)) }` through `ForgeBindings::insert_way` with a new `WaysRow::Classes`, so a refusal shows under it as kind 7's do; every check disabled while a kind 8 change is out; the locked unticked last line, "Money past a budget, what can’t be undone, and changes to these rules: always wait"; the "Away until" select as it is, while `away_shown` alone. The "Limits" focus stays.
6. **The card in a chat** (`temper/crates/temper-client-view/src/cards/proposal.rs`): `Proposed::Classes` draws "What reaches you:" and a line a class the table gives any subject, then a line a set, as 5.13 words them; `setting_base` names kind 8 `project-classes`, as `temper ctl` does once `classes-tools` lands.
7. **The goal's asking words** (`temper/crates/temper-client-view/src/words.rs`): `StartGoalFollow` "As the project says", `StartGoalFollowDetail` "Decides what the project’s settings let agents decide, Away included, and tells you; asks you the rest.", `StartGoalAlwaysDetail` "Asks you every question it can’t answer from the plan, even while you’re away.", `GoalFollowsAway` " · asks as the project says" (5.14; 5.8, *How it runs*). The variants may be renamed to what they say; the call sites in `pages/start_goal.rs` and `pages/goal.rs` change only for a rename.
8. **Rows, no new test function:**
   - the domain: `src/tests/forge_ways.rs`'s kind 7 tables gain kind 8's rows (read as the project opens and dropped as it closes; no value is the stock; a refused read and a value `check` refuses leave `None`; a changed line reads again; a set sends one change of one item against the newest; a stale is sent once more; the proposal's kind 8 item decoded, in the tests about lines 648 and 680); `src/tests/away.rs`'s `shown_by_goal_by_on_or_by_ways` becomes the classes' table (stock: shown; You approve's: not; You approve's with the row on: shown; unread: not; a goal alone: not), renamed to what it checks; `asking` and `turned` as rows of that table or of the forge-ways table, a kind with no item and an item with words among them;
   - the view: `src/tests/rail.rs`'s `no_switch_on_a_fresh_project`, `the_switch_shows_with_a_goal` and `the_switch_shows_with_ways_away` say the new rule (a fresh project's switch shows once kind 8 is read as no value; a goal alone shows none; You approve's table shows none), renamed to what they check; `src/tests/settings.rs`'s Away tests (about lines 1157 to 1281) check the section's lines, names, ticks, the locked line, a check's `ChangeSettings` of the whole turned table, the disabled state while out, a refusal under the row, the section on You approve's table with no select; `src/tests/cards.rs`'s proposal previews (about line 552) a kind 8 row; `src/tests/start_goal.rs` and `src/tests/goal.rs` the new words;
   - the gallery (rows only, its tests over budget): `settings-away` reworked and `settings-away-you-approve` added (`temper/docs/design/client/browser.md`, 7.2), their fixtures answering the kind 8 read in `worlds.rs` (`settings_bytes`, about line 4836, today shows the section through kind 7's `away`); `settings_away_draws_the_default_choices` checks the stock's three ticks and the locked line; `start-goal-draft`'s row the new words. `rail-away-on`, `rail-away-off` and the `home-away*` samples show the switch through kind 7's `away` today (`samples.rs`, `home_domain`, about line 2693): they answer kind 8's read with the stock (no value), so the classes show it, and their rows hold as they are.
9. **Generated and recorded files:** the client now sends kind 8's read, so the journeys' recordings are stale: re-record them with `temper-client-journeys again` (the request's retries holding), and their prints with them; the terminal shell's goldens (`temper/crates/temper-client-shell-terminal/src/tests/golden/`) that now show the section or the rail's switch are regenerated by `each_fixture_page_prints_as_its_golden_file` with `SKEIN_CODEGEN_WRITE=1`. Each named in the report, its diff explained by the new read or the new rule.
10. **Runs,** counts in the report: `heavy cargo nextest run` for `-p temper-client-domain`, `-p temper-client-view`, `-p temper-client-gallery`, `-p temper-client-world`, `-p temper-client-tui`, `-p temper-client-shell-terminal` and `-p temper-client-journeys` (`away_shown` changes meaning: every reader runs); `heavy cargo check --tests -p temper-client-browser-world -p temper-one-process-world -p temper`; `heavy cargo check -p temper-client-shell-web --target wasm32-unknown-unknown`; clippy on the packages changed, and on `temper-client-shell-web` for wasm32.
11. **Status:** delete the **Not yet** entries naming `ui-reaches/away-client` in `temper/docs/design/client/README.status.md` and `browser.status.md`, and, in the former's **Design departs** list of the request `ui-away` (about line 889), the sub-entry “5.8: "follows the away switch" stands …”, whose words go.

## Inputs

- `temper/docs/plan/ui-reaches/notes/common.md`, all of it.
- `temper/docs/plan/ui-reaches/notes/contracts.md`, section 1 (`Classes`, `Kind::ALL`, `Class`, `Subject`, `away_may_decide`, `class_of`, `stock`) and section 3 (`classes_from`, `classes_record`; kind 8 read through the forge's settings read).
- `temper/docs/design/client/README.md`, 3.5, *Away*; 5.13: *While you're away, agents may decide*, *What the client holds*, *Its acts*, *What the settings change*, *The card in a chat*; 5.14, *A form*'s "When it needs you"; 5.8, *How it runs*.
- `temper/docs/design/client/browser.md`, 7.2 (`settings-away`, `settings-away-you-approve`, `start-goal-draft`).
- `temper/docs/design/domain/forge.md`, 5.10 (the table, the stock tables, `Classes::away_may_decide`); `temper/docs/design/protocol/configuration.md`, 13.2 (kind 8, its scope and readers).
- The code: `forge/ways.rs` (the slots, `progress`, `read`, `told`, `value`, `stock`, `kind`, the set path about line 540, the proposal reads about line 650 and `proposal_values`), `domain.rs` (`away_shown`), the view's `pages/settings.rs` (`shows`, `away_settings`) and `forge/bindings.rs` (`WaysRow`, `insert_way`, `remember`), `cards/proposal.rs` (the kind 6 and 7 previews, `setting_base`).

## Owns and may change

Owns kind 8's slot, the `classes` module, `away_shown`'s rule, Settings' section, the kind 8 preview, the words of acceptance 7, the rows and states, the regenerated journeys and goldens, and the status entries. Not kind 7's `away` member or its "Away switch" row (`ways-away`), not `decided-client`'s store.

Sketch, binding at its outer layer:

```rust
// temper/crates/temper-client-domain/src/forge/ways.rs
pub enum Control { Ways, Questions, Classes }
pub enum Asked { Ways(Ways), Questions(Route), Classes(temper_protocol_reaches::Classes) }
pub enum Proposed { Ways { .. }, Questions { .. }, Classes { project: u32, classes: temper_protocol_reaches::Classes } }
impl Side {
    /// The open project's classes (README 5.13): the stock where the setting holds no value; None until read, or refused.
    #[must_use] pub fn classes(&self, project: u32) -> Option<&temper_protocol_reaches::Classes>;
}

// temper/crates/temper-client-domain/src/forge/classes.rs, `pub mod classes` in forge.rs
//! Settings' view of a project's classes (temper/docs/design/client/README.md, 5.13, While you're away).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Asking { pub subject: temper_protocol_reaches::Subject, pub item: u8, pub away: bool } // item: its place from 1, 0 for a kind with none; away: class Ask
#[must_use] pub fn asking(classes: &temper_protocol_reaches::Classes) -> Box<[Asking]>;
#[must_use] pub fn turned(classes: &temper_protocol_reaches::Classes, asking: Asking) -> temper_protocol_reaches::Classes;
```

May change, each only as said:

- `temper/crates/temper-client-domain/src/forge.rs`: the `pub mod classes` line; `temper/crates/temper-client-view/src/shared/reach.rs`: a capitalising helper beside `object`, if the section needs one.
- A test or fixture elsewhere that matches `Control`, `Asked` or `Proposed` exhaustively (found by the compiler): the arm the new variant needs, named in the report.
- The gallery's and the client world's fixtures that answer the forge's setting reads: an answer for kind 8 where a state or story reads it, the stock's (no value) unless the state is about the table. The view's `src/tests/fixtures.rs` answers every setting read but kind 7's with no value, so kind 8 reads as the stock there: a parameter for a test's own table where it needs one (You approve's, a set's), nothing else.
- `temper/crates/temper-client-view/src/words.rs`: the words of acceptance 5 to 7 and their ids, the next free ones.

Options, each decided:

- **A story or test counts the forge's reads** and now sees kind 8's beside kind 7's: add kind 8 to what it expects, keeping what it checks, and name it.
- **The rail's switch now shows** in a test or state that drew none (a fresh project whose kind 8 reads as the stock): that is the rule; update it to the switch drawn, or give its fixture You approve's table where the test is about something else, and name each.
- **A recording fails to re-record** on "a key mark caught a moving page": retry up to three times, each on a fresh fake box, then widen the mark's wait in the recording driver and say so.
- **`classes-tools` has not named kind 8 `project-classes`** when this runs: keep that name in `setting_base`; it is the name the plan gives (`temper/docs/plan/ui-reaches/plan.md`).

## Steps

1. Read the Inputs. Kind 8's slot and the proposal's items (acceptance 1, 2); their rows, and bite one: keep a value `check` refuses, see its row fail, restore.
2. `away_shown`, `classes` and their rows (acceptance 3, 4); bite: leave `has_goals` in `away_shown`, see the goal-alone row fail, restore.
3. The view: Settings' section, the preview, the words (acceptance 5 to 7) and their rows (acceptance 8); bite a check's row by sending the untouched table, see it fail, restore.
4. The gallery's states; the goldens and journeys (acceptance 9); the runs of acceptance 10; the status entries; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`; push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report: kind 8's slot and its stock, `away_shown`'s rule, `asking` and `turned`, the section and preview as drawn, the words, the rows and their bites, the options taken, the journeys and goldens regenerated (each with why it moved), and the runs' counts.
