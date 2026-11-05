+++
id = "job-refused"
part = "job-refused"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-host/src/boundary.rs", "jig/crates/jig-domain-host/src/tests.rs", "jig/crates/jig-domain-host/src/tests/**", "jig/examples/ops/domain/src/link.rs", "jig/testing/jig-test-worker/src/domain.rs"]
effort = "medium"
minutes = 20
+++

# jig: the fixed answers to a job leave the root's template

## Goal

D004's part 3. `ops`'s `host_out` spells the fixed answers to a job it has no capability for, and `jig-test-worker` spells
the same two values. `ToJob::refused` spells them once. The choice to refuse stays in the root's arm, so no capability
choice moves into jig.

Acceptance:

- `rg -n 'no job capability' jig temper --glob '*.rs'` finds one site, in `jig-domain-host`, and its test.
- `ops`'s `host_out` has one `host::Request::Job` arm, and so has `jig-test-worker`'s `down`.
- `job_refused_answers_a_run_and_a_stop` passes.
- Production net at most +3: `ops` -8, `jig-domain-host` about +10.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `jig/docs/design/domain/hosts.md`, 6.9: this cut has already written its last bullet.
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`

## Owns and may change

- **Owns:** `jig-domain-host/src/boundary.rs` (`impl ToJob`); a test in the crate's test modules.
- **May change, these arms only:** `jig/examples/ops/domain/src/link.rs` (`host_out`),
  `jig/testing/jig-test-worker/src/domain.rs` (`down`).
- **Stays:** the `DeliverJob` and `Prepare` arms. A `Delivery::refused` would save no line, and `Prepare` has one caller.
- **Not this task:** `temper-worker-domain/src/jobs.rs`, whose delivery is a real capability's.

## Steps

1. `boundary.rs`:

   ```rust
   impl ToJob {
       /// What a root with no job capability answers (domain/hosts.md, 6.9).
       #[must_use]
       pub fn refused(self) -> FromJob
   }
   ```

   `Run` to `StartRefused { owner, why: b"no job capability" }`, `Stop` to `Gone { owner, detail: empty }`.

2. Both roots: `host::Request::Job(job) => work.push(Work::Host(host::Event::Job(job.refused())))`, and the test worker's
   `hub(..)` form.

3. `job_refused_answers_a_run_and_a_stop` pins both answers. No `ops` test reaches these arms, so this is the only pin.
   Break by hand: swap the arms' answers; see it fail; restore.

4. `heavy cargo nextest run -p jig-domain-host -p jig-test-worker -p jig-ops-domain`, then `heavy cargo nextest run -p
   jig-host-world -p jig-ops-world -p jig-engine-workers-world -p jig-one-process-world`.

5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-host -p jig-test-worker -p
   jig-ops-domain --all-targets -- -D warnings`; the citations check. Commit and push.

## Report

The common note's items, with the break above.
