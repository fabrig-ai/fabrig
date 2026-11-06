# ui-pause-close: Pause, Resume, Close

One vocabulary for stopping work, in the browser and the terminal alike. The request: `inputs/request.md`. Its design is on the
trunk with this plan: `temper/docs/design/client/README.md`, 3.5 *A person's pause*, 3.6, 5.8, 5.11 and section 11;
`client/browser.md`, 7.2; `client/greenfield/README.md`'s departure.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `chat-pause` | `tasks/chat-pause.md` | | A chat's Pause, Resume, "Paused", a message that resumes it, no "Applied"; the world test; `chat-paused`; the browser check |
| `close-words` | `tasks/close-words.md` | `chat-pause` | A goal's and an agent's task's Close, an agent's task's Pause and Resume, closed's words; the gallery; the journeys; the status |

Critical path: `chat-pause`, then `close-words`. They both write `temper-client-view` and `temper-client-domain`, so they run in a
chain. The user's story, a chat that can come back, goes first.

## What this plan decided

- **Telling a person's pause from another hold needs no new fact.** A chip's hold already names its placed hold: owner, mode,
  scope, lifted (`jig/docs/design/client/domain/contract.md`, 5.6). `chat::paused(chip, me)` reads them. No wire or store change.
- **Message and Resume are one act already.** jig's compound request names this case: "words to a task the party stopped, and its
  release" (`contract.md`, 3.4), and a lone message releases nothing (`jig/docs/design/domain/tasks.md`, section 7). So a message to
  a paused chat is `Ask::Compound [Message, Release]`, the words first since the thread finds their number by place. It commits whole
  or not at all, which is how a refused message can't release: nothing in temper keeps that but jig's all-or-none.
- **An agent's task's "Stop" isn't an end, whatever its word.** The request reads it as one ("Stop task" becomes "Close task"), but
  it is jig's Stop, a hold ("Its run stops, and the task is held until it is released"). A renamed hold would make "can't be reopened"
  false. So its page gets a goal's acts: "Pause" or "Resume", the hold it was, sent at once like a chat's; and "Close", a true
  `Ask::Cancel`, on the one sheet. Nothing is lost, and "Close task" says what the request asks.
- **A message resumes a chat alone.** The request decides that for a chat. A paused goal's or agent's task's words wait for Resume,
  since a goal's release lifts its whole subtree: `client/README.md`, section 11 keeps it as a default the user may change.
- **"Cancel this task?" goes.** Only the gallery reached it: no page draws Cancel but a goal's and a chat's. The agent's task's
  "Close this task?" takes its place, with what it changes.
- **A close sheet's keeping act is "Keep it",** a chat's, on all three: the request asks that they say the same. A goal also keeps
  "What landed on its branch stays there.", which is true and useful there. "Keep going" stays for "Start over".
- **"Paused" is quiet** (`STATUS_QUIET`): a person's own pause is no alarm. It shows on the bar and on Chats' and Home's rows, by
  `chats::status_words`.
- **The request's `client/README.md`, 5.5 is 3.6:** "Applied" and the sheet live there (5.5 is LLM accounts' words). The design
  edits went to 3.6.
- **The gallery's and the state's names follow the words:** `goal-closed`, `goal-closing`, `task-closing`; `State::Closed`;
  `GOALS_STATE_CLOSED`. `Act`s and `ActKind`s stay, as the request says.
- **No stored format or protocol changes,** so temper/forge needs no word about one.
- **A browser check, not a new browser test:** the request asks for a check with proof; the world test on the real engine root holds
  the walk from here on, at a lower tier and no cost to the browser tier's budget.

## Overlaps I couldn't avoid

- `temper-client-view/src/pages/chat.rs`: `chat-reading-c/reading-browser` and `model-catalogue-1/configure-picker`. This plan's edits
  stay in `bar_actions`, `session_composer`'s hint and `act_status`.
- `temper-client-view/src/words.rs`: `model-catalogue-1/configure-picker` and `picker-offer`.
- `temper-client-view/src/tests/task.rs`: `chat-reading-c/reading-browser`.
- `temper.css`: `chat-reading-c/reading-browser`, `ui-walk-3` (merged to its trunk). `close-words` only renames one class in place.
- `tools/gallery/**`: `chat-reading-c/reading-close`, `ui-walk-3`, `sign-in-new-tab`, `model-catalogue-1/picker-offer`. Regenerate on a
  catch-up, as `notes/common.md` says.
- `client/README.md` and `README.status.md`: `chat-reading-c/reading-close`, `model-check/trial-close`. The status line
  `temper/docs/design/STATUS.md` too. Each edit here is a line or a new entry.
- `client/journeys/**`: `chat-reading-c/reading-close`. `close-words` re-records last, so whichever lands second re-records again.
