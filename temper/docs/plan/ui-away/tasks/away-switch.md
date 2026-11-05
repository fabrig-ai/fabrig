+++
id = "away-switch"
part = "away-switch"
kind = "feature"
needs = ["away-client"]
files = [
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/src/tests/toast.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/**",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 90
+++

# The control and its toasts

## Goal

The switch, in the rail, on a phone and in the terminal: `temper/docs/design/client/README.md`, 3.5 (*Away*: *Shown*, *The control*,
*Its act*, *Past `until`*, *Its toasts*). The look is the mock's: `temper/docs/design/client/greenfield/mock.css`, `.away` (lines 59-68)
and `.away-chip` (437-438); `app.js`' lines 261 and 268.

Acceptance:

- **In the foot,** just before `settings_link`: a `Role::Button`, `temper-away`, `State::Pressed` while on, with the track's slot,
  `temper-away-track`; words as 3.5 says; disabled while its request is out. Drawn while `away_shown`.
- **`frame::rail` takes `&mut Times`,** `root.rs` passes it: `until`'s words by `TimeForm::Clock`, and `times.next` asks to wake at
  `until`, so the rail draws off with no event.
- **The chip:** a second button, `temper-away-chip`, the browser's alone (`Shell`), a child of the rail beside `temper-projects`. In
  `temper.css`: hidden on a wide page, and excepted from `#app > .temper-rail > :not(.temper-projects) { display: none; }` on a phone.
- **The toast,** `Toasted::AwayOn`, `AwayOff`, `AwayRefused`, with "Limits" `Act::Away(Limits)` on `AwayOn`: a link's look, by the
  button the toast already has.
- **`temper.css`:** under `/* ==== ui: away ==== */`, placed after `/* ==== ui: settings ==== */`'s block. Tokens per
  `notes/contracts.md`, *The view*: `AWAY`, `AWAY_CHIP`, `AWAY_TRACK`.
- **The terminal:** the same button, in its rail, numbered as any. A project with no goal shows none, so a golden moves only where its
  fixture holds a goal.
- **`rail-away-on`** in the gallery: the switch on, "Away until 08:00", toast open.
- `browser.status.md` loses `rail-away-on`.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5 (*Away*), 5.3; `browser.md`, 7.2.

## Owns and may change

Owns, as `files` lists them. May also change: `tests/fixtures.rs`; `temper/docs/design/client/browser.md`'s row for its state, if the
sample's words differ; `temper-client-view`'s other test files for a `rail` call.

Recipe for the gallery state: the name in `names.txt`, sorted; a `Sample` in `samples.rs`, as `home-morning`'s; the name matches; the
document in its `Watch` match (now `Watch::Away`, and its arm in `worlds.rs`); a `Holds` row in `tests.rs`.

Options, each pre-approved:

- **`aria-pressed`:** if `State::Pressed` doesn't render so, style by whichever attribute it does, and say so.
- **The child-order pins** in `tests/frame.rs` and `tests/rail.rs`: extend them for the chip. Where a pin says the rail holds exactly its
  children, the chip joins only when shown.
- **Words new to `Word`:** the four lines of `TimeClock`'s pattern (enum, list, number, phrase); `limits.rs`' pins move with them.

## Steps

1. `Word`s, tokens, `tests/look.rs`' rows and counts.
2. `rail`, the chip, `root.rs`. Tests in `tests/rail.rs`: `no_switch_on_a_fresh_project`, `the_switch_shows_with_a_goal`,
   `the_switch_shows_with_ways_away`, `on_it_says_until_and_is_pressed`, `on_with_no_until_it_says_away`,
   `past_until_it_draws_off_and_asks_a_wake`, `disabled_while_out`, `the_chip_is_the_browsers_alone`, `in_the_terminal_it_is_numbered`.
3. The toast: `tests/toast.rs`: `on_says_until_and_offers_limits`, `off_says_questions_wait`, `refused_says_try_again`.
4. CSS, then the gallery. Check `rail-away-on` against `greenfield/home.html` at 1440 and, for the chip, `phone.html`, at 420 (`notes/common.md`, *Render proofs*).
5. `SKEIN_CODEGEN_WRITE=1 TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-shell-terminal` if a golden moved, then unset.
6. The check of the implementer's text, then `-p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal -p temper-client-shell-web`. Push.

## Report

As `notes/common.md`, Your report: the render's check, on which builder.
