# The worker's hosted runs

This document holds the worker's hosted runs, split from [`README.md`](README.md).

## 1. Hosted runs

### 1.1 The assignment

What the engine gives the worker for one run is jig's assignment (jig's
`domain/hosts/README.md`, section 2; jig's `domain/engine/executors.md`, 1), with
temper's workspace items beside it:

- **Run and attempt.** The engine's names for the run (its task's number
  and the run's) and for this attempt at it; every message about the run
  carries both.
- **Workspace.** A workstream, which names the checkout so later runs of
  the same work find it cached: the number of the task that holds what is
  written (jig's `domain/engine/executors.md`, 1), by which the checkout keys its
  cache. And the repositories, each with:
  - its deployment tag, by which what the run left names it;
  - its name, the directory it sits in and what the agent calls it, one
    safe path component;
  - its remote: the URL git fetches from and pushes to, which the
    engine's protocol layer makes from its forge's configuration
    (`../../protocol/channel.md`, section 4), so a worker needs no forge
    configuration;
  - its git, where its forge has its own: the program every git
    operation on it runs, a fake forge's, which the domain carries to
    each operation and never reads; otherwise the worker's own git
    (`../../protocol/git.md`, section 3);
  - where to start: a base branch, a branch, a commit by its full object
    id, saved work, **a merge in progress**: a branch, and the commit
    of its base to merge into it, or **a base with heads merged in**: a
    base branch, and the commits to merge into its tip, one by one in
    their order, each by its full object id (a change's early start:
    `../forge/change/questions.md`, 1) (`checkouts.md`);
  - whether it may be written, and if so the branch a change is pushed
    to, and the head that branch must still be at, if the assignment
    names one (`checkouts.md`);
  - a landing branch, the branch its change lands on, for each writable
    repository of a change's producer, whatever its start, and of a goal's
    later run; no other item names one. Prepare pins its tip as
    `refs/temper/landing` for the project's checks (`checkouts.md`); a writable
    item that names none (a repair, a resolution, a merge job, a chat's
    run) has the head its delivery replaces pinned there instead
    (`checkouts.md`, *The replaced head's pin*), unless it names a pin
    (below). For an
    inherited producer that starts on its push branch it is also the
    comparison for existing work, separate from the checkout start and the
    push's expected head; and on a merge in progress it marks a producer's
    start, as a goal's later run's does (`checkouts.md`);
  - a pin, for a writable repository that names no landing branch: a
    commit by its full object id, which prepare points
    `refs/temper/landing` at in place of the head its delivery replaces.
    The engine names one on the item of a repair asked by a landing alone,
    the base that landing used (`checkouts.md`, *A named pin*); an item
    naming both a pin and a landing branch is refused at the entrance as
    an invalid start that names it;
  - the task's own delivery, for a writable repository: the head of this
    task's last checked push to its push branch, as the engine recorded
    it (`../forge/change/questions.md`, 1, *A checked head*), or none; a
    new hold starting on it holds the task's delivered work, so a
    delivery with nothing new pushes it again (`checkouts.md`, *A task's
    own delivery outlives its hold*);
  - the identity for every git operation on it: one of temper's
    application credentials (jig's `domain/engine/README.md`, 6.1), granted
    with the assignment, which the protocol layer maps to a token and a
    commit author; and on GitHub the account whose entry of that
    identity's value authenticates git on it (`../../protocol/channel.md`,
    sections 4 and 6), carried with the identity into each git operation
    that reaches the forge, which the domain copies and never reads.
- **Saving.** The assignment's typed resume policy requests saving
  unfinished work from the first attempt, before a transcript exists
  (a chat's, a coding agent's: ../agent/charter.md). Each writable
  repository has its own optional saved-work destination, belonging to
  the running task (../forge/branches.md), and the next
  attempt starts from the saved work: the tree follows the transcript,
  so a retry that remembers a write finds the file it wrote, and a run
  that starts fresh from its brief starts from a clean checkout. A run
  that ended with its change landed saves nothing, and one holding a merge
  in progress saves it as a merge commit only once its conflicts are resolved
  (step 7).
- **Charter.** smith's, as the engine wrote it (../agent/charter.md),
  carried as bytes. The worker gives it to the agent with the workspace
  it prepared: where the repositories sit, which may be written, and for
  a merge in progress the files in conflict (smith's `domain/run.md`,
  3.2).
- **Transcript,** for a run that resumes: its current window, and the
  calls committed after its last turn (jig's `domain/engine/executors.md`, 2;
  jig's `domain/hosts/hub.md`, 4). Across the link, those calls' answers
  fit its room: the newest whole, each older host answer cut to words
  saying the call ran and how many bytes its answer had (jig's
  `protocol/channel/README.md`, 8, *A resumed start fits its room first*), so a
  producer whose program worked an hour since its last turn resumes,
  told of every call it made. A delivery among those calls comes with
  its evidence, smith's record of the push's terminal, which must decode
  and agree with the outcome the engine committed (1.3;
  `../../protocol/channel.md`, section 2). The worker carries the rest
  opaque, within its limits.
- **Grants.** The credentials the run's LLM accounts and git identities
  need, as names the protocol layer fills (`../../protocol/credentials.md`,
  sections 3 and 4). The hub hands the agent its LLM accounts' grants
  only; the worker keeps the git identities' for its own git operations.

Its sizes and names are checked at the entrance against the worker's
limits, and so is a repository named twice: what is beyond them is
refused as an invalid start that names it (jig's `domain/hosts/hub.md`, 3).
A run with no repository needs no workspace: the worker prepares nothing
and starts smith with none (jig's `domain/hosts/README.md`, section 6). A chat
that names no repository runs so.

### 1.2 Lifecycle

A hosted run's lifecycle is the hub's, in nine steps (jig's
`domain/hosts/hub.md`, 2). What the worker puts in them:

- **Step 2, prepare:** the checkout prepares the workspace from the items
  (`checkouts.md`), a few preparations at a time, runs' and jobs' alike:
  one queued for its turn holds its run's slot and nothing in flight.
- **Step 3, start:** smith's agent process (`agent-processes.md`, section 1), with the charter
  and the transcript decoded by hand-out, and the workspace as the
  checkout prepared it.
- **Step 4, relay:** smith's `deliver` is a push, which the checkout
  serves, one at a time (`checkouts.md`; ../agent/README.md, section 5); every host tool
  goes to the engine, and its answer comes back.
- **Step 6, stop:** a push in flight goes on to its end, and what it
  lands counts (`checkouts.md`).
- **Step 7, save,** if the assignment asks, at park, at an unfinished end
  and at cancel: commit what the writable repositories hold and push it
  to each repository's own saved-work destination, where one is named,
  so a later run on any worker starts from it.
  A run that ended with a landed change has nothing left to save, and
  nor has one whose agent never started. A run that started from a merge
  in progress saves it as a merge commit once its conflicts are resolved;
  unresolved conflict markers refuse the save. One that delivered mid-run,
  then parked, failed or was cancelled saves its completed work, so its
  next attempt retains that delivery's history.
- **Step 8, release** the workspace to the cache (`checkouts.md`).
- **Step 9, answer:** jig's answer, with temper's parts beside it: what
  the run left on the forge (the last commit landed in each repository,
  the head a pull request is to show) and how its save went, each
  repository named by its tag (`../../protocol/channel.md`, section 5).

A push that lands while its run winds down is the run's result (smith's
`domain/run.md`, 8.2). The hub arms no timers: each wait is bounded
below it, by the deadlines of git operations, the agent's watchdog and
wall time, and the grace of cancel, then kill.

### 1.3 Failures

jig's types (jig's `domain/hosts/hub.md`, 3), with temper's detail:

- **Refused** at the entrance: busy, or invalid, saying what is beyond
  the limits or which item the workspace cannot take. Once prepared, a
  start smith refuses, or whose charter, transcript, call names or
  delivery evidence do not decode or agree, is refused as invalid too,
  which the engine counts as permanent (jig's `domain/tasks/lifecycle.md`, 6). A
  start smith refuses as busy, an agent or a run it has no room for, is
  refused busy, as at the entrance (jig's `domain/hosts/charters.md`).
- **A start the link cannot carry** never reaches the worker. An
  assignment or job whose frame is past the room the engine's stream
  grants at once, `io.largest_room` (its `io.output`), is answered on the
  engine's side by jig's workers' component, refused as an invalid start
  whose words name the record, its frame's bytes and the room (jig's
  `protocol/channel/records.md`, 2, and `jig/docs/design/protocol/channel/README.md`, 8). The run fails as any invalid start
  does, with serve's operator line below, and serve writes the link's
  `worker unsent` line too (`../../protocol/cluster.md`, 3). The link, its
  pings and the worker's other runs go on; nothing waits for an answer
  the worker was never sent. A resumed start's committed calls are cut
  to the room first (1.1, *Transcript*), so a start fails so only when
  it is past the room with every answer cut, by its charter, window,
  inbox and grants, as `standard` admits by its inbox alone. Such a
  start fails so, with its numbers, rather than silencing the link
  until the worker's silence deadline drops it and every run on it.
  Limits under which even the smallest resumed start, every call
  `core.call_records` admits cut beside an assignment's fixed fields,
  passes the engine's room are refused at start, naming the bound
  (`../../protocol/configuration/limits.md`).
- **An answer past the agent's reply bound** (`Limits::reply_bytes`,
  smith's `host_reply_bytes`) is given to smith as its `TooLarge`,
  never as a body its host codec would refuse: such a send would
  leave the call unsent and the agent draining.
- **Preparation failed:** transient (the forge could not be reached, the
  worker's side failed or ran out of time, another run of the workstream
  holds its checkout, or every checkout is held), or permanent until
  something changes on the forge, naming the repository: the forge does
  not have it, a branch or a commit, or it refuses the identity. jig
  counts a transient one as an infrastructure failure, within its own
  longer allowance, its detail on the task's hold (jig's
  `domain/tasks/lifecycle.md`, 6). The detail names the worker, the
  workspace, the repository, the step, git's command and fault, and the
  operation's and the prepare's times (`checkouts.md`, 1), and the worker
  logs a line for it (`checkouts.md`, 1.3).
- **The run failed,** the **agent failed,** or the run was
  **cancelled,** as jig's say, each with bounded detail for operators
  that no LLM is shown.
- **A program that started without smith's tools,** smith's
  `ModelFault::Untooled` (smith's `protocol/programs.md`, 5.6), is
  jig's `RunFailure::Untooled` with `model` none (jig's
  `domain/hosts/hub.md`, 3), a fault of its place, the program source's
  login on this worker: the engine keeps the task off that place
  without spending a try (jig's `domain/engine/fleet.md`). Its detail
  is a program fault's, naming the program and carrying smith's line.
  Every other program fault stays jig's `RunFailure::Model`.

When the hub accepts a run's failed or refused answer, serve writes one operator line
on stderr, which the box keeps in `<data>/box/logs/serve.log`
(`../../protocol/setup/box.md`, 2). The line names the task, run, attempt and
failure class or typed refusal reason. A refusal distinguishes a busy host,
an oversized charter, transcript, grants or messages, and an invalid start;
the invalid start names its cause and carries its decoding words. A
failed run's line prints the worker's bounded failure detail whole, which
the wire bounds at 4,096 bytes; past them it keeps the first 4,096, where a
failed preparation's fields are (`checkouts.md`, 1.4). A refusal's line
includes the last at most 4,096 bytes of its start words. Newlines,
control bytes and non-ASCII bytes are escaped,
so the record is one line. Acceptance emits it once; resending a kept
answer emits no second line. A full operator-note sink retains the diagnostic
until the sink drains. In-process and socket workers share this
path. The evidence is volatile operator output, outside turns, facts,
`TaskResult` and the review's reason sentence; no wire or stored field
carries an extra copy.

**An agent its host kit stops** (each of jig's agent failures: no
progress, wall time, a broken rule, an exit, an agent never started) is
logged by the worker as the kit reports the fault, before the run's
tree is gone and its answer goes up. The line's time is when the kit
stopped the agent; the engine's record of the failure is written once
the attempt concludes, which may be much later.

```
<binary> worker: run <run> attempt <attempt> agent stopped <failure>
```

`<failure>` is jig's agent failure as serve's line writes it
(`Agent(NoProgress)`). A stop for no progress ends `: no progress for
<ms> ms`, the run's no-progress deadline as its host limits derive it
(smith's `domain/host.md`, section 4). The line goes through the
worker's notes (`worker_alone`'s `Note`), which its shell writes, as a
failed preparation's does (`checkouts.md`, 1.3), once per attempt: a
kept answer sent again after a relink logs none.

**Each fetch and check out** a hold asks is logged by the worker as the
checkout asks it, one line each, so that a start assigned again and again
shows in the log with each attempt that fetched it (the user, 2026-11-19):

```
<binary> worker: run <run> attempt <attempt> fetch <repository> <source> into <into>
<binary> worker: run <run> attempt <attempt> check out <repository> <commit>
```

A job's lines put `job <kind> <verb>` after its attempt, its kind's
number and verb as its failure line prints them (`jobs.md`), so that
both join by the same words: `temper worker: run 9551 attempt 3 job 1
prepare fetch fabrig commit <id> into fetched`. `<run>` and `<attempt>` are its assignment's,
`<repository>` the repository's directory, `<source>` what the fetch
names (`branch <name>`, `commit <id>` or `default`; `../../protocol/git.md`,
4.3), `<into>` `fetched` or `landing`, and ids are in full, as
`../../protocol/git.md`, section 3 writes them. Each line goes through the
worker's notes, escaped to one line, as the agent's stop line does: operator
output that no fact or turn holds.

**Each assignment** the engine sends a worker, a run's or a job's, is one
serve line as it leaves, so that the worker's lines join it by run and
attempt and show why a start came again (the user, 2026-11-19):

```
<binary> serve: task <task> run <run> attempt <attempt> assigned to worker <name>: <why>
```

A job's puts `job <kind>` after `<name>`, its kind as serve's job line
prints it (`jobs.md`). `<run>` is the worker's `run`. `<name>` is the
worker's configured name (`workers[].name`), or `channel <n>` for a channel
no configured worker holds. `<why>` is the claim's cause (jig's
`domain/tasks/lifecycle.md`, 6): `recheck` where the attempt before ended in
one; else, with the retry notice the claim cleared, `retry after <class>
failure <count> of <allowance>` or `retry after refusal <count>`, then
`: <reason>` where it carries one; else the due reason, `made`, `ready`,
`message <n>`, `released`, `retry`, `narrowed` or `changed`, as the client's
activity names it. An attempt placed again after a busy refusal is a
second line. Escaped to one line as serve's other lines are; nothing
stores it.

**A configured model-call bound** is reported as jig's
`RunFailure::Exceeded(Exceeded { bound, value })`. Smith's
`CompletionFailure::Limit { which, bound }` supplies `value` unchanged:
the configured value in the bound's own unit, never a measured size.
The worker classifies both `Limit` and `Limit::Path(Cap)` exhaustively,
without a wildcard, so an added smith cap requires a classification.

| Smith limit | Jig bound | Unit |
|---|---|---|
| `Limit::Path(Cap::Reasoning)` | `Reasoning` | bytes, per item or in all |
| `Limit::Path(Cap::Metadata)` | `Metadata` | bytes |
| `Limit::Path(Cap::OutputItems)` | `Items` | answer items |
| `Limit::Path(Cap::Answer)` | `Answer` | bytes |
| `Limit::Path(Cap::Tools)` | `Tools` | tools |
| `Limit::Path(Cap::HistoryItems)` | `HistoryItems` | request messages and blocks, counted together |
| `Limit::History` | `History` | a retained-history ceiling on messages, blocks or bytes; the tag does not identify which |
| `Limit::Window` | `Window` | tokens |
| `Limit::Path(Cap::Output)` | `Output` | declared output-ceiling tokens |

For `History`, the value is the ceiling that compaction could not bring
the retained history under. It is carried unchanged, without assigning
it a unit the tag does not identify.

- **The included bounds name the call's allowance.** The answer's
  reasoning, metadata, items and bytes, and the request's tools, history,
  window and declared output ceiling become `Exceeded`. The tasks child
  holds the run at once without counting a try (jig's `domain/tasks/lifecycle.md`,
  6). The classification does not infer a measured size from the
  failure or require proof that every provider response will repeat.
- **Other limits stay `Model`.** `Cap::Request` stays `Model` rather than
  being identified with a retained-history or model-window bound.
  `Cap::RequestHead` depends on the credential. `Limit::Call` and
  `Limit::Message` have no production constructor. `Cap::ResponseHead`,
  `ResponseFields`, `Skip`, `Field`, `Retained`, `Tokens`, `Depth`,
  `Strings`, `Number` and `ErrorBody` can describe framing or a transient
  error body, rather than the call's configured allowance.
  `Cap::AccessToken` depends on credentials.
  `Limit::Pool` is a connection reservation, which can depend on
  credential-bearing request headers; `CompletionFailure::ApplicationLimit`
  also covers temporary call-slot refusal. None is classified as one of
  the nine call bounds. Other completion failures retain their `Model`
  classification.

**A model call's failure is named.** A run failed for its completion
(smith's `ModelFault::Completion`) and reported as `Model` carries jig's
`ModelFailure` beside it (jig's `domain/hosts/hub.md`, 3), from smith's
structured fields alone:

- **The endpoint and the model** are those the run's charter names for
  the failure's model index (`Charter::model`): the endpoint's configured
  name and the model's name, each cut to `ModelFailure::NAME_BYTES` at a
  character's boundary. The worker keeps them from the start it decoded
  until the run's answer; a run whose index names no model carries none.
- **The kind** is smith's `CompletionFailure` by name, one to one:
  `Unavailable`, `Overloaded`, `RateLimited`, `Exhausted`, `TimedOut`,
  `ContextTooLong`, `Invalid`, `Refused`, `Unsupported`, `Unauthorized`,
  `Protocol` and `Cancelled` as themselves, and `Limit` and
  `ApplicationLimit` as `Limit`; the worker classifies it exhaustively,
  without a wildcard. The evidence is smith's, one to one.
- **The status** is the evidence's response status where it has one,
  else the report's status, which a program's result gives (smith's
  `protocol/programs.md`, 5.6), else none.
- **The refusal** is the report's code where the evidence is `Unsent`,
  the kind `Invalid` or `Unsupported` and no status came: the name smith
  gave the cause of a call it refused before sending it (smith's
  `protocol/llm.md`, section 6), which no provider sent. It is kept
  where it is at most `ModelFailure::REFUSAL_BYTES` of lowercase ASCII
  letters and hyphens, and is none otherwise, an empty code among them.
- **Every other model fault** (a provider failed for good, a model that
  kept declining, ran out of tokens or named no tool, a program that
  could not run) carries none, nor does an account's or a bound's
  failure, which have their own (above).

No provider code, request id, detail or diagnostic goes into it: the
report's other fields, a code a provider sent, and the bounded detail,
stay the operator's.

## 2. Open questions

- **A hold naming an agent run's last failure.** Settled for a model
  call's failure (the user, 2026-11-13; `../../draft/run-provider-outage.md`,
  2): its hold names the call from structured fields alone, the endpoint,
  model, kind, evidence and status (`hosted-runs.md`, 1.3; jig's `domain/hosts/hub.md`, 3),
  and no free text. For any other run failure the question stands: an agent task held
  `out of tries: run failures` names no failure: the reason is `hosted-runs.md`, 1.3's
  operator line alone. Job failures carry their bounded cause into the
  hold (`jobs.md`). Default for an agent run: the line alone, which `task
  show`'s way out points to and `temper help hold` and `temper help
  server` say how to read, until that proves too little. Keeping agent
  text means one last failure per hold, replaced on each failure, with
  a byte bound and readers the user decides first. The operator line's
  4,096-byte tail is no choice of that stored bound. Task readers could
  then see text now confined to the engine's log, including tool output,
  paths, provider errors or echoed credentials, which cannot be reliably
  redacted. Restricting it to owners and maintainers needs a reading path
  of its own. An agent ending's producer and its payload must carry the
  cause, as the job's do. The user's to decide.

- **A start past the link's room, made to start.** A resumed run's
  committed calls are cut to the room (`hosted-runs.md`, 1.1, *Transcript*), but
  `standard` admits a start past its 1,048,576-byte room by a full run
  inbox alone, 82 messages of up to 262,144 bytes (`hosted-runs.md`, 1.3). Making such a
  start run takes a figure or a form (jig's `protocol/channel/README.md`,
  section 13). temper/migration answered on 2026-11-09 that a later plan
  chooses from the deployment's refusal line: a start near the room
  sizes `io.output` from the configured room, and one far past it bounds
  what the start carries, as the committed calls' cut does. Default: it
  fails with its numbers.
