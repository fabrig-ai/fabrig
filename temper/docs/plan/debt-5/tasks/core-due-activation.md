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
