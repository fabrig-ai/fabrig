+++
id = "hub-jobs"
part = "worker-channel"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-link/**",
  "jig/crates/jig-domain-host/**",
  "jig/tests/host/**",
]
effort = "high"
minutes = 160
+++

# The hub hosts jobs

## Goal

jig's hub hosts a job task as it hosts a run, with no agent (`jig/docs/design/domain/hosts.md,
6.9`): admit, prepare, run through the root's job capability, deliver, stop, release, answer,
kept until acknowledged. The hello states the job capability's kinds. The engine side and the
wire are other tasks'; this task builds the domain values and the hub, and gives every other
crate only the arms and literals the compiler forces.

Acceptance:

- `jig-domain-link` has `JobKind`, `JobBound` and `Hello::jobs` exactly as
  `notes/contracts.md`, section 1; `jig-domain-host` has section 2's types, link records, job
  capability vocabulary, `Request::DeliverJob` and `Event::Opened::jobs`, exactly as written
  there, re-exported from the crate root as its run types are.
- The hub's stories below pass, in `jig-domain-host`'s unit tests (`src/tests.rs`'s `Harness`)
  and in `jig-host-world`, whose `Parent` gains a scripted job capability beside its agent.
- Every run story of both still passes unchanged: a run's types, states and answers are as
  they were.
- The crates the compiler names build and their tests pass (see Steps 6).

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md` and `notes/contracts.md`, sections 1 and 2.
- `jig/docs/design/domain/hosts.md, 6.9` (the job's lifecycle and answer), `6.1` and `6.2` (the
  link, hosted runs: the job's lifecycle is 6.2's with steps 3 to 5 replaced), `6.3` (typed
  failures), `section 4` (the job capability beside the agent and the workspace).
- `jig/docs/design/protocol/channel.md, 5.4` (what a job's records carry; phases in a hello).
- The code it extends: `jig-domain-host/src/hosted.rs` (`State`, `assign`, `prepared`, `stop`,
  `conclude`, `settle`, `release`, `answer`), `answers.rs` (the hello's assembly, the kept
  answers), `boundary.rs`, `link.rs`, `limits.rs`.

## Owns and may change

Owns: `jig/crates/jig-domain-link/**`, `jig/crates/jig-domain-host/**`, `jig/tests/host/**`.

May change, each only for what the compiler forces, as written here:

- **`Hello::jobs` literals:** every `Hello { .. }` literal in jig and temper (tests and fixtures
  included: `jig/tests/core`, `jig/tests/fleet`, `jig-domain-core`, `jig-domain-fleet`, the link
  crates and worlds, `temper-service`, `temper-engine-domain`) gains `jobs: Box::default()` (or
  the file's idiom for an empty boxed slice), nothing else. Where a fixture helper builds them,
  change the helper alone.
- **`Event::Opened::jobs`:** each root that sends `Opened` gives `jobs` empty
  (`temper/crates/temper-worker-domain/src/route.rs`, `jig/testing/jig-test-worker`,
  `jig/examples/ops`, the engine roots that compose a hub, test parents).
- **New link records in the translators** (the wire's are `link-jobs`'s):
  `jig-protocol-engine-workers/src/translate.rs`'s `down`: `LinkDown::AssignJob` is not
  encoded (the function's path for a record it does not send; if it has none, return the
  value it returns for a record past its limits); `jig-protocol-worker-link/src/translate.rs`'s
  `up`: `LinkUp::JobAnswer` returns `None`; `jig-domain-hub-link/src/payload.rs`'s `up`:
  `LinkUp::JobAnswer` returns `None` (the fleet has no job claims yet).
- **The job capability's arms in roots without one** (contracts, section 2's last bullet):
  `Request::Job(ToJob::Run { owner, .. })` is answered `Event::Job(FromJob::StartRefused {
  owner, why: b"no job capability" })`, `Request::Job(ToJob::Stop { owner })` with
  `Event::Job(FromJob::Gone { owner, detail: empty })`, and `Request::DeliverJob { owner, .. }`
  with `Event::Delivered { owner, delivery }` whose outcome is `DeliveryOutcome::Refused`,
  `changed: false` and `left` the token the root gives a delivery that left nothing (if it has
  none, the owner's). In: `temper/crates/temper-worker-domain/src/route.rs`,
  `jig/testing/jig-test-worker/src/domain.rs`, `jig/examples/ops/domain/src/link.rs`, any other
  root or test parent the compiler names. `temper-service` routes the new `LinkUp`/`LinkDown`
  values as it routes the others, if it matches them.

Pre-approved options:

- **Where the job's state lives:** a variant of the hosted entry beside a run's (`hosted.rs`), or
  a sibling module `jobs.rs` with its own entries in the same slab; either, so long as slots,
  hello listing, kept answers and acknowledgement are shared with runs. Say which.
- **The hub's limits:** if a grant count or a spec's bytes need a hub-wide bound beyond the
  kinds', add a field to `Limits` with its validation, and a literal where `Limits` is built.
- **Phases:** a job reports `Phase::Starting` from `Run` asked until `Started`, `Active` until
  `Ended`, `Ending` until answered.
- If a run story fails because a shared helper's behaviour changed, the helper is wrong for
  runs: keep runs' behaviour and give jobs their own path.

## Steps

1. `jig-domain-link`: section 1's values; `Hello::jobs`.
2. `jig-domain-host`: section 2's types and variants; the entrance's checks against the kinds
   `Opened` gave (unknown kind, spec past its bound, deadline past its bound, grants other than
   `Form::Application`: each `JobInvalid`), busy as a run's.
3. The job's lifecycle (hosts.md, 6.9): prepare when it has a workspace; `ToJob::Run` with the
   prepared workspace's token; `Started`; `Ended`; for `Finished` with a delivery and a
   workspace, `Request::DeliverJob`, then its `Delivered`; a result past its kind's bound fails
   `JobFailure::Result` with no delivery; `Gone`; `Release`; the answer, kept until
   `LinkDown::Acknowledge`, listed in the hello as `Answered` while kept. Cancel, `CancelAll`
   and a lost link: `ToJob::Stop`, no delivery not begun, a begun one waited for, `Gone`,
   release, answer `Cancelled(reason)`. A job saves nothing.
4. Stories, in `src/tests.rs` (the `Harness`) and `jig/tests/host` (its `World`, with a scripted
   job capability in `Parent`), each citing `jig/docs/design/domain/hosts.md, 6.9`:
   - `a_job_of_a_declared_kind_runs_in_its_workspace_and_answers_its_result`: `Opened` with kind
     7; `AssignJob` kind 7 with a workspace; `Prepare`, `Run` naming the prepared workspace,
     `Started`, `Ended(Finished { result: b"r", delivery: None })`, `Gone`, `Release`, then
     `LinkUp::JobAnswer` finished with `b"r"`; acknowledged, the slot is free.
   - `a_job_of_a_kind_not_declared_is_refused_as_invalid`: and one past its spec bound, one past
     its deadline bound, one granting an LLM account: each `JobRefusal::Invalid(..)`, kept by
     nothing, no `Prepare` asked.
   - `a_finished_job_delivers_through_its_workspace_before_release`: `Finished` with a delivery
     asks `DeliverJob` with those bytes; `Delivered` answers finished; a second story where the
     outcome is `Stale` answers `Failed(Undelivered(Stale))`.
   - `a_cancelled_job_is_stopped_and_answered_once_its_processes_are_gone`: `Cancel` while
     active asks `ToJob::Stop`; nothing is released before `Gone`; the answer is
     `Cancelled(Engine)`. And with a delivery in flight at the cancel, the answer waits for its
     `Delivered`.
   - `a_job_past_its_deadline_fails_deadline`: `Ended(Deadline)` answers `Failed(Deadline)`.
   - `a_result_past_its_kinds_bound_fails_and_delivers_nothing`.
   - `the_hello_states_the_kinds_and_lists_a_hosted_job`: the hello carries `Opened`'s kinds,
     and a job hosted across a redial is listed with its phase; its kept answer goes again
     after the hello.
   - In `jig/tests/host`, one seeded story mixing a run and a job on a two-slot host, both
     answered, the restart/replay oracle the world has holding.
5. Forced arms and literals in other crates, as above.
6. `cargo fmt`; `heavy cargo clippy -p jig-domain-link -p jig-domain-host -p jig-host-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-link -p
   jig-domain-host -p jig-host-world`; then `heavy cargo check --tests` naming each other
   package you touched (`-p jig-domain-fleet -p jig-domain-core -p jig-domain-hub-link -p
   jig-protocol-worker-link -p jig-protocol-engine-workers -p jig-test-worker -p
   temper-worker-domain -p temper-service -p temper-engine-domain` and the worlds), then run
   the tests of `jig-fleet-world`, `jig-worker-link-world`, `jig-engine-workers-world` and
   `temper-worker-domain`, whose fixtures you changed.
7. Commits: the values and the hub, then the forced arms; push your branch.

## Report

As the implementer's text says, and also: where the job's state lives in the hub; any limit
added; every file outside the owned crates you changed and why (each a forced literal or arm).
