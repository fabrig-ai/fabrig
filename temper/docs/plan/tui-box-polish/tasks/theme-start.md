+++
id = "theme-start"
part = "theme"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-tui/src/theme.rs", "temper/crates/temper-client-tui/src/tests/theme.rs", "temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/run.rs", "temper/crates/temper-client-shell-tui/src/journey/**", "temper/crates/temper-client-shell-tui/src/tests.rs", "temper/crates/temper-client-shell-tui/tests/suite/run.rs", "temper/tools/journeys/src/tui.rs", "temper/tools/journeys/src/tests.rs", "temper/docs/design/client/README.status.md", "temper/docs/design/client/tui/journeys.status.md"]
effort = "high"
minutes = 90
+++

## Goal

The TUI's theme becomes a choice kept across runs: `ThemeChoice`
(Auto, Light, Dark), its record `theme` in the TUI's small store, the
host taking it out of `Start.kept` and starting in it, and a host-side
`keep_theme` that draws in a choice and saves its record, with the
store's answers kept apart from the root's. `Start.scheme` becomes the
terminal's word, or none, so that Auto can say whether the terminal
said. Until `theme-list` gives the list, the root's `/theme` flip is
kept through `keep_theme`, so the next run starts in the flipped theme.

Acceptance:

- `temper_client_tui::theme::{ThemeChoice, THEME_KEY}` as
  `temper/docs/plan/tui-box-polish/notes/contracts.md` has them; the
  crate root's `lib.rs` is not edited.
- `host::Start.scheme: Option<temper_client_tui::Scheme>`; the run sets
  it to `COLORFGBG`'s scheme, else the answered background's
  (`Scheme::of_background`), else `None`. The recording's `screen`
  writes and reads `dark`, `light` or `none`, its facts' scheme an
  `Option<Scheme>`, and the committed journeys still replay unchanged.
- At `Start` the host takes every record keyed `THEME_KEY` out of
  `kept` (the root never sees it) and starts in `ThemeChoice::read` of
  the last one's bytes, Auto with none: the root's `LookKept` theme is
  `choice.scheme(start.scheme)`.
- `keep_theme(choice, duties)` per the contracts: the choice drawn at
  once, `LookKept` with the root's look but its theme, and one
  `jig_client::Request::Save` of `THEME_KEY` = `choice.print()` pushed to
  `duties.asked` after the report's routed requests.
- Writes are counted in order: each `Save` or `Erase` leaving in
  `duties.asked` (the root's, through `route`, and the host's) is noted
  as the root's or the host's; each heard `Stored` or `NotStored`
  answers the oldest noted, and reaches the root only when it answers a
  root's write. After closing, `Heard` still does this.
- Interim, until `theme-list`: after each report, a root look whose
  theme differs from the scheme the host's choice draws keeps the
  matching `Light` or `Dark` through `keep_theme` (today's `/theme`
  flip, now kept for the next run).
- The two status entries naming this task's parts are gone: in
  README.status.md the one for 6.13's `Start.scheme` and
  `Duties.asked` and 6.15, *The scheme* and *The theme chosen*, and in
  `tui/journeys.status.md` the one for 3.2's `none`.

## Inputs

- `temper/docs/design/client/README.md`, 6.12 (*What it holds*, the
  small store), 6.13 (`Start`, *The reports*, *The duties*, *The look*,
  *The seam's shape*) and 6.15 (*The scheme*, *The theme chosen*).
- `temper/docs/design/client/tui/journeys.md`, 3.2 (`screen`).
- `temper/docs/plan/tui-box-polish/notes/contracts.md`, *temper-client-tui*
  (the theme chosen) and *temper-client-shell-tui* (`theme-start`).
- `temper/docs/plan/tui-box-polish/notes/common.md`.
- For the store's order: `jig/crates/jig-client-shell-native/src/shell.rs`
  (`Shell::down`, `Save` and `Erase` each a batch) and `src/files.rs`
  (`written`, one answer per batch, in order). If the code there does
  not answer each `Save` and `Erase` once and in order, stop and ask.

## Owns and may change

- Owns: `temper-client-tui`'s `src/theme.rs` (the type and constant
  after `impl Scheme`, before `rgb`; nothing near `styles()`, which
  `box-ink` edits) and `src/tests/theme.rs` (a new test function after
  the last one); `temper-client-shell-tui`'s `src/host.rs` (`Start`;
  three fields declared after `background: Option<Rgb>,` and
  initialised after `background: None,`; `start`; `route`; `heard`;
  `keep_theme`; the block in `report` that sets `self.theme`), `src/run.rs`
  (`started`), `src/journey/**` (the `screen` record, the facts, the
  recorder, the replay and their tests), `src/tests.rs` and
  `tests/suite/run.rs`; `temper/tools/journeys/src/tui.rs` and
  `src/tests.rs` (readers of the screen's scheme: the print draws in
  `ThemeChoice::Auto.scheme(scheme)`); the two status files.
- Reader edits allowed: every literal or match of `Start.scheme` or the
  journey facts' scheme, in any crate of temper, takes `Some(...)` or
  the option, call sites only.
- Not yours: `src/state.rs`, `src/words.rs`, `src/event.rs`,
  `src/view/**` of `temper-client-tui` (the list and its notice are
  `theme-list`'s), the root's `/theme` (unchanged).

## Steps

1. `ThemeChoice`, `THEME_KEY`, `print`, `read`, `scheme`, `ALL`. Test
   `a_theme_choice_prints_reads_and_draws_its_scheme`: each round-trips,
   unknown and empty bytes read Auto, Auto draws the terminal's or
   Dark, Light and Dark their own whatever the terminal says.
2. `Start.scheme` to an option; `run.rs`'s `started` as above; the
   journey's `screen` record and facts; a codec row for `none` and the
   replay of a `none` start (in `journey/tests.rs`, beside the rows that
   build `Line::Screen`).
3. The host: the fields (`terminal: Option<tui::Scheme>`, `choice:
   ThemeChoice`, and the writes noted, a `Queue<bool>` or a small enum,
   your call); `start` takes the record out and gives `LookKept` with
   the chosen scheme; `route` notes each root `Save` and `Erase` it
   pushes; `keep_theme` pushes and notes the host's; `heard` takes each
   `Stored` or `NotStored` against the oldest note and steps the root
   only for a root's write; the interim keep in `report`.
4. Host tests, placed right after
   `launchpad_selection_clears_after_command_composition` in
   `src/tests.rs`: `a_kept_theme_starts_the_next_host_in_it` (a `theme`
   record of `light` with `scheme: None` draws light and the root's
   `Start` has no `theme` record; `junk` reads Auto, dark with no word
   and light where the terminal says light); and
   `a_theme_write_answer_never_reaches_the_root` (a `/theme` flip asks a
   `Save` of `theme` = `light`; a root write asked before it in the same
   report and a root write after: three answers given in order reach the
   root as two; a `NotStored` to the host's write says nothing and the
   next flip writes whole). Each bites.
5. A run row in `tests/suite/run.rs`, if its simulated start fixes
   `Start.scheme`: no `COLORFGBG` and no answer starts with `None`.
6. Runs: the narrowest first, then `heavy cargo nextest run -p
   temper-client-tui -p temper-client-shell-tui -p temper-client-world -p
   temper-client-journeys`; the committed journeys' print test must
   pass unchanged (no reprint: frames do not change).
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui -p temper-client-shell-tui -p
   temper-client-journeys --all-targets -- -D warnings`; the status
   entries; commit, push.

## Report

As the implementer's text asks; say how the writes are noted, and the
bites.
