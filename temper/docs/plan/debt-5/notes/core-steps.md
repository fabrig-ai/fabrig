# debt-5: what jig's core takes back from its roots

Binding for the `jig-core-steps` part. The design is `jig/docs/design/domain/root.md`, 3 ("A task that comes
due waits in the core"); the item is `jig/docs/development/debt/D001-roots-sequence-core-steps.md`.

## What the cut found

The item says the three roots write the same arms. They do not. Each arm at this plan's base:

| `Now` | ops | test root | temper | Verdict |
|---|---|---|---|---|
| `Activate` | `domain.ready()` | `domain.ready()` | `ready()` and `within_budget` | goes: all three roots already gate `take_due` |
| `PrepareAgent` | a fixed token | a fixed token | takes a read slot, can answer `busy` | goes: the slot moves to `StartPreparation` |
| `BriefCorePlanned` | a plan from its limits | the same | binds documents, home access, forge sections; may answer not ready | stays |
| `CompleteBrief` | keeps the brief | keeps the brief | takes forge sections, counts bytes | stays: a payload to keep |
| `WorkspaceRequest` | all empty | real checks, credentials | a real workspace | stays |

The three that stay are the root's by the design: `jig/docs/design/domain/engine.md`, 7.1 and 9;
`jig/docs/design/domain/root.md`, 7; `temper/docs/design/domain/root.md`, 7. Removing them is a design change
and the user's call: `plan.toml`'s question `d001-scope`. So this plan pays the item in part, and **D001
stays open**: the item's Paid-when does not hold for ops's and the test root's fixed answers to
`BriefCorePlanned` and ops's to `WorkspaceRequest`.

The measure reads 40 at base, not the item's 39: 41 arms, one line of `route.rs` holding two. It ends
at 38.

## Why the due set and not `Work`

The item names crate-private `Work`. It does not fit: `lifecycle::activate` runs where no `Work` queue
exists (`tag_task_outputs`, the `Timer::Tasks` path), and the fleet and account paths assert that no
sibling appears. And `Now::Activate` carries a root's answer, temper's byte budget, which `Work` cannot
ask. The due set is the core's own, and it already holds a task the root is not ready for; all three
roots already gate `take_due` on what `ready` gives them.

## What this changes, on purpose

An activation leaves the decision that made the task due and runs in a later one, one per idle pass,
in `due_before` order. The item's step is the core's now; that is the change the item asks for. The
tasks must keep everything else:

- `take_due`'s `context_ready` answers true for `Hold`, so a held task is still held at activation.
- A `Person` task never passes `context_ready`, and `Core::quiescent` needs `due` empty, so it never
  enters `due`. A recurring template stays out too: today's activation does nothing with either.
- While stopping, nothing new enters `due`, as `stopping::gate` drops `Event::Activate` today.
- A restart's activations already go this way. `Event::Tasks(Restored)` just stops passing through
  `Now::Activate` first.
- ops and the test root park on `due_ready`. temper's `Resume` loop re-tries until `settled`, and
  every store answer clears `settled`, so a due task wakes after its decision commits.

`Event::Activate { context, ready }` keeps its shape: `take_due`'s callers still send it.

## What runs

Each task, a change of meaning, runs its readers' tests:

```
heavy cargo nextest run -p jig-domain-core -p jig-protocol-store -p jig-test-domain -p jig-test-service -p jig-test-store -p jig-ops-domain
heavy cargo nextest run -p jig-core-world -p jig-ops-world -p jig-client-world -p jig-client-protocol-world
heavy cargo nextest run -p temper-engine-domain -p temper-service
heavy cargo nextest run -p temper-engine-domain-world -p temper-conformance-world -p temper-system-world
heavy cargo clippy -p jig-domain-core -p jig-ops-domain -p jig-test-domain -p temper-engine-domain --all-targets -- -D warnings
```

Each count is above zero.

Pre-approved, each with its default:

- A test that saw `Now::Activate` or `Now::PrepareAgent` takes the next step as a root would
  (`core.take_due()`, then `Event::Activate`), and keeps its assertions about what follows.
- A story that only counts decisions, passes or steps takes the new count, listed in the report with
  the old one.
- If a story's **order of visible events or its outcome** changes, stop and ask: that is more than the
  item asked for.
- Over budget is red, as ever; a new cost here goes to the report with measurements.
