+++
id = "settings-away"
part = "settings-away"
kind = "feature"
needs = ["overturn"]
files = [
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/away.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 75
+++

# Settings' section

## Goal

The switch's kinds and its hour, on *How work runs*: `temper/docs/design/client/README.md`, 3.5 (*Away*, *Settings' section*). The target: `temper/docs/design/client/greenfield/app.js`, lines 565 to 569; `mock.css`, 266 to 272.

Acceptance:

- **The section,** in `how_work_runs`, after `needs`, before `telling`, while `away_shown`: a level-2 "While you’re away, agents may decide", words and rows as 3.5 says, each `.setting` by the rows `pages/settings.rs` already has.
- **Each writes through `SetAway`:** a kind's check, `Act::Away(Kinds { project, kinds })`; the select, `Act::Away(Until { project, eight })`. Read from the row; with none, `DEFAULT_KINDS` and 08:00. Past its row's `until` a select still reads 08:00.
- **Locked and unticked:** "Budgets, sign-ins, machines, what is hard to reverse: always wait", a check that's disabled, whatever the row says.
- **While off they work:** a test sends each and sees the row's choice come back drawn, with `on` false.
- **"Limits" lands here:** `Builder::focus` on the section's heading (it asks for focus "after the shell applies the turn's patches", and a browser scrolls what it focuses into view).
- **`settings-away`** in the gallery, as `browser.md`, 7.2.
- **The terminal:** its checks and select numbered, as the rest of Settings'.
- `README.status.md`'s and `browser.status.md`'s lines lose what this closes.

Departure from `away-client`'s spec: `Builder::focus` takes a count, and the view holds `&Domain`. So where `away-client` left `Pages::away_focus(&mut self) -> bool`, make it `pub fn away_focus(&self) -> u64`, a count `Limits` raises, and draw `b.focus(count)` while it's above 0.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5 (*Away*), 5.13; `browser.md`, 7.2.
- The models: `pages/settings.rs`' `how_work_runs`, `shows` (its checks), `working` and `needs` (a select), `row`, `row_line`; `temper-client-shell-web/temper.css`'s settings rules.

## Owns and may change

Owns, as `files` lists them. May also change: `tests/fixtures.rs`; the terminal's goldens, regenerated; `temper/docs/help/settings.md`, one line for the section if its test asks, within its bound.

Options, each pre-approved:

- **If `ui-settings`' `how_work_runs` isn't on this trunk** (`client-base` took it, so it should be): after the existing list, by the same code, and say so.
- **`.checks`' rules** exist under the settings marker if `ui-settings` drew any; otherwise add them under `/* ==== ui: away ==== */`.
- **The section's id:** `Address` holds no anchor, and none is added.

## Steps

1. `Word`s, tokens.
2. The section. `tests/settings.rs`: `the_section_follows_what_needs_you`, `no_section_while_no_switch`, `defaults_with_no_row`, `a_kind_writes_through_set_away`, `the_select_writes_until`, `it_works_while_off`, `locked_stays_unticked_and_disabled`, `limits_focuses_the_section`.
3. CSS, the gallery, the render beside the mock at 1440 and 420.
4. The check of the implementer's text, `-p temper-client-domain -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal`. Push.

## Report

As `notes/common.md`, Your report: the render's check, on which builder.
