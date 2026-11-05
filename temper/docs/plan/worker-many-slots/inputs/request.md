# worker-many-slots: a worker can start with more than one slot

Urgent, for a comparison waiting on it: a real worker refuses to start with `workers[0].slots` above 1, on any profile, and nothing in `agent.json` raises it:

```
workers[0].slots: must agree with the agent's agents
```

## The route, from reading (confirm it)

- `smith/crates/smith-domain-run/src/limits.rs`, `derive`: `runs: 1`, always. An agent process holds one run, and that's right.
- `smith/crates/smith-local-service/src/profile.rs`, `host_limits`: `agents: run.runs`. So smith's host is sized for one agent.
- `temper/crates/temper/src/agent.rs`, `read`: takes those limits as they are.
- `temper/crates/temper-worker-domain/src/limits.rs`, `unfit`: `host_limits.slots <= agent_limits.agents`, `Unfit::Agents`, named in `temper-service/src/startup/checks.rs`.
- The worker hosts an agent per slot (`temper-worker-domain/src/agents.rs`; smith-host-domain's `agents` slab). `temper/tests/worker/domain` runs three slots, from a fixture's own numbers, so nothing has gone through a real start.

## The change

The number of agents a host holds at once follows the worker's slots, where temper composes them. A run still holds one agent's work. Take the fewest lines that are right, and not a hack: this is how a worker was meant to work, by `temper/docs/design/protocol/worker.md`. Say in the report which of smith's or temper's limits scale with that number (`alarms`, an outbox, a bound of memory), and check those scale too: a limit sized for one agent that fails at the second would only move the fault. The other rule in `unfit`, `Unfit::Workspaces`, stays. Don't change smith's `runs: 1`. A change in smith's public surface is fine if it's a small one, skein first if the mechanism's missing; a protocol change is a `Question:`.

## What must rise together

`temper help server` (`temper/docs/help/server.md`) and `temper/docs/help/reference/configuration.md` say what a person must raise to run more slots: `workers[0].slots`, `checkout.workspaces`, `hub.slots`, `fleet.slots`, `git_component.operations`, and whatever else a start requires. An operator found these by trial. Each start's refusal should name what to raise, and most do: check that the words say so at each one. Help pages have a byte bound: `change.md` and `task.md` are at theirs; read which applies.

## Tests

A failing test first, through the real start: a deployment's documents with `workers[0].slots` 8 and 10 workspaces starts, in the tier that runs `temper`'s real settings check (`temper/crates/temper/tests/suite`, by `serve_prepares_once_on_the_owned_example`'s example, or lower if a lower tier reads the real `host_limits`). And two agents at once through the real host, in the lowest tier that uses the real `host_limits`, not a fixture's numbers. Memory: a start's memory bound with 8 slots is a real question: say what it comes to under `standard`, and if the deployment's `engine.memory_bytes` would refuse, say by how much.

## Size

One task. `temper/docs/design/draft/dogfood-hacks.md` has 2.10 with those counts: no new entry is needed, as this is no hack. fabrig/ui has priority: leave its files alone. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). Formats change in place before v0.1.0.

## Walk

`walk = false`: the next deployment at 8 and 10 is the proof.
