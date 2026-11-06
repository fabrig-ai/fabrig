# chat-reading-c: a conversation is read by scrolling, and the newest lines are never lost

Section 5 of `temper/docs/design/draft/chat-reading.md`, on top of part (a) and the look `chat-work` (`look-round-6`). In skein, jig and temper.

This file grows with each push. What isn't written yet is named under *Still to come*.

## What didn't hold in the request

- **`Thread::newer` exists, but reads nothing.** The engine never mints a page's newer cursor: `jig_domain_core::reads::Page.before` is `None` on every path. So after a slide lets the newest lines go `Edges.before` stays `Edge::End`, the window believes it reaches the end, and `newer` falls through to `moved`, which reads nothing while `committed <= loaded`. Calling `newer` from the chat page alone wouldn't bring the end back. `look-round-6`'s "Latest" is bound only where `!reaches(End::Before)`, which against a real engine never holds. The nearest thing: a read beside a line, by the line's position, which is dense and already in the contract. A cursor names a page's edge, and a window bounded by size lets lines go anywhere in a page.
- **The terminal has no keys.** `skein/docs/design/ui-text.md`, 6 and 7: the input is never raw, one line, one command. The nearest thing: the line PgUp and PgDn make, with Enter, and the words `up` and `down`. `up` and `down` are the commands a person finds, in a line under a bound log.

## Parts and tasks so far

| Part | Task | Needs | What it makes true |
|---|---|---|---|
| `log-acts` | `log-surface` | | a log may be bound; `Act::Near`, `State::Continues`, an end request |
| `activity-beside` | `beside-surface` | | `Read::ActivityBeside` crosses the wire and reaches the core |
| `activity-later` | `beside-store` | `beside-surface` | the store walks upward, through the real engine |
| `thread` | `thread-window` | `beside-surface` | a window by size, `Edge::Dropped`, `newer` and `latest` read, `Line::Left` gone |
| `page-part` | `page-reading` | `log-surface` | `Near` from measures, `Continues`, the held view, PgUp and PgDn, in real Chromium |
| `text-shell` | `text-reading` | `log-surface` | `up`, `down`, and what PgUp and PgDn make |

**Critical path so far:** `log-surface` → `page-reading`, about 150 implementer-minutes.

## Decisions so far

- **`thread::Limits.lines` stays, and bites for temper no more.** jig's reference UI and temper's tools price their trees by rows, and both read a thread: removing the field would make their trees' bounds false. temper sets it to `document_bytes / LINE_ROOM` (16,384), which the bytes reach first, so that no number of lines bounds a chat. The draft allows this: "or it grows far past what the bytes hold". `pages::chat::LINES` and every 16 in temper go.
- **`LINE_ROOM`, 64 bytes a line, message, block, object and part,** so the bytes bound what lines make of a page. Without it a window of many empty lines would hold 1 MB of nothing and make a page no bound priced.
- **The page part holds the view itself,** rather than relying on `overflow-anchor`, which Safari doesn't have. It measures the words' move after an apply, so it adds nothing where a browser's own anchoring has already moved them.
- **What's run and what's only read:** `skein/tests/browser` runs Chromium alone. Stories run with `overflow-anchor: none` as well, which is what Safari gives, and that is the nearest a run comes. Firefox and Safari themselves are only read, and the report says so.

## Still to come

temper's domain, its view's bounds, its pages, the terminal's page, the worlds, the journeys and the design's close.

## Overlaps

- **`skein-ui/src/tree.rs`, `builder.rs`, `skein-ui-web/src/mapping.rs`, `skein/docs/design/ui.md`:** `look-round-6/log-end`, `State::ToLogEnd` beside `State::Continues`. The design's rows stand a line apart so they merge.
- **`jig/crates/jig-client-protocol-wire/schema/**`:** `ui-walk-large/proposal-projects`, merged on its trunk.
- **`temper/crates/temper-client-shell-web/src/journey/line.rs`, `replay.rs`, `walk.rs`:** `real-journeys`, one arm each from `log-surface`.
- **`temper/tools/gallery/src/worlds.rs`:** `ui-walk-large`, `look-round-6`, a forced arm.
