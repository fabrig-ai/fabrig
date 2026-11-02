+++
id = "core-jobs-run"
part = "job-tasks"
kind = "feature"
needs = ["tasks-jobs-lifecycle", "core-jobs-admit", "fleet-jobs"]
files = [
  "jig/crates/jig-domain-core/**",
  "jig/tests/core/**",
]
effort = "high"
minutes = 180
+++

# The core runs a job: its claim, its placement, its answer, its restart

## Goal

jig's core carries a due job task to a host and its result back to its procedure
(`jig/docs/design/domain/engine.md, 7.5`; `jig/docs/design/domain/tasks.md, 5.3.1`): it gathers
the job's workspace items and the application's credentials from its connectors as a run's are
gathered, checks what its items write, claims it with nothing reserved, starts it through the
hub link's `StartJob` on a host declaring its kind, commits its answer as a delegate's result
or a failure by class, ends it at once on a cancel, and after a restart adopts or loses a claimed
job as a run's. No LLM account, charter, model, brief or transcript takes part.

Acceptance:

- **Activation** (`src/routing/run/preparation.rs`, `route_activate`, the `Job` arm the surface
  left making nothing): a due job, with decisions open, is taken to `Preparing`
  (`tasks::Event::Prepare`), given a fresh attempt (`crate::fresh(.., Family::Run)`) and a
  workspace flight, and its items asked with `Now::WorkspaceRequest { task, attempt, context }`
  as a run's are (`brief_assembled`); no `run_admission`, no `Now::PrepareAgent`, no brief, no
  transcript. Not ready, or the start not opened, it is remembered due as a run is.
- **Its check** (`workspace_prepared`, a job's branch before any transcript or account is
  read): the writes its connectors named and its credentials are checked by the new
  `jig_domain_authority::check_job` (below), the findings mapped to holds and waits as
  `run_admission` maps a run's (`Wait` remembers it due; a `Deadline` finding holds
  `Hold::Deadline`; a resource finding the hold `run_admission` gives it). Allowed, its grants
  are the application's credentials alone (`core.run_credentials`, `credential_grant`), its
  `RunProof` is `start_proof`'s with `conversation = attempt`, no accounts, no window, no
  placement, `host: HostKind::Worker` and its credentials, and the core claims it at once:
  `tasks::Event::Claim { budget: 0, account: None, resumed: false, writes }`, with the claim's
  pending entry (`core.claiming`) as a run's. A claim the tasks child refuses or makes wait
  for a writer slot goes back as a run's does (`route_claim_refused`, the writer's wait).
- **Its start** (`settings/charters.rs`, `claim_done`): for a job, saves its `RunProof` as for a
  run and asks each connector `Ask::JobClaimed { task, attempt, kind, spec, deadline, result,
  grants }` (new, beside `Ask::ClaimDone`), never `charter_version`. Each root routes it as it
  routes `ClaimDone`, to `jig_domain_hub_link::Event::StartJob { job: JobStart { task,
  attempt, workstream: task, kind, spec, deadline, result, grants, .. } }`, held until the
  decision is durable as `ClaimDone`'s `Event::Start` is: `jig/testing/jig-test-domain/src/
  routes.rs` (beside `ClaimDone` at about 234), `temper/crates/temper-engine-domain/src/route/
  asks.rs` (about 293), `jig/examples/ops/domain/src/route.rs` (about 612). Then
  `fleet-jobs`'s placement puts it on a worker whose hello declares its kind and covers its
  bounds; nothing of a run's placement applies.
- **Its answer** (`src/routing/children.rs`, `fleet::Request::Answered` at about 381;
  `src/routing/run/payloads.rs`): a job's answer reaches the tasks child through the core's
  existing answer path as the `End` of `notes/contracts.md`, section 4's last bullet, with
  `cumulative` 0 and no turn, read, refusal or account terminal; its `RunProof`'s terminal is
  set as a run's (`payloads.rs` at about 75) and erased with its end. Its result commits as a
  delegate's result (tasks.md, 5.6): the procedure is woken and its next step's context holds
  `MessageKind::Result(ResultKind::Job)` with the result's bytes; a failure is tried again
  within its class's tries (a new attempt, a new start), then holds it, its procedure hearing
  the hold.
- **Cancel** ends it at once: a cancel of the job or of any task above it sends the live
  attempt's cancel down as a run's (`Held::Cancel`), and its `Cancelled` answer ends it.
- **Restart** (engine.md, section 6 and 7.5): a claimed job's `RunProof` is adopted by
  `adopt` (`src/routing/sibling_routes.rs`, about 211) as a run's, with no account grants and
  its credentials' grants; a worker that lists it at its hello keeps it, and its answer ends it;
  one no worker lists past the grace is lost (`children.rs`'s `fleet::Request::Lost` at about
  213; `stored.rs` at about 590) and tried again, as a run's.
- `jig_domain_authority::check_job` (new, `src/check.rs`, re-exported as `check_run` is):

  ```rust
  /// A job's claim check (domain/engine.md, 7.5; domain/authority.md, 8.3 for what it shares
  /// with a run's): its deadline, its credentials and what its items write; no account,
  /// budget, reserve or model.
  #[derive(Clone, PartialEq, Eq, Debug)]
  pub struct JobAsk {
      pub project: u32,
      pub authority: Authority,
      pub wall: Wall,
      /// Whether each application credential its workspace needs is usable now.
      pub credentials: Box<[bool]>,
      pub writes: Box<[Write]>,
  }
  #[must_use]
  pub fn check_job(domain: &Domain, ask: &JobAsk, why: &mut Queue<Finding>) -> Answer;
  ```

  It answers as `check_run` does for the same inputs, less its account, budget, reserve and
  spend findings: oversized shapes and an unknown project refuse; a past deadline (the task's
  and the ceiling's) finds `Deadline`; an unusable credential waits; each write is checked by
  the same grant, writable and held checks (`grants`, `writable`, `Writer`) as a run's.
- The stories below pass; every existing test of the packages in Steps 7 passes unchanged.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md`; `notes/contracts.md`, sections 2 (the
  job's ending and failures), 4 (classes; the answer as an `End`) and 5 (the claim).
- `jig/docs/design/domain/engine.md, 7.5` (a job: check, claim, answer, restart), `7.1` (a
  run's preparation and claim, which a job's follows for its items and credentials), section 6
  (restart: claims adopted, lost) and section 8 (the fleet: attempts fenced, a refused start,
  losing contact); `jig/docs/design/domain/tasks.md, 5.3.1`, `5.5` and `5.6`;
  `jig/docs/design/domain/connectors.md, 6.2` (a procedure steps on its delegate's result) and
  section 10 (a workspace's items); `jig/docs/design/domain/authority.md, 8.3`.
- `tasks/fleet-jobs.md` and its report: the names this task calls (`Event::StartJob`,
  `JobStart` and any field it added, such as a workspace's items the core fills), the root arm
  it wrote for `Request::AssignedJob` in `jig-test-domain` (the delivery a job's assignment
  becomes), and how a job's answer reaches the core. `tasks/core-jobs-admit.md`'s report (the
  job fixture in `jig/tests/core`, the test connector's `Job` action) and
  `tasks/tasks-jobs-lifecycle.md`'s (what the tasks child refuses of a job's terminals: never
  send `Parked`, `FinishedWithFollowups`, `Exceeded` or `Recheck` for a job).
- The code: `src/routing/run/preparation.rs` (`route_activate` at about 488, `brief_assembled`
  at about 15, `workspace_prepared` at about 40, `start_proof` at about 242,
  `route_claim_prepared` at about 563, `route_claim_refused`); `src/runs/state.rs`
  (`run_admission` at about 102, its findings-to-holds mapping; `run_credentials`);
  `src/settings/charters.rs` (`claim_done` at about 130); `src/vocabulary.rs` (`Ask` at about
  42, `ClaimDone` at about 120); `src/routing/task/lifecycle.rs` (`activate`);
  `src/routing/children.rs`, `src/routing/run/payloads.rs`, `src/routing/sibling_routes.rs`,
  `src/stored.rs` (`RunProof` at about 614); `jig-domain-authority/src/check.rs` (`check_run`
  at about 479).
- The core world: `jig/tests/core` (`src/world.rs`'s `World`, `Setup`; `src/world/
  deliveries.rs`, where a delivery is captured; `src/oracle.rs`, which restarts on every
  story's drop and compares); the stories to follow: `tests/suite/charters.rs`
  (`a_delegate_waits_for_a_host_declaring_its_charters_model` at about 37: a hello sent as
  `core::Event::Fleet(fleet::Event::Hello { .. })`), `tests/suite/crossed_claims.rs`
  (`hosting`, a claim listed at hello), `tests/suite/faults.rs`
  (`a_vanished_worker_is_lost_and_another_worker_takes_the_retry` at about 110),
  `tests/suite/jobs.rs` (`core-jobs-admit`'s fixture and stories); a job's answer sent up as
  `root::Event::Link { channel, link: link::Event::Up(host::LinkUp::JobAnswer { .. }) }`
  (`jig/testing/jig-test-domain/src/link.rs`).

## Owns and may change

Owns: `jig/crates/jig-domain-core/**`, `jig/tests/core/**`.

May change, as written:

- `jig/crates/jig-domain-authority/src/check.rs` and `src/lib.rs`: `JobAsk` and `check_job`
  above, and the private helpers they share with `check_run`, which keeps its behaviour; a unit
  test beside `check_run`'s.
- **`Ask::JobClaimed`'s routes**, routing and translation only (`jig/AGENTS.md`):
  `jig/testing/jig-test-domain/src/routes.rs`, `temper/crates/temper-engine-domain/src/route/
  asks.rs`, `jig/examples/ops/domain/src/route.rs`, and each other root or test connector that
  matches `core::Ask` exhaustively (an arm that ignores it where `ClaimDone` is ignored).
- `jig/testing/jig-test-domain/**` beyond that route only where a job's delivery or answer is
  not yet reachable from the core world (a delivery `fleet-jobs` did not add): add it as the
  run's is, and say so.
- `jig/crates/jig-domain-hub-link/**`: its core-facing side only, if the core's answer path
  needs a value from it that `fleet-jobs` did not give; the fleet's path is `fleet-jobs`'s.
  Say what and why.

Pre-approved options:

- **Where a job's branches sit:** a module of their own (`src/routing/run/jobs.rs`) holding
  the job's activation, check, claim and start, called from the run's functions at their
  executor split; or arms in place. `pub(crate)` only.
- **The job's start request:** `Ask::JobClaimed` as above (an `Ask`, because `ClaimDone` is
  the handoff the root holds until the claim's decision is durable). If the core's commit
  order makes a `Held` request the cleaner carrier (as `Held::Cancel` is), use it with the same
  fields and say so; either way, never a `Now`, which is not held for the commit.
- **Workspace items in the start:** if `fleet-jobs` added a field to `JobStart` for them, the
  core fills it from what the workspace request answered (`WorkspacePrepared`) and carries it
  in `Ask::JobClaimed`.
- **`check_job`'s findings:** if a finding a run's check gives has no meaning for a job
  (`RunBudget`, `RunCap`, `Price`, `Account`), it never arises from `check_job`; if the
  findings-to-holds mapping must be shared, move it to a `pub(crate)` function both call.
- **The world:** capture a job's assignment where `deliveries.rs` captures a run's (a `pub
  jobs: Vec<(Token, host::JobAssignment)>` on `World`, or the delivery `fleet-jobs` made), and
  helpers `World::job_hello(channel, worker, kinds)` and `World::answer_job(channel, task,
  attempt, ending)` in `src/world.rs` or `src/fixtures.rs`.
- If the restart oracle finds a difference for a job that is a run's too (not this task's),
  stop and say so rather than changing the oracle.

## Steps

1. `check_job` and its unit test in `jig-domain-authority`
   (`a_jobs_writes_are_checked_as_a_runs_without_an_account_or_budget`).
2. The job's activation, check and claim (`route_activate`, `workspace_prepared`), its start
   (`claim_done`, `Ask::JobClaimed`, the roots' routes).
3. The answer and its result to the procedure; cancel; restart (adoption and loss), each as
   the run's beside it, never through `run_admission` or a transcript.
4. World support (above).
5. Stories, appended to `jig/tests/core/tests/suite/jobs.rs`, each citing
   `jig/docs/design/domain/engine.md, 7.5` (and section 6 for the restarts), with
   `core-jobs-admit`'s fixture (kind 7 declared, the delegation and ceilings naming it):
   - `a_procedures_job_waits_for_a_worker_declaring_its_kind_and_its_result_reaches_the_next_step`:
     the procedure's step makes a job of kind 7 (64 spec bytes, deadline 60s); its live row is
     `Executor::Job { kind: 7 }` with contract `Job { bytes: 32 }`; it is claimed with
     `reserved` 0 and a `RunProof` with no accounts; no job is assigned while no worker
     declares kind 7, nor after a hello declaring kind 9 only; a hello declaring kind 7 gets
     `LinkDown::AssignJob` with kind 7, the spec's bytes, its deadline and no LLM grant; its
     `JobAnswer` finished with `b"result"` ends the job done, and the procedure's next step's
     context holds a `Result(ResultKind::Job)` word with `b"result"`; the procedure finishes.
   - `a_failing_job_is_held_past_its_tries_and_its_procedure_hears_it`: each attempt answered
     `Failed { failure: Deadline, .. }`; a new attempt is assigned after each backoff (advance
     the clock) until past the run class's tries; then held `Hold::Failures(Class::Run)`, the
     procedure woken by its escalation.
   - `a_job_whose_workspace_could_not_be_prepared_for_good_fails_permanent`: one answer
     `Failed { failure: Unprepared(Preparation::Permanent { resource: None }), .. }` counts a
     permanent try (`tries.permanent` 1) and backs off.
   - `cancelling_the_requesters_subtree_ends_a_running_job_at_once`: a party cancels the tree
     above the procedure while the job runs; the job's attempt is cancelled down at once (no
     grace word); its `Cancelled(Engine)` answer ends it cancelled, then the procedure.
   - `a_restart_adopts_a_claimed_job_its_worker_still_lists`: the job assigned; restart; the
     worker's hello lists it hosted (`Hosting { phase: Active, .. }`); its answer after the
     restart ends it done; precedent `crossed_claims.rs`.
   - `a_claimed_job_no_worker_lists_after_a_restart_is_lost_and_tried_again`: past the grace
     it is lost and a new attempt goes to a worker declaring its kind; precedent `faults.rs`.
6. Narrow tests first (`heavy cargo nextest run -p jig-domain-authority --lib -E
   'test(/job/)'`; `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/jobs::/)'`;
   check each count), then `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p
   jig-domain-authority -p jig-test-domain -p jig-core-world --all-targets -- -D warnings`.
7. `heavy cargo nextest run -p jig-domain-authority -p jig-domain-core -p jig-test-domain -p
   jig-core-world`; then `heavy cargo check --tests -p temper-engine-domain -p
   temper-engine-domain-world -p jig-ops-domain` (the ops root's package as the workspace
   names it; `cargo metadata` if it differs) and every other package you changed, and the
   tests of `temper-engine-domain`.
8. Commits: the authority check, then the core's job path, then the roots' routes, then the
   world and stories; push your branch.

## Report

As the implementer's text says, and also: where the job's branches sit; the start's carrier
(`Ask::JobClaimed` or otherwise) and each root arm; what of `fleet-jobs`'s names you used and
any field you filled; any hub-link or test-domain change and why; anything the restart oracle
showed.
