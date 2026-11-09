+++
id = "box-ink"
part = "box-ink"
kind = "feature"
needs = ["focused-value", "quit", "notice", "typing", "theme-list"]
files = ["temper/crates/temper-client-tui/src/theme.rs", "temper/crates/temper-client-tui/src/tests/theme.rs", "temper/crates/temper-client-tui/src/tests/golden/*.txt", "temper/docs/design/client/tui/journeys/**", "temper/docs/design/client/README.status.md"]
effort = "medium"
minutes = 50
+++

## Goal

The person's words in the TUI's box are the terminal's own ink: the
accent lies on "❯" alone, and the words and the caret's cell are drawn
in the terminal's default colour, where today "❯ fa" is painted in the
accent across the mark, the words and the cursor's cell (the user's
request of 2026-11-09). Lists keep their own active-item rule, and the
sign-in's link field keeps the focus's paint. This task turns skein's
focused value's paint (`focused-value`) on for `tui-field`, then brings
every golden and journey frame that draws the box up to date. It runs
after every other task of the plan so that it repaints their goldens
too.

Acceptance:

- `temper-client-tui`'s `theme.rs`: `look::FIELD`'s look names
  `focused_value: Some(Paint::NONE)`, in both schemes, with and
  without a band; nothing else in the theme changes.
- `tests/theme.rs`'s table expects it on `look::FIELD` alone.
- Every golden under `src/tests/golden/` whose box line drew the accent
  across the words or the caret now draws the mark (and the space after
  it) in the accent and the words and the caret's cell in no paint; no
  other cell of any golden changes. The cursor's position line is
  unchanged.
- The committed TUI journeys' print test passes, their frames reprinted.
- The README.status.md entry for 6.15, *Its glyphs, its focus and the
  active choice*, and its `tui-field`, gone.

## Inputs

- `temper/docs/design/client/README.md`, 6.15 (*Its tokens*, the
  `tui-field` row; *Its glyphs, its focus and the active choice*).
- `skein/docs/design/ui-screen.md`, section 3 (`focused_value`) and 5.4.
- `temper/docs/plan/tui-box-polish/notes/contracts.md`, *skein-ui-screen*.
- `temper/docs/plan/tui-box-polish/notes/common.md` (*Goldens*,
  *Journeys*).

## Owns and may change

- Owns: `temper-client-tui`'s `src/theme.rs` (the `look::FIELD` line in
  `styles` alone), `src/tests/theme.rs`, every file under
  `src/tests/golden/`, the reprinted journeys, and the status entry.
- A golden that changes elsewhere than its box line is a red, not
  stale: stop and say which, with the printed difference.
- `temper-client-shell-tui`'s or the joint world's tests that read a
  frame's paint for the box (grep `ACCENT`, `focus` and `paint` in their
  frame assertions) follow, assertion lines only; say which.

## Steps

1. The theme line; the table's expectation in `tests/theme.rs`.
2. `heavy cargo nextest run -p temper-client-tui --lib`: each golden
   test prints its mismatches; check each is the box line's paint row
   alone (and only the words' and caret's cells), then copy it.
3. `heavy cargo nextest run -p temper-client-journeys --lib -E
   'test(=tests::the_committed_journeys_are_their_print)'`; reprint
   (`TEMPER_BUNDLE=none heavy cargo run -p temper-client-journeys`) and
   check with `git diff --stat` that only the TUI journeys' pages
   changed.
4. `heavy cargo nextest run -p temper-client-tui -p
   temper-client-shell-tui -p temper-client-world -p
   temper-client-journeys`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui --all-targets -- -D warnings`; the status
   entry; commit, push.

## Report

As the implementer's text asks; the goldens and journey pages changed,
by name.
