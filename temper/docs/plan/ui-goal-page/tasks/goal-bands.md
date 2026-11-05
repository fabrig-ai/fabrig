+++
id = "goal-bands"
part = "goal-bands"
kind = "feature"
needs = ["goal-side"]
files = [
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/ask.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
]
effort = "high"
minutes = 60
+++

# A goal's needs, as bands

## Goal

One `section.band` per open need of the goal, between the header and `.two`, answering as Needs you does, as
`temper/docs/design/client/README.md`, 5.8, says.

Acceptance:

- Each band: `Role::Region` named "Needs you", `look::GOAL_BAND`; `<p>` with the tag "Needs you · KIND" (`GOAL_BAND_TAG`)
  and QUESTION, from `ask::read`.
- Parsed choices in `.choices` (`GOAL_CHOICES`): one button per choice, `Act::Needs(needs::Act::Choose { object, words })`,
  the recommended one `GOAL_PRIMARY` with its `.rec`, no Reply or Later.
- No choices: `.btn.primary` "Answer" for a question or a person task, "Open" otherwise, to `Address::Inbox { filter:
  Filter { project, goal: Some(task), kind: None } }`.
- A funds card: `confirm::needs_controls` while Release or Leave held is offered, else "Open"; no "Add $10", no "Stop the goal".
- In the inbox's order (`needs::Session::open`). Answered, it leaves, as on Needs you; its toast is the frame's.
- The state follows: `goals::state`'s `needs` is whether any band stands.
- The tests below pass, each having bitten.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.8; 3.5, *Needs you*.
- The mock's `goalPage`, `choiceButtons`.
- `pages/ask.rs`, `pages/needs.rs`'s `card_main`, `confirm.rs`'s `needs_controls`.

## Owns and may change

Owns, as `files` lists them. `pages/ask.rs`: `pub(crate)` or a small shared helper, nothing it draws changes.

Options, each pre-approved:

- **`.rec`:** "recommended", as `pages/needs.rs` draws it. If it's private there, share it.
- **KIND:** `ask::need_kind`'s word in lower case where it isn't already, or `Word::NeedsKindFunds`.
- **A question whose card is still loading** (`card.page` none): draws no band until it has read.
- The band's `Role::Region` is named once; several bands share the name, as the request says.

## Steps

1. The bands.
2. **Tests,** `tests/goal.rs`, with the page's inbox holding cards (as `tests/needs.rs` does):
   - `one_band_per_open_need_in_the_inboxs_order`;
   - `a_questions_choices_answer_as_needs_you_does`, by pressing one and seeing the session's answered line;
   - `no_reply_and_no_later_on_a_band`;
   - `no_choices_says_answer_or_open`, by kind, and its link's address;
   - `a_funds_band_offers_release_and_leave_held`, and "Open" with neither offered;
   - `a_need_makes_the_state_needs`, "needs funds" for funds.
3. `heavy cargo nextest run -p temper-client-view --lib -E 'test(goal)'`, then the crate whole.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
