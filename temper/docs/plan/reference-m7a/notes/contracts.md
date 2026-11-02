# reference-m7a: the contracts

The binding shapes the plan's tasks build to (`inputs/request.md`): jobs in jig
(`jig/docs/design/domain/core.md, 3.2`; `domain/tasks.md, 5.3.1`; `domain/engine.md, 7.5`;
`domain/hosts.md, 6.9`; `client/domain/contract.md, 5.7`; `protocol/channel.md, 5.4`;
`protocol/store.md, 5`) and in temper's worker (`temper/docs/design/domain/worker.md, 4.4`).
Each task builds its half to this note; where it names a type, the name, the fields and their
types are binding, and the doc comments are the implementer's. Everything is pre-release:
every schema below changes **in place**, keeping its version number, its generated code and
one set of golden bytes regenerated (`jig/docs/design/protocol/README.md, 7`;
`protocol/store.md, 5`). No version number moves.

## 1. Kinds and bounds: `jig-domain-link`

```rust
/// A job's kind, a number its application gives (domain/hosts.md, 6.9).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct JobKind(pub u32);

/// The most a job of one kind may be (domain/engine.md, 7.5; hosts.md, 6.9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct JobBound {
    pub kind: JobKind,
    /// The most bytes its spec may hold.
    pub spec: u32,
    /// The most bytes its result may hold.
    pub result: u32,
    /// Its longest deadline.
    pub deadline: Duration,
}
```

- `Hello` gains `pub jobs: Box<[JobBound]>`: the kinds its job capability runs, ascending by
  kind, each once; empty for a host with none. The same `JobBound` is the application's
  declaration to the core (section 5) and a host's in its hello.
- A job hosted is listed in `Hello::hosting` as a run is (`Hosting { run, attempt, phase }`),
  its phase `Preparing`, `Starting`, `Active`, `Ending` or `Answered`, never `Waiting`.

## 2. The hub: `jig-domain-host`

In `boundary.rs`, beside the run's types:

```rust
/// What the core gives the host for one job (domain/hosts.md, 6.9).
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct JobAssignment {
    /// The engine's names for the job task and this attempt at it.
    pub run: Token,
    pub attempt: Token,
    pub kind: JobKind,
    /// The application's bytes, carried and never read.
    pub spec: Box<[u8]>,
    pub deadline: Duration,
    /// Nothing when the job has no workspace items.
    pub workspace: Option<Workspace>,
    /// The application's credentials its workspace needs: `Form::Application` only.
    pub grants: Box<[Grant]>,
}

/// How a hosted job ended, as the hub answers it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum JobEnding {
    Refused(JobRefusal),
    Finished { result: Box<[u8]>, work: Work },
    Failed { failure: JobFailure, detail: Box<[u8]>, work: Work },
}

/// Why a hosted job failed (domain/hosts.md, 6.9): what the engine acts on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum JobFailure {
    /// Its workspace could not be prepared.
    Unprepared(Preparation),
    /// It ended and said it failed.
    Failed,
    /// Its deadline passed: it was stopped and its processes killed.
    Deadline,
    /// Its result holds more bytes than its kind's bound.
    Result,
    /// Its processes could not be started, or ended without its capability's word.
    Unstarted,
    /// What it made was not delivered: `Stale`, `Refused` or `Failed`, as the workspace said.
    Undelivered(DeliveryOutcome),
    Cancelled(Reason),
}
```

```rust
/// Why a job's assignment was refused at the entrance; a run's `Refusal` is unchanged.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum JobRefusal {
    /// As a run's `Refusal::Busy`.
    Busy,
    Invalid(JobInvalid),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum JobInvalid {
    /// A kind this host does not run.
    Kind,
    /// More spec bytes than its kind's bound.
    Spec,
    /// A deadline longer than its kind's bound.
    Deadline,
    /// Its grants are not the application's, or more than the limits allow.
    Grants,
    /// The job capability refused its start; its words are for operators.
    Start { why: Box<[u8]> },
}
```

- A job's assignment is checked at the entrance against the `JobBound`s the root gave at
  `Opened`. `Refusal`, `Invalid` and every run type stay as they are.
- `link.rs`: `LinkDown::AssignJob { job: JobAssignment }` and
  `LinkUp::JobAnswer { run: Token, attempt: Token, answer: JobEnding }`. `LinkDown::Cancel` and
  `LinkDown::Acknowledge` serve jobs unchanged. A refused job's answer is kept by nothing, as a
  refused run's is; every other job answer is kept until acknowledged and sent again after a
  hello.
- The job capability, a sibling of the agent capability, by the hub's `owner` token alone:

```rust
/// host -> parent, inside `Request::Job(ToJob)`.
#[derive(PartialEq, Eq, Debug)]
pub enum ToJob {
    /// Run the job until it ends or its deadline passes, in `workspace` if it has one.
    Run { owner: Token, kind: JobKind, spec: Box<[u8]>, deadline: Duration, workspace: Option<Token> },
    /// Stop it at once and kill its processes. `Gone` follows.
    Stop { owner: Token },
}

/// parent -> host, inside `Event::Job(FromJob)`.
#[derive(PartialEq, Eq, Debug)]
pub enum FromJob {
    /// Its first process started.
    Started { owner: Token },
    /// The capability refused to start it, before anything ran; nothing of it is live.
    StartRefused { owner: Token, why: Box<[u8]> },
    /// How it ended. At most once per job; a stopped job may end with nothing said.
    Ended { owner: Token, end: JobEnd },
    /// Terminal for `Run`: every process of the job is gone. `detail` is for operators.
    Gone { owner: Token, detail: Box<[u8]> },
}

#[derive(PartialEq, Eq, Debug)]
pub enum JobEnd {
    /// It ended well, with its result and, for a kind that delivers, what to deliver: the
    /// application's bytes, which the hub hands the workspace as they are.
    Finished { result: Box<[u8]>, delivery: Option<Box<[u8]>> },
    /// It ended and said it failed; `detail` is for operators.
    Failed { detail: Box<[u8]> },
    /// Its deadline passed and it was killed.
    Deadline,
}
```

- `Request::DeliverJob { owner: Token, workspace: Token, delivery: Box<[u8]> }`: the hub asks the
  workspace to deliver what the job made; its terminal is `Event::Delivered { owner, delivery }`,
  a run's. `Delivered` and `Nothing` let the job finish; `Stale`, `Refused` and `Failed` fail it
  `Undelivered(outcome)`.
- `Event::Opened` gains `jobs: Box<[JobBound]>`, which the hub states in its hello. A root
  without a job capability gives none.
- The lifecycle (hosts.md, 6.9): admit (busy as a run's; invalid as above), prepare, `Run`,
  `Started`, `Ended`; on `Finished` with a delivery and a workspace, `DeliverJob`, then its
  terminal; `Gone`; release; answer. A cancel, a lost link, shutdown: `Stop`, no delivery not
  yet begun, wait for a delivery begun and for `Gone`, release, answer `Cancelled(reason)`. A
  result past its kind's bound fails `Result`, and nothing is delivered. A job saves nothing.
- **A root that composes no job capability** gives `jobs` empty, so the hub refuses every job
  as `JobInvalid::Kind` and never asks `Request::Job`; its router still routes the arm:
  `ToJob::Run { owner, .. }` is answered `FromJob::StartRefused { owner, why: b"no job capability" }`
  and `ToJob::Stop { owner }` with `FromJob::Gone { owner, detail: empty }`.

## 3. The worker wire: `jig-protocol-worker-wire`, version 9 in place

In `schema/worker-wire-v9.schema` (`protocol/channel.md, 5.4`):

- `Hello` gains `jobs`, a list of `JobBound { kind: u32, spec: u32, result: u32, deadline }`,
  the deadline in the schema's duration form as the hello's other durations are.
- A new record `Job` (down, a call): `task`, `attempt`, `kind: u32`, `spec` (bytes, bounded by
  the hub's limit for a spec), `deadline`, `workspace` (as `Assignment`'s), `grants` (as
  `Assignment`'s).
- A new record `JobAnswer` (up): `task`, `attempt`, `ending: JobEnding { refused: JobRefusal,
  finished: { result: bytes, work }, failed: { failure: JobFailure, detail, work } }`, with
  `JobFailure`'s and `JobRefusal`'s (`busy`, `invalid: JobInvalid { kind, spec, deadline,
  grants, start }`) arms section 2's in order, reusing `Work`, `Preparation`, the delivery
  outcome and the cancel reason as the run's records spell them. Existing records are
  unchanged but for the hello's `jobs`.
- New kind numbers for `Job` and `JobAnswer` in `src/kinds.rs`, after the last.

## 4. The tasks child and authority: `jig-domain-tasks`, `jig-domain-authority`

- `Executor::Job { kind: u32 }` (still `Copy`), after `Person`.
- `Spec` gains `pub job: Option<Box<JobSpec>>`, present exactly when the executor is `Job`:

```rust
/// A job task's spec (domain/tasks.md, 5.3.1): the application's bytes, never words.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct JobSpec {
    pub bytes: Box<[u8]>,
    /// How long one attempt may run.
    pub deadline: Duration,
}
```

- `Contract::Job { bytes: u32 }`: the most bytes its result may hold, the kind's declared
  result bound. `TaskResult::Job { bytes: Box<[u8]> }`; `ResultKind` gains `Job`.
- `jig_domain_authority::Executor` gains `Job(u32)`, and the tasks child's mirror
  `AuthorityExecutor` gains `Job(u32)`; a delegation names a job kind as it names a charter.
- A job task takes no messages: words, questions and amendments to it are refused with the
  existing refusals' executor arm (`Refusal::Executor` where one exists); a cancel ends it at
  once. Its claim reserves nothing (its budget may be zero). It is never a conversation.
- Its failures' classes (tasks.md, 5.5), from section 2's `JobFailure`: `Unprepared(Transient)`
  transient; `Unprepared(Permanent)` and `Undelivered(Stale | Refused)` permanent;
  `Failed` and `Deadline` the run's own; `Unstarted` the agent's; `Result` invalid;
  `Undelivered(Failed)` transient; `Cancelled(Contact | Shutdown)` lost; `Cancelled(Engine)` a
  cancel's end.
- **A job's answer reaches the tasks child as an existing `End`,** no new variant:
  `JobEnding::Finished { result, .. }` is `End::Finished { result: TaskResult::Job { bytes:
  result }, cancel_delegates: false }`; `Failed { failure, .. }` is `End::Failed(class)` by the
  classes above; `Refused(JobRefusal::Busy)` is `End::Refused`; `Refused(JobRefusal::Invalid(_))`
  is `End::Failed` of the permanent class, as an assignment its host found invalid. The
  function that maps one to the other is `jig-domain-hub-link`'s, beside the run's
  `result_event` (`fleet-jobs`).

## 5. The core: `jig-domain-core`

- `Config` gains `pub jobs: Box<[jig_domain_link::JobBound]>`: the application's kinds,
  ascending, each once (engine.md, 7.5). temper's engine gives none until a later plan adds
  its first kind; jig's test kits give theirs.
- A procedure's step makes a job as a `Delegate` whose executor is `Job { kind }`, with its
  `JobSpec`, its holdings as any task's, no LLM choice, and the contract `Job { bytes }` the
  core sets from the kind's bound. The batch is refused when the kind is not declared, the
  spec or deadline is past its kind's bound, or the creator's delegation lacks
  `Executor::Job(kind)` (propose/refuse as authority.md, 8.1 has it).
- The claim of a due job task gathers its workspace items and the application's credentials
  from its connectors as a run's are gathered, then goes to the fleet with its kind and
  bounds, never through `run_admission`'s LLM, account or charter checks.

## 6. Stored records: `jig-protocol-store`, in place

- `tasks-v9.schema`: `Executor` gains `job: JobExecutor { kind: u32 }`; `Spec` gains an optional
  `job: JobSpec { bytes, deadline }`; `Contract` gains `job: JobContract { bytes: u32 }`;
  `TaskResult` gains `job: JobResult { bytes }`; `ResultKind` gains `job`; `AuthorityExecutor`
  gains `job: u32`. Each new arm appended after the last.
- `people-v14.schema` and `settings-v1.schema`: their delegation `Executor` gains `job: u32`,
  appended. `TaskExecutor`, `TemplateExecutor` and the people and settings `Contract`s gain
  nothing: no party and no template makes a job.
- Golden bytes for each new arm (`_full`/`_smallest`, as `no_change`'s), `layouts.rs`
  regenerated, translations exhaustive both ways.

## 7. The client: `jig-client-protocol-wire` v19 in place, `jig-client-domain`

- Wire (`client-v19*.schema`): `Executor` gains `job: { kind: u32 }`; `ExecutorKind` gains `job`;
  `ChipExecutor` gains `job: { kind: u32 }`; `TaskContractDocument` gains `job: { bytes: u32 }`;
  `AuthorityExecutor` (a delegation's kinds) gains `job: u32`; each appended, with the
  record wrappers the schema's idiom gives (`ExecutorJob`, `ChipExecutorJob`,
  `TaskContractDocumentJob`).
- Domain (`contract/part.rs`, `queries.rs`, `common.rs`, `proposed_task.rs`):
  `Executor::Job { kind: u32 }`, `ExecutorKind::Job`, `ChipExecutor::Job { kind: u32 }`,
  `TaskContractDocument::Job { bytes: u32 }`, and `contract/authority.rs`'s
  `AuthorityExecutor` gains `Job(u32)`. `Create` naming `Executor::Job` is refused
  `Refusal::NotOffered`, on the client (`admission.rs`) and on the engine (`create.rs`). A job
  task's spec and result show as its application's part (contract.md, 5.3 and 5.7): a
  document carries no job bytes.
- Text a client shows for the executor: `job <kind>` (jig's view and temper's ctl alike).

## 8. temper's job capability: `temper-worker-domain-tooling`

A new step crate under the strict subset, depending on `skein-lib` and `jig-domain-link` (for
`JobKind`, `JobBound`) only: it links no protocol crate, so it takes its spec decoded. Its root-facing boundary, which `temper-worker-domain`'s root
translates to and from section 2's `ToJob`/`FromJob`:

```rust
/// One kind the capability runs (temper/docs/design/domain/worker.md, 4.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Kind {
    pub bound: jig_domain_link::JobBound,
    /// The command's second argument (temper/docs/design/protocol/tooling.md, 2).
    pub verb: Box<[u8]>,
    /// Whether a job of it that exits 0 delivers its writable repository's head.
    pub delivers: bool,
}

pub struct Domain { /* jobs in flight, by owner */ }
impl Domain {
    /// Refuses kinds not ascending and distinct, or past the limits.
    pub fn new(limits: &Limits, kinds: Box<[Kind]>) -> Result<Domain, Invalid>;
    /// The kinds' bounds, for the hello.
    pub fn bounds(&self) -> Box<[jig_domain_link::JobBound]>;
}
/// As `temper-worker-domain-checkout`'s `step`.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>);

/// parent -> capability
pub enum Event {
    Run { owner: Token, kind: JobKind, spec: Spec, deadline: Duration, directory: Box<[u8]> },
    Stop { owner: Token },
    /// Terminal for `Write`.
    Written { owner: Token, written: bool },
    /// The command's process started.
    Spawned { owner: Token },
    /// The command ended: its exit, and the end of its error output, bounded by the limits.
    Exited { owner: Token, exit: Exit, tail: Box<[u8]> },
    /// Terminal for `Read`: `None` when the file is missing or past `most`.
    Read { owner: Token, bytes: Option<Box<[u8]>> },
    /// Terminal for `Spawn`: every process of the job is gone.
    Gone { owner: Token },
}

/// A job's spec, as the service decodes it from temper's `JobSpec` record.
pub struct Spec {
    /// The command's path in the project's home, relative.
    pub command: Box<[u8]>,
    /// The request file's bytes.
    pub request: Box<[u8]>,
}

pub enum Exit {
    Code(u8),
    /// Ended by a signal, `Kill`'s included.
    Signal,
    /// Its deadline came first: the protocol layer killed its process group.
    Deadline,
}

/// capability -> parent
pub enum Request {
    Write { owner: Token, path: Box<[u8]>, bytes: Box<[u8]> },
    /// The protocol layer kills the process group at `deadline` and answers `Exited` with
    /// `Exit::Deadline`, as the checkout's `Io` races its deadline.
    Spawn { owner: Token, program: Box<[u8]>, args: Box<[Box<[u8]>]>, env: Box<[(Box<[u8]>, Box<[u8]>)]>, dir: Box<[u8]>, deadline: Time },
    /// Kill the job's process group now.
    Kill { owner: Token },
    Read { owner: Token, path: Box<[u8]>, most: u32 },
    /// Answers to the root, which it translates to section 2's `FromJob`.
    Started { owner: Token },
    StartRefused { owner: Token, why: Box<[u8]> },
    Ended { owner: Token, end: End },
    Gone { owner: Token, detail: Box<[u8]> },
}

pub enum End {
    Finished { result: Box<[u8]>, deliver: bool },
    Failed { detail: Box<[u8]> },
    Deadline,
}
```

- A job's steps: `Run` (its kind among the entries, its request and deadline within the
  kind's bound and the limits, its command a relative path with no `..`, else
  `StartRefused`); `Write` the request file;
  `Spawn`; `Started`; on `Exited`, `Read` the result file for exits 0 and 3; `Ended`; on `Gone`,
  `Gone`. `Stop` before `Spawn` ends it with `Gone` alone; after, `Kill`, then `Gone` once the
  protocol layer's `Gone` comes. Each owner gets exactly one `Gone`, last.

- The command line (tooling.md, 2 and 3): program `<directory>/home/<command>`, args
  `[<program>, <verb>, <directory>/request.json]`, directory `<directory>`, environment exactly
  `PATH` (the limits'), `HOME` and `TEMPER_JOB` (the directory) and `LANG=C.UTF-8`.
- **The spec** is temper's payload record `JobSpec { command, request }` in
  `temper-protocol-worker-wire` (payload version 4, in place), beside `Items`; the service
  decodes it from the `Job` record's spec bytes as it decodes the items, and gives the root
  `Spec` with the assignment. **The result** is the result file's bytes as the command wrote
  them, no record of temper's around them. Exit 0 and 3 finish the job (`deliver` only on 0
  and for a kind that delivers); any other exit, or a result file missing or past its bound,
  fails it, with the error output's tail as detail.
- The deadline is the job's wall time, given to the protocol layer with the `Spawn` (the
  hub arms no timers): `Exit::Deadline` becomes `End::Deadline`.
- **Delivery** (worker.md, 4.4): `End::Finished { deliver: true }` becomes `JobEnd::Finished`
  with a delivery naming the writable repository; the root hands it to the checkout's existing
  fast-forward push (`Write::Push`) of that repository's head.
