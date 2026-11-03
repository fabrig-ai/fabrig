+++
id = "change-merge-route"
part = "change-merge"
kind = "feature"
needs = ["change-merge-surface", "change-merge-steps", "merge-settings"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/**", "temper/crates/temper-engine-domain/Cargo.toml", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/tests/engine/domain/src/**", "temper/tests/engine/domain/tests/suite/forge_routes/**", "temper/tests/engine/domain/tests/suite/forge_routes.rs"]
effort = "high"
minutes = 170
+++

## Goal

The engine makes the merge job and reads it (`temper/docs/design/domain/forge.md, 8.4`;
`temper/docs/design/domain/worker.md, 4.4`): the root builds the job's delegate with C2's spec,
gives a change the merge kind where its run's authority holds it, keeps the job's result for the
resolution's words; the forge fills the step's facts. Stories in the engine's domain world show
a conflict settled by the job alone, and one it leaves to a resolution, under rules asking no
update (a goal's branch) and under rules asking one (main).

Acceptance:

- `change_delegate`'s `Merge` arm makes a job task: `tasks::Executor::Job { kind: MERGE_KIND }`,
  `Spec::job` `Some(JobSpec { bytes, deadline })` with C2's bytes and the core's declared
  deadline for kind 3, no LLM choice, the item `change-merge-surface` made; a deployment whose
  core declares no kind 3 never reaches this arm (`merge_job` false);
- `change_delegate_status` reads a `ResultKind::Job` result as the delegate's `Passed`, keeping
  its bytes, at most `MERGE_RESULT_BYTES`, as `ChangeRow::merge_result`; a job's failure as a
  delegate's failure;
- the `Resolve` arm's words, when `merge_result` is `Some`, are its line today, then a blank line,
  then `The merge job settled what the project's merge command knows and left the rest:`, then
  the result's bytes cut from their end to the words' room and marked as the root marks a cut;
  `merge_result` is cleared once that resolution is made;
- `chat_change` gives the change `AuthorityExecutor::Job(3)` exactly where the run's authority
  delegates to it (C5); the forge's facts builder fills C3's `merge_job`, C3.1's `delegate` (the
  row's) and `catch_up` (the row's), and `contains_catch_up` (the comparison of the landing
  branch's tip with the commit, read as `contains_base` is; `Pending` until read);
- the hold's words for `Hold::Resolutions` under rules asking no update
  (`hold_change_decision`) are the resolutions hold's words every change has: the special words
  saying the rule asks no update go, since such a change is now resolved;
- engine domain world stories, each citing `temper/docs/design/domain/forge.md, 8.4`:
  - `a_conflict_into_a_goal_branch_is_settled_by_the_merge_job_and_merges` (rules asking
    neither; the project names `merge_job`; the run's authority holds `Job(3)`): the merge is
    refused for a conflict, a job of kind 3 is assigned whose spec decodes to C2's command and
    request, the world pushes a merge onto the change's branch and answers the job finished,
    and the change merges at that head;
  - `a_conflict_the_merge_job_leaves_goes_to_a_resolution` (same, the world answers the job
    finished without pushing): a resolution is assigned whose words carry the job's result,
    and after its push the change merges;
  - `an_update_conflict_goes_to_the_merge_job_before_a_resolution` (rules asking an update):
    the forge's update conflicts, the job pushes, and the change is checked again at the new head;
  - `without_a_merge_job_a_conflict_goes_to_a_resolution` (no `merge_job`): today's resolution,
    under rules asking neither, where the change used to hold;
  - `a_conflict_under_rules_asking_neither_says_why_it_holds` is rewritten to the new rule:
    a change past its resolutions there holds with the resolutions hold's words.

## Inputs

- `temper/docs/design/domain/forge.md, 8.2` (step 6), `8.4` ("The merge job", "A resolution",
  "A step its authority no longer covers"), `5.3`; `temper/docs/design/domain/worker.md, 4.4`
  ("The merge kind", "The spec is written by the engine's root");
  `temper/docs/design/protocol/tooling.md, 4.1`; `temper/docs/design/domain/agent.md, 4.3.1`
  (the change's authority).
- `temper/docs/plan/reference-m7b/notes/contracts.md`, C1 to C5 and C3.1; `notes/common.md`.

## Owns and may change

- `temper/crates/temper-engine-domain/src/route/forge_route.rs`: `change_delegate` (`Merge`,
  `Resolve`'s words), `change_delegate_status`, `chat_change`, `hold_change_decision`, and a
  small encoder of C2's request and spec (a private function or a `route/forge_route/merge.rs`
  module).
- `temper/crates/temper-engine-domain/Cargo.toml`: `temper-protocol-worker-wire` as a
  dependency (C2: the one protocol record the root writes; `worker.md, 4.4` states why).
- `temper/crates/temper-engine-domain-forge/src/lib.rs`: C1's `pub const MERGE_KIND: u32 = 3;`.
- `temper/crates/temper-engine-domain-forge/src/domain.rs`: the facts builder; keeping
  `merge_result` on the row when the root hands it the job's result; whatever the forge needs to
  know the core declares kind 3 (a flag the root sets at construction, beside
  `project_merge_job`).
- **The change's authority in the facts:** the root's view of the change task's authority
  reaches the forge's step as `Heard::lacking` does; add what you need on that per-step input (a
  bool, never stored), and say how in the report. If that cannot reach the facts builder
  without a wider change, compute `merge_job` without it (project and kind only) and let a
  change whose authority lacks the kind hold through `missing()` as any delegate it cannot make;
  say so in the report.
- Tests: `temper/tests/engine/domain/tests/suite/forge_routes/` (`light_rules.rs`, a new
  `merge_job.rs` registered where its siblings are) and `forge_routes.rs`; the world's support
  files under `temper/tests/engine/domain/src/` where a story needs a helper to answer a job
  (as `world.protocol.answer` answers a run), to configure a project's `merge_job`, or to
  declare kind 3 in the world's core configuration (`merge-kind` declares it in the service;
  the world builds its own).
- Readers: tests of `hold_change_decision`'s words elsewhere in temper, updated to the
  resolutions hold's words only.
- Not this task's: `change`'s `merge` member (`catch-up-call`), the forge-change crate (a step
  that does not decide as `change-merge-steps` says is a question).

## Steps

1. The facts builder; the root's arms; the encoder (C2: `JobSpec::new` of the worker wire's
   generated code, its request as C2 spells it).
2. Unit tests in the root where its route tests live (`route/forge_route/tests.rs`):
   `a_merge_jobs_spec_is_its_command_and_its_two_commits` (decodes C2's bytes with the worker
   wire's decoder and checks both fields), `a_resolution_after_a_merge_job_carries_its_result`.
3. The world stories above.
4. `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/merge/)'`, then
   `heavy cargo nextest run -p temper-engine-domain-world -E 'test(/merge_job|light_rules|conflict/)'`,
   then the packages' tests: `-p temper-engine-domain`, `-p temper-engine-domain-forge`,
   `-p temper-engine-domain-world`; `heavy cargo check -p temper-service --tests` (a reader);
   `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy on
   `temper-engine-domain`, `temper-engine-domain-forge` and `temper-engine-domain-world`,
   `--all-targets -- -D warnings`.
5. This changes what a conflict under rules asking no update means: run the forge's world too,
   `heavy cargo nextest run -p <temper/tests/engine/forge's package> -E 'test(/conflict|settle/)'`.
6. Commit and push.

## Report

As the implementer's text says; say how the change's authority reached the facts.
