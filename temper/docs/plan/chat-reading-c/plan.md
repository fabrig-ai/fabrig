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

**Critical path so far:** either of these, about 40 implementer-minutes.

## Still to come

The store's forward walk; the thread's window by bytes; the page part's measures, its held view and its keys; the text shell's commands; the temper domain and view; the worlds; the journeys and the design's close.

## Overlaps

- **`skein-ui/src/tree.rs`, `builder.rs`, `skein-ui-web/src/mapping.rs`, `skein/docs/design/ui.md`:** `look-round-6/log-end`, `State::ToLogEnd` beside `State::Continues`. The design's rows stand a line apart so they merge.
- **`jig/crates/jig-client-protocol-wire/schema/**`:** `ui-walk-large/proposal-projects`, merged on its trunk.
- **`temper/crates/temper-client-shell-web/src/journey/line.rs`, `replay.rs`, `walk.rs`:** `real-journeys`, one arm each from `log-surface`.
- **`temper/tools/gallery/src/worlds.rs`:** `ui-walk-large`, `look-round-6`, a forced arm.
