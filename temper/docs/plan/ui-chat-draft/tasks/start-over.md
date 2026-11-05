+++
id = "start-over"
part = "start-over"
kind = "feature"
needs = ["document-pane"]
files = [
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/pages/document.rs",
  "temper/crates/temper-client-view/src/confirm.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/document.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client/tests/suite/configuring_chat.rs",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 60
+++

# Start over, and the configuring chat's line

## Goal

A configuring chat's bar has "Start over", which closes it and opens a fresh one, and its document's line says where its proposal stands.

Acceptance, temper's `client/README.md`, 3.5, *A chat's document*: *The bar* and *The pane*. Every commit carries `Look-request: configure-chat-draft`.

- **"Start over,"** `.temper-start-over`, a quiet outlined button, after the group on a wide page and alone on a phone, on a configuring chat while it lives and has a document, and in place of "Start goal…". `Act::StartOver`: asks, through `crate::confirm`, "Start a new configuring chat? This one stays in Chats."; on `Act::Confirm`, closes the chat with the chat's own close (`shapes::close_chat`, `Chat::prepare_close`, `chat.close`), and on the close's answer shows `Configure { project }`. A refused close stays on the page with its words, as a close does. The closed chat stays in Chats, Closed, its conversation and document kept.
- **`configure-chat-document-split`** shows it.
- **The line's left,** on a configuring chat: "T7 · not proposed yet" with no card and no departed proposal; "Proposal 12 · waits for an owner" while a proposal's card waits; "Accepted at 14:03" once one was accepted. The time's form as 5.12's.
- **The world:** `temper/tests/client/tests/suite/configuring_chat.rs`, `start_over_closes_and_opens_a_fresh_configure`.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The client*, 4 and 5.
- `git show 727f3523ee`: its message's "Start over" and its states.
- `temper-client-domain/src/pages.rs`, `confirmation`; `temper-client-view/src/confirm.rs`, `close`: it names `Word::Close`, so Start over needs its own dialog's words, and `pages/chat.rs`' `cards`, `departures`, and `jig`'s `c::Proposal`'s `waiting`.

## Owns and may change

Owns, as `files` lists them. May also change `temper/crates/temper-client-domain/src/acts.rs`, if `Confirm` is there, for its new kind, `temper/crates/temper-client-shell-terminal/src/tests/**`, regenerated.

Options, each pre-approved:

- **"Accepted at":** a proposal's key leaves `task.proposals` once decided. The chat keeps a `decided: Option<(u64, Wall)>`: when a departed key's own read, `Read::Proposal`, comes back ended with `EndedValue::Proposal`, accepted, `Accepted at` with its `at`. If `Read::Proposal` answers unknown for a decided one, the line says "Proposal 12 · decided" with no time, in a word of its own, and the report says so.
- **"Waits for an owner":** from `waiting.holder`: any holder takes these words.
- **A confirm answered after the page moved:** the chat's own close handles it: nothing more.

## Steps

1. The act and its confirm; tests: `start_over_asks_first`, `confirmed_it_closes_with_the_chats_own_close`, `the_close_answered_shows_configure`, `a_refused_close_stays`, `cancel_changes_nothing`.
2. The line; tests: `the_line_says_not_proposed_yet`, `the_line_says_it_waits`, `the_line_says_accepted`.
3. `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal`; the client world's `configuring_chat` suite.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
