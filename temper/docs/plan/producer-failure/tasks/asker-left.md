+++
id = "asker-left"
part = "asker-left"
kind = "feature"
needs = ["own-question-parks"]
files = ["temper/crates/temper-engine-domain-forge-change/src/relay.rs", "temper/crates/temper-engine-domain-forge-change/src/tests/relay.rs", "temper/crates/temper-engine-domain/src/route/forge_route/relay.rs", "temper/tests/engine/domain/tests/suite/forge_routes/questions.rs", "temper/tests/engine/domain/tests/suite/forge_routes/foremen.rs"]
effort = "high"
minutes = 75
+++

## Goal

Two things in the change's relay. First, the story the request asked for, on jig's rule
from `own-question-parks`: a producer asks, finishes, is not ended; its answer resumes
it; its finish goes through. Second, no question goes silently: an asker that ends with
a question open tells the change's requester, once.

Acceptance:

- Under `foreman`: a producer asks, finishes `failure`. It is live and idle, its foreman
  runs on, the question stays open, the change holds nothing. The foreman's answer
  reaches it, its next run finishes with its change, and the change goes on.
- The same under `up`, the requester answering.
- A cancelled asker whose question was taken or with a foreman: the requester gets one
  `Words` message, `Task A ended with its question N open; it reached no one: LINE`.
- A cancelled asker whose question was asked of the requester, or whose answer was
  held: one `Words` message at the step that takes or holds the answer, `Task A ended;
  your answer to its question N goes no further: LINE`. The change's own question goes
  as it does today.
- With no message room the question stays on the record, `more` is set, and a later
  step says it once. A restart in between says it once.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11: *Its agents may ask*, *An asker that ends
  first*, and the closing paragraph on message room.
- `jig/docs/design/domain/tasks.md`, 5.7.
- `temper/docs/plan/producer-failure/notes/common.md`.

## Owns and may change

Owns the `files` above.

May change: `temper/tests/engine/domain/tests/suite/forge_routes/producer_failure.rs`
only if a helper you need moves. Any other `match` on `RelayAct` the compiler names:
the new arm only. No store change: `Stands` and `Open` stay as they are, so a passed-up
question says so at its answer, not at its asker's end.

Options:

- Ending the asker in the two rewritten tests: `engine::Tool::Cancel` from the requester,
  as `foremen.rs`'s `a_cancelled_foreman_goes_up` does. If the requester may not cancel
  it, use `goal_landing.rs`'s `cancel_task` shape, and say so.
- The foreman's cancel when there is no room for the words: keep the question and its
  foreman whole until a step has room, or cancel at once and keep the words' duty some
  other way that stays on the record without a store change; your call, and the report
  names it.

## Steps

Sketch, `temper-engine-domain-forge-change/src/relay.rs`:

```rust
/// Tell the requester that a question's asker ended.
Left { asker: u64, number: u64, line: Box<[u8]>, answered: bool },
```

`answered: false` for `Stands::Taken` and `Stands::Foreman` dropped in `take_questions`;
`answered: true` for `Stands::Held` dropped there, and for `Stands::Asked` at the answer
in `relay`'s loop, where `heard.ended.contains` drops it today. `Left` counts against
`room`, as `Answer` and `Ask` do.

`temper-engine-domain/src/route/forge_route/relay.rs`, `apply`: the arm pushes a
`StepMessage { target: requester, kind: tasks::MessageKind::Words, words }`, fitted to
`tasks.message_bytes` with `…`; the target as the `Ask` arm reads it. `has_foreman`
and `defer_foreman` take the arm.

Tests, failing first:

1. `foremen.rs`: `a_producers_finish_waits_for_its_foremans_answer`, from
   `foreman_world`, `ask`, `finish_review` with a failure, `verdict`, `wait_answer`,
   `next_producer`, then `finish_producing`.
2. `questions.rs`: `a_producers_finish_waits_for_its_requesters_answer`, from `up_world`.
3. `foremen.rs`, `an_ended_asker_cancels_its_foreman`, ending the asker by cancel, and
   the requester's inbox holds the words once. `questions.rs`,
   `an_asker_that_ends_drops_its_question`, likewise, with the answered words.
4. `temper-engine-domain-forge-change/src/tests/relay.rs`: `Left` for each `Stands`;
   none for a live asker; `room` 0 keeps the question and sets `more`, the next step says
   it once.

Each new test bites: remove the rule once, see it fail, restore.

Then `cargo fmt`, clippy on the changed packages, and focused runs of
`temper-engine-domain-forge-change`, `temper-engine-domain-forge`, `temper-engine-domain`,
`temper-engine-domain-world` and `temper-engine-forge-world`. Commit, push.

## Report

The choice on the foreman's cancel with no room; the two tests' new ending; what is left.
