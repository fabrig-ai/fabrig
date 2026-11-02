+++
id = "tasks-jobs-surface"
part = "job-tasks"
kind = "interface"
needs = ["client-jobs"]
files = [
  "jig/crates/jig-domain-tasks/**",
  "jig/crates/jig-domain-authority/**",
  "jig/crates/jig-protocol-store/**",
  "jig/tests/store-codecs/**",
]
effort = "medium"
minutes = 180
+++

# A job task's surface: the executor, spec, contract and result in jig's domain and store

## Goal

Give jig's tasks child, authority and store the job task's values (`notes/contracts.md`,
sections 4 and 6; `jig/docs/design/domain/tasks.md, 5.3.1`; `jig/docs/design/protocol/store.md,
5`) and every arm they force across jig and temper, with no job task possible yet: the tasks
child's batch admission refuses `Executor::Job` as it refuses an executor it does not admit.
The lifecycle (`tasks-jobs-lifecycle`), the core's admission (`core-jobs-admit`) and its run
(`core-jobs-run`) build on this surface; this task decides no job behaviour beyond the inert
arms written here.

Acceptance:

- `jig-domain-tasks`: `Executor::Job { kind: u32 }`, `JobSpec`, `Spec::job`, `Contract::Job {
  bytes }`, `TaskResult::Job { bytes }`, `ResultKind::Job`, `AuthorityExecutor::Job(u32)`;
  `jig-domain-authority`: `Executor::Job(u32)`; all exactly as contracts, section 4.
- The batch check (`src/batch.rs`, the executor match) refuses `Executor::Job { .. }` with
  `Refusal::Executor`, and refuses a `Spec::job` present on any other executor the same way.
- `jig-protocol-store`'s `tasks-v9`, `people-v14` and `settings-v1` schemas hold contracts,
  section 6, in place; generated code, `layouts.rs` and goldens regenerated; translations
  exhaustive both ways; the store's codec, drift and words tests pass.
- The core's documents carry the kind: `jig-domain-core`'s `documents::ExecutorKind` gains
  `Job`, `task_tree::ContractLine` gains `Job { bytes: u32 }`, and the engine's client
  translation maps them, the task's executor, its chip and a delegation's job kind to the client
  wire's job arms (`client-jobs` added them), replacing the three refusals `client-jobs` left
  in `jig-protocol-engine-clients` (`queries.rs`' executor-kind filter, `settings/**`'s
  `AuthorityExecutor::Job`) with translations; a template's `Job` contract stays refused.
- Every package the compiler names builds, and its tests pass.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md` and `notes/contracts.md`, sections 4, 6
  and 7.
- `jig/docs/design/domain/tasks.md, 3` (a task's spec, contract and result) and `5.3.1`;
  `jig/docs/design/domain/authority.md, 3` (delegation's kinds) and `8.1`;
  `jig/docs/design/protocol/store.md, 5` and `5.1` (codecs: total translations, golden bytes).
- The precedent for an appended result arm through tasks, store, core, engine-clients and
  temper: commit `32c5d659d3` (`no_change`), `git show --stat 32c5d659d3`.
- The engine-side refusals `client-jobs` left (`rg -n 'Job' jig/crates/jig-protocol-engine-clients/src/translate/`), and
  `jig/crates/jig-protocol-engine-clients/src/translate/owner.rs` (`executor` at about line 83,
  `contract` about 436, `specification` about 360), `chips.rs` (`chip_executor` about 376),
  `queries.rs` (about 35), `settings/change.rs`.

## Owns and may change

Owns: `jig/crates/jig-domain-tasks/**`, `jig/crates/jig-domain-authority/**`,
`jig/crates/jig-protocol-store/**`, `jig/tests/store-codecs/**`.

May change, for the arms and literals the new variants and the new `Spec::job` field force,
each edited only for that:

- `jig/crates/jig-domain-core/**`, `jig/crates/jig-domain-people/**`,
  `jig/crates/jig-protocol-config/**`, `jig/crates/jig-domain-views/**`,
  `jig/crates/jig-domain-hub-link/**`, `jig/crates/jig-protocol-engine-clients/**`,
  `jig/testing/**`, `jig/examples/**`, `jig/tests/**`.
- temper: `temper/crates/temper-engine-domain/**` (`route/forge_route.rs`, `route/asks.rs`),
  `temper/crates/temper-engine-smith/**` (`nested.rs`), `temper/crates/temper-service/**`,
  `temper/tests/**`, and any other temper file the compiler names.
- `Spec` literals gain `job: None`.

The inert arms, where a match decides what an executor does:

- **Where the arm makes work happen** (activation, a run's preparation, a step, an inbox entry,
  a wake, an offer), `Job` takes the arm that makes none: as `Person` does in
  `jig-domain-core/src/routing/run/preparation.rs`'s `route_activate` (nothing), or the
  match's refusal where it has one. A job task cannot exist yet, so none of these runs.
- **Where the arm describes** (documents, chips, views, history, temper's render of an
  executor), `Job` maps to its own new variant, or to its kind's number, never to another
  executor's.
- **In temper's forge** (`forge_route.rs`), a `Job` executor is not the forge's: the arm the
  file gives an executor the forge does not own (a person's, or a refusal).
- Never `panic!`, `unreachable!`, `todo!` or `_`.

Pre-approved options:

- **`Spec::job`'s place in the store:** an optional field appended to `record Spec` if the
  schema's records take appended optional fields; if they must be a new record, add
  `record JobSpec` and make `Spec`'s field an `option JobSpec`, and say so.
- **The spec's bytes bound in the store:** `Limits::spec_bytes`, as words are; the deadline as
  the schema's durations are.
- If `jig-domain-people`'s `TaskExecutor` (a party's Create) would need `Job` for a match, it
  does not gain it: a party creates no job; map at the boundary that produces it instead, and
  say so. `temper-protocol-store` (which embeds people's `TaskExecutor`) then needs nothing.
- If a golden of temper's (`temper-client-tools/src/tests/golden/`) would change, it should
  not: stop and say which, in the report, rather than regenerate it.

## Steps

1. The domain variants and `JobSpec`, in `jig-domain-tasks/src/boundary.rs`, `value.rs`; the
   authority's in `jig-domain-authority/src/value.rs`; `Spec::job` literals.
2. The batch check's inert refusal (`src/batch.rs`, the executor match): `Job` refused
   `Refusal::Executor`; `Spec::job` on another executor refused the same way. A unit test:
   `a_batch_naming_a_job_is_refused_until_jobs_are_admitted` beside the batch's tests
   (`tasks-jobs-lifecycle` replaces it).
3. The store: the three schemas, appended arms; translations (`codec/tasks/value.rs`
   `executor`/`read_executor`, `spec`/`read_spec`, `contract`/`read_contract`,
   `authority_executor`; `codec/tasks/phase.rs` `result`/`read_result`/`check_result`;
   `codec/people/batch/parts.rs`; `codec/settings.rs`); regenerate (`notes/common.md`); unit
   tests that a job executor, spec (with its deadline), contract, result and each family's
   delegation job kind round-trip, and that a spec's bytes one past their bound are refused.
4. Every forced arm, by the rules above; the core's `ExecutorKind::Job` and `ContractLine::Job`;
   the engine-clients mappings to the client wire's job arms, and the two of `client-jobs`'s
   refusals replaced (a query on executor kind `job` now filters on `ExecutorKind::Job`; a
   party's delegation kind `job` becomes `authority::Executor::Job`).
5. Narrow tests first (the store's codec tests, the batch test), then `cargo fmt`; `heavy cargo
   clippy -p jig-domain-tasks -p jig-domain-authority -p jig-protocol-store -p jig-domain-core
   -p jig-protocol-engine-clients --all-targets -- -D warnings`; `heavy cargo nextest run -p
   jig-domain-tasks -p jig-domain-authority -p jig-protocol-store -p jig-store-codecs-world -p
   jig-domain-core -p jig-protocol-engine-clients -p jig-tasks-world -p jig-core-world`; the
   words tests (`notes/common.md`); then `heavy cargo check --tests` for every other package
   you changed, jig's and temper's, and the tests of `temper-engine-domain`.
6. Commits: domain and store, then forced arms; push your branch.

## Report

As the implementer's text says, and also: every inert arm you wrote in a deciding match, by
file and function (the later tasks replace them: list them so they can); the store's shape for
`Spec::job`; any arm where neither rule above fit and what you did.

## Answers

2026-11-02. **Trim the turn fixture's input, or raise `TURN_OVERLAP_BYTES` to 2520?** Raise it
to 2520, in `jig/tests/client-protocol/src/fixtures.rs`, with one more line in the comment above
it in the form of its others: "A job spec's pointer adds eight bytes to the task owner." The
constant is not a user's budget (`AGENTS.md`, section 7 means test tiers' CPU budgets and
timeouts): it is the fixture's `load_bytes`, the exact measured cost of the single row, set
between that row and the activity row (2768) so the test reads the turn whole once. The
`Spec::job` field grows the stored owner, so the new value is a literal the field forces, within
this task's leave for `jig/tests/**`; the comment lines above it show the same rise each time the
owner grew. Leave `TURN_INPUT` as it is. Then run
`heavy cargo nextest run -p jig-client-protocol-world --test suite -E 'test(=turn_read::a_page_sized_turn_is_read_whole_once)'`
and finish the checks the first run left (the standalone drift run, the store's words tests, the
focused run across the changed packages).
