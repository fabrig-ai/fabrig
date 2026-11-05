+++
id = "add-draft"
part = "add-draft"
kind = "feature"
needs = ["chat-bar"]
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/crates/temper-client-view/src/pages/new_chat.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/document.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/new_chat.rs",
  "temper/crates/temper-client-view/src/tests/document.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 90
+++

# + Draft

## Goal

"+ Draft" on a chat, or on a new chat at `/chats/new?draft=1`, opens the empty document in Split, and the agent is told to write it.

Acceptance, temper's `client/README.md`, 3.5, *A new chat*, *With a draft*, and *A chat's document*: *The bar*, *Empty*, *"+ Draft" on a chat*; `browser.md`, 2.3:

- **`Address::NewChat { draft: bool }`,** printed `/chats/new` and `/chats/new?draft=1`, read back; another query is dropped, as today. A move between `Chats` and `NewChat`, or between `draft` false and true, keeps the page's state. `Address::NewChat` is a change of a shared item's signature: the survey's readers are listed below.
- **On a chat:** with no document and `drafts_offered`, and the chat not ended: `.btn.small` "+ Draft", `Act::AddDraft`: `writing` true, `mode` Split, and the chat's `Ask::Message` `WRITE_THIS_UP`, sent as `Send` sends it, with its pending state. A refused send leaves the pane and says jig's words under the composer, as a message's refusal does.
- **On a new chat:** "+ Draft" in the bar moves to `NewChat { draft: true }`, replacing the address; while `drafts_offered`. There: the empty pane in Split, `.meta-line` "A chat that writes a draft. Say what it should hold.", placeholder and name "What should this draft hold?". The first message starts the chat with, under the person's words after a blank line, `WRITE_THIS_UP`, within `CUE_BYTES` as words are. The chat opens in Split, with `writing` true. A suggested request, in the playground's start, is its own and takes no draft.
- **Empty:** `pane` with `writing` and no `draft`: `DOC_EMPTY` "The draft appears here as you write it together.", the line's left `untitled.md`, its right "writing".
- **The meta line,** a chat with an empty thread and a document or `writing`: "A chat that writes NAME. Say what it should hold."
- **Gallery:** `chat-draft-empty`, `chat-new-draft`.
- `browser.status.md`'s entry for `?draft=1` goes, its other half, on `?state=drafts`, staying.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The client*, 6.
- The mock's `addDraft`, `chatPage` and `sendChat`.
- `temper-client-domain/src/pages/chats.rs`, `start`, `send`, `show_chat`.

## Owns and may change

Owns, as `files` lists them.

May also change, each only for `Address::NewChat`'s new field (`NewChat { .. }` in a pattern, `NewChat { draft: false }` in a value, unless it means a draft):

- `temper-client-domain/src/{departure,tests/**}`, `pages/switcher.rs`, `pages/connect_model.rs`.
- `temper-client-view/src/{root,frame,choice}.rs`, `pages/{chats,connect_model,home,inbox}.rs`, `tests/**`.
- `temper/tests/one-process/src/{world,stories,stories/playground}.rs`, `temper/tests/playground/src/stories.rs`, `temper/crates/temper/tests/support/person.rs`, `temper/crates/temper-client-shell-terminal/src/tests/fixtures/chat.rs`.
- `temper/docs/design/client/browser.status.md`.

Options, each pre-approved:

- **A chat's mode across `Chats::start`:** `Pages` holds the opening mode for the next `Chat::new`, set by the start and taken by the chat's page, or a field of the start's: yours. `writing` follows it.
- **A chat's "+ Draft" with `WRITE_THIS_UP` already sent,** the agent not having written yet: pressed once, it's gone: `writing` shows the pane, and no further button draws.
- **The terminal:** "+ Draft" prints as any button; `NewChat { draft: true }`'s pane prints after the composer's conversation as Split says; a golden changes only if one holds these.

## Steps

1. The address, its readers; `heavy cargo check -p temper-client-view --tests`.
2. The domain: `add_draft_sends_and_goes_to_split`, `a_refused_add_keeps_the_pane_and_its_words`, `the_first_message_carries_the_words`, `the_chat_opens_in_split`, `a_new_chat_with_no_draft_opens_in_chat`, `the_address_round_trips`, `moving_to_a_draft_keeps_the_words`.
3. The view: `plus_draft_shows_with_a_tracked_template`, `none_on_an_ended_chat`, `none_with_a_document`, `the_empty_pane`, `the_placeholder_and_the_meta_line`, each having bitten.
4. `heavy cargo nextest run` of the client's packages; goldens; reprint; `heavy cargo check --tests -p temper-one-process-world`, and `temper`'s tests' support, by `Cargo.toml`'s name.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.

## Rebase (2026-11-05, step merge, foreman ui-chat-draft/add-draft/foreman-1)

rebase + Draft onto the integration’s Settings changes, preserving both features and regenerating prints.
The merge failed on eight conflicting paths; no test failure is reported.
Rebase onto the updated integration containing `32e0a3bc1f` (merged by `d59768cc67`), which reconciles main’s Settings work.

- `temper/crates/temper-client-domain/src/tests/pages/chat.rs`: retain both the appended Settings acceptance-time test and `draft_ready`, `add_draft_sends_and_goes_to_split`, and `a_refused_add_keeps_the_pane_and_its_words`.
- `temper/crates/temper-client-view/src/look.rs`: retain integration’s token assignments and Settings element mappings. Append `DOC_EMPTY` and `ADD_DRAFT` after `DOC_FLASH` (391 on the inspected integration); branch tokens 380/381 now collide with `DOC`/`DOC_INNER`. Keep `NAMES` and `tests/look.rs` aligned.
- `temper/crates/temper-client-view/src/words.rs`: preserve integration’s Settings vocabulary and reassigned draft IDs. Assign the six new draft phrases unused IDs; branch IDs 3443–3448 collide with Settings phrases. Merge enum, `ALL`, ID and English tables, and combine derived capacities, including the additional 142 literal bytes. Recount phrase and segment declarations.
- `temper/tools/gallery/src/samples.rs`: retain Settings samples and dispatch alongside `chat-draft-empty`, `chat-new-draft`, and `Then::DraftEmpty`. The inspected integration has 211 samples; these two additions make 213. Preserve both additions in `names.txt` and tests.
- Audit automatically merged `Address::NewChat` readers for integration-added values needing `{ draft: false }` or patterns needing `{ .. }`.
- Regenerate terminal goldens with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-shell-terminal --lib`, including conflicting `chat_delegation_landed.txt`, `chat_delegation_running.txt`, and `first_run.txt`. Preserve Settings-driven navigation visibility together with the new draft button; let rendering determine numbering.
- Regenerate journeys with `heavy cargo run -p temper-client-journeys`, including conflicting `temper/docs/design/client/journeys/welcome-playground/02.html`; preserve navigation visibility and the draft action.
- Run the spec’s focused domain/view checks, client-package checks, and downstream signature checks through heavy. Confirm selected test counts and review generated diffs before pushing the rebased branch.
