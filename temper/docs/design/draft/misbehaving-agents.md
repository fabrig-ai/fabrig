# misbehaving-agents: test worlds whose agents act as real models do

Draft, 2026-11-26, on the user's word. The prompt was an agent crash on the reference deployment, fixed in a0f6f9d91b, which no test world could draw. The fresh-seed campaigns draft is this one's companion: it widens how many seeds run, after landing, and this draft widens what a seed can draw and what each case checks. Its reasons are in [misbehaving-agents.notes.md](misbehaving-agents.notes.md), its execution in [misbehaving-agents.execution.md](misbehaving-agents.execution.md).

## 1. In one page

The worlds that run smith's real session draw faults, timing, restarts and people. But an agent's own tool calls are either a fixed script or random picks from a few fixed argument bodies that can name nothing the run made. On 2026-11-26 a model's reply polled a job its session no longer had, beside a write and a read, and the agent crashed. No seed of any world could draw that reply. This draft makes the worlds' agents do what real models do, and makes each domain say at once when it is wrong:

1. **Two bodies first** (4.1). smith's session world's menu gains a `shell` body that asks to yield, and a `job` body. With them its existing seeds draw both answers the tools give at their entrance that a batch can't foresee. This is the widening 11-26's repair should have made.
2. **A domain checks what it conserves where it changes** (3). Wherever a quantity is reserved, taken, given back or released, the step that moves it checks it, fail-stop, with the invariant named. A breach fails at its cause in every world and in production, not later at a debit.
3. **The fake model names what the run made** (4.2 to 4.4). A menu body may hold a reference slot, which the fake fills from the conversation with an id seen earlier (live or ended) or one never issued. The fake's answers and its failures draw from two seeds, and a drawn agent prints as a script that replays.
4. **Each tree's vocabulary** (5). smith's tools, jig's tasks, questions and messages, and temper's producer turns, with weights toward the shapes that reach a domain's refusals.
5. **Parks against wakes, as a table** (6). jig's kinds of park and answer are one half, smith's states of a session at its park the other. Their pairs are kinds of a family of their own, drawn where real smith runs under jig.
6. **Agreements across components, at quiet points** (7). What no single domain can see, such as jig's call records against smith's open calls, is checked by the composed worlds through the inspection adapters.
7. **Production as the coverage target** (8). A reader counts the shapes runs on a deployment hit, as abstract tuples over 5's vocabulary and 6's cells, and names those no world draws.

Fresh seeds, replay, reach counted over a campaign, the generated producer for landings and goals, and the rule that an escape widens its world are the fresh-seed campaigns draft's (section 9).

## 2. Where it starts

| What happened | Why it went on | Section |
|---|---|---|
| On 11-26 the agent shell panicked ten times, in three runs, at `smith/crates/smith-domain-session/src/session/accounting.rs` ("call owned result credit"). A resumed model batched a poll of a job ended at the park, an edit and a read. The tools answered the poll invalid at their entrance, the batch was marked failed, and the read started with no room reserved for its result. One goal stuck for five hours | smith's session world draws random batches over 2000 seeds, but its menu's five bodies hold no `job` and no `yield_seconds` (`smith/tests/session/src/provider.rs`). Only a call the tools refuse at their entrance escapes the batch's forecast of its room (`smith/crates/smith-domain-session/src/session/batching.rs`, `scheduled`). The two such answers, an unknown job and full job cells, need those two fields | 4.1, 5.1 |
| The fix pinned its case in two scripted run-world stories and a session step test, and widened no world (a0f6f9d91b) | A repair's rule asks for the instance, not the class | 4.1 |
| The underflow was caught where the read's result arrived, not where the read started without room | The session checks room at a result, and dispatch didn't check that a call lay within its reserved batch | 3 |
| smith's agent, host and local worlds use the same menu. jig's channel and one-process worlds and temper's one-process and system worlds play scripted turns with an empty menu | A script names its calls and values. The fake's random answers copy whole bodies and ignore the conversation's values | 4, 5 |
| A drawn agent's tool picks change whenever a fault's chance changes | The fake draws answers, faults, cache routing and latency from one seed (`skein_fake_llm_domain`, `Domain::played`) | 4.3 |
| jig's parks and answers are drawn with scripted run answers, and smith's park states with scripted wakes. No sweep crosses the two, and smith's inspection adapter reads none of a session's room | Each half was built in its own tree | 6, 7 |

## 3. A domain checks what it conserves where it changes

### 3.1 The rule

- **Every conserved quantity changes only in a step that checks it:** room reserved and taken, credit admitted and released, a cell held and freed, a slot given and returned, a member joined and gone. The step that reserves checks that the room exists. The step that takes checks that the room was reserved for this taker. The step that gives back checks that it was taken and not already given. Each is fail-stop, its `expect` naming the invariant (`skein/docs/foundation/programming-model.md`, 10.4).
- **At the cause, not at a later debit.** A read started with no room reserved for its result fails as it starts, naming the read and its batch. It doesn't fail when its result arrives, naming a subtraction.
- **Expected limits stay values.** Full, busy and refused are answers a caller handles. The rule is for what only the domain's own bug can break (the same section).
- **Written into skein's programming model,** beside the assertions it already describes, so every tree's domains follow it and its reviews check it.

### 3.2 Where it applies first

Each of these is read against 3.1, and a check is added where a transfer has none. Each added check gets a step test showing that a breach fails at the transfer:
- **smith's session:** the provider credit, `reserved` for a batch's results, the transcript's room, and each call's share of its batch. Since a0f6f9d91b, a batch's calls are fixed when it starts (`smith/docs/design/domain/session.md`, 5). Dispatch checks that a call it starts lies within its batch's reserved end, and a result checks that its call held room.
- **smith's tools:** job cells, held on insertion and freed by a terminal read or the session's close (`smith/docs/design/domain/tools.md`, 5).
- **jig's core:** call records, pending calls and kept parts, each against its own cap. A sum is checked only where a design states one.
- **temper's forge:** write room and its reservations, branch writer slots, and a landing's members.

## 4. The fake model draws what a model can say

### 4.1 Two bodies first

- **smith's session world's menu gains two bodies:** a `shell` body with `yield_seconds`, and a `job` body naming job 1. The world gives a yielding command a run past its yield, so its job is held in one of the world's four cells. Its existing seeds then draw both answers the tools give at their entrance: an unknown job, where job 1 was never started or has ended, and full cells.
- **The sweep shows they fall.** It counts the answers its cases got, and its existing reach check asks for both kinds. The world's agent, host and local siblings share the menu, so they gain the bodies too.
- **No new machinery:** the bodies are menu entries, and the world's seeds and counts stay as they are.

### 4.2 Reference slots

- **A body may hold a slot,** a placeholder the fake fills as it draws the call. It is declared with `before` and `after` delimiters, as a scripted turn's take is (`skein/docs/design/testing/fakes/llm.md`, 2.6), and the placeholder forms are the take's: `"{{#NAME}}"` for a number, `{{NAME}}` in a string.
- **The fake fills it by drawing among:**
  - every value the conversation holds between those delimiters, in any message, not only the newest, since a stale id is the point;
  - with its own weight, a value never issued: one past the largest seen, or the slot's declared far value.

  A slot may also declare an `ended` pattern, a second pair of delimiters marking a value as ended (`job 1 ended`). The fake then weighs ended values on their own.
- **The fake stays the application's servant.** It reads the query it already holds, and knows no tool's name or meaning: the application declares each slot's delimiters and weights. An empty conversation fills a slot with its never-issued value.
- **Freshness is the world's to count.** Whether a drawn id was live, ended, from before the last opening, or foreign is known only to the world, which counts it beside the answer the domain gave.

### 4.3 Two seeds

- **The fake's answers draw from their own seed,** an agent's part of `Parties` (`agent/NAME`, `skein/docs/design/testing/sweeps.md`, 2). Its failures, cache routing and latency draw from `Faults` (`llm`) and `Scheduling`. So adding a fault kind, or changing a chance, moves no agent's draws, and the reverse.
- **A world not yet on streams** derives the two seeds from its own seed, as it now derives one.
- **The split moves every random-mode world's draws once.** Each such world is run at its gate seeds after the split. Where a reach check then misses a kind, it re-picks its seeds by the rule it was picked by (`temper/docs/design/testing/workloads.md`, 8.4), its counts staying within its budget.

### 4.4 A drawn agent prints as a script

- **A failing case prints the turns its fake drew** as a `Turn` list: each call with its tool, its filled body and its slot's draw, and each text. The list replays as a fixed script, without the menu or the seed.
- **A finding is pinned as that script,** since a seed draws another agent once a menu or a weight changes (the fresh-seed campaigns draft, its section on pinning).

## 5. Each tree's vocabulary

### 5.1 smith's tools

- **Its menu becomes the session's vocabulary:** a body or more for each tool the world grants, with slots where a tool names what the run made:
  - `job` from a shell's `job N`;
  - `read`'s offset from its continuation;
  - a path from an earlier input, listing or search hit.
- **Weights toward the shapes a domain refuses:** a batch with a call the tools refuse at their entrance before a write, with reads after it; calls past the view; a path not yet read; a stale version; an oversized body. Each refusal the tools give is reached by some body, and the sweep counts the answers it reached.
- **The agent, host and local worlds** take the same vocabulary with their own grants (`smith/docs/design/testing.md`, 4).

### 5.2 jig's and temper's runs

- **Task numbers, questions and messages.** A run's creation replies, inbox labels and `read_task` show task numbers. `Message N sent` shows message numbers, and a question's number comes from its label. These fill slots in `read_task`, `message`, `cancel`, `amend`, `release`, `subscribe` and an answer's `question`. Each slot draws stale, foreign and never-issued numbers as well as live ones, and a domain's refusals (an unrelated task, an answered question, an ended recipient) are counted as answers reached.
- **Generated turns beside scripted ones.** jig's one-process and temper's one-process and system worlds build turns from stories. A story keeps its scripted turns where it depends on a cue, a hold or a value. Beside those, a script kind whose turns the fake draws from the producer's tools joins each world's workload, starting with temper's composed workload (`temper/docs/design/testing/workloads.md`, 5).
- **The forge's references** (branches, heads, pulls, CI runs) are the vocabulary of the fresh-seed campaigns draft's generated producer for landings and goals. It uses this draft's slots.

## 6. Parks against wakes

### 6.1 Two halves

- **jig's half,** unchanged: its family of runs that park, resume and move, with what is open at the park (a question, a watch, nothing) and how the answer comes (while parked, after the end, to a successor, a coordinator ending, a circle). See `jig/docs/design/domain/testing-workloads.md`, 4.
- **smith's half,** a family of its own, of what a session holds as its run parks:
  - a withheld `wait`;
  - jobs ended by the park;
  - a cut turn with kept parts;
  - a delegated call answered or not;
  - a sub-agent's conversation kept.

  It is drawn in smith's session, run and agent worlds. Each kind is restored on resume as `smith/docs/design/domain/run-messages.md`, 1 says, and a resumed model's slots then draw ids from before the opening.

### 6.2 The cells

- **A cell is a pair,** one kind of each half that can fall on the same run. The table lists every pair, and the reason for each pair that can't occur.
- **The cells are kinds of a third family,** one kind per cell. A world that runs both halves injects a cell's kind when both its kinds fall on the same run. The fault ledger keeps sets of kinds and has no run identity (`skein/docs/design/testing/sweeps.md`, 3), so the pair is the world's to see, and the ledger stays as it is.
- **Where they're drawn:** the worlds that run real smith under jig's wakes, which are jig's channel and one-process worlds and temper's one-process and system worlds. At the gate each sweep configures cells as it configures any family, one kind each in the composed sweep (`temper/docs/design/testing/workloads.md`, 8.2). The rest are for campaigns, counted as any kind.

## 7. Agreements across components, at quiet points

- **What no single domain sees is checked by the worlds,** at each world's quiet point, through the inspection adapters (`skein/docs/design/testing/inspection.md`, 5):
  - each delegated call smith holds open for a run has its record or pending call in jig's core, and each of jig's records for a run's open call has its call in smith's session;
  - the engine's placements on a worker match the worker's hosted slots;
  - temper's forge write room taken matches the outbox entries and reservations of its live changes and landings.
- **smith's inspection adapter reads a session's room:** its reserved room, its provider credit, its open calls and their batches, and its job cells, so the session world checks 3.2's sums at each quiet point too.
- **A finding names both sides,** each component's reading and the run or entry they disagree on.

## 8. Production as the coverage target

- **A shape** is an abstract tuple read from a run's activity on a deployment, never its text:
  - the tools of each assistant message, in order, and the calls per batch;
  - which calls the domain refused, and with what kind of answer;
  - whether a call named an id from before the last opening, or never issued;
  - the run's end class, its next begin reason, and the kind of message that woke it.

  Its words are 5's vocabulary and 6's cells, so the repository holds the vocabulary and no deployment's data.
- **A reader,** `tools/shapes/`, reads a deployment's task activity (`temper ctl task activity --json`). It counts each shape over a window, and compares with what the worlds reach, as a campaign's report counts it. Its report lists the shapes a deployment hit that no world draws, by how often, and goes to the user, never into the repository.
- **Each such shape** becomes a menu body, a slot, a weight or a cell, in the world that should draw it.
- **The reader runs after landing,** on the user's word, never in a plan.

## 9. Not in this draft

- **Fresh seeds after landing,** one case replayed, a finding's words and pinning, reach counted over a campaign, and the rule that a fault found outside the worlds widens the nearest world's draws: the fresh-seed campaigns draft's.
- **The generated producer for landings and goals** in temper's forge and engine worlds: the same draft's first temper workload, with 5.2's forge vocabulary.
- **Real models in tests:** benchmarks, outside the suites (`smith/docs/design/testing.md`, 2.4).
- **Coverage-guided fuzzing** of decoders, and conversion across two releases under a crash.
- **Shrinking** a drawn agent's script before it is pinned.

**Tests stay cheap.** The suites are the check. A goal runs a test it adds or changes by name, alone, at most. The gate keeps its seeds and its counts: widened menus change what those seeds draw, not how many run. A plan runs the focused and fuzzy suites alone and is complete when it lands. walk = false.
