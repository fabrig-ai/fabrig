# ui-wait-words: a run waiting for a host says why, with a command that exists

`inputs/request.md`: proof-2's goal showed "no host has room, or none is connected: deployment show", a command ctl doesn't have, while its charter's 100,000 turns kept it off the box's 800-turn worker for good.

## Parts and tasks

| Part | Task | Tree | What it makes true |
|---|---|---|---|
| `bounds` | `bounds-wait` | jig, and temper's compile-forced lines | the fleet answers which bound keeps every admitted host out (`bounds_refusal`), the standing refusal keeps its figures, and both reach the client's task document through v19; the wire keeps a turns, time or waiting shortfall's charter |
| `words` | `wait-words`, needs `bounds-wait` | temper | the page, the terminal and ctl say "No worker can take this: its charter asks 100,000 turns, and the largest worker takes 800." for each bound; every waiting line's ctl command exists, checked by a column of an existing table |

**Critical path:** `bounds-wait`, then `wait-words`. Two tasks, as the request allows, and sequential: the words read the contract's new fields.

## What this cut found and decided

- **Where the engine decides "never" against "not now"** (request, item 3). temper's startup refuses a charter larger than the agent allows, so proof-2's charter grew after the worker was admitted. `needs::replace` then clears that worker's eligibility for the charter and keeps it admitted; `fits` keeps the run off it for ever; no hello is refused, and `standing_refusal` is suppressed by the admitted worker. The task reads plain `TaskRunWait::Host`. The difference never reached the client.
- **The carrying:** `TaskWaitRefused::Bounds { kind, why, figures }`, beside `Hello`, which gains `figures`. Derived, like the standing refusal: no stored record, task state or placement changes. `figures` is the charter's need and the most any of the kind's hosts offers, so the words can say "the largest worker". Where each host falls short of another bound, `why` is none and the words say so.
- **A protocol change, named here** (request, item 3): the client wire's v19 schema changes in place, with no version bump: `TaskWaitHello` gains `figures`, `TaskWaitRefused` gains `bounds`, and `AgentShortfall`'s turns, time and waiting gain their charter, which closes engine.status's *Departs* on the dropped charter. The client contract (`jig-client-domain`) changes with it. No stored format changes.
- **`deployment show` stays unbuilt** (`client/tools.md`, 5.5, area 3). The host wait's ctl line names no command: it starts by itself.

## Overlaps with the drafts beside this one

- `model-catalogue-3/list-close` globs `**/*.status.md` and names `jig/docs/design/STATUS.md` and `temper/docs/design/STATUS.md`: this plan adds and removes single entries in `engine.status.md`, `contract.status.md`, `protocol/client.status.md` and, with `wait-words`, `client/tools.status.md`.
- `model-catalogue-3/client-facts` (merged) changed the same v19 schema and goldens; `bounds-wait` regenerates them from the trunk.
- `temper-service`'s `startup/checks.rs`, which other drafts change, is left alone: it reads jig-domain-fleet's own `AgentShortfall`, which doesn't change.
