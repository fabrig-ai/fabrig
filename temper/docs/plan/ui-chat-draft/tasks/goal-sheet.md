+++
id = "goal-sheet"
part = "goal-sheet"
kind = "feature"
needs = ["goal-sheet-state"]
files = [
  "temper/crates/temper-client-view/src/pages/chat_sheet.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/document.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/chat_sheet.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 85
+++

# The sheet, "Start goal…" and "Goal TITLE →"

## Goal

The chat's bar starts a goal and links to it afterwards, the sheet is the mock's, and the conversation says "Started goal" at its place.

Acceptance, temper's `client/README.md`, 3.5, *Starting a goal from a chat's document*, and *The bar*:

- **The bar:** `.btn.primary.small` "Start goal…", once the document has a section, it's a chat of a project with a home repository, `drafts_offered`, and no goal linked, `Act::SheetOpen`. With a linked goal, a link `.btn.small` "Goal TITLE →", TITLE from `goal_chip`, to `Task { task }`. Neither on a configuring chat.
- **The sheet,** `pages/chat_sheet.rs`, `Role::Dialog` named "Start a goal from NAME", `SHEET`, drawn as `confirm`'s dialogs are, at the end of `chat`: `h2` and a muted line; `.field` "Name" focused; fieldset "How it runs", `ChoiceGroup`, `.option` per `ways`, the first's small line ending " PROJECT’s default."; the muted link "The project’s ways of working" to `Settings { project }`; `.field` "Budget", `TEXT` 7em, beside it muted "asks you before going over"; `.foot` `.btn.quiet` "Cancel", with `Shortcut::ESCAPE`, `.btn.primary` "Start goal", disabled while the name is empty or one is out; under the form, `SHEET_ERROR`, jig's words, or a word for `TooLong`, or for each `NotOffered`. On a phone, the mock's width rule. No "When it needs you".
- **The way's words:** "Plan, tasks, land whole", "A plan from the draft, its tasks on a goal branch, landed as one change once all are checked."; "Task by task", "The same plan; each task lands on its own once checked."; "Plan, then wait for me", "Proposes the plan and its tasks, and starts nothing until you approve."
- **Above the composer:** `Chat.proposed`'s "Asked RECIPIENT to approve it; it waits in Needs you."
- **The line:** the fact's chat shows `.turn-tool`: pen, "Started goal", `TOOL_FILE` TITLE, its state, "Open", a link to its `Task`, placed among the turns by `at`: before the first line after it, and after the last if none. An agent's `goal` tool's line stays `ui-chats'`, and no row says it.
- **Gallery:** `chat-start-goal`, `chat-start-goal-refused`, `chat-goal-linked`.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The client*, 4 and 7.
- The mock's `startSheet`; `mock.css`, `dialog.sheet`, `.sheet form`, `h2`, `fieldset`, `legend`, `.foot`, `.option*`, `.field`, `.text`.
- `temper-client-view/src/forge/connect.rs`, the welcome's sheet, the same kind of thing, `WELCOME_SHEET`, `WELCOME_OPTION`; `confirm.rs`.

## Owns and may change

Owns, as `files` lists them. May also change `temper/crates/temper-client-shell-terminal/src/tests/**`, regenerated; `temper/docs/design/client/journeys/**`, prints your pages change.

Options, each pre-approved:

- **`.temper-welcome-sheet`:** its rules are scoped to `.temper-welcome`, so write the mock's under the marker, and share nothing with it.
- **`ui-goal-page`'s `look::SHEET` on your base:** a name your merge meets: reuse the token if its rules are the mock's; name it in the report.
- **`pages/chat.rs`' size:** the sheet and the document stay in their own files; only calls go there.
- **The terminal:** the dialog prints as `confirm.rs`' does.
- **"Started goal" while `tool-lines` isn't on the base:** as `chat-bar`'s option, the same names.

## Steps

1. Tokens and rules; `chat_sheet.rs`; the bar; the line.
2. **Tests,** `tests/chat_sheet.rs`: `start_goal_shows_when_it_may`, `none_on_a_configuring_chat`, `the_sheet_names_its_document`, `one_option_per_way_in_order`, `the_first_is_chosen_and_says_so`, `start_is_disabled_while_empty_or_out`, `cancel_carries_escape`, `the_refusal_is_jigs_words`, `a_linked_goal_shows_its_link`, `the_line_stands_by_its_time`. Each bites.
3. Render beside the mock's sheet, opened, at 1440, 1000 and 420.
4. `heavy cargo nextest run` of the client's packages; goldens; reprint.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.

## Answers

2026-11-05 (fabrig/ui). Ruling (rulings.md, 3): the sheet is ui-goal-page's start-sheet; here only "Start goal…" in the chat's bar opening it, and "Goal TITLE →" and its line.

## Rebase (2026-11-05, step merge, foreman ui-chat-draft/goal-sheet/foreman-1)

resolve words.rs by retaining the chat words and combining vocabulary counts
The merge conflicts in `temper/crates/temper-client-view/src/words.rs`.
Rebase onto the landing’s current integration base.
Main’s vocabulary corrections in `f0bbb909d1`, brought into integration by `29a48b5fe9`, overlap this branch’s count changes.
Preserve `ChatStartGoal` and `ChatGoalLink`, their English text, phrase IDs 3558/3559, and both `ALL` entries.
Preserve the incoming agent-check charter comment and vocabulary corrections.
For the inspected integration tip `d7d8dba129`, the combined declarations are `PHRASES = 1706`, `SEGMENTS = 2974`, and `ALL: [Word; 1107]`.
These combine integration’s 1704 phrases/2970 segments with this task’s two phrases/four segments.
Verify the merged counts with the existing `english_declarations()` assertion in `src/tests/accounts.rs`, then run the focused chat-sheet tests through heavy.
Keep the shared start-sheet implementation as required by the task’s ruling.
