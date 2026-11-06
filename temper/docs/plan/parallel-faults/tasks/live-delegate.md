+++
id = "live-delegate"
part = "stalls"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge-change/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge-change/src/domain.rs",
  "temper/crates/temper-engine-domain-forge-change/src/tests.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/change.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/live_delegate.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/questions.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
  "temper/docs/plan/parallel-faults/notes/stall-trace.md",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 100
+++

# A live delegate isn't stalled, and a result on a stalled hold is taken

## Goal

A change holds `its wait stalled at producing` while its producer is working. `waiting_on_agent` resets `since` only while asking, so a producer that works past `change_policy.stall` (30 minutes on the standard profile, 30 s on the small) holds its change. When that run then finishes with a change form and its branch on the forge, the change stays held at `Producing`, head unknown, until someone releases it. This is the blocker of the request's item 1.

Acceptance:

- **Reproduced first, and traced.** Before any fix, a world story on the small profile's stall shows the fault, and `notes/stall-trace.md` records what it found (Steps, 1).
- While a change's producer, repair, resolution or merge job is a live task, however long, the change does not hold `Stalled`, at `Producing`, `Repairing` or `Resolving`, on either stall path (`waiting_on_agent` and `titled_change`).
- Its forge reads don't rise: such a step waits until `now + stall`, not at `queue_window`.
- The stall counts from the delegate's end: a delegate that ended and left nothing to go on from holds `Stalled` one stall later, measured from its end.
- A wait the design still bounds holds as before: CI that never reports at `Checking`, and a gate's task at `Gating`.
- A result on a change held `Stalled` at that delegate's wait ends the hold with no party's release and is taken: a head and title open the pull request; a no-change ends the change with its words.
- Under another hold (`Questions`, say), the result stays on the row, and a release takes it. A no-change there isn't lost: the hold says its words.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.4: the bullets **Stalls**, **A stall holds as any hold does** and **A result that arrives on a held change is kept**, and **A hold ends too** under "A green head ends a CI repair" for the route this one copies.
- `jig/docs/design/domain/tasks.md`, 5.5 and section 7 (a deployment's release of a procedure's own hold).
- `temper/docs/plan/parallel-faults/notes/common.md`.

## Owns and may change

Owns the files in the header.

Reader and neighbour files, literal or call sites only:

- `Heard` gains a field, so each literal gains one line: `temper/crates/temper-engine-domain-forge/src/tests/verification.rs`, `temper/crates/temper-engine-domain-forge/src/tests/fixtures.rs`, `temper/crates/temper-engine-domain-forge-change/src/tests/green.rs` if it builds one whole.
- Helpers in `temper/tests/engine/domain/tests/suite/forge_routes/early_start.rs`, `questions.rs` and `reviews.rs` may become `pub(super)`.
- `temper/tests/engine/domain/tests/suite/forge_routes/goal_branch.rs`, `repair_green.rs`, `resource_admission.rs`, `ci_poll.rs`, `temper/tests/engine/domain/src/walking.rs` and `temper/tests/engine/domain/src/forge_routes.rs` set or read `stall`. Run them. Change one only where it pinned a live delegate holding.

This changes what a stall means, so the task runs its readers' tests: `heavy cargo nextest run -p temper-engine-domain-forge-change -p temper-engine-domain-forge -p temper-engine-domain` and the package holding `temper/tests/engine/domain`.

Departs from the implementer's text in two ways: this task writes `temper/docs/plan/parallel-faults/notes/stall-trace.md`, which `hold-reaches` reads; and it deletes the first `8.4` line it closes from `temper/docs/design/domain/forge.status.md`, the one beginning `8.4: a wait for a live delegate still counts the stall`, leaving the next one (`a stall's hold can leave the change held`) for `hold-reaches`.

Options, each pre-approved:

- If `a_question_open_past_the_stall_holds_nothing` in `questions.rs` pins a hold on a live producer after its answer, as it reads now, flip its tail: past the deadline, still no hold. Say so in the report.
- If the world can't reach a held row with a result arriving once the fix is in, as expected, pin that part in `temper-engine-domain-forge/src/tests/change.rs` from a row given by `Event::Change`, as `title wait` there does.
- If the trace finds a second cause of a late hold in this task's own files, fix it. If it's outside them, write it in the note and leave it.

## Steps

1. **Reproduce and trace, on the trunk's code.** New world `temper/tests/engine/domain/tests/suite/forge_routes/live_delegate.rs`, with `mod live_delegate;` in `forge_routes.rs`, built on `goal_world` and `make` from `early_start.rs` and `next_producer`/`finish_producing` from `questions.rs`. Run with the producer live past the stall, then finish it with `tasks::TaskResult::Change` and a pushed branch. Record in `temper/docs/plan/parallel-faults/notes/stall-trace.md`, as facts with the test that shows them:
   - The row's state and `head` after the hold and after the run's result.
   - The change task's jig phase (`live_task(world, change).phase`) and its `escalation` at each of those points, and whether the goal got a new assignment or a delivered message. `hold-reaches` starts from this.
   - **Why about 57 minutes and not 30.** The request measured from the change's start. From reading, `since` is set by `request_producer`, so a wait for a dependency before the producer starts (8.11, "Dependency waits arm no stall") comes before the clock. Test it: the hold's time minus the producer's request, for a change with no `after` and for a `make(.., &[dependency])`. Say whether that is the cause, or what else delays the hold (a step not offered, a dropped deadline). What can't be told from a world is `unknown`.
   Commit the note, and the story as it fails, before fixing.
2. **Policy,** `temper-engine-domain-forge-change`:

   ```rust
   // boundary.rs, Heard
   /// The change's producer, repair, resolution or merge job is a live task (domain/forge.md, 8.4).
   pub delegate_live: bool,

   // domain.rs
   fn waiting_on_agent(mut change: Change, facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
       if facts.asking {
           change.since = now;
           Stepped { change, decision: Decision::Wait { until: None } }
       } else if standing.delegate_live {
           change.since = now;
           let until = Wall::from_nanos(now.as_nanos().saturating_add(limits.stall.as_nanos()));
           Stepped { change, decision: Decision::Wait { until: Some(until) } }
       } else {
           waiting(change, now, limits)
       }
   }
   ```

   `standing` threads through `producing` and `delegated`, which have it. Gate paths keep calling `waiting`. Tests beside `an_agent_with_an_open_question_never_stalls`, for each of the three states and for `Early::Started`/`Spent`.
3. **Fact,** `temper-engine-domain/src/route/forge_route.rs`: `change_delegate_live(domain, task)` beside `change_delegate_running`: `row.delegate` names a kind other than `Delegate::Gate`, and `core.tasks.task(child)` is there with a phase other than `Phase::Ended(_)`. Set it in `change_step`'s `Heard`. In `temper-engine-domain-forge/src/domain.rs`, the restart at `pending.heard.delegate_running = heard.delegate_running` takes it too.
4. **The other stall path,** `titled_change` in `temper-engine-domain-forge/src/domain.rs` has its own stall check. Pass it `pending` from `decide_step`, and where `delegate_live` is set, do what the policy does: `since = env.wall`, save, and `Wait { until: Some(env.wall + stall) }`.
5. **The stall counts from the end.** In `delegate_result`, a delegate other than a gate sets `row.change.since = env.wall`. A gate's result leaves `since` alone, its stall staying as 8.4 names it.
6. **A result on a stalled hold,** `delegate_result` in `temper-engine-domain-forge/src/domain.rs`, before `producer_result` runs, so a no-change finds `Producing`:

   ```rust
   // domain/forge.md, 8.4: a result ends a stall held at its delegate's wait.
   if let change::State::Held { was, why: change::Hold::Stalled } = &row.change.state
       && delegate_wait(was) // Producing/Repairing/Resolving { requested: true, .. }, listed exhaustively
   {
       row.change.state = (**was).clone();
       row.change.owns_turn = false;
       ended = true;
   }
   ```

   Then, after the row is saved, emit `Request::ChangeDecision { task, decision: change::Decision::EndHold, entry: None, evidence: None }`, which `end_change_hold` routes, and insert `d.procedure_wakes` as `produced_title` does, so a task that wasn't held steps too. `produced_title` keeps taking the title and report on a held row, as it does now: cover that order, the title before the result, and add the reverse if the world allows it.
7. **Under another hold,** in `producer_result`: a `no_change` on `State::Held { was: Producing, .. }` keeps its words in `row.producer_failure`, fitted as that function fits them, so the hold's account says them. A test for `Hold::Questions` holding a passed result that a release takes.
8. Break each rule once by hand (the `delegate_live` arm, the stall reset in `delegate_result`, the hold's ending) and see its test fail. Then fmt, `old_forms.py`, clippy and the focused tests of `temper-engine-domain-forge-change`, `temper-engine-domain-forge`, `temper-engine-domain` and `temper/tests/engine/domain`. Commit and push.

## Report

The implementer's report, plus: the trace's answers in its first lines (why 57; what the jig task and escalation did on the report's path), whether the request's cause holds as worded, and which existing stories you changed and why.
