# ui-stop-coding-row: the plan

2026-11-07. From `inputs/request.md`: a box's stop is to cost a coding run nothing, a follow-up to `ui-box-children-2`.

## Which it was (request item 2)

Read only, by this cut and its surveys, before any row ran:

- **The Shutdown cancel doesn't reach the journal.** Once the engine has drained, serve closes the engine's worker connections before it stops the worker, and a closing connection ignores bodies (`jig-protocol-engine-workers/src/component.rs`). The core, while stopping, drops any answer that gets through (`fleet::Request::Answered` to `DropPayload`, `jig-domain-core/src/routing/children.rs`). So the run stays in flight, as after a crash.
- **But "as after a crash" is not free.** The next start settles a claim on an in-process worker as `Failed(Lost)` when its attempt committed a turn or a call (`Core::adopt`, then `children.rs`'s `fleet::Request::Lost` arm). That spends `tries.lost` (`jig-domain-tasks/src/failures.rs`), three by stock, then the task is held. Only an attempt that committed nothing goes free (`NotStarted`, `End::Refused`). Both one-process worlds pin it today: temper's `a_signal_mid_run_drains_and_the_next_start_settles_the_claims` counts one `Failed(Lost)` at the restart, and so does jig's `the_next_start_settles_the_worker_s_claims_at_once`.

So a coding run that had done any work pays a try for every stop. The row the request asks for fails on "no attempt is counted against it" for that reason, the request's "another reason", not because of the cancel's outcome or the slot or the checkout. The lowest place is jig's core, and jig's design first (`jig/AGENTS.md`: jig owns the invariant).

## The design this cut wrote

- `jig/docs/design/domain/engine.md`, 6.1, *Stopping*: a new bullet, *a run in the engine's own process*. On hearing it stops, before anything else, the core settles each agent task's claim on a host in its own process as interrupted, in one decision. Its attempt ends with no try and no backoff, keeping its committed turns, calls and saved work, and the task is due again at the next start. The fleet forgets the attempt and tells its host nothing. The writer slots are released, since nothing is placed while stopping and the next start reads every resource afresh first. A job's claim is left as it stands. A crash, or a second signal before that decision commits, leaves every such claim to the next start, so a run that brings its engine down still spends its tries. Section 6, step 3 says those left are a crash's or a job's. Its status gains the **Not yet** that `jig-stop-interrupts` closes.
- `jig/docs/design/domain/tasks.md`, 5.2: such a claim ends interrupted, as a recheck ends, with no try spent.
- `jig/docs/design/protocol/service.md`, 5, step 2 and *Runs in the engine*, and 10, *Starting and ending together*: the same, from the service's side.
- `temper/docs/design/protocol/README.md`, 8, *Starting and ending together*: a stop spends no try of a run's task, a coding run's among them, and moves none of its change's counts. The task is due again, its next attempt resuming from its committed turns and its last saved work (`domain/worker.md`, 4.2 and 5). After a crash the next start settles them, lost or not started. Its status gains a **Not yet** for 8, which `coding-stop-row` closes.

The interrupted end is jig's existing `tasks::End::Recheck` ("release the run without a try or timer"), so no stored format changes. A run interrupted this way reads, in its activity, as a run ended refused (`EndKind::Refused`), as a recheck does today.

## Parts and tasks

| Part | Task | Needs | Request item | Crates |
|---|---|---|---|---|
| `stop-interrupts` | `jig-stop-interrupts` | none | 2 | `jig-domain-core`, `jig-domain-fleet`; `jig-core-world`, `jig-one-process-world` (the rows) |
| `coding-stop` | `coding-stop-row` | `jig-stop-interrupts` | 1, 2, 3 | `temper-one-process-world` |

- **`jig-stop-interrupts`:** failing rows first. The core world's in-process restart story becomes a table by how the engine ended (crashed: a lost try; stopped: one `Recheck` terminal at the stop, no try, due, saved kept; stopped with the worker elsewhere: left to adoption). jig's one-process drain row becomes `a_drained_stop_interrupts_the_worker_s_claims_and_spends_no_try`, beside its crash rows, which still settle one claim lost. Then the core's stopping decision and the fleet's `Interrupt`.
- **`coding-stop-row`:** the request's row in temper's one-process world, beside `a_signal_mid_run_drains_and_the_next_start_settles_the_claims`, as a second row of that test made a table. A coding run's agent is held mid-turn, after a committed edit, at the first signal. After the drain and a restart on the same store, the row checks four things: the task is due again with its next attempt started, no try is spent, the change's counts and the task's saved work are as before, and its writer slot and checkout serve the new attempt. It also repairs the chat row's oracle, which waits for a lost settle that no longer comes.

The one-process world is the one that hosts a coding run's agent and restarts on one store cheaply (`world::drain_mid_run`, the same disk and store). The real-loop world hosts one on Claude Code (`fuzzy/program_landing.rs`), but its fake forge can't be carried across a restart (`forge::State::new` per run), so the request's alternative (lifecycle.rs) is not taken.

## Critical path

`jig-stop-interrupts` (about 210 implementer-minutes), then `coding-stop-row` (about 200), which builds on its merged code: about 410 implementer-minutes end to end. The coding row passes on arrival; its red is shown by the bite, the jig fix reverted locally in its slot and never committed. Between their merges the trunk's `temper-one-process-world` drain row is red by design; the second task repairs it.

## Decisions

- **The request's checks, made exact:** "no attempt counted" is the task's `tries` (jig's per-class failure counts) equal before and after, since every attempt takes a new attempt number by design (jig's `tasks.md`, 5.5). "Its change's tries" are the change row's `repairs`, `resolutions`, `updates` and `gate_failures`, since a change has no try count of its own. "Its slot" is the jig writer slot its claim holds, and the in-process worker's slot is shown by the new attempt's start. "Its checkout" is that start's prepared workspace, from the task's saved work, or from the base when it holds none.
- **Cost:** the coding row adds a coding story's drain and restart to temper's one-process focused suite, which stands over its budget already (`temper/docs/development/limits-worlds.md`: 27.9 CPU s). The coding stories already live in that suite (`an_adopted_repository_gets_its_change_pushed_through_git`), and the request puts the row beside the drain row, so it goes there, the shortest story that shows the rule, its CPU time reported. The gate judges the budget. If it rejects, the repair rule follows, not a move to fuzzy.

- **The fix is jig's, under jig's design revised,** because the cost is the restart's settle, which jig owns. temper's serve and worker change nothing.
- **At the stop, not at the restart:** the restart can't tell a drained stop from a crash without a new stored mark. Deciding at `Event::Stopping` uses records that exist, and keeps a crash's lost try, which bounds a run that brings its engine down.
- **`End::Recheck`, not a new `End::Interrupted`:** Recheck already has the meaning wanted and is stored as it is. A new variant would change jig's stored core and tasks records for a word.
- **Jobs and other processes' workers are left alone:** a job's one effect may have happened, and a remote worker's runs go on through the engine's restart and are adopted.
- **Outside this request, seen on the way:** a worker in another process that shuts down on its own cancels its runs with `Reason::Shutdown`, which jig's hub link maps to `Failed(Agent)` for a hosted run (`jig-domain-hub-link/src/payload.rs`). So a cluster worker's restart spends an agent try per run. A box's stop is one process, so this plan leaves that alone; it is the cluster's to take up.
- **No stored format, protocol or public interface changes** but the fleet's new crate-public event.

## Overlaps

None with the drafts beside this one: no other draft's files are in `jig-domain-core`, `jig-domain-fleet`, `jig/tests/core`, `jig/tests/one-process`, `jig/testing/jig-core-kit` or `temper/tests/one-process`.
