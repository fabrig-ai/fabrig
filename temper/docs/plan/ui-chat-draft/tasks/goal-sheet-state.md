+++
id = "goal-sheet-state"
part = "goal-sheet-state"
kind = "feature"
needs = ["add-draft", "person-facts", "person-words"]
files = [
  "temper/crates/temper-client-domain/src/pages/chat/sheet.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/forge/dismissed.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat_sheet.rs",
  "temper/crates/temper-client-domain/src/tests/dismissed.rs",
  "temper/crates/temper-engine-domain/src/route/goal.rs",
  "temper/crates/temper-engine-domain-forge/src/dismissed.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/dismissed.rs",
  "temper/crates/temper-protocol-store/schema/store-v11.schema",
  "temper/crates/temper-protocol-store/src/generated/v11.rs",
  "temper/crates/temper-protocol-store/src/keys.rs",
  "temper/crates/temper-protocol-store/src/tests/**",
  "temper/crates/temper-protocol-store/golden/**",
  "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema",
  "temper/crates/temper-client-protocol-wire/src/generated/v1.rs",
  "temper/crates/temper-client-protocol-wire/src/lib.rs",
  "temper/crates/temper-client-protocol-wire/src/tests.rs",
  "temper/crates/temper-client-protocol-wire/golden/**",
]
effort = "high"
minutes = 85
+++

# Start a goal from a chat's document, in the domain

## Goal

The sheet's state, what it sends and its three ends. This task draws no sheet.

Acceptance, temper's `client/README.md`, 3.5, *Starting a goal from a chat's document*, as `notes/contracts.md`, *A chat's sheet*, *A goal's lines* and its Revision 1 of *A person's facts*:

- **The lines:** `GOAL_SCOPE`, `GOAL_AGREED` move, public, to `temper-client-protocol-wire`, and `route/goal.rs` reads them.
- **`GoalFrom.at`,** Revision 1, through the wire, the store and the domains; the client's `goals_from` gives `(goal, at)`. `temper/docs/design/domain/forge.status.md`'s line on `at` goes.
- `pages/chat/sheet.rs`: `Sheet`, `Refused`, `Way`, `WayKind`, `ways`, and `words(name, slug, body, kind) -> Box<[u8]>`, exactly:

  ```
  NAME

  Implement this draft. First commit it, as it stands, at docs/design/draft/SLUG.md.

  BODY

  GOAL_SCOPE
  ```

  and for `WayKind::Agreed`, `GOAL_AGREED` on a line after `GOAL_SCOPE`.
- **`Act::SheetOpen`,** once the document has a section, on a chat with a home repository (`pages::home::repository`) and no linked goal: `name` the slug, `way` 0, `budget` the decimal of `creating.within.budget`. **`SheetName`, `SheetWay`, `SheetBudget`, `SheetCancel`.**
- **`Act::SheetStart`,** unless the name is empty, one is out or the budget doesn't read as a number: `Text::new(words, long_text)`, `TooLong` in place if over; `shapes::set_goal_from(me, project, way.template, GoalDraft { words, executor, budget, deadline: None, priority: None, llm: None }, reason)` or `set_agreed_goal_from` for `Agreed`; `executor` the template's `TemplateExecutor::Charter(n)` as `Executor::Agent { charter: n }`, else `Agent { charter: 1 }`; `reason` "Started from its draft".
- **Its ends:** `Judged::Create`: sent; on `DocumentOutcome::Created { first, .. }`, `Dismiss { what: GoalFrom { project, chat, goal: first, at: now } }` is sent and `Request::Show` opens `Task { task: first, .. }`, the sheet closed. `Judged::Propose { to, .. }`: sent; on its done answer closes, and `Chat.proposed: Option<Recipient>` holds the words above the composer. `Judged::Refused`, `NotOffered`, a refused or lost answer: `refusal`, the sheet open and its words kept.
- `temper-client-domain`'s readers: the chat's `Chat::goal_from(&self, root) -> Option<(u64, u64)>` from the forge side's facts, the newest; a chip watch, `Subject::Chips { tasks: [goal] }`, once a fact names a goal, for its title and state, and `Chat::goal_chip`.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The client*, 7.
- The mock's `startSheet`, `startGoal`.
- `jig/crates/jig-client/src/shapes.rs`; `temper-engine-domain/src/route/goal.rs`; `temper-client-domain/src/pages/chats.rs`' `send`, for how a `Created` answer moves a page.
- `temper/docs/design/domain/agent.md`, 4.3.2 and 8.2.

## Owns and may change

Owns, as `files` lists them. May also change, for `GoalFrom`'s new field, what `person-facts` named: `temper/tests/codecs/tests/suite/fuzzy/store.rs`, `temper/tests/client/**`, `temper/crates/temper-service/src/family.rs`, `temper/crates/temper-engine-domain-forge/src/{domain,boundary}.rs`, and `temper/crates/temper-client-view/src/tests/fixtures.rs`. Regenerate by `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`.

Options, each pre-approved:

- **`ui-goal-page`'s `pages/start_goal.rs`, if your base has it:** this is the chat's sheet: it's its own, and no field of it moves. Say so; a later plan may fold the two together.
- **The chat's project's budget read:** with no `creating`, `SheetOpen` is a no-op: "Start goal…" is the view's to hide.
- **`Judged::Create` with a template's `Chosen`:** `executor` is charter 1, as the design says.
- **A `Dismiss` lost or `TooLarge`:** nothing shows, as `person-facts` made it: the goal has started.
- **`Chips` watch and a page's window:** one watch, the chat's newest goal, closed in `leave`. A handle next to `draft_read`, `next = first + 4`.

## Steps

1. The lines, `at`, regenerated.
2. `sheet.rs`, with tests in `tests/pages/chat_sheet.rs`: `words_are_exact`, `agreed_adds_its_line`, `create_sends_the_fact_and_opens_the_goal`, `propose_closes_with_the_line`, `refused_stays_with_jigs_words`, `not_offered_stays`, `over_long_text_is_refused_in_place`, `a_double_press_joins_the_pending_request`, `the_ways_in_order`, `the_default_budget`, `cancel_closes`, `a_budget_past_a_number_doesnt_send`. Each bites.
3. A story, `temper/tests/client/tests/suite/chat_draft.rs`: `a_goal_starts_from_a_document`: the words read back in the goal's spec, the fact, and the goal's page.
4. `heavy cargo nextest run -p temper-client-domain -p temper-engine-domain -p temper-engine-domain-forge -p temper-protocol-store -p temper-client-protocol-wire -p temper-codecs-world`; the client world's `chat_draft`, `dismissed` and `task` suites; `heavy cargo check --tests -p temper-client-view -p temper-client-gallery`.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.

## Answers

2026-11-05 (fabrig/ui). Ruling (rulings.md, 3 to 8): one Start goal sheet, ui-goal-page's start-domain and start-sheet; don't build a second. Take it from main or origin/ui-goal-page/integration, and only add opening it from a chat with the draft filled in and the Draft: and Chat: lines. Ways come from goals::ways; default is the setting, else first. drafts_offered reads the drafts setting.

2026-11-05 (fabrig/ui). Before starting: git fetch; if ui-goal-page's start-sheet is on origin/main, use it. Otherwise merge origin/ui-goal-page/integration if start-sheet is landed there. If it is in neither, end with Question: saying so and change nothing; fabrig/ui will answer when it's in.

## Rebase (2026-11-05, step merge, foreman ui-chat-draft/goal-sheet-state/foreman-1)

resolve words.rs using integration’s reconciled phrase IDs and bounds
The merge conflicts in `temper/crates/temper-client-view/src/words.rs`.
Branch merge `41a77e81d9` and integration merge `86fc4a8a15` independently combined the same parents with different resolutions.
Rebase onto the landing trunk, preserving `840e47a4ac`’s shared-sheet implementation.
For `words.rs`, retain integration’s version from `86fc4a8a1545d87c8ed695369e3a8a8e7f8f8efe`; it contains the same variants and English text.
Keep its disjoint draft phrase IDs: `DraftGroup` through `DraftSendHint` use 3600–3621; `PlusDraft` through `DraftName` use 3622–3627.
Keep `PHRASES = 1690`, `SEGMENTS = 2938`, `ALL` length 1093, and `text_bytes = 42785`; the branch’s expanded byte expression equals that value.
Validate with the focused `temper-client-view` library test `tests::frame::the_browser_words_fit_the_views_limits` through heavy, then the spec’s view/gallery checks.
The refusal reports a merge conflict; it provides no test results.
