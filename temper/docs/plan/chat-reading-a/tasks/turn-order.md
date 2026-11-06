+++
id = "turn-order"
part = "committed-turn"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-protocol-transcript/src/lib.rs",
  "jig/crates/jig-protocol-transcript/src/tests.rs",
  "jig/crates/jig-client-domain/src/contract/turn.rs",
  "jig/crates/jig-client-protocol-link/src/tests/turn_text.rs",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "high"
minutes = 60
+++

# A committed turn keeps its calls' names, whatever its results weigh

## Goal

`jig-protocol-transcript` shares one budget across a turn's text, in block order, so a few results use it up and later calls arrive with no names. Price the names first, and make a call's outcome its first line, so that what a result weighs can no longer take a name or the answer.

Acceptance:

- A turn whose results fill the budget keeps every call's name whole, within `TurnLimits.blocks` and `messages`.
- A turn's texts come before inputs, outcomes and summaries: a text within the budget's room after the names is whole.
- A call's outcome outlives an input that fills the budget: a call with an input past the budget and a one-line result keeps its result's line.
- `Call.outcome` is the result's first line with words, within `TurnLimits.outcome`. `Block::Result.summary` stays whole, priced last, within what's left.
- A text cut, by `outcome` or by the budget, ends in `…` where it was cut, on a UTF-8 boundary. The text bytes of all of a turn's blocks stay within `TurnLimits.text`. `Turn.cut` is set exactly when something was cut or left out.
- `Delegated tasks: 1, 2` and `Note 4 revision 2 written · 12 lines` outcomes read as they are written, so `temper-client-domain`'s `delegated` and `note_write` read what they read today. A test holds both.
- The **Not yet** line beginning `10, 14:` is gone from `jig/docs/design/client/domain/client.status.md`.

## Inputs

- `jig/docs/design/client/domain/client.md`, section 10, *Turns, decoded below the client's surface*, and section 14, on `turn_text`.
- `temper/docs/design/draft/chat-reading.status.md`, *Design departs*, 4: why an outcome's line comes before inputs.
- `temper/docs/plan/chat-reading-a/notes/common.md`.
- `temper/docs/plan/chat-reading-a/notes/contracts.md`, `TurnLimits`, which holds the pricing exactly.

## Owns and may change

Owns `jig-protocol-transcript`, `TurnLimits`, and `jig-client-protocol-link`'s `tests/turn_text.rs`.

May also change, this edit alone:

- `jig/crates/jig-client-protocol-link/src/limits.rs`: the one line `outcome: client.text` in the `TurnLimits` literal. `live-whole` owns the rest of that file and that crate this wave.
- The `TurnLimits { .. }` literals in `jig/testing/jig-test-store/src/route.rs` and `jig/testing/jig-client-kit/src/joint/{account_moves,fixture,engine_stories,session,role_fixture,front}.rs`: the new field, at their `text`.
- A test elsewhere that pins the old pricing or a whole multi-line `Call.outcome`, in the readers below: its expected value, to the new rule. Say which in your report.

Options, each pre-approved:

- **How the passes are built:** build the blocks with names, then fill each later price in its own pass over `source` and the turn in step, or work out each block's allowance first, whichever reads plainest. Keep `no_std`, and allocate each text at its length.
- **`brief_text`:** it renders the decoded blocks, so it gains the new pricing with no change of its own. A brief's whole-result test that this breaks is a red to look at: if `brief.rs` read `Call.outcome`, that would be one, but it reads `Block::Result.summary`.
- **`temper-client-tools`' `read_turns.rs`** prints `outcome` as a turn has it. Its tests build turns directly, so they should not move. If one does, change its expected value and say so.

## Steps

1. **Red first,** in `jig/crates/jig-protocol-transcript/src/tests.rs`, with its `source`, `message`, `call` and `result` helpers:

   ```rust
   #[test]
   fn results_that_fill_the_budget_leave_every_name_whole()
   #[test]
   fn texts_are_priced_after_names_and_before_the_rest()
   #[test]
   fn an_input_that_fills_the_budget_leaves_its_outcomes_line()
   #[test]
   fn an_outcome_is_its_first_line_with_words_and_its_summary_stays_whole() // "\n\r\nfirst\r\nsecond"
   #[test]
   fn an_outcome_line_past_its_bound_ends_in_the_mark()
   #[test]
   fn a_text_cut_at_the_bound_ends_in_the_mark_on_a_character_boundary()
   #[test]
   fn fewer_than_three_bytes_left_leave_a_text_empty_and_unmarked()
   #[test]
   fn the_outcomes_cards_read_stay_as_written() // "Delegated tasks: 1, 2", "Note 4 revision 2 written · 12 lines"
   ```

2. **The pricing and the first line,** in `translate`. `receiving_limits_cut_without_growing_or_losing_call_outcomes` changes to the new order.
3. **`tests/turn_text.rs`** in the link: `texts_one_past_the_turn_budget_are_cut` expects the mark; a page's archived turn past a field's bound keeps decoding.
4. **A change of meaning:** run the readers' tests, each by `heavy cargo nextest run`: `-p jig-protocol-transcript -p jig-client-protocol-link -p jig-client -p jig-client-view -p jig-client-kit`, `-p temper-client-domain -p temper-client-view -p temper-client-tools`, and `heavy cargo check -p jig-test-store --tests`.
5. Fmt, clippy on the packages you changed, commit and push.

## Report

As the implementer's text says, and: every test outside the crate whose expected value moved, and why.
