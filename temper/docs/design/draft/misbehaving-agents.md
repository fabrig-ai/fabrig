# misbehaving-agents: test worlds whose agents act as real models do

Draft, 2026-11-26, on the user's word. The prompt was an agent crash on the reference deployment, fixed in a0f6f9d91b, which no test world could draw. The fresh-seed campaigns draft is this one's companion: it widens how many seeds run, after landing, and this draft widens what a seed can draw and what each case checks. Its reasons are in [misbehaving-agents.notes.md](misbehaving-agents.notes.md), its execution in [misbehaving-agents.execution.md](misbehaving-agents.execution.md).

## 1. In one page

The worlds that run smith's real session draw faults, timing, restarts and people. But an agent's own tool calls are either a fixed script or random picks from a few fixed argument bodies that can name nothing the run made. On 2026-11-26 a model's reply polled a job its session no longer had, beside a write and a read, and the agent crashed. No seed of any world could draw that reply. This draft makes the worlds' agents do what real models do, and makes each domain say at once when it is wrong:

1. **The escaped shape, drawn** (4.1). smith's session world offers and decodes `job` and yielding commands, its menu's bodies gain the fields they need, and its sweep counts the shape that escaped, and its outcome.
2. **A domain checks what it conserves where it changes** (3). Admission refuses, as a value. A step that takes, moves or gives back what the domain reserved asserts that the reservation is its own. A breach fails at its cause.
3. **The fake model names what the run made** (4.2 to 4.4). A menu body may hold a reference slot, which the fake fills with a value the conversation holds, or one never issued. Each answer draws from its agent's own seed, keyed by its request. A drawn agent is captured as it plays, and the capture replays.
4. **Each tree's vocabulary** (5). smith's tools, and jig's and temper's tasks, questions and messages, with each refusal they target named with its setup, and with stretches of drawn turns inside stories that still deliver and finish.
5. **Parks against wakes, as a table** (6). jig's kinds of park and answer are one half, smith's states of a session at its park the other. A table states what each pair expects, and a driver in each world that runs both observes the pairs as kinds of a family of their own.
6. **Relations across components, at quiet points** (7). What no single domain can see, such as jig's call records against smith's open calls, is checked by the composed worlds through the inspection adapters. The relations allow every state a valid quiet point can be in.
7. **Production as the coverage target** (8). The worlds report the shapes they reached, and a reader counts the shapes a deployment's runs hit, both in one abstract vocabulary. It names the shapes a deployment hits that the worlds' samples didn't reach.

Fresh seeds, a case's replay, reach counted over a campaign, and the rule that an escape widens its world are the fresh-seed campaigns draft's (section 9).

## 2. Where it starts

| What happened | Why it went on | Section |
|---|---|---|
| On 11-26 the agent shell panicked ten times, in three runs, at `smith/crates/smith-domain-session/src/session/accounting.rs` ("call owned result credit"). A resumed model batched a poll of a job ended at the park, an edit and a read. The tools answered the poll invalid at their entrance, the batch was marked failed, and the read started with no room reserved for its result. One goal stuck for five hours | smith's session world draws random batches over 1000 noisy seeds. But its translator neither offers nor decodes `job`, it builds every shell call without a yield, and its menu's bodies hold no `job` and no `yield_seconds` (`smith/tests/session/src/provider.rs`, `translate.rs`). Only a call the tools refuse at their entrance escapes the batch's forecast of its room (`smith/crates/smith-domain-session/src/session/batching.rs`, `scheduled`), and the two such answers, an unknown job and full job cells, need both | 4.1 |
| The fix pinned its case in two scripted run-world stories and a session step test, and widened no world (a0f6f9d91b) | A repair's rule asks for the instance, not the class | 4.1 |
| smith's agent, host and local worlds use the same menu. jig's channel and one-process worlds and temper's one-process world play scripted turns. temper's composed workload plays scripted turns too, with smith's menu on its typed provider and an empty one on its wire provider | A script names its calls and values. The fake's random answers copy whole bodies and ignore the conversation's values | 4, 5 |
| A drawn agent's tool picks change whenever a fault's chance changes | The fake draws answers, faults, cache routing and latency from one seed (`skein_fake_llm_domain`, `Domain::played`) | 4.3 |
| jig's parks and answers are drawn with scripted run answers, and smith's park states with scripted wakes. No sweep crosses the two. smith's inspection adapter is designed but not built (`smith/docs/design/testing.status.md`) | Each half was built in its own tree | 6, 7 |

## 3. A domain checks what it conserves where it changes

### 3.1 The rule

- **Admission refuses, as a value.** A step asked to reserve room, a cell, a slot or a credit it doesn't have answers full, busy or refused, and its caller handles that (`skein/docs/foundation/programming-model.md`, 10.4).
- **A step that uses a reservation asserts it.** Taking reserved room, moving it, giving it back, or freeing a cell or slot is done only by the step that holds the reservation the domain made. That step asserts what it relies on: the room was reserved for this taker, it was taken and not already given back, the cell is held. Each is fail-stop, its `expect` naming the invariant. A breach is the domain's own bug.
- **At the cause, not at a later debit.** A read started with no room reserved for its result fails as it starts, naming the read and its batch. It doesn't fail when its result arrives, naming a subtraction.
- **Tested by what it guards, never by tripping it.** Tests show each check's valid edge and the admission's refusal through their observable effects, as the fix's step test does: the read starts only after the batch's reclaim, and the credit balances (`smith-domain-session`'s `tests/batched_change.rs`). In a world, a seed that breaches a check fails that world at the transfer.
- **Written into skein's programming model,** beside the assertions it describes, so every tree's domains follow it and its reviews check it.

### 3.2 Where it applies first

Each owner is read against 3.1, and an assertion is added where a use of a reservation has none:
- **smith's session:** the provider credit, a batch's reserved room and each call's share of it, and the transcript's room. Dispatch already starts only the calls in its batch's reserved range (a0f6f9d91b). A call's result now asserts that its call held room in that batch, and the batch's reclaim asserts that each share is given back once (`smith/docs/design/domain/session.md`, 5).
- **smith's tools:** job cells, held on insertion and freed by a terminal read or the session's close (`smith/docs/design/domain/tools.md`, 5).
- **jig's core:** the one admission bound shared by pending and retained calls, and kept parts in their own room.
- **jig's tasks:** the branch writer slots they hold.
- **temper's landing child:** a landing's members, at insertion and departure.
- **temper's forge room:** the outbox's room over all its holders: live entries, reservations, asked waiters, agent calls, goal projections, and entries of ended owners still settling (`temper/docs/design/domain/forge/room.md`, 2). Each holder's share is compared with its owner's lifecycle state.

## 4. The fake model draws what a model can say

### 4.1 The escaped shape, drawn

- **smith's session world offers `job` and decodes it,** and decodes a shell call's `yield_seconds`, as the product's decoder does.
- **Its menu's bodies gain both fields.** Each body already carries the fields of every tool (a path, a content, a command, a pattern, edits), and a call reads the fields of its own tool. So some bodies gain `"job": 1` and a `yield_seconds` of 0, and a drawn `job`, `shell`, `read` or `write` call reads its own fields from any of them.
- **A yielding command outlasts its yield** by the world's scheduling of its completion (`Settings::tool`), not by a real sleep. A drawn yielding shell then holds a job in one of the world's four cells, and the world's referee accepts a running job's answer.
- **The sweep counts the escaped shape and its outcome:** a completion in which the tools refuse a call at their entrance, before a write, with a read after it. The write is answered not run, and the read starts in the next reserved batch. It also counts unknown jobs, and full cells from four held jobs and a fifth yielding shell. Its reach check asks for all three.
- **The agent, host and local worlds** share the menu, so they gain the bodies.

### 4.2 Reference slots

- **A body may hold a slot,** a placeholder the fake fills as it draws the call. It is declared by `before` and `after` delimiters, and its placeholder forms are the take's: `"{{#NAME}}"` for a number, `{{NAME}}` in a string (`skein/docs/design/testing/fakes/llm.md`, 2.6).
- **Where it looks:** the parts a take reads (each text part's text and each tool output's output, never a tool call or an opaque block), from the newest back. Each distinct value counts once, between 1 and 256 bytes, and where two slots' delimiters overlap, the first declared wins. The scan stops at a bounded number of candidates. The fake keeps the candidates' places, not copies of them.
- **What it draws:** a candidate the conversation holds, or, by its own weight, the slot's declared value that the world never issues. A numeric slot has a checked range, and a candidate outside it is skipped. An empty set of candidates gives the never-issued value.
- **Sized before it is copied.** The answer is measured with each slot filled, against the fake's admission (`script_bytes`), before the body is copied. An answer past it is drawn again without the slot.
- **The fake knows only delimiters.** It stays the application's servant: no tool's name, meaning or lifetime.
- **The world classifies what was drawn.** Whether a drawn reference was live, ended and held, consumed, from before the last opening, foreign, or never issued is known only to the world, which records it beside the answer the domain gave, `unknown` where it can't tell. A mention isn't a lifetime: a job can end and still be held until its end is read, and a number one past the largest the model saw may already be issued.
- **The renderer shows what the product shows.** smith's session world renders a job's result and a read's continuation as the product's renderer does, so the model sees `job N` and the continuation, and its slots can find them.

### 4.3 Each answer from its agent's own seed

- **An agent's draws are a part of `Parties`,** `agent/NAME` (`skein/docs/design/testing/sweeps.md`, 2). The fake's failures draw from `Faults` (`llm`), and its cache routing and latency from `Scheduling`.
- **A named draw for each request.** The answer to a request is drawn from the agent's part, keyed by its conversation, its logical turn and the request's attempt. So the same request draws the same answer whatever the faults. A fault that prevents a request or adds a retry changes which requests are made, not what any one of them draws.
- **A world not yet on streams** derives the three seeds from its own seed, as it now derives one.
- **The move changes every random-mode world's draws once.** Before it, each world's cases pinned on a behaviour are listed and kept as scenarios, or their draw is kept by name. Then each world runs at its gate seeds. Where a reach check then misses a kind, its seeds are picked again by the rule they were picked by (`temper/docs/design/testing/workloads-sweeps.md`, 1.4). Each affected binary's seed and check counts, CPU and instructions are given before and after the move.

### 4.4 A drawn agent is captured as it plays

- **The capture:** each answer the fake drew, keyed by its conversation, logical turn and attempt, with its lines, its finish and its tokens, as a `Turn` holds them (`skein_fake_llm_domain`'s `api.rs`). Beside each answer, not in it, go each slot's draw and the world's classification of it. The capture is bounded by a count of answers and of bytes, and a capture that hits its bound says so.
- **The capture replays.** Played back, it answers each request by the same keys, so a retried or cut request gets the answer it got before, and two conversations keep their own answers. Faults, timing and every input the fake didn't draw still come from the case's seed, as the fresh-seed campaigns draft replays a case.
- **A finding is pinned as its capture,** a fixed scenario, since a seed draws another agent once a menu or a weight changes.

## 5. Each tree's vocabulary

### 5.1 smith's tools

- **The menu becomes the session's vocabulary:** bodies for each tool the world grants, with slots where a tool names what the run made:
  - `job` from a shell's `job N`;
  - `read`'s offset from its continuation;
  - a path from an earlier input, listing or search hit.
- **Each refusal it targets is listed** with the setup it needs and the answer that shows it. Most need no more than a body: a path past the view, a write to a protected path, an oversized body, a NUL byte. Some need prior state (a stale version, a path not yet read), timing (an expired deadline) or scarce capacity (busy, full cells), and their setup says how the world provides it.
- **Weights toward the shapes a domain refuses,** above all a completion whose earlier call is refused before a write, with reads after it.
- **The agent, host and local worlds** take the same vocabulary with their own grants (`smith/docs/design/testing.md`, 4).

### 5.2 jig's and temper's runs

- **Task numbers, questions and messages.** A run's creation replies, inbox labels and `read_task` show task numbers. `Message N sent` shows message numbers, and a question's number comes from its label. These fill slots in `read_task`, `message`, `subscribe` and an answer's `question`, and a domain's refusals (an unrelated task, an answered question, an ended recipient) are counted as answers reached. A question's slot draws from its own task's labels, and a foreign one, by its own weight.
- **Drawn stretches inside a story.** A story's contract holds: it writes its files, sets its CI outcome, delivers, parks where it says, and finishes. At stated points between its steps, a bounded number of drawn turns from the producer's reading and messaging tools are played. After the stretch, the story's next scripted turn plays. The drawn turns never deliver, finish, cancel or amend.
- **Where:** temper's composed workload, through the provider it actually plays: its typed provider takes a menu and a round count, and its wire provider, `temper-fake`, takes them too. Then jig's one-process world. The workload's goals' expectations stay as they are, and a drawn stretch adds its own: each drawn call answered once, each refusal of a reachable kind.
- **The forge's references** (branches, heads, pulls, CI runs) belong to a generated producer for landings and goals. That producer isn't this draft's (section 9). It would use these slots.

## 6. Parks against wakes

### 6.1 The two halves

- **jig's half,** unchanged: its family of runs that park, resume and move (`jig/docs/design/domain/testing-workloads.md`, 4). Some of its kinds resume the run: an answer while parked, one to its successor, a park on nothing, a watch that can answer. Others end it without a resume: an answer after its end, which is dropped, a coordinator's end, a circle held.
- **smith's half,** a family of its own. Each kind is a state observed as the run's park is answered, with what the design says becomes of it:

| A session's state at the park | Observed | If the run resumes | If it ends |
|---|---|---|---|
| a withheld `wait` | the last turn told without its result | answered by the waking messages, once (`smith/docs/design/domain/run-messages.md`, 1) | nothing owed |
| a job running | ended by the session's close, before the park is answered | its number unknown to the resumed kit; a model naming it is refused (`smith/docs/design/domain/tools.md`, 5) | ended already |
| a cut turn with kept parts | the parts told before the cut | rebuilt from its eligible parts, else planned afresh (`smith/docs/design/domain/turn-parts.md`, 5) | parts dropped with the task |
| a host call answered or unanswered | its part told or not | an answered call replays, an unanswered one is sent again (the same, 5) | dropped with the task |
| a sub-agent's conversation kept | its parts told | it continues from its eligible parts (the same, 6) | dropped with the task |

- **Where it is drawn:** in smith's run and agent worlds, which own a run's park and wake. The session world drives what a session alone does, its close and its restore.

### 6.2 The cells and their driver

- **A cell is one kind of each half, for the same park:** one run, one attempt's park, and the answer or end that follows it, in that attempt or its successor.
- **Its expectation is the table's row,** under its column's outcome. A pair that can't occur is listed with its reason.
- **The cells are kinds of a third family,** one kind per cell. A driver in each world that runs both halves chooses an eligible target and arms both halves for it. It keys the park by its task, its attempt and the park's number, and injects the cell's kind once per park, when it has seen both halves fall for that key. The fault ledger records what it is given and keeps sets of kinds, with no run identity (`skein/docs/design/testing/sweeps.md`, 3), so the pairing is the driver's.
- **Its tests:** two tasks that hit opposite halves in one seed make no cell, and neither do two parks of one task, each with one half.
- **Where they're drawn:** the worlds that run real smith under jig's wakes, which are jig's one-process world and temper's one-process and system worlds. At the gate a sweep configures a cell only where its held seeds show it falls within their sample. One kind per family is drawn in the composed sweep (`temper/docs/design/testing/workloads-sweeps.md`, 1.2). The rest wait for a sweep adopted by the fresh-seed campaigns draft, which counts them as any kind.

## 7. Relations across components, at quiet points

- **What no single domain sees is checked by the worlds,** at each world's quiet point, through the inspection adapters (`skein/docs/design/testing/inspection.md`, 5).
- **Relations, not equalities.** A valid quiet point can hold a relay in flight, an answer jig retains for replay after smith has used it, or a terminal answer a worker holds until it is acknowledged. So each relation is keyed by task, attempt, completion and position, and allows each such state by name:
  - each call smith's session holds open for a run has its record in jig's core, or its relay is in flight;
  - each of jig's records for a run's call is matched by smith's open call, or is an answer retained for replay, or its relay is in flight, or its run was cancelled;
  - the engine's placements on a worker match the worker's hosted slots, or a placement's answer awaits its acknowledgement;
  - temper's forge room taken matches 3.2's holders.
- **An orphan** is a record on one side that matches none of these, and is a finding naming both sides.
- **Its tests:** each relation's valid skews pass, and a planted orphan fails.
- **smith's inspection adapter is built,** as `smith/docs/design/testing.md`, 6 designs it, with readings of a session's reserved room, provider credit, open calls and their batches, and job cells. The session world wires it in, so 3.2's sums are read at its quiet points too. The composed worlds' inspection reads jig's core placements and flights as well as its task child.

## 8. Production as the coverage target

- **One vocabulary for both sides.** A shape is an abstract tuple:
  - the tools of each completion, in order;
  - each call's answer kind;
  - whether a drawn reference was live, stale, foreign or never issued;
  - the run's end class, its next begin reason, and the kind of message that woke it.

  Its words are an allowlist from 5's vocabulary and 6's cells, with `unknown` for anything else. It is versioned, and the repository holds it, never a deployment's data.
- **The worlds report what they reached:** each sweep that draws agents counts, per case, the shapes its cases reached, in that vocabulary, beside its sample's size.
- **A reader,** `tools/shapes/`, reads a deployment's task activity (`temper ctl task activity --json`), every page of each task in a window, so that an id issued before a turn is seen. A field the activity doesn't carry is `unknown`, as is a page cut or a turn unread. Each call's answer kind is read from the archived turn, which holds it typed, and the activity's call rows gain that kind where they now give only a failed flag.
- **Its report** lists the shapes a deployment hit that the worlds' samples didn't reach, by how often, with the samples' sizes and the incomplete cases counted. Absence from a sample isn't proof that no world can draw a shape. The report goes to the user, never into the repository.
- **Nothing raw escapes.** The reader keeps only allowlisted labels and counts. Its tests show that no input text, value or id reaches its report, its fixtures or its error lines.
- **Each shape the worlds missed** becomes a menu body, a slot, a weight or a cell, in the world that should draw it.
- **The reader runs after landing,** on the user's word, never in a plan.

## 9. Not in this draft

- **Fresh seeds after landing,** a case replayed, a finding's words and pinning, reach counted over a campaign, and the rule that a fault found outside the worlds widens the nearest world's draws: the fresh-seed campaigns draft's.
- **A generated producer for landings and goals** in temper's forge and engine worlds, drawing the forge's references through this draft's slots. It is named in the fresh-seed campaigns draft, but neither draft schedules it yet.
- **Real models in tests:** benchmarks, outside the suites (`smith/docs/design/testing-tiers.md`, 4).
- **Coverage-guided fuzzing** of decoders, and conversion across two releases under a crash.
- **Shrinking** a captured agent before it is pinned.

**Tests stay cheap.** The suites are the check. A goal runs a test it adds or changes by name, alone, at most. Widened menus change what the gate's seeds draw, not how many run. A plan runs the focused and fuzzy suites alone and is complete when it lands. walk = false.
