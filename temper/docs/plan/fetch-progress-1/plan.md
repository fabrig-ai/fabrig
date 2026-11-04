# fetch-progress-1: the plan

From `inputs/request.md`: while an owned Forgejo's set up fetches its binaries, the person sees how far it has gone, in words in both shells, and in the browser a bar that fills.

The plan is released in pieces; the client's tasks follow, and this file with them.

## Parts and tasks

| Part | Task | Needs | Minutes | What it does |
|---|---|---|---|---|
| progress-value | `progress-value` | | 55 | skein-ui's fraction, a progress's own runs, the web's attributes, the stylesheet's 101 rules |
| fetch | `fetched-line` | | 20 | the line's type, its writer and its reader, for both sides |
| fetch | `fetch-count` | `fetched-line` | 65 | the size, the refusal, the lines on stdout |
| box | `box-relay` | `fetched-line` | 80 | the pipe's `fetched` line, none on a terminal, replies kept |
| row | `engine-fetched` | | 35 | the forge domain keeps the latest, the row carries it |
| row | `wire-fetched` | | 30 | `SettingUp.fetched`, generated, every literal `None` |

## What the cut found

- **`aria-valuetext` never reaches a page.** `skein-ui-web/src/mapping.rs` writes it from the progress node's own runs, and `Builder::run` refuses every role but text, so a built progress has no such runs. A text child does not stand in: `role="progressbar"` presents no children to assistive technology. The draft says the mapping writes it "today"; the code holds the line, and no tree reaches it.
- **The box's reply queue is short, and its `Say` drops on no room.** It holds the longest group plus two lines. A plain relay of a hundred lines could lose a `started` or a `failed`, and an ask would never end. So the box holds one line, the latest, and writes it only once its replies are.
- **A step's lines reach the terminal three ways:** `--echo steps`, a log not written, and a failure's tail. A plain relay would fill a failed fetch's tail with percentages and push out why it failed.
- **The machine moves on at the step's exit,** so a line left in the pipe could follow the next `step` line. It stays in the log.
- **`box.line_bytes`'s least is 64,** and `forgejo-runner` as the binary's word takes the longest line to 71. The word is `runner`, and the longest line is exactly 64.
- **skein-http delivers no byte past `Content-Length`,** so "a body past its size refused" can be shown at the machine alone.
- **`forge_fetch_command` asserts an empty stdout** on every row, so its `digest` row changes with the fetch.

## What the cut decided

- **A progress's text is runs of its own** (`skein/docs/design/ui.md`, 3.4 and 5): `Builder::run` takes a progress as well as a text node, so "its text as `aria-valuetext`" (`skein/docs/design/ui-web.md`, section 6) comes true, and `close` can assert that a fraction has its words. It widens a public builder call by one role, in the direction the draft already asks for, so it is decided here and not asked. A text child stays allowed with no fraction, so no temper fixture moves. Recorded under **Design departs** in `progress-value.status.md`.
- **The forge domain keeps the latest line, not the box's link** (draft 4 says the link): the link decides nothing and keeps no state a row reads (`temper/docs/design/protocol/forge.md`, 2.8). Still in memory alone, gone at the next step, at `started` and at `failed`. Recorded under **Design departs** in `fetch-progress.status.md`.
- **The progress sits beside the connection,** `Row.fetched` in the engine, as `failed` does, and not in `Connection::SettingUp { step }`: forty-odd literals and patterns stay as they are, and that keeps this plan off the lines `repository-copies-1` and `look-round-3` are on.
- **A new pipe word's form:** `fetched FORGE BINARY DONE TOTAL`, `BINARY` `forgejo` or `runner`, `TOTAL` `none` with no size. On stdout, `fetched forgejo DONE of TOTAL bytes` and `fetched runner DONE bytes`, which a person can read. The draft left the forms open.
- **A zero `Content-Length` is no size,** since a fraction's total is above zero.
- **A binary already there says nothing,** and on macOS the bytes are the archive's, its extraction saying nothing.

## skein

`skein-ui`, `skein-ui-web`, `skein-ui-bundle`, and `skein-ui-text`'s fixture. **skein-ui isn't on hearth's skein surface:** hearth builds nothing of this and nothing of hearth changes.
