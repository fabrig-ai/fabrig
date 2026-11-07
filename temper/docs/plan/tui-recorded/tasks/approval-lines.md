+++
id = "approval-lines"
part = "tui-approval-flow"
kind = "feature"
needs = ["tui-surface"]
files = ["temper/crates/temper-client-tui/src/view.rs", "temper/crates/temper-client-tui/src/words.rs", "temper/crates/temper-client-tui/src/look.rs", "temper/crates/temper-client-tui/src/limits.rs", "temper/crates/temper-client-tui/src/tests/**"]
effort = "high"
minutes = 150
+++

## Goal

The TUI's sign-in screen draws the browser's approval, as `temper/docs/design/client/README.md`, 7.1, *The TUI's flow* has it: "Asking temper for a code…" while the ask is in flight; the tab's lines with the code, the address and "The code expires in SPAN." while it waits, as opened or, with no browser, the address printed with the tunnel's line; each outcome's line from 7.1's table; and the paste's lines as before, the approval's giving way once a pasted link was sent. The span ticks (5.12). Goldens of the approval's frame sit beside M1's.

Acceptance:

- The screen's lines, in the view's tree, from `root.asked()`, `root.client().link().not_signed_in`, `Tui::browser()` and `Tui::pasted()` (contracts, section 1), in 7.1's words exactly, laid above the paste's lines (6.14, *Room for the browser's approval*): the screen's first line stays 6.14's "Sign in this terminal to HOST:PORT", said once, then the approval's lines (7.1's block from its second line), then "Paste the link temper printed, then Enter." and the field as today.
- A table test in `src/tests/screens.rs` (or its nearest table): one row per state of 7.1's table (waiting as opened, waiting with no browser, asking, refused, expired, used, unknown, full, the take busy, not ready and lost, not asked refused, busy, not ready and lost) and a pasted link's row, each the frame's text lines expected; red first for the outcome lines.
- The span: "The code expires in 10 min." at the ask, "under 1 min" in its last minute, as 5.12's *span* of the time to the expiry; `View::tick` gives the next instant the drawn span changes, and `None` when no span is drawn; a row steps the wall to either side of a minute's bound.
- Goldens, written by hand from the design and the theme (as `src/tests/goldens.rs` says of the others): `approval-80x24-dark.txt`, `approval-80x24-light.txt` (waiting, opened, code `WDJB-MJHT`, ten minutes left) and `approval-no-browser-80x24-dark.txt` (the printed address), rows of `the_tuis_screens_are_their_goldens`.
- The TUI's tree stays within its limits with the longest approval lines (`src/tests/limits.rs`'s table gains a row); `src/limits.rs` grows a bound only where a row needs it.
- `heavy cargo nextest run -p temper-client-tui -p temper-client-shell-tui` passes; clippy clean for both.

## Inputs

- `temper/docs/design/client/README.md`, 7.1, *The TUI's flow* (its lines, the no-browser lines, *Asking again*, *A pasted link, sent*, the outcomes' table); 6.14 (the screens, their words, the paste); 6.15 (tokens: `tui-hint`, `tui-waiting`, `tui-refused`, `tui-down`); 5.12 (*span*, *The tick*); 5.1 (the full deployment's words, "the earliest ends MOMENT UTC").
- `temper/docs/plan/tui-recorded/notes/contracts.md`, section 1.
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code: `src/view.rs` (`sign_in`, `sign_in_line`), `src/words.rs` (the phrase table, `Word`, `ALL`, `english`), `temper-client-view`'s `words` for shared phrases and its span form (`rg -n 'fn span|Span' temper/crates/temper-client-view/src`), `src/tests/goldens.rs` and `golden/`.

## Owns and may change

- Owned: `temper-client-tui`'s view, words, look, limits and tests (the files above).
- Not yours: `state.rs`, `approval.rs` and `lib.rs`' exports (`tui-surface`'s, merged): read them; a change they seem to need is a question.
- Readers: `temper-client-shell-tui`'s host tests read the frame's text (`rg -n 'Signing in|That is not' temper/crates/temper-client-shell-tui temper/tests/client`); a line of theirs your words move is yours to repair, keeping what it checks. `temper/tests/client` is `approval-story`'s; if one of its assertions on the sign-in screen's text breaks, repair that line alone and say so.
- Options:
  - The span's words through `temper-client-view`'s phrase (default) or a TUI phrase with a span hole, if the view's is not reachable from `words.view()`: say which.
  - The expiry's line drawn dim (`tui-hint`), the outcome's line refused (`tui-refused`) and "temper is busy; still trying…" waiting (`tui-waiting`) (default); another token only where 6.15 names it.
  - A line too long for 40 columns wraps as the renderer wraps it; no golden at 40 by 12 is asked.

## Steps

1. Read the inputs. Add the words (each phrase exactly as 7.1 writes it), then the view's lines, then the tick.
2. Tests: the outcome rows red first, then the lines; the span rows; the goldens; the limits row. Break a rule once by hand (an outcome mapped to another line; the tick never set), see it fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tui -p temper-client-shell-tui --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-tui -p temper-client-shell-tui`.
4. Commit and push.

## Report

The implementer's report, with: each phrase added; the goldens written and how each was checked against the design; the rows and their broken-rule failures; any line of another crate's test repaired.
