+++
id = "change-something"
part = "card-change"
kind = "feature"
needs = ["reason-markdown"]
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-domain/src/acts.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/tests/client/tests/suite/configuring_chat.rs",
]
effort = "high"
minutes = 50
+++

# "Change something" shows what it did

## Goal

On a real walk, "Change something" had no visible effect: the focus was already in the composer. Pressing it now focuses the
composer, brings it into view and sets its placeholder to "What should change?". The placeholder goes back once something is
sent or the proposal is decided.

Acceptance:

- Pressing "Change something" moves `composer_focus` by one and sets `Chat.change_asked` to that proposal, so a second press focuses again.
- The composer's placeholder says "What should change?" while `change_asked` is `Some`, and "Reply to T9" otherwise, in both shells.
- A message that goes out clears it, an empty Send doesn't, and a decision from anywhere clears it: `task.proposals` no longer holding the
  key, whichever act or place decided it. A different proposal's decision leaves it.
- The world test presses the card's button in a chat on temper's real engine root.
- Bringing the composer into view comes from skein's focus request (`skein/docs/design/ui-web.md`, 7), which the task `focus-reveal` builds;
  you don't need it merged, since `composer_focus` moving is what asks for it. The plan's `card-proof` render shows it at 420.

## Inputs

- `temper/docs/plan/ui-proposal-card/notes/common.md`.
- `temper/docs/plan/ui-proposal-card/notes/contracts.md`: *"Change something"*. It binds.
- `temper/docs/design/client/README.md`, 5.13, *The card in a chat*, and 3.5's `Chat { task }`, on the composer.

## Owns and may change

**Owns:** the `files` above.

**Readers, each edited only as said:**

- `Act::FocusComposer` goes, and every arm naming it takes `Act::ChangeProposal { .. }`: `pages.rs` (the `Event::Act` arm that routes it to the
  chat, else the task; and its long no-op list), `acts.rs`, `pages/task.rs` (two no-op lists).
- `temper-client-view/src/tests/frame.rs`, near line 1634: the rename-focus test sends `Act::ChangeProposal` with any key in place of
  `FocusComposer`, and keeps what it checks.
- `temper-client-view/src/tests/cards.rs`, near line 659: the assert on what "Change something" is bound to follows the new act.
- `Word::SettingsProposalChangeAsks`, as `notes/common.md`, *A new `Word`*, says.

**Pre-approved options:**

- **`long-words`' composer tasks may land first,** moving the placeholder or `words` into `pages/composer.rs`: on a catch-up put the
  placeholder's branch and the clearing beside their new home, and keep `change_asked` on `Chat`, since it is about a chat's proposal and
  not about a composer's words.
- **If a `Send` refused after going out** would leave the person's words back in the composer: `change_asked` stays cleared. The request says
  "once something is sent".

**Not yours:** skein's `Op::Reveal`; the card's place and state lines; `docs/design/`.

## Steps

1. **The domain's failing tests,** in `temper-client-domain/src/tests/pages/chat.rs`, on the shape of the test near its line 51 (`f.chat.act(...)`,
   `fixtures::acts::sent(&mut f.out)`):
   - `change_something_focuses_the_composer_and_remembers_its_proposal`, twice pressed, the count moving each time;
   - `a_sent_message_ends_the_change_asked`, and an empty `Send` with no words keeping it;
   - `a_decision_ends_the_change_asked`: a delivered task document whose `proposals` lacks the key ends it, and one that still holds it or that
     dropped another key keeps it.
2. **The domain,** as the contract says.
3. **The view's failing test,** `temper-client-view/src/tests/chat.rs`, beside
   `the_message_composer_keeps_its_name_and_has_a_numbered_placeholder` and in both shells as its neighbour near line 2402:
   `the_composer_asks_what_should_change_until_it_is_answered`: "Reply to T9"; after the act, "What should change?"; after a message or the
   decision's document, "Reply to T9".
4. **The view:** the card binds `Act::ChangeProposal { proposal: proposal.key }`; `session_composer` picks the placeholder.
5. **The world test,** `temper/tests/client/tests/suite/configuring_chat.rs`,
   `change_something_asks_in_the_composer_until_a_message_goes_out`, on `propose(false, None)` and the `world.go(person, Address::Chat { task })`
   shape near its line 423: press "Change something", assert `change_asked` is `Some` on `world.root(person).person().pages().chat()`, type and
   press "Send", assert `None`. Run it first against the unfixed code to see it fail.
6. **Checks, once:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view
   -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view
   -p temper-client-shell-terminal -p temper-client-gallery`; `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(configuring)'`. A filter that selects nothing proves nothing: check the count.
7. Commit in small steps that say why, and push.

## Report

As your text says, and: the rule each new test bit on and the failure seen; whether the terminal needed anything; anything under `## Left`.
