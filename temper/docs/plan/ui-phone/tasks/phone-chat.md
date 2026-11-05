+++
id = "phone-chat"
part = "phone-chat"
kind = "feature"
needs = ["phone-bits"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "medium"
minutes = 45
+++

# A chat on a phone: check it landed

## Goal

`ui-chats` owns a chat's phone frame. Measure each item of the request's section 1 against the mock's `chatPage()` at 390 and 420, and fix only a miss. Every other page keeps its frame.

Acceptance, each a measurement in your report, the client's beside the mock's `chat.html`:

- **The frame:** no top bar and no tab bar; the session's height `100dvh`; the log scrolls and the composer keeps the foot when the thread is long (`chat-reference-T1`, `chat-solo-T10`).
- **The top:** `.temper-session-top` padding `10px 12px`, gap `8px`; `.temper-back`, "‹" to Chats, 36 by 36, its glyph 20px; `.temper-trail` hidden; the title strong on one line, a long one cut rather than wrapped; the byline's words hidden, "Change model" in their place.
- **The thread:** the log's padding `20px 16px`.
- **The foot:** `8px 16px`, then 14px and the inset. **It stays at 16,** by a look request (`notes/common.md`, What stays as it is); say that in your table beside the mock's 12.
- **The byline open** (`chat-byline-open`): its panel wholly on screen.
- **Every other page keeps its frame:** `task-reference-T12`, `task-solo-T3`, `goal-running` and `goal-sending` keep their rail and tab bar at 390 and 420. A configuring chat (`configure-chat`) is a chat: its rail and tab bar go, as every chat's do (`temper/docs/design/client/browser.md`, 2.5, *The Chats page and every chat*).
- **With a draft:** Chat and Draft alone, `.seg .split` hidden, `.doc { padding: 24px 20px 60px; border-left: 0 }`: that's `ui-chat-draft`'s. If it isn't on your base, it's "not there yet" in your report. If it is, measure it and name a miss; don't touch it.

## Inputs

- `temper/docs/plan/ui-phone/inputs/request.md`, section 1.
- `temper/docs/design/client/browser.md`, 2.5, *The Chats page and every chat*.
- `temper/docs/design/client/greenfield/app.js`, `chatPage`; `mock.css`, the last `@media (max-width: 760px)` block.
- `temper/docs/plan/ui-phone/notes/common.md`; `temper/docs/plan/ui-phone/notes/sweep.md`, The driver.

## Owns and may change

Owns the `files` above: a miss is fixed where the rule stands, under `/* ==== ui: chats ==== */` or `/* ==== ui: settings ==== */`.

- **A miss the sheet can't fix** may change `temper/crates/temper-client-view/src/pages/chat.rs` or `session.rs` and `look.rs` by a line or two. Those files are beside `ui-chat-draft`: keep it small, and say so. More than that goes under `## Left`.
- **Which states:** the gallery's `chat-*` and `configure-chat*`. The screenshots of each at 390 and 420 go in `~/src/fabrig/ui-phone/chat/`.
- **A chat with its composer focused** is the proof's, not this task's.

## Steps

1. **Measure** with a throwaway driver, or the sweep's if it's on your builder: a table, item by item, the client's and the mock's.
2. **Fix a miss,** smallest first.
3. **Check:** the implementer's text: `heavy cargo nextest run -p temper-client-gallery`, and `-p temper-client-view` with clippy if you touched the view. `cargo fmt`. Commit and push. If nothing missed, there's nothing to commit: say so in the report's first line, and push nothing.

## Report

As `notes/common.md`, Your report, with the table.
