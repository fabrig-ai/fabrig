# Parks in smith's worlds: a session's state at a park

Provisional, 2026-11-27. The family of kinds that names what a run's session holds as its park is answered, what the design says each state becomes if the run resumes and if it ends, and where smith's worlds draw them. Written from `temper/docs/design/draft/misbehaving-agents.md`, 6.1. The park's other half, the wake that follows it, is jig's family of runs that park, resume and move (`jig/docs/design/domain/testing-workloads.md`, 4), and the cells that pair the two halves for one park are jig's (`jig/docs/design/domain/testing-cells.md`). The ledger, streams and sweeps are skein's (`skein/docs/design/testing/sweeps.md`), as for smith's faults (`faults.md`).

## 1. In one page

- **A park,** in smith's worlds, is an activation's end that a later activation of the same run may resume: a parked answer, or a turn cut by a drain at the answer or by a crash (`domain/turn-parts.md`, 4 and 5).
- **Six kinds** (section 2), family `a session's state at a park`, each a state observed as the park is answered, with its outcome if the run resumes and if it ends. Three fall at any park; three only at a cut, since a parked answer tells its turn whole.
- **Drawn** (section 3) in the agent world, which parks a real session and resumes it through a new activation, all six; and in the run world, the five its run domain holds against its fake session. Each at the seeds its sweeps already hold.
- **The session world** (section 4) drives what a session alone does at a park, its close and its restore, in a scripted story for each state it holds.

## 2. The kinds

`smith_faults::parks`, `FAMILY = "a session's state at a park"`, in the kit of the tree whose mechanism makes them (`smith/testing/smith-faults`):

| Constant | Name | Observed | If the run resumes | If it ends |
|---|---|---|---|---|
| `WITHHELD_WAIT` | `a withheld wait` | the last turn told without the `wait`'s result | the `wait` answered by the waking messages, once (`domain/run-messages.md`, 1) | nothing owed |
| `JOB_RUNNING` | `a job running` | the job's end told by the session's close, before the park's answer | its number unknown to the resumed kit: a `job` naming it is refused at the tools' entrance (`domain/tools.md`, 5) | ended already |
| `CUT_TURN` | `a cut turn with kept parts` | the parts told before the cut | rebuilt from its eligible parts, with no completion asked for it, else planned afresh (`domain/turn-parts.md`, 5) | parts dropped with the task |
| `HOST_CALL_ANSWERED` | `a host call answered` | its answer told: in the turn's whole record, or in its result part at a cut | its answer comes from the start's answered calls, and the call isn't asked again (the same, 5) | dropped with the task |
| `HOST_CALL_UNANSWERED` | `a host call unanswered` | no result told for it | asked again once, under the new activation's name for it (the same, 5; `domain/run.md`, 5.2) | dropped with the task |
| `SUB_AGENT_KEPT` | `a sub-agent's conversation kept` | its parts told | continued from its eligible parts, with no completion asked for an answer they hold, its new parts under the same call position (the same, 6) | dropped with the task |

- **The draft's one row for a host call is two kinds,** since its two outcomes differ and a kind carries one expectation.
- **Which park each needs:** `WITHHELD_WAIT`, `JOB_RUNNING` and `HOST_CALL_ANSWERED` fall at a parked answer or at a cut. `CUT_TURN`, `HOST_CALL_UNANSWERED` and `SUB_AGENT_KEPT` fall only at a cut: a run parks once its turn is told whole, without the withheld `wait`'s result alone (`domain/run-messages.md`, 1), so a parked answer leaves no cut turn, no host call without its answer and no sub-agent's conversation to go on. jig's pairing relies on this (`jig/docs/design/domain/testing-cells.md`, 3).
- **"Dropped with the task"** is the host's. In smith's worlds a run that ends asks nothing more of anyone: no completion, no host call, no job, and nothing told after its answer.
- **A park may hold several states at once** (a `wait` beside a yielding shell and a host call). Each state that falls is recorded; a world that arms one for a cell names only that one to the cell's pairing (`jig/docs/design/domain/testing-cells.md`, 4).

## 3. Where they are drawn

- **The agent world** (`smith/tests/agent`) draws all six. In its sweep that parks runs and resumes them through a new activation from their transcripts, parts and answered calls, each parked run draws, from `Faults`, part `parks`, one kind and whether its run resumes or ends. Its script arms the state: a `wait` with nothing queued; a yielding shell before the `wait`; a host call in the turn before the `wait`; or a drain at the answer, with a grace, or a crash, while main's calls run, while a host call is unanswered, or while a sub-agent runs. The world records the kind as it falls, at its observation point, with `FaultLedger::inject`, its request naming the run as its host numbers it (`run 1`), and checks the kind's outcome: resuming, in the next activation; ending, that nothing is asked after the answer.
- **The run world** (`smith/tests/run`) draws the five its run domain holds against its fake session: `WITHHELD_WAIT`, `CUT_TURN`, `HOST_CALL_ANSWERED`, `HOST_CALL_UNANSWERED`, and `SUB_AGENT_KEPT` where its partner runs a child. It has no tools, so no `JOB_RUNNING`, and resumes no activation, so it checks the observed column and the ends column alone, in its sweeps that park and cut runs.
- **Their own part.** Each kind keeps a fixed place in part `parks`, so a sweep's other draws stay as they were, and a kind a run can't take (its charter grants no host call or `sub_agent`) is recorded ineligible (`FaultLedger::ineligible`) and spends its place.
- **At the gate,** at the seeds each sweep already holds (`temper/docs/design/draft/misbehaving-agents.execution.md`, 4): a sweep configures each kind its held seeds make fall. A kind they don't reach is named, with why, beside its `configure` calls, and its figures go to the user, since more seeds are theirs to give.

## 4. The session world's close and restore

The session world (`smith/tests/session`) closes a session and restores one from its transcript, as a run's park and its resume do to the session. A scripted story for each state a session holds at its close, each checking the row's outcome at the session's edge:

- **a withheld `wait`:** restored, only its missing result supplied, once;
- **a job running:** told ended at the close, before the session ends; the restored session numbering its next job after the last its transcript holds, the old number unknown;
- **a cut batch:** closed while its batch runs, then restored with that batch's answer and the results the close left, its unanswered calls run, no completion asked for the answer;
- **a host call's committed result:** restored from the transcript with no ticket of its own;
- **a sub-agent's session:** closed with its history and restored to go on from it.

A story the world already holds for one of these is kept and named in its place, not written again.

## 5. Testing

Focused and fuzzy suites only.

- **The kinds,** in `smith-faults`' own tests: names unique within the family, each kind's family `FAMILY`.
- **Each sweep** (section 3), in its world's fuzzy suite: each kind it configures fell, and each outcome held, its finding naming the run, the kind and the outcome that failed.
- **The session world's stories** (section 4), in its focused suite.

## 6. Later

- The kinds no held seed reaches, configured in a sweep a campaign runs (`temper/docs/design/draft/fuzzy-campaigns.md`).
