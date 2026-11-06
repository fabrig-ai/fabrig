+++
id = "card-reads"
part = "card-reads"
kind = "feature"
needs = ["change-something"]
files = [
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
]
effort = "high"
minutes = 45
+++

# A card's read that ends without a value is asked again

## Goal

Any answer to a chat card's read that isn't a value sets its page to `None`, and nothing asks again, so a busy engine leaves a card stuck: this is one route to a
rejected card that says "Waiting for an owner". A card's read follows the rule its conversation's reads follow. In the same crate, the chat's cards take a stable order,
which the next tasks draw by.

Acceptance:

- A card's read ending transiently (`read_end(..).transient()`) keeps the page it had and is asked again, after one second, doubling to at most thirty, and a value resets the wait.
- A lasting end (`Gone`, `Unknown`, `Refused`, too large) sets the page to `None`, as today, and isn't asked again.
- A retained settings card's failed re-read keeps its old page and is asked again, so it can't stay stale.
- `Chat.cards` hold ascending `key.number`, whatever order the task document lists them in, retained cards among them.
- `Chat.decided` and `Chat.departed_proposal` keep the newest proposal's facts: a read answered later for an older proposal doesn't overwrite them.

## Inputs

- `temper/docs/plan/ui-proposal-card/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5's opening (a transient read end, asked again on these waits) and 5.13, *The card in a chat*.

## Owns and may change

**Owns:** the `files` above.

**Readers, none edited:** `Chat.cards` is read by object in `pages.rs` and the view's `confirm.rs`, so order doesn't matter there; the view's `pages/chat.rs` draws them in
list order. `pages/document.rs` reads `decided` and `departed_proposal`.

**Pre-approved options:**

- **The waits' state:** `Asking` holds one read and a chat has many cards, so a chat-level wait over the handles of the cards owed a read is expected: `Chat::fire` re-reads
  them at its deadline and `Chat::next_deadline` includes it. If that proves clumsy, your choice, and say so.
- **A test that assumes the document's order** of `chat.cards` moves with the rule.

**Not yours:** the view, which `card-states` is changing beside you; `pages/needs.rs`; the settings snapshots, which `card-reload` removes; `docs/design/`.

## Steps

1. **The failing tests,** in `tests/pages/chat.rs`, beside `a_retained_settings_card_keeps_the_documents_acceptance_time` and on its shape:
   - `a_card_read_that_ends_busy_is_asked_again`, each of `ReadNone::Busy`, `ReadNone::Unreachable` and `Readout::Limit`, seeing the read queued only once its deadline is reached;
   - `the_wait_doubles_and_a_value_resets_it`;
   - `a_lasting_end_is_not_asked_again`, `ReadNone::Gone` and `Refused`;
   - `a_retained_card_s_failed_re_read_keeps_its_page_and_asks_again`;
   - `cards_hold_their_proposals_in_number_order`;
   - `an_older_proposal_s_answer_keeps_the_newest_decision`.
2. **The domain,** in `told`'s card arm, `proposals`, `fire` and `next_deadline`.
3. **Checks, once:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-domain -p temper-client-view`; and, since this changes when a page asks, `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(configuring) | test(chat)'`. Check the count.
4. Commit in small steps that say why, and push.

## Report

As your text says, and: the rule each new test bit on and the failure seen; the wait's shape you chose; anything under `## Left`.
