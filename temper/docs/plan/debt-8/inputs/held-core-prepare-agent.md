+++
id = "core-prepare-agent"
part = "jig-core-steps"
kind = "feature"
needs = ["core-due-activation"]
files = [
  "jig/crates/jig-domain-core/src/routing/run/preparation.rs",
  "jig/crates/jig-domain-core/src/routing/mod.rs",
  "jig/crates/jig-domain-core/src/routing/stopping.rs",
  "jig/crates/jig-domain-core/src/vocabulary.rs",
  "jig/crates/jig-domain-core/src/tests/**",
  "jig/examples/ops/domain/src/route.rs",
  "jig/testing/jig-test-domain/src/routes.rs",
  "temper/crates/temper-engine-domain/src/route/now.rs",
  "temper/crates/temper-engine-domain/src/domain.rs",
  "jig/docs/design/domain/root.status.md",
  "jig/docs/development/debt/D001-roots-sequence-core-steps.md",
  "jig/docs/development/debt/README.md",
]
effort = "high"
minutes = 80
+++

# jig D001: the core prepares an agent without asking its root

## Goal

`Now::PrepareAgent` and `Event::PreparedAgent` go: `route_activate` continues into `route_prepared_agent` itself. A root's transcript load needs no waiter from the core, and temper takes its read slot where it loads, keeping a full table from starting an activation. D001 stays open, its text brought up to date.

Acceptance:

- `rg -n 'Now::PrepareAgent|Event::PreparedAgent|transcript_waiter|load_transcript' jig temper --type rust` reads nothing.
- `rg -c 'core::Now::' jig/examples/ops/domain/src/route.rs` prints 38.
- `temper-engine-domain/src/route/now.rs` has no `unreachable!` in `start_preparation`.
- A new step test in `temper-engine-domain`: with `archive_reads` full, `Resume` takes no due activation and the task stays due; once a read retires, it activates.
- `jig/docs/design/domain/root.status.md` has no entry for 3 and its date is today's.
- D001's file holds the state below, and `tools/citations/citations.py --all` reads 0.
- What `temper/docs/plan/debt-5/notes/core-steps.md` calls `What runs` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/core-steps.md`, `AGENTS.md`, `jig/AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/development/debt/D001-roots-sequence-core-steps.md`, whole.
- `jig/docs/design/domain/root.md`, 3.

## Owns and may change

- Owns: `jig-domain-core`'s `routing/run/preparation.rs`, `routing/mod.rs`, `routing/stopping.rs`, `vocabulary.rs` and its tests.
- `routing/mod.rs` is also changed on `fabrig-scale`'s trunk, one line of the `AnswerPayload` arm beside this one.
- Removing the variants, only their arm or line: the list in `tasks/core-due-activation.md`'s `Owns and may change`, and `jig/tests/client-protocol/src/engine.rs`.
- `Now::StartPreparation` loses its waiter: `jig/testing/jig-test-store-files/src/tests/routes.rs`, and each reader, that field only. ops's and the test root's stores ignore it.
- temper's root: `route/now.rs` and `domain.rs`, as sketched. This departs from a branch staying in its tree, for the same reason as the last task.
- `jig/docs/design/domain/root.status.md`: the entry for 3 and the date line. `jig/docs/development/debt/D001-roots-sequence-core-steps.md` and its row's title, as step 5 says.
- Do not touch `D004` or `D002`: they read as they do, and a later plan refreshes them.

## Steps

1. `routing/run/preparation.rs`: `route_activate`'s `Allow` arm returns `route_prepared_agent(core, work, context)`, with no `load_transcript`. It agrees with `from.is_some()`: `route_prepared_agent`'s fresh-window reset and `runs/state.rs`'s window give `from`. Check that against both before removing it, and if they can differ, stop and ask. `route_prepared_agent` loses `transcript_waiter`, `busy` and its busy branch, and its phase guard stays. Emit:

   ```rust
   Now::StartPreparation { task: number, from }   // from: (u64, u32), once `from.is_some()` is known
   ```

   Keep `from` an `Option` only if a reader needs that.

2. `vocabulary.rs`, `routing/mod.rs`, `routing/stopping.rs`: the variants go, and the event's arm.

3. temper, `route/now.rs`:

   ```rust
   fn start_preparation(domain, env, decision, task, from) {
       let waiter = domain.archive_reads.insert(Some(Read::Transcript { task }))
           .expect("Resume takes an activation only with read room")
           .token();
       let range = Range::Core(crate::CoreRange::TaskTranscript { task, from });
       emit(decision, &env.limits, Delivery::Load { waiter, range, after: None });
   }
   ```

   `domain.rs`'s `Resume` pass, beside `within_budget`: the activation also needs room in `archive_reads`. This pass is the only way into `route_activate` in temper, and `domain.work` is empty when it runs, so no other read takes the room first. While the table is full, jobs and agents with no transcript wait too, briefly: accepted. The arm for `Now::PrepareAgent` goes.

4. ops and the test root: their arms go, and each `StartPreparation` arm keeps its form.

5. D001's file. Its **Status** stays open and its **Paid when** stays word for word: the question `d001-scope` in `temper/docs/plan/debt-5/plan.toml` is the user's. Its **Measure** reads `38 at <your base's commit>`; add a **Checked** line for today; `## What`'s list drops `Now::Activate` and `Now::PrepareAgent`, and it says what `temper/docs/plan/debt-5/notes/core-steps.md`'s table says of the three left: temper does real work in each, and the fixed answers left are ops's and the test root's. No row changes in `jig/docs/development/debt/README.md` unless its title stops being true.

6. Run what the note names; commit (`D001: the core prepares an agent without asking its root`, saying why; D001's text and the status in a second commit) and push.

## Report

The measure; each story's count that moved, old and new; whether `load_transcript` agreed with `from` by reading; the lines added and removed across this part's two tasks.
