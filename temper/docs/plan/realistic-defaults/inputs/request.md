# temper's defaults sized for real work: a fresh box runs a project the size of fabrig's own development (the user, 11-03)

## Why

A box made by `temper init` with the stock settings cannot run the reference case
(`temper/docs/design/draft/reference-case.md`; a goal of up to a hundred changes, N21) or anything
like fabrig's own development. QA's walks hit a default at nearly every step:
- two chats exhaust the owner's pool;
- a goal's tree is capped at 18 delegated tasks, 4 deep;
- 8 live tasks and 4 live delegates;
- charters get 16 turns and 120 s;
- a change takes 4 questions and 6 tasks;
- the stock tool set has no `control` family, so `change` offers a goal no control schema;
- sub-agents fault on per-completion limits.

Each walk then spends minutes finding and overriding them, and some overrides are refused with
"invalid override" naming nothing. The user's decision: the defaults allow realistic workflows, with
headroom, and are sized from what fabrig's own development needs today.

## The target: fabrig's development as fab runs it (measured 10-31 17:20Z to 11-03 17:20Z)

| What | Largest seen (median) | The default to allow |
|---|---|---|
| tasks in one goal's tree, lifetime | 73 (2) | 256 |
| depth of the work tree | 5 | 8 |
| live tasks in a deployment (jobs in flight) | 25 (3) | 64 |
| live tasks under one goal | 15 | 32 |
| live goals at once | 14 | 32 |
| live direct delegates of one task | 15 | 32 |
| questions per change or task | 15 (1) | 16 |
| runs (repairs, retries) per task | 14 (1) | 16 |
| turns in one run | 156 (37) | 400 |
| one run's wall time | 76 min (8) | 3 h |
| tool calls in one model response | 9 (1) | 16 |
| sub-agents at once in one run | 5 | 8 |
| task spec / message | 19 kB (5.9 kB) | 64 kB |
| report (a result) | 46 kB (4.9 kB) | 64 kB |
| chats or goals funded at once from one owner's pool | 14 | 32 |
| live changes in a project | 18 implement runs at once | 64 |

The figures and how they were measured are in fabrig/qa's survey (to be quoted in the design's own
words, not cited): plan and run figures from fab's log over three days, 1,834 Codex runs and 636
Claude calls.

## What to change

1. **The store profile:** init writes jig's `standard` profile in place of `small`.
   - Where: `temper/crates/temper-init/template/temper.json`, its three `"profile": "small"`, and
     `docs/examples/owned.json` in lockstep.
   - Why: `standard` already sizes a team's shared deployment (`jig/crates/jig-protocol-config/src/profile/standard.rs`:
     tasks 32, tree_tasks 128, delegates 8, batch 4, operations 16, a store value of 10.7 MB). small's
     fixed store row is the wall behind most refused overrides.
   - Then raise, by init's overrides, what the table asks and `standard`'s store bounds hold:
     tasks.tasks and project_tasks 64, tree_tasks 256, depth 8, project_goals 32, delegates 32 with
     people.operations 33 and inbox_tasks 128 (temper/docs/design/protocol/configuration.md, 13.9's
     table already asks delegates 32 / operations 33), result_bytes and message_bytes as large as
     the store holds up to 64 kB, documents.lines 32.
   - Where `standard`'s store cannot hold a figure, take the largest that fits and say which in the
     design's status file. A larger profile or a new store schema is out of scope (see below).
2. **The agent profile** (init's `agent.json` over smith's `standard`):
   - max_turns 400, calls_per_response 16, conversations 9 (a run and 8 sub-agents);
   - tool_payload equal to checkout.message_bytes, as large as T and the store allow;
   - the carriers that follow from T (configuration.md 13.8's table) recomputed.
3. **The stock settings** (`temper/crates/temper-service/src/stock.rs`):
   - **Budgets:** the role's authority, the rules' and policy 1's ceilings and periods sized so 32
     chats or goals fit at once (the chat-to-pool ratio 1:32, not 1:2).
   - **Charters:** chat (1) 32 turns and 30 min; charters 2, 4 and 6 at 128 turns and 1 h;
     reserves as now.
   - **Tool families:** add `control` (4), so a goal's `change` gets its control schema.
   - **Delegation:** 256 tasks, depth 8.
   - **Templates' report bounds:** as large as the store's result allows.
4. **The forge's application limits** (`temper/crates/temper-service/src/limits.rs`):
   - forge.tasks, holds and changes 64; landings 8;
   - change_policy: questions 16, tasks 16, repairs, resolutions and updates 8;
   - issue_policy.plan_items 64.
5. **Concurrency** that is a machine's to size (fleet, hub and worker slots, the runner's capacity)
   stays as init sets it, unless the design already names a figure. Say so in the status file;
   hardware sizing is ops', not this plan's.

The design first: these defaults are written in `temper/docs/design/protocol/setup.md`, 8.1 (init's
limits and agent profile), `configuration.md`, 13.4 (the stock) and 13.8 and 13.9, and smith's and jig's
profile pages where they name them. Change those sections to the new figures, giving the
reason (a fresh box runs a project of fabrig's size). Then change the code, and update every test that
pins a figure:
- `temper-init/src/tests/documents.rs` and `sub_agents.rs`;
- `temper-service/src/tests/stock.rs`;
- the 207,552 / 449,696 figures in `temper/tests/suite/startup.rs`, `tests/proof_live.rs`,
  `tests/support/forge_land/rung.rs` and jig's `funding/settings.rs`;
- `reference_settings.rs`;
- `real-loop/tests/suite/template_bounds.rs`.

A test: a fresh box from `temper init`, with no override, starts. Its `me` lists the stock charters,
and its stock budget funds 32 chats at once.

## Out of scope

- A store schema or codec change: tasks.batch past 4, the codec ceilings in tasks-v9 / people-v14, the
  client wire's v19 settings counts. These are a stored format and a protocol, so the user's call.
  If a figure above needs one, stop at the largest that doesn't, and ask.
- A profile larger than jig's `standard`.
- Machine sizing (slots, runner capacity, memory).
- The reference's own settings and limits files (`docs/examples/reference/`), unless a stock change
  makes them inconsistent.

## Done when

A fresh box runs the reference case's steps 1 to 6 with no override and no settings change beyond
the reference's charters and template. QA's rung rm3 walks it. Main holds the new design sections and
code, with every pinning test updated.
