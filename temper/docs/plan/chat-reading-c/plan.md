# chat-reading-c: a conversation is read by scrolling, and the newest lines are never lost

Section 5 of `temper/docs/design/draft/chat-reading.md`, on top of part (a) and the look `chat-work` (`look-round-6`). In skein, jig and temper.

## What didn't hold in the request

- **`Thread::newer` exists, but reads nothing.** The engine never mints a page's newer cursor: `jig_domain_core::reads::Page.before` is `None` on every path. So after a slide lets the newest lines go `Edges.before` stays `Edge::End`, the window believes it reaches the end, and `newer` falls through to `moved`, which reads nothing while `committed <= loaded`. Calling `newer` from the chat page alone wouldn't bring the end back. `look-round-6`'s "Latest" is bound only where `!reaches(End::Before)`, which against a real engine never holds. The nearest thing: a read beside a line, by the line's position, which is dense and already in the contract. A cursor names a page's edge, and a window bounded by size lets lines go anywhere in a page.
- **The terminal has no keys.** `skein/docs/design/ui-text.md`, 6 and 7: the input is never raw, one line, one command. The nearest thing: the line PgUp and PgDn make, with Enter, and the words `up` and `down`. `up` and `down` are the commands a person finds, in a line under a bound log.

## Parts and tasks

| Part | Task | Needs | What it makes true |
|---|---|---|---|
| `log-acts` | `log-surface` | | a log may be bound; `Act::Near`, `State::Continues`, an end request |
| `activity-beside` | `beside-surface` | | `Read::ActivityBeside` crosses the wire and reaches the core |
| `activity-later` | `beside-store` | `beside-surface` | the store walks upward, through the real engine |
| `thread` | `thread-window` | `beside-surface` | a window by size, `Edge::Dropped`, `newer` and `latest` read, `Line::Left` gone |
| `page-part` | `page-reading` | `log-surface` | `Near` from measures, `Continues`, the held view, PgUp and PgDn, in real Chromium |
| `text-shell` | `text-reading` | `log-surface` | `up`, `down`, and what PgUp and PgDn make |
| `page-busy` | `log-busy` | `page-reading` | `Near` again when a log's busy state goes |
| `page-part` | `head-reads-on` | `page-reading`, `log-busy` | a log at its head goes on reading, and nothing throws; from a check on this trunk |
| `view-bounds` | `view-window` | `thread-window` | the page's bounds priced from the window's bytes |
| `domain` | `chat-reads` | `thread-window` | `Newer` is read, `Latest` and sending go to the end, no `fill` |
| `page` | `chat-page` | `chat-reads`, `view-window`, `text-reading`, `log-surface` | the log bound, its lines keyed, no "Older", the terminal's prints |
| `worlds` | `reading-worlds` | `chat-page`, `beside-store` | the user's journey, through the real engine and store |
| `browser` | `reading-browser` | `chat-page`, `page-reading`, `log-busy`, `beside-store` | a long chat in real Chromium |
| `close` | `reading-close` | `reading-worlds`, `reading-browser` | prints, the journeys named for fabrig/ui, design text and statuses |

**Critical path:** `beside-surface` (40) → `thread-window` (90) → `chat-reads` or `view-window` (60) → `chat-page` (90) → `reading-worlds` (60) → `reading-close` (40), about 380 implementer-minutes. `log-surface` → `page-reading` → `log-busy` → `reading-browser` joins it at the end, about 225.

**The tasks that stand on `look-round-6`:** `view-window`, `chat-reads`, `chat-page`, `reading-worlds`, `reading-browser` and `reading-close` read what it landed (`Act::Latest`, `older`, `latest`, `look::LATEST`, `State::ToLogEnd`, `data-log-away`). It has no landing yet as this is cut, its tasks all merged and its trunk in the queue, and a plan's `needs` can name only its own tasks. They begin after `thread-window`, which leaves it time, and each checks its base first and asks rather than guessing. The skein and jig tasks don't need it.

## Decisions so far

- **`thread::Limits.lines` stays, and bites for temper no more.** jig's reference UI and temper's tools price their trees by rows, and both read a thread: removing the field would make their trees' bounds false. temper sets it to `document_bytes / LINE_ROOM` (16,384), which the bytes reach first, so that no number of lines bounds a chat. The draft allows this: "or it grows far past what the bytes hold". `pages::chat::LINES` and every 16 in temper go.
- **`LINE_ROOM`, 64 bytes a line, message, block, object and part,** so the bytes bound what lines make of a page. Without it a window of many empty lines would hold 1 MB of nothing and make a page no bound priced.
- **The page part holds the view itself,** rather than relying on `overflow-anchor`, which Safari doesn't have. It measures the words' move after an apply, so it adds nothing where a browser's own anchoring has already moved them.
- **What's run and what's only read:** `skein/tests/browser` runs Chromium alone. Stories run with `overflow-anchor: none` as well, which is what Safari gives, and that is the nearest a run comes. Firefox and Safari themselves are only read, and the report says so.

- **A log's `Busy`,** beside the height, is what ends a stall: a page of lines the conversation doesn't draw moves no height, so `Near` wouldn't come again and an earlier page would stay unread. And held through a failure it stops a loop: `log-busy`, a later task than `page-reading`, whose spec was already out.
- **Each line is keyed.** The conversation's lines had no key, so a page read in above would match by position and rewrite every line. Neither the page part's holding nor a browser's anchoring can keep a view through that.
- **The lines of a window that let its end go:** new words and live text don't draw until the end is back. Drawn at the foot of a window with lines missing above them, they'd read as if they followed the lines above.
- **A chat's "Older" ends with an agent's:** they share `older` and the walk, and the draft says no "Older". An agent's page still has no pill, as `look-round-6` left it.
- **`fill`'s remains:** with `rows` at 16 in both shells it read a second page only for an empty window. That much stays, so that a shell measuring nothing isn't stranded by an empty page.
- **temper's README takes a new 5.15,** and `reading-close` reconciles 3.5, 5.4, 5.9 and 5.11 after `look-round-6` has landed, so as not to meet its 145 lines there. `chat-reading.status.md`'s entry waits for the same reason.

## Overlaps

Each is kept to a small hunk, and each task's spec says so.

- **`look-round-6`, the most:** `skein-ui/src/tree.rs`, `builder.rs`, `skein-ui-web/src/mapping.rs`, `page.rs`, `page/web.rs`, `skein/docs/design/ui.md`, `ui-web.md`; `temper-client-view/src/pages/chat.rs`, `chat_line.rs`, `agent.rs`, `words.rs`, `limits.rs`; `temper-client-domain/src/pages.rs`, `pages/chat.rs`; `temper/docs/design/client/README.md` and its status; the terminal's goldens; `browser_pages.rs`. The design's rows here stand a line apart from its rows so that they merge. The skein tasks take what it landed, by catch-up.
- **`ui-rename-focus`:** `temper.css`'s `#app > .temper-session .temper-thread > [role=log]`, taken by catch-up and kept; `skein-ui-web/src/page.rs`.
- **`real-journeys`:** `temper-client-shell-web/src/journey/line.rs`, `replay.rs`, `walk.rs` and `temper/tools/journeys/src/again.rs`, one arm each from `log-surface`; `temper/docs/design/client/journeys*`, in `reading-close`.
- **`ui-walk-large`:** `jig/crates/jig-client-protocol-wire/schema/**`, merged on its trunk; `temper-client-view/src/words.rs`; the gallery.
- **`look-round-8`, `ui-walk-2`, `ui-walk-large`:** `temper-client-domain/src/pages.rs`, one match arm; `temper-client-view/src/words.rs`, one word.
- **`first-usable-model`:** `temper/docs/design/client/journeys.md`, in `reading-close`.
- **`reference-walks`, `v0.1.0-ladder`, `configuring-walks`,** the queues: nothing they name yet.

## Journeys

Named for fabrig/ui by `reading-close`'s report. A chat no longer reads a second page by itself and no page presses "Older", so a recording of a long chat may go stale (`temper/docs/design/client/journeys.md`, section 8). `chat-draft/21.html` and `22.html` hold "Older" today.

## What the user reads

`walk = false`: the user reads it on a real chat. Firefox and Safari aren't run by any tier, so the position staying put there is theirs to see: the stories run Chromium with `overflow-anchor: none` as the nearest thing to Safari.
