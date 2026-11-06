+++
id = "pending-proposal"
part = "pending"
kind = "feature"
needs = []
files = ["temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/client/tests/suite/configuring_chat.rs", "temper/tests/client/tests/suite/adoption.rs", "temper/docs/help/proposal.md", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 60
+++

# What the words say a person and a run can do, they can

## Goal

The configuring agent will now tell a person that it can't withdraw or replace a pending proposal, that an owner rejects it under Needs you, and that the person adopts the same repository again on its page. Hold each of those in the lowest tier, reproducing the walk's refusal there, and say in `proposal.md` what's true.

Acceptance, each story passing and biting:

1. `a_pending_proposal_holds_the_next_until_an_owner_rejects_it`, in `temper/tests/engine/domain/tests/suite/configuring.rs`: a stock configuring run proposes. A second `propose_settings` answers an error holding `a proposal is pending (at most 1 at once)`. `withdraw` of the first answers an error, and the proposal still waits, its review unchanged, the head revision unchanged. An owner rejects it with a reason; the next run hears `rejected: REASON`, proposes afresh, and that one is admitted.
2. `no_answer_a_configuring_run_hears_says_nothing_changed`, in the same file: a run reads the inventory, a charter, writes a note, proposes and hears the outcome; no answer or message holds `nothing has changed`, `nothing changed` or `changes nothing`, case aside. It pins the plan's finding that the tools aren't the source.
3. `an_owner_rejects_a_chats_pending_proposal_under_needs_you`, in `temper/tests/client/tests/suite/configuring_chat.rs`: beside `a_configuring_proposal_of_a_projects_ways_reaches_its_needs_and_its_rail`, the owner's Needs you offers `Reject with reason` on a chat's settings proposal; pressing it with a reason ends the proposal, and the chat's task hears the reason.
4. `the_same_repository_adopted_again_from_its_page_replaces_its_record`, in `temper/tests/client/tests/suite/adoption.rs`: using `press_adoption`, an adopted repository is adopted again with the same fields from the page, the second under its own key. It's answered as adopted, and the project holds one record for it.
5. `temper/docs/help/proposal.md` is true (below), within its bound, and `heavy cargo nextest run -p temper-docs` passes.
6. `temper/docs/design/domain/agent.status.md` has lost the **Not yet** entry that names `pending-proposal`, and no other line.

## Inputs

- `temper/docs/plan/configuring-words-2/notes/common.md`, `temper/docs/plan/configuring-words-2/plan.md`
- `temper/docs/design/domain/agent.md`, 8.1, *Its tools*
- `jig/docs/design/domain/tasks.md`, section 9
- `temper/docs/help/repository.md`, *Adopted again*

## Owns and may change

- Owns the four stories and their helpers in those files, `proposal.md`, and the status edit: a departure from your text, which closes `docs/help/` and `docs/design/`.
- `configuring_chat.rs` is also written by `model-catalogue-1/configure-picker`: add your story at the end of its group and touch nothing else there.

**Options:**

- **Story 1 and `withdraw`'s words:** pin `error` and the state unchanged, not `withdraw`'s exact text, which is jig's. Jig's busy refusal says `or withdraw it`, which charter 3 can't: leave it, as jig is closed here, and name it in the report. The words tell the run otherwise.
- **If story 1 finds a second proposal admitted or `withdraw` succeeds,** the plan's reading is wrong: make the story hold what's true, and report `Done:` with it first under `## Left`, since the stock's sentence then needs changing.
- **If Needs you offers no Reject on a settings proposal:** the controls take their acts from the page's offers (`temper-client-view/src/confirm.rs`, `card_controls`). Fix it in `temper-client-view` if it's the view's. If it reaches jig, leave the story `#[ignore = "..."]` naming why, and put it under `## Left`.
- **If the page refuses the second adoption:** fix the form if the refusal is `temper-client-domain`'s. If it's the engine's, make the story hold what's true and put it under `## Left`.

## Steps

1. Run each story against today's code first: stories 1 to 4 are expected to pass, as they pin what the words rely on. Each one's bite is by hand, as your text says: for 1, give policy 1's authority the decide family and see `withdraw` succeed; for 2, add `nothing has changed` to `proposed as`'s words; for 3, drop `ActKind::Reject` from `ACTIONS`; for 4, refuse an adopted repository in the form. Restore each.
2. `proposal.md`, *Limits*. Read `temper-client-view/src/cards/proposal.rs`, `settings`, and `confirm.rs`, `card_controls`, and correct the bullet that says the client can reject a proposal of settings but not accept it, to what they do. Add:

   ```
   - A run can't replace a proposal of settings that waits, since a task holds one pending proposal, and the stock's configuring chat can't withdraw it, holding no decide family. An owner rejects it, with a reason the chat hears, and the chat proposes afresh.
   ```

   If that passes the page's bound, shorten the page, losing no fact, and never raise the bound.
3. `cargo fmt`, clippy for `temper-engine-domain-world`, `temper-client-world` and any crate you fixed; the named stories; `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/configuring/)'` and the client world's `adoption` and `configuring_chat` groups. Commit and push.

## Report

Your text's report, with what `withdraw` and the second `propose_settings` answered, word for word: that's what the plan's finding rests on.
