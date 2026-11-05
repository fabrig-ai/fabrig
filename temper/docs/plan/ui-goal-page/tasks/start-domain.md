+++
id = "start-domain"
part = "start-domain"
kind = "feature"
needs = ["goal-acts"]
files = [
  "temper/crates/temper-client-domain/src/pages/start_goal.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/pages/start_goal.rs",
  "temper/crates/temper-client-domain/src/tests/pages.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
]
effort = "high"
minutes = 75
+++

# Starting a goal, in the domain

## Goal

The sheet's state and its three ends, as `temper/docs/design/client/README.md`, 5.14, says. This task draws no sheet.

Acceptance:

- `pages/start_goal.rs`, the `Act`s and `Pages.start`, as `temper/docs/plan/ui-goal-page/notes/contracts.md`, Start goal, gives them.
- **`Act::OpenStart`** opens it for the page's project, its name FILE without `.md`, its way `default_way`. **`StartCancel`** closes it. Any page can open it; going to
  another address closes it.
- **`Act::StartGoal`,** unless Name or Draft is empty or a request is out: `shapes::set_goal_from(me, project, way, GoalDraft { words: words(..), executor, budget,
  deadline: None, priority: None, llm: None }, "Started from its draft")`; `executor` the template's `TemplateExecutor::Charter(n)` as `Executor::Agent { charter: n }`,
  else `Agent { charter: 1 }`; `budget`, the creating bound's `within.budget`.
- **Its ends:** `Judged::Create`: sent, and on `DocumentOutcome::Created { first, .. }`, `Request::Show` the goal's `Address::Task`, closing the sheet. `Judged::Propose`: sent, and on
  its done answer closes, `Toasted::Proposed { to }`. `Judged::Refused`, `NotOffered`, a refused or lost answer: `error` set, the sheet open.
- `Toasted::Proposed`: its view arm in `frame.rs`, "Proposed to WHOM: it starts once accepted.", whom as a card's "This needs a proposal to" says.
- The tests below pass, each having bitten.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.14, 3.6.
- `jig/crates/jig-client/src/shapes.rs`, `set_goal_from`, its tests in `src/tests/shapes.rs`.
- `temper-client-domain/src/pages/chats.rs`: `shapes::chat`'s send and how its `Created` answer moves the page.

## Owns and may change

Owns, as `files` lists them. `lib.rs`: a re-export if the view needs one.

Options, each pre-approved:

- **The draft's path, typed:** taken as written, trimmed. A path that doesn't end in `.md` is taken too: the goal reads what it's told.
- **Name following Draft:** until `StartName` has changed it, a `StartDraft` sets it to FILE without `.md`.
- **Bounds:** a name or draft past the document's text bound (`env.limits.client.domain.text`) refuses as `Text::new` does: the words aren't sent, and `error` says it in the shape
  `NotOffered` or a refusal gives; if neither fits, keep a `Failure::TooLong`, for the view to word.
- **A missing `me`:** `OpenStart` is a no-op until `me` is read.

## Steps

1. The module, `Act` arms, the send, the answers.
2. **Tests,** `tests/pages/start_goal.rs`, on `me`s with bounds that create, propose and refuse: `create_sends_and_goes_to_its_goal`, `propose_sends_closes_and_toasts`, `refused_stays_open_with_jigs_words`, `not_offered_stays_open`,
   `a_refused_answer_stays_open`, `start_needs_name_and_draft`, `a_double_press_joins_the_pending_request`, `the_words_read_back_through_draft_of`, with and without a chat, `the_default_way_is_the_highest_numbered`, `cancel_closes`,
   `name_follows_draft_until_edited`.
3. `heavy cargo nextest run -p temper-client-domain`; `heavy cargo check --tests -p temper-client-view`.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
