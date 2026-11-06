+++
id = "card-place"
part = "card-place"
kind = "feature"
needs = ["proposal-facts", "card-reads", "card-states"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/tests/client/tests/suite/configuring_chat.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 90
+++

# A card stands where it was made, and what comes later comes after it

## Goal

A person stays in the chat and asks for a second proposal: the view rests on the old "✓ Accepted", with the new pending card off-screen above it, so they can think it's done; and the chat's reply lands above the card. No real model's
call names its proposal, so every card takes the fallback, after the conversation. Make that fallback the turn running when it was made. A new pending card is then the last thing in the chat by construction, and the reply after an Accept follows the card.

Acceptance:

- A card its call names stands at the call, as today.
- Any other card stands after the first committed turn following its proposal's making line (`notes/contracts.md`, *A card's place*). With no such turn yet it stands at the conversation's end, after any provisional text.
- A later reply and a second proposal's card stand below it. Cards at one place stand in number order.
- A pending card with no making line in the window goes by `waiting.since`, and isn't lost; a card past the window's end isn't drawn until the window reaches it.
- A making line draws no "Decision made" row, on every page that draws lines.
- A card's element stays keyed as lines arrive around it.
- Both shells draw the same order.

## Inputs

- `temper/docs/plan/ui-proposal-card/notes/common.md`.
- `temper/docs/plan/ui-proposal-card/notes/contracts.md`: *The facts jig gives*, *A card's place*. They bind.
- `temper/docs/design/client/README.md`, 3.5's `Chat { task }`; 5.13, *Where a card stands* and *"Decision made"*.
- `temper/docs/design/client/greenfield/chat.html`, the target's card among its lines.

## Owns and may change

**Owns:** the `files` above.

**Readers, each edited only as said:**

- `conversation_tail`: only cards whose place is `End`. `committed` and `call_draws` keep `proposal_anchor` for `Call`.
- `conversation_with_times`: computes the places before its lines; after a `Line::Turn` is drawn, `run.flush` then its `After` cards. If `chat-reading-c` has landed, that is after the line's keyed `Slot` and its `conversation_tail` keeps its `fixed_key` wrappers.
- `hidden`: `Activity::Decided { proposal: Some(DecidedProposal { made: true, .. }), .. }` is hidden, for chat and agent pages both.
- `proposal_card`: `Role::Slot` for an `After` card, keyed by `inbox::card_key` as an `Item` is (`skein-ui/src/tree.rs`: a `Region` can't be keyed, and these must stay stable as turns arrive). A `Call` card keeps `Role::Region`.
- `tests/chat.rs`, near lines 574 and 905, assert a card in the tail or after the turns: a card with no making line and no time still takes `End`, so change only what the new rule moves.

**Pre-approved options:**

- **The window's calls:** `chat-reading-c` removes `Line::Left`, `LINES` and `offers_older`, and `edges.reaches(End::Before)` stays its test for the newest lines. Take its window as it is on a catch-up; Older's thin first page isn't this plan's.
- **Staying at the end:** skein's log follows on any patch while the person is at its end (`skein-ui-web`'s `page.rs`), so nothing is asked of it here. If `chat-reading-c` has landed and its following no longer does this for a card in the last place, ask nothing of skein: list it under `## Left`, for `card-proof`.
- **The tail's `Role::List` and an `After` card's `Role::Slot`** differ, so a card's element is replaced when its turn commits. Take that.
- **A delegate's proposal** has no line in this activity: it goes by its time, rule 3.

**Not yours:** drawing a decided card with no session snapshot, and a decided proposal's rows: `card-reload`. `docs/design/`.

## Steps

1. **The failing tests,** in `temper-client-view/src/tests/chat.rs`, in both shells, by a helper that reads the tree's order of named things. Every call's `objects` empty, as a real model's:
   - `a_card_stands_after_the_turn_that_made_it_and_a_reply_after_it`: turn 1, the making line, turn 2, turn 3;
   - `a_new_card_is_last_while_its_turn_runs`: the making line and no turn after it, with provisional text;
   - `a_second_proposal_stands_below_the_first`: two making lines, each with its turns, the first accepted;
   - `a_card_with_no_line_goes_by_its_making_time`: a pending card, `waiting.since` between two turns;
   - `a_tie_goes_by_the_line_and_not_the_clock`: every `at` equal, the making line between two turns;
   - `a_card_past_the_window_s_end_is_not_drawn`: a window not reaching the newest lines;
   - `a_call_naming_its_proposal_keeps_the_card_at_the_call`;
   - `a_making_line_draws_no_decision_row`, chat and agent's task.
2. **The view,** as the contract says.
3. **The world,** `temper/tests/client/tests/suite/configuring_chat.rs`, on `propose(false, None)` and `front().configuring_reply(task, words)`, reading the view's tree as `accounts.rs`, near its line 76, builds it:
   `a_second_proposal_stands_below_the_first_and_a_reply_below_it`: propose, reply, reply; accept; propose again with a later revision, reply. The order: first reply, first card, second reply, second card, third reply. Run it first against the unfixed code to see it fail.
4. **The gallery:** a chat's states that show a card and a later reply (`configure-chat-*`) take making lines in `worlds.rs`, so the gallery draws the order; regenerate what they print.
5. **Checks, once:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-gallery -p temper-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal -p temper-client-shell-web`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(configuring) | test(chat)'`. Check the count.
6. Commit in small steps that say why, and push.

## Report

As your text says, and: the rule each new test bit on and the failure seen; whether a real engine's making line precedes its call's turn, as seen in the world; anything under `## Left`.
