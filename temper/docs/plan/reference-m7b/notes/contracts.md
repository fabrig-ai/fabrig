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

## C3.1. The steps' facts (added with the steps, batch 3; compatible with C3)

`Facts` gains two more fields, which `change-merge-steps` adds and `change-merge-route` fills:

```rust
    /// The change's current delegate, as the forge's row keeps it (`ChangeRow::delegate`);
    /// `None` when it has none.
    pub delegate: Option<Delegate>,
    /// A catch-up's commit (`ChangeRow::catch_up`), `None` for every other change.
    pub catch_up: Option<Commit>,
```

- A merge job runs while the change is `Resolving { requested: true, base }` (a conflict) or
  `Producing { requested: true }` (a catch-up), and `facts.delegate` says it is
  `Delegate::Merge { base, from }`. On its passed result, `facts.branch` (read afresh, as every
  fact a decision depends on: `temper/docs/design/domain/forge.md, 5.3`) other than `from`
  means the merge was pushed; equal to `from` means the command left conflicts.
- The resolution after a merge job is `Delegate::Resolve { base }` with the same base, made in
  the step that reads the job's result, adding nothing to `Change::resolutions`; a resolution
  asked again after a release is a resolution, never a merge job.
- A catch-up's merge job is its producer and counts no resolution; the resolution after it,
  where the job left conflicts, counts one. Each conflict is counted once either way, as
  `change-merge-steps` has it.
- A catch-up's resolution runs while the change stays `Producing { requested: true }`,
  `facts.delegate` being `Resolve { base }`: its passed result with a branch read goes to
  `Opening`, as a producer's push does, so its pull request is opened. (`delegated`'s path,
  `pushed` to `Checking`, is a conflict's alone.)

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

## R1. Revision 1: the merge job's repository keeps its name (supersedes C2's `home`)

C2 named the merge job's one item `home`, so that the job capability, which runs
`<directory>/home/<command>`, would find the merge command in it. An item's name is also its
remote's (`temper-service/src/translate.rs` builds the remote from the owner and the name), so
an item named `home` fetches the wrong repository. Revision 1, binding from batch 4 on
(`temper/docs/design/domain/worker.md, 4.4`; `temper/docs/design/protocol/tooling.md`, 2 and
4.1):

- the merge job's item keeps the repository's own name, as every other item does;
- the job capability (`temper-worker-domain-tooling`) resolves a spec's `command` against the
  job's directory, `<directory>/<command>`, no longer under `home/`; the command stays relative,
  without `..`, NUL or a leading `/`;
- the merge job's spec's `command` is the repository's name, `/`, then the project's
  `merge_job` path (`fabrig/.temper/merge`); a future prepare or key job's is `home/` then its
  path;
- the merge command finds its repository from its own path, never by the name `home`.

`change-merge-surface` and `change-merge-route` build to C2 as released; `merge-r1-paths` moves
their item, the root's command and the capability to R1, with the stories that pin them.

## R2. Revision 2: what the trunk holds the merge job to (supersedes C1's bounds)

Binding from batch 5 on; each item names the task it concerns.

- **C1's bounds** (`merge-kind`): `MERGE_SPEC_BYTES = 4_096` and `MERGE_RESULT_BYTES = 512`.
  jig's core asserts every declared kind's spec and result within `tasks.spec_bytes` and
  `tasks.result_bytes` (`jig/crates/jig-domain-core/src/lib.rs`, `Core::new`), 4,096 and 512
  in the small profile temper's engine takes, so C1's 8 KiB and 64 KiB stop the engine at its
  start. The worker declares the same. The deadline stays 10 minutes. The merge command keeps
  its result within 512 bytes (`temper/docs/design/protocol/tooling.md, 4.1`: the paths that
  fit, then `"more": N`).
- **A job task takes no words, parameters or inputs** (`jig/crates/jig-domain-tasks/src/batch.rs`
  refuses them with `Refusal::Spec`; `change-merge-route`): the `Merge` arm of
  `change_delegate` builds its own delegate with none of the parameters `change_delegate`
  pushes for an agent, `llm: None`; the core sets the job's contract from the declared bound;
  the job's item comes from its parent change's row, as a run's does.
- **The engine world's worker declares kind 3** (`change-merge-route`): its hello
  (`temper/tests/engine/domain/src/protocol.rs`, `jobs`) holds kind 3's bound, or the fleet
  never places the job; the world's core declares it too.
- **`chat_change`'s delegation kinds** (`change-merge-route`, C5) are a list of capacity 3 today,
  each push expected: its capacity becomes 4 for `Job(3)`.
- **Kind 3's declared bound in the root** (`change-merge-route`): the core keeps its `jobs`
  private once it takes `config.core`; the root copies kind 3's bound from `config.core.jobs`
  into its own `RootConfig` before (`temper/crates/temper-engine-domain/src/domain.rs`, where
  the core is built, and its restore path), a reader edit that task may make. The domain uses
  that bound's `result` wherever a spec names `MERGE_RESULT_BYTES` (temper-service's constant is
  not the domain's to name).
- **After R1** (`merge-r1-paths`, `catch-up-call`): `merge-r1-paths` also updates the engine
  world's story that decodes C2's command and the forge's item test named for `home`, and runs
  `heavy cargo nextest run -p temper-engine-domain-world -E 'test(/merge_job/)'`;
  `catch-up-call`'s stories pin neither the merge job's item's name nor its command's form.
- **The merge command's result within 512 bytes** (`merge-r1-paths`, after `merge-reference`):
  where `tools/merge/merge.py` writes `settled` and `left` uncut, `merge-r1-paths` makes it list
  the paths that fit, in index order, then `"more": N` (`tooling.md, 4.1`), with a test of a
  merge leaving more paths than fit.

