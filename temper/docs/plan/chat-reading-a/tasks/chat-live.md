+++
id = "chat-live"
part = "live-page"
kind = "feature"
needs = ["growing-reader", "turn-prices"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/cards/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/tests/client/tests/suite/live_text.rs",
  "temper/tests/real-loop/src/terminal.rs",
  "temper/tests/real-loop/tests/suite/terminal.rs",
  "temper/tests/real-loop/tests/live_text.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# The answer only grows, and takes its committed place without moving

## Goal

Draw the turn being written as its committed turn will be drawn: the same message, the same blocks, read by `read_growing`, with a small block after the last word in place of "Writing...". When the turn commits, only that block goes. This is what the user sees.

Acceptance:

- A turn being written draws as a committed turn's message does: `look::MESSAGE`, `look::BY_AGENT`, a who slot saying `P::Temper`, and a says slot read by `read_growing`.
- For a text that ends closed, the says slot's nodes while it's being written, the end's nodes aside, are the nodes its committed turn draws. A test compares them, in a chat and on an agent's page.
- A block closed by a blank line keeps its nodes, and the diff emits no patch for them, as words arrive after it. A test compares the patches of two deliveries.
- No raw marker shows: a live text ending in `**bo`, `##` or an open fence draws a bold, nothing and code, as `read_growing`'s tests say. One test here proves the page takes that reader.
- The end is `P::Provisional`'s `▍`, styled `look::WRITING_END` (class `temper-writing-end`), inside the block holding the last word, and it's gone once the turn commits. "Writing..." is drawn nowhere.
- A turn with no words yet draws no message, and the three dots stand, as `working` says: an empty provisional line no longer holds them back. Once it holds a word, they go.
- In the browser the mark is faint and in the line of the last word: a single rule in `temper.css`, beside `.temper-typing`.
- `temper/tests/client`'s and `temper/tests/real-loop`'s stories read the new mark.
- The **Not yet** line beginning `3.5, *the turn being written*` is gone from `temper/docs/design/client/README.status.md`.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *the turn being written* and *While it works*, and 6.9.
- `skein/docs/design/ui.md`, section 10, *text still being written*.
- `temper/docs/plan/chat-reading-a/notes/common.md`.
- `temper/docs/plan/chat-reading-a/notes/contracts.md`, `read_growing` and *The turn being written's end*.

## Owns and may change

Owns the provisional line's drawing and its word, and the look's one new style.

May also change, these edits alone:

- `temper/crates/temper-client-shell-web/temper.css`: one rule, beside `.temper-typing`: the class inline, in `--gf-hint`, with its children inline. `ui-rename-focus`, `ui-final-walk` and perhaps `chat-work` hold this file: keep the hunk this small.
- `temper/crates/temper-client-view/src/limits.rs`: its comment on the provisional line, if `turn-prices` left it untrue. The nodes are priced.
- `temper/crates/temper-client-shell-terminal/src/tests/**`: only if a print test reads the view's word. Its fixtures' own "Writing..." are a fixture's text: leave them.

Options, each pre-approved:

- **The look's token:** the next number above the highest in `look.rs`, and its row in `tests/look.rs`. On a merge with another draft's new token, take the next free one.
- **Where the end's nodes sit:** a `Role::Status` from `words.status`, inside a slot styled `WRITING_END`, or the style on the status itself, whichever `words.status` allows without a new word function.
- **How the terminal prints the mark:** as the text shell prints a status inside its block, on its own line if it must. `PROVISIONAL_LINE` in `temper/tests/real-loop/tests/live_text.rs` becomes what it prints, read from a run, and your report gives it.
- **`text_limit`:** `provisional` stops taking it. If `markdown` in `pages/chat.rs` then has no reader, it goes, and so does the argument where nothing else reads it.
- **An empty line and the clock:** `turn_time` is not called for a line that draws nothing, so an empty turn writes no day or clock.

## Steps

1. **Red first,** in `temper/crates/temper-client-view/src/tests/chat.rs`; the tests at its lines naming `b"Writing..."` move to the mark:

   ```rust
   #[test]
   fn a_turn_being_written_draws_as_its_committed_turn_with_its_end() // chat and agent page
   #[test]
   fn a_closed_block_emits_no_patch_as_words_arrive_after_it()
   #[test]
   fn a_turn_being_written_shows_no_raw_marker()
   #[test]
   fn a_turn_with_no_words_draws_nothing_and_the_dots_stand()
   #[test]
   fn the_first_word_takes_the_dots_place()
   #[test]
   fn the_end_goes_when_the_turn_commits()
   ```

2. **The drawing:** `provisional` in `pages/chat.rs`, calling `skein_ui::markdown::read_growing` with the end, and `working` in `pages/chats.rs`.
3. **The worlds:** `WRITING` in `temper/tests/client/tests/suite/live_text.rs` becomes the mark. `PROVISIONAL` and `PROVISIONAL_LINE` in the real loop likewise, and `Act::See`'s `lacks` and `printed` in `temper/tests/real-loop/src/terminal.rs`.
4. **Run:** `heavy cargo nextest run -p temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-world -p temper-client-gallery`. The real loop's stories run in their own tier: `heavy cargo check -p temper-real-loop-world --tests`, then its `live_text` binary once, by that tier's profile (`tools/tiers/design.md`). A test you can't run is named in your report.
5. Fmt, clippy on the packages you changed, commit and push.

## Report

As the implementer's text says, and: what the terminal prints for a turn being written, and the rule added to `temper.css`.
