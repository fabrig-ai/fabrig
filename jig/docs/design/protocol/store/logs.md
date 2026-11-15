# Task logs

2026-11-15. Where a task's history is kept: not in the map, but in two
of skein-kv's payload logs per task (skein's `data/kv.md`, section 6),
in the store's generation, compressed, read when asked and dropped
whole (temper's `draft/store-shape.md`, section 2, milestone S7 of
`draft/store-shape.execution.md`). The map then holds state: live work,
indexes, summaries, and a few rows per task saying where its logs are.

- **Two logs per task** (1): its history log, the rows of section 2's
  families, and its turns log, its turns from S7 on.
- **What moves** (2): the families a task appends to and never changes,
  each row kept by its key as the map kept it.
- **Written before the commit** (3): a decision's rows of a task are one
  record of its history log, appended and synced before the commit
  that records its extent.
- **Read through a view** (4): every read of the map's families reads
  these keys from the map and the task's history log, fetching the
  log's rows a read needs and running the read again with them.
- **Turns** (5): a task's turns go to its turns log, but where the task
  already has a transcript file, which it keeps writing until it ends.
- **The job `task-logs`** (6) moves a store's history out of the map at
  the open that brings this design.
- **Generations, layouts, retention** (7 to 9): the logs are linked into
  a new generation, their value types' layouts never retire, and a task's
  logs go whole at its horizons.

## 1. A task's logs

- **Named by the task:** its history log is skein-kv's log `2 × task`,
  its turns log `2 × task + 1` (skein's `data/kv.md`, 6.1, which puts
  them in the store's `logs/`). A task number of 2^63 or more has no
  log, and a commit that would make one fails `Check(Capacity)`
  (`README.md`, section 5).
- **Their extents** are skein-kv's rows under the log rule the component
  gives its store at every open, the prefix `00 09`: so `00 09 <log>`,
  eight bytes big-endian, holds that log's length. The component never
  writes them; their presence is how it knows a task has a log. The
  secrets' store has no log rule.
- **Their places,** the component's own rows (`records.md`, 1), added to
  its family in place as its rows are:

```
# 00 0a <task>: a task's history log, from its first record
record HistoryLog { record: option u64 }          # the start of the frame holding its ended record, while it has one
# 00 0b <task>: a task's turns log, from its first turn
record TurnLog { window: option u64 }             # the start of its current window's first turn
# 00 0c <task> <attempt>: an attempt's first turn in the turns log
record LogAttempt { first: u64 }
```

- **The census** names them `component.logs`, `component.histories`,
  `component.turn-logs` and `component.log-attempts` (`README.md`, 6.3).

## 2. What moves

A **log-held key** is a key of one of these families, its task the
key's first field after the family's tag:

| Family | Key | Range |
|---|---|---|
| `core.terminals` | task, attempt | `01 00 05` |
| `core.proposal-decisions`, a task's proposals | `Task(task)`, proposal | `01 00 06 00` |
| `core.escalation-decisions` | task, revision | `01 00 07` |
| `tasks.ended`, an ended task's full record | task | `01 01 06` |
| `tasks.history` | task, revision | `01 01 09` |
| `tasks.milestones`, none since S5's job | task, position | `01 01 0a` |
| `tasks.activity` | task, position | `01 01 0b` |
| `tasks.attempts` | task, attempt | `01 01 16` |
| `tasks.holds` | task, position | `01 01 17` |

- **Turns** are not rows: they go to the turns log (5).
- **What stays in the map:** a person's proposals and their decisions
  (`tasks.decided-proposals`, a party proposer's `core.proposal-decisions`),
  which are no task's and have a horizon of their own (`retention.md`,
  6.5); past funding periods (`tasks.closed-ledgers`), keyed by funder
  and read by spend pages; parties' updates, the summaries, retention's
  rows and every index (`README.md`, 4.4); and the application's records,
  temper's change records among them, which go with the full record by
  temper's own rules (temper's `protocol/store.md`).
- **One function says it:** `keys::logged(key) -> Option<u64>`, the
  task whose history log holds a key, none for a key the map holds. A
  test holds it to this table over generated keys of every family.

## 3. Written before the commit

- **A history record** is one commit's rows of one task: after a
  commit's writes become operations (`README.md`, section 5, one per
  key, its last write), its log-held keys are taken out, grouped by
  task, and each task's written as one record in key order:

| A history record's field | Bytes |
|---|---|
| format, 1 | 1 |
| then each row: its key's length | 2 |
| its key, the map's key whole | as long |
| kind: 1 put, 2 erase | 1 |
| for a put: its value's length, then its value | 4, as long |

  Big-endian; the value as the map would hold it, its head the binary's
  layout id (`layouts.md`, 4). A record holds at least one row, and a
  format past 1 is a record the component cannot read. Its golden bytes
  are kept as the transcripts' framing's are (`README.md`, section 9).
- **Appended in the commit:** the record framed by skein-kv
  (`Store::frame`) and appended to the task's history log
  (`Op::Append`), placed at `log_end` (skein's `data/kv.md`, 6.2); the
  commit is answered once skein-kv has written and synced the log and
  then the commit. A record past skein-kv's `Limits::record` fails the
  commit `Check(Encoding)`.
- **The map's own row goes with it:** where the map holds a log-held key
  the commit writes (a store before the job of 6, or that job's own
  pages), the commit also erases it from the map, so the map and a log
  never hold one key.
- **The ended record's place:** a record that puts a task's
  `tasks.ended` key puts `HistoryLog { record }` at `00 0a <task>`, the
  start of that frame; one that erases it puts `HistoryLog { record:
  none }`. A task's first record with neither puts `HistoryLog { record:
  none }`, so that each task with a history log has its row.
- **An erase of a log-held key** is an erase row in the task's record:
  the rows before it in the log no longer hold the key. A forgotten
  task's record is erased so (`retention.md`, 6.4, below 9), and so are
  the rows the walks that end a task's history erase before its drop
  (9).
- **`Write::DropHistory { task }`** drops the task's history log
  (`Op::Drop`) and erases its `00 0a` row. A commit that drops a log
  and writes one of its keys fails `Check(Capacity)`: a writer's fault.
- **Bounds:** a commit's operations grow by an append, an extent and a
  place per task it writes history of; its bounds count them
  (`Check(Capacity)`).

## 4. Read through a view

Every read the component answers over the map (a restart's page, a
child's load page, a party's read and its joins, a get, and the
erasure's pages) reads through one **view**: the map, and the rows of
task logs this read has fetched. The jobs' pages read the map alone
(6). Transcripts are read as 5 says.

- **A log-held key** is answered from the map's rows and, where its
  task has a history log (`00 09 <2 × task>` holds an extent), from the
  rows this read fetched of it. A key the two held would be a fault the
  writes of 3 never make.
- **A miss:** a get or an interval of a task's log-held keys whose log
  this read has not fetched, its task having a history log, answers
  **missing**. The run goes on without those rows where it can, so that
  one run names every task it misses, and then answers nothing.
- **A fetch,** for each task a run missed: the task's history log read
  forwards from its start to its extent (`ReadLog`, a page of records
  at a time), each record's rows decoded, and the rows of the intervals
  the run asked of that task kept, the newest record's row for a key, an
  erase removing it. The ended record is not walked for: a get of
  `tasks.ended` reads the one frame its `HistoryLog.record` names, and
  a task whose row names none has no ended record in its log. Fetched
  values pass the read edge (`layouts.md`, 4) as the map's do.
- **Run again:** once its fetches are read, the read runs again from
  its start with the fetched rows in its view; a run that misses
  nothing answers. A run may miss a task the last one could not see (a
  join found by a fetched row), and fetches again: each round fetches
  at least one log more, within the read's own join bound.
- **Commits wait:** while a read fetches, the component holds its
  commits, as it holds them behind a transcript read (`transcripts.md`,
  6), so the map and the logs are as the read's runs found them.
- **Bounds:** a read's fetched rows are bounded by the component's
  `Limits::fetch` bytes (`README.md`, section 8), each row its key and
  value. A fetch that would
  pass it fails its read as a read past its bounds fails
  (`loads.md`, 1 and 3): a party's read cut, a load page refused. A
  history page's or an activity page's intervals are narrow (positions
  from its own, `loads.md`, 4), so only a task's whole rows read at once
  (the erasure's pages) can meet it.
- **A frame the log cannot give** (`LogUnread`) fails the read as a row
  that does not decode does (`loads.md`, 1).
- **What reads differ:** none of the scopes of `loads.md`, 3, nor the
  restart's census and question lines (`loads.md`, 2), changes what it
  answers. Two scans change their source: a project's deletion finds
  its ended tasks by their summaries (9), and no read walks the ended
  records of every task.

## 5. Turns

- **New turns go to the turns log:** a turn's save (`README.md`,
  section 5) of a task with no transcript place (`00 01 <task>`,
  `transcripts.md`, 1) is a record of its turns log, the record a
  transcript frame's body (`transcripts.md`, 1), appended in its
  commit; `TurnLog { window }` moves to it where it opens a window, and
  `LogAttempt { first }` is put where its attempt has none. The frame's
  framing is skein-kv's, compressed, not the transcript file's.
- **A task with a transcript file keeps it:** where the task already
  has a transcript place, written before S7, its turns go on to its file
  as `transcripts.md`, 2 says, until it ends. No transcript is
  converted; the files go by retention (9).
- **Read as a file's frames are:** every read kind of `transcripts.md`,
  3 (resuming, a turn, a page's turns, a page of turns, a tail) walks a
  turns log's records by `ReadLog` from its places as it walks a file's
  frames, each record a frame's body, patched as read alike
  (`transcripts.md`, 6). A record past what is left of a page is read
  whole alone, its fields kept and its bytes left out, as a cut turn is.
  A record skein-kv cannot give is a frame unread.
- **No cut at open:** skein-kv's extent leaves a torn tail as garbage
  (skein's `data/kv.md`, 6.2), so a turns log needs none of
  `transcripts.md`, 4.
- **`Write::DropTranscript { task }`** drops both: the file as
  `transcripts.md`, 5 says, and the turns log (`Op::Drop`) with its `00
  0b` row; its attempts' `00 0c` rows are orphans gone in the commits
  that follow, as the file's attempts' places are.

## 6. The job `task-logs`

jig's job at open (`jobs.md`), last of jig's set, that moves a store's
history into its tasks' logs (temper's `draft/store-shape.md`, 10.1, S7).

- **Its ranges:** section 2's, in key order.
- **Its answer** puts each row it is handed at its own key, and consumes
  the page through its last row that fits a quarter of its room's writes
  and its room's bytes (each row may cost an erase, and its task an
  append, an extent and a place). The commit path of 3 does the rest: the
  page's rows of each task become one record of its history log, the
  logs are synced, then the commit erases the rows from the map and
  records each log's extent. So the map holds no log-held key once it is
  done.
- **No transcript moves** (5), and no value is converted beyond the
  read edge every page's values already pass (`jobs.md`, 3).
- **Its cost** on dev.fabrig.ai: about 100,000 rows of some 3,000 tasks
  once S5's job has folded their milestones, a few tens of MB; a log
  file made for each task, and a sync per task per page.
- **An empty store** marks it done at its first commit (`jobs.md`, 2).

## 7. Generations

The logs are in the store's `logs/`, so in its generation (`README.md`,
section 2):

- **Linked into a new generation** with the snapshot and the sealed
  segments (`opening.md`, 2): `logs/` and each directory in it made in
  `A + 1`'s `store/`, each log file given a second name there, each
  directory synced after its links. A shared log stays readable from
  `A` up to `A`'s extent, since `A + 1` writes it only past that (skein's
  `data/kv.md`, 6.4).
- **A rehearsal copies them** file by file, as it copies the maps
  (`opening.md`, 5).
- **A generation dropped or a stray removed** takes its `logs/` with
  it, directory by directory; a file another generation names stays.

## 8. Layouts

A value in a log keeps the layout it was written at, and is converted
as it is read (temper's `draft/store-shape.md`, section 3):

- **A log-held value type's layouts never retire** (`layouts.md`, 5.4):
  the census counts the map alone, and a log may hold any of them until
  retention drops it.
- **The open's check** takes each of them as held (`layouts.md`, 5.3):
  one with no plan to the binary's layout refuses the open, since no
  written job converts what a log holds. A change to a log-held value
  type is therefore one a plan derives.

## 9. Retention

- **Turns** (`retention.md`, 6.1): `Write::DropTranscript` drops the
  turns log and any transcript file (5).
- **The full record** (`retention.md`, 6.2), **a deleted project's
  task** (`README.md`, 6.1) and **a forgotten task's remains**
  (`retention.md`, 6.4): the walk reads and erases a task's rows a page
  at a time as it always has, through the view, each erase of a
  log-held row an erase row of its log (3), the activity lines naming
  the replies' updates it erases with them; `Write::DropHistory` goes
  in the decision that erases what the walk keeps last: the task's ended
  record for the first two, its summary for the remains. A forgotten
  task's record is erased at once by its forgetting, as an erase row (3),
  so nothing reads it from then on.
- **A project's deletion finds its ended tasks by their summaries**
  (`tasks.summaries`, written for every ended task since S6), not by
  walking ended records, which no read walks across tasks.

## 10. Tests

- **Step tests,** in `jig-protocol-store`: `keys::logged` against
  section 2's table; a history record's bytes both ways, with golden
  bytes; a commit with history of two tasks appends one record to each,
  places the ended record, and leaves the map without the keys; an
  erase row hides a key; `DropHistory` drops the log and its row; each
  read kind of `loads.md`, 3 over a task whose history is in its log
  answers as over the map, a run missing then fetching then answering;
  a fetch past `Limits::fetch` failing its read; the job over a store whose
  history is in the map, leaving none there and every read answering as
  before; a turn of a task with no transcript file in its turns log, one
  of a task with one in its file; every read kind of `transcripts.md`, 3
  over a turns log.
- **The store's world,** `jig-store-world`: commits with history and
  turns crashed at every file operation, the referee finding each
  task's history and turns exactly as the acknowledged commits left
  them; a new generation's logs linked, read from both up to each
  extent; a drop, a rehearsal and a going back with logs.
- **The domain worlds** run as they are, over skein-kv's in-memory logs
  (skein's `data/kv.md`, 6.5): every story reads its history through the
  view, and the restart oracle reads back what it wrote.

## 11. Open questions

- **Reading a long log:** a fetch reads a task's whole history log, so a
  page of a task with tens of thousands of lines reads them all. An
  index of positions to frames, kept in the log, would read a page's
  frames alone.
- **A rehearsal's copy** grows with the logs; linking them, with a copy
  made only of a log the rehearsal appends to, would spare it.
- **The logs' bytes in the census** (skein's `data/kv.md`, section 12).
- **The walks' erase rows:** a walk that ends a task's history erases
  its log-held rows one by one before the log goes whole, so the log
  grows by an erase row per row first. Leaving them would need the
  domain to know which of its families a log holds.
