+++
id = "worker-jobs"
part = "worker-jobs"
kind = "feature"
needs = ["hub-jobs", "tooling-capability"]
files = [
  "temper/crates/temper-worker-domain/**",
  "temper/tests/worker/domain/**",
]
effort = "high"
minutes = 170
+++

# The worker's root composes the job capability

## Goal

temper's worker root composes `temper-worker-domain-tooling` beside smith's agent processes and
the checkout (`temper/docs/design/domain/worker.md, 4.4`): its hello states the capability's
kinds, a job the hub hosts runs in the checkout the hub prepared, the capability's file and
process operations go down to the protocol layer as the checkout's do, and a delivering kind's
delivery is the checkout's push of the writable repository's head. The service's side (the
spec's decoding, the operations themselves, the limits) is `service-jobs`'s; production gives
the capability no kind until M7b, so this task's stories run with a test kind.

Acceptance:

- `temper-worker-domain`'s `Domain` holds a `tooling: temper_worker_domain_tooling::Domain`,
  built from the root's limits and a kinds list the root's construction takes (production: an
  empty list); `host::Event::Opened`'s `jobs` is `tooling.bounds()`.
- `Event::AssignJob { job: host::JobAssignment, items: Result<wire::Workspace, wire::Undecodable>,
  spec: Result<wire::JobSpec, wire::Undecodable> }` takes a job from the service, with
  `wire::JobSpec { command: Box<[u8]>, request: Box<[u8]> }` the root's mirror beside
  `wire::Workspace`. The root keeps the spec by the job's run and attempt and hands the hub
  `host::LinkDown::AssignJob`; on `ToJob::Run` it starts the capability with the kept spec and
  the prepared workspace's directory; a spec that did not decode is `FromJob::StartRefused`.
- The capability's `Write`, `Spawn`, `Kill` and `Read` leave the root as one new request,
  `Request::Job { owner: Token, op: tooling::Request }` (the operations only), and their
  terminals come back as `Event::Job { owner: Token, event: tooling::Event }`; its `Started`,
  `StartRefused`, `Ended` and `Gone` become `host::FromJob`'s (`End::Finished { deliver: true }`
  becomes `JobEnd::Finished` with a delivery naming the writable repository).
- `host::Request::DeliverJob` goes to the checkout's push of that repository's head
  (`workspace::write`, `Write::Push`), and its outcome back as `host::Event::Delivered`.
- `hub-jobs`'s interim arms in `src/route.rs` (no job capability) are gone.
- The stories below pass in `temper-worker-domain`'s tests and `temper-worker-domain-world`;
  every existing story passes unchanged.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md`; `notes/contracts.md`, sections 2 and 8.
- `temper/docs/design/domain/worker.md, 3` (the root and its children), `4.4`, `5` (checkouts);
  `temper/docs/design/protocol/tooling.md, 2 and 3`; `jig/docs/design/domain/hosts.md, 6.9`.
- The code: `src/domain.rs` (`Domain`, `step`, `fire`), `src/route.rs` (`from_host` about 228,
  `from_checkout` about 277, `Event::Ready` about 55), `src/workspace.rs` (`prepare`, `write`,
  `release`), `src/wire.rs`, `src/boundary.rs`, `src/limits.rs`; the tooling crate's boundary
  as `tooling-capability` built it.

## Owns and may change

Owns: `temper/crates/temper-worker-domain/**`, `temper/tests/worker/domain/**`.

May change:

- `temper/crates/temper-service/**`: only the arms the new root `Event`/`Request` variants force,
  inert until `service-jobs`: the service never sends `AssignJob` (a `LinkDown::AssignJob` is
  handed on as today's arm does, or refused as an assignment without items), and answers
  `Request::Job { owner, op }` with the operation's failing terminal (`Written { written: false
  }`, `Exited { exit: Exit::Signal, .. }` then `Gone`, `Read { bytes: None }`). Say which arm
  you wrote where.
- `temper/crates/temper-worker-domain-checkout/**`: if `Write::Push` cannot push a repository's
  head when its tree holds nothing to commit (the command committed it), the smallest change to
  its push step that pushes the head as it is, tested in the checkout's own tests; say so.
- The root `Cargo.toml` and `temper-worker-domain/Cargo.toml` for the new dependency.

Pre-approved options:

- **The job's directory** is the prepared workspace's root. The home is the repository the
  items name `home`, so the command's program is `<directory>/home/<command>`; the other
  repositories sit beside it as the checkout lays them out (`<directory>/<name>/`), not under
  `repos/` (`tooling.md, 3`). Say so in the report: the design's layout waits for the first kind.
- **The kinds list's place:** a field of the root's `Limits` or a constructor argument of
  `Domain::new`; either; production's is empty.
- **The delivery's message:** the checkout's push takes a title; give it the job's kind
  (`job <kind>`) and an empty body.

## Steps

1. The composition, the hello's kinds, `Event::AssignJob`, the kept spec.
2. Routing both ways: hub to capability, capability to protocol layer and back, capability to
   hub; delivery to the checkout.
3. Stories, each citing `temper/docs/design/domain/worker.md, 4.4`, in `src/tests` (the
   `Harness`; precedent `lifecycle.rs`'s `a_run_with_no_repository_starts_smith_with_no_workspace`)
   and `temper/tests/worker/domain` (`whole.rs`'s `Case` and `run`, with the fake checkout):
   - `the_hello_states_the_job_capabilitys_kinds` (a root built with kind 9, verb `check`).
   - `a_job_runs_its_command_in_its_prepared_checkout_and_answers_its_result`: the hub's
     `AssignJob`, the checkout prepared, `Request::Job` `Write` then `Spawn` naming
     `<directory>/home/<command>`, scripted `Exited(Code(0))` and the result file read, the
     answer finished with the file's bytes, the workspace released after `Gone`.
   - `a_delivering_kinds_job_pushes_its_writable_repositorys_head_before_answering`, and the
     push refused as stale answering failed undelivered.
   - `a_cancelled_job_is_killed_and_answered_once_its_processes_are_gone`.
   - `a_job_whose_spec_did_not_decode_is_refused_its_start`.
4. `cargo fmt`; `heavy cargo clippy -p temper-worker-domain -p temper-worker-domain-world -p
   temper-service --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-worker-domain -p temper-worker-domain-world -p temper-worker-domain-tooling`; then
   `heavy cargo check --tests -p temper-service` and its worker tests
   (`-E 'test(/worker/)'`).
5. Commits; push your branch.

## Report

As the implementer's text says, and also: the job directory's layout as built (for the design's
status); where the kinds list lives; the service arms left for `service-jobs`; any checkout
change.
