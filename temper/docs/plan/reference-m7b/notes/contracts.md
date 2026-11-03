# reference-m7b: the contracts

The binding shapes the plan's tasks build to (`inputs/request.md`): the merge job
(`temper/docs/design/domain/worker.md, 4.4`; `temper/docs/design/protocol/tooling.md, 4.1`), its
uses in the change procedure (`temper/docs/design/domain/forge.md, 8.1`, `8.2`, `8.4`, section
10), the project's `merge_job` (`temper/docs/design/protocol/configuration.md, 3`) and `change`'s
`merge` (`temper/docs/design/domain/agent.md, 4.3.1`). Where a section names a type, its name,
fields and types are binding; doc comments and private helpers are the implementer's.

## C1. The merge kind: number 3, verb `merge`, delivering

In `temper/crates/temper-service/src/limits.rs`, beside the `TOOLING_*` constants, the small
profile's constants and one function both sides use:

```rust
/// The merge job's kind and bounds (temper/docs/design/domain/worker.md, 4.4).
pub(crate) const MERGE_KIND: u32 = 3;
pub(crate) const MERGE_SPEC_BYTES: u32 = 8_192;
pub(crate) const MERGE_RESULT_BYTES: u32 = 65_536;
pub(crate) const MERGE_DEADLINE: Duration = Duration::from_secs(600);

pub(crate) fn merge_bound() -> JobBound; // JobBound { kind: JobKind(MERGE_KIND), spec, result, deadline }
```

- The worker's root is given one kind entry, `tooling::Kind { bound: merge_bound(), verb:
  b"merge", delivers: true }` (`temper-service/src/worker.rs`, where it gives `Box::new([])`).
- The engine declares the same bound to jig's core: `config.core.jobs` holds `[merge_bound()]`
  wherever the service builds the core's configuration (`temper-service/src/system.rs`, and
  `startup/checks.rs` where it builds one too).
- The forge's crate names the number for the engine's domain:
  `temper_engine_domain_forge::MERGE_KIND: u32 = 3` (public, in its `lib.rs` or `boundary.rs`).
  The engine's root takes the kind's deadline and result bound from the core's declared jobs
  (`jig_domain_core::Config::jobs`); a deployment whose core declares no kind 3 makes no merge
  job (C5).

## C2. The merge job's spec and request

- The job's spec bytes are the worker wire's `JobSpec { command, request }` record
  (`temper/crates/temper-protocol-worker-wire`, payload version as it is), encoded by the
  engine's root as it makes the job: `command` the project's `merge_job` path, `request` the
  request file's bytes. `temper-engine-domain` gains `temper-protocol-worker-wire` as a
  dependency for this alone (`worker.md, 4.4`, "The spec is written by the engine's root").
- `request` is exactly `{"version":1,"head":"<HEAD>","base":"<BASE>"}`, no spaces, each commit
  in lowercase hexadecimal, 40 digits for a SHA-1 commit, 64 for a SHA-256 one, as
  `forge_route.rs`'s `commit_text` spells a commit for a resolution's parameter 6.
- The job's `tasks::JobSpec { bytes, deadline }`: `deadline` the declared kind's deadline. Its
  executor `tasks::Executor::Job { kind: 3 }`, no LLM choice; its workspace one item: the
  change's repository, writable, start `Start::Merge { branch, base }`, its push the change's
  branch, its holder the change, its **name `home`** (the capability runs `home/<command>`).

## C3. The forge-change domain: `temper-engine-domain-forge-change`

```rust
pub enum Delegate {
    Produce,
    Repair(Repair),
    Resolve { base: Commit },
    Gate { number: u64, head: Commit },
    /// The merge job (forge.md, 8.4): `base` merged into the branch's head; `from`, the
    /// branch's head when the job was made, none for a catch-up's (its branch not made yet).
    Merge { base: Commit, from: Option<Commit> },
}
```

- `Delegate` stays `Copy`. No new `State`: the job runs while the change is `Resolving {
  requested, base }` (a conflict) or `Producing { requested }` (a catch-up), the delegate saying
  which.
- `Facts` gains `merge_job: bool`: the project names a `merge_job`, its core declares kind 3,
  and the change's authority delegates to `Job(3)` (C5); and `contains_catch_up: Status`: for a
  catch-up, whether its landing branch's tip contains the catch-up's commit (`Passed`), does
  not (`Failed`), or is not known yet (`Pending`); `Pending` for every other change.
- The change's own record gains `catch_up: Option<Commit>` (the commit a catch-up merges, from
  parameter 16; C6) on the forge's `ChangeRow`, and `merge_result: Option<Box<[u8]>>`, the last
  merge job's result bytes, kept until the resolution after it is made, at most
  `MERGE_RESULT_BYTES` (64 KiB).
- **Counting:** making a merge job adds one to `Change::resolutions` (and holds past the bound
  as `resolving_due` does); the resolution made after a merge job that left conflicts adds none.

## C4. Stored records: `temper/crates/temper-protocol-store`, `store-v11.schema`, in place

- `record MergeDelegate { base: Commit from: option fixed 32 }`;
  `enum Delegate { produce, repair: RepairValue, resolve: Commit, gate: DelegateGate, merge:
  MergeDelegate }`, the arm appended.
- `ChangeRow` gains, appended after `report`: `catch_up: option fixed 32` and `merge_result:
  option bytes 65536`.
- Regenerated with `SKEIN_CODEGEN_WRITE=1` on the store's drift test; the store's version stays.

## C5. The project's `merge_job`, and the change's authority

- `application.landing.projects[]` gains `merge_job`, a JSON string: a relative path of 1 to
  4,096 bytes, no `..` segment, no leading `/`, no NUL; absent is none. One past those bounds
  refuses the start at `application.landing.projects[i].merge_job` (`What::Form` or the parser's
  kind for a malformed member, `What::Bound` for the length).
- `temper_engine_domain::translate::ProjectLanding` gains `pub merge_job: Option<Box<[u8]>>`;
  `translate::project_landing` hands it to the forge after `Domain::project_landing`, by a
  setter of its own, so `project_landing`'s callers are unchanged:
  `temper_engine_domain_forge::Domain::project_merge_job(&mut self, project: u32, merge_job:
  Box<[u8]>) -> bool` (false where the project's landing is not configured or the forge's
  bound on projects is reached, as `project_landing`'s is), and
  `Domain::merge_job(&self, project: u32) -> Option<&[u8]>`.
- **The change's authority** (`forge_route.rs`, `chat_change`): its delegation kinds gain
  `AuthorityExecutor::Job(3)` exactly when the run's own authority's delegation holds `Job(3)`;
  nothing else changes there. `Facts::merge_job` is true when the change's project names a
  `merge_job`, the core declares kind 3, and the change task's authority holds `Job(3)`.

## C6. `change`'s `merge`, parameter 16

- The tool's plan schema (`temper-engine-smith/src/calls.rs`, `PLAN_CHANGE`) gains `merge`, a
  string, beside `landing`; the plain schema does not. `engine::ChangeCall` gains `pub merge:
  Option<[u8; 32]>`, the forge-change `Commit` (a SHA-1 commit padded as the forge pads one,
  as `commit_text` reads it back); `ChangeCall::words` gives `None`.
- The procedure's spec carries it as `Parameter::Bytes { name: 16, value: <commit text, C2's
  spelling> }`; `start_change` reads parameter 16 into `ChangeRow.catch_up`, refusing one that
  is not 40 or 64 lowercase hexadecimal digits as it refuses a malformed parameter today.
- Refusals at the call, each naming what it lacks, as the root refuses a call's other members:
  `merge` from a run whose authority lacks jig's control family (the plan members' rule); `merge`
  whose project names no `merge_job` ("the project names no merge_job"); `merge` from a run whose
  authority does not delegate to `Job(3)` ("its authority does not delegate to the merge job");
  a `merge` that is not 40 or 64 hexadecimal digits, naming it.
