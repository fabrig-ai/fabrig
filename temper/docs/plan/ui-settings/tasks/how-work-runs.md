+++
id = "how-work-runs"
part = "how-work-runs"
kind = "feature"
needs = ["ways-session"]
files = [
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/forge/bindings.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
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

# How work runs: its sections and live controls

## Goal

The tab draws all of `temper/docs/design/client/README.md`, 5.13, *The tab*, and its controls send at once. What temper can't do yet is drawn disabled, showing what temper does today.

Acceptance, in the tab's order, under the "Configure temper" row `settings-frame` drew:

- **What the project shows,** `h2.section-title` 6 px above its first row: 5.13's table, a row each, at the right `.toggle-row`: the state's word then `.switch`, a `Role::Check` named by its label, checked by the value. While `ways(project)` is `None`, the section is its title and rows with the switches disabled, unchecked.
- **Ways of working,** only while Goals is shown (`ways.goals`, or the value asked of it): a row per way: its name; `small`, its line, and where a goal's template is recorded, " Used by N." or " Used by none yet."; at the right `.toggle-row`, a `Role::Choice` radio in one `Role::ChoiceGroup`, " The default", checked by `default_way`. `goals::ways` if it is on the trunk, given the template's `number` (a new field of its `Way`); if not, a private function here with the same table of names and lines, as a `Way { number, name, line }`, a one-line swap when `ui-goals` lands, and the report says so. A choice sends the whole record with `way` the template's number. A goal's line (`c::TaskLine`) records no template today: leave the "Used by" sentence out; if the trunk's does, count them over what the client has delivered.
- **Status,** disabled. `Role::Select`, `State::Disabled`, one option selected. Read the trunk at your base and pick the first true:

  | Label | Small | Shows |
  |---|---|---|
  | Status lines and reports | Written words, beside the counts temper keeps itself. | "Written by each goal’s agent" if `temper-client-domain/src/pages/report.rs` is on the trunk; else "None: counts only" |
  | Rewritten | | "When someone opens it" |
  | The brief on Home | What changed since you last looked. | "Written by the status agent" if `ui-home-status`' agent is on the trunk (`rg -l 'status agent' temper/crates/temper-service/src/stock`); else "Off" |

- **What needs you:** "Questions a goal can answer from its plan", a live `Role::Select`, options in the table's order: "Answered by the goal, kept in its decisions" (`foreman`); "Asked of me too" (`up`); "Not asked: each agent decides alone" (`none`); and "Funds", disabled, showing "Ask when a budget is used up", its `small` "Not built yet: this is what temper does today."
- **While you're away:** if `ui-away`'s section is on the trunk (`rg -l SetAway temper/crates/temper-client-domain/src`), it moves here, unchanged, after What needs you, shown only while `ways.away`. If not: nothing is drawn, and its request takes this page.
- **Telling you,** disabled: "Notifications", `small` "Only when something needs you; never for status.", showing "Off"; "Quiet hours", showing "None".
- **A disabled section** (Status, Telling you) ends once with `.facts`, "Not built yet: this is what temper does today."
- **Acts** by 5.13, *Its acts*, through `Meaning::Ways` (`notes/contracts.md`, *How work runs' rows*): each is `State::Disabled` while `ways_asked(its control)`, showing the value asked meanwhile, and its refusal, by `crate::cards::refusal`, is under its row, `Control::Ways`' under the row last pressed, the control back at its value.
- **Home's links:** where the trunk draws "Turn on drafts and goals", "See the settings" or "Limits" (`rg` the words), they go to `Settings { project }`.
- **The look** under `/* ==== ui: settings ==== */`: `.checks`, `.switch`, `.toggle-row`, `.setting select`, `.setting .text`, `.setting select:disabled`, `.facts`, as `temper/docs/design/client/browser.md`, 2.5, *Settings' look* and the request's values. A focus ring as the rest of the frame's.
- **The terminal:** the switches as checks, the selects as selects, a disabled one marked so; `settings.txt` regenerates and reads as a page; `each_fixture_page_prints_as_its_golden_file` passes.
- **The gallery:** `settings`, all shown; new `settings-all-hidden`, `settings-goals`, `settings-chat-open`, `settings-refused`, each with its `Holds`. `settings-refused` presses a switch and answers a refusal: it is a state by acts, as 7.2 says.
- `README.status.md` and `browser.status.md` lose what this closes.
- The tests below pass and bite.

## Inputs

- `temper/docs/plan/ui-settings/notes/common.md`; `temper/docs/plan/ui-settings/notes/contracts.md`, *The session's facts*, *How work runs' rows*.
- `temper/docs/design/client/README.md`, 5.13, *The tab*, *Its acts*, *The terminal*. `temper/docs/design/client/browser.md`, 2.5 *Settings' look*, 7.2. `README.md`, 3.6, a form's one request.
- The request's section 2 for the rules' values: `temper/docs/plan/ui-settings/inputs/request.md`.
- The mock: `temper/docs/design/client/greenfield/app.js`, `settingsPage`; `settings.html`, `settings.html?project=hello`; `mock.css`.

## Owns and may change

Owns `files`. May also change:

- **`frame.rs`' `decode`,** only if a `Role::Select`'s `Input` can't reach `forge::bindings::Meaning::Ways` through its own table.
- **`temper-client-shell-web/src/limits.rs`, `temper-client-tools/src/limits.rs`, `temper-client-shell-terminal/src/limits.rs`:** a count of nodes or bindings a page's rows trip: follow its formula.
- **`pages/home.rs`' links** as above, and `pages/goals.rs` for the `number` field if it is on the trunk.
- **Design status:** `temper/docs/design/client/README.status.md`, `browser.status.md`.

Pre-approved:

- If `skein-ui-web`'s `Role::Check` draws an input you can't class as `.switch` with a token (`SETTINGS_SWITCH`), name the token on the node; if its mapping doesn't let a check take a style at all, that's a skein gap: use `Role::Button` with `State::Pressed`, named by the label, and say so in the report. The mock's look is what's checked.
- If a `Role::Choice` can't stand outside a form as a live radio: a `Role::Button` with `State::Pressed` and a dot in the stylesheet, and say so.
- If the status words' test finds both plans on the trunk: the first that's true, as the table says.

## Steps

1. A `row` function; each section a function; `how_work_runs` the order. Words in `words.rs`.
2. `forge/bindings.rs`: `Meaning::Ways`, and its `Toggle`/`Input` arms.
3. Stylesheet; gallery; the terminal's golden.
4. Tests, `tests/settings.rs`:
   - `the_shows_section_reads_kind_seven`, table: each of the four, words and check.
   - `a_switch_sends_the_whole_record_with_its_part_changed`; `the_way_is_chosen_and_sends_the_whole_record`; `the_questions_select_sends_its_route`.
   - `kinds_of_seven_are_disabled_together_while_one_is_out`; `the_value_asked_shows_while_out`.
   - `a_refusal_is_said_under_its_row_and_the_control_goes_back`; `stale_goes_unseen`, by acts, its second refusal said.
   - `ways_of_working_only_while_goals_shown`, `the_stored_way_or_the_first_is_the_default`, `no_used_by_with_no_template_recorded`.
   - `disabled_sections_say_so_once_and_their_controls_are_disabled`; `funds_says_so_in_its_small`.
   - `until_read_the_switches_are_disabled_and_unchecked`.
5. `cargo fmt`; old forms; clippy and nextest of `temper-client-domain`, `temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`, `temper-client-gallery`, `temper-client-tools`; `heavy cargo check -p temper --tests`. Commit and push.

## Report

As the implementer's text says, with the status words you found true, and each pre-approved option taken.
