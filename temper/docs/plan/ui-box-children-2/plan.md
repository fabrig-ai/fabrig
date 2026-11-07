# ui-box-children-2: the plan

2026-11-07. From `inputs/request.md`: `red-70`'s second repair. After-landing at main `8e15507354` named the survivor of `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`: a chat's `claude -p … --tools Read,Glob,Grep … --resume <session>`, its own group's leader at the stop, its parent other than the story, reparented to the story after it.

## What starts it, and why it's left (request item 1)

Read only, by this cut and its survey:

- **The chain** is box → `temper serve` → `temper internal agent` (serve's in-process worker, `Worker::process_spawn`, `temper-service/src/worker.rs`) → `claude` (smith's program backend, `smith-protocol-program/src/backend/start.rs`, `launch`). Each spawn goes through skein-io, so each child leads a group of its own, with no parent-death signal and no cgroup (`skein-io`'s `kernel::Spawn` has neither; cgroups are skein's **Not yet**, `io.status.md`, 6.2 to 6.4).
- **Why it's left:** on serve's first `SIGTERM`, jig's core drains without waiting for runs; once it has, `Stage::Worker` (`temper-service/src/system.rs`, near 811) calls `worker.abort()`, which aborts each agent child, and skein-io closes a child by killing its group. The agent dies before it can end its program, and `claude`, in its own group, outlives it. The coding run's agent had ended 38 s in, so only the chat's was left at the stop.
- **smith is right when asked:** the agent ends its program on a channel `Cancel`, on `SIGTERM` and on its channel's end, `SIGINT` then `SIGTERM` then a kill to `claude`'s group, and its exit waits for the program's reap. serve never asks: the design said to kill (`protocol/README.md`, 8, after jig's `protocol/service.md`, 10), on the belief that an agent's group holds its program.

So the lowest place is temper's serve, and the design above it first.

## The design this cut wrote

- `jig/docs/design/protocol/service.md`, 10, *Starting and ending together*: once the engine has drained, the one process ends its workers with no grace for their turns and answers, but each worker's hub cancels every run first, so each agent ends its program within smith's cancel, then kill; the worker is closed once no agent process is left. A second signal closes them at once, and a program an agent started outlives it until contained trees. Its status gains a **Not yet**: jig's test service still aborts its workers at once (its agents have no program, so nothing is left).
- `temper/docs/design/protocol/README.md`, 8: the same for temper's serve, and why killing an agent's group leaves its program. Its status gains the **Not yet** the task closes.
- `temper/docs/design/protocol/git.md`, 7, *At the worker's end*: the worker is closed once no agent process is left.
- `temper/docs/design/protocol/setup.md`, 7.2, *Its children*: serve's agents and programs are serve's children, ended by its first signal; a forced end leaves a program, the new Later item 15, *Agents in contained trees*. Its status's 7.2 entry now names the survivor and this plan.

## Parts and tasks

| Part | Task | Needs | Request item | Crates |
|---|---|---|---|---|
| `agents-stop` | `serve-stops-agents` | none | 1, 2, 3 | `temper-service`; `temper-real-loop-world` (the test) |

- **`serve-stops-agents`:** first a failing row beside `a_started_service_drains_on_termination` in `temper/tests/real-loop/tests/suite/lifecycle.rs`: a chat on Claude Code, the fake's turn held (`Step::Hang`), the service stopped mid-turn, checking it ends drained with the fake having seen its interrupt. Then the fix: at the drain's close, `worker.stop()` (every run cancelled) and `worker.abort()` only once no agent process is left, or on the second signal. The stop's readers' focused tests run (real-loop, one-process, system, temper's suite, temper-service's lib), since when serve ends changes. Then the two status entries go.

One task: the fix is a few lines in one crate, and its test is in the one world where serve, a real smith agent and the fake Claude share a ring (the request's item 2: "beside temper's serve stop tests"). smith's `program_children.rs` world gets no row: smith ends its program on every stop it's given, and nothing there is red.

## Critical path

`serve-stops-agents`, about 180 implementer-minutes, most of it the real-loop row and the readers' runs. The plan lands when it has merged and the trunk's end check is green; after-landing's next run of the live story is the proof the request asks for (item 3), its recorder naming any survivor.

## Decisions

- **The fix is temper's, under jig's design revised.** jig's one-process sentence is jig's to promise (`jig/AGENTS.md`), so its design changes first; jig's own code (its test service) lags harmlessly and is left, recorded **Not yet**, not built here: its agents run smith's direct backend, with no program to leave.
- **No skein, smith or jig code changes, and no skein `io`, `shell` or `world` interface changes:** hearth/core needs no heads-up.
- **A forced end still leaves a program** (the box's close of serve after its second grace, or serve's abort on a second signal): ending it needs contained trees (skein's `io.md`, 6.3 and 6.4) or a parent-death signal skein-io's spawn can ask for, both beyond this repair. It's the design's Later (`setup.md`, 15), not dropped silently. The live story meets it only if serve's stop outlasts the box's first grace (10 s), since its box then sends the second signal.
- **No deadline widens:** the drain's wait is bounded by smith's host supervision (cancel, `cancel_grace`, terminate, `kill_after`, kill) and by the box's second signal; the task reports the stock figures.
- **No stored format, protocol or public interface changes.** `Worker` gains one crate-private predicate.
- **Nothing dropped:** the request's three items are the task's. The greenfield mockup's sources bear on no part of this request.

## Overlaps

None with the drafts beside this one. `live-change-call/run-words` changes `temper-service/src/limits.rs`, `tests/limits.rs` and `tests/stock_brief.rs`, and `tui-approval/temper-config` (merged) `tests.rs`, `tests/fixtures.rs` and `tests/checks.rs`: this task owns none of them, and its `system.rs`, `worker.rs`, `stop.rs` and `tests/stop.rs` are no other draft's.
