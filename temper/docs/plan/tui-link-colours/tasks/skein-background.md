+++
id = "skein-background"
part = "skein-background"
kind = "feature"
needs = []
files = ["skein/crates/skein-ui-screen-terminal/src/**", "skein/tests/ui-screen/**", "skein/docs/design/ui-screen.status.md", "temper/crates/temper-client-shell-tui/src/limits.rs"]
effort = "high"
minutes = 150
+++

# tui-link-colours: skein's full-screen terminal asks the terminal's background

## Goal

skein's full-screen terminal can ask the terminal its background with OSC 11 at start,
wait a given time for the answer, and say what it heard, and its decoder never reads such
an answer as a key, a late one included. A colour that is none is the terminal's own
default, drawn as SGR 39 and 49, which the drawing's reset already gives: that holds
today, and this task pins it with one row. What the colour means, dark or light, is the
application's (temper's `tui-colours` and `run`).

Acceptance:

- `skein_ui_screen_terminal` gains, as `temper/docs/plan/tui-link-colours/notes/contracts.md`,
  section 2, spells them: `ASK_BACKGROUND`, `Background`, `Settings.background`,
  `Terminal::background`, `Decoder::answers`, `Decoder::background`, exported from the
  crate's root beside `ENTER` and `LEAVE`.
- The decoder (`keys.rs`), as `skein/docs/design/ui-screen.md`, 6.1, *The terminal's
  answers*: before `answers`, `ESC ]` is `Alt(']')` as now. After it, `ESC ]` begins an
  answer read through BEL (`0x07`) or `ESC \`, never a key, split across reads at any
  byte; `ESC ] 11 ; rgb:R/G/B` with 1 to 4 hex digits a component, either case, is the
  background, each component its first two digits, or its one digit doubled (`f` is
  `0xff`); another `11` answer is the background unreadable; any other OSC is dropped; one
  past 64 bytes is dropped whole through its terminator, and what follows decodes as
  keys. `Decoder::background` gives the first `11` answer only. The partial buffer
  grows to the answer's bound; CSI sequences keep their 16 bytes.
- The terminal (`terminal.rs`), as `ui-screen.md`, section 8, *Entering* and *The
  terminal's background*: with `background: Some(wait)`, `ASK_BACKGROUND` is written right
  after `ENTER`, in the same control write or the next, and the decoder is set to
  answers; `background()` is `Waiting` until the decoder holds an answer (`Answered(rgb)`,
  or `Unanswered` for an unreadable one), the wait passes from the first `iterate` after
  `adopt`, or the input ends (`Unanswered`); `next_deadline` includes the wait's end while
  `Waiting`, and `work_pending` is true once it is due. With `background: None` nothing
  is asked and it is `Unanswered` from `new`. `Terminal::operations` and `new`'s refusals
  are unchanged: a wait of zero is allowed and passes at the first iteration.
- Rows, each a failing test first:
  - `src/tests/keys.rs`, in its decoder table and its split-at-every-byte loop: the
    background with BEL and with `ESC \`, one, two, three and four digits, upper and lower
    case; `11;rgb:` with a component of five digits or none, and `11;rgba:...`, as the
    background unreadable; `ESC ] 0 ; title BEL` dropped; an answer of 65 bytes dropped and
    the key after it kept; `ESC ]` as `Alt(']')` before `answers`; no answer row yields a
    key.
  - `src/tests/draw.rs`: a cell whose paint names no colour, after a coloured one, is
    written after `ESC [ 0 m` with no `38;` and no `48;` (the terminal's defaults).
  - `skein/tests/ui-screen/tests/suite/terminal.rs`, beside
    `enter_is_written_before_the_first_frame`: with a wait, `ASK_BACKGROUND` follows
    `ENTER` and precedes any frame; with none, the output never holds `\x1b]11;`;
    the answer fed answers `Answered`; nothing fed past the wait is `Unanswered` and the
    deadline was the wait's end; the input's end while waiting is `Unanswered`; keys fed
    with the answer in one read reach `keys()` and the answer does not. Its
    `settings(most)` helper gains `background: None`; the refusal table's literal its
    field.
- The cross-tree literal: `temper/crates/temper-client-shell-tui/src/limits.rs`'s
  `skein_ui_screen_terminal::Settings { .. }` gains `background: None`, and nothing else
  there changes (the task `run` sets it). Any other literal of these `Settings` the
  compiler names in temper is edited the same way and named in the report.
- `skein/docs/design/ui-screen.status.md`: delete the **Not yet** entry "6.1, *The
  terminal's answers*, and 8, ... are not written." and nothing else (a stated departure
  from the implementer's text).

## Inputs

- `skein/docs/design/ui-screen.md`: section 3, *The terminal's default*; 4, *Its print*;
  6.1, the table, *Split across reads*, *Bounded* and *The terminal's answers*; 7,
  *Colours*; 8, *Its settings*, *Entering*, *The terminal's background*, *Keys*; 9, *The
  keys*, *The drawing*, *The answers*, *The terminal*.
- `temper/docs/plan/tui-link-colours/notes/common.md`;
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, section 2.

## Owns and may change

- Owns `skein-ui-screen-terminal` and `skein/tests/ui-screen` for this wave.
- Not `skein-ui-screen`: `Rgb` and `Paint` keep their shapes; this task adds no colour
  type. If the decoder needs `Rgb`, it takes `skein_ui_screen::Rgb`, which the crate
  already depends on.
- `Key` and `Decoder::new(paste)` keep their shapes, so `temper/tools/journeys` and the
  TUI's journey module need nothing.
- If the terminal's control write is a single bounded buffer that `ENTER` fills, write
  `ASK_BACKGROUND` as its own control write after it and say so.
- If a terminal's answer arrives between `ENTER`'s write and the first read, it is read
  as any input: no ordering beyond "after `ENTER`" is promised.

## Steps

1. The rows; run them (`heavy cargo nextest run -p skein-ui-screen-terminal --lib`,
   `heavy cargo nextest run -p skein-ui-screen-tests`); see them fail.
2. The decoder's answers, then the terminal's question and wait.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-ui-screen-terminal -p skein-ui-screen-tests --all-targets -- -D warnings`; the
   focused tests of both; `heavy cargo check -p temper-client-shell-tui -p
   temper-client-journeys -p temper --tests`; `heavy cargo nextest run -p
   temper-client-shell-tui` (its run on the simulated terminal must stay as it is).
4. Commit, push.

## Report

As the implementer's text says; name the bound you gave the partial buffer, how the wait
is timed from the first iteration, and each row's failure seen.
