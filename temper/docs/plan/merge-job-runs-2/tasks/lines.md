+++
id = "lines"
part = "lines"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain/src/link.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-service/src/lib.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/worker.rs", "temper/crates/temper-service/src/tests/workers.rs", "temper/crates/temper-service/src/tests/worker_job_answers.rs", "temper/crates/temper-engine-shell/src/serve/host.rs", "temper/crates/temper-worker-domain/**", "temper/crates/temper-worker-domain-tooling/**", "temper/crates/temper-worker-domain-checkout/**"]
effort = "high"
minutes = 160
+++

## Goal

Every failed or refused merge job is a line on the side that met it and
on the engine's, naming the job, its kind and why
(`temper/docs/design/domain/worker.md`, 4.4, the list dated 2026-11-09).
Today a job's failure is silent on both sides, a preparation's failure
reaches the hub with no detail, and the way out of a job held out of
tries points at an engine's log that names none of its failures.

Acceptance:

- **The engine's line.** When the engine's hub accepts a job's `Failed`
  or `Refused` answer (`temper-engine-domain/src/link.rs`, `answer`,
  today `hub::Ending::Job(_) => {}`), it keeps a job diagnostic beside
  the run diagnostics, popped by the service into a note, written by
  `temper-engine-shell` as `notes/contracts.md`, section 5's engine
  forms: `<binary> serve: task TASK run RUN attempt ATTEMPT failed CLASS:
  job KIND: DETAIL`, and `refused REASON: job KIND: WHY`. Once per
  accepted answer: a kept answer resent emits no second line, as 4.3's
  line does. KIND is the job's kind number.
- **The worker's line.** When the worker's hub answers a job failed or
  refused (`temper-service/src/worker.rs`, `job_answer`), one note,
  written as `<binary> worker: run RUN attempt ATTEMPT job KIND VERB
  failed CLASS: WHY` or `refused REASON: WHY`, once per answer, never
  for a finished job.
- **The why** is worker.md 4.4's: a preparation's step and git's fault,
  then git's error output's end where the operation kept it; a command's
  exit or signal and its error output's end; `deadline passed`; `result
  missing or past its bound`; a delivery's outcome and the push's
  diagnostic; a refusal's cause. A preparation's and a command's why is
  also the answer's `detail` (cut to the host's `detail_bytes` by jig's
  host as today), so the engine's line carries it.
- **Tests,** each a row of the existing table or story where one exists:
  - `temper-engine-shell`, beside `run_failure_line`'s and
    `run_refusal_line`'s tests (around `host.rs` line 1579): the two
    engine forms and the two worker forms, escaping a newline and a
    non-ASCII byte, a why past 4,096 bytes cut to its last 4,096;
  - `temper-service/src/tests/workers.rs`, beside `diagnostic_answer`
    and `check_answer_diagnostic` (line 794 on): a job's failed answer
    (`Unprepared(Transient)` with a detail) and a refused one accepted
    give one job note each, with task, attempt, kind 3 and the detail;
    resent, none more;
  - `temper-service/src/tests/worker_job_answers.rs`: the scripted merge
    worker (`merge_worker`, `merge_git`) whose base's fetch fails gives
    one worker note naming run, attempt, kind 3, `merge`,
    `Unprepared(Transient)` and the fetch's step and fault, and the
    answer's detail holds the same words; a finished job gives none;
  - `temper-worker-domain-checkout`'s tests: a prepare failing at a
    named step carries that step and fault in its failure's detail.
- No line, note or detail reaches a fact, turn, result, hold or anything
  an LLM is shown.

## Inputs

- `temper/docs/plan/merge-job-runs-2/notes/common.md`;
  `temper/docs/plan/merge-job-runs-2/notes/contracts.md`, section 5.
- `temper/docs/design/domain/worker.md`, 4.3 (the operator line: its
  form, escaping, once per acceptance) and 4.4 (the list dated
  2026-11-09); `temper/docs/design/domain/forge.md`, 8.4 (*The merge
  job*, its failures' lines); jig's `jig/docs/design/domain/hosts.md`,
  6.9 (a job's answer).
- The run's path to copy: `temper-engine-domain/src/link.rs`, `answer`;
  `domain.rs`, `RunDiagnostics`, `pop_run_diagnostic`,
  `pop_run_refusal`; `boundary.rs`, `RunDiagnostic`,
  `RunRefusalDiagnostic`; `temper-service/src/engine.rs` (lines 288 to
  298); `temper-service/src/lib.rs`, `Note::RunFailed` and
  `Note::RunRefused`; `temper-engine-shell/src/serve/host.rs`,
  `run_failure_line`, `run_refusal_line`, `run_detail_line`.
- The worker's side: `temper-service/src/worker.rs`, `job_answer` (607)
  and `notes` (225); `temper-worker-domain/src/jobs.rs` (`refuse`,
  `up`, `deliver`); `workspace.rs`, `unprepared` (its empty detail) and
  the job's delivery; `temper-worker-domain-tooling/src/domain.rs`,
  `exited_job` and the `Read` arm (the tail it ends with);
  `temper-worker-domain-checkout/src/hold.rs`, `failure` and each
  `Next::Failed`, and `boundary.rs`, `Prepared::Failed`.
- jig's host fills a delivery's and a deadline's failure with no detail
  (`jig/crates/jig-domain-host/src/jobs.rs`, `failed`): their why is the
  worker's alone, so it cannot come from the answer.

## Owns and may change

- **Owns:** the engine's job diagnostic (a type beside `RunDiagnostic`,
  its queue in `RunDiagnostics` or beside it, a `pop_` method, its
  re-export); `temper-service`'s `Note` variants for both sides and the
  engine's pop in `engine.rs`; `worker.rs`'s `job_answer` and what it
  reads, but not `worker_root` (task `story` edits it); the lines in
  `temper-engine-shell`; the why's path through `temper-worker-domain`,
  `temper-worker-domain-tooling` and `temper-worker-domain-checkout`.
- **The why reaches the service without changing a boundary enum's
  fields:** `temper_worker_domain::Request::JobAnswer` and
  `worker::Request`'s variants keep their shape, since the composed world
  (task `story`) matches them. Default: the worker domain keeps the why
  of each job it hosts, bounded by its slots as `job_specs` is, and the
  service takes it with a method, `Domain::job_why(&mut self, run:
  Token, attempt: Token) -> Option<Box<[u8]>>`, when it sees the failed
  or refused answer. If that cannot be done without changing a variant,
  stop with a `Question:`.
- **`checkout::Prepared::Failed`** may gain a `detail: Box<[u8]>` field
  (default) or the detail may travel another way you choose; its readers
  take the field or `..` only, each edited for that alone:
  `temper/tests/worker/checkout/src/world.rs`,
  `temper/tests/worker/checkout/src/merges.rs` (task `conflicts` adds a
  `Case` there: keep your edit to the match arms),
  `temper/tests/worker/checkout/tests/suite/{memory,reuse,simulation}.rs`,
  `temper/tests/worker/checkout/tests/suite/fuzzy/simulation.rs`,
  `temper-worker-domain-checkout/src/tests.rs`.
  `temper-worker-domain/src/workspace.rs` passes it to `unprepared`, which
  then fills the hub's `detail`; a run's preparation failure gets that
  detail too, which 4.3's run line then carries.
- **The kind on the engine's side:** read it where the engine knows the
  task made the job (the task's executor, `Executor::Job { kind }`, or the
  forge route that made it). If it is not at hand in temper's code without
  a jig change, stop with a `Question:`; jig is not changed.
- **Neighbours, compiler-forced only:** a new `Note` variant's arms
  (`temper/crates/temper/src/tests/help.rs` if it matches `Note`).
- **Not yours:** `temper/docs/help/` (the engine's form keeps
  `hold.md`'s `temper serve: task TASK run RUN attempt ATTEMPT failed
  CLASS: DETAIL` true); `temper/tests/system/**`.

## Steps

1. Engine side: the diagnostic, its note, its lines; the tests in
   `workers.rs` and `host.rs`, written failing first.
2. Worker side: the why through checkout, tooling and the worker domain;
   the note and line; the tests in `worker_job_answers.rs` and the
   checkout's tests, failing first.
3. Narrow runs, e.g.
   `heavy cargo nextest run -p temper-service --lib -E 'test(/worker_job_answers::/)'`
   (check the count).
4. Bite: drop the job arm in `answer` and see the engine's row fail; drop
   the worker's note and see its row fail; restore both.
5. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain -p temper-service -p temper-engine-shell -p temper-worker-domain -p temper-worker-domain-tooling -p temper-worker-domain-checkout --all-targets -- -D warnings`;
   the focused tests of those packages (`--lib`), of
   `temper-worker-checkout-world` and `temper-worker-domain-world`
   (`--test suite`); `heavy cargo check -p temper-system-world -p temper --tests`.
6. Commit and push.

## Report

`Done:`, `Failed:` or `Question:` first. Then: the four forms as
written, an example of each from the tests; where the kind and the why
come from on each side; the bites; what a delivery's and a deadline's
engine line say (no detail); implementation and check minutes apart.
