# A job's state

Provisional, 2026-11-14. The read that says what a job not yet ended is
doing: waiting for a runner, blocked by the jobs it needs, or running at
a step. A service reads it when a wait on CI has passed its bound, to
say why in the hold it makes (temper's
`domain/forge/change/repairs.md`, *A stall's hold says where*;
`domain/landing.md`, 4.2). It is a call of the vocabulary (README.md,
3.2), rendered by each dialect (forgejo.md, github.md), and the fake
forge models the states it reads (../testing/fakes/forge.md, 2.3).

## 1. In one page

- **`Read::JobState { attempt }`** asks for one job attempt's state,
  with no log: its `JobState`, the labels a runner must have to take
  it, and the step it is at.
- **Four states:** `Waiting` (no runner has taken it), `Blocked` (the
  jobs it needs have not ended), `Running`, and `Completed`.
- **No log is read:** the job itself, and on Forgejo its run, as `Job`
  reads them (forgejo.md, 1.2), its steps streamed and only the one
  running kept.
- **`MissingJob`,** as `Job`'s (README.md, 3.4): the job absent, or its
  run, head or attempt not the attempt's.

## 2. The call and its answer

- **The call:** `Read::JobState { attempt: JobAttempt }`, in the CI
  family beside `Job`. It reads nothing but the job.
- **The answer:** `Answer::JobState { attempt, state, labels, step }`:
  the attempt asked; `state`, a `JobState`; `labels`, the runner labels
  the job asks for, in the forge's order, each at most 255 bytes and at
  most 16, kept while the answer's bytes hold them (README.md, 3.7),
  those past either bound dropped and `cut` set; `step`, the `Step` (its
  number and name) the job is at while `Running`, the first of its
  steps the forge says is running, none in any other state or when no
  step says so yet.
- **`JobState`:** `Waiting`, `Blocked`, `Running`, `Completed`. A state
  word the dialect does not know reads as `Waiting`: a job the forge has
  not given a runner.
- **`MissingJob`** when the job answers `404`, or its run, its head or
  its attempt is not the attempt's: the job ran again since, as for
  `Job`.

## 3. On Forgejo

- **Its requests:** `GET R/actions/runs/<run>`; `GET
  R/actions/jobs/<job>`, as `Job`'s first two (forgejo.md, 1.2).
- **Its answer:** the run's `commit_sha`, the job's `run_id` and
  `attempt` checked against the attempt's, as `Job` checks them;
  `status`: `waiting` and `unknown` as `Waiting`, `blocked` as
  `Blocked`, `running` as `Running`, and `success`, `failure`,
  `cancelled` and `skipped` as `Completed`; `runs_on` as the labels;
  `steps[]` streamed, the first whose `status` is `running` kept as the
  step, its `number` and `name`.
- **What no run records yet** (forgejo.md, section 2): that a job no
  runner takes is `waiting` with its `runs_on` and no steps, that one
  whose needs have not ended is `blocked`, and that a running job's
  step says `running`. Read from Forgejo's API description and
  sources; each awaits a fact of the conformance (section 6).

## 4. On GitHub

- **Its request:** `GET R/actions/jobs/<job>`.
- **Its answer:** `run_id`, `run_attempt` and `head_sha` checked
  against the attempt's, as `Job` checks them (github.md, 4.1);
  `status`: `queued`, `requested` and `pending` as `Waiting`, `waiting`
  (a deployment's protection rule holds it) as `Blocked`,
  `in_progress` as `Running`, `completed` as `Completed`; `labels` as
  the labels; `steps[]` streamed, the first whose `status` is
  `in_progress` kept as the step.
- GitHub's pending checks name no job (github.md, 4.4), so a service
  reading a GitHub change's pending checks has no attempt to ask about;
  the call serves a run's jobs there, as a landing's gate run names
  them.

## 5. The fake

- **A runner's labels:** a fake repository's workflow job names its
  `runs_on` labels; the world says which labels its runners serve, all
  by default. A job asking for a label no runner serves stays
  `Waiting`, with no steps, its run and its status pending, until the
  world serves the label.
- **Needs:** a job naming jobs it needs is `Blocked` until they end.
- **Running:** a job running has its steps, the one it is at
  `running`, so a job running past a bound reads at its step.
- Each wire face, Forgejo's and GitHub's, answers the job's read with
  its forge's words for these states (sections 3 and 4).

## 6. Open questions

- **The facts:** the conformance's facts for section 3's states are not
  recorded; a live run of the conformance adds them. Until then the
  dialect rests on Forgejo's API description, and the fake follows it.
