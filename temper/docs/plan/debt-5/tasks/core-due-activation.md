+++
id = "core-due-activation"
part = "jig-core-steps"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/routing/**",
  "jig/crates/jig-domain-core/src/vocabulary.rs",
  "jig/crates/jig-domain-core/src/tests/**",
  "jig/examples/ops/domain/src/route.rs",
  "jig/testing/jig-test-domain/src/**",
  "temper/crates/temper-engine-domain/src/route/now.rs",
  "temper/docs/design/domain/root.md",
  "temper/tests/engine/domain/src/escalation.rs",
  "temper/tests/engine/domain/src/forge_routes.rs",
  "temper/tests/engine/domain/tests/suite/escalation.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
]
effort = "high"
minutes = 100
+++

# jig D001: a task that comes due waits in the core

## Goal

`Now::Activate` goes. A task that comes due enters the core's due set, and every root takes it through the `take_due` path it already has, so `Now::Activate`'s arm leaves all three roots. D001 stays open.

Acceptance:

- `rg -n 'Now::Activate' jig temper --type rust` reads nothing.
- `rg -c 'core::Now::' jig/examples/ops/domain/src/route.rs` prints 39.
- `lifecycle::activate` holds no `out`, and a `Person` task, a recurring template and a task activated while stopping do not enter `due`.
- A new step test in `jig-domain-core` covers: a due agent task waits in `due` until `take_due`; a `Person` task and a recurring template are not kept; nothing enters `due` while stopping.
- `temper/docs/design/domain/root.md`, 4.1 no longer lists an activation that comes ready of itself.
- What `temper/docs/plan/debt-5/notes/core-steps.md` calls `What runs` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/core-steps.md`, `AGENTS.md`, `jig/AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/development/debt/D001-roots-sequence-core-steps.md`, whole.
- `jig/docs/design/domain/root.md`, 3; `jig/docs/design/README.md`, 6.3 and 6.4; `temper/docs/design/domain/root.md`, 4.1.

## Owns and may change

- Owns: `jig-domain-core`'s `routing/task/lifecycle.rs`, `routing/task.rs`, `vocabulary.rs` and its tests.
- Removing the variant, and only its arm or its line in each list: `jig/examples/ops/domain/src/route.rs`, `jig/testing/jig-test-domain/src/routes.rs`, `temper/crates/temper-engine-domain/src/route/now.rs`, `jig/crates/jig-protocol-store/src/ended.rs`, `jig/testing/jig-test-service/src/route.rs`, `jig/testing/jig-test-service/src/clients/translation.rs`, `jig/testing/jig-test-store/src/route.rs`, `jig/tests/client/src/joint/engine.rs`, `jig/tests/core/src/client_peer.rs`, `jig/tests/ops/src/joint/engine.rs`, `jig-domain-core/src/tests/runs/completion_retry.rs`, `temper/crates/temper-service/src/clients/translation.rs` if it lists the variant.
- This departs from a branch staying in its tree: the variant's removal is one atomic change, and those lines of temper are named here.
- Tests of `jig-domain-core`, `jig-test-domain` and temper's root that saw `Now::Activate`: as the note's pre-approved list says. At base they are `tests/jobs.rs`, `tests/runs/notes_room.rs`, `tests/start/runs.rs`, `tests/windows.rs`, `tests/brief_documents.rs` and `jig-test-domain/src/tests/link.rs`.
- The four world files in `files`, and any other fixture of a `What runs` package, as `## Answers` below says.
- Design: `temper/docs/design/domain/root.md`, 4.1, the words named in step 4, nothing else. `jig/docs/design/domain/root.status.md` stays: `core-prepare-agent` closes it.
- `Event::Activate` keeps `context` and `ready`.

## Steps

1. `routing/task/lifecycle.rs`:

   ```rust
   /// A task that comes due waits for the root's `take_due` (domain/root.md, 3).
   pub(crate) fn activate(core: &mut Core, context: Box<tasks::RunContext>) {
       if core.pending_note_briefs.get(&context.task) == Some(&crate::routing::NoteBrief::Activation) {
           let _pending = core.pending_note_briefs.remove(&context.task);
           crate::routing::children::remember_due(core, context);
           return;
       }
       if core.stopping() || core.tasks.recurring_template(context.task).is_some() {
           return;
       }
       match context.executor {
           tasks::Executor::Person(_) => {}
           tasks::Executor::Job { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Agent { .. } => {
               crate::routing::children::remember_due(core, context);
           }
       }
   }
   ```

   `routing/task.rs`'s call follows.

2. `vocabulary.rs`: the variant goes.

3. The arms and lists; then the tests, by the note's pre-approved list, and the new step test.

4. `temper/docs/design/domain/root.md`, 4.1: `` `Resume`'s second choice (it goes on to the fleet's progress), an activation that comes ready of itself, and one queued as the root's work `` becomes `` `Resume`'s second choice (it goes on to the fleet's progress) and one queued as the root's work ``.

5. Read `temper/crates/temper-engine-domain/src/domain.rs`'s `Resume` pass, to confirm the note's wake: a task made due by a decision must activate after that decision commits with no further outside event. If it does not, stop and ask.

6. Run what the note names; commit (`D001: a task that comes due waits in the core`, saying why) and push.

## Report

The measure; each story's count that moved, old and new; any story whose order moved; the lines added and removed.

## Answers

2026-11-04. Retime the two failing temper world fixtures, or revise the later-decision activation? Retime; the contract stays. An activation runs in a later decision than the one that made its task due (`temper/docs/plan/debt-5/notes/core-steps.md`, "What this changes, on purpose"), and no step of this task goes back on it. These two stories set their race and their cut by the same-decision activation, which is what the item removes; `core-steps.md` now pre-approves that retiming, under the conditions below. Commit `3619c0ac46` and `1197f80a70` stand: continue from them, and do not redo that work.

1. `escalation::held_and_decision_commit_cuts_restore_paged_real_children_and_exact_scripts`. The claim no longer commits with the release, so `Cut::Decision` cannot cut it. Add one `Cut` variant in `temper/tests/engine/domain/src/escalation.rs`, say `Cut::Claim`: lose completion after the first commit that saves a fresh `RunProof` (`turn` and `terminal` both `None`) once a store row holds an `EscalationDecision`, before its assignment escapes. The first assignment's claim precedes every decision and must not trigger it. Handle it at both places `Cut::Decision` is handled (`apply_store`'s `decided` and `observe`'s `decision_cut`). In the test, only the third world's `cut` changes to it. `unplaced_claims() == 1` stays, as do the observer and the three corruptions; drop `with release` from the assertion's message, and make the comment `A decision can commit its new claim` say a claim. The two-cut loop above it keeps `Cut::Decision`, as do `suite/memory.rs` and `suite/fuzzy/escalation.rs`; the fuzzy matrix gains no cut. If `unplaced_claims()` is still not 1 at that cut, that is a question.

2. `forge_routes::a_merge_asked_while_kept_is_refused_naming_the_pending`. The cause is not known yet: read a trace first, with `c`'s phase, `procedure_due(c)` and the change's state, from the amendment until the 80 ticks end. `change_decision` (`temper-engine-domain/src/route/forge_route.rs`) returns with no step and no refusal when `procedure_due` is `None`, so look there and at `take_due`'s still-due check. Then:
   - If the merge is only asked at a different time than the story's pause expects (`c`'s activation is now in a later decision, so the held reads or the amendment fall on the other side of it), retime the story so the merge is asked after the amendment commits. Change only the pause condition, the tick loops, or the hold in `forge_routes.rs`. Every assertion stays: `merges == 0`, the `StepRefused` naming `e`, its rendered text, the phase, and the rest of the story through `Met` to landing.
   - If a step is lost, a wake is dropped, or `c` could stay parked with no later retry, that is a fault of this change and not of the fixture. It may be fixed in the files this task owns, with a step test in the lowest tier that shows it. A fix that needs other production code is a question: stop and name the code.
   - If no timing can make a kept change ask for its merge, the refusal has become unreachable, and that is a question. Do not delete it.

3. Your last run left 638 tests of the fourth command unobserved. Run the whole of `What runs` to the end with `--no-fail-fast`, and read every failure before changing anything. Another fixture in a `What runs` package that set a race or a cut by the same-decision activation is treated like the two above, without asking: assertions stay, and the report lists each one. A count moves only as the note says. A story whose visible order or outcome changes for any other reason is still a question. A memory ceiling or a test budget that goes red is the user's: report the measurements and raise nothing.

Report the root cause of (2) and which case held.
