+++
id = "tasks-jobs-lifecycle"
part = "job-tasks"
kind = "feature"
needs = ["tasks-jobs-surface"]
files = [
  "jig/crates/jig-domain-tasks/**",
  "jig/tests/tasks/**",
]
effort = "high"
minutes = 150
+++

# A job task's lifecycle in jig's tasks child

## Goal

jig's tasks child carries a job task through its lifecycle (`jig/docs/design/domain/tasks.md,
5.3.1`): a procedure's step makes it with its `JobSpec` and a `Job` contract; it is due once its
dependencies are done, claimed with nothing reserved, running, and done with a `TaskResult::Job`
checked against its contract; it fails by class, is held past its tries, refuses words,
questions and amendments, and a cancel ends it at once. This lifts `tasks-jobs-surface`'s inert
refusal as far as the tasks child's own rules go: the kind, its bounds and the creator's
delegation are the core's checks (`core-jobs-admit`), and nothing here places or runs a job.

Acceptance:

- **Admission** (`src/batch.rs`, `check_members`, the executor match where the surface refuses
  `Executor::Job`): a `New` whose executor is `Job { kind }` is admitted only when all hold, each
  with the refusal named:
  - its creator is `Party::Task(n)` and task `n`'s executor is `Executor::Procedure { .. }`;
    any other creator (a person, the deployment, an agent, a person task, a job) is refused
    `Refusal::Executor`;
  - `spec.job` is `Some(JobSpec { bytes, deadline })` with `bytes.len() <= limits.spec_bytes`
    and `deadline` nonzero, and `spec.words`, `spec.parameters` and `spec.inputs` are empty
    and `spec.asks` is false: else `Refusal::Spec`;
  - its contract is `Contract::Job { bytes }` with `bytes <= limits.result_bytes`: else
    `Refusal::Contract`;
  - `llm` is `None`, `recurring` is `None`, `flags.conversation` is false and
    `flags.ended_by` is `EndedBy::Result`: else the refusal the existing check gives a
    procedure's member for the same fault (`llm::check`, `valid_recurring`, `flags::valid`).
  A `spec.job` on any other executor stays refused `Refusal::Executor` (the surface's), and a
  `Contract::Job` on any other executor is refused `Refusal::Contract`. A batch whose creator
  is a job task is refused `Refusal::Executor`: a job delegates nothing (`jig/docs/design/
  domain/core.md, 3.2`). The kind is not checked here.
- **Restore** (`src/stored.rs`, `src/stored/fit.rs`): a live or ended job row is restored when
  it holds the admission rules above (its executor, spec and contract), and refused as the
  restore refuses any other malformed row when it does not.
- **Due and active:** a waiting job becomes `Active(Due)` when its last dependency is done, as
  any task does; `activate` and `run_context` (`src/domain.rs`) give a job's `RunContext`, so
  `Request::Activate` goes out for it; `run::prepare` takes a job from `Due` to `Preparing`.
- **Claim** (`src/run.rs`, `claim`): a job's claim takes `budget: 0` and reserves nothing;
  `budget != 0` is refused `Refusal::Funding`, and `account: Some(_)` `Refusal::Executor`. Its
  writer slots are taken as any claim's (`writers::claim`). `started` is as a run's.
- **Its terminal** (`run::activation`): `End::Finished { result: TaskResult::Job { bytes },
  cancel_delegates: false }` is checked against `Contract::Job { bytes: bound }` in
  `invalid_result`: `bytes.len() <= bound` is valid; longer is `InvalidResult::Words`, and any
  other result form against a `Job` contract (but `TaskResult::Failure`, valid under every
  contract) is `InvalidResult::Form`, both a failed attempt of class `Invalid` as today.
  `End::Failed(class)` counts a try of its class and backs off, or holds
  `Hold::Failures(class)` past the class's tries (5.5); `End::Refused` pauses without a try. A
  job's `End::Parked`, `End::FinishedWithFollowups`, `End::Exceeded` and `End::Recheck`, as
  received, are refused `Refusal::Executor` and change nothing, the attempt still live.
- **Its result** reaches its requester as any delegate's: `Request::Ended` with
  `Ending::Done(TaskResult::Job { bytes })`, and the procedure's inbox holds a
  `MessageKind::Result(ResultKind::Job)` word whose words are those bytes, which wakes it.
- **No messages** (contracts, section 4): `inbox::check_message` refuses words, questions and
  answers whose recipient is a job with `Refusal::Executor`, after its existing sender and
  project checks (so an unauthorised sender still hears `State`); `control::check_amend_fields`
  refuses every amendment of a job with `Refusal::Executor`, after its `Unknown` and
  `Reference` checks. A held job is released through the release paths as any task
  (`control::release`, a compound's release, an escalation's decision), which are not
  amendments.
- **Cancel ends it at once:** `grace::check_finish` takes a job always to `FinishPlan::Cancel`,
  so a cancel with a grace cancels it with no `FinishRequested` word; a cancelled job with a
  live attempt closes at `Stage::Run { attempt }` until its terminal is heard (5.1, step 1), as
  any live run does; its ancestors' cancels reach it as any descendant.
- The surface's unit test `a_batch_naming_a_job_is_refused_until_jobs_are_admitted` is replaced
  by the admission tests below. The stories below pass; every existing test of
  `jig-domain-tasks` and `jig-tasks-world` passes unchanged.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md`; `notes/contracts.md`, section 4 (the
  executor, `JobSpec`, contract, result, failure classes, the job's answer as an `End`).
- `jig/docs/design/domain/tasks.md, 5.3.1` (job tasks), `5.1` (phases; closing's step 1),
  `5.5` (tries and holds), `5.6` (results), section 7 (amending, cancelling) and `8.2` (the
  inbox's entrance); `jig/docs/design/domain/core.md, 3.2` (a job delegates nothing).
- `tasks-jobs-surface`'s report: its list of the inert arms it wrote in `jig-domain-tasks`
  (find them also with `rg -n 'Job' jig/crates/jig-domain-tasks/src`). Each inert arm on this
  task's path (admission, restore, `run_context`, `activate`, `prepare`, `claim`,
  `invalid_result`, `check_message`, `check_amend_fields`, `check_finish`) takes the behaviour
  above; an inert arm off this path (a view, a wake rule for a message a job never gets) stays
  as the surface wrote it.
- The code: `src/batch.rs` (`check_batch`, `check_members` at about 200, its executor match at
  about 261, `valid_spec` at about 367, `valid_contract` at about 404, `check_placed`),
  `src/run.rs` (`prepare` at about 43, `claim` at about 71, its budget check at 94-100,
  `invalid_result` at about 174, `activation` at about 353), `src/domain.rs` (`activate` at
  about 908, `run_context`), `src/inbox.rs` (`check_message` at about 299), `src/control.rs`
  (`check_amend_fields` at about 417, `check_cancel` at about 1071), `src/grace.rs`
  (`check_finish` at about 31), `src/flags.rs` (`shaped`, `valid`), `src/procedure.rs`
  (a step's batch, `make_admitted`).
- The world: `jig/tests/tasks` (package `jig-tasks-world`; `src/world.rs`'s `World`, `task`,
  `make`, `claim`, `claim_budget_writing`, `terminal`, `settle`, `restart`;
  `tests/suite/procedures.rs`'s `step` and
  `a_procedure_steps_through_delegates_to_its_result` are the precedent for a procedure's
  delegate).

## Owns and may change

Owns: `jig/crates/jig-domain-tasks/**`, `jig/tests/tasks/**`.

Nothing outside them. If a change here would need another crate's code to change (a core arm
that the new admission makes reachable and that no longer compiles or no longer holds its
tests), stop and say so in the report: `core-jobs-admit` and `core-jobs-run` own the core.

Pre-approved options:

- **Where the job rules sit:** a function of their own (`fn job_member(..) -> Result<(),
  Refusal>`, in `batch.rs` or a new `src/jobs.rs` module, `pub(crate)`), called from the
  executor match, and reused by restore's fit check; or inline arms. No new public item.
- **Order of refusals:** the creator check first, then spec, then contract, so each story's
  refusal is the one named; where an existing check (`flags::valid`, `valid_recurring`,
  `llm::check`) runs earlier in `check_members` and refuses first, keep its refusal and say so.
- **The tasks world's referee** (`src/referee.rs`): if a rule assumes every claim reserves a
  budget or every active task has an agent's run, narrow it to agent tasks and say so; add no
  job rule beyond what the stories need.
- If the procedure's inbox word for a job's result is built elsewhere than the surface put it
  and its words are not the result's bytes, make them the bytes, within the room a delegate's
  result keeps (8.2), and say so.

## Steps

1. Admission and restore: the rules above; replace the surface's batch test with these unit
   tests, beside it, each citing `jig/docs/design/domain/tasks.md, 5.3.1`:
   - `a_procedures_batch_admits_a_job_with_its_spec_and_contract`;
   - `a_job_is_refused_unless_a_procedure_makes_it` (a person's Create, an agent's, a job's
     own batch: each `Refusal::Executor`);
   - `a_job_without_its_spec_or_with_words_is_refused` (no `spec.job`, words present, an
     input, `asks`, a zero deadline, bytes one past `spec_bytes`: each `Refusal::Spec`);
   - `a_job_contract_must_be_a_job_and_within_the_result_bound` (a `Report` contract on a
     job; a `Job` contract on an agent; `bytes` one past `result_bytes`: each
     `Refusal::Contract`);
   - `a_job_is_never_a_conversation` (`flags.conversation` true refused).
2. Activation, claim, terminal and result, as above.
3. Messages, amendments, cancel, as above.
4. Stories in `jig/tests/tasks/tests/suite/jobs.rs` (a new module, added to `main.rs`), each
   citing `jig/docs/design/domain/tasks.md, 5.3.1` (and 5.5 where it holds), with a procedure
   made as `procedures.rs` makes one and a job built from `task(..)` with `executor: Job {
   kind: 7 }`, `spec.job` some, an empty `spec.words`, `contract: Job { bytes: 16 }` and
   zero `numbers`:
   - `a_procedures_job_is_due_once_its_dependency_is_done_and_ends_with_its_result`: one step
     makes an agent delegate and a job depending on it; the job waits; the agent finishes and
     settles; the job is `Active(Due)` with its activation out; claimed with budget 0 (its
     `reserved` stays 0), started, finished `TaskResult::Job { bytes: b"result" }`, settled:
     ended done; the procedure is due with a `Result(ResultKind::Job)` word holding
     `b"result"`; `world.restart()` restores the records unchanged.
   - `a_job_result_past_its_contract_is_an_invalid_attempt`: 17 bytes against 16: a failed
     attempt of class `Invalid`, its tries counted.
   - `a_failing_job_is_retried_within_its_tries_then_held_and_its_procedure_hears_it`:
     `End::Failed(Class::Run)` until past `LIMITS.retries.run`; held `Hold::Failures(Run)`;
     the hold escalates to the procedure, which is woken; then a release by the procedure
     returns it to due.
   - `words_questions_and_amendments_to_a_job_are_refused_as_its_executor`: a procedure's
     message, a question, an amendment (spec, and dependencies): each `Refusal::Executor`, the
     job's record unchanged.
   - `a_cancel_ends_a_running_job_at_once_even_with_a_grace`: the procedure cancels its
     running job with a grace; no `FinishRequested` word; the job closes at `Stage::Run`; its
     terminal heard, it ends cancelled; and a cancel of the procedure's requester's subtree
     ends a waiting job at once.
   - `a_job_claim_reserves_nothing`: a claim with budget 1 is refused `Refusal::Funding`; with
     0 it is claimed.
5. Narrow tests first (`heavy cargo nextest run -p jig-domain-tasks --lib -E 'test(/job/)'`;
   `heavy cargo nextest run -p jig-tasks-world --test suite -E 'test(/jobs::/)'`; check each
   count), then `cargo fmt`; `heavy cargo clippy -p jig-domain-tasks -p jig-tasks-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-tasks -p
   jig-tasks-world`.
6. Readers whose meaning this changes (the tasks child now emits a job's activation and admits
   its batch): `heavy cargo nextest run -p jig-domain-core -p jig-core-world`, then `heavy cargo
   check --tests -p jig-domain-people -p jig-protocol-store -p temper-engine-domain`.
7. Commits: admission and restore, then the lifecycle, then the stories; push your branch.

## Report

As the implementer's text says, and also: each inert arm of the surface you replaced, by file
and function, and each you left; where the job rules sit; any refusal order that differs from
the acceptance's; any referee rule narrowed.
