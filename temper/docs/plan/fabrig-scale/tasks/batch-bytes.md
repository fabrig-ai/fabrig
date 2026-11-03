+++
id = "batch-bytes"
part = "jig-rows"
kind = "feature"
needs = ["override-relations", "transcript-ceilings"]
files = ["jig/crates/jig-domain-tasks/src/**", "jig/crates/jig-domain-core/src/routing/run/payloads.rs", "jig/crates/jig-domain-core/src/tests.rs", "jig/crates/jig-domain-core/src/tests/runs/run_choice.rs", "jig/crates/jig-protocol-config/src/overrides.rs", "jig/crates/jig-protocol-config/src/profile/small.rs", "jig/crates/jig-protocol-config/src/tests/overrides.rs", "jig/crates/jig-protocol-store/src/tests/fixtures.rs", "jig/testing/jig-test-profile/src/lib.rs", "jig/tests/tasks/src/world.rs", "jig/tests/tasks/tests/suite/lifecycle.rs", "jig/tests/tasks/tests/suite/proposals.rs", "jig/tests/store/src/root.rs", "temper/crates/temper-engine-domain-forge/src/tests/reconcile.rs", "temper/crates/temper-engine-domain/src/tests/resume_fixture.rs", "temper/tests/engine/domain/src/walking.rs", "jig/docs/design/domain/tasks.status.md"]
effort = "high"
minutes = 120
+++

# A batch's bytes: a limit of the tasks child, checked wherever a batch is

## Goal

The tasks child bounds a batch by its bytes in all, not only by its count, so the store can
price a proposal's creations, a verdict's follow-ups and a recurring template as one piece
(`jig/docs/design/protocol/store.md`, 5.1; the next task, `tasks-pieces`, builds that). This
task adds the limit `tasks.batch_bytes`, its measure, its admission and its refusal; it changes
no codec.

Acceptance:

- `jig_domain_tasks::Limits` has `pub batch_bytes: u32`, documented "Maximum bytes one batch's
  tasks hold in all, as `batch_bytes` measures them, a recurring member's template's tasks
  among them (domain/tasks.md, sections 4 and 11)." `valid` refuses `batch_bytes == 0`.
- `jig_domain_tasks::batch_bytes(batch: &[New]) -> Option<u64>` is public (exported from
  `lib.rs`): `owned.rs`'s private `batch_bytes`, made public, and for a member with
  `recurring: Some(template)` it adds `size_of::<RecurringTemplate>()` and the template's
  batch's own `batch_bytes`. `None` is overflow. `owned.rs`'s `proposal_bytes` takes it for its
  `ProposalAction::Batch` arm (it then also counts the members' LLM choices).
- Wherever the tasks child bounds a batch's count by `limits.batch`, it bounds the batch's
  bytes by `limits.batch_bytes` beside it (`None` counts as past), with the same refusal:
  - `batch.rs`, `check_with_result_proposal`: `Refusal::Batch`, right after the count's check
    (so a Make, an accepted proposal's batch, a verdict's follow-ups, a procedure's step and a
    recurring period's batch are all held);
  - `batch.rs`, `bounded_batch` (a person's proposal and the core's retained batch);
    `recurring.rs`, `valid_template`; `proposals.rs`, `check_result`'s batch arm (a result's
    proposal): each returns `false`;
  - `proposals.rs`, `admission_problem_staged`, whose `ProposalAction::Batch(_)` arm checks
    nothing today (the core counts a run's proposed batch before): a batch past its count or
    its bytes is `proposal_problem(proposal.proposer, Refusal::Batch)`;
  - `stored/fit.rs`, `batch`: a stored batch past the bytes is `RestoreBound::Batch` (the
    `bound(..)` helper the file uses for byte values);
  - `run.rs`, `invalid_followups`: follow-ups (a `Delegates` batch, or a proposal's `Batch`)
    past the bytes are `Some(InvalidResult::Followups)`.
  No new `Refusal`, `InvalidResult` or `RestoreBound` variant.
- The core's answer route (`jig/crates/jig-domain-core/src/routing/run/payloads.rs`,
  `route_answer_payload`) treats an end whose follow-ups' batch is past `tasks.batch_bytes` as
  it treats `invalid_saved`: the end becomes `End::Failed(Class::Invalid)` before the terminal
  is recorded, so no terminal the store cannot price is ever committed.
- `tasks.batch_bytes` is an override (`jig-protocol-config/src/overrides.rs`, `set_tasks`:
  the destructure and `b"batch_bytes" => *batch_bytes = count(value)?`).
- small's `TASKS` sets `batch_bytes: 131_072`, with a comment: "A batch's bytes: two of small's
  largest tasks, one of them recurring with a template of two, hold under 90 KiB; 128 KiB." The
  standard profile inherits it (`..small::TASKS`); `standard-scale` sets its own.
- The status line's **Not yet** "4, 11: a batch's bytes: ..." in
  `jig/docs/design/domain/tasks.status.md` is removed, the rest of the line kept.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K2 (the last bullet).
- `jig/docs/design/domain/tasks.md`, 4 ("Checked whole", "within the limits") and 11 (the
  bullet "a batch's bytes"); 5.6 ("Checked twice").
- `jig/docs/design/protocol/store.md`, 5.1 (the bullets "The tasks family's pieces" and "No
  batch inside a batch"), for why; nothing of the store changes here.

## Owns and may change

- `jig-domain-tasks` (its `src/**`), for the above and its tests.
- **Every literal of `jig_domain_tasks::Limits` that lists all fields** gains one line,
  `batch_bytes: 131_072,` (small's), and nothing else: `jig/testing/jig-test-profile/src/lib.rs`,
  `jig/crates/jig-domain-core/src/tests.rs`, `jig/crates/jig-protocol-store/src/tests/fixtures.rs`
  (`TASKS_LIMITS`), `jig/tests/tasks/src/world.rs`, `jig/tests/store/src/root.rs`,
  `jig/crates/jig-domain-tasks/src/tests.rs` and `src/tests/revisions.rs`, and in temper (the
  one line each; a struct field the compiler requires):
  `temper/crates/temper-engine-domain-forge/src/tests/reconcile.rs`,
  `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`,
  `temper/tests/engine/domain/src/walking.rs`. Find any other with
  `grep -rln "escalation_stall:" --include=*.rs jig temper`. If a test of a fixture then fails
  with `Refusal::Batch` (or a restore's `RestoreBound::Batch`) on a batch within its count,
  raise that fixture's figure to the least power of two that passes and list it in the report.
- `jig-protocol-config`: `overrides.rs` (the two lines above) and `profile/small.rs` (the line
  and its comment), and `src/tests/overrides.rs`'s `SMALL` table, which gains
  `(b"tasks.batch_bytes", 131_072),` after `tasks.batch`. No other file of the crate.
- `jig-domain-core`: `routing/run/payloads.rs` and the test below; no other production file.
- If a check in `jig-domain-core` other than the answer route (a delegation's preflight, a
  procedure's batch admission) counts a batch against `tasks.batch`, leave it: the tasks
  child's check is the batch's one admission, and its `Refusal::Batch` reaches the caller as
  today.
- If you find a path that stores a batch the tasks child never checked (a `New` batch written
  to a row without passing one of the checks above), stop and ask, naming it.

## Steps

1. `limits.rs`: the field, its doc, `valid`. `owned.rs`: `pub fn batch_bytes`, counting a
   member's template; `proposal_bytes` takes it. `lib.rs`: `pub use owned::batch_bytes` (beside
   the existing exports).
2. A private helper in `batch.rs`, `pub(crate) fn within(limits: &Limits, batch: &[New]) -> bool`
   (count and bytes), used by every check listed; keep each check's existing order otherwise.
3. `stored/fit.rs` and `run.rs` as above; `payloads.rs` as above (the end's follow-ups are
   `tasks::ResultFollowups::Delegates(batch)` or `Proposal(p)` with `ProposalAction::Batch(batch)`).
4. Override and small's figure; the fixture lines.
5. Tests:
   - `jig-domain-tasks`, `src/tests.rs`, `valid_refuses_each_unrepresentable_limit`: a row
     `("batch_bytes zero", crate::Limits { batch_bytes: 0, ..limits }, false)`.
   - a new module `src/tests/batch_bytes.rs` (`mod batch_bytes;` in `src/tests.rs`):
     `a_batch_counts_its_members_templates` (a recurring member's template adds its members'
     bytes and the template's own); `a_batch_past_its_bytes_is_not_bounded` (`bounded_batch`
     and `recurring::valid_template` at the batch's bytes pass, one byte less refuse);
     `followups_past_the_batch_bytes_are_invalid` (`run::invalid_followups` gives
     `Some(InvalidResult::Followups)` one byte under, `None` at); and
     `a_restored_batch_past_its_bytes_is_the_batch_bound` (the fit check of a stored proposal's
     batch).
   - `jig/tests/tasks/tests/suite/lifecycle.rs`,
     `batch_is_atomic_and_cycles_and_limits_refuse_at_entrance`: a row whose limits are
     `Limits { batch_bytes: <the two tasks' batch_bytes> - 1, ..LIMITS }` refused with
     `Refusal::Batch` (compute it with `jig_domain_tasks::batch_bytes`).
   - `jig/tests/tasks/tests/suite/proposals.rs`: `a_proposed_batch_past_its_bytes_is_refused`:
     a person's proposal through `propose_batch`, in a world whose `batch_bytes` is one less
     than the batch's, is refused and no `Key::PersonProposal` row is kept; in a world whose
     limit is the batch's bytes it is kept (the `bounded_batch` path).
   - `jig/crates/jig-domain-core/src/tests/runs/run_choice.rs`:
     `an_answer_whose_followups_pass_the_batch_bytes_is_invalid`, modelled on
     `an_invalid_start_spends_a_try_across_fences_and_endings`: an `AnswerPayload` whose end is
     `FinishedWithFollowups` with a `Delegates` batch past the limits' `batch_bytes` records the
     terminal `End::Failed(Class::Invalid)`.
6. Narrow runs, one at a time:
   `heavy cargo nextest run -p jig-domain-tasks --lib -E 'test(/batch_bytes::|valid_refuses_each_unrepresentable_limit/)'`;
   `heavy cargo nextest run -p jig-tasks-world --test suite -E 'test(=lifecycle::batch_is_atomic_and_cycles_and_limits_refuse_at_entrance) + test(=proposals::a_proposed_batch_past_its_bytes_is_refused)'`;
   `heavy cargo nextest run -p jig-domain-core --lib -E 'test(=tests::runs::run_choice::an_answer_whose_followups_pass_the_batch_bytes_is_invalid)'`.
   Then the packages: `-p jig-domain-tasks`, `-p jig-tasks-world`, `-p jig-domain-core`,
   `-p jig-core-world`, `-p jig-protocol-config`, `-p jig-protocol-store --lib`,
   `-p jig-store-world`, `-p jig-test-service`; and temper's fixtures:
   `heavy cargo nextest run -p temper-engine-domain-forge --lib`,
   `-p temper-engine-domain --lib`, `-p temper-engine-domain-world`, and
   `heavy cargo check -p temper-service --tests`.
7. The status line; `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-domain-tasks -p jig-domain-core -p jig-protocol-config -p jig-tasks-world -p jig-test-profile -p jig-store-world -p jig-protocol-store -p temper-engine-domain -p temper-engine-domain-forge -p temper-engine-domain-world --all-targets -- -D warnings`;
   the citations check; commit and push.

## Report

As the implementer's text asks, with: each fixture whose `batch_bytes` is not 131,072 and why;
`size_of::<New>()` and `size_of::<RecurringTemplate>()` on the build machine; and any check
outside the tasks child that counts a batch against `tasks.batch`, left as it is.
