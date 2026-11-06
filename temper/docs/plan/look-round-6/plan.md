# look-round-6: how a chat's calls and its answer read

Lands the look request `chat-work` (`3f707babb9`), by `temper/docs/design/draft/look-requests.md`, sections 3 to 5. `8ad410e969` is held.

## Parts and tasks

1. `log-end` (skein): a log says when the person has scrolled away (`data-log-away`), and a button with `State::ToLogEnd` takes it back. Out at once: it needs nothing, and the pill needs it.
2. `call-line`: a run of calls on one line, and its open state, on a chat's page and an agent's. Out at once.
3. `answer`: no reasoning, no notices, the writing end. Needs `call-line`. In the next push.
4. `reading`: "Latest ↓", "Loading earlier messages…", the reprints, the render check, the screenshots and the account. Needs `answer` and `log-end`. In the next push.

**Critical path:** `call-line`, `answer`, `reading`.

## Decisions

- **A skein task beyond the request's three.** Only skein's page part knows a log's scroll position, and a pill that can't tell is either always shown or never. The mechanism is thin and generic, so it goes first (`AGENTS.md`, 1). `chat-reading`'s section 5 builds a log's head beside it.
- **A document's write ends a run,** where the draft names a text, a person's words and a card: the write keeps its line where it falls, so a run can't span it without moving calls past it.
- **A failed call shares the description with the time,** in the signal's colour: skein's summary holds one name and one description. The account names it as a difference.
- **An agent's page:** a run's results become its rows' outcomes. Left as lines they'd end every run.
- **The stored format, protocol and public interfaces** are unchanged by this plan so far.

## Overlaps

- `ui-rename-focus/focus-target`: `skein-ui-web/src/page.rs`, `page/web.rs`, and `skein/docs/design/ui-web.status.md`, with `log-end`.
- `ui-rename-focus/rename-row`, `ui-final-walk`: `temper.css`; new rules stand under `ui: chats`.
- `ui-final-walk`, `ui-walk-large/home-true`, `ui-walk-large/needs-proposal`: `temper-client-view/src/words.rs`.
- `ui-walk-large`, `ui-final-walk`: the gallery's `tests.rs`, regenerated on a catch-up.
- `chat-reading-a`: `pages/chat.rs`, `limits.rs`, tests reading "Earlier text omitted", from the other side.
