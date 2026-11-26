# misbehaving-agents: notes

The reasons behind [misbehaving-agents.md](misbehaving-agents.md), and what it rests on.

## The incident and its fix

- **On 2026-11-26** the reference deployment's agent shell panicked ten times, in three runs, at `receive_result`'s `expect("call owned result credit")`. Each run had started a shell command in the background, called `wait` with only its own job open, and parked. Woken by "Nothing more will come", its resumed model batched a poll of job 1 (ended at the park), an edit and a read. Two runs got through on their first retry. One crashed on all eight resumes, and its goal stuck for five hours.
- **The cause,** from the fix's change (a0f6f9d91b): the poll was answered invalid at the tools' entrance, which marked the batch failed. Advancement skipped the edit, where the batch's reservation had stopped, and started the read with no room reserved for its result. When the read's result arrived, `reserved` underflowed. A batch's calls are now fixed when it starts (`smith/docs/design/domain/session.md`, 5).
- **The park wasn't needed.** The fix's own step test names job 7, never started, in one batch of `job`, `write` and `read`. A park and resume is only how production put a stale job number in front of the model.
- **The notice was right.** Parking closes the session and ends its jobs, so a host that knows of nothing else to come may say so (`smith/docs/design/domain/run-messages.md`, 1).

## The inventory, at main d97da9dd3e

Taken on 2026-11-26 by a read-only survey, and read again where this draft relies on it.

- **The fake's random answers.** When the script has no turn for a request, the fake draws 1 to `calls_per_answer` calls from the offered tools, each copying a whole menu body or one of its invalid entries, for up to `tool_rounds` rounds. One SplitMix64 generator, seeded once by `Domain::played`, draws answers, provider faults, cache routing and latency. The fake sees the whole query. Random mode ignores its values, while a scripted turn's take reads them (`skein/docs/design/testing/fakes/llm.md`, 2.6). A `Script` is a list of `Turn`s of `Line`s, and nothing exports a drawn answer as one.
- **Who draws.** smith's session world: 1000 noisy and 1000 calm seeds, four job cells, five bodies (paths, contents, commands, patterns, edits) and three invalid entries. Its agent, host and local worlds use the same menu. Its run world feeds typed peer events. jig's channel and one-process worlds and temper's one-process and system worlds play scripted turns with an empty menu, and temper-fake decodes JSON into cue-selected turns.
- **The entrance's answers.** Only `JobProblem::Unknown` (a `job` naming a number the kit doesn't hold) and `JobProblem::Full` (a yielding `shell` with every cell held) become invalid answers at the tools' entrance. `batching::scheduled` forecasts calls that don't decode, and known failures. Other immediate outcomes are owned answers: not granted, outside the view, too long, read-only, protected, a NUL byte, too large, an empty edit, a zero offset or window, timed out, busy.
- **Ids a model names.** smith: a job number from a yielding shell's `job N`, kit-local, answered once after it ends, unknown after that or from before the opening. Paths from inputs, listings and search hits, refused as not found, not read or stale. A read's continuation. jig and temper: task numbers from creation replies, inbox labels and `read_task`, used by `read_task`, `message`, `cancel`, `amend`, `release` and `subscribe`. Question and message numbers. Forge references (repository, branch, head, pull, CI run).
- **Room.** smith's session checks credit and batch room by checked arithmetic and assertions, its tools check job cells on insertion, and no smith inspection reading covers them. jig's core checks `call_parts + pending_calls` for new calls, and caps kept parts on their own. No three-table sum is checked anywhere. temper's forge checks write room at admission and frees it at a terminal or abandonment, and its inspection reads writes, holds and landings.
- **The rule's place.** skein's programming model, 10.4: assertions are fail-stop and for the service's own bugs, and `expect` names the invariant it relies on. There is no separate smith programming model.
- **Parks.** jig draws `Faults/parks` with scripted run answers. smith closes jobs on a park and restores only their high-water number, answers a withheld `wait` with the waking messages, and rebuilds a cut turn from kept parts. No sweep crosses jig's wake kinds with smith's park states.
- **Activity.** jig stores a run's activity: attempt, begin reason, end class and heard kind. temper's JSON expands archived turns into messages with numbered calls, tool, input, outcome summary and a failed flag. Tool names, calls per message, failure flags, park and end classes and wake kinds can be counted without prose. Ids' freshness needs the inputs read as structured values.

## Why these choices

- **Two bodies before anything else.** They need no machinery, they reach both entrance answers on the gate's existing seeds, and they are what the repair's widening would have been. Everything after them is about reaching the next such shape before production does.
- **At the transition, not at a quiet point alone.** A check where the room is taken fails in every world and in production, naming its cause. A quiet point sees a sum after the fact, and only in worlds. The fix took this road for results, so the rule makes it general.
- **Slots, not a grammar.** The fake already draws random batches. What it lacked was the run's own ids. Slots reuse the take's delimiters and placeholders, so the fake still invents no application name or body.
- **Every occurrence, not the newest.** A take wants the newest value. A slot wants stale ones too, since that is where the faults are.
- **The agent in `Parties`.** An agent's choices are a party's actions, as jig's people are owner and member. A part keeps its draws apart from faults' and scheduling's without a new stream.
- **Pin the script, not the seed.** A seed's draws are fixed only while the menu and the weights are. A printed script stays a scenario whatever the generator becomes.
- **Cells as a family of kinds.** The ledger and `Sweep` count sets of kinds, so a cell kind injected by the world that sees both halves fall on one run is counted, checked and excluded like any fault, and a campaign counts it as any kind. The ledger needs no run identity.
- **Shapes stay abstract.** The repository is public, so it holds the vocabulary and the reader, and a deployment's counts stay with its user. The reader runs after landing, since it reads a deployment, and a plan runs only the suites.

## The brainstorm

Written with the fresh-seed campaigns draft's drafter on 2026-11-26, on the user's word ("you two will brainstorm together, directly"). That draft owns fresh seeds and replay, and this one owns what a seed can draw and what a case checks. Each needs the other: fresh seeds over hand-written agents find nothing new, and generated agents want more seeds than the gate affords. The agreed order: this draft's two bodies and its checks at the transition first, then that draft's campaigns, with a generated agent as their first workload. That draft's adoption of its seam in smith's session world waits for this draft's two bodies, so the two don't change the same file at once.
