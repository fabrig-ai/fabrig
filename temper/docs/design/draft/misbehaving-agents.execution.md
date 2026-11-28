# misbehaving-agents: execution

2026-11-28. How [misbehaving-agents.md](misbehaving-agents.md) gets built: one milestone of eight goals, all landed. What the design carries, what lags and what departs is its status, [misbehaving-agents.status.md](misbehaving-agents.status.md). It designs nothing: each goal writes its design from the draft as it runs. Plans use the focused and fuzzy suites only, and no goal walks.

## 1. The milestone and its goals

One milestone, *agents that misbehave like real models*. Its goals land into main one by one.

| Goal | Draft parts | Where | Needs | State |
|---|---|---|---|---|
| A1, the escaped shape, drawn: smith's session world offering and decoding `job` and a shell's `yield_seconds`; its bodies gaining both fields; a yielding command outlasting its yield through `Settings::tool`; its referee accepting a running job; its sweep counting the escaped shape and its outcome, unknown jobs and full cells, its reach check asking for all three; the agent, host and local worlds taking the shared bodies | 4.1 | smith's session world (`smith/tests/session/src/{provider,translate,world}.rs`) and its fuzzy suite | — | landed |
| A2, checks at the transfer in smith: the rule in skein's programming model; smith's session asserting a result's room against its call's batch, a reclaim's shares given back once, the provider credit and the transcript's room at each use; its tools' job cells at insertion and release; tests of each check's valid edge and of the admission's refusal through their effects, none tripping an assertion | 3.1, 3.2's smith bullets | `skein/docs/foundation/programming-model.md`, 10.4; `smith-domain-session`, `smith-domain-tools` | — | landed |
| A3, checks at the transfer in jig and temper: jig's core admission bound over pending and retained calls, and its kept parts; jig's tasks' branch writer slots; temper's landing child's members; temper's forge room over all its holders, each share against its owner's state; tests as in A2 | 3.2's jig and temper bullets | `jig-domain-core`, `jig-domain-tasks`; `temper-engine-domain-forge-landing`, `temper-engine-domain-forge` | A2 | landed |
| A4, slots, named draws and capture in skein's fake model: slots by delimiters over the take's parts, bounded, distinct values, checked numeric ranges, a never-issued value, sized before copying; answers drawn from an agent's part of `Parties` keyed by conversation, logical turn and attempt, failures from `Faults`, latency from `Scheduling`; a bounded capture that replays by those keys; the session world's renderer showing a job's result and a read's continuation as the product does, and classifying each drawn reference; the pinned cases of smith's random-mode worlds listed and kept, then those worlds moved, their seeds picked again only where a reach check misses; A1's `job` field made a slot | 4.2 to 4.4 | `skein_fake_llm_domain` and `skein/docs/design/testing/fakes/llm.md`; `skein/docs/design/testing/sweeps.md`, 2; smith's session, agent, host and local worlds | A1 | landed |
| A5, smith's vocabulary: bodies and slots for each tool the worlds grant; each targeted refusal listed with its setup and its answer, and reached; weights toward a refused call before a write with reads after; each sweep counting the shapes its cases reached in the shared vocabulary | 5.1, 8's world side | smith's session world's menu and its siblings; `smith/docs/design/testing.md`, 4 | A4 | landed |
| A6, drawn stretches in jig's and temper's worlds: slots over task, question and message numbers; bounded stretches of drawn reading and messaging turns at stated points of a story, through temper's composed typed and wire providers (`temper-fake` taking a menu and rounds), then jig's one-process world; each drawn call answered once and each reachable refusal counted; the shapes reached, counted | 5.2, 8's world side | `temper/tests/system`, `temper-fake`; `jig/tests/one-process`; `temper/docs/design/testing/workloads.md`, 5 | A4; the composed workload's goal of `temper/docs/design/draft/fuzzy-campaigns.md`, when it is scheduled | landed |
| A7, parks against wakes, and relations: smith's family of a session's states at a park, drawn in its run and agent worlds; the table's cells and their driver, keyed by park, in jig's one-process world and temper's one-process and system worlds; smith's inspection adapter built and wired into its session world; the composed worlds' inspection reading jig's core placements and flights; the relations of 7, with tests of their valid skews and of a planted orphan | 6, 7 | `smith/docs/design/faults.md`; `jig/docs/design/domain/testing-workloads.md`, 4; `skein/docs/design/testing/inspection.md`; `smith/testing/smith-inspect`; the composed worlds | A3, A5, A6 | landed |
| A8, production's shapes: the versioned vocabulary from A5's, A6's and A7's words; the activity's call rows gaining each call's answer kind from the archived turn; `tools/shapes/`, reading every page of each task in a window, `unknown` where the activity is incomplete, comparing with the worlds' counts and their samples; tests with a cut page, an unread turn and an id issued before the window, and that nothing raw reaches the report, fixtures or error lines | 8 | `tools/shapes/` and its design; temper's activity rendering (`temper-client-tools-jig`) | A7 | landed |

A1 and A2 start first, side by side: A1 changes smith's session world, and A2 changes smith's domain crates and skein's programming model. A3 follows A2, whose rule it applies. A4 follows A1, whose `job` field becomes its first slot. A5 and A6 follow A4, each in its own tree. A7 joins the halves once A3's checks, A5's vocabulary and A6's stretches are in. A8 comes last.

## 2. Keeping this file and the status

Only A8, the last, updates this file's states and the draft's status file, from the milestone's results (the user, 2026-11-24, for goals that run side by side). A1 to A7 leave both alone, record their design in the design documents they change, with their status files, and say in their results what of the draft they carried.

## 3. With the fresh-seed campaigns draft

The fresh-seed campaigns draft is `temper/docs/design/draft/fuzzy-campaigns.md`.

- **smith's session world** is changed by A1 and A4 here, and by that draft's adoption of its case seam in smith's sweeps. That adoption waits for A4, not only A1, so the world's seeds, reach checks and loop change in one order: A1, A4, then the seam.
- **temper's composed workload** is changed by that draft's goal that makes its draws valid at every seed and takes its sweep through the seam, and by A6 and A7 here. A6 waits for that goal when both are scheduled, since a drawn stretch needs draws valid at every seed.
- **Its first campaign** is temper's engine-domain walking sweep. Its first sweep of generated agents is smith's session world's, once the seam's adoption there follows A4.
- **Its pinning of a generated agent's finding** is A4's capture.
- **The generated producer for landings and goals** is scheduled by neither draft yet (draft, 9).

## 4. Budgets and the gate

- **The gate keeps its seeds and its checks.** A goal whose widened draws change a binary's fuzzy cost gives its seed and check counts, CPU and instructions before and after, by binary.
- **Where reach would need more gate seeds,** other checks, or a budget figure moved, or where a growth allowance stops a goal below its ceiling, the goal holds that part and goes to the user with the figures, since budgets are the user's (`test-budgets.md`, 2).
- **Seeds picked again never drop a pinned behaviour:** that is kept as a scenario first (4.3).
- **A cell joins a gate sweep** only where its held seeds show it falls. The rest wait for campaigns.

## 5. After the milestone

- **The shapes reader's first run** against the reference deployment, on the user's word. Each shape it finds that the worlds' samples didn't reach becomes a widening, by the rule for escapes (`temper/docs/design/draft/fuzzy-campaigns.md`, 8).
- **The worlds' run fields:** a sweep that runs smith under jig's wakes counting each shape's run end, begin and wake (`smith/docs/design/shapes.md`, 7), the draft's one lag left. No goal is scheduled for it.
