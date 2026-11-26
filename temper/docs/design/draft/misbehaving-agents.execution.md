# misbehaving-agents: execution

2026-11-26. How [misbehaving-agents.md](misbehaving-agents.md) gets built: one milestone of eight goals. It designs nothing: each goal writes its design from the draft as it runs. Plans use the focused and fuzzy suites only, and no goal walks.

## 1. The milestone and its goals

One milestone, *agents that misbehave like real models*. Its goals land into main one by one.

| Goal | Draft parts | Where | Needs |
|---|---|---|---|
| A1, two bodies: a yielding `shell` body and a `job` body in smith's session world's menu; a yielding command's run past its yield; the sweep counting the answers its cases got, and its reach check asking for an unknown job and full cells; the agent, host and local worlds taking the bodies with the shared menu | 4.1 | smith's session world (`smith/tests/session/src/provider.rs`) and its fuzzy suite | — |
| A2, checks at the transition in smith: the rule in skein's programming model; smith's session checking a call's start against its batch's reserved end, a result against its call's room, and the provider credit and transcript room at each transfer; its tools' job cells at insertion and release; a step test for each check showing a breach fails at the transfer | 3.1, 3.2's smith bullets | `skein/docs/foundation/programming-model.md`, 10.4; `smith-domain-session`, `smith-domain-tools` | — |
| A3, checks at the transition in jig and temper: jig's core call records, pending calls and kept parts; temper's forge write room and reservations, branch writer slots and a landing's members; a step test for each check added | 3.2's jig and temper bullets | `jig-domain-core`; `temper-engine-domain-forge` | A2 |
| A4, slots, two seeds and a printed script in skein's fake model: a body's slots by delimiters, filled from every occurrence or a never-issued value, with an `ended` pattern and weights; answers seeded from an agent's part of `Parties`, failures and latency from `Faults` and `Scheduling`; a drawn agent printed as a `Turn` list that replays; smith's random-mode worlds moved to the split, their seeds re-picked only where a reach check misses; A1's `job` body turned into a slot | 4.2 to 4.4 | `skein_fake_llm_domain` and `skein/docs/design/testing/fakes/llm.md`; `skein/docs/design/testing/sweeps.md`, 2; smith's session, agent, host and local worlds | A1 |
| A5, smith's vocabulary: bodies and slots for each tool the worlds grant, weights toward a refused entrance call before a write with reads after, every refusal the tools give reached and counted | 5.1 | smith's session world's menu and its siblings; `smith/docs/design/testing.md`, 4 | A4 |
| A6, generated turns in jig's and temper's worlds: slots over task, question and message numbers, and a script kind of drawn producer turns beside the scripted stories, first in temper's composed workload, then in jig's one-process world; domains' refusals counted as answers reached | 5.2 | `temper/tests/system` and `temper-fake`; `jig/tests/one-process`; `temper/docs/design/testing/workloads.md`, 5 | A4 |
| A7, parks against wakes, and agreements: smith's family of a session's states at a park, drawn in its session, run and agent worlds; the table and its family of cells, injected where both halves fall on one run, in jig's channel and one-process worlds and temper's one-process and system worlds; smith's inspection reading a session's room, open calls and job cells; the agreements across components at quiet points in the composed worlds | 6, 7 | `smith/docs/design/faults.md`; `jig/docs/design/domain/testing-workloads.md`, 4; `skein/docs/design/testing/inspection.md`; `smith/testing/smith-inspect`; the composed worlds | A3, A5, A6 |
| A8, production's shapes: the shape's vocabulary from A5's and A6's words and A7's cells; `tools/shapes/`, reading `temper ctl task activity --json` and counting shapes over a window against what the worlds reach; its report kept outside the repository | 8 | `tools/shapes/` and its design | A7 |

A1 and A2 start first, side by side: A1 changes smith's session world, and A2 changes smith's domain crates and skein's programming model. A3 follows A2, whose rule it applies. A4 follows A1, whose `job` body becomes its first slot. A5 and A6 follow A4, each in its own tree. A7 joins the halves once A3's checks, A5's vocabulary and A6's generated turns are in. A8 comes last.

## 2. Keeping this file and the status

Only A8, the last, updates this file and the draft's status file, from the milestone's results. A1 to A7 leave both alone, record their design in the design documents they change, with their status files, and say in their results what of the draft they carried.

## 3. With the fresh-seed campaigns draft

- **Its adoption of its case seam in smith's session world** waits for A1, so the two don't change that world's fuzzy suite at once.
- **Its pinning of a generated agent's finding** as its printed script needs A4.
- **Its first workloads** are this milestone's generated agents, and its generated producer for landings and goals uses A4's slots with 5.2's forge vocabulary.
- **If a goal's widened draws pass a tree's fuzzy budget,** the goal goes to the user with its figures, since budgets are the user's (`test-budgets.md`, 2). Gate seeds and counts stay as they are.

## 4. After the milestone

- **The shapes reader's first run** against the reference deployment, on the user's word. Each shape it finds that no world draws becomes a widening, by the fresh-seed campaigns draft's rule for escapes.
