# temper's defaults sized for real work: a fresh box runs a project of fabrig's scale, with 5x headroom (the user, 11-03)

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
"invalid override" naming nothing. The wall behind most of them is jig's `small` store profile:
its fixed store row and codec ceilings, which no override can raise.

The user's decision covers every default: limits, settings and store formats alike. Each default
allows a deployment of fabrig's scale, its own development as fab runs it today, with **5x
headroom** on the largest figure seen. temper is pre-release, so a store format, codec ceiling or
schema may change in place: no version bump, no migration of existing stores.

## The target: fabrig's development as fab runs it (measured 10-31 17:20Z to 11-03 17:20Z)

| What | Largest seen (median) | Default (5x, rounded up) |
|---|---|---|
| tasks in one goal's tree, lifetime | 73 (2) | 384 |
| depth of the work tree | 5 | 32 |
| tasks made in one atomic creation (a plan's tasks at once) | 30 | 160 |
| live tasks in a deployment (jobs in flight) | 25 (3) | 128 |
| live tasks under one goal | 15 | 80 |
| live goals at once | 14 | 80 |
| live direct delegates of one task | 15 | 80 |
| questions per change or task | 15 (1) | 80 |
| runs (repairs, retries) per task | 14 (1) | 80 |
| turns in one run | 156 (37) | 800 |
| one run's wall time | 76 min (8) | 8 h |
| tool calls in one model response | 9 (1) | 48 |
| sub-agents at once in one run | 5 | 25 |
| task spec / message | 19 kB (5.9 kB) | 128 kB |
| report (a result) | 46 kB (4.9 kB) | 256 kB |
| a run's brief | 200 kB (55 kB) | 1 MB |
| chats or goals funded at once from one owner's pool | 14 | 80 |
| live changes in a project | 18 implement runs at once | 128 |

The figures and how they were measured are in fabrig/qa's survey (to be quoted in the design's own
words, not cited): plan and run figures from fab's log over three days, 1,834 Codex runs and 636
Claude calls. A limit this table does not name moves with the ones it is coupled to (a store
row, a carrier, an inbox), sized the same way.

## What to change

1. **The store profile and formats.**
   - init's deployment gets a profile whose store rows, pages, journal writes and codec ceilings
     hold the table: jig's `standard` profile raised, or a new one beside it. That is the
     designer's choice; `jig/crates/jig-protocol-config/src/profile/standard.rs` already sizes "a team's
     shared deployment" with tasks 32, tree_tasks 128, delegates 8, batch 4 and a 10.7 MB value.
   - init's `temper/crates/temper-init/template/temper.json` (its three `"profile"` lines) and
     `docs/examples/owned.json` move in lockstep.
   - Codec ceilings that cap a figure change in place, with no new store schema version: tasks-v9's
     batch ceiling of 4, people-v14's envelope, the client wire's settings counts, the terminal's and
     the settings form's receive capacities. Their goldens are regenerated.
   - init's overrides then say only what differs from the profile.
2. **The agent profile** (init's `agent.json` over smith's `standard`):
   - max_turns 800, calls_per_response 48, conversations 26 (a run and 25 sub-agents);
   - tool_payload equal to checkout.message_bytes;
   - the carriers that follow from T (configuration.md 13.8's table) recomputed.
3. **The stock settings** (`temper/crates/temper-service/src/stock.rs`):
   - **Budgets:** the role's authority, the rules' and policy 1's ceilings and periods sized so 80
     chats or goals fit at once (the chat-to-pool ratio 1:80, not 1:2).
   - **Charters:** chat (1) 64 turns and 1 h; charters 2, 4 and 6 at 800 turns and 8 h; reserves
     as now.
   - **Tool families:** add `control` (4), so a goal's `change` gets its control schema.
   - **Delegation:** 384 tasks, depth 32.
   - **Templates' report bounds:** the report figure above.
4. **The forge's application limits** (`temper/crates/temper-service/src/limits.rs`):
   - forge.tasks, holds and changes 128; landings 32;
   - change_policy: questions 80, tasks 80, repairs, resolutions and updates 80;
   - issue_policy.plan_items 384.
5. **Concurrency** that is a machine's to size (fleet, hub and worker slots, the runner's capacity)
   stays as init sets it, unless the design already names a figure. Say so in the status file.

## The resource gate (the user's, before landing)

Measure a fresh box at the new defaults: temper's resident memory idle and with one goal running,
and the store's size on disk, fresh and after the reference-scale tests. Write the figures in the
design's status file.
- **Expected:** tens of MB of memory, up to about 100 MB; a few hundred MB on disk, up to about 1 GB.
- **Past either bound**, at any point (for example a profile's preallocated pages or rows), stop and
  ask the user with the figures and what drives them. Don't land a smaller figure silently.

## How

The design first: these defaults are written in `temper/docs/design/protocol/setup.md`, 8.1 (init's
limits and agent profile), `configuration.md`, 13.4 (the stock) and 13.8 and 13.9, jig's
configuration and store design (profiles, store sizes, codec envelopes), and smith's profile pages
where they name them. Change those sections to the new figures, giving the reason: a fresh box runs
a project of fabrig's scale, with 5x headroom, the store format pre-release.

Then the code, and every test that pins a figure:
- `temper-init/src/tests/documents.rs` and `sub_agents.rs`;
- `temper-service/src/tests/stock.rs`, `startup.rs` and `stock_brief.rs`;
- the 207,552 / 449,696 figures in `temper/tests/suite/startup.rs`, `tests/proof_live.rs`,
  `tests/support/forge_land/rung.rs` and jig's `funding/settings.rs`;
- `reference_settings.rs`;
- `real-loop/tests/suite/template_bounds.rs`;
- jig's `tests/overrides.rs` (the batch ceiling);
- the generated tasks-v9 / people-v14 goldens.

Also mend the refusal found on the way: an override that breaks one core-family relation marks
every core-family override "invalid override". It names the relation instead, as jig's configuration
design promises.

A test: a fresh box from `temper init`, with no override, starts. Its `me` lists the stock charters,
its stock budget funds 80 chats at once, and one goal's tree takes 384 tasks.

## Out of scope

- Machine sizing (slots, runner capacity, memory limits).
- The reference's own settings and limits files (`docs/examples/reference/`), unless a stock change
  makes them inconsistent. Then they are trimmed to what still differs from the stock.

## Done when

A fresh box runs the reference case's steps 1 to 6 with no override and no settings change beyond
the reference's charters and template. QA's rung rm3 walks it. Main holds the new design sections,
code and tests, and the status file holds the measured memory and disk.
