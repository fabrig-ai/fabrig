+++
id = "core-jobs-admit"
part = "job-tasks"
kind = "feature"
needs = ["tasks-jobs-surface"]
files = [
  "jig/crates/jig-domain-core/**",
  "jig/testing/jig-test-connector/**",
  "jig/testing/jig-test-domain/**",
  "jig/tests/core/**",
]
effort = "high"
minutes = 130
+++

# The core admits a procedure's job: declared kinds, their bounds, a delegation's job kinds

## Goal

jig's core takes the application's job kinds in its `Config` and lets a procedure's step make a
job as a delegate (`jig/docs/design/domain/engine.md, 7.5`; `jig/docs/design/domain/tasks.md,
5.3.1`): the core checks its kind is declared and its spec and deadline within the kind's
bounds, sets its `Job` contract from the kind's result bound, and the creator's delegation
answers propose or refuse for a kind it lacks (`jig/docs/design/domain/authority.md, 8.1`).
Admission only: no job is activated, claimed or placed here (`core-jobs-run`). This task runs
beside `tasks-jobs-lifecycle`, which lifts the tasks child's own refusal of a job: until both
land, a job that passes every core check is still refused by the tasks child, so this task's
stories are the core's refusals, and its passing path is a unit test of the core's check.

Acceptance:

- `jig_domain_core::Config` gains `pub jobs: Box<[jig_domain_link::JobBound]>` (contracts,
  section 5). `Core::new` asserts, as it asserts the rest of its configuration, that the kinds
  are ascending and each once, and that each kind's `spec` is at most `limits.tasks.spec_bytes`
  and its `result` at most `limits.tasks.result_bytes` (the tasks child's bounds for a job).
  The core keeps them for its checks. Every `Config` literal gives `jobs: Box::new([])`
  (temper's engine and `jig-protocol-config` give none); the core world's restart oracle copies
  `jobs` with the rest of its configuration.
- **A procedure's step** (`src/routing/run/delegation.rs`, `procedure_step`, before
  `procedure_batch_admit`): for each member whose executor is `tasks::Executor::Job { kind }`,
  - its kind is one `Config::jobs` declares, else the step is refused `StepRefusal::Refused`
    with `tasks::Refusal::Executor` naming the procedure task;
  - its `spec.job` is present, with `bytes.len()` at most the kind's `spec` and `deadline` at
    most the kind's `deadline`, else refused `tasks::Refusal::Spec` the same way;
  - its `contract` is set to `tasks::Contract::Job { bytes: bound.result }`, whatever the step
    gave, and its `llm` is `None` (a job has no LLM choice; one given is refused
    `tasks::Refusal::Executor`).
  The members' holdings are asked of the connectors as any member's (`Ask::ProcedureHoldings`),
  unchanged.
- **The delegation** (`src/runs/delegation.rs`): `delegation_executor` maps `Job { kind }` to
  `Some(authority::Executor::Job(kind))`, so `procedure_batch_admit`'s authority check
  (`jig-domain-authority/src/check.rs`, `check_batch`, the kinds loop at about 172-181) answers
  propose when the creator's delegation lacks `Executor::Job(kind)` but its project's ceiling
  has it, and refuse when the project's or the deployment's ceiling lacks it, as for a
  charter. If `tasks-jobs-surface` already wrote this arm, keep it and test it.
- **No other creator makes a job** (engine.md, 7.5; tasks.md, 5.3.1): an agent's `delegate`
  (`delegate_preflight`) naming a `Job` member is refused `tasks::Refusal::Executor` before any
  authority check; a party's Create is already refused (`client-jobs`).
- **The test kits:** `jig-test-connector`'s `ProcedureAction` gains `Job { kind: u32, spec:
  Box<[u8]>, deadline: Duration }` and `StepDecision` a `Job { kind, spec, deadline }`: the
  procedure's program makes one job delegate and awaits `ProcedureSignal::DelegateDone`, as
  `Delegate` does; `jig-test-domain/src/steps.rs`'s `translate` makes it one `core::Delegate`
  with `executor: Job { kind }`, `spec: Spec { words: empty, parameters: empty, inputs: empty,
  asks: false, job: Some(JobSpec { bytes: spec, deadline }) }`, a placeholder `Contract::Job {
  bytes: 0 }` (the core sets it), no LLM choice, zero spend, an empty delegation, and the
  dependencies, holdings and wake of an ordinary delegate.
- The stories and unit tests below pass; every existing test of the packages named in Steps 6
  passes unchanged.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md`; `notes/contracts.md`, sections 1 (`JobKind`,
  `JobBound`), 4 and 5.
- `jig/docs/design/domain/engine.md, 7.5` (its check; its kinds are the application's);
  `jig/docs/design/domain/tasks.md, 5.3.1`; `jig/docs/design/domain/authority.md, 3`
  (delegation's kinds) and `8.1` (one check for every creator; propose or refuse);
  `jig/docs/design/domain/connectors.md, 6.3` (what a step decides).
- `tasks-jobs-surface`'s report (the inert arms it wrote in `jig-domain-core`; whether
  `delegation_executor` already maps `Job`).
- The code: `jig-domain-core/src/lib.rs` (`Config` at about 122, `Core::new` at about 375);
  `src/runs/delegation.rs` (`Delegate` at about 26, `ProcedureStep` at about 50,
  `procedure_batch_admit` at about 104, `delegate_preflight`, `delegation_executor` at about
  274); `src/routing/run/delegation.rs` (`procedure_step` at about 351);
  `src/routing/effect/procedure.rs` (`problem`, `refused`); `jig-domain-authority/src/check.rs`
  (`check_batch`, `batch`).
- The test kits: `jig/testing/jig-test-connector/src/boundary.rs` (`ProcedureAction` at about
  345, `StepDecision` at about 424), `src/procedures.rs` (`decide`'s `Delegate` arm at about
  134); `jig/testing/jig-test-domain/src/steps.rs` (`translate`, the `Delegate` arm at about 60-95).
- The core world: `jig/tests/core` (package `jig-core-world`): `tests/suite/procedure_steps.rs`
  (`program_world`, `decide` at 55, `make_delegate` at 66,
  `a_procedure_hears_the_first_task_its_step_made_with_its_revision` at 80, whose stored
  `connector::Record::ProcedureDelegateOutcome` is the way to read a step's outcome);
  `src/faults.rs` (`plan_fixture`); `src/fixtures.rs` (`member`, `live_row`); `src/oracle.rs`
  (`configuration`; the restart oracle runs on every story's drop).

## Owns and may change

Owns: `jig/crates/jig-domain-core/**`, `jig/testing/jig-test-connector/**`,
`jig/testing/jig-test-domain/**`, `jig/tests/core/**`.

May change, as written:

- `jig/crates/jig-domain-core/Cargo.toml`: `jig-domain-link.workspace = true`, for
  `JobBound` (an internal path dependency; `jig-domain-fleet` already links it). No other
  dependency.
- **`Config` literals**, each gaining `jobs: Box::new([])` and nothing else:
  `jig/testing/jig-test-profile/src/lib.rs`, `jig/crates/jig-protocol-config/src/core.rs`,
  `jig/tests/engine-clients/src/fixtures/receiving.rs`,
  `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`,
  `temper/tests/engine/domain/src/walking.rs`; and
  `temper/tests/engine/domain/src/oracle/config.rs` copies `jobs` as it copies the other
  fields. Any other literal the compiler names, the same way.
- **The new test-kit variants' forced arms**, routing only: any root or world matching
  `jig_test_connector::ProcedureAction` or `StepDecision` exhaustively
  (`jig/testing/jig-test-profile/src/connector.rs`, `jig/tests/test-connector/**`,
  `jig/testing/jig-conformance/**`, if they do; `jig/examples/ops` has types of its own and
  needs nothing unless the compiler names it).
- No file of `jig-domain-tasks`, `jig-domain-authority` or `jig-tasks-world`:
  `tasks-jobs-lifecycle` owns them meanwhile. If the authority check cannot answer propose for
  a missing job kind without a change there, stop and say so.

Pre-approved options:

- **Where the core's job check sits:** a `pub(crate)` pure function `fn job_member(jobs:
  &[jig_domain_link::JobBound], member: &mut Delegate) -> Result<(), tasks::Refusal>` in
  `src/runs/delegation.rs` (or a new `src/runs/jobs.rs`), called per member by
  `procedure_step`; the step's refusal names the procedure task as `problem(task, why)` does.
  No new public item but `Config::jobs` and the test kits' variants.
- **Order:** the job check runs after `take_models`, `take_deadline` and `task_as_allowed` and
  before `procedure_batch_admit`, so an undeclared kind is refused before the authority answers.
- **The fixture:** a helper in `jig/tests/core/src/fixtures.rs` (or `faults.rs`), `pub fn
  job_world(seed: u64, ceiling: bool) -> (World, u64)`, built as `procedure_steps.rs`'s
  `program_world` is, with `core.jobs` declaring kind 7 (`spec: 64, result: 32, deadline: 60s`),
  the procedure's delegation naming `AuthorityExecutor::Job(7)`, and `Executor::Job(7)` in the
  deployment's and the project's ceilings when `ceiling` is set (as `plan_fixture` sets the
  ceiling's kinds). If the ceilings are only reachable through `jig-test-profile`, add the
  kind there in a function of its own (`jig_test_profile::with_job_kinds` or similar) and say so.
- If a story here is refused by the tasks child (`Refusal::Executor` from the tasks child, not
  the core) because `tasks-jobs-lifecycle` has not landed, that story belongs to
  `core-jobs-run`: drop it and say so.

## Steps

1. `Config::jobs`, its assertions in `Core::new`, the core's copy; every literal.
2. The job check in `procedure_step`; `delegation_executor`'s arm; `delegate_preflight`'s
   refusal.
3. The test kits' `Job` action and decision, and `steps.rs`'s translation.
4. Unit tests in `jig-domain-core` (the crate's idiom, beside `runs/delegation.rs`'s or in
   `src/tests/`), each citing `jig/docs/design/domain/engine.md, 7.5`:
   - `a_declared_job_within_its_bounds_takes_its_kinds_result_bound_as_its_contract`: kind 7
     declared, spec 64 bytes, deadline 60s: `Ok`, the contract `Job { bytes: 32 }`;
   - `a_job_of_an_undeclared_kind_is_refused`; `a_job_past_its_kinds_spec_or_deadline_is_refused`
     (65 bytes; 61s); `a_job_without_its_spec_is_refused`; `a_job_naming_an_llm_is_refused`;
   - `delegation_executor` maps `Job { kind: 7 }` to `authority::Executor::Job(7)`.
5. Stories in `jig/tests/core/tests/suite/jobs.rs` (a new module, added to `main.rs`), each
   citing `jig/docs/design/domain/engine.md, 7.5` (and `authority.md, 8.1` for the
   delegation's), in a world from the fixture above, the step sent with `decide` as
   `procedure_steps.rs` sends it:
   - `a_procedures_job_of_an_undeclared_kind_is_refused_and_its_procedure_told`: a job of kind
     9; the step is refused `Executor`; the procedure has no delegate, and its stored
     `ProcedureDelegateOutcome` names no child.
   - `a_procedures_job_past_its_kinds_bounds_is_refused`: spec 65 bytes; then a deadline of
     61s: each refused `Spec`, nothing made.
   - `a_job_kind_its_procedures_delegation_lacks_is_answered_propose`: the procedure's
     delegation without `Job(7)`, the project's ceiling with it: the step is refused
     `StepRefusal::Denied { answer: Propose, .. }` with an `Executor` finding; and with the
     project's ceiling also lacking it, `Denied { answer: Refuse, .. }`.
   - `an_agents_delegate_naming_a_job_is_refused`: the root run's `DelegateValidated` with a
     job member is answered `DelegationRefused` with `Refusal::Executor`.
   - `a_connector_procedure_programmed_with_an_undeclared_job_hears_its_step_refused`: the
     test connector's procedure `[Job { kind: 9, .. }, Finish { Succeeded }]` (not
     `hold_procedures`): its step is refused, as above, through the test kits' translation.
6. Narrow tests first (`heavy cargo nextest run -p jig-domain-core --lib -E 'test(/job/)'`;
   `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/jobs::/)'`; check each
   count), then `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-test-connector -p
   jig-test-domain -p jig-core-world --all-targets -- -D warnings`; `heavy cargo nextest run
   -p jig-domain-core -p jig-test-connector -p jig-test-domain -p jig-core-world -p
   jig-protocol-config`; then `heavy cargo check --tests -p jig-test-profile -p
   jig-engine-clients-world -p jig-test-connector-world -p temper-engine-domain -p
   temper-engine-domain-world` (the names the workspace gives the packages of the literals you
   changed; find them with `cargo metadata` if one differs), and the tests of
   `temper-engine-domain`.
7. Commits: the config and the check, then the test kits, then the stories; push your branch.

## Report

As the implementer's text says, and also: where the job check sits and its refusal order;
whether `delegation_executor`'s arm was already the surface's; every `Config` literal you
changed; any story dropped to `core-jobs-run` and why.
