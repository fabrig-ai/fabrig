+++
id = "jig-stop-interrupts"
part = "stop-interrupts"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/routing/stopping.rs",
  "jig/crates/jig-domain-core/src/routing/mod.rs",
  "jig/crates/jig-domain-core/src/routing/children.rs",
  "jig/crates/jig-domain-fleet/src/boundary.rs",
  "jig/crates/jig-domain-fleet/src/domain.rs",
  "jig/crates/jig-domain-fleet/src/attempt.rs",
  "jig/crates/jig-domain-fleet/src/tests/**",
  "jig/testing/jig-core-kit/src/world/stopping.rs",
  "jig/tests/core/tests/suite/restart.rs",
  "jig/tests/core/tests/suite/stopping.rs",
  "jig/tests/one-process/**",
  "jig/docs/design/domain/engine.status.md",
]
effort = "high"
minutes = 210
+++

# A stop settles its in-process runs' claims as interrupted

## Goal

A box's stop spends a lost try of every run that was at work in it. When jig's core hears it is stopping, it settles nothing (`jig-domain-core/src/routing/stopping.rs`, `start`). Once the engine has drained, a run on a worker in the engine's own process is cancelled and its answer dropped, which is right. But the next start then settles its claim as `Failed(Lost)` (`routing/children.rs`, the `fleet::Request::Lost` arm, from `jig-domain-fleet/src/domain.rs`'s `Event::Adopt` with `HostKind::Engine`), since it committed a turn or a call. That spends `tries.lost`, and three stops hold the task. Build `jig/docs/design/domain/engine.md`, 6.1, as this plan's cut revised it. On `Event::Stopping`, before anything else, the core settles each **agent task's** claim on a host in its own process as interrupted. That is one `tasks::End::Recheck` terminal per claim, in one decision: no try, no backoff, its committed turns, calls and saved work kept, the task due again. The fleet forgets the attempt, telling its host nothing. A crash still leaves those claims to the next start, as now.

This task departs from the implementer's text in two ways: its first commit is rows that fail, and one status entry is yours (acceptance 5).

Acceptance:

1. **Failing rows first,** committed alone and failing on the trunk's code for the reason given:
   - **The core world** (`jig/tests/core`): `a_restart_with_workers_in_process_settles_their_claims_at_once` (`tests/suite/restart.rs`) becomes a table of rows by how the engine ended, one runner, each row its own `Setup` seed:
     - **crashed** (the row as it stands: a turn committed, a restart with `workers_in_process` and no stop): `tries.lost == 1`. It passes before and after.
     - **stopped:** a world whose configuration has `workers_in_process` from its start, a turn committed, `world.stop()`, then `restart_with` the same configuration. Before the restart, the store holds one `CoreRecord::Terminal` with `End::Recheck` for that task and attempt, committed before the drain's notice. After it, the restored task has `tries` all zero, `refusals` unchanged, `phase == Active(Due)`, `saved` as before the stop, and no writer slot naming the stopped attempt. On the trunk it fails: no terminal at the stop, `tries.lost == 1` after.
     - **stopped, worker elsewhere** (the same with `workers_in_process` false): no terminal at the stop, and the claim reaches the fleet at restart as today. It passes before and after.
   - **The one-process world** (`jig/tests/one-process`): `the_next_start_settles_the_worker_s_claims_at_once` becomes `a_drained_stop_interrupts_the_worker_s_claims_and_spends_no_try`. `outcome.claims_settled_at_restart == 0`, a new `outcome.claims_interrupted_at_stop == 1` (Recheck terminals committed in the drain phase), and a new `outcome.lost_tries == 0` (the sum of `tries.lost` over live task records at the restart's end). `a_power_loss_mid_run_restarts_and_adopts_nothing_from_the_dead_worker` keeps `claims_settled_at_restart == 1`: a crash still costs a try. On the trunk the renamed row fails on `claims_interrupted_at_stop`.
2. **The fix,** by the sketch below: the rows pass. Every other test of `jig-domain-core`, `jig-domain-fleet`, `jig-core-world` and `jig-one-process-world` passes, or has its oracle repaired where this change's correct behaviour trips it, each named.
3. **What's left alone.** A job's claim (`tasks::Executor::Job`) is not interrupted: the next start settles it as now. A claim on a worker in another process is not interrupted: it waits for adoption as now. A restored claim the start has not adopted yet (`Core::unreported_restored`, `restoring_proofs`) is left to the next start. The second signal's abort is unchanged.
4. Clippy is clean for every package changed. The focused tests of `jig-domain-core`, `jig-domain-fleet`, `jig-core-world`, `jig-one-process-world`, `jig-fleet-world`, `jig-tasks-world` and `jig-test-service` pass. `heavy cargo check --tests` compiles every other dependent of `jig-domain-core` and `jig-domain-fleet`, temper's included.
5. **Status:** the **Not yet** for 6.1 and 6 that names `ui-stop-coding-row` is gone from `jig/docs/design/domain/engine.status.md`, the rest unchanged.
6. **The readers in temper (step 6):** temper's one-process world reads the old meaning. Its `a_signal_mid_run_drains_and_the_next_start_settles_the_claims` counts `Failed(Lost)` terminals at restart and waits for one (`temper/tests/one-process/src/referee.rs`, `settled`, and `phase_complete`'s `Phase::Restart`). It goes red with this change by design, and the plan's next task, `coding-stop-row`, repairs it. Report it under `## Left` with what you saw. Change nothing in temper.

## Inputs

- `temper/docs/plan/ui-stop-coding-row/notes/common.md`, all of it.
- `temper/docs/plan/ui-stop-coding-row/inputs/request.md`.
- `jig/docs/design/domain/engine.md`, 6.1, *Stopping*, the bullet on a run in the engine's own process, and section 6, step 3, as this cut revised them; `jig/docs/design/domain/tasks.md`, 5.2 (the interrupted claim) and 5.5 (tries, `Refused` and `Recheck`); `jig/docs/design/protocol/service.md`, 5, step 2, and 10, *Starting and ending together*.
- What this cut's survey found, read only:
  - **The answer is dropped already.** Once drained, temper's serve calls `engine.workers.close()` before the worker's `stop()`, so a closing connection ignores bodies (`jig-protocol-engine-workers/src/component.rs` near 424 and 455). Any answer that still gets through is dropped by the core while stopping (`fleet::Request::Answered` to `DropPayload`, `routing/children.rs` near 429). The store stays writable until it closes last (`temper-service/src/stop.rs` near 75), so the stop's decision commits.
  - **The restart's settle** is `Core::adopt` (`routing/sibling_routes.rs` near 211). It maps a claim to `HostKind::Engine` when `workers_in_process`, and counts it `worked` for kept turns or a committed call. The fleet answers `Lost` or `NotStarted` (`jig-domain-fleet/src/domain.rs` near 543), and `children.rs` routes `Lost` to `End::Failed(Lost)` and `NotStarted` to `End::Refused`. Each goes through `remember_unpriced_terminal`, `Now::DropAssignment` and a tasks `Activation` with `Cause::Unpriced` and `saved: None`.
  - **`End::Recheck`** (`jig-domain-tasks/src/run.rs` near 558) leaves `tries` and `refusals` as they are and sets `Active(Due)`, or `Idle` when kept. `saved: None` keeps the record's `saved` (near 601). Recheck is refused for a job (`refuses_job_end`), so jobs are left alone.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours, edits allowed:**

- `jig/crates/jig-domain-core/src/**` and `jig/crates/jig-domain-fleet/src/**`: compiler-forced arms for the new fleet event and request (`vocabulary.rs`, the oracle, diagnostics), each only for that. The core's own step tests (`jig-domain-core/src/tests/**`): an oracle this change trips, repaired, named.
- `jig/testing/jig-core-kit/src/**`: a `World` helper the rows need (reading the store's terminals, a restart with the same configuration), each named. `jig/testing/jig-core-kit/src/oracle.rs`: what its model of the core must now expect at a stop, if it models the stop.
- `jig/tests/core/tests/suite/*.rs` other than the owned two: an oracle tripped, repaired, named.
- Nothing in temper, smith or skein changes.

**Options, each with its default:**

- **The fleet's surface.** Default: one new `fleet::Event::Interrupt { run: Token, attempt: Token }` that forgets the attempt: its placement, relays and grants. It sends its host nothing and answers no request, or only the `DropCall`s a relay in flight needs. The core decides which claims from its `proofs` and `workers_in_process`, as `adopt` does. If the fleet must say which attempts it held, add `fleet::Request::Interrupted { run, attempt }` instead and route it in `children.rs` beside `NotStarted`. Say which you took.
- **Where the decision is made.** Default: `stopping::start(core, env)` (`routing/stopping.rs`), from `routing/mod.rs`'s `Event::Stopping` arm. For each live proof whose task is an agent task (`Executor::Agent`) and whose host is in the process (`workers_in_process`, or `proof.host == HostKind::Engine`), it sends the fleet its `Interrupt` and then does what `children.rs`'s `NotStarted` arm does, with `tasks::End::Recheck`. `core.stopping` is set after these are queued, so the tasks child takes them. If the stopping gate (`stopping::gate`) refuses one of the internal events this needs, admit that one event, and name it.
- **A late word from the interrupted run** (its answer, a turn, its channel's loss, a relay's answer) must reach no task and trip no `expect`. `children.rs`'s `Lost` arm `expect`s a proof, and `unreported_restored` and `call_parts` assume a live attempt. Default: the fleet has forgotten the attempt, so it emits nothing for it. Where a path still reaches the core for it, drop it there as for an attempt that is not current, and add that case to the core world's stopped row.
- **The writer slots.** Default: released, as `Recheck` releases them. The next start reads every resource afresh before it places anything (`engine.md`, 6.1). If `run.rs`'s terminal admission refuses `Recheck` for a state an interrupted claim can be in (closing, held), take `End::Refused` for that state alone, and name the state.
- **A claim still preparing or claimed but not yet running** (a placement in flight): interrupted alike. Default: the same path. If the fleet holds it in a state `Interrupt` can't reach cleanly, leave that state to the next start, and name it.
- **The test service's in-process workers** (`jig-test-service`) need no change: the decision is the core's.

## Steps

1. **The rows** (acceptance 1). Run each alone, checking the count, e.g. `heavy cargo nextest run -p jig-core-world --test suite -E 'test(=restart::a_restart_with_workers_in_process_settles_their_claims_at_once)'` and `heavy cargo nextest run -p jig-one-process-world --test suite -E 'test(=one_process::a_drained_stop_interrupts_the_worker_s_claims_and_spends_no_try)'` (the binary and path may differ: find them first). See each fail for the reason given, and commit them alone ("jig: rows that fail while a stop spends its in-process runs' tries").
2. **The fix,** by the options. Sketch, inside which the choices are yours:

   ```rust
   // jig-domain-fleet/src/boundary.rs, Event
   /// The engine stops and settles this attempt as interrupted: forget it,
   /// telling its host nothing (domain/engine.md, 6.1).
   Interrupt { run: Token, attempt: Token },

   // jig-domain-core/src/routing/stopping.rs
   /// Settle each agent claim on a host in this process as interrupted, then
   /// refuse what would commit (domain/engine.md, 6.1).
   pub(crate) fn start(core: &mut Core, env: &Env<Limits>) -> Queue<Request>
   ```

   Run the rows: they pass. Then the bite: make `start` settle nothing once, see the stopped rows fail for the rule (no terminal at the stop, a lost try after), restore.
3. **The status entry** (acceptance 5).
4. **The readers,** each once, the rows alone first with their CPU time: `heavy cargo nextest run -p jig-domain-core --lib`; `-p jig-domain-fleet --lib`; `-p jig-core-world`; `-p jig-one-process-world`; `-p jig-fleet-world`; `-p jig-tasks-world`; `-p jig-test-service`. Repair what this change's correct behaviour trips (Owns and may change).
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p jig-domain-core -p jig-domain-fleet -p jig-core-world -p jig-one-process-world [-p each other package changed] --all-targets -- -D warnings`; `heavy cargo check --tests` naming the other dependents of `jig-domain-core` and `jig-domain-fleet` (`cargo metadata` lists them), `temper-engine-domain`, `temper-service` and `temper-one-process-world` among them.
6. **temper's reader, once, to report it:** `heavy cargo nextest run -p temper-one-process-world --test suite -E 'test(=one_process::a_signal_mid_run_drains_and_the_next_start_settles_the_claims)'` (the path may differ). It is expected red, the restart waiting for a lost settle that no longer comes. Quote its failure in `## Left`. Commit ("jig: a stop settles its in-process runs' claims as interrupted, spending no try"), push.

## Report

As the implementer's text says, and: each row's red at step 1 and the bite; each row's CPU time alone; the fleet surface and the decision's place as taken; each oracle repaired and why; any state left to the next start (options); temper's reader under `## Left`.
