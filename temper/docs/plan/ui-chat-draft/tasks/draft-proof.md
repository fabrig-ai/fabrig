+++
id = "draft-proof"
part = "draft-proof"
kind = "docs"
needs = ["chat-draft-journey"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/pages/document.rs",
  "temper/crates/temper-client-view/src/pages/chat_sheet.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/client/journeys.md",
]
effort = "high"
minutes = 85
+++

# The proof, and `configure-chat-draft`'s account

## Goal

Everything rendered beside what it copies, what differs fixed or named, and the look request `configure-chat-draft` accounted for. Every commit carries `Look-request: configure-chat-draft`, and the last holds its account's table.

Acceptance:

- **The renders,** at 1440, 1000 and 420 wide, of the client beside the mock's page or the request's hand-made page: a chat in Chat, Split and Draft; the empty pane; a section tinted; Show; the sheet, and refused; "Goal X →"; Chats under Drafts; New chat with a draft; a configuring chat in the three modes; the strip, and the rail open over it. Each from a gallery state or the journey. The mock's modes by a throwaway copy of `chat.html` setting `localStorage['temper-greenfield-v2']`, as the request says. Saved on the builder under `~/src/fabrig/ui-chat-draft/`, one folder per look request, outside any worktree, never committed.
- **The account's render check:** `configure-chat-draft`'s pages from `git show 727f3523ee`, under its overlay, against the reprint under the landed sheets, comparing computed style and box, then the screenshots: identical, or each difference with its reason, the drawing left out, whose `figure` and `skein-ui-flow-*` the request has out.
- **Each difference found** that isn't named is fixed here, where it's the look's, under the trailer; what isn't goes in the report with its owner.
- **The account:** in the last commit's message, one row per overlay rule, page change and state `configure-chat-draft`'s message names: landed, differing and why, or deferred, to whom and where it's tracked. The drawing and `figure`: deferred to skein's `diagrams.md`, which waits for the user; the turns' rules: `ui-chats`' tasks by name if so, or landed.
- **A story in the browser tier,** at its cap of 12 tests, so a few steps added to an existing one, `browser_pages.rs`: a chat's bar, Split and Draft by press, and `Ctrl+\`, on the same chat. Run `heavy cargo nextest run --profile browser-temper -E 'test(=NAME)'` for that test alone.
- **The status:** `README.status.md`'s and `browser.status.md`'s remaining entries of this plan go; `mockups/STATUS.md`, `client/journeys.md`, 9, and any journey's own `README.md` as `look-requests.md`'s section 3, step 8, says: the request's own `configure-chat/02.html` to `04.html` and their README aren't landed (`journeys.md`, 7). `temper/docs/design/draft/look-requests.status.md` if the round taught a step: and that step's line in the draft, under section 7's last bullet.
- `tools/citations --all` stays at zero.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The mock and the requests*, *The proof*.
- `temper/docs/design/draft/look-requests.md`, sections 3 to 5, 7.
- The reports of the tasks before you: `git log --grep='Look-request: configure-chat-draft'`, what they landed.

## Owns and may change

Owns, as `files` lists them, for fixes of what the render finds. May also change, for the same reason alone, any file of `temper/crates/temper-client-view/src/pages/` and `temper/tools/gallery/src/samples.rs`, `temper/crates/temper-client-shell-terminal/src/tests/**` regenerated, and `temper/docs/design/client/journeys/**` prints.

Options, each pre-approved:

- **A difference that a later plan owns,** or `ui-chats`' tasks not yet landed: the report names it, and the account names its owner.
- **The configuring chat's states** with no gallery way to reach them: a hand-made throwaway copy, untracked, gone before the push.
- **A flake in the browser tier:** as `AGENTS.md`, 5.

## Steps

1. Render, compare; fix or name.
2. The browser story.
3. The statuses; `python3 -I -B tools/citations/citations.py --all`.
4. `heavy cargo nextest run` of the client's packages; goldens; reprint.
5. **Check,** the implementer's text; the account; push.

## Report

As `notes/common.md`, Your report: the builder, the folder, and each difference left, with why. Name any part of the request that this plan didn't reach.

## Answers

2026-11-05 (fabrig/ui). Addition: include the walk ui-goal-page left to you: a chat with a draft, Start goal…, the one sheet, Start goal, the goal's page, with its Draft button back to the chat.

2026-11-05 (fabrig/ui). Narrow: this task proves the gallery renders against the mock, and the browser walk moves to chat-draft-journey, which owns it now. Finish on that.
