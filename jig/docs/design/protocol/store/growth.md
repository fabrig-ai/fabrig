# The store: growth

## 1. Growth

### 1.1 Retention

Retention is the domain's: passes on the core's timer, tested in its
worlds, with horizons the deployment's limits give, each keyspace's
declared by its owner (`retention.md`). What becomes of an ended task
is its retention class, set from its template when it was made
(`../../domain/tasks/README.md`, section 3):

- **A task kept** loses its turns, then its full record, then its
  summary, each past its horizon (`retention.md`, 2 and 6). Its summary
  is written when it ends, so what parties read of old work comes from
  summaries once its record has gone: its rows in the ended indexes
  stay with the summary, and a task query joins it (`loads.md`, 3;
  `retention.md`, 7). A live task keeps its transcript, whatever its
  age.
- **Answered keys expire** past the people child's retention.
- **A task forgotten once read** (`../../domain/people/README.md`, section 5) goes
  in one decision: its record, its rows in the ended indexes, its
  result's update and its ended read position (`loads.md`, 4).
- **A forgotten task's remains go** at retention's next pass, whatever
  their age: its activity, history, terminals, transcript and summary,
  and its replies' rows in its requester's updates (`loads.md`, 4;
  `retention.md`, 6.4). Nothing reads them, and the requester's pages
  of updates skip those rows until then.
- **A deleted project's tasks go whole** (`../../domain/people/acts.md`, 2),
  whatever their retention class and age, once every task of the
  project has ended: each task in decisions of its own, one task at a
  time, its rows first, a page at a time (its history, activity and
  terminals, with its replies' rows in its requester's updates), then
  its transcript dropped (`transcripts.md`, 5) with its rows in the ended indexes and
  its ended read position, and its record, its history log and its summary last
  (`logs.md`, 9: each task found by its summary), so that
  a deletion a restart cut finds the task's record and erases what is
  left of it. A task of the project that retention already shrank is
  found by its summary, and its summary goes with its rows in the ended
  indexes. Its retention row is left, and erased alone when it comes due
  (`retention.md`, 4).

### 1.2 Fill

- **The store reports its fill** with each commit's answer: the map's
  bytes against its budget (skein's `data/kv.md`, section 7).
- **Above a high-water mark** of that budget (both are limits,
  `../limits.md`), the domain refuses new work at its
  entrances: new tasks, new sign-ins. Work under way goes on,
  and a refusal is never a failed commit. The deployment's document
  shows the fill.
- **Transcripts' disk** is the deployment's: retention bounds it, and its
  directory's free space is the operator's.

### 1.3 The census

What the store holds and what its log writes, per keyspace, so that a
deployment's owner sees where the store grows and what an upgrade would
walk (temper's `draft/store-shape.md`, 8). skein-kv counts (skein's
`data/kv.md`, 2.2); the component names the keyspaces and reports the
count up, and nothing of it is stored.

- **The rule:** the component gives every skein-kv store it opens for
  the map (the active generation's and an upgraded copy's, never the
  secrets' store) the census rule of its tags (`keys.md`, 1.2): a key under tag 0
  counts by its first two bytes, the component's family; under tag 1 by
  three, jig's child and family (`01 01 16`, the tasks child's
  attempts); under tag 2 by two, the application's first key tag
  (`02 0c`); any other first byte by one.
- **Names,** `OWNER.FAMILY`, ASCII lower-case letters, digits, `.` and
  `-`, at most 48 bytes: `component.format`, `tasks.attempts`,
  `core.terminals`, `temper.changes`. The component's and jig's come
  from a table beside the component's key encoders (`keys`), one name per
  tag they write; the application's from a table it hands the component
  with its owner sets, its own name and a name per first key tag it
  writes (`Store::keyspaces`). A prefix no table names has an empty name,
  and readers show its bytes. A test holds each table to its key
  writers: every tag written has one name, and no two names repeat.
- **The report,** `StoreCensus` (the core's vocabulary, beside
  `StoreFill`): the last commit counted (`commit`), the commit the log's
  period began at (`since`, skein's `data/kv.md`, 2.2), and a row per
  keyspace in prefix order, each its prefix, its name, its rows, keys'
  and values' bytes, its heads with their rows, and its puts, erases and
  bytes written since `since`. The component sends it up as
  `Event::Census`:
  - once the store has opened, after `Opened`: what recovery counted, a
    new generation's after registration and the jobs;
  - then after a commit's answer, once `CENSUS_EVERY` (60 s of the
    component's clock) has passed since the last report, so never while
    no commit is answered.
  The fill goes on with every commit's answer (1.2).
- **Up to the core:** the root hands each report to the core as the
  core's `Event::Census`, keeping the last one while it starts as it
  keeps the fill; the core keeps the last report and marks the
  deployment's document, which carries it
  (`../../client/domain/contract/documents.md`, 1). So a deployment
  document's watchers hear of the census at most once a minute, and a
  reader sees it as of its `commit`.
- **Its cost** is skein-kv's: a search of at most 128 keyspaces per
  operation applied, and a census of at most 129 rows, the empty
  prefix's included, of at most 8 heads each; the component's report
  copies it once a minute.
