# ui-composer-focus: the composer keeps the focus after sending

The request: `inputs/request.md`. Its design is on the trunk with this plan: `temper/docs/design/client/README.md`, 3.5 *Where the
focus stands on a chat*, 5.3, 5.11 and section 11; `client/browser.md`, 4.1.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `composer-focus` | `tasks/composer-focus.md` | | A started chat's composer focused; every `Act::Send` asks for the composer; a goal's and an agent's task's composers too; the browser story; the phone's reading |

Critical path: `composer-focus` alone.

## What the cut found, by reading (nothing here was run)

- **The heading's focus isn't `frame.rs`, about line 303, as the request reads it:** that line is the rename's field. A new page's
  heading focus is skein's rule (`skein/docs/design/ui-web.md`, 7, *Going to another page*), and a focus request in the same apply
  beats it. A later insert carrying a request moves the focus too (*A focus request*).
- **Every start that sends and moves ends at one place:** `Chats::told` sets `chats.opening` and shows the chat, for `StartChat` at a
  new chat, its draft and `Configure`, `Act::Starter` and a suggested request. `went` reads `opening` before it drops the page, so it
  holds once, and a list, a reload or Back never sees it.
- **Start goal opens no chat:** it shows `Address::Task` for the goal, so it keeps its heading's focus and changes nothing.
- **`Act::Send` has two more composers:** a goal's and an agent's task's, on `pages.task`. A goal's "Send" disables as the words clear,
  as a chat's does; an agent's task's doesn't, and leaves the focus on the button. The request says "however it was sent", so they're in.
- **The terminal draws no focus:** `skein-ui-text` reads none, so its shell doesn't change.

## What this plan decided

- **The count is raised in `Chat::act`'s `Act::Send` arm, not in `send`:** `Act::Stop` and `Act::AddDraft` go through `send` and ask for
  nothing. It is raised whether or not the send went out: a person who pressed a live "Send" while one is out still wants to type.
- **The phone: one rule at every width.** The tree has no width, and a kit mechanism is outside "one task".
  - After a send, the request's own answer holds: the keyboard stays up, as a messaging app's does. Opening a chat from a list asks
    for nothing, so it raises none.
  - What's left open: a starter's or a suggestion's press on a phone asks for the composer's focus too, and a browser that lets a press's
    activation outlive the answer may raise a keyboard nobody asked for. Android's Chrome may; I couldn't try it, and headless Chromium
    draws no keyboard. The request lists a starter among the starts, so the wide page's rule wins, as `client/README.md`, section 11's
    default, for the user to overturn. Closing it takes a focus request that raises no keyboard, in skein first.
- **A new browser story, as the request asks,** at the floor: one boot and no reload. It may be tight. The tier's last three runs were
  57.8, 73.8 and 78.6 CPU s of 85, before `chat-reading-c/reading-browser`'s and `sign-in-new-tab/browser-story`'s stories. The tier's
  cheapest stories cost 3 to 4.5 CPU s, and one that starts a chat and sends, about 8. If it goes over, the task folds it into the
  `composer` story under a name that says the focus, and gives both measures.
- **A start that's refused keeps today's focus:** "Send" on a new chat or `Configure` disables while the start is out, so a refused start
  leaves the focus where the browser put it. The request doesn't ask for it; named here, not built.
- **A composer that comes back takes the focus:** a composer's request rides its insert, so a chat that loses and regains Message after a
  send takes the focus again. That was already true after "Change something". It only bites someone typing elsewhere on that page at that
  moment; not guarded.
- No stored format, protocol or public interface changes.

## Overlaps I couldn't avoid

- `temper-client-domain/src/pages/chat.rs`, `pages.rs`, `tests/pages/chat.rs`, `temper-client-view/src/pages/agent.rs`, `pages/goal.rs`:
  `ui-pause-close`, in flight. This task's edits are single arms and a four-line `if`.
- `temper-client-domain/src/tests/pages/chats.rs`, `pages/chats.rs` (read only here): `model-catalogue-1/configure-picker`.
- `temper-client-view/src/tests/task.rs`, `tests/browser_all/browser_pages.rs`: `chat-reading-c/reading-browser`. New tests only.
- `client/README.md`, `README.status.md`: `ui-pause-close`, `chat-reading-c/reading-close`, `model-check/trial-close`. New bullets and
  one-line edits.
