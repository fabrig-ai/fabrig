+++
id = "fleet-jobs"
part = "worker-channel"
kind = "feature"
needs = ["hub-jobs"]
files = [
  "jig/crates/jig-domain-fleet/**",
  "jig/crates/jig-domain-hub-link/**",
  "jig/tests/fleet/**",
]
effort = "high"
minutes = 170
+++

# The fleet places jobs on hosts that declare their kind, and carries their answers

## Goal

The engine's side of a job between the core and a host (`jig/docs/design/domain/engine.md,
7.5` and section 8): the fleet records each host's job kinds from its hello, places a job only
on a host whose hello declared its kind with bounds covering the job's and a free slot, fenced
by its attempt, and carries its answer back; a restart adopts or loses a claimed job as a run's.
`jig-domain-hub-link` gives the core one event to start a job and maps a job's answer to the
tasks child's `End`. Which task becomes a job and when is the core's (`core-jobs-run`); this
task builds the fleet's and the hub link's halves, tested in the fleet's world and the crates'
unit tests.

Acceptance:

- The fleet refuses a hello whose `jobs` are not ascending and distinct, with the fleet's
  existing not-admitted refusal and a reason of its own; a hello with none is admitted as
  today.
- `jig-domain-hub-link` takes `Event::StartJob { job: JobStart }` (below) from the core; the
  fleet places it on a channel whose hello's `JobBound` for its kind covers its spec's bytes,
  its result bound and its deadline, with a free slot, preferring a channel holding its
  workstream as a run's placement does; nothing of a run's placement (charter's host kinds,
  models, accounts, logins, program sources) applies. No such channel: it waits unplaced, as a
  run does, and is placed when one says hello.
- Placed, hub-link emits `Request::AssignedJob { channel, job: JobStart }` to the root, which
  builds `host::JobAssignment` with its workspace items and sends it down as
  `host::LinkDown::AssignJob`, as it builds a run's from `Request::Assigned`.
- `host::LinkUp::JobAnswer` is retained as a payload and reaches the fleet as `Event::Answer`
  with the fleet's `Answer` (`Ended` for finished; `Failed { turned: None }` for failed;
  `Busy` and `Invalid` for refusals); the core's existing answer path (`Now::AnswerPayload`,
  `answer`, `result_event`) gives the tasks child the `End` of `notes/contracts.md`, section
  4's last bullet. A job's answer is acknowledged as a run's.
- `Inbound`, `Relay`, turns and grants renewed are never sent for a job; an `Inbound` naming one
  is refused as undeliverable.
- Restart: an `Adopt` of a job's claim is listed, answered or lost by the hello and the grace as
  a run's (`attempt.rs`, `listed`, `expire`).
- The stories below pass; every existing fleet and hub-link test passes unchanged.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md`; `notes/contracts.md`, sections 1, 2 and 4.
- `jig/docs/design/domain/engine.md, 7.5` (a job's check, claim, answer, restart) and section 8
  (the fleet); `jig/docs/design/domain/hosts.md, 6.9` (the assignment and answer);
  `jig/docs/design/protocol/channel.md, 4.3` (the hello's refusal) and `5.4`.
- The code: `jig-domain-fleet/src/channel.rs` (`hello` about 131, `admit_worker` about 107,
  `choose` about 296), `needs.rs` (`shortfall`, `fits`, `allows`), `attempt.rs` (`place` about
  544, `assigned` about 1135, `answer` about 647, `listed` about 586, `expire` about 764,
  `loaded` about 801), `boundary.rs`; `jig-domain-hub-link/src/lib.rs` (`Event`, `Request`,
  `step` about 336, `answer` about 519, `result_event` about 537), `payload.rs` (`up`, which
  `hub-jobs` left answering `LinkUp::JobAnswer` with `None`), `down.rs`.

## Owns and may change

Owns: `jig/crates/jig-domain-fleet/**`, `jig/crates/jig-domain-hub-link/**`,
`jig/tests/fleet/**`.

The new outer values, binding (inside them the choices are yours):

```rust
// jig-domain-hub-link
/// A job the core has claimed, for the fleet to place (domain/engine.md, 7.5).
#[derive(PartialEq, Eq, Debug)]
pub struct JobStart {
    pub task: u64,
    pub attempt: u64,
    pub workstream: u64,
    pub kind: jig_domain_link::JobKind,
    pub spec: Box<[u8]>,
    pub deadline: Duration,
    /// Its contract's result bound, which the host's bound for the kind must cover.
    pub result: u32,
    /// The application's credentials its workspace needs.
    pub grants: Box<[jig_domain_accounts::Grant]>,
}
// Event::StartJob { job: JobStart }        core -> hub-link
// Request::AssignedJob { channel: Token, job: JobStart }   hub-link -> root
```

May change, as written:

- **Roots that route hub-link's requests** gain the `AssignedJob` arm: build
  `host::JobAssignment { run, attempt, kind, spec, deadline, workspace, grants }` from it, the
  workspace and grants as the root builds a run's from `Request::Assigned`, and send
  `host::LinkDown::AssignJob` as it sends a run's assignment:
  `temper/crates/temper-engine-domain/src/link.rs` (about 164), `jig/examples/ops/domain/src/link.rs`
  (about 46), `jig/testing/jig-test-domain` and any other root or test parent the compiler names.
  This is routing and translation only (`jig/AGENTS.md`).
- **The core** (`jig/crates/jig-domain-core/**`): only the arms the new hub-link values force
  in its routing, each the shape of the run's arm beside it; the core sends no `StartJob` in
  this task.
- `jig/tests/core/**`: fixtures only, if a hub-link value's literal is forced.

Pre-approved options:

- **The fleet's own events:** `fleet::Event::StartJob` and `fleet::Request::AssignJob` beside
  `Start` and `Assign`, or the same variants with a job arm in their payload; either, if a
  run's fields and behaviour stay as they are. Say which.
- **A job's entry in the fleet:** a `Run` entry marked as a job, or one of its own in the same
  table; slots, claims, adoption and answers shared with runs.
- **Covering bounds:** spec bytes `<=` the host's `spec`, the contract's `result` `<=` its
  `result`, the deadline `<=` its `deadline`.
- **Where the job's `End` is computed:** in `result_event`'s job arm, or a sibling
  `job_end(ending) -> tasks::End` called from `answer`; the mapping is contracts, section 4's.
- If the root's run assembly (`temper-engine-domain/src/link.rs`) gathers its workspace from a
  core value the job's start does not carry, carry the job's workspace items as the core's
  `Prepared` carries a run's (a `JobStart` field the core fills), and say so: `core-jobs-run`
  fills it.

## Steps

1. The hello: record `jobs` per channel; refuse out-of-order or repeated kinds.
2. `JobStart`, `Event::StartJob`, the fleet's job placement, `Request::AssignedJob`.
3. The answer: `payload::up`'s `LinkUp::JobAnswer` arm retains the ending and gives the fleet its
   `Answer`; `answer`/`result_event` give a job's `End` (contracts, section 4).
4. Restart: adopted job claims as runs'.
5. Stories, each citing `jig/docs/design/domain/engine.md, 7.5`, in `jig/tests/fleet` (its
   `World`, scripted workers in `src/workers.rs`; precedent
   `a_worker_refused_for_its_agent_hosts_nothing`) and the crates' `Harness`es:
   - `a_job_is_placed_only_on_a_worker_whose_hello_declares_its_kind`: two workers, one
     declaring kind 7; a job of kind 7 goes to it; a job of kind 9 waits unplaced until a third
     worker declaring 9 says hello.
   - `a_job_past_a_workers_bounds_for_its_kind_is_not_placed_there`: spec, result and deadline
     each one past.
   - `a_hello_with_job_kinds_out_of_order_is_not_admitted`.
   - `a_jobs_answer_reaches_the_core_as_its_end`: finished gives `End::Finished` with
     `TaskResult::Job`; failed `Deadline` gives `End::Failed` of the run's own class;
     `Unprepared(Permanent)` the permanent class; busy is placed again.
   - `a_worker_never_gets_a_message_or_relay_for_a_job`.
   - `an_engine_restart_adopts_a_job_its_worker_still_hosts` and one lost past the grace.
6. `cargo fmt`; `heavy cargo clippy -p jig-domain-fleet -p jig-domain-hub-link -p
   jig-fleet-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-fleet
   -p jig-domain-hub-link -p jig-fleet-world`; then `heavy cargo check --tests` for each other
   package you changed, and the tests of `jig-core-world` and `temper-engine-domain`.
7. Commits; push your branch.

## Report

As the implementer's text says, and also: the fleet's event and entry shapes chosen; the names
`core-jobs-run` calls (`Event::StartJob`, and any field you added to `JobStart`); each root arm
you wrote.
