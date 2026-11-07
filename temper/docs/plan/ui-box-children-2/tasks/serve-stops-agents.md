+++
id = "serve-stops-agents"
part = "agents-stop"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/worker.rs",
  "temper/crates/temper-service/src/stop.rs",
  "temper/crates/temper-service/src/tests/stop.rs",
  "temper/tests/real-loop/tests/suite/lifecycle.rs",
  "temper/tests/real-loop/src/claude.rs",
  "temper/docs/design/protocol/README.status.md",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 180
+++

# serve stops its agents before it closes its worker

## Goal

After-landing's `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` (`temper/crates/temper/tests/live.rs`) failed at main `8e15507354` on `no surviving children`: a chat's `claude -p … --tools Read,Glob,Grep … --resume <session>`, its own group's leader at the stop, its parent a `temper internal agent`, was reparented to the story after it. This cut found why, read only: on serve's first `SIGTERM`, once the engine has drained (jig's core drains without waiting for runs), `Stage::Worker` in `temper-service/src/system.rs` (near 811) calls `worker.abort()`. That aborts every agent child, which skein-io closes by killing the agent's group (skein's `io.md`, 6). The agent never gets to end its program, and `claude`, spawned by smith's program backend in a group of its own (`smith/crates/smith-protocol-program/src/backend/start.rs`, `launch`), outlives it. Make serve's drain stop the worker (each run cancelled, each agent ending its program as smith's cancel does) and close it only once no agent process is left, as the revised `temper/docs/design/protocol/README.md`, 8 says.

This task departs from the implementer's text in two ways: its first fix commit is a test that fails, and two status entries are yours (acceptance 5).

Acceptance:

1. **A failing row first.** In the real-loop world's focused suite, a row beside `a_started_service_drains_on_termination` (`temper/tests/real-loop/tests/suite/lifecycle.rs`) runs a chat on Claude Code whose fake program holds its turn (`smith_fake_claude::Step::Hang`), stops the service while that turn is held, and checks that the service ends `End::Drained` with the fake Claude having seen its interrupt (`smith_fake_claude::Seen::Interrupted`, from the source's `fake-claude.observations`), so the agent's cancel reached the program before serve ended. It's committed failing on the trunk's code, for that reason (no interrupt seen), before the fix.
2. **The fix.** Once the engine has drained, the service calls `worker.stop()` (the worker domain's `Event::Shutdown`, which cancels every run, `temper-worker-domain/src/route.rs`) and keeps the worker iterating; it calls `worker.abort()` only once the worker owns no agent process, a spawn in flight among them, or once the engine has ended (a second signal, `Service::shutdown`, unchanged). serve still exits only once the worker is empty. The row passes; every other test the change reaches passes.
3. **Nothing waits longer than smith says.** No limit, grace or deadline changes: the wait is bounded by smith's host supervision (a cancel, then past `cancel_grace` a terminate, then past `kill_after` a kill; smith's `domain/host.md`, 4) and by the box's second signal. The report gives the stock figures that bound it: temper's agent `cancel_grace` and `kill_after`, and the agent's program grace (`limits.io.close_timeout`), as the configuration gives them, and says whether the program's stop fits inside the host's cancel grace.
4. Clippy is clean for every package changed; the focused tests of `temper-service`, `temper-real-loop-world`, `temper-one-process-world` and `temper-system-world`, and `temper`'s `suite` binary, pass (step 5: the change is one of meaning, when serve's stop ends); `heavy cargo check --tests` compiles every other dependent of `temper-service` (step 6).
5. **Status:** the **Not yet** for 8 that names `ui-box-children-2` is gone from `temper/docs/design/protocol/README.status.md`, and the **Not yet** for 7.2, *Its children*, that names it is gone from `temper/docs/design/protocol/setup.status.md`, the rest of each file unchanged. What a forced end still leaves is the design's Later (`setup.md`, 15, *Agents in contained trees*), no gap.

## Inputs

- `temper/docs/plan/ui-box-children-2/notes/common.md`, all of it.
- `temper/docs/plan/ui-box-children-2/inputs/request.md`.
- `temper/docs/design/protocol/README.md`, 8, *Agents and git as child processes* and *Starting and ending together*, as this cut revised them; jig's `jig/docs/design/protocol/service.md`, 10, *Starting and ending together*, and 6, *Ending*.
- `temper/docs/design/protocol/setup.md`, 7.2, *Its children*, and 15, *Agents in contained trees*.
- `smith/docs/design/protocol/programs.md`, 2 (*The child is inside the run's containment*) and 5.7 (*A cancel*); `smith/docs/design/domain/host.md`, 4, *Cancel, then kill*.
- `skein/docs/design/io.md`, 6, *A child is closed* and *A group is not a tree*.
- `temper/docs/design/domain/testing.md`, 2.4: the real-loop world.
- What this cut's survey found, read only: the chain is box → serve → `temper internal agent` (spawned by `Worker::process_spawn`, `temper-service/src/worker.rs` near 697, in its own group, no parent-death signal) → `claude` (the agent's child, in its own group). The agent ends its program on a channel `Cancel`, on `SIGTERM` (`smith-agent-service/src/service/teardown.rs`, `shutdown`) and on its channel's end, and its exit waits for the program's reap (`service/program.rs`, `wait_for_close`); the program backend sends `claude`'s group `SIGINT`, then past its grace `SIGTERM`, then kills it (`backend/stop.rs`, `backend/stopping.rs`). `Worker::stop` (near 221) sends the git component its `Shutdown` and the worker domain `Event::Shutdown`; `Worker::abort` (near 229) drops domain work and aborts every owned io entity, and a spawn that lands after it is aborted at once (near 384). `Worker::closed` and `is_empty` (near 944) already wait for `processes` to empty.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours, edits allowed:**

- `temper/tests/real-loop/src/lib.rs` and other `temper/tests/real-loop/src/*.rs`: a re-export or a small helper the row needs (reading the source's observations; a wait until the fake's turn is held), each named in the report. The world's referee and judge change only as an option below says.
- Other tests that stop serve with a run at work (`temper/tests/real-loop/tests/suite/*.rs` with `ends_leaving`, `temper/tests/one-process/**`, `temper/tests/system/**`, `temper/crates/temper/tests/suite/*.rs`, `temper/crates/temper-service/src/tests/**`): an oracle your change's correct behaviour trips (an extra cancel seen, an agent's end now polite where it was killed, a count), repaired keeping what it checks for everything else, each named.
- Nothing in skein, smith or jig changes.

**Options, each with its default:**

- **Where the row lives.** Default: `a_started_service_drains_on_termination` becomes a table of two rows, the chat it runs now and the held Claude Code turn, one runner and one check per row. If the two can't share a runner (the Claude row needs `run_forge_on` and a scratch, as `suite/model_check.rs` has, where the chat row uses `run`), keep the chat test as it is and add the held turn as a second test in `lifecycle.rs`, and say why in the report.
- **The held turn's script.** Default: copy the sign-in and the chat's start from `temper/tests/real-loop/tests/suite/fuzzy/program_landing.rs` (`claude::sign_in()`, `claude::source(&scratch, …, &story)`), the story `Story { turns: vec![Turn::new(vec![Step::Hang])], ..Default::default() }`; wait until the run is at work with the fake's turn begun (a `--until` its siblings use, or the fake's invocation in its observations), then `ends_leaving(&[Leftover { task: <the chat>, may_be: &[Left::HostedActive, Left::Running], why: "the held Claude Code turn the stop ends (protocol/README.md, 8)" }])`, widening `may_be` to the states the run can be in at that moment if one more shows, and saying which.
- **What the row checks.** Default: `ran.served() == Some(&End::Drained)` and `Seen::Interrupted` among `smith_fake_claude::seen(…)` of the source's `fake-claude.observations`. If the world also shows whether the hosted fake was left running at the service's end, check that it wasn't, too. If the fake's turn can't be held through the world's stop (the referee ends hosted children itself before the service's stop), make that the world's one change: it leaves a hosted program to the service's stop when a declared leftover is `HostedActive`, and name it.
- **A row the trunk passes.** If the row passes on the trunk's code, read why before anything else (did the stop come before the agent spawned the fake? did the world's stop never reach `drain.closing`?) and fix the row so it holds the turn at the stop; if the trunk does end the program there after all, stop with `Question:` and what you saw.
- **A step test beside it.** Default: none. Add a case to `temper-service/src/tests/stop.rs` only if the real-loop row can't show the rule (say why): the service, its worker owning one agent child, drained to `closing`, sends that child no `io::Request::Abort` and the worker domain its `Shutdown`, and aborts it once the agent's exit is seen.
- **A late spawn.** An agent whose spawn is in flight at the stop is held in `processes` and so waited on. If smith's host domain leaves it running uncancelled, so the service waits until the second signal, report it under `## Left` with the evidence, and change nothing in smith.
- **The worker's other parts.** Git invocations, jobs and source reads still end at `worker.abort()`, as io closes them (`git.md`, 7). If `worker.stop()`'s git `Shutdown` ends an agent's git work earlier than the agent's own cancel would, keep it; it's no change of what the stop leaves.

## Steps

1. **The row,** by the options, in `temper/tests/real-loop/tests/suite/lifecycle.rs`; run it alone (`heavy cargo nextest run -p temper-real-loop-world --test suite -E 'test(=lifecycle::a_started_service_drains_on_termination)'`, checking the count, since the path may differ), see it fail for no `Interrupted`, and commit it alone ("temper: a test that fails while serve kills its agents mid-turn").
2. **The fix,** in `temper-service`. Sketch, inside which the choices are yours:

   ```rust
   // worker.rs
   /// Whether every agent process the worker spawned has closed, one still
   /// spawning among them (protocol/README.md, section 8).
   pub(crate) fn agents_ended(&self) -> bool

   // system.rs, Stage::Worker
   if let Some(engine) = &service.engine {
       if engine.end.is_some() {
           worker.abort();
       } else if drain.closing {
           worker.stop();
           if worker.agents_ended() {
               worker.abort();
           }
       }
   }
   ```

   The worker keeps iterating while it stops (the `runs` gate), and nothing it sends up after the engine's link has closed reaches the engine's journal. Run the row (it passes), then the bite: put `worker.abort()` back at `drain.closing` once, see the row fail for no `Interrupted`, restore.
3. **The figures** for acceptance 3, read from configuration and limits, not measured.
4. **The status entries** (acceptance 5).
5. **The stop's readers,** each once, the row alone first with its CPU time: `heavy cargo nextest run -p temper-service --lib`; `-p temper-real-loop-world --test suite`; `-p temper-one-process-world`; `-p temper-system-world`; `-p temper --test suite`. Repair what your change's correct behaviour trips (Owns and may change).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p temper-service -p temper-real-loop-world [-p each other package changed] --all-targets -- -D warnings`; `heavy cargo check --tests -p temper -p temper-fake -p temper-playground -p temper-client-browser-world -p temper-client-world -p temper-playground-world`, the rest of `temper-service`'s dependents. Commit ("temper: serve stops its agents, which end their programs, before it closes its worker"), push.

## Report

As the implementer's text says, and: the row's red (the failure seen at step 1), the bite, the row's CPU time alone; the figures of acceptance 3; each oracle repaired and why; the option taken where a default didn't hold; anything under `## Left`.

## Answers

2026-11-07 (temper/ui). Added before the task starts, at temper/migration's ask: a stop must not cost a coding run. Add a second row in the same world, as a row of the same table test where one fits. A coding run's task has its agent mid-turn at the stop. After the drain, and a restart on the same store, the task is due again as after a crash: no attempt counted against it, its saved work kept (temper's domain/worker.md, 12), and its slot and checkout as the restart expects. If the Shutdown cancel reaches the engine's journal as a cancelled run or a failed attempt, that is this fix's fault. Fix it at its lowest place: keep the cancel's outcome out of the journal, since the engine has drained, or record the run as interrupted. Say in the report which it was. If the world can't restart on one store cheaply, check instead that nothing the cancel sends reaches the journal after the drain, which leaves the run in flight as after a crash, and say so. Run each row alone, as the first.
