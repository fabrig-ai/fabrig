# CI on Forgejo v17: re-runs, stalls and the agents' log tools

Draft, 2026-10-16, revised 2026-10-18. What temper takes from Forgejo
v17's Actions beyond what the design adopts. The design reads a failed
job's failed step and the end of its log (`../protocol/forge.md`, 3.2,
`Read::Job`; `../domain/forge.md`, 8.4), and hints CI by the jobs'
deliveries (`../protocol/forge.md`, section 5). This draft adds three
steps, each a plan of its own: a failure the runner caused is run again
instead of repaired (section 2), a stall says why CI never reported
(section 3), and an agent reads the log it needs (section 4). Section 5
says what each changes in the design, section 6 which facts each needs
checked, section 7 their order, and section 8 what is open. Nothing here
is adopted.

## 1. In one page

- **Run again what the runner broke; repair what the change broke.** A
  job whose set-up step failed, whose completion step failed with every
  workflow step passed, or which failed with no step failed (its runner
  lost) failed for the forge, not for the change. The change runs it
  again, through Forgejo's API, before it asks for a repair: at most
  `reruns` times at one head, 1 by default. A failure in the workflow's
  own steps is a repair, as the design has it.
- **A re-run keeps what the head earned.** It moves no head: the gates'
  verdicts stay valid, no repair is counted, and the wait on CI counts
  its stall afresh from the re-run.
- **A re-run is found by a read.** Forgejo keeps no key for it, so after
  an uncertain failure the change reads the job: an attempt past the one
  it re-ran says the re-run was made.
- **A stall says why.** A change held because its CI never reported says
  what Forgejo shows of each pending job: waiting for a runner with its
  labels, blocked by the jobs it needs, or running past the stall. The
  person told knows whether to start a runner or to look at the job.
- **An agent reads the log it needs.** The agent's `job` tool takes a
  step, to read another step of the attempt, and a search, which answers
  the log's lines holding a string; each answer is cut to the tool's
  limit.
- **Webhooks need nothing more.** The job deliveries
  `../protocol/forge.md`, section 5 subscribes to hint CI as each job
  ends, its re-runs included.

## 2. Running again what the runner broke

### 2.1 Which failures are the runner's

From the job's steps, which `Read::Job` reads for the brief
(`../protocol/forge.md`, 3.2): step 0 is the runner's set-up (`Set up
job`), the last its completion (`Complete job`), the workflow's own
steps between.

- **The runner's:** the failed step (`../protocol/forge.md`, 3.2: the
  first whose status is `failure`) is the set-up step (the runner could
  not prepare the job: its image, its workspace, the actions it
  fetches), or the completion step, every workflow step having passed;
  or the job failed and no step did (the runner was lost, or the job
  timed out before a step ran). The completion step reports `failure`
  whenever the job fails, so only the first failed step tells.
- **The change's:** a failure in any of the workflow's own steps. It is a
  repair, as `../domain/forge.md`, 8.4 says.
- **Never by the log's words.** Nothing reads a log to guess at a flaky
  test or a network failure inside a step: that is an agent's judgement
  (section 8).
- **A cancelled job is not re-run:** a person or a policy cancelled it.

### 2.2 The re-run

- **When:** CI on the head fails (`../domain/forge.md`, 8.2, step 3). The
  change reads each failed check's job, as the brief would. When every
  failed check is the runner's and the head's re-runs are fewer than
  `reruns`, it re-runs those jobs; otherwise it asks for a repair, as
  the design has it, whose brief says which failures were the runner's
  and which jobs were re-run and failed again.
- **The write:** `Write::Rerun { attempt }`, `POST
  R/actions/jobs/<job>/rerun`, which runs the job again with the jobs
  that need it, and answers `204`. Forgejo refuses a job that has not
  completed (`400`).
- **Its effect:** a transition, conditional (jig's
  `domain/connectors.md`, 4.3): the job's next attempt, made by the
  forge. Found again after an uncertain failure by reading the job: an
  `attempt` past the one named says it was made; the same attempt,
  completed, says it was not, and the write may be sent again, within
  `reruns`.
- **Its authority:** the `push` kind on the change's branch, since a
  push of the same head would run the same CI (`../domain/forge.md`,
  section 2; jig's `domain/authority.md`, section 4). Forgejo asks the
  API token's `write:repository` scope, which the bot's token has
  (`../protocol/setup.md`, section 4).
- **Counted at the head.** The change keeps its re-runs at its head; a
  new head starts at none. `reruns` is one of the change's bounds
  (`../domain/forge.md`, 8.1), with repairs and resolutions.
- **What it keeps:** the head, so every gate's verdict stays valid
  (`../domain/forge.md`, 8.3), and no repair is counted. The stall of the
  wait on CI counts from the re-run.
- **What CI reads next:** the job's status on the head, and the head's
  combined state, are pending again until the new attempt ends, and the
  status's link is unchanged (section 6); `Read::Checks` answers the new
  attempt, so a later repair's brief reads its log. The job keeps its
  id, and has no steps while it waits for a runner.
- **What people see:** the change's record says each re-run, its job and
  its reason (`set-up failed`, `completion failed`, `no step failed`).

## 3. A stall's reasons

- **When:** the stall of the wait on CI passes (`../domain/forge.md`,
  8.4). Before the change is held, it reads the job of each pending
  check whose link names one (`../protocol/forge.md`, 3.2, "Job
  attempts"):
  `GET R/actions/jobs/<job>`, its `status`, its `runs_on` labels, and,
  when it runs, the step it is at.
- **The hold's reason** names each pending job and what Forgejo shows:
  `waiting` (no runner with its labels took it), `blocked` (the jobs it
  needs have not ended), or `running` (at which step). A pending check
  that links no job is CI another system reports, and the reason says so.
- **Its cost:** one read per pending job, only when the stall passes;
  nothing on the periodic path.

## 4. The agents' log tools

The agent's `read_job` tool (`../domain/agent.md`, 4.3) reads the
failed step's end, as the brief does. It grows:

- **A step:** the end of the named step's log, of the same attempt; each
  answer lists the job's steps (number, name, status), so an agent can
  choose another.
- **A search:** `GET R/actions/jobs/<job>/logs?attempt=<a>&q=<string>`,
  with `qi=true` when the agent asks to ignore case, and `step` when it
  names one: the lines that hold the string, from the first, as many as
  the tool's limit holds, `truncated` when more came. Forgejo answers a
  search without ranges, so the component reads up to the limit and
  closes the connection (`../protocol/forge.md`, section 8: a raw body
  past `max_bytes`).
- **Not NDJSON:** its times are nothing an agent needs, and it answers no
  range either.

## 5. What it changes in the design

- **`../protocol/forge.md`:** 3.2, `Write::Rerun` and `Read::Job`'s
  step and search; a read of a pending job's state for a stall
  (`Read::Checks` resolving the pending checks' jobs when asked, or a
  read of its own); 3.4, the re-run's `400` as `Refused`; section 9, the
  fake forge's re-runs through the API as new attempts, `waiting` and
  `blocked` jobs, and searches; section 13, the facts of section 6 here.
- **`../domain/forge.md`:** section 6, the re-run's effect and how it is
  found again; 8.1, `reruns` among the bounds; 8.2, step 3, the runner's
  failures re-run before a repair; 8.4, a stall's reasons; section 18,
  the world's stories: a set-up failure re-run and passing; one re-run
  that fails again, then repaired; a re-run whose answer is lost, found
  made; a change held waiting for a runner, its reason naming the labels.
- **`../protocol/configuration.md`:** `reruns`, with the change's other
  bounds.
- **The agent's tools** (`../domain/agent.md`, 4.3;
  `temper-engine-smith`'s `read_job`): its step and search arguments,
  and the steps it answers.

## 6. Facts to check on v17.0

By `tools/forgejo-conformance`, as facts `../protocol/forge.md`,
section 13 names, before a plan relies on each:

- a job's re-run: `204`, and `400` while the job or its run has not
  completed; the job keeps its id and its `attempt` moves on, with no
  steps while it waits; the job's status on its head, and the combined
  state, go back to `pending` until the new attempt ends, the status's
  link unchanged; `logs?attempt=<n>` still reads an earlier attempt; a
  token without `write:repository` is refused `403`; a job's re-run
  leaves the jobs that do not need it at their attempt;
- which failures leave no step failed (a runner killed mid-job, a job's
  timeout), and what a cancelled job's status on its head is;
- a job no runner takes: `waiting`, with no steps; one whose needs have
  not ended: `blocked`;
- a search: `200`, plain lines, no `Accept-Ranges`; with `step` and
  `qi`.

## 7. Order

1. **Re-runs** (section 2): first, since each failure the runner causes
   costs a repair's run.
2. **A stall's reasons** (section 3): small, and most useful once people
   run their own runners (`../protocol/setup.md`, sections 4 and 5:
   connected and managed).
3. **The agents' log tools** (section 4): with the next change to
   temper's host tools.

## 8. Open questions

- **An agent's judgement of flakiness:** whether a repair's agent may
  answer that a failure is flaky and ask for a re-run instead of pushing
  a head. Default: no; a repair pushes a head or fails, and section 2.1
  decides re-runs.
- **The completion step:** whether its failure, with every workflow
  step passed, is the runner's; it runs the post-steps of the actions a
  workflow used, a cache's upload among them. Default: yes; a project
  whose post-steps matter could name it otherwise.
- **A whole run re-run** (`actions/runs/<run>/rerun`): not used. Jobs are
  re-run one by one, each with the jobs that need it.
- **`reruns` per project:** a project with an unreliable runner may want
  more. Default: one bound for every change, in configuration.
