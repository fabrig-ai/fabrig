+++
id = "proof"
part = "proof"
kind = "feature"
needs = ["chat-stories", "playground-recording"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 75
+++

# The proof beside the mock

## Goal

The client beside the mock, state by state, with each difference fixed or named; and nothing of the old chat page left.

Acceptance:

- **Renders,** the client's beside the mock's page, at 1440, 1000 and 420 wide: Chats (All, Live, Closed, empty, no repository), New chat, a live chat
  with tool lines, a chat at work with dots, a chat with a change running and landed, a closed chat, a configuring chat, the byline open. Saved under
  `~/src/fabrig/ui-chats/` on the builder, outside any worktree, never committed.
- Each difference a rule can fix is fixed, under the marker. Your report lists each that remains, with why: the mock's samples aren't the client's
  words, a part the terminal needs, `look-round-5`'s column.
- **The sweep:** `rg` finds no `temper-sidebar`, `temper-chatbar`, `temper-chat\b`, `at=start` (but its one test) or `look::CHAT\b`; no `temper.css` rule
  whose class no view draws among those this plan replaced; `look::NAMES` holds no name for a token gone, reserved slots as they stand.
- **The gallery:** every fixture of the request's *Gallery* exists, and `temper-client-gallery`'s tests pass.
- `README.status.md`, `browser.status.md`, `journeys.status.md` say nothing this plan closed; design text that the code taught otherwise is corrected.
- `python3 -I -B tools/citations/citations.py --all` is at zero.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, *The proof*, *Gallery*, *The mock*; its 420 px note under *For every task of this plan*.

## Owns and may change

Owns, as `files` lists them.

May also change: anything of `temper/crates/temper-client-view/src/pages/{chats,new_chat,chat,session}.rs`, `choice.rs`, `cards/delegation.rs` for a
one-line fix a render shows; the goldens and prints it makes stale; a design line or status entry, naming each in your report.

Options, each pre-approved:

- A render needing a state the gallery lacks: a throwaway script, gone before the push.
- A difference that would take more than a rule or a one-line view fix goes in your report's `## Left`, with its file and the fix you'd make.

## Steps

1. Render the mock's pages and the gallery's, by a throwaway script. Compare at each width.
2. Fix, re-render. The sweep.
3. **Check,** `notes/common.md`, clippy, push. Delete the script.

## Report

As `notes/common.md`, *Your report*: the builder, the directory, and the list of differences, one line each with why.
