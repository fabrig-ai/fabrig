# Hot and cold parts

How the records written on most decisions keep what changes rarely in
rows of their own, so that the log carries what changed and not each
record whole again (temper's `draft/store-shape.md`, section 7). On
dev.fabrig.ai three kinds of record were over 90% of what the log
wrote, each about 3 to 9 KB put again for a change of a few bytes
(the store's assessment of 2026-11-13, which that draft cites): live tasks,
goals' projections, and temper's change records. Each is kept as a cold
part, put when the record is made and again only when an act changes
it, and a hot part holding the rest.

- **The rule** (1): the owner's domain keeps its record whole in
  memory; a write puts each part whose fields changed; every read hands
  the record whole.
- **Live tasks** (2): their spec, contract, authority and instructions
  in a keyspace of their own, the rest at the live key.
- **Goals' projections** (3): their words in rows of their own, beside
  the goal's and each plan task's phase.
- **Jobs at open** (4) put a store's whole rows into their parts once.
- temper's change records are temper's (temper's `protocol/store.md`,
  section 2, families 47 and 48), by the same rule.

## 1. The rule

- **A record written on most decisions** keeps what an act changes
  rarely (a spec's words, an authority, words copied for a feed) in a
  row apart from what changes on most writes (a phase, a claim,
  counters). The two rows are its parts; the record is never one row.
- **The domain keeps the record whole,** as before: its fields, its
  reads and its tests are the record's, not the parts'. The parts are
  the store's rows, made from the record by the codec as it encodes a
  save and joined back as it decodes.
- **What a write puts:** a record made puts both parts in its commit.
  Each later save puts the hot part, and the cold part only when one of
  its fields differs from the cold part last put (the domain knows the
  record it last saved). An erase of the record erases both parts, in
  one commit.
- **What a read hands:** every read, the restart's, a point read's, a
  join's and a job's after this one, hands the record whole, joined from
  its two rows. One part without its other is a row that does not
  decode (`loads.md`, 1), which stops the engine.
- **Every field placed:** the codec's split and join name every field of
  the whole record, with no rest pattern, so a field added to the record
  does not compile until it is placed in a part. A field is cold when
  only a person's or an owner's discrete act changes it (making, an
  amendment, a producer's report); otherwise it is hot.
- **Bounds:** each part is a row of skein-kv's bounds; a record is two
  rows in the store's worst case (`README.md`, section 8) and in the
  commit edge (`records.md`, 4). A decision that makes or amends n
  records puts up to 2n rows, and admission that prices a decision's
  writes against the commit edge counts them.
- **Seen in the census** (`README.md`, 6.3): writes per keyspace, the
  hot part's keyspace written on most commits and the cold part's
  rarely.

## 2. Live tasks

| Part | Key | Value | Holds |
|---|---|---|---|
| hot | `01 01 00 <task>`, the live key as before | `Row`'s variant `live_hot`: `LiveHot` | every field of the live task's record but the cold part's |
| cold | `01 01 16 <task>` | `Row`'s variant `live_cold`: `LiveCold` | `spec`, `contract`, `authority` and `instructions` |

```
# The live task's part that changes, at its live key (parts.md, 2).
record LiveHot { activation_open: bool session: option Session closer: option u64 history: HotHistory run: TaskRun definition: HotDefinition started_from: list 64 Number interests: TaskInterests outcome: TaskOutcome asked_requester: bool question_woken: u64 talk: option Talk }
# TaskHistory without instructions; TaskDefinition without spec, contract and authority.
record HotHistory { ... }
record HotDefinition { ... }
# The live task's part an act changes, at 01 01 16 (parts.md, 2).
record LiveCold { spec: Spec contract: Contract authority: Authority instructions: option bytes 524288 }
```

- **Value types** (`layouts.md`, 2): the live key's range stays value
  type 0, the tasks' `Row` whole, which now holds `live_hot` there.
  `01 01 16` is value type 19, `Row`'s variant `live_cold`, over
  `[01 01 16, 01 02)`; placed holds narrow to `[01 01 15, 01 01 16)`.
  The census names `01 01 16` `tasks.live-cold`. `Row`'s `Value` gains
  `live_hot` and `live_cold` after its last variant. `live` and
  `live_talk` stay declared, read by the job (4) alone.
- **The ended record stays whole,** `ended_talk` at `01 01 06 <task>`:
  it is written once, when the task ends.
- **The tasks child's boundary:** its `Key` gains `LiveCold(u64)`, in
  the order of its bytes (`README.md`, 4.1: keys sort as they derive);
  its `Stored` gains `LiveCold`, the task's number and its four cold
  fields. `Stored::Live` saved puts the hot part; read, it is the whole
  record. A live task's save is therefore `Save { Live }` and, when the
  cold part changed, `Save { LiveCold }`, in the same decision.
- **When the cold part is put:** at the task's making; and at a publish
  where `spec`, `contract`, `authority` or `instructions` differ from
  the task's published record (the record last saved): an amendment
  (`../../domain/tasks/changes.md`), a procedure's decision replacing
  its words, a funding transfer moving `authority.budget.spend`. Every
  other save puts the hot part alone. A coalescing of a decision's
  saves (the questions' reconciliation keeps the last live save per
  task) keeps the cold save.
- **The end and every erase:** a task's end erases both live keys and
  puts its ended record whole, in one commit; wherever a live task's key
  is erased, its cold key is erased with it.
- **Reads:** the restart's live range stays `[01 01 00, 01 01 06)`
  (`loads.md`, 2); at each hot row the reader gets the task's cold row by
  a point read, as the ended joins get theirs, and hands `Stored::Live`
  whole. A live task's point read and the owner joins (`loads.md`, 3) do
  the same. `01 01 16` is in no restart range and no page walks it.
- **Models:** each part is a `Row` with its own `models`, the names its
  own value's selectors use.

## 3. Goals' projections

Under `01 00 01`, value type 1 (`Row`'s variant `projection`), as
before; `ProjectionRecord` gains four variants after `milestone`:

| Sub | Key | Variant | Holds | Part |
|---|---|---|---|---|
| 00 | goal | `goal_hot`: `ProjectionGoalHot` | `number`, `project`, `priority`, `phase`, `pending`, `changed`, `fed_at` | hot |
| 01 | goal, task | `task_hot`: `ProjectionTaskHot` | `goal`, `number`, `requester`, `phase` | hot |
| 02, 03 | goal, task, position or revision | `milestone`, as before | a pending milestone, written once, erased when fed | |
| 04 | goal | `goal_words`: `ProjectionWords` | `goal`, `task` (the goal's own number), `words` | cold |
| 05 | goal, task | `task_words`: `ProjectionWords` | `goal`, `task`, `words` | cold |

```
record ProjectionGoalHot { number: u64 project: u32 priority: u32 phase: PhaseBytes pending: list 1024 Connector changed: bool fed_at: option u64 }
record ProjectionTaskHot { goal: u64 number: u64 requester: Party phase: PhaseBytes }
record ProjectionWords { goal: u64 task: u64 words: bytes 262144 }
```

- **The domain keeps its cache whole,** the core's `Projection` with its
  goal's and plan tasks' words, and the feed is built from it as before
  (`../../domain/connectors.md`, 8).
- **A save puts only what changed:** the core's projection save, on each
  save of a task in a goal's plan, puts each of the goal's and the
  task's hot rows whose fields differ from what the cache held before
  it, and each words row whose words differ; a row new to the cache is
  put whole. A save that changes none puts nothing. History, the feed's
  decision and the last settlement write as before, through these rows:
  a milestone row; the header's hot row when `changed` or `fed_at`
  moves; and, at the forgetting, every row of the goal, its words rows
  among them.
- **Restore:** the core assembles its cache from the rows in any order,
  as it does today, joining each words row to its goal's header or its
  plan task; a hot row without its words, or words without their hot
  row, is an orphan as a plan row without its header is today.
- `goal` and `task` stay declared, read by the job (4) alone.

## 4. Jobs at open

Jobs of jig's patch set (`jobs.md`, 1), declared after the jobs it holds
when they land:

| Job | Walks | Writes for each row |
|---|---|---|
| `live-task-parts` | `[01 01 00, 01 01 01)` | a live task held whole (`live`, `live_talk`): its hot part at its key and its cold part at `01 01 16 <task>` |
| `projection-parts` | `[01 00 01 00, 01 00 01 02)` | a header or plan row held whole (`goal`, `task`): its hot row at its key and its words at sub 04 or 05 |

- **Its pages** come through the read edge (`jobs.md`, 3), so each row is
  at the binary's layout, where the whole variants stay declared. The
  job decodes it by the binary's code, splits it by the codec's split
  (2, 3) and encodes each part as a save of it is, `models` included.
  A row already in parts passes with no write.
- **Each answer** stays within the commit edge less the job's row:
  a whole row makes two puts, so an answer consumes the page's rows that
  fit (`through`) and the next page goes on after them.
- **Its cost:** on dev.fabrig.ai about 20 to 70 live tasks and about 80
  projection rows: a commit or a few, read and written in memory.
- **A later job** that rewrites live tasks or projections, declared after
  these, reads and writes their parts.

## 5. Tests

- **The split and the join,** step tests in `jig-protocol-store`: a
  generated live task's record and a generated projection's goal and
  plan task, split and joined, equal themselves; each part's key and
  record agree; the keys sort as they derive.
- **The writes,** in the tasks child's and the core's tests: a task's
  making saves both parts; a claim, a phase change or a message saves
  the hot part alone; an amendment of the spec, the authority or the
  instructions, a procedure's words and a funding transfer save the
  cold part; an end erases both live keys. A plan task's save that
  changes nothing the projection holds puts no projection row; a phase
  change puts its hot row alone; an amendment of its words puts its
  words row.
- **The reads,** step tests over the store: a restart hands each live
  task whole, equal to the record saved; a point read and an owner join
  likewise; a hot row without its cold row stops as a row that does not
  decode. The core's projections restart from their parts with the same
  cache and the same feed (`jig/tests/core`'s projections suite).
- **The jobs,** step tests: a store holding whole rows, as a binary
  before this one wrote them, runs each job once in pages within the
  commit edge; afterwards no whole row remains, every record reads back
  equal to the one written before, ended tasks and milestones are as
  they were, and a later start runs it no more.
- **Fuzzy:** the tasks world's lifecycle and amendments suites, with
  their restart cuts, read every live task back equal after a restart;
  the store codecs' fuzzy decoders take the new variants.
- **The real families** (`layouts.md`, 8): the new value type is new
  since the kept text, and every other type derives.

## 6. Open questions

- **Retiring the whole variants** (`live`, `live_talk`, the projections'
  `goal` and `task`) once no supported store can hold them: a later
  binary's, after every deployment has run the jobs.
- **A recurring task's state and a pending proposal** stay hot: each may
  hold a task's spec, and changes with its period or its decision. A
  census that shows them heavy would move them.
