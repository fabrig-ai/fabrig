+++
id = "chat-decides"
part = "decide"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/acts.rs",
  "temper/crates/temper-client-domain/src/tests/acts.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/tests/client/tests/suite/chat_accept.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 40
+++

# A proposal's decision goes under the proposal's own project

## Goal

Accepting a settings proposal on its card in the chat that made it is refused, "No visible object with that name", and the sheet stays open. The cause, read and not yet run: `acts.rs`, `send`, decides under `card.project`; `pages/chat.rs`, `proposals`, sets that from `task.project`, which is 1; `jig/crates/jig-domain-core/src/routing/party/decisions.rs`, `task_settings_decide`, refuses any project but the deployment's, 0. A card in Needs you holds `proposal.project`, 0, and works. Show it red, then fix it.

Acceptance:

- `send` decides a proposal under the `project` of the proposal it read, for Accept and for Reject, from any card that holds it: chat, task, Needs you, Home.
- A domain test and a joined-world test were each seen red for this cause before the fix, and pass after.
- From the chat, in the joined world: the card's Accept and the sheet's primary close the sheet; the card says "✓ Accepted · new work follows it"; the rail's count for the project is 0; the settings' newest revision is 2; the rail has Goals; a message sent afterwards is taken.
- From the chat, the Reject act with a reason closes the sheet, clears the count and leaves the revision at 1.
- If the world shows another fault behind this one (the card not updating, the count not clearing), it is fixed here if it is in the files you own, and named in the report.
- The **Not yet** entry `5.13, *The card in a chat*: a decision on a chat's card goes under the chat's project` is gone from `README.status.md`.

## Inputs

- `temper/docs/design/client/README.md`, 5.13, *The card in a chat*; 3.6, *The sheet*.
- `temper/docs/plan/ui-chat-accept/notes/common.md`.
- `temper/docs/plan/ui-chat-accept/plan.md`, *The cause*.

## Owns and may change

Owns: `temper-client-domain`'s `src/acts.rs` and `src/tests/acts.rs`; the new `temper/tests/client/tests/suite/chat_accept.rs`.

May also change:

- `temper/tests/client/tests/suite/main.rs`: the `mod chat_accept;` line, between `chat` and `chat_draft`.
- `temper/crates/temper-client-domain/src/pages/chat.rs`: only for a fault the world shows behind this one.
- `temper/docs/design/client/README.status.md`: remove the one entry above. This departs from the implementer's text, which closes `docs/design/`: the fix closes this entry, so the task does.

Stay out of `temper/tests/client/tests/suite/configuring_chat.rs` but for making a private helper `pub(super)`, one word: another plan is changing it. It already gives `pub(super) fn propose`.

Options, each pre-approved:

- If `card.project` is `None` for a proposal: decide under the proposal's project all the same.
- If the chat's composer stays closed after acceptance because the world's scripted run hasn't finished its turn: finish it as `chat.rs` does (`fixtures::finish_replied_chat`). The test's point is that the message isn't refused.
- The Reject act has no control on a chat's card (5.13, and the mock has none): the world sends `Act::CardWords` and `Act::Decide` as `a_configuring_proposal_of_a_projects_ways_reaches_its_needs_and_its_rail` sends its Accept. Draw no Reject. After a rejection the card shows neither acts nor a state line: leave that, and name it in the report.

## Steps

1. **Red, the domain.** In `src/tests/acts.rs`, beside `settings_rejection_sends_its_reason_without_a_running_choice`, with its fixtures:

   ```rust
   #[test]
   fn a_proposal_is_decided_under_its_own_project() {
       // For Accept and Reject: a settings proposal whose `project` is 0, on a card
       // whose `project` is `fixtures::PROJECT`, sends `c::Ask::Decide { project: 0, .. }`.
   }
   ```

2. **Red, the joined world.** `temper/tests/client/tests/suite/chat_accept.rs`, whose module line cites `temper/docs/design/client/README.md`, 5.13:

   ```rust
   #[test]
   fn a_configuring_chat_accepts_its_own_proposal() { /* the kind 7 item of
       a_configuring_proposal_of_a_projects_ways_reaches_its_needs_and_its_rail, through
       super::configuring_chat::propose; world.client(fixtures::PARTY, Address::Chat { task });
       a person's Step::Press on the card's "Accept", then the sheet's primary, as
       chat.rs's refusal test presses within its dialog. Run it now: it sees
       "No visible object with that name" and the sheet stays; name that in the report. */ }
   #[test]
   fn a_configuring_chat_rejects_its_own_proposal() { /* Act::CardWords, Act::Decide, Act::Confirm */ }
   ```

3. **The fix,** in `acts.rs`, `send`, in the subset's form, with a line saying why:

   ```rust
   // A proposal is decided under its own project, which a chat's card doesn't
   // hold (temper/docs/design/client/README.md, 5.13).
   let project = if let ReadValue::Proposal(proposal) = page { proposal.project } else { card.project? };
   ```

4. Then the rest of the acceptance's assertions, the status entry, and a bite for each new test (undo the fix: both red).
5. **A change of meaning runs its readers' tests:** `send` serves every card. `cargo fmt`; `heavy cargo clippy -p temper-client-domain -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-world`, and `-p temper-client-view`. `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
6. Commit, saying why, and push.

## Report

As the implementer's text says, and: the red each test showed, by its words; any fault found behind this one; whether a later fault in the path belongs to `finding-400`'s account (`notes/common.md`).
