# chat-reading-b: a run of calls is one short line

Section 2 of `temper/docs/design/draft/chat-reading.md`, on top of part (a) and the look `chat-work` (`look-round-6`). `look-round-6` drew the line, and its account deferred four things here: a run's time of one turn, the table's further tools, an agent's page's running state and its pill. Nothing on the trunk runs the line through a shell, so this plan adds those stories too.

## What didn't hold in the request

- **A turn's start can't be known.** A committed turn says only when it ended, so a run of one turn has no start to measure from (section 7). The nearest thing is the line before it, below.

## Parts and tasks

| Part | Task | Needs | What it makes true |
|---|---|---|---|
| `words` | `run-words` | | the table, a cut input's argument, the terminal's "…" |
| `agent-surface` | `agent-call` | | an agent's page keeps the call under way, `Latest` reads newer |

More follows as it holds together: the time, the agent's page, the stories, the close.

**Critical path, as it stands:** `run-words` (80).

## Choices for the user to read on a real chat

- **The verbs,** section 7: `read`/`Read` "read N files", `list`/`Glob` "listed N folders", `write`/`Write` "wrote N files", `edit`/`Edit` "edited N files", `remove` "removed N files", `search`/`Grep` "searched the files N×", `shell`/`Bash` "ran N commands", beside today's `read_docs` and `settings`. These are the tools a chat has that aren't a card's: a native run's and a program's names for the same work. `read_file` and the forge's other reads stay their names, as the draft's example has it. `recall` and `message` stay their names too: their first members are a number or a body, which read badly as an argument. All of it is one function, `chat_line.rs`.
- **The time,** section 7: from the line before the run's first turn to the run's last turn, any activity line, drawn or not. The run's beginning is such a line, so a chat woken after hours doesn't count the hours. At a window's head, and for a turn's second run, there's no such line, and it measures from its first turn's end: often no time at all. It runs a little long, as it holds the model's writing of any text that opened the turn.

## Decisions

- **No engine change for the time.** Storing a turn's start would be a stored format and a protocol, for a figure the line before it gives nearly as well.
- **A call under way with an unreadable input is the tool's name,** where `read_docs` today says "reading " with nothing after it.
- **A cut input reads to its first member:** a `write`'s content is nearly always past the live bound, and "write" would say much less than "writing src/a.rs".
- **jig's reference UI keeps a folded line a call:** it knows no tool's words (`chat-reading.status.md`, 2).

## Overlaps

Each is kept to a small hunk, and each task's spec says so.

- **`chat-reading-c`, the most:** `pages/chat.rs`, `chat_line.rs`, `agent.rs`, `words.rs`, `tests/chat.rs`, `tests/task.rs`, `temper-client-domain/src/pages.rs`, `pages/task.rs`, `tests/chat_line.rs`, the terminal's prints; `temper/docs/design/client/README.md`, 5.9, and `chat-reading.status.md`.
- **`look-round-8`, `ui-walk-2`, `ui-walk-large`, `sign-in-new-tab`, `plainer-panels`, `first-usable-model`:** `temper-client-view/src/words.rs`, new lines in this plan's id block, 3800..=3839.
- **`look-round-8`, `ui-walk-2`:** `temper-client-shell-web/temper.css`.
