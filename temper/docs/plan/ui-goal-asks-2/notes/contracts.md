# ui-goal-asks-2: contracts

The run's binding calls. `question-whole` and `goals-start` run side by side in the client's crates, on separate files but for the two tables below: each keeps to its own lines, and neither needs the other's code.

## New words in `temper/crates/temper-client-view/src/words.rs`

Ids from 4101, clear of the other drafts' on their trunks (model-catalogue-2 has reached 3976). Each `Word` goes beside its family in the enum, in `ALL`, in `id` and in the phrase table.

| Task | Word | Id | English |
|---|---|---|---|
| `question-whole` | `NeedsCut` | 4101 | This question is cut here. |
| `question-whole` | `NeedsCutRest` | 4102 | Read the rest |
| `goals-start` | `StartGoalDraft` | 4111 | Draft |
| `goals-start` | `StartGoalDraftHint` | 4112 | docs/design/draft/name.md |
| `goals-start` | `StartGoalNoFunds` | 4113 | Nothing is left in your pool to fund a goal. |
| `goals-start` | `StartGoalNoFundsHeld` | 4114 | Nothing is left in your pool to fund a goal: your open chats hold it. Close one you’ve finished with, and you can start this. |
| `goals-start` | `GoalTasksDone`, replacing `GoalTasksLanded` | 3263, the id it replaces | `N of M tasks done.` |
| `goals-start` | `GoalDoneAt` | 4115 | `done TIME`, one `Name` hole, as `GoalLanded` |

## The funds card's hold

`Word::Hold(HoldWord::Budget)`, temper's override in `temper/crates/temper-client-domain/src/words.rs`, keeps its one `Number` hole, the task. `question-whole` owns it. Its words, exactly:

```
Insufficient activation budget. A release alone holds it again: if a proposal to give task N more budget waits for you, accept it, then release it; otherwise close it, and start the work again once your pool has room: closing an open chat makes some.
```

## A landing

`goals-start` owns it. In `temper/crates/temper-client-domain/src/pages/goal.rs`, `goal::line` answers `Line::Landed` only for `c::ChipEnding::Done(c::ResultValue::Change { kind: 1, .. })`, the test `temper/crates/temper-client-domain/src/pages/delegation.rs` already makes for a delegation's "landed", and a new `Line::Done` for every other `ChipEnding::Done`. `Counts.done` stays every ended done, `Line::Landed` and `Line::Done` alike, since `TaskLine`'s `delegates_done` can say no more.

## Not changed

Funding's rules, `TaskLine`, the wire, the store, `Me`, and the rail's "Goal" (`temper/docs/design/client/README.md`, 5.3). No budget field is added to the sheet or to Amend. No new browser test.
