# look-round-16: a chat's terminal

The request: `inputs/request.md`. It lands the look requests `chat-terminal`, `chat-terminal-font`, `chat-terminal-draft` and `chat-terminal-esc` (`060312d7e8`, `e08fc0004f`, `66b3894988`, `e8dd3197b5` on `origin/look-requests` when cut), in that order, by `temper/docs/design/draft/look-requests.md`, sections 3 and 5. `notes/common.md` holds the rules every task follows.

This file grows with each push; what is not yet in it is still being cut.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `md-underscore` | `tasks/md-underscore.md` | | skein's Markdown reads `_x_` and `__x__` as emphasis and strong, never within a word |

Critical path, as it stands: being cut.

## What was decided

- **Emphasis in a draft has two causes,** and two tasks. skein's reader takes `*` alone, so `_Draft, with the user._`, which the stock instructions ask agents to write, stays literal; and temper's draft pane draws its first paragraph as plain text, past the reader. The first is skein's (`md-underscore`, released first, as it needs nothing); the second is the draft task's. `_` follows CommonMark's rule within a word, so `snake_case` and `read_branch` stay as written; this cut wrote it in `skein/docs/design/ui.md`, 10.

## Overlaps

- `skein/docs/design/ui.status.md`: `look-round-15/node-hint` (integration) edits its section 3 entry; `md-underscore` adds and deletes a section 10 one. Apart.
