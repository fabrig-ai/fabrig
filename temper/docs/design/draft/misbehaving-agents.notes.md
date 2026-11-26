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
- **Who draws.** smith's session world: a noisy sweep of 1000 seeds draws 1 to 3 calls an answer, and a separate calm sweep of 1000 checks prompt-size ratios with one call an answer. Both have four job cells, five bodies (paths, contents, commands, patterns, edits) and three invalid entries. Its translator neither offers nor decodes `job` and builds every shell call without a yield. Its agent, host and local worlds use the same menu. Its run world feeds typed peer events. jig's channel and one-process worlds and temper's one-process world play scripted turns. temper's composed workload plays scripted turns through a typed provider with smith's menu, or a wire provider, `temper-fake`, with an empty menu and no random rounds.
- **The entrance's answers.** Only `JobProblem::Unknown` (a `job` naming a number the kit doesn't hold) and `JobProblem::Full` (a yielding `shell` with every cell held) become invalid answers at the tools' entrance. `batching::scheduled` forecasts calls that don't decode, and known failures. Other immediate outcomes are owned answers: not granted, outside the view, too long, read-only, protected, a NUL byte, too large, an empty edit, a zero offset or window, timed out, busy.
- **Ids a model names.** smith: a job number from a yielding shell's `job N`, kit-local, answered once after it ends, unknown after that or from before the opening. Paths from inputs, listings and search hits, refused as not found, not read or stale. A read's continuation. jig and temper: task numbers from creation replies, inbox labels and `read_task`, used by `read_task`, `message`, `cancel`, `amend`, `release` and `subscribe`. Question and message numbers. Forge references (repository, branch, head, pull, CI run).
- **Room.** smith's session checks credit and batch room by checked arithmetic and assertions, and its tools check job cells on insertion. smith's inspection adapter is designed but not built. jig's core shares one admission bound between pending and retained calls, and caps kept parts in their own room. jig's tasks own the durable branch writer slots, and temper's landing child owns its members. temper's forge room counts live entries, reservations, asked waiters, agent calls, goal projections and ended owners' entries still settling (`temper/docs/design/domain/forge/room.md`, 2), and its inspection reads writes, holds and landings.
- **The rule's place.** skein's programming model, 10.4: assertions are fail-stop and for the service's own bugs, and `expect` names the invariant it relies on. There is no separate smith programming model.
- **Parks.** jig draws `Faults/parks` with scripted run answers. smith closes jobs on a park and restores only their high-water number, answers a withheld `wait` with the waking messages, and rebuilds a cut turn from kept parts. No sweep crosses jig's wake kinds with smith's park states.
- **Activity.** jig stores a run's activity: attempt, begin reason, end class and heard kind. temper's JSON expands archived turns into messages with numbered calls, tool, input, outcome summary and a failed flag. It is paged and per task, and a turn not yet archived or too large leaves a hole. It shows completions, not smith's batches within them. A failed flag doesn't tell an unknown job from bad JSON. Ids' freshness needs every page of a task, read as structured values.

## Why these choices

- **The escaped shape before anything else.** It needs no new machinery in the fake, only the session world's own translator, renderer and bodies. It reaches both entrance answers and the whole escaped sequence on the gate's existing seeds, and it is what the repair's widening would have been. Everything after it is about reaching the next such shape before production does.
- **Superset bodies.** The fake picks a tool and a body independently, and every body already carries every tool's fields. Adding `job` and `yield_seconds` as fields keeps that, so no pairing of tools to bodies is needed.
- **Refuse at admission, assert at use.** Full and busy are values a caller handles, so admission refuses. Using a reservation the domain made itself can only go wrong by the domain's own bug, so that use asserts. An assertion at the use fails in every world and in production, naming its cause, where a quiet point sees a sum after the fact, and only in worlds. Tests show what an assertion guards, never trip it (the programming model, 10.4).
- **Slots, not a grammar.** The fake already draws random batches. What it lacked was the run's own ids. Slots reuse the take's delimiters, parts and placeholders, so the fake still invents no application name or body.
- **Every value, not the newest.** A take wants the newest value. A slot wants stale ones too, since that is where the faults are. The scan is bounded and keeps places, not copies, within the fake's admission.
- **The world classifies, not the fake.** A delimiter marks a mention, not a lifetime or an owner. A job can end and still be held, and a number past the largest the model saw may be issued. Only the world knows, so it records each draw's class, `unknown` where it can't tell.
- **The agent in `Parties`.** An agent's choices are a party's actions, as jig's people are owner and member. A part keeps its draws apart from faults' and scheduling's without a new stream.
- **A named draw per request.** Separate generators keep answers apart from faults only while the same requests are made. Keying each answer by its conversation, logical turn and attempt keeps it the same when a fault adds or removes other requests.
- **Pin the capture, not the seed.** A seed's draws are fixed only while the menu and the weights are. A capture keyed as the draws are replays a retried or cut request with its own answer, and stays a scenario whatever the generator becomes. Faults and timing still replay from the seed.
- **Drawn stretches, not drawn stories.** The composed workload's goals expect each story's logical work across restarts and interleavings. A drawn stretch inside a story that still delivers and finishes keeps those expectations, and adds its own.
- **Cells as a family of kinds, paired by a driver.** The ledger and `Sweep` count sets of kinds and know no run, so pairing is a driver's. It keys each park by task, attempt and park number, and injects one cell per park, so two tasks or two parks never pair by accident. The cell is then counted, checked and excluded like any fault, and a campaign counts it as any kind.
- **Relations, not equalities.** A quiet point can hold a relay in flight or an answer retained for replay. An equality would fail valid states, so each relation names the states it allows, and only an orphan fails.
- **Shapes stay abstract, on both sides.** The repository is public, so it holds the vocabulary and the reader, and a deployment's counts stay with its user. The worlds count in the same vocabulary, so the two can be compared, and a missing shape is read against the sample's size, never as proof. The reader runs after landing, since it reads a deployment, and a plan runs only the suites.
- **Only the last goal updates the execution and status files.** Goals that run side by side would otherwise conflict on them (the user, 2026-11-24).

## The brainstorm

Written with the fresh-seed campaigns draft's drafter on 2026-11-26, on the user's word ("you two will brainstorm together, directly"). That draft owns fresh seeds and replay, and this one owns what a seed can draw and what a case checks. Each needs the other: fresh seeds over hand-written agents find nothing new, and generated agents want more seeds than the gate affords. The agreed order: this draft's escaped shape and its checks at the transfer first. That draft's first campaign is temper's engine-domain walking sweep, and its first sweep of generated agents is smith's session world's, after this draft's slots and named draws (A4), so the two never change that world at once.

## The review

One Codex round on 2026-11-26 gave 8 blockers, 9 should-fixes and 1 minor, all taken in. The main ones:
- the session world must offer and decode `job` and yields, not only carry bodies;
- assertions are tested by what they guard, never tripped;
- the activity lacks batches, answer kinds and complete ids, so the reader says `unknown`, and the call rows gain answer kinds;
- cells need a driver keyed by park;
- smith's states at a park are a table with each row's observation point and outcome;
- the composed workload takes drawn stretches inside stories;
- a capture keyed by request replays where a turn list wouldn't;
- the companion's adoption in smith's session world waits for A4;
- the conservation checks sit with each quantity's owner;
- relations allow a valid quiet point's skews;
- the gate's figures go to the user;
- the worlds count shapes in the reader's vocabulary.
