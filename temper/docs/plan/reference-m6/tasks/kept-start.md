+++
id = "kept-start"
part = "kept-jig"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/**", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-protocol-config/src/core.rs", "jig/tests/tasks/**", "jig/tests/core/**", "jig/docs/design/domain/connectors.status.md"]
effort = "high"
minutes = 90
+++

# kept-start: a procedure that starts kept

## Goal

A connector may declare, per procedure, that its tasks made with a pending dependency are
active and kept rather than waiting (contract C1). jig's tasks child admits such a task as
it admits any, then takes it out of `Phase::Waiting` as soon as its holds are taken, so its
procedure starts and steps while its dependency goes on, under every refusal of a kept task.

Acceptance:

- `jig_domain_tasks::KeptStart`, `Domain::set_kept_starts` and `jig_domain_core::Config`'s
  `kept_starts` exist as C1 has them; the core gives them to its tasks child.
- A procedure task whose `(connector, code)` is a kept start, made with a pending
  dependency, becomes active once its holds are taken (its connector asked to start it,
  `Ask::StartProcedure`), and is kept (tasks.md, 5.7): its step that ends it done is refused
  whole, its effect of a kind that waits for dependencies is refused naming the dependency,
  its step that holds it or ends it failed is admitted. Its dependency's done wakes it with
  the `Met` notice as for a task kept by an amendment, after which its done step is
  admitted.
- The same task made with no pending dependency, or with only done ones, is as today.
- A procedure task of a code no kept start names, made with a pending dependency, waits as
  today (`Phase::Waiting`), and so does every agent, person and job task.
- A kept-start task whose holds are not yet taken waits for them as any task does (6.2),
  and is not refused a hold's queue for its pending dependency (`holds::queued`).
- The stories below pass, and every story of `jig/tests/tasks` and `jig/tests/core` passes.

## Inputs

- `jig/docs/design/domain/connectors.md`, 6.1 ("A procedure that starts kept", first
  paragraph and the declaration's sentence) and 6.2 (when it steps).
- `jig/docs/design/domain/tasks.md`, 5.1 (a task at creation where its procedure starts
  kept is kept) and 5.7 (kept by a dependency).
- `jig/docs/design/domain/connectors.status.md`: its **Not yet** entry "6.1: a procedure that
  starts kept: no connector declares one (...)." up to its full stop, which this task
  removes; the next entry, "6.1, 6.3: a dependency marked started from ...", is task
  `kept-started-from`'s.
- `temper/docs/plan/reference-m6/notes/common.md`; `notes/contracts.md`, C1.

## Owns and may change

Owns the files in the header.

- Where the change goes: admission builds the record in `domain.rs`, `make_admitted`
  (`phase: Phase::Waiting`, `waiting_on`); a waiter is activated by `closing::progress`'s
  `Phase::Waiting` arm once no `waiting_on` entry is live. Its readiness admits a kept start
  whatever its pending dependencies; keep `waiting_on` as is, so `run/ends.rs`'s `Kept`
  computes the keep from it. Readers that assume a task leaving `Waiting` has no pending
  dependency (`holds::queued` at `holds.rs`, the check near `holds.rs:452`, `milestone`'s
  `Why::Made` in `domain.rs`) admit a kept start; change them only for that.
- Reader and neighbour files: every `jig_domain_core::Config { .. }` literal, its new field
  `kept_starts: Box::new([])` only: `jig/crates/jig-domain-core/src/tests.rs`,
  `jig/testing/jig-test-profile/src/lib.rs`, `jig/testing/jig-test-domain/src/tests/fixtures.rs`,
  `jig/tests/core/src/oracle.rs`, `jig/tests/engine-clients/src/fixtures/receiving.rs`, and
  temper's `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`,
  `temper/tests/engine/domain/src/walking.rs`, `temper/tests/engine/domain/src/oracle/config.rs`
  (the other tree's lines, the field alone).
- Options: if the core's tasks child is made elsewhere too (a restore path), hand the kept
  starts there as well and say so. If a world builds `tasks::Domain` directly
  (`jig/tests/tasks/src/world.rs`), give it a way to set kept starts for its stories in the
  world's own code.

## Steps

1. C1's type, setter and `Config` field; the core passes `config.kept_starts` to its tasks
   child right after `tasks::Domain::new(..)`.
2. The readiness rule in the tasks child, and the readers above.
3. Stories in `jig/tests/tasks`, a new suite file `tests/suite/kept_start.rs` (its `mod` line
   in the suite's main), modelled on `met.rs`'s
   `a_kept_procedure_wakes_on_met_steps_with_no_pending_and_can_end_done` and `jobs.rs`'s
   `a_procedures_job_is_due_once_its_dependency_is_done_and_ends_with_its_result`:
   - `a_kept_start_procedure_made_with_a_pending_dependency_is_active_and_kept`: active at
     once, its done step refused while kept, its hold and failure steps admitted;
   - `a_kept_start_procedure_goes_on_when_its_dependency_is_done`: the `Met` notice, then
     its done step admitted;
   - `a_procedure_of_another_code_made_with_a_pending_dependency_waits`.
4. One core story in `jig/tests/core` (`tests/stories/dependencies.rs` or
   `tests/suite/effect_fences.rs`, beside
   `a_kept_procedures_waiting_effect_is_refused_naming_its_pending_dependency`):
   `a_configured_kept_start_starts_its_procedure_and_refuses_its_waiting_effect`, through
   the oracle's `Config` with one kept start.
5. Remove the status text above. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-domain-tasks -p jig-domain-core -p jig-protocol-config
   --all-targets -- -D warnings`; the focused tests of `jig-domain-tasks`, `jig-domain-core`
   and the `jig/tests/tasks` and `jig/tests/core` worlds; `heavy cargo check --tests` of each
   temper package whose literal you edited (`-p temper-engine-domain -p
   temper-engine-domain-world`, or the package names `cargo metadata` gives). Commit, push.

## Report

As the implementer's text says; name where the readiness rule sits and each reader you
changed for it.
