# look-round-16: a chat's terminal

The request: `inputs/request.md`. It lands the look requests `chat-terminal`, `chat-terminal-font`, `chat-terminal-draft` and `chat-terminal-esc` (`060312d7e8`, `e08fc0004f`, `66b3894988`, `e8dd3197b5` on `origin/look-requests` when cut), in that order, by `temper/docs/design/draft/look-requests.md`, sections 3 and 5. `notes/common.md` holds the rules every task follows.

This file grows with each push; what is not yet in it is still being cut.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `md-underscore` | `tasks/md-underscore.md` | | skein's Markdown reads `_x_` and `__x__` as emphasis and strong, never within a word |
| `web-screen` | `tasks/web-screen.md` | | skein-ui-web's kept store (local storage) and full screen, Escape held by the keyboard lock |
| `field-options` | `tasks/field-options.md` | | skein-ui's state *options*: a menu its text field moves through, Enter pressing and Tab completing, as a web `listbox` |

Critical path, as it stands: being cut.

## What was decided

- **Emphasis in a draft has two causes,** and two tasks. skein's reader takes `*` alone, so `_Draft, with the user._`, which the stock instructions ask agents to write, stays literal; and temper's draft pane draws its first paragraph as plain text, past the reader. The first is skein's (`md-underscore`, released first, as it needs nothing); the second is the draft task's. `_` follows CommonMark's rule within a word, so `snake_case` and `read_branch` stay as written; this cut wrote it in `skein/docs/design/ui.md`, 10.

- **Three generic mechanisms go into skein first** (`AGENTS.md`, 1): the browser keeps no choice across tabs (its small store is the tab's session storage), nothing asks for full screen or holds Escape, and no key but Enter and Escape reaches a field's owner. `web-screen` gives the first two, written in `skein/docs/design/ui-web.md`, 10 and 11; `field-options` the third, written in `ui.md`, 3.2, and `ui-web.md`, 6 and 7. A field's options take no new act: Enter presses the active option and Tab's completion is the field's input, so temper's view binds them as it binds any item.
- **Two skein-ui-web tasks at once,** against one writer per crate: `web-screen` owns its platform, store and a new `screen` module, `field-options` its mapping and page; no file and no interface is shared, so they run side by side.

## Overlaps

- `look-round-15/node-hint` (integration, landing now) changes `skein-ui`'s `tree.rs`, `builder.rs` and `tests/tree.rs`, and `skein-ui-web`'s `mapping.rs`, `tests/mapping.rs` and `golden/mapping.html`, which `field-options` also changes: a node's hint beside a new state, apart but for the golden, which the second to merge regenerates.

- `skein/docs/design/ui.status.md`: `look-round-15/node-hint` (integration) edits its section 3 entry; `md-underscore` adds and deletes a section 10 one. Apart.
