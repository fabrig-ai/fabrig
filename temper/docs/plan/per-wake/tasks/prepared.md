+++
id = "prepared"
part = "parts"
kind = "feature"
needs = ["offering", "parts-core", "later"]
files = ["jig/crates/jig-domain-core/src/routing/run/preparation.rs", "jig/crates/jig-domain-core/src/routing/run/parts.rs", "jig/crates/jig-domain-core/src/routing/run/results.rs", "jig/crates/jig-domain-core/src/routing/task/save.rs", "jig/crates/jig-domain-core/src/routing/children.rs", "jig/crates/jig-domain-core/src/routing/sibling_routes.rs", "jig/crates/jig-domain-core/src/tests/runs/parts.rs", "jig/crates/jig-domain-core/src/tests/runs.rs"]
effort = "high"
minutes = 90
+++

# per-wake: a message no prepared part offers leaves the preparation current

## Goal

`jig/docs/design/domain/tasks.md`, 8.6: "The parts are those its preparation chose ... so a
message that arrives between its preparation and its claim is offered or kept by them." Today
the core treats a preparation as stale whenever the task's `run_message` passes its context's
`last_message`, so any message arriving before the claim discards the context and the run
backs off. The story (`tasks/story.md`, row 6) shows it: a question arriving while a parts
charter's first run is between its preparation and its claim leaves the task
`BackingOff`, refusals 1, and no claim. This task makes a preparation stay current against a
message its prepared parts don't offer; the message then keeps its place, and the offering
task's rule wakes the next run for it when this one ends.

Acceptance:

- **One predicate.** The core asks one question, "is this preparation still current", in one
  function beside the parts (`routing/run/parts.rs` or `preparation.rs`): the task is
  `Preparing`, and every word newer than the context's `last_message` (inbox, proposals and
  escalations waiting on it, as `run_message` counts them) is one that
  `tasks::offered(word, offered_kinds(charter, own, context.reasons))` refuses. A word it
  offers, or an always-offered kind, leaves the preparation stale, as today, so the start
  carries it. A charter without parts offers everything, so its preparations behave as today.
- **Used everywhere the marker is compared:** `routing/task/save.rs`
  (`clear_stale_preparation`), `routing/run/preparation.rs` (the brief's two checks and
  `workspace_prepared`'s `current != context`, which must not refuse a context that differs
  from the prepared one only by kept words and the reasons they add), `routing/run/results.rs`
  (`ReadFor::Preparation`), `routing/children.rs` (`GatherRequest::Complete`) and
  `routing/sibling_routes.rs` (`Core::sent`, which adds a sent word to a preparing context only
  when the prepared parts offer it). The claim
  goes ahead with the prepared context: its reasons, its instructions, its start messages; the
  kept word is in none of them.
- **Tests first, in `jig-domain-core`'s lib tests** (`tests/runs/parts.rs`, rows of its
  existing table where one fits): on a charter with a `first` part and a `question` part, a
  question arriving at each point between preparation and claim (before the brief completes,
  during a dependency result read, before the workspace reply) leaves the preparation current
  and the claim made with the prepared reasons and no question at its start; a `words` (always
  offered) arriving at the same points still makes it stale; a charter without parts still
  makes it stale on the question.
- `heavy cargo nextest run -p jig-domain-core --lib` passes; the story's row 6
  (`heavy cargo nextest run -p jig-core-world --test suite -E 'test(/per_wake/)'`) is the
  story task's to finish, but run it once and say in the report whether its claim now happens.

## Inputs

- `temper/docs/plan/per-wake/notes/common.md`; `notes/contracts.md`.
- `jig/docs/design/domain/tasks.md`, 8.6; `jig/docs/design/domain/engine.md`, 7.1 and 7.4.
- The story run's report on row 6: the failing assertion at
  `jig/tests/core/tests/suite/per_wake.rs:199`, `phase: Active(BackingOff { .. }), refusals: 1`.

## Owns and may change

- The files in the header. Not the story's files, nor `jig-domain-tasks` (`run_message` and
  `run_context` stay as they are; the core filters what they give it). A change the predicate
  can't make without the tasks child is a `Question:`.

## Steps

1. The failing lib tests; the predicate; each call site moved to it.
2. `cargo fmt`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the lib
   tests; the core world's per-wake bundle once.
3. Commit (why: a preparation keeps its parts against a message they don't offer); push.

## Answers

2026-11-07. **Q:** `Core::sent` (`routing/sibling_routes.rs`) appends every sent word to a
preparing context and moves its `last_message`, so a kept question still changes the prepared
snapshot: widen this task to that file, or give the guard to a prerequisite task?
**A:** Widen this task: `routing/sibling_routes.rs` is now in its files, and no other task of
the plan owns it. The task's own acceptance (the claim made with the prepared context, the kept
word in none of it) cannot hold without the guard. In `Core::sent`, update the context only when
the word is offered by the prepared parts, through the same `offered`/`offered_kinds` the
predicate in `parts.rs` uses (one rule, not a second copy of it); a kept word leaves the context,
its inbox and `last_message` untouched. Its terminal routing (`land_task` and the `SentRoute`
match) runs whatever the word's fate, as today. A charter without parts offers everything, so
its contexts update as today. Keep the branch's pushed work and finish from it. The story's
`jig/tests/core/tests/suite/per_wake.rs` is the `story` task's and lands after this one, so it
may be absent from this branch: if so, skip that run and say so in the report.
