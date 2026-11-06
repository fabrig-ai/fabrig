# ui-pause-close: the run's binding shapes

Both tasks build to these. The design: `temper/docs/design/client/README.md`, 3.5, *A person's pause*; 3.6, *The sheet*; 5.11.

## A person's pause, in the domain

In `temper/crates/temper-client-domain/src/pages/chat.rs`, public, created by `chat-pause`, read by `close-words`:

```rust
/// The person's own pause (client/README.md, 3.5): held by a placed hold of theirs, mode Stop, over the task's own subtree.
#[must_use]
pub fn paused(chip: &c::Chip, me: &c::Me) -> bool
```

True when `chip.details.phase` is `ChipPhase::Held { .. }` and `chip.details.details.hold`'s `placed` is `Some`, with
`mode == HoldMode::Stop`, `scope == HoldScope::Subtree { task: chip.task }`, `!lifted` and
`owner == HoldOwner::Party { person: me.party.number }`. Nothing else is a pause: an `Exceeded` hold, a hold another person
placed, a hold over a task above it.

`goals::state` keeps its own test of the placed hold, with no `me`: a goal's "paused by you" doesn't change here.

## The asks

- **A chat's Resume:** `Sending::Resume`, new in `chat::Sending`; `Act::Resume` on an open chat sends, at once,
  `c::Ask::Release { project, task, revision: c::Revision { value }, again: None }`, `value` the chip's
  `ChipPhase::Held { revision, .. }`, only when `paused` and `allowed(task, ActKind::Release)`.
- **A message to a paused chat,** when `paused` and `allowed(task, ActKind::Release)`, is one compound in this order, the
  words first so that the thread's place stays 0:

  ```rust
  c::Ask::Compound { project, operations: [
      c::Operation::Message { project, task, words },
      c::Operation::Release { project, task, revision: c::Revision { value }, again: None },
  ] }
  ```

  Any other message stays `c::Ask::Message`. Both go through `Sending::Message`, with one key, and both mark the thread's
  words `sending` at place 0: `jig_client::thread` finds a compound's message number by its place
  (`jig/crates/jig-client/src/thread/model.rs`). A supplied message (`document::WRITE_THIS_UP`) takes the same path.
- **All or none:** `jig/docs/design/client/domain/contract.md`, 3.4. A refused compound changes nothing in the engine; the
  page handles it as a lone message's refusal (`thread.words.refused`, the composer's refusal). Nothing else keeps it.
- **An agent's task's** acts are `task::Sending::Stop` and `task::Sending::Resume`, at once, from `Act::Pause` and
  `Act::Resume` (`close-words`). Its message stays `c::Ask::Message`.

## Words

| `Word` | Says | Task |
|---|---|---|
| `Pause` (was `GoalPause`) | "Pause" | `chat-pause` |
| `Resume` (was `GoalResume`) | "Resume" | `chat-pause` |
| `ChatPaused`, new | "Paused" | `chat-pause` |
| `ChatSendResumes`, new | "Sending resumes this chat." | `chat-pause` |
| `CloseGoal` (was `StopGoal`) | "Close goal" | `close-words` |
| `GoalCloseTitle` (was `GoalStopTitle`) | "Close this goal?" | `close-words` |
| `GoalCloseStays` (was `GoalStopStays`) | "What landed on its branch stays there." | `close-words` |
| `CloseTaskTitle` (was `CancelTaskTitle`) | "Close this task?" | `close-words` |
| `CloseTask` (was `CancelTask`) | "Close task" | `close-words` |
| `GoalClosed` (was `GoalStopped`) | "closed" | `close-words` |
| `GoalClosedLine` (was `GoalStoppedLine`) | "Closed by you." | `close-words` |
| `GoalClosedHeading` (was `GoalStoppedHeading`) | "Closed by you. What landed on the goal branch stays there." | `close-words` |
| `GoalClosedBefore` (was `GoalStoppedBefore`) | "Closed before it landed." | `close-words` |

`close-words` removes `Word::Stop`, `Word::StopTaskTitle`, `Word::StopTask` and `Word::StopHolds`. A word's phrase id doesn't
change with its name. Every close sheet's keeping act is `Word::KeepIt`; `Word::KeepGoing` stays for "Start over".

## The chat's status

`chats::status_words` answers `(look::STATUS_QUIET, "Paused")` when `chat::paused(chip, me)`, before it reads `chat_status`.
So a chat's bar, its row on Chats and its row on Home all say it. "Held" and `STATUS_HELD` stay for every other hold.

## Gallery names

| State | What | Task |
|---|---|---|
| `chat-paused`, new | "Paused", Resume, Close, the composer's "Sending resumes this chat." | `chat-pause` |
| `task-paused`, new | "Paused", Resume, Close, no own held card | `close-words` |
| `goal-closed` (was `goal-stopped`) | closed's words | `close-words` |
| `goal-closing` (was `goal-stop-dialog`) | its sheet | `close-words` |
| `task-closing` (was `task-cancelling`) | its sheet, with a task below it | `close-words` |
| `task-stopping` | goes, with its `Confirmation::StopTask` | `close-words` |

`temper/docs/design/client/browser.md`, 7.2 already lists them.
