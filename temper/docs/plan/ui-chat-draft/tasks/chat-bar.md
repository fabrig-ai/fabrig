+++
id = "chat-bar"
part = "chat-bar"
kind = "feature"
needs = ["start-over"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/document.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/document.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 90
+++

# Every chat's group, tool line and Show

## Goal

The pane opens to every chat, and the conversation says what the agent wrote: the pen, "Wrote" or "Edited", NAME, a count, and "Show", which goes to that section.

Acceptance, temper's `client/README.md`, 3.5, *A chat's document*: *The tool line*, *The bar*, *The pane*; as `notes/contracts.md`. Every commit carries `Look-request: configure-chat-draft`, whose states these are.

- **The group on any chat,** after the byline (`chat-byline`'s, if your base holds it), once a document exists. The composer's hint gains " · Ctrl+\ switches the draft" while one shows: `session::hint` takes it.
- **The pane's line's left** on a chat that isn't configuring: `docs/design/draft/NAME.md`, NAME `document::slug(title)` and `.md`.
- **The tool line.** A `note` call whose outcome `note_write` parses is a `.turn-tool`, where `committed` draws a call, and `other_call` and `folded_calls` leave it out of the "N tool calls" count, name and content: pen; "Wrote" for a `First`, else "Edited"; `TOOL_FILE` NAME; a `.turn-tool` detail, "64 lines" or "+18 −6"; "Show", a button, `Act::ShowSection { line }`: `First`'s line the first `##` section's, `Later`'s its `first`. A failed call, or one whose words don't parse, stays in the disclosure as today. NAME is the document's, or `untitled.md` before it's read.
- **Show:** with `chat.shown`, the section holding that line takes `DOC_FLASH`, whose rule runs `@keyframes flash` once, and `b.focus(chat.shown_count)`, which brings it into view, on the mock's rule. Pressing Show again moves the count, and the flash runs again: keyed by count in the section's flash, by whatever makes the animation restart.
- **A turn cut only by a note's input** draws no `TurnCut` words: the transcript cuts a `note` call's whole body as a matter of course, and a tool line says what happened instead. Any other cut stays.
- **The terminal:** the tool line prints in full, "Show" a numbered button. It prints in the conversation, in any mode.
- **Gallery:** `chat-draft-chat`, `chat-draft-split` (a section tinted, the strip), `chat-draft-draft`, with `HOLDS` and `SPENDS` rows.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The client*, 3, 4 and 5.
- `temper/docs/design/client/greenfield/app.js`, `chatPage`, its `turn`; `mock.css`, `.turn-tool*`, `.doc-sec.flash`, `@keyframes flash`.
- `temper/docs/plan/ui-chats/notes/contracts.md`, `TURN_TOOL`, `TOOL_FILE`, `DelegationOpen`, the pen.

## Owns and may change

Owns, as `files` lists them. May also change `pages/configure.rs`, `pages/new_chat.rs`, for `session::hint`'s callers; `temper/docs/design/client/journeys/**`, prints your pages change.

Options, each pre-approved:

- **`ui-chats`' `tool-lines` isn't on your base** (`chats-base`' report, and `rg TURN_TOOL`): add `TURN_TOOL`, `TOOL_FILE` and `pen.svg` yourself, exactly as `temper/docs/plan/ui-chats/notes/contracts.md` and its `tool-lines.md` spec name them, so that the merge meets the same names, and say so in the report. The same for `chat-byline`: put the group where the byline's line stands, in `SESSION_RIGHT`.
- **The composer's focus:** `b.focus` is one per tree. A chat's composer asks for none. If yours does by then, the section's request wins while `shown` moved.
- **A long body and `TurnCut`:** if a turn is cut for a further reason beside a `note`'s input and that can't be told apart, keep the words.
- **Named "The draft":** in the terminal the group prints as buttons with those names, "Chat", "Split", "Draft".

## Steps

1. The tool line and its count, then the group, the hint, the line.
2. **Tests,** `tests/document.rs` and `tests/chat.rs`: `a_first_write_says_wrote`, `a_later_write_says_edited_and_counts`, `a_note_call_isnt_counted`, `a_failed_note_stays_in_the_disclosure`, `a_recall_stays_in_the_disclosure`, `show_presses_its_line`, `show_flashes_and_focuses_the_section`, `the_hint_names_the_key_only_with_a_document`, `no_document_no_group_on_a_chat`, `a_note_turn_draws_no_cut_words`, each having bitten.
3. Render beside `chat.html?id=T4` at 1440, 1000 and 420, as `notes/common.md` says.
4. `heavy cargo nextest run` of the client's packages; goldens; reprint.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
