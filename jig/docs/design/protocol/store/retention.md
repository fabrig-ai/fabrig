# Retention

What the store keeps of ended work, and for how long: each keyspace's
horizon, declared by its owner, and the domain's passes that erase what
has passed it (temper's `draft/store-shape.md`, sections 6 and 10.1,
milestone S6). Until S7 moves history into a log per task, all of it is
in the map, so retention is what bounds the map's growth.

## 1. In one page

- **Horizons by keyspace** (2): an ended task's transcript goes 14 days
  after it ends, its full record and history 30 days after, and its
  summary and index rows a year after. These are three limits of the
  deployment. The prunings that run today (settings' history, answered
  keys, temper's landing history and decisions for an away person) keep
  their own rules.
- **A summary per ended task** (3): about 1 KB, written in the decision
  that ends the task, beside its full record. Once the full record has
  gone, a task query's row is drawn from the summary.
- **A retention row per ended task** (4), keyed by its stage and the time
  its horizon counts from, so a pass reads only what is due, oldest
  first.
- **Passes on the core's timer** (5): one at start, then one an hour
  after the last ended. A pass takes each stage's due rows a page at a
  time and handles one task at a time, behind the commits before it. It
  shares the walk of a deleted project's erasure, so the two never erase
  the same task at once.
- **Stages** (6): the turns are dropped; the full record is shrunk to
  its summary, which waits while live work names the task or its
  requester has not read it; the summary goes; a forgotten task's
  remains are collected; and a person's decided proposal goes.
- **Readers** (7): a shrunk task is listed from its summary, and any
  other read of it answers Gone. A task whose turns were dropped shows
  its activity lines without their turns.
- **At the upgrade** (8), a job at open writes the summary and the
  retention row of each task already ended, and a row for each person
  proposal already decided. Nothing is erased at once.
- **Connectors** (9) hear `Unname` when a task's full record goes.

## 2. Horizons

| What | Owner | Kept | Erased by |
|---|---|---|---|
| live tasks, claims, holds, slots, pools | tasks child | while live | the decisions that end them, as today |
| an ended task's turns (its transcript's file and places) | component, for the tasks child | `retention.turns_ms` after it ends (14 days) | the turns stage (6.1) |
| an ended task's full record, history, milestones, activity, terminals, decisions on its own proposals and escalations, its result's index row, its read positions, and its updates | tasks child, core, people child | `retention.records_ms` after it ends (30 days), and while live work names it or its requester has not read it | the record stage (6.2) |
| an ended task's summary and its rows in the ended indexes (by requester, by project, ended delegates) | tasks child | `retention.summaries_ms` after it ends (a year) | the summary stage (6.3) |
| a task forgotten once read: what its forget leaves | tasks child | until the next pass | the forgotten stage (6.4) |
| a person's decided proposal, its decision and its update | tasks child, core | `retention.records_ms` after its decision | the proposal stage (6.5) |
| goals' projections | core | until the goal ends | the core's projections, as today |
| settings' history | core | its history limit | as today (`../../domain/engine/settings.md`) |
| answered keys and acts | people child | the request retention | as today (`../../domain/people/acts.md`, 1) |
| temper's records of a task | temper | with the task's full record, where temper's own rules let them go | temper, on `Unname` (9) |

- **The three horizons are engine limits** (`../limits.md`), named
  `retention.turns_ms`, `retention.records_ms` and
  `retention.summaries_ms`, with the defaults the user set (11-13): 14
  days, 30 days and 365 days, in every profile. A deployment overrides
  them by name. The start refuses a horizon of 0, and three that are not
  in order (turns at most records, records at most summaries), naming
  the limit.
- **A task's horizons count from its end,** `ended_at`, whatever its
  retention class. A deferral (6.2) counts again from the time it
  deferred.
- **A deleted project's tasks** go whole, with no horizon waited for
  (`README.md`, 6.1).

## 3. The summary

`TaskSummary`, in the tasks child (`jig_domain_tasks`), stored as the
tasks family's `Stored::Summary(Box<TaskSummary>)` at
`Key::Summary(task)`: tag 30 under the tasks child, census name
`tasks.summaries`.

- **What it holds:** what a task line of an ended task shows
  (`../../client/domain/contract/documents.md`, 1), and what the stages
  after the record's need:
  - its number, root, depth, requester, project, label, flags (with its
    retention class), tracked priority and executor;
  - its title: the first line of its spec's words, at most 256 bytes,
    cut at a character's boundary, and whether it was cut;
  - its ending, as the chip shows it: done, failed or cancelled (with
    who cancelled and when); the result's kind; and its words (a
    report's words, a change's title, a verdict's code and words, an
    answer's words, a failure's or cancellation's reason), at most 256
    bytes each, cut, and whether it was cut. A verdict's fields, an
    approval's state and a job's bytes are left out;
  - its times: made, last changed, last activity, ended;
  - what its budget line reads: its spend and reservation, its budget's
    spend and deadline, its funder;
  - its ended delegates' counts (done, failed), its last attempt, its
    activity's last position;
  - its result's position and its last reply, so its requester's read
    state can be judged once the full record has gone.
- **Its size:** at most 1 KiB encoded at its texts' bounds; a test holds
  the bound against the schema.
- **Written when the task ends,** in the decision that ends it, beside
  `Stored::Ended` (`../../domain/tasks/README.md`, section 3), by
  `TaskSummary::of(&TaskRecord)`. It is never rewritten.
- **Erased** by the summary stage (6.3), by the forgotten stage (6.4),
  by the record stage for a task forgotten once read (6.2), and by a
  deleted project's erasure of its tasks (`README.md`, 6.1).
- **Not a live family:** the restart does not load it (`loads.md`, 2).

## 4. The retention family

`Retention { stage, since, number }`, stored as the tasks family's
`Stored::Retention(Retention)` at
`Key::Retention { stage, since, number }`: tag 31 under the tasks
child, census name `tasks.retention`, keyed by the stage, then `since`
(a wall time, in nanoseconds), then the number. The value holds the key's
fields and nothing more.

| Stage | Tag | `number` | `since` | Due at |
|---|---|---|---|---|
| `Turns` | 0 | a task | its end, or its last deferral | `since + retention.turns_ms` |
| `Record` | 1 | a task | its end, or its last deferral | `since + retention.records_ms` |
| `Summary` | 2 | a task | its end | `since + retention.summaries_ms` |
| `Forgotten` | 3 | a task | its forget | at once |
| `Proposal` | 4 | a person proposal | its decision | `since + retention.records_ms` |

- **One row per ended task,** naming its next stage: put as `Turns` by
  the decision that ends a task whose record says it turned
  (`ever_turned`), otherwise as `Record`. Each stage that finishes
  erases its row and puts the next stage's, with the same `since`.
- **A row is all a pass needs to find its work:** a stage's due rows are
  the prefix of its range up to `(stage, now − horizon)`, oldest first.
- **A row whose task has nothing left for its stage** (no full record
  for `Record`, no summary for `Summary`) is erased alone when it comes
  due. So an eraser other than retention (a forget, a deleted project's
  erasure) leaves the task's row in place, and needs no way to find it.
- **Not a live family:** the restart does not load it.

## 5. A pass

- **The core's timer,** `Timer::Retention`: due once the start has
  restored the core, then one hour (`RETENTION_EVERY`, a constant of the
  core) after the last pass ended. A pass that is running is not started
  again. The deadline is in memory: a restart runs a pass at once.
- **Its order:** the stages in tag order (`Turns`, `Record`, `Summary`,
  `Forgotten`, `Proposal`). For each, it reads the next page of due rows
  (at most `limits.documents.rows` rows), handles them in key order, and
  reads again until the stage has no due row. Then the pass ends and
  arms the next one.
- **One task at a time,** in the core's serialized walk of ended tasks
  (the erasure that a deleted project's tasks go by, `README.md`, 6.1).
  A retention pass and a deletion queue behind each other, so no task is
  erased by both at once.
- **In pages, behind the commits before them:** each read of a task's
  rows waits for the commits before it to be durable, as a deletion's
  does, and each decision erases at most what one page of rows brings,
  each page within its load's bounds (`../../domain/engine/README.md`,
  5.3).
- **Its clock** is the wall time of the step that reads the page.

## 6. Stages

### 6.1 Turns

- **Deferred** while live work names the task (the tasks child keeps its
  stub, `Key::Stub`): the row is erased and put again with `since` now.
- **Otherwise,** in one decision: `Write::DropTranscript { task }`
  (`transcripts.md`, 5), the row erased, and `Record` put with the same
  `since`. Several tasks' rows may go in one decision, within its bound.
  It reads no record: dropping a missing transcript is harmless.

### 6.2 Record: the full record shrinks to its summary

- **Loads** the task's ended record. With none, the row is erased alone.
- **Deferred,** the row put again with `since` now, while:
  - live work names the task (its stub); or
  - its requester is a person who has not read it: its result's position
    is past the party's read results, or, for a task flagged
    conversation, its last reply is past its ended read position. This
    is the read test of a task forgotten once read
    (`../../domain/people/README.md`, section 5). So the people child's
    census at start always finds the record of an unread result.
- **Its own rows,** a page at a time, a decision each: every row keyed
  by its number (its terminals, the decisions on its own proposals and
  escalations, its history, milestones and activity, and its ended read
  positions), read as `Scope::TaskOwnRows { task }`. Each activity line
  that names a reply or a notice to a person takes that update with it,
  by the key the line and the record name, as a deleted project's
  erasure does. The task's own ranges are one list in the store,
  shared with `Scope::TaskRows`, so that a family a later milestone
  keys by task joins both. The families keyed by something other than
  the task (decided person proposals, their decisions, parties'
  updates) are not scanned: a task's rows in them are erased by key.
- **Then, in one decision:** its transcript dropped (again, harmlessly,
  if the turns stage ran), its result's index row and its result's
  update, its live and ended read positions with the people child's
  caches of them (`people::Event::Forget`), and its row in the index of
  delegates (`Key::Delegate`) for a task requester.
- **Last, in one decision:** its ended record, `Unname` to every
  connector (9), its row erased and `Summary` put with the same `since`.
  Its summary and its rows in the ended indexes stay.
- **A task forgotten once read** that reaches this stage read (one its
  forget missed: the inbox no longer held it when it was read, or a live
  task named it then) goes whole here: its last decision also erases its
  summary and its rows in the ended indexes, and puts no `Summary` row.

### 6.3 Summary

- **Loads** the task's summary. With none, the row is erased alone.
- **Otherwise,** in one decision: the summary, its rows in the ended
  indexes (by requester, by project for a listed task, its ended
  delegate's row for a task requester), and the row. Nothing of the task
  is left.

### 6.4 Forgotten

The forget of a task forgotten once read (`../../domain/people/README.md`,
section 5) erases its record, its result's update, its rows in the ended
indexes and its read position in one decision, as today, and now puts a
`Forgotten` row with `since` the forget's time. Its remains go at the next
pass:

- **Loads** the task's summary, for its requester. With none, the row is
  erased alone.
- **Its own rows,** as the record stage's, a page a decision, each
  reply's and notice's update with its line; then its transcript dropped,
  its result's index row and its row in the index of delegates; last,
  its summary and the row. Its connectors heard `Unname` at its forget.

### 6.5 Proposal

A person's proposal, once decided (accepted, rejected or withdrawn),
keeps three rows: its decided proposal
(`Key::DecidedProposal`), its decision (the core's
`ProposalDecision { proposer: Party, proposal }`, which keeps the
decision's time) and its proposer's update. The decision that decides it
also puts a `Proposal` row, `since` the decision's time.

- **Loads** the decided proposal and its decision. With neither, the row
  is erased alone.
- **Otherwise,** in one decision: the three rows and the row. The update's
  key is the proposer, the decision's time, the proposal's first task
  and the proposal's number, as the decision wrote it.

## 7. Reading what retention left

- **A task query's ended rows** (`loads.md`, 3) join the task's ended
  record, or its summary once the record has gone. A row from a summary
  is a task line (`../../client/domain/contract/documents.md`, 1) built
  from it: its chip's title, executor, ended phase and ending from the
  summary's; its spend and budget from the summary's figures; no
  conversation read, since the read position went with the record; no
  live subtree. Its document's other fields are absent, as a stored
  closing row's are. The same holds for an ended delegate listed in a
  live task's document. No wire changes: a line from a summary is a
  line.
- **Any other read of a shrunk task,** its document, its result, its
  text, its history or activity, answers it as gone: a point read
  answers `Why::Gone` where a summary is found and no record; a page of
  its history or activity is empty, as for an erased owner (`loads.md`,
  3).
- **A task whose transcript was dropped** keeps its activity lines until
  the record stage. A page of its activity carries each turn line
  without its turn (`UnjoinedTurn::Unread`, which a client shows as a
  line with no decoded turn), and goes on: the page does not end at
  the first such line, since no frame of the task can be read. A single
  turn's read answers as for a dropped transcript, as today
  (`transcripts.md`, 3).

## 8. At the upgrade

Two jobs at open of jig's (`jobs.md`), run once per store:

- **`retention-rows`,** over `tasks.ended`: for each ended task, its
  summary (3) and its retention row (4), `Turns` if its record says it
  turned, otherwise `Record`, with `since` its `ended_at` (its `changed`
  where none is kept). About 2,600 tasks on dev.fabrig.ai, two puts each.
- **`retention-proposals`,** over the core's decisions on proposals by a
  party: for each, a `Proposal` row with `since` the decision's time.

No row is erased by either. The first pass after the upgrade erases only
what is due: on dev.fabrig.ai, the first turns about 11-24 and the first
full records about 12-10.

## 9. Connectors

- **`Unname { task }`** goes to every connector in the decision that
  erases a task's full record: the record stage's last decision, a
  forget, or a deleted project's erasure (`../../domain/connectors.md`,
  3.4). A summary may stay a year after; no connector reads one.
- **temper** drops the task's change record then, where its room's rules
  let it go (temper's `domain/forge/change/README.md`, 7), beside the
  names it erases today.

## 10. Tests

- **Step tests in `jig-protocol-store`:** a summary and a retention row
  round-trip through the tasks family's codec, and their keys sort as
  section 4 says; the census names `tasks.summaries` and
  `tasks.retention`; the summary's bound holds at its texts' bounds;
  the two jobs write the rows section 8 gives over a fixture store; a
  task query's row joins a summary when the record is gone; an activity
  page of a task with no transcript carries every turn line and goes on.
- **The core's world** (`jig/tests/core`), its clock moved on: a task
  ended past the turns horizon loses its transcript; past the records
  horizon it loses its record, history and activity and keeps its
  summary, listed by a task query from it, its document Gone; past the
  summaries horizon nothing of it is left; a task named by live work,
  or unread by its person requester, is deferred and goes once released
  or read; a forgotten task's remains go at the next pass; a decided
  person proposal goes past its horizon; a restart runs a pass at once;
  a deleted project's tasks go whole while a pass waits behind it.
  Focused cases name each stage; a fuzzy sweep interleaves ends, reads,
  forgets, deletions and passes and checks that nothing of a task
  outlives its last stage and nothing goes before its horizon.

## 11. Open questions

- **Summaries' weight:** at dev.fabrig.ai's pace, a year of summaries is
  about 330 MB in the map (temper's `draft/store-shape.md`, 11). If that
  weighs too much, older ones move to a log by month.
- **A requester who never reads** keeps their unread tasks whole. A
  bound on how long an unread result defers its task is open.
- **Remains of tasks forgotten before S6** have no retention row and stay
  until their project is deleted: finding them would walk every history
  family once, for what jig's own templates never leave (its chat
  template keeps).
- **A live conversation's updates** stay while it lives, read or not, and
  go with its record (6.2): an update names no read mark of its own.
