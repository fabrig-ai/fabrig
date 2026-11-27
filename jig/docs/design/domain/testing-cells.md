# Testing the domain: parks against wakes

Provisional, 2026-11-27. The cells: each pair of a kind of jig's runs that park, resume and move (`testing-workloads.md`, 4) and a kind of smith's session states at a park (`smith/docs/design/parks.md`, 2), for the same park, as kinds of a family of their own; the pairs that can't occur; and the pairing that a driver in each world running both halves keys by park. Written from `temper/docs/design/draft/misbehaving-agents.md`, 6.2. jig names smith and temper names jig, so the cells are jig's, in `jig_core_kit::cells`, beside the parks' kinds.

## 1. In one page

- **A cell** is one kind of each half for the same park: one run, one attempt's park, and the answer or end that follows it, in that attempt or its successor.
- **Its expectation** is smith's row under jig's column (section 2): what smith's state becomes when the wake jig's kind names resumes the run, or ends it without a resume.
- **The cells are kinds** of the family `parks against wakes`, one kind per cell (section 2). A pair that can't occur is listed with its reason (section 3).
- **The pairing** (section 4) keeps each park's halves by its task, its attempt and the park's number, and injects the cell's kind once per park, when both halves have fallen for that key. The ledger keeps sets of kinds and knows no run (`skein/docs/design/testing/sweeps.md`, 3), so the pairing is the driver's.
- **Where they're drawn** (section 5): the worlds that run real smith under jig's wakes, jig's one-process world and temper's one-process and system worlds. At the gate a sweep configures a cell only where its held seeds show it falls.

## 2. The cells

jig's half is its park kinds, each with its column:

| jig's kind | Column | Why |
|---|---|---|
| `ANSWERED_WHILE_PARKED` | resumes | the answer wakes it, and its next run reads it once |
| `ANSWERED_TO_ITS_SUCCESSOR` | resumes | another wake resumes it first; the answer reaches that run |
| `PARKS_ON_NOTHING` | resumes | Nothing left to await makes it due again, its notice the waking words (`tasks/lifecycle.md`, 2) |
| `WATCHES_A_TASK_THAT_CAN_ANSWER` | resumes | its watch's notice wakes it |
| `ANSWERED_AFTER_ITS_END` | ends | its task ends first, and the answer is dropped |
| `COORDINATOR_ENDS_WHILE_A_CHILD_WAITS` | ends | the child's task ends with its coordinator's |
| `STUCK_IN_A_CIRCLE` | ends | the goal that closes the circle is held, never resumed by it |

smith's half is the three states a parked answer can leave, `WITHHELD_WAIT`, `JOB_RUNNING` and `HOST_CALL_ANSWERED` (`smith/docs/design/parks.md`, 2), each with its outcome under each column:

| smith's kind | Resumes | Ends |
|---|---|---|
| `WITHHELD_WAIT` | the resumed attempt's first request carries the `wait`'s result with the waking messages, once, and no other attempt answers it | nothing owed: no attempt answers the `wait` |
| `JOB_RUNNING` | the job's end told before the park's answer; a `job` naming its number in the resumed attempt is refused as unknown, and the run goes on | the job's end told before the park's answer, nothing of it after |
| `HOST_CALL_ANSWERED` | the call's effect made once; its answer reaches the resumed attempt from the transcript or the start's answered calls, and the call isn't asked again | its retained record goes with the task's end (`engine/decisions.md`, 1.4), kept while the task is held |

- **21 cells,** one for each of the three smith kinds and the seven jig kinds. Each constant names both halves, smith's first, `JOB_RUNNING_PARKS_ON_NOTHING`, and its name joins theirs with a comma: `a job running, a run parks with nothing to await`. The kit holds them in one list, each with its two halves and its column.
- **The incident's cell** is `JOB_RUNNING_PARKS_ON_NOTHING`: a run with a job running parked with nothing to await, was woken by Nothing more will come, and its resumed model named the ended job (`temper/docs/design/draft/misbehaving-agents.notes.md`, *The incident and its fix*).

## 3. Pairs that can't occur

- **`CUT_TURN`, `HOST_CALL_UNANSWERED` and `SUB_AGENT_KEPT` with any jig kind:** each jig kind falls on an attempt that answered parked and left its task idle (`testing-workloads.md`, 4), and a parked answer tells its turn whole, so it leaves no cut turn, no host call without its answer and no sub-agent's conversation to go on (`smith/docs/design/parks.md`, 2). These states follow a drain or a crash, which jig takes as a suspended or lost attempt, never a park.
- **`PARKS_WITH_A_QUESTION_OPEN` with any smith kind:** it is the park that `ANSWERED_WHILE_PARKED`, `ANSWERED_TO_ITS_SUCCESSOR`, `ANSWERED_AFTER_ITS_END` and `COORDINATOR_ENDS_WHILE_A_CHILD_WAITS` follow, so each such park's cell is its outcome's.

The kit lists these 27 pairs with their reasons, and a lookup of one gives its reason, never a cell.

## 4. The pairing

- **A park's key:** its task's number; its attempt, as jig numbers it; and the park's number, its ordinal among its task's parks from 1, counted by the world. An attempt ends at its park, so the two move together; keying by both means that neither a per-deployment attempt nor a per-task count can pair two parks.
- **Each half as it falls:** the driver tells the pairing each half it armed, by the park's key and the half's kind, at the moment it observed it. The halves' own kinds are recorded in their own families by the world as before; the pairing adds only the cell.
- **Once both halves are in** for one key, the pair is looked up. A cell's kind is injected into the ledger at that moment, `FaultLedger::inject`, its request naming the park's task (`task 12`), once for that key, however often a half is told again. A pair that can't occur is a finding naming the park, both kinds and its reason. A second, different kind of one half for a key is a finding: a driver arms one of each.
- **Nothing pairs across keys:** two tasks that hit opposite halves make no cell, and neither do two parks of one task, each with one half.

## 5. Where they're drawn

- **A driver in each world that runs real smith under jig's wakes:** jig's one-process world (`../protocol/service-testing.md`, 1), and temper's one-process and system worlds (`temper/docs/design/testing/tiers.md`, 1.5; `temper/docs/design/testing/workloads.md`, 5). Each chooses an eligible target from its own draw; arms both halves for it, smith's by the target's script and jig's by the answers, ends and wakes the world gives it; observes each half where it falls, smith's in the agent's turns and answer, jig's in the committed rows; feeds the halves it armed to a pairing; and checks the cell's expectation (section 2). A finding fails the run.
- **jig's one-process world:** a focused story, at the world's fixture seed. A person's chat delegates a child, whose script calls `wait` with nothing queued, and parks with nothing to await: `WITHHELD_WAIT_PARKS_ON_NOTHING`, or `JOB_RUNNING_PARKS_ON_NOTHING` where the world's agent runs a yielding shell first. jig wakes it with Nothing left to await's notice, its resumed run reads it once and finishes, and the story's ledger holds the cell once, its expectation held. The world's fuzzy sweeps park no delegate, so none configures a cell there.
- **At the gate,** a sweep configures a cell only where its held seeds show it falls, within their sample. The composed sweep draws one (`temper/docs/design/testing/workloads.md`, 8.2). The rest wait for a sweep a campaign runs, which counts them as any kind (`temper/docs/design/draft/fuzzy-campaigns.md`).

## 6. Testing

Focused and fuzzy suites only.

- **The cells,** in the kit's own tests: 21 cells and 27 impossible pairs, every pair of the two halves exactly one of them; names unique within the family; each cell's name its halves' names joined; each column as section 2 gives it.
- **The pairing,** in the kit's own tests, over a ledger it makes, with no world: two tasks that hit opposite halves at one moment make no cell; two parks of one task, each with one half, make none; one park with both makes its cell once, however often each half is told again; an impossible pair, and a second kind of one half, are findings naming the park.
- **Each world's driver,** in its world: its story or sweep's ledger holds each cell it armed once, and the cell's expectation held.

## 7. Later

- The cells no held seed reaches, configured in a sweep a campaign runs.
- A driver in jig's core world, once a fake run there holds smith's states: its scripted run answers hold none today.
