# worker-many-slots: a worker can start with more than one slot

The request (`inputs/request.md`) names the route; reading confirms it. `smith-domain-run`'s `derive` has `runs: 1`, `smith-local-service`'s `host_limits` has `agents: run.runs`, `temper/src/agent.rs` takes that as it is, and `temper-worker-domain`'s `unfit` refuses `slots > agents`. The request's `temper/docs/design/protocol/worker.md` doesn't exist: what it means is `temper/docs/design/domain/worker.md` and `temper/docs/design/protocol/README.md`, 15.6 and 15.7, both of which this cut changes.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `many-slots` | `many-slots` | | the host's agents follow the slots in `root_limits`; each start's refusal names what to raise; the help says it; tests through the real start and the real `host_limits` |

One task, by the request. It is the whole critical path.

## What was decided

- **Compose in `root_limits`,** in one line, as `tooling(slots)` already does. Nothing in `smith/` changes, so `runs: 1` and `host_limits` stay, and `agent.json` raises nothing.
- **`Unfit::Agents` leaves `unfit` for `valid`.** A start can't break it now, and its words named nothing a person could raise; worlds that build `Limits` by hand still answer to it, the idiom `limits.tooling.jobs` has.
- **A new start refusal, for `fleet.slots`.** Reading found no start compares them: `jig-domain-fleet/src/channel.rs` uses a hello's slots up to `fleet.slots`, so 8 slots against the owned example's 2 would start and silently run 2. Where engine and worker share a start, it now refuses. An engine alone can't see them and stays as jig designed.
- **`workers[0].slots: past its bound` names `hub.slots`,** through one new `jig-protocol-config` variant, since jig's parse is where it's found and temper can't reword it from outside. That crosses trees in one branch, by the request's "one task"; the task says so.
- **No protocol change, and none seems needed.** The link already sizes by `worker.slots`, and the engine by `fleet.slots`.
- **Tiers:** the real start in `temper/crates/temper/tests/suite`; two agents in `temper-service`'s lib tests, the lowest that reads the real `host_limits`, with `temper/tests/one-process` as the pre-approved fallback.
- **Help:** `server.md` is exactly at its walk's bound of 12,288 bytes, so what it gains it takes back. `reference/configuration.md` is generated, so its words go in `startup/reference.rs`. `change.md` and `task.md` are concepts and aren't touched.

## Overlaps

None with the early work of the drafts beside this one, on what's still open. `temper/docs/design/protocol/configuration.status.md` belongs to `ui-away/away-proof` and could conflict, so this plan writes its contract in `temper/docs/design/domain/worker.md`, 6, with its status in `worker.status.md`, and leaves `configuration.md` as it is. The drafts that touched `startup/checks.rs` have merged.

## Left out

- `temper/docs/design/draft/dogfood-hacks.md`, 2.10 needs no entry, by the request.
- The cluster's workers stay at one slot (`temper/docs/design/protocol/README.md`, 15.7).
- `walk = false`: the next deployment at 8 and 10 is the proof.
