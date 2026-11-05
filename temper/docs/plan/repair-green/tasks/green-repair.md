+++
id = "green-repair"
part = "green-repair"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge-change/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge-change/src/domain.rs",
  "temper/crates/temper-engine-domain-forge-change/src/tests.rs",
  "temper/crates/temper-engine-domain-forge-change/src/tests/green.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/result.rs",
  "temper/crates/temper-engine-domain-forge/src/result_tests.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/tests/engine/forge/src/changes.rs",
  "temper/tests/engine/forge/tests/suite/green.rs",
  "temper/tests/engine/forge/tests/suite/main.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/repair_green.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
  "temper/tests/engine/domain/src/tier_task_show.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 130
+++

# A green head ends a CI repair the change can still step on

## Goal

From a real goal: CI went red, a repair began, CI was run again by hand and passed at the same head, and the change stayed `repairing`; a release made a fresh repair with nothing to repair. Today `repairing`, `repair_due` and `released_state` read no CI. Make a step that reads CI passed at the head end a CI repair that isn't running, and leave a run in progress alone. A change whose own task is held hears nothing yet: that is `green-hold`, which builds on this.

Acceptance, each with a test below:

- A CI repair (`Repair::Ci`, `Repair::Semantic`) at a green head, due and unmade, makes no delegate and the change goes on: `Checking` or `Gating` at that head, by `take_pushed_head`.
- Its task live and not running (`delegate_running` false): `Decision::CancelRepair { head }`, the task cancelled with the reason in `notes/contracts.md`. When that result is heard the change goes on, holding nothing, and `row.producer_failure` is cleared.
- A repair that ended failed or cancelled, or finding nothing to do, at a green head holds nothing and the change goes on. At a head not green each holds as today, and the existing tests of that stay as they are.
- A run in progress (`delegate_running` true) is left alone: nothing cancelled, and what it delivers is taken as today. While so, a `Wait { until: Some(_) }` is capped to the forge client's `poll` from now, so a run that ends into a retry is found.
- Released at a green head, no fresh repair: `Hold::Failed` and each other reason, and a bare release of `Hold::Authority` at a repair or checking goes on rather than holding again.
- `CancellingGates` whose gates have ended at a green head goes on, asking no repair and writing no result line.
- A push moves the head: CI passed on an earlier head changes nothing, as `facts.ci` is read at the branch's head. A head its repair pushed before it stopped running is the head the rule asks of.
- `Repair::Gate` and `Repair::Landing` are untouched at any CI.
- `change.repairs` stays as asked; only a release resets it, as today.
- The result says it: `repairs: 1: CI failed: build, then CI passed at 0123456789ab`.
- `forge.status.md`'s entry `8.4: a green head ending a CI repair` is gone.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.4 ("A green head ends a CI repair"; "A release"); 8.2, step 3 and step 7; 5.3.
- `temper/docs/plan/repair-green/notes/common.md`, `notes/contracts.md`.

## Owns and may change

- Owns the files in the header's `files`.
- `Heard` gains a field: every literal naming all its fields takes `delegate_running: false`, in `temper-engine-domain-forge-change/src/tests/**`, `temper-engine-domain-forge/src/tests/**`, `temper-engine-domain-forge/src/landing.rs`, `temper-engine-domain/src/route/forge_route/{relay,tier_route,tests}.rs`, `temper-engine-domain/src/admission.rs`, `temper/tests/engine/{forge,domain,protocol-forge}/**`. A literal only, and the compiler names them; `..heard()` spreads need nothing. Don't touch `RelayHeard` or the public `Heard`s, which are other types.
- `Decision` gains variants: the matches that list it (`settle.rs`'s `cap`, `decide_step`, `landing.rs`) take the arm their neighbours take. `settle.rs` may change for that alone.
- If the question's asker already goes when a repair is cancelled (`route/forge_route/relay.rs`, "its asker ended"), leave it. If a cancelled asker's question stays open on the record, drop it the way a person's cancel does and say so in the report.
- If `facts.writer_taken` stays true for a task between attempts, so that `advance` waits before reaching `repairing`, take the writer's state as a run in progress and say so in the report: don't reorder `advance`.

## Steps

**Stories first.** Scripted root, `temper/tests/engine/forge/tests/suite/green.rs` (and `mod green;` in `main.rs`), fixtures from `changes.rs`, in the shape of `simulation.rs`'s `a_review_asking_for_changes_repeats_at_the_repaired_head`. `change_step_task` takes a status; add a variant that also says `delegate_running`.

- `a_green_head_ends_a_due_ci_repair`, for `Ci` and `Semantic`
- `a_green_head_cancels_a_repair_not_running_then_goes_on`
- `a_run_in_progress_is_left_to_finish`, then delivers at a new head, checked as ever
- `a_release_at_a_green_head_makes_no_fresh_repair`
- `a_release_at_a_failing_head_still_asks_a_fresh_repair`
- `ci_passed_on_an_earlier_head_changes_nothing`: a push, then a passing status written to the earlier commit
- `a_review_repair_is_untouched_at_a_green_head`, and released too

Through the root and jig, `temper/tests/engine/domain/tests/suite/forge_routes/repair_green.rs` (`mod repair_green;` in `forge_routes.rs`), from `failed_ci_repair`, `repair_cancel.rs`'s helpers and `ci_poll.rs`'s `status`. This world is above the forge world because a task's phase, its cancel and a release only meet here.

- `a_retrying_repair_ends_on_a_green_head`: the run answers `tasks::End::Failed(tasks::Class::Run)`; then green; the repair's task ends cancelled, `temper_engine_domain_world::tier_task_show::show` reads `CI passed at head`, and no failure word; `made` stays; the change is past `Repairing`.
- `a_repair_held_for_its_brief_ends_on_a_green_head`: held as `brief_failure.rs`'s `a_repair_with_neither_section_ends_held_naming_the_reason` holds it; the change still `Repairing`; then green; as above.
- `a_run_in_progress_finishes_at_a_green_head`: `Running`, green, ticks: the same run still runs, no cancel; its answer is taken.
- `a_retry_after_green_is_found_within_a_poll`: green first, then the run ends into a retry; within `poll` its task is cancelled.
- `a_cancelled_repair_released_at_a_green_head_makes_no_fresh_one`: `cancelled_repair`, the status written and the release sent with no tick between; `made` stays 3, no delegate, `repairs` 0, and the change goes on.
- `a_failed_repair_at_a_green_head_holds_nothing`.

`repair_cancel.rs`'s `a_released_cancelled_repair_asks_again_at_a_head_still_failing` stays as it is, and passes.

**The policy,** pure tests in `src/tests/green.rs` (`mod green;` beside `mod merge;` in `tests.rs`), a row for each acceptance line above:

```rust
fn repairing(change, requested, why, facts, standing, now, limits) -> Stepped {
    if ci_reason(why) && let Some(head) = green_head(facts) {
        if !requested || standing.delegate == Status::Failed {
            return repair_ended(change, head, now);
        }
        // Passed stands as a delivery, below. A task not yet made has nothing to cancel.
        if standing.delegate != Status::Passed
            && let Some(Delegate::Repair(_)) = facts.delegate
            && !standing.delegate_running
        {
            return Stepped { change, decision: Decision::CancelRepair { head } };
        }
    }
    // as today
}

fn repair_ended(change: Change, head: Commit, now: Wall) -> Stepped {
    let mut stepped = take_pushed_head(change, head, now);
    stepped.decision = Decision::RepairEnded { head };
    stepped
}
```

`cancelling_gates`: after its live-gate wait, a green head takes `take_pushed_head`. `released_step`: skip the `Hold::Authority` re-hold where the hold is on CI's reason and the head is green: `was` `Repairing` with a CI reason, `Checking` or `CancellingGates`; `Gating` keeps it. `repair_due` needs no change: nothing reaches it at a green head.

**The top,** `temper-engine-domain-forge/src/domain.rs`:

- `step_change`'s merge into a pending step copies `delegate_running`.
- `decide_step`'s loop: `Decision::RepairEnded { head }` calls `row.result.ci_passed`, clears `producer_failure`, sets `delegate_status` to `Unknown`, and steps on as `None if advanced` does.
- Beside `settle::cap`, cap the `Wait`: while the state is `Repairing` with a CI reason, `heard.delegate_running`, and the head green, `until` is at most `env.wall` plus `env.limits.client.poll`.

**The root,** `forge_route.rs`:

- `start_change`'s `Heard` fills `delegate_running` by `notes/contracts.md`'s table: a new `change_delegate_running(domain, task)`, listing every variant.
- `change_decision`: `CancelRepair { head }` makes `cancel_change_repair(domain, task, head)`, as `cancel_change_gates` does, its child the row's `Delegate::Repair`, none when it is no longer live. `RepairEnded` is `unreachable!`, as `Cancel` is, with its reason.

Each new rule's test must bite: remove `ci_reason`'s check and see the review story fail; remove the `delegate_running` check and see the run story fail.

Then: `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`; `heavy cargo clippy -p temper-engine-domain-forge-change -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-forge-world -p temper-engine-domain-world --all-targets -- -D warnings`.

**A change of meaning runs its readers' tests:** `repairing` and `released_state` change what a step decides, so run, whole, `heavy cargo nextest run -p temper-engine-domain-forge-change -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-forge-world -p temper-engine-domain-world`, and `heavy cargo check -p temper-engine-protocol-forge-world --tests` (take its name from `temper/tests/engine/protocol-forge/Cargo.toml`). Delete the status entry, commit and push.

## Report

As the implementer's text says, and: what each story failed on before the rule, whether `writer_taken` or a question's asker needed the options above, and any test of a `Repair::Ci` whose meaning you changed, with why.

## Answers

2026-11-05. May the run fix `temper/tests/engine/domain/src/tier_task_show.rs`, whose empty model table fails the required task-show on a repair's document (`ModelEffortModel`, `Bound`), or defer those assertions?

Fix it; the assertions stay. The file is added to `files`. The helper's `answer` passes `&[]` to `jig_protocol_engine_clients::translate::objects::document`, which cannot show any task naming a model, and a repair's task names model 0; a deployment's translator has its table.

`world.root.models()` alone does not compile: it is `&[jig_domain_core::Model]`, and `document` takes `&[jig_protocol_engine_clients::Model]`. Map one to the other in the helper, field for field as `client_models` does in `temper/tests/engine/domain/tests/suite/root_routes/open_chat.rs`, `provider` and `description` `None`, and pass the result. Only this helper changes: no jig change, no change to `repair_cancel.rs`'s callers.

Then the three stories' task-show assertions and their teardown run for the first time. A failure word or a missing `CI passed at head` in that output is this task's to find and fix in the rule or the cancel, not in the assertion; a change to what the assertion expects is a question.
