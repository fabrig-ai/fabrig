+++
id = "card-states"
part = "card-states"
kind = "feature"
needs = ["change-something"]
files = [
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/tests/client/tests/suite/configuring_chat.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/src/worlds.rs",
]
effort = "high"
minutes = 70
+++

# A decided card ends in its state line, and "Waiting for an owner" means it

## Goal

A real walk showed a rejected proposal's card saying "Waiting for an owner", "Owners of this deployment", with no acts, to the owner who had just rejected it. Prove the route, then
give the card its ruling: a rejected card ends "Rejected · nothing changes", a withdrawn one with nothing replacing it "Withdrawn · nothing changes", and "Waiting for an owner" shows only
while the proposal is pending and this person can't act. A decision made elsewhere reaches an open chat's card without a reload.

Acceptance:

- **The route is proved by a test that fails before the fix.** A survey's reading, not proved: a chat reopened after the decision restores a snapshot with its acts cleared and its page `None`
  (`pages/needs.rs`, `restore_settings`), so with no answer yet the card takes the waiting branch; a read that ends without a value holds it there. Show it red, and say in the report what you
  found, whatever it is.
- **The state lines,** muted, as an accepted one ends: rejected, "Rejected · nothing changes"; withdrawn, "Withdrawn · nothing changes"; withdrawn with a later settings card of the chat's,
  "Replaced by the proposal below", dimmed. A rejected card stays "Rejected" with a later card below it. Accepted is unchanged.
- **Waiting** shows only on a page that is a live `Proposal`, with no Accept offered. With no page yet, the card draws its title and lines and none of these.
- **Replaced** no longer needs an anchor: `settings_replaced` asks whether a settings card with a higher `key.number` is in `chat.cards`.
- **The world:** `an_owner_rejects_a_chats_pending_proposal_under_needs_you` runs, un-ignored, and a new story has a decision from Needs you reach a chat open in another client.
- **The gallery:** states `configure-chat-rejected` and `configure-chat-withdrawn`.

## Inputs

- `temper/docs/plan/ui-proposal-card/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.13, *The card in a chat*; section 11, *A Reject on a chat's card*.
- `temper/docs/design/client/browser.md`, 7.2, for the gallery's states.

## Owns and may change

**Owns:** the `files` above.

**Readers, each edited only as said:**

- `settings_replaced`'s one caller is `proposal_card`, in the view's `pages/chat.rs`: drop its `rows`. No other edit to that file: `card-place` owns it next.
- `SettingsState`: `Withdrawn` splits into `Rejected` and `Withdrawn`, and `ProposalChoice::Rejected` and `ProposalChoice::Withdrawn`, `ProposalHistoryState::Rejected` and `Withdrawn`, map each to its own.
  A new `Reading` for a card with no page and no outcome saying otherwise.
- `Word::SettingsProposalRejected` and `Word::SettingsProposalWithdrawn`, as `notes/common.md`, *A new `Word`*, says.
- `temper.css` needs nothing: `.temper-settings-proposal-state` is the muted line, and `-ok` alone colours accepted.

**Pre-approved options:**

- **The ignored test's reason says jig refuses the rejection as `NotWellFormed(Running)`.** A survey found that stale: `temper-client-domain/src/acts.rs`, near line 230, sends `running` on Accept alone, and
  `chat_accept.rs`' `a_configuring_chat_rejects_its_own_proposal` passes. Un-ignore it and run it. If it passes, say so. If its steps are stale, repair them keeping what it asserts. If the refusal is still real, fix it
  (`acts.rs` is one line of the domain, which `card-reads` holds beside you: keep to that line) and say so.
- **A member's view:** `a_members_configuring_proposal_waits_for_an_owner` keeps its "Waiting for an owner". If a non-owner's read is refused so no settings card is drawn, that is today's and stays; say so in the report.
- **`names.txt`** must match `samples()`' order (`tests.rs`, near line 8): keep it so, sorted as it is.

**Not yours:** the card's place (`card-place`); the domain's re-asking (`card-reads`, beside you); the snapshots' removal and a decided card after a reload (`card-reload`); `docs/design/`.

## Steps

1. **Prove the route.** In `temper-client-view/src/tests/cards.rs`, on `settings_root` and `draw_settings`: `a_restored_card_says_no_waiting_before_its_read`: a settings proposal told while on Needs you, the chat opened,
   drawn before any answer; assert no "Waiting for an owner". See it red. Then, with no answer but `ReadNone::Busy`, the same, for the route that stays. Write down what you saw.
2. **The state lines' failing tests,** same file: `a_rejected_card_says_rejected_and_nothing_changes`, `a_withdrawn_card_says_withdrawn_and_nothing_changes`, `a_withdrawn_card_with_a_later_one_is_replaced`,
   `a_rejected_card_with_a_later_one_stays_rejected`, each from an `Ended` page and from an answered outcome, asserting no "Waiting for an owner" and no acts. The neighbours near lines 684 to 735 show the shape.
3. **The card,** in `settings`, `settings_state` and `settings_replaced`.
4. **The world,** `temper/tests/client/tests/suite/configuring_chat.rs`: remove the `#[ignore]`; and `a_decision_from_needs_you_reaches_an_open_chats_card`, on `propose(false, None)`: one client on
   `Address::Chat { task }`, seeing the proposal; another of the same party rejecting it under Needs you, by the first's steps; then the chat's client sees "Rejected · nothing changes" and neither "Waiting for an owner" nor "Accept". Run it first
   against the unfixed code to see it fail.
5. **The gallery:** the two states, each by the neighbouring `configure-chat-accepted`'s way, with `Holds`.
6. **Checks, once:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-gallery -p temper-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(configuring) | test(chat_accept) | test(owner_rejects) | test(decision_from_needs)'`.
   Check the count.
7. Commit in small steps that say why, and push.

## Report

As your text says, and: **the route you proved, and how**; whether the ignored test's refusal was real; the rule each new test bit on and the failure seen; anything under `## Left`.

## Rebase (2026-11-06, step merge, foreman ui-proposal-card/card-states/foreman-1)

preserve the card-state tests while adding the retained proposal fields from integration
The merge conflicts in `temper/crates/temper-client-view/src/tests/cards.rs`.
Integration commit `566a54948c` (`Carry proposal card facts through the client wire and contract`), merged by `d79403f88e`, adds `ProposalHistory.reason` and `RetainedActionSettings.projects` to the old `replaced_and_accepted_say_so` fixture.
This branch replaces that fixture with `decided_settings` and `an_accepted_card_keeps_its_state_line`.
Keep both new helpers/tests and all restored, rejected, withdrawn, and replacement assertions.
In both new `ProposalHistory` initializers, add integration’s complete `"review"` reason fixture; in both nested `RetainedActionSettings` initializers, add `projects: List::with_capacity(0)`.
Do not restore the superseded `replaced_and_accepted_say_so` test.
`temper/tools/gallery/src/worlds.rs` merged automatically; preserve its combined changes: `proposal.reason`, retained `projects`, and the branch’s rejected/withdrawn state selection.
After resolving, format and run the focused card tests and relevant view/gallery checks through heavy. Compile slow-tier stories without running them.

## Rebase (2026-11-06, step merge, foreman ui-proposal-card/card-states/foreman-2)

merge both vocabularies in words.rs and regenerate the combined declarations
The conflict is in `temper/crates/temper-client-view/src/words.rs`, against integration commit `81358d9f1c` (`Merge main's conversation paging with proposal cards`).
Keep integration’s removal of `Older` and addition of `LoadingEarlierFailed` throughout the enum, `ALL`, code table and English phrases.
Keep this branch’s `SettingsProposalRejected` and `SettingsProposalWithdrawn` throughout those same tables, including their exact state-line text.
Their codes 3873 and 3874 remain free in integration; preserve them.
Regenerate declarations using `SKEIN_CODEGEN_WRITE=1` on `english_phrase_declarations_match_the_vocabulary` through heavy.
The combined declarations should be PHRASES 1865, SEGMENTS 3260, ENGLISH_BYTES 45516, and ALL length 1264.
Preserve the automatically merged changes in `pages/chat.rs` and gallery `tests.rs` and `worlds.rs`.
Format and run the focused vocabulary, card and gallery checks through heavy; check the selected test counts. Compile slow-tier stories without running them.
