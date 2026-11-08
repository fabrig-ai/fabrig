# wake-faults: contracts

The shapes that cross this plan's tasks. Binding: a change one of them
needs is a `Question:`.

## 1. An agent's busy start (tasks `start-refusals`, `worker-busy`)

`jig-domain-host` gains one variant of its agent capability's events
(`jig/crates/jig-domain-host/src/boundary.rs`, `FromAgent`), beside
`StartRefused`:

```rust
/// The agent had no room for the run's start, before any run began.
StartBusy { owner: Token },
```

The hub takes it as it takes `StartRefused` for the same start (a
pending start's workspace released, its held refusals ended, nothing
kept, a start the engine no longer wants taking the cancellation's
path), and answers the engine once with
`Ending::Refused(Refusal::Busy)`: the existing busy refusal, which the
hub link already maps to `tasks::End::Refused` (a pause, no try). No
wire value is added: the worker wire already carries a busy refusal.

`start-refusals` adds and handles the variant; `worker-busy` sends it
from temper's worker. Neither changes `StartCause`.

## 2. A refused start's class (task `start-refusals`)

`jig/crates/jig-domain-hub-link/src/payload.rs`, `ending`, maps a run's
refused start:

| `host::Ending` | `tasks::End` |
|---|---|
| `Refused(Busy)` | `Refused` (as today) |
| `Refused(Invalid(Transcript))`, `Refused(Invalid(Start { cause: Transcript, .. }))` | `Failed(Class::Transient)` (as today) |
| `Refused(Invalid(_))`, every other | `Failed(Class::Permanent)` (was `Invalid`) |

`Class::Permanent` and its retry policy exist; nothing stored changes.

## 3. The per-wake story table (tasks `covered-entries`, `late-reasons`)

Both tasks add rows to `per_wake_scenarios`
(`jig/tests/core/tests/suite/per_wake.rs`) and helpers to
`jig/testing/jig-core-kit/src/stories/per_wake.rs`. So that their
branches merge cleanly, each writes only where this says:

| | `covered-entries` | `late-reasons` |
|---|---|---|
| seeds | 868, 869 | 866 (its existing row, rewritten), 870 |
| new functions in `per_wake.rs` | after `a_goal_interrupted_in_its_first_cut_keeps_its_parts_and_choice`, before `#[test]` | in place of `a_question_arriving_after_preparation_waits_for_the_next_run`, which it rewrites |
| `case!` lines | after seed 867's | seed 866's line rewritten, seed 870's right after it |
| kit helpers | after `escalation`, before `park` | after `question_from`, before `escalation`'s doc comment |
| test charter | unchanged | `per_wake_charter` gains a fourth part, `(Reasons::RESULT, b"R")` |

`covered-entries` never asserts a run's instructions where a result is
among its reasons, so the fourth part changes none of its rows.

## 4. `jig-domain-tasks`'s two writers (tasks `finish-parks`, `covered-entries`)

- `finish-parks` owns, in `jig/crates/jig-domain-tasks/src/run.rs`, the
  terminal admission's live-delegates check (where `refuses_live` is
  decided and `Problem::live_delegates` is pushed, today lines ~503–512),
  and `src/run/ends.rs` if the park belongs beside `run_parks`.
- `covered-entries` owns `wake.rs`, `escalation.rs`, `proposals.rs`,
  `domain.rs`, `admission.rs`, `reasons.rs` and `tests/offering.rs`, and
  in `run.rs` only the claim's `read_escalations` reset (today line
  ~136).
- Each adds its unit rows in test files the other doesn't name.
