+++
id = "hold-reaches"
part = "stalls"
kind = "feature"
needs = ["live-delegate"]
files = [
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/stall_hold.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 90
+++

# A stall's hold reaches the goal, or a person

## Goal

In the request's item 2, a change's panel said `held: its wait stalled at producing`, the inbox was empty, the goal wasn't woken, `task show` said `Waiting for news`, and it sat idle for 1 h 43. By jig's rule a hold escalates and wakes its holder, and stock's `tasks.escalation_stall_ms` of an hour passes an undecided one up to a person. So an empty inbox after 1 h 43 suggests no escalation was ever made: the change's row held with its jig task not held. Prove or refute that, and make a change and its task hold together.

Acceptance:

- A stalled change under a goal: the task's phase is `Phase::Held { why: tasks::Hold::Stalled, .. }`, its `escalation` is `Waiting { holder: EscalationHolder::Task(goal), .. }`, and the goal has a run with that entry, whether it was idle (a fresh assignment) or running (a delivered message).
- A stalled change with no deciding task above it reaches a person's inbox, as `goal_tool.rs` reads one.
- A goal that doesn't decide: past `escalation_stall`, the entry passes up and is in the person's inbox.
- A row is never left `Held` with its task not: a hold decision that finds its procedure not due still holds the task.
- If the fault doesn't reproduce on any path, these stories stand as the proof, the code is unchanged, and the report says so plainly with what was tried.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.4: **A stall holds as any hold does**.
- `jig/docs/design/domain/tasks.md`, 5.5, 5.7 ("Decisions wait above it") and section 9 (**Where they wait**, **Waking**, **Stalled**).
- `temper/docs/plan/parallel-faults/notes/stall-trace.md`, which `live-delegate` wrote: what the report's path did before that fix.
- `temper/docs/plan/parallel-faults/notes/common.md`.

## Owns and may change

Owns the files in the header.

Reader and neighbour files: helpers in `temper/tests/engine/domain/tests/suite/forge_routes/early_start.rs`, `goal_landing_held.rs` and `goal_tool.rs` may become `pub(super)`.

jig is not this task's. A fault in jig's escalation routing (`jig/crates/jig-domain-core/src/admission/escalation.rs`, `jig/crates/jig-domain-tasks/src/escalation.rs`, `wake.rs`) is reproduced in jig's own world (`AGENTS.md`, 5, fix in the lowest tier that shows it). Default: if the fix is a few lines there, make it on this branch with a test in that crate and say so; if it changes what a holder is or jig's public interface, stop and ask.

Departs from the implementer's text: delete from `temper/docs/design/domain/forge.status.md` the line beginning `8.4: a stall's hold can leave the change held`. If the story shows the reading was wrong, replace that line's account with what you found only if a gap stands; otherwise delete it and say so in the report.

Options, each pre-approved:

- **The expected cause.** `change_decision` in `forge_route.rs` opens with `procedure_due(task)`, and when the procedure isn't due it returns, dropping the choice. By then the forge child has already saved the row `Held`, and a later step reads a held state as `Decision::None`, so nothing ever holds the task. Default fix: in that arm, a `Decision::Hold(why)` pushes `Work::Tasks(tasks::Event::Hold { task, why })`, mapped as `hold_change_decision` maps it, its account kept where it has one; see the `tasks::Hold::Effects` push beside the `ChangeDecision` arm for the shape. A hold jig's `Event::Hold` can't carry, for want of its account, takes the next option.
- **Level-triggered instead, or as well.** A due procedure whose row is `Held` and which is not being released steps to `Decision::None` and goes idle. Default: in `change_decision`, a `Decision::None` with the row `Held` and the task due decides `hold_change_decision` for the row's `why`, so that any wake mends it. Take this if the world shows a path the first misses.
- **The decision was refused, not dropped.** If `ProcedureStep` is refused after the row held (a read fence, say), keep the row and task together by the same mend, and say which refusal.
- **Escalation made, holder wrong or not woken.** If the world shows the task held but no holder hearing, trace `recipient_after`'s `holder.deciding`, kept goals and `wake_holder`, then follow the jig default above.
- **The words only.** If the task holds and the holder hears, and only `task show`'s line misleads, that's `temper-client-*`, not this plan's: report it, under `## Left`.

## Steps

1. New world `temper/tests/engine/domain/tests/suite/forge_routes/stall_hold.rs`, with `mod stall_hold;` in `forge_routes.rs`, on the small profile's stall. Read `notes/stall-trace.md` first. Its waits are those that still stall after `live-delegate`: CI that never reports at `Checking`, and a producer ended with nothing to go on from at `Producing`. Stories, each asserting row, task phase, escalation and who heard:
   - `a_stalled_change_holds_its_task_and_wakes_its_idle_goal`
   - `a_stalled_change_reaches_its_goals_live_run`
   - `a_stalled_change_with_no_goal_reaches_its_persons_inbox`
   - `an_undecided_stall_passes_up_to_a_person` (advance past `escalation_stall`)
   - `a_hold_decided_while_its_procedure_is_not_due_still_holds_its_task`: deliver the decision out of step, as `held_change.rs` holds back a read's event. Also at the route's unit tests (`forge_route/tests.rs`) if the world can't time it.
2. Run them and read what fails, then fix by the options. Break the fix once by hand and see the test fail.
3. fmt, `old_forms.py`, clippy and the focused tests of `temper-engine-domain` and `temper/tests/engine/domain`, and `jig-domain-tasks`/`jig-domain-core` if you touched them. Commit and push.

## Report

The implementer's report, plus, first: whether the request's account holds, what was dropped or refused, and on which path. Say plainly if it didn't reproduce.
