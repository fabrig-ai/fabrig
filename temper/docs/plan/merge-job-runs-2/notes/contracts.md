# merge-job-runs-2: contracts

The shapes that cross tasks. Binding for the specs that cite them.

## 1. The merge kind, as production declares it (`story`)

In `temper/crates/temper-service/src/limits.rs`, beside `merge_bound`:

```rust
/// The merge job's kind and bounds (temper/docs/design/domain/worker.md, 4.4).
pub fn merge_bound() -> JobBound;                       // was pub(crate); unchanged body
/// Declare merge only where the tasks child holds its bounds (worker.md, 4.4).
pub fn merge_jobs(jig: &jig_protocol_config::Limits) -> Box<[JobBound]>; // was pub(crate)
/// The worker root's job kinds: the merge kind alone (worker.md, 4.4).
#[cfg(feature = "worker")]
pub fn job_kinds() -> Box<[temper_worker_domain::tooling::Kind]>;
```

`worker.rs`'s `worker_root` passes `crate::limits::job_kinds()` to
`root::Domain::new`; its kinds are the same as today's (kind 3, verb
`merge`, delivering, `merge_bound()`).

## 2. A job's assignment, translated as production does (`story`)

In `temper/crates/temper-service/src/assignment.rs`, beside `down`:

```rust
/// A job's assignment and its forge workspace as the worker link carries them
/// (temper/docs/design/domain/worker.md, 4.4).
pub fn job_down(
    job: jig_domain_host::JobAssignment,
    workspace: Option<&temper_engine_domain::ForgeWorkspace>,
    forges: &crate::forges::Forges,
    encoding: &Encoding,
    values: &crate::values::Values,
    channel: Token,
    now: Time,
) -> Option<kit::Event>;
```

`route::job_assignment` keeps its signature and calls `job_down` with the
engine's forges, encoding and values; what it returns is unchanged
(`tests/workers.rs`' `a_jobs_forge_workspace_becomes_worker_items` and
`a_job_without_a_forge_workspace_sends_no_part` hold it).
`worker_route::job_items` and `worker_route::job_spec` become `pub`, their
signatures unchanged.

## 3. The composed world's merge job (`story`, then `story-rows`)

In `temper/tests/system/src/composed/merge_job.rs`:

```rust
/// One row of the merge job's story (temper/docs/design/domain/testing.md, 2.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Settled,
    Left,
    // story-rows adds: Clean, OtherBranch, PastTheBound
}

/// What a row's run showed, read from the store, the fake forge and the fake disk.
#[derive(Debug)]
pub struct MergeJobReport {
    /// The job task's ending, as the tasks child keeps it.
    pub job: jig_domain_tasks::End,
    /// The failures the job's task counted, by class (all zero on a run that finished first time).
    pub failures: u32,
    /// The change's branch on the fake forge after the job: its head and that commit's parents.
    pub branch: u64,
    pub parents: Vec<u64>,
    /// The program, arguments and request the worker's spawn named, and whether the program's file was on the fake disk.
    pub program: Vec<u8>,
    pub args: Vec<Vec<u8>>,
    pub request: Vec<u8>,
    pub command_found: bool,
    /// A resolution made after the job, with its words.
    pub resolution: Option<Vec<u8>>,
}

impl World {
    /// A world whose project names `.temper/merge` as its `merge_job`, both
    /// roots declaring the merge kind as production does, set up for `row`.
    pub fn merge_job(seed: u64, row: Row) -> World;
    /// Runs until the job's task has ended and the change has taken its next step.
    pub fn run_merge_job(&mut self) -> MergeJobReport;
}
```

The test is `temper/tests/system/tests/suite/merge_job.rs`,
`a_changes_merge_job_runs_on_its_worker`, one table of `(Row, expected)`
rows; `story-rows` adds its rows to that table and to `Row`, never a
second test. Field names may grow; none is removed or renamed by
`story-rows`.

## 4. The fake git layer within the checkout's bound (`conflicts`)

In `temper/tests/worker/checkout/src/translate.rs`, beside `perform`:

```rust
/// `perform`, with the git layer's rules on the checkout's `conflicts`
/// (temper/docs/design/protocol/git.md, 4.6 and 4.7).
pub fn perform_within(forge: &mut impl Remote, disk: &mut Checkout, op: Op, conflicts: u32) -> Done;
```

A `Merge` whose conflicts number more than `conflicts` ends
`Done::Conflicted` with the first `conflicts` of them, in the fake's
order, the merge's state and every marked file left on the fake disk.
`perform` keeps its signature and today's behaviour, so worlds that do
not pass a bound are unchanged. `story-rows` switches the composed
world's call to `perform_within` with its worker's
`limits.checkout.conflicts`.

## 5. The failure lines (`lines`)

The forms are `temper/docs/design/domain/worker.md`, 4.4's:

- worker: `<binary> worker: run RUN attempt ATTEMPT job KIND VERB failed CLASS: WHY`
  and `<binary> worker: run RUN attempt ATTEMPT job KIND VERB refused REASON: WHY`;
- engine: `<binary> serve: task TASK run RUN attempt ATTEMPT failed CLASS: job KIND: DETAIL`
  and `<binary> serve: task TASK run RUN attempt ATTEMPT refused REASON: job KIND: WHY`.

CLASS is the `Debug` form of `jig_domain_host::JobFailure`
(`Unprepared(Transient)`, `Undelivered(Failed)`, ...), REASON that of
`jig_domain_host::JobRefusal`'s variant as `run_refusal_line` names a
run's (`temper/crates/temper-engine-shell/src/serve/host.rs`). WHY and
DETAIL are escaped by `run_detail_line`'s rule, their last 4,096 bytes.
Nothing else in the plan reads these notes; their Rust names are
`lines`' own.
