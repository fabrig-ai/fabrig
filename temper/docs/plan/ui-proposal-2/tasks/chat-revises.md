+++
id = "chat-revises"
part = "chat"
kind = "feature"
needs = ["replace-own"]
files = [
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-engine-smith/src/answers.rs",
  "temper/crates/temper-engine-smith/src/tests.rs",
  "temper/tests/engine/domain/tests/suite/configuring.rs",
  "temper/tests/client/tests/suite/configuring_chat.rs",
  "temper/docs/help/proposal.md",
  "temper/docs/design/domain/agent.status.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "high"
minutes = 60
+++

# "Change something" revises: the chat's words, its refusal, the stories and the help

## Goal

`replace-own` lets a chat's fresh `propose_settings` replace its own. Teach the configuring chat to use it, stop it sending a person to Needs you, and prove the person's whole path through the real engine: press "Change something", say what should change, the new card stands below and the old reads "Replaced by the proposal below".

Acceptance:

- `configuring.txt` holds the sentences of `notes/contracts.md`, *The configuring chat's words*, and no longer says "You can't withdraw or replace it" or sends a person to Needs you to revise.
- `temper-engine-smith/src/answers.rs`'s busy refusal ends `wait for its decision`, and `temper-engine-smith/src/tests.rs` reads it so.
- `temper/tests/engine/domain/tests/suite/configuring.rs`: `a_pending_proposal_holds_the_next_until_an_owner_rejects_it` becomes `a_chats_fresh_proposal_replaces_its_own_that_waits`, which holds: the fresh proposal is answered and names a new number; `withdraw` is still refused to the chat by its authority; the first's review reads withdrawn; the head hasn't moved; deciding the first is refused and changes nothing (jig answers `Refused(Unknown)`; `world.decide` may need a small refused form); accepting the fresh one makes revision 2; the run hears no withdrawn message for the first and hears `accepted` for the fresh one. A second story, `a_chats_refused_fresh_proposal_leaves_its_own_waiting`, has a stale or out-of-shape fresh one answered with an error, the first still pending and its review unchanged.
- `temper/tests/client/tests/suite/configuring_chat.rs`, a new `change_something_replaces_the_card_below`, in the joined world: the person presses "Change something" and sends words; `world.front().host_tool(.., engine::Tool::ProposeSettings { .. })` makes the fresh one, as `a_second_proposal_stands_below_the_first_and_a_reply_below_it` does; the person sees "Replaced by the proposal below", exactly one "Accept", and the new card after the old in the tree's order. An Accept sent on the old one by its act, as a stale tab's press, leaves `settings show` at `newest revision 1`, and the card still says it was replaced.
- `temper/docs/help/proposal.md`: *Limits* no longer says rejecting under Needs you "currently fails" or to use `temper ctl` for it (`an_owner_rejects_a_chats_pending_proposal_under_needs_you` runs and passes today), and says a run's fresh proposal of settings replaces its own that waits, in one commit, a refused one leaving it standing. *How it changes* names that way of being withdrawn. The stock chat still holds no decide family: keep that fact.
- These status entries are gone or corrected:
  - `temper/docs/design/domain/agent.status.md`: the **Not yet** naming `configuring.txt`, the busy words and Settings, Accounts. The **Not yet** "no story that runs has an owner reject a chat's proposal": that story has no `ignore` and runs, so it goes. In the first **Not yet** of 8.1, `configuring_txt_says_how_a_pending_proposal_goes` stays named if the test keeps its name.
  - `temper/docs/design/client/README.status.md`: the **Not yet** on "Change something".
  - `temper/docs/design/STATUS.md`, its sentence on `temper help proposal` and the refusal offering a `withdraw`.

## Inputs

- `temper/docs/plan/ui-proposal-2/inputs/request.md`, item 1 and item 4's first and third bullets.
- `temper/docs/plan/ui-proposal-2/notes/contracts.md`, *The busy refusal's words* and *The configuring chat's words*: binding, word for word.
- `temper/docs/plan/ui-proposal-2/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 8.1, *Its tools*, and the inventory paragraph, on a model no account can run.
- `temper/docs/design/client/README.md`, 5.13, *The card in a chat*.
- `jig/docs/design/domain/tasks.md`, section 9.

## Owns and may change

Owns the files in the header. `temper/docs/help/proposal.md` and the three status files are yours here: departures from the implementer's text, on purpose.

In `temper/tests/client/tests/suite/configuring_chat.rs`, `card-and-latest` edits `a_configuring_proposal_of_a_projects_ways_reaches_its_needs_and_its_rail`. Add your story after `a_second_proposal_stands_below_the_first_and_a_reply_below_it`, and leave that function alone.

Readers, as found by survey:

- `temper/crates/temper-service/src/tests/stock.rs`: `configuring_txt_says_how_a_pending_proposal_goes` reads the old sentences. Change it to the new ones and add a phrase test for the model line, in the file's style. `configuring_txt_starts_*` and the others stay.
- `temper/tests/engine/domain/tests/suite/configuring/fixture.rs`'s `propose` helper and `world.review`, `world.decide`, `world.next`: use them; a small helper there is allowed.
- `temper/docs/help/proposal.md` is read by tests (`temper/docs/help/**` in `.config/gate/read-by-tests.txt`): run `temper-docs`' tests and `temper`'s help test. It has room (5,116 of 12,288 bytes).
- `temper/docs/design/draft/configuring.status.md` and the design documents are already written; don't change them unless a citation breaks.

Options, each decided:

- **`configuring.txt` has moved under you** (`model-catalogue-2` is changing it): keep its other changes, and make only this task's edits.
- **A byte bound on the stock's brief trips:** shorten this plan's sentences, keeping their facts, and never raise the bound.
- **The domain drops the stale Accept** because the page already knows the proposal ended: that passes. The story still proves the head hasn't moved and the card's words, and the report says which happened. `replace-own`'s `an_owners_decision_of_a_replaced_proposal_changes_nothing` holds the refusal at jig's level.
- **A scripted cue in `temper-fake` breaks on the new words:** repair it, and name it.

## Steps

1. The words: `configuring.txt`, `answers.rs`, their tests.
2. The engine stories, then the joined world's.
3. The help page and the status entries.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service -p temper-engine-smith -p temper-engine-domain-world -p temper-client-world --all-targets -- -D warnings`.
5. `heavy cargo nextest run -p temper-service -p temper-engine-smith -p temper-engine-domain-world -p temper-docs`, `heavy cargo nextest run -p temper-client-world -E 'test(configuring) or test(chat_accept)'`, and `heavy cargo nextest run -p temper --lib -E 'test(help)'`.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
7. Commit and push.

Bite each new test once by hand, as the implementer's text says. No browser, walk or render check.

## Report

What ran and how many; the rule each new test bit on; whether the stale Accept reached jig; the sentences as they landed; your implementation and check minutes apart.
