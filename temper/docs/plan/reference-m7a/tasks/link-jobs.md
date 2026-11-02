+++
id = "link-jobs"
part = "worker-channel"
kind = "feature"
needs = ["worker-wire-jobs", "hub-jobs"]
files = [
  "jig/crates/jig-protocol-worker-link/**",
  "jig/crates/jig-protocol-engine-workers/**",
  "jig/tests/worker-link/**",
  "jig/tests/engine-workers/**",
]
effort = "high"
minutes = 120
+++

# Jobs cross the worker channel: both links translate the job's records

## Goal

Both ends of jig's worker channel translate jobs (`jig/docs/design/protocol/channel.md, 5.4`):
the worker's link encodes the hello's kinds and a job's answer and decodes a job; the engine's
side decodes the hello's kinds and a job's answer and encodes a job. `worker-wire-jobs` gave the
records, `hub-jobs` the domain values and interim arms (`LinkUp::JobAnswer` not sent,
`LinkDown::AssignJob` not encoded), which this task replaces.

Acceptance:

- `jig-protocol-worker-link`'s `translate::hello` carries `Hello::jobs` whole; `up` encodes
  `LinkUp::JobAnswer` as a `JobAnswer` frame; `down` decodes a `Job` record into
  `LinkDown::AssignJob`, its workspace items handed to the root as an assignment's are.
- `jig-protocol-engine-workers`'s `hello` decodes `jobs`; `up`/`answer` decode `JobAnswer` into
  `LinkUp::JobAnswer`; `down` encodes `LinkDown::AssignJob` as a `Job` record, its workspace
  items and grants as an assignment's are.
- Both translations are total and exhaustive both ways; a record past its bounds is refused as
  the links refuse an assignment past them.
- Kept answers go again after a redial in order, a job's among them (`channel.md, 5.3`).
- The stories below pass, in both link worlds; every existing story passes unchanged.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md`; `notes/contracts.md`, sections 1 to 3.
- `jig/docs/design/protocol/channel.md, 5.1` to `5.4` and `section 12` (testing).
- The code: `jig-protocol-worker-link/src/translate.rs` (`hello` about 10, `up` about 66,
  `ending` about 214, `down` about 390, `assignment` about 537); `jig-protocol-engine-workers/src/
  translate.rs` (`hello` about 6, `up` about 130, `answer` about 285, `down` about 469,
  `assignment_record` about 549); the worker wire's generated `v9.rs` and `kinds.rs` as
  `worker-wire-jobs` left them (its report names the job enums' wrapper records).

## Owns and may change

Owns: `jig/crates/jig-protocol-worker-link/**`, `jig/crates/jig-protocol-engine-workers/**`,
`jig/tests/worker-link/**`, `jig/tests/engine-workers/**`.

May change: the reference service's and worker's routing of the new records, if they match
link frames by kind (`jig/testing/jig-test-service/**`, `jig/testing/jig-test-worker/**`), arms
only, as their assignment's and answer's arms.

Pre-approved options:

- **A job's spec in the parts table:** carry it inline in the record (it is bounded by its
  kind) rather than as a part, unless the links' limits require parts; then as a part, as
  smith's charter is.
- If the terms' largest-body computation (`kinds.rs`'s rules) does not cover the new records
  as `worker-wire-jobs` wrote them, fix the rule in `jig-protocol-worker-wire/src/kinds.rs`
  (that one file) and say so.

## Steps

1. Worker side: `hello`, `up`'s `JobAnswer` (with `ending`'s job twin), `down`'s `Job`.
2. Engine side: `hello`, `up`/`answer`'s `JobAnswer`, `down`'s `AssignJob`.
3. Stories, each citing `jig/docs/design/protocol/channel.md, 5.4`:
   - in `jig/tests/worker-link` (its `World` and `ScriptedPeer`; precedent
     `the_hello_is_the_first_record_on_every_channel`): `a_hello_carries_its_job_kinds`;
     `a_job_record_reaches_the_hub_as_an_assign_job`; `a_job_answer_goes_up_and_again_after_a_redial`.
   - in `jig/tests/engine-workers` (precedent `a_hello_not_admitted_is_refused_with_the_fleets_text`):
     `a_job_is_encoded_with_its_workspace_and_grants`; `a_job_answer_reaches_the_fleet_as_link_up`
     (finished, failed undelivered stale, refused invalid kind).
   - In each world's seeded load story (`cascades`), a job mixed with runs keeps the records'
     order.
4. `cargo fmt`; `heavy cargo clippy -p jig-protocol-worker-link -p jig-protocol-engine-workers -p
   jig-worker-link-world -p jig-engine-workers-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p jig-protocol-worker-link -p jig-protocol-engine-workers -p
   jig-worker-link-world -p jig-engine-workers-world`; then `heavy cargo check --tests -p
   temper-service` and its worker tests.
5. Commits; push your branch.

## Report

As the implementer's text says, and also: where the job's spec travels (inline or as a part).
