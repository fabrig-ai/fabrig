+++
id = "change-story"
part = "change-temper"
kind = "feature"
needs = ["change-root"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/change_producer.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/src/forge_routes.rs"]
effort = "high"
minutes = 90
+++

# change-story: a change under its named producer and LLM choices, in the engine's world

## Goal

An engine world story proves contract C4 end to end through the root, jig and the forge: a
run holding jig's control family calls `change` naming a producer's charter and an LLM choice
per role; the change's producing task is made under that charter at that choice, and its agent
gate at the review choice; a call naming a charter the run may not delegate to, and one naming
a model the producer's charter does not allow, are each refused, naming why, with nothing made.

Acceptance, in a new file `temper/tests/engine/domain/tests/suite/forge_routes/change_producer.rs`
(registered in `forge_routes.rs` beside `mod goal_changes;`), one thread and one loop, a seed
replaying:

- `a_change_names_its_producer_and_llm_and_its_tasks_run_under_them`: the world's charters 1
  (the forge's change charter), 2 and 4 (the review charter), with two configured models, the
  second allowed by charter 2 alone (below). A goal (as `goal_changes.rs` makes one with
  `goal_request`, holding the control family) calls `change` with `"producer": 2` and
  `"llm": {"producer": {"main": {"model": SECOND}}, "review": {"main": {"model": FIRST}}}`.
  The change task's spec carries parameter 11; its row's `producing_charter` is 2 and
  its `producing_llm` and `review_llm` the choices; its producing task's record has executor
  `Agent { charter: 2 }` and `llm` the producer's choice; its run's assignment runs the second
  model. Where the world's landing rule asks an agent gate, its gate task has charter 4 and the
  review choice; if no rule of this world asks one, assert the row's `review_llm` alone and say
  so in the report.
- `a_change_naming_a_charter_the_run_may_not_delegate_to_is_refused`: the same call from a run
  whose authority's delegation kinds leave out charter 2 is refused as beyond authority (the
  refusal `change-root`'s report names), its answer an error naming it, and no task made.
- `a_change_naming_a_model_its_producer_s_charter_does_not_allow_is_refused`: `"producer": 1`
  with the producer's choice naming the second model is refused naming that model and
  `model outside the charter` (jig's rendering of `LlmFault::Charter`), and no task made.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1 (`producer`, `llm`).
- `temper/docs/design/domain/forge.md`, 8.1 and 8.9.
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C4, C5.

## Owns and may change

Owns the files in the header.

- `temper/tests/engine/domain/src/forge_routes.rs`: one hook, if the world has none, giving a
  story a second configured model and charter 2's models (e.g. a `StoryLimits` member or a
  `World` builder method beside `with_landing_deployment`), the model's provider as the first
  model's; no other hunk. If the world already lets a story set models and a charter's
  models, use that and leave the file alone.
- The run's delegation kinds for the second story: narrow them as the world's other
  authority stories do (the goal's authority, or policy 1's delegation in the world's
  configuration for that story); default: the policy's delegation kinds without charter 2.
- Not yours: anything under `temper/crates/`. A failure that shows the root, jig or the
  forge wrong is reported, not fixed here.

## Steps

1. The hook, if needed; the three stories, using `goal_changes.rs`'s helpers (`world`,
   `change_row`, `finish`) through `super::` where they are public to the suite, or their
   equivalents in the new file.
2. Narrow: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/change_producer::/)'` (check the count is 3); then `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/forge_routes::/)'`, since the world's
   hook is shared.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D
   warnings`. Commit ("Show a change made under its producer and LLM choices") and push.

## Report

As the implementer's text says, with the three stories' names and their run times.

## Answers

### 2026-11-03: Should fab arrange the owning task's repair of `temper/crates/temper-protocol-store/src/records/application/refusal.rs:500` and resume this task (recommended), or explicitly authorize that repair here? The missing `people_v14::Refusal::Judged(_)` match arm prevents compiling the acceptance tests. Work is committed and pushed. (reference-m3/change-story-1, foreman reference-m3/change-story/foreman-1)

Leave the decoder repair to its owning task, then resume change-story to complete validation.
The spec settles the ownership boundary: “Not yours: anything under `temper/crates/`. A failure that shows the root, jig or the forge wrong is reported, not fixed here.”
`temper/crates/temper-protocol-store/src/records/application/refusal.rs:500` is outside this task’s permitted files; no repair there is authorized here.
After the owning task’s repair is available, resume the narrow three-story run, confirm the count is 3, then run the forge-route stories and required checks.
Acceptance remains unobserved: the report says no tests executed and “The task is not accepted yet.”

### 2026-11-03: Should fab arrange the owning tasks' repairs of the two compilation blockers below and resume validation (recommended), or defer change-story? The existing implementation and current branch head are pushed. (reference-m3/change-story-2, foreman reference-m3/change-story/foreman-2)

Leave both compilation repairs to their owning tasks, then resume change-story to complete validation.
The spec states: “Not yours: anything under `temper/crates/`. A failure that shows the root, jig or the forge wrong is reported, not fixed here.”
This covers both `temper/crates/temper-protocol-store/src/records/application/refusal.rs:500` and `temper/crates/temper-engine-domain/src/restart.rs:544`; neither repair is authorized here.
The recorded answer already directs: “Leave the decoder repair to its owning task, then resume change-story to complete validation.”
Fab should arrange both owning tasks’ repairs. Once available, resume the narrow three-story run, confirm the count is 3, then run the forge-route stories and required checks.
Acceptance remains unobserved: the report says “No tests executed” and “The task is not accepted yet.”

### 2026-11-03: Should fab arrange the owning tasks’ repairs of temper/crates/temper-engine-domain/src/restart.rs:544 and temper/crates/temper-protocol-store/src/records/application/refusal.rs:500, then resume validation (recommended), or defer change-story? The current branch head is pushed. (reference-m3/change-story-3, foreman reference-m3/change-story/foreman-3)

Leave both compilation repairs to their owning tasks, then resume change-story to complete validation.
The spec states: “Not yours: anything under `temper/crates/`. A failure that shows the root, jig or the forge wrong is reported, not fixed here.”
This covers `temper/crates/temper-engine-domain/src/restart.rs:544` and `temper/crates/temper-protocol-store/src/records/application/refusal.rs:500`.
The recorded answer already settles this exact question: “Fab should arrange both owning tasks’ repairs.”
Once those repairs are available, resume the narrow three-story run, confirm 3 tests pass, then run the forge-route stories, formatting and required clippy check.
Acceptance remains unobserved: the run reports “No tests executed” and “The task is not accepted yet.”
