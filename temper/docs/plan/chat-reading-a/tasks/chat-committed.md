+++
id = "chat-committed"
part = "conversation"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/cards/words.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/delegation.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "medium"
minutes = 45
+++

# A conversation says nothing of what was cut, and draws no reasoning

## Goal

Three small removals in temper's conversation drawings, for a chat and for an agent's page. "Earlier text omitted" goes. A turn's opaque blocks ("Content in 3", "2164 bytes") go, with nothing in their place. And a result's work line on an agent's page draws its first line, not a result's body.

Acceptance:

- "Earlier text omitted" is drawn nowhere: not for a cut committed turn, a cut request, a cut live text, nor past `rows` messages or blocks. The phrase is gone, and its number stays unused.
- A cut turn draws what it holds and no notice. A cut text's `…` is the bytes' own (`turn-order` writes it): this task adds no mark.
- A turn's `c::Block::Opaque` draws nothing, in `committed` and in `committed_lines`. A message holding only opaque blocks draws no message at all: no "Temper" and no empty body.
- `committed_lines` draws a result's summary as its first line with words. `delegation::results` keeps reading the summary whole, so a delegate's card still anchors at its result.
- `Line::Left`'s "Earlier lines omitted N" stands (section 5's).
- The **Not yet** line beginning `3.5, *the agent's words*` is gone from `temper/docs/design/client/README.status.md`.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *A chat's conversation* (*the agent's words*, *not drawn*), and its agent's page's *The conversation*.
- `temper/docs/plan/chat-reading-a/notes/common.md`.

## Owns and may change

Owns the conversation's drawing in `pages/chat.rs`, and the phrases in `cards/words.rs`.

May also change, these edits alone:

- `temper/crates/temper/tests/support/person.rs`: the one `Step::Gone` naming "Earlier text omitted" goes, as it would now hold of nothing. No other step moves.
- `temper/crates/temper-client-view/src/limits.rs`: nothing. Its comments on the cut status go with `turn-prices`, which reprices that file; a node priced and no longer drawn is harmless.

Options, each pre-approved:

- **`P::OpaqueDialect` and `P::OpaqueBytes`:** if nothing else draws them (`rg` says), they go too, their numbers unused. If something does, they stay.
- **`prefix`:** it keeps its slicing, and loses its status and `words`. `markdown` in the same file loses its status. If `provisional`'s `cut` then has no reader, its argument goes.
- **The first line helper:** a small private function in `pages/chat.rs`, no new module. It skips leading blank lines and drops a trailing `\r`, as `notes/contracts.md` says of an outcome.

## Steps

1. **Red first,** in `temper/crates/temper-client-view/src/tests/chat.rs`, beside `long_provisional_text_says_that_earlier_text_was_omitted`, which becomes the first of these:

   ```rust
   #[test]
   fn a_long_provisional_text_says_nothing_of_what_was_cut()
   #[test]
   fn a_cut_committed_turn_says_nothing_of_what_was_cut() // chat page and agent page
   #[test]
   fn a_turns_opaque_blocks_are_not_drawn() // chat page and agent page: no "Content in", no "bytes"
   #[test]
   fn a_message_of_opaque_blocks_alone_draws_no_message()
   #[test]
   fn an_agents_result_line_is_its_first_line() // "first\nsecond": "first" shown, "second" not
   ```

   `tests/delegation.rs`' `b"Earlier text omitted"` entries and its `let cut` test follow; a test there that a delegate's result still anchors its card, from a multi-line summary, stands or is added.
2. **The removals,** as the acceptance says. Keep the hunks in `pages/chat.rs` small. `ui-final-walk` has changed this crate lately, and `chat-live` follows you here.
3. **Run:** `heavy cargo nextest run -p temper-client-view -p temper-client-gallery`, and `heavy cargo check -p temper --tests`.
4. Fmt, clippy on the packages you changed, commit and push.

## Report

As the implementer's text says, and: whether the opaque phrases went.
