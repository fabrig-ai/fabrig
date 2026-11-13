# Landing history

Provisional, 2026-11-13. How temper keeps each change's and each
landing's steps as events after their rows have gone: each event
written by the step that decides or observes it, in the same decision,
under `queue` and under `turn`; kept by change in families the store
holds and the forge reads on demand, never restored; swept by age and
by count; and read through `temper ctl landing history`, and by
`change show` for a change whose row went. It is the draft's milestone
H1 (`../draft/landing-history.md`, sections 1 to 6, 8, 9 and 10; its
execution's H1). Where the time went, `landing times`, is H2's
(section 9). A change's steps are `forge/change/README.md`, 2 and
`forge/queues.md`; a landing's are `landing.md` and `landing-search.md`;
after landing, `after-landing.md`, 6.

## 1. In one page

- **An event per step, at both ends of each interval** (section 2): a
  row naming one change, with its project's number, its time, its
  landing, round and unit where it has them, the heads it concerns, its
  identity and its kind's fields.
- **Written by the step** (section 3): in the decision that changes the
  state it records, through one writer in the forge's top; a step that
  finds its state already recorded writes nothing, so a retry, a rerun
  under its id or a restart's relearning appends no second event.
- **One change per event:** a landing's step that concerns several
  members writes an event for each, so each change's history is whole
  on its own and goes with it (3.3).
- **Kept apart from the rows** (section 4): five families, one restored
  row per project (byte 127) and four demand-loaded families (bytes 128
  to 131), which the restart's interval leaves out; the forge loads them
  through the root.
- **Swept by change** (4.5): a change's events go together once its last
  is older than `history_days`, or, past `history_events`, the changes
  whose last event is oldest first; on writes and on the forge's hourly
  check, so idle projects are swept; a project's deletion sweeps all of
  it.
- **Read by `ctl`** (section 5): `landing history`, a page of at most 32
  events under an upper bound the first page fixes; and `change show` of
  a change whose row went prints its last events and the read that lists
  them all.

## 2. An event

### 2.1 Its row

An event's record, `HistoryEvent`, keyed by its project and number
(4.1):

| Field | What it holds |
|---|---|
| `project` | the project's number |
| `number` | its place in the project's order, from 1, given by the project's row (4.2): it never goes back, even once every event has gone |
| `at` | temper's time, the wall time of the decision that wrote it (3.4) |
| `source_at` | a source's own time for the step, where the source gives one temper reads (3.4); none otherwise |
| `change` | the change's task |
| `repository` | the change's repository, forge and id |
| `landing` | under `turn`, its landing: the landing's task, its round (from 1) and the member's unit; none otherwise |
| `head`, `base` | the commits it concerns, as its kind says (2.2); none where it names none |
| `operation`, `attempt` | its identity (2.3) |
| `what` | its kind and the kind's fields (2.2) |

Bounded records (`../draft/landing-history.md`, 5): each field is a
number, a hash, an enumeration or a reason of at most 256 bytes; a
`formed` event's members are at most `forge.landing_members`. No event
carries a log, a body or words.

### 2.2 Its kinds

Each interval's start and end are both written. *Head* and *base* name
what the row's `head` and `base` hold.

| Kind | Written when | Its fields | Head; base | Operation, attempt |
|---|---|---|---|---|
| `ready` | the change becomes ready (2.4) | none | its head | own |
| `unready` | it ceases to be ready, other than by landing or ending | `why`: `held`, `head_moved`, `refused` (a gate or CI at its head said no), `dependency`, `approval` (an approval lapsed), `conflict`, `closed` (its pull request closed) | its head | its `ready`'s |
| `joined` | it joins its queue (2.4), a landing (its first `formed` round), or a forge's own merge queue admits it | `line`: `queue`, `landing` or `forge_queue`; `place`, from 1, where its line has one; `priority`, its goal's, where its line orders by one | its head | `queue`: own; `landing`: the landing's task, the round |
| `departed` | it leaves that line without landing | `line`; `cause`: from a landing, its leave cause (`landing.md`, 4.3, the stored `LandingLeave`); from a queue, `unready`, `conflict`, `repair` or `ejected` | its head | its `joined`'s |
| `selected` | under `queue`, it is taken as the first to land (`first`) | none | its head; the base's tip | own |
| `update` | an update of its branch from the base is asked (an outbox entry) | `style`: `merge` or `rebase` | its head; the base's tip | the entry's number, 0 |
| `updated` | that update settles | `outcome`: `updated`, `conflict` or `failed` | the new head, when updated; the base's tip | the entry's number, 0 |
| `job` | a prepare job (`turn`) or a merge job (`queue`, `merge_job`) is delegated, its task heard | `job`: `prepare` or `merge`; `task` | its head; the base | the job's task, its attempt |
| `job-ended` | that job's result is heard | `job`; `outcome`: `done`, `conflict`, `failed` or `lost` | the prefix or head it pushed, where it pushed one | the job's task, its attempt |
| `conflict` | a conflict is found: by a job, an update, the forge's mergeability or its merge | `with`: `branch` or `earlier` (an earlier member alone); `files`, how many, 0 where unknown | its head; the base | own |
| `asked` | a resolution, a repair or a judgement is delegated to the change, its task heard | `ask`: `resolution`, `repair` or `judgement`; `task`; `gate`, the judgement's or a repair's gate number, where it has one | the head asked at; the base, for a resolution | the delegate's task, 0 |
| `answered` | that delegate's result is heard | `ask`; `outcome`: `pushed`, `passed`, `refused`, `nothing`, `failed` or `cancelled` | the head it pushed, where it pushed one | the delegate's task, 0 |
| `pushed` | a head reaches the change's branch, observed | none | the head | own |
| `delivered` | temper accepts a head delivered to the change | `checked`: a checked delivery, or an unchecked save | the head | own |
| `paused` | under `queue`, the change waits in a queue paused for its branch (`forge/queues.md`) | `why`: `branch_red` (CI failed on the branch's tip) or `tier_red` (a tier's red pauses it, `after-landing.md`, 7) | none; the branch's tip | own |
| `resumed` | that queue goes on | none | none; the branch's tip | its `paused`'s |
| `formed` | under `turn`, a landing forms, or a new round of it starts, with the change a member | `members`, in the stack's order, each its change, unit and head; `place`, the change's in it, from 1 | its head; the round's base | the landing's task, the round |
| `round-ended` | that round ends | `cause`: `turned`, `left` (a member left), `stale`, `cancelled`, `cut` (its search cut the stack) | none; the round's base | the landing's task, the round |
| `gate` | a gate's run at the round's top or a probed prefix is first known by its id, not yet ended: its dispatch's run found, or its rerun made; its `source_at` the run's creation | `gate`, its number in the repository's gates; `key` | the prefix; none | the run's id, its attempt |
| `gate-ended` | that run's outcome is heard | `gate`; `key`; `verdict`: `passed`, `failed`, `fault` (the runner's, retried), `timeout` or `superseded` (its round or head gone) | the prefix; none | the run's id, its attempt |
| `proved` | the round's top, or its search's green prefix, is proved | `proofs`, each a gate and its run, or none where a receipt already held proved it (`landing.md`, 7.2) | the prefix; the round's base | the landing's task, the round |
| `search` | the round's search by prefixes starts (`landing-search.md`, 2) | none | the round's top; its base | the landing's task, the round |
| `searched` | the search holds its boundary (`landing-search.md`, 4) | `green`, the green prefix, none for the base; `red`, the red prefix; `blamed`, the blamed unit | none; the round's base | the landing's task, the round |
| `probe` | a probe starts at a prefix (`landing-search.md`, 3) | none | the prefix | the round, 0 |
| `probe-ended` | the probe's outcome is heard | `green` | the prefix | the round, 0 |
| `turn` | the branch moves to the proved prefix (`turn`), or temper's merge of a `queue` change is made | `members`, how many landed in it | the prefix or the merge; the base | `turn`: the landing's task, the round; `queue`: the merge's entry number, 0 |
| `landed` | the change has landed: its landed commit is the branch's, by containment or by merge | none | the landed commit | own |
| `recorded` | the forge records its pull request merged, or a squashed member's is recorded merged, or closed for a squash | `state`: `merged` or `closed` | the commit it is recorded at; none | the pull request's number, 0 |
| `ended` | the change ends otherwise | `ending`: `no_change`, `cancelled` or `failed` | none | own |
| `landing-ended` | a landing ends: done, cancelled or held for good | `outcome`: `done`, `cancelled` or `held` | none | the landing's task, its last round |
| `held`, `released` | a hold on the change, or on its landing, is placed, or lifted | `on`: `change` or `landing`; `reason`, at most 256 bytes; `by`, the party or task that placed or lifted it, where known | none | `held`: own; `released`: its `held`'s |
| `after` | after landing, its verdict: verifying ended `verified`, `red`, `cleared` or `uncleared` (`after-landing.md`, 6) | `verdict` | the landed commit | own |

- **`queue` writes** every kind but `formed`, `round-ended`, `proved`,
  `search`, `searched`, `probe`, `probe-ended` and `landing-ended`;
  **`turn` writes** them all but `selected` and `paused`.
- **A forge's own merge queue:** `joined` and `departed` with its line
  are written where the forge's top observes the queue admitting or
  ejecting the change; where it observes neither, nothing is written for
  it (`forge/queues.status.md`).

### 2.3 Its identity

- **What it names:** `operation` and `attempt` name the operation the
  event records, as the table's last column says: a run's id and
  attempt, a job's task and attempt, a delegate's task, an outbox
  entry's number, a landing's task and round, a pull request's number.
- **Own:** where the column says *own*, `operation` is the event's own
  number and `attempt` 0; the event that ends its interval carries the
  start's number as its `operation`. So each interval's ends pair by
  identity, and no two events of a change share a kind and an identity.
- **Read, not decided by:** readers pair intervals by identity, and
  tests check uniqueness by it; the writer keeps no index of written
  identities. What keeps one step from writing twice is section 3's
  rule.

### 2.4 Ready, and its line, under each way

- **Ready** is the change's way's ready state: under `queue`, from
  entering `queued` or `first` (`forge/change/README.md`, 2, steps 5
  and 6) while every blocking gate holds at its head and its task is
  not kept by a dependency; under `turn`, from entering `ready`
  (`landing.md`, 3). It stays ready while it moves within its way's line
  (`queue`: queued, first, updating, merging; `turn`: ready, in its
  landing, judging), and is unready once it leaves that line for any
  other state but landed or ended, or the change policy finds it no
  longer available (an approval lapsed, a dependency not kept).
- **Its line, apart from readiness:** under `queue`, `joined` once it is
  counted in its landing branch's queue, the changes queued or first
  (`forge/change/README.md`, 5, *Its place in its queue*), with its
  place and its goal's priority, and `departed` once it is no longer;
  under `turn`, `joined` with its landing's first `formed` round and
  `departed` with the member's leaving (`landing.md`, 4.3).

## 3. Written by the step

### 3.1 The writer

One writer in the forge's top, `Domain::write_event`
(`temper-engine-domain-forge`), which every step calls:

```rust
pub(crate) fn write_event(
    &mut self,
    wall: Wall,
    change: u64,
    step: history::Step,
    out: &mut Queue<Request>,
) -> Option<u64>
```

- **`history::Step`** (the forge's vocabulary, `history` module): `what`,
  the kind and its fields as `history::What`, a variant per kind of 2.2;
  `landing`, an `Option<history::InLanding { task, round, unit }>`;
  `head` and `base`, each an `Option` of a commit; `operation`, a
  `history::Operation`, `Own` or `Of(u64)`; `attempt`, a `u32`; and
  `source_at`, an `Option<Wall>`.
- **What it does,** in the caller's decision: finds the change's row and
  its project (through its repository's adoption); takes the project's
  next number from its `HistoryProject` row; saves the `HistoryEvent`,
  its `HistoryChange` row and, under a landing, its `HistoryLanding`
  row; moves the change's `HistoryLast` row (erases the one its row's
  history names, saves the new one); saves the project's row with
  `next` and `kept` moved and its hour's mark (4.2); and saves the
  change's row with `history.last` set. It answers the number written.
- **A change with no row** has nothing written and answers none: every
  step that writes acts on a change's row, so this is a fault the
  tests show, never a guess.
- **Its rows are saved as every row of the forge is,** `Request::Save`
  and `Request::Erase` of `Stored` (`../protocol/store.md`, 2), in the
  same commit as the step's.

### 3.2 Once per step

- **On a transition, never on a reading:** an event is written by the
  decision that changes the stored state it records (the change's row,
  the landing's row, an outbox entry, a delegate's record), when that
  decision changes it. A decision that finds the state already holding
  the step (a retry, a rerun under the run's id, a lost answer found
  again, a restart restoring and relearning it) changes nothing, and so
  writes nothing (`landing.md`, 15: recovering an effect already made is
  no new step).
- **Where no row records the step,** the change's row keeps it: its
  `history` member, `ChangeHistory`, holds `last`, the number of its last
  event, and, for each interval no other state records, the number of
  its open start or none: `ready`, `queued` (its queue's `joined`),
  `paused`, `held` (a hold on the change) and `landing_held` (a hold on
  its landing). A start is written when its member is none and the step
  finds the interval open; its end when the member is set and the step
  finds it closed. A restart restores the member with the row, so
  relearning writes nothing.
- **Only what changed:** a poll or a hint that finds nothing new writes
  nothing.
- **Advancing steps in one decision** each write their event: a change
  that passes through `first` and saves `updating` in one decision
  writes `selected` and `update`, in that order.

### 3.3 A landing's steps, per member

A landing's step that concerns several members (`formed`,
`round-ended`, `gate`, `gate-ended`, `proved`, `search`, `searched`,
`probe`, `probe-ended`, `turn`, `landing-ended`, a hold on the landing)
writes one event for each member of the round it concerns, each with
the landing's task, the round and that member's unit, in the stack's
order. `formed` carries the round's members whole in each. A landing
that ends holding no member writes its `landing-ended` for the member
that left it last. So each change's events are its whole history,
swept with it (4.5), and a landing's are read through its index (4.1).

### 3.4 Times

- **`at`** is the wall time of the decision that wrote the event
  (`env.wall`): when temper decided the step, or observed one it does
  not make, such as a push or a run's end. Every split is computed on
  it.
- **`source_at`** is kept beside it where the source gives the step a
  time temper reads today: a run's creation, for `gate` (skein's
  `forge/client.md`). A run's start and end, its jobs', and a pull
  request's merge time are not read today (section 9).
- **Seen late is recorded late:** a run whose end is heard without its
  start having been recorded writes its `gate-ended` alone; no event is
  backdated or invented.

## 4. Kept

### 4.1 The families

Under temper's tag (`../protocol/store.md`, 3), by jig's key rule:

| Byte | Family | Key | What a row holds | Loaded |
|---|---|---|---|---|
| 127 | `HistoryProject` | `{ project }` | the project's history: `next`, `kept`, `whole_since`, `marks`, `sweeping`, `removing` (4.2) | restored with the forge's families |
| 128 | `HistoryEvent` | `{ project, number }` | the event (2.1) | on demand |
| 129 | `HistoryChange` | `{ project, change, number }` | its key's fields and `landing`, the event's landing task or none: the index of a change's events | on demand |
| 130 | `HistoryLanding` | `{ project, landing, number }` | its key's fields and `change`: the index of a landing's events | on demand |
| 131 | `HistoryLast` | `{ project, number, change }` | its key's fields and `at`, the event's: each change's last event, so the oldest is first | on demand |

- **Bytes 128 and above are demand-loaded families:** the restart's
  interval, `Range::Forge`, is the tag's keys whose family byte is below
  128 (`../protocol/store.md`, 5), so nothing of them is restored, and
  `Range::contains` admits them only for the history's own range (4.3).
- **Byte 127** sits below the demand-loaded bytes and apart from the
  next free byte, so a family another goal adds there meets no history
  family.
- **The change's row** gains `history`, an option of `ChangeHistory`
  (3.2), at the end of its record.
- **Each event once:** a change's events are its `HistoryChange` rows,
  their `HistoryEvent`s and, under a landing, their `HistoryLanding`
  rows, so the index row alone names every key the event's sweep
  erases.

### 4.2 The project's row

- **`next`:** the number the next event takes, from 1; **`kept`:** how
  many of the project's events the store holds.
- **`whole_since`:** the `at` of the last event of the newest change the
  sweep took, or none: every change whose last event is later is whole.
  `landing history` says it.
- **`marks`:** for each wall hour in which the project wrote an event,
  the hour's start and the first number written in it, at most 2,048,
  the oldest dropped first and those older than the oldest kept event
  dropped by the sweep. A read from a time starts at its hour's mark
  (5.1).
- **`sweeping`:** the change whose events the sweep is erasing over
  several decisions, or none; reads of that change answer it gone.
- **`removing`:** the project's deletion is sweeping it (4.5); its row
  is erased once its events are.

### 4.3 Loaded on demand

The forge asks the root for its rows, as jig's notes child asks for its
own (`root.md`; `../protocol/store.md`, 5), and the root asks the store
with its application intervals (jig's `protocol/store/README.md`, 7):

- **The forge's request,** `Request::LoadHistory { load, range, after,
  most }`: `load` the forge's own number for it; `range` a
  `HistoryRange`; `after` the last key the previous page returned, the
  page starting past it, or none; `most` the rows wanted, within the
  root's `loads.rows`.
- **`HistoryRange`:** `Events { project }` (byte 128 in number order),
  `Change { project, change }` (129), `Landing { project, landing }`
  (130), `Last { project }` (131), and `Numbers { project, numbers }`,
  the events with those numbers, at most 32, one interval each.
- **The answers:** `Event::HistoryLoaded { load, rows, next }`, the rows
  as `Stored` in key order and the cursor to go on from, none at the
  range's end; or `Event::HistoryUnloaded { load }` where the root
  could not take or finish the load. A late answer to a load the forge
  no longer waits on is dropped.
- **The root's range** is `Range::History(HistoryRange)`, routed to
  jig's `Load::Application` with the range's intervals; its pages,
  cursors and bounds are the root's loads' (`loads.rs`).

### 4.4 Read, never restored

- **The forge keeps in memory** each project's `HistoryProject` row and,
  on each change's row, its `history` member: never an event.
- **A restart restores** those rows with the forge's families; a sweep
  or a read in flight is dropped and begun again by its next trigger
  (the hourly check, the client's read again).

### 4.5 Swept

- **By age:** once a change's last event is older than `history_days`,
  its events go together.
- **By count:** while `kept` is above `history_events`, the change whose
  last event is oldest goes, whole, whatever its age.
- **Whole changes, oldest first:** the sweep loads the first
  `HistoryLast` row of the project (`Last { project }`, one row): its
  change, its last number and `at`. If it is due, it loads that
  change's `HistoryChange` rows, a page of at most 32, and erases, in
  one decision, each one with its `HistoryEvent` and `HistoryLanding`
  rows; on the change's last page it erases the `HistoryLast` row it
  loaded, sets `whole_since` to that `at` where later, and goes on with
  the next `HistoryLast` row. A change of more than a page is
  `sweeping` until its last page is erased. `kept` moves with every
  erasure.
- **When:** after a write that leaves `kept` above `history_events`,
  and each hour on the forge's own deadline for every project with a
  row, so an idle project's events still go. One sweep at a time per
  project.
- **A change written during its sweep:** its new event's `HistoryLast`
  row is the writer's, which the sweep, erasing only the row it loaded,
  leaves; what remains is a history from that event on.
- **A project's deletion** (`forge/adoption.md`, 1, *A project
  deleted*) sets its row's `removing`; the sweep then takes every change
  of `Last { project }` whatever its age, and erases the row once its
  last change is gone, so a deletion a restart cut goes on.

## 5. Read

### 5.1 `landing history`

`temper ctl landing history --project PROJECT [--change TASK | --landing
TASK] [--since TIME] [--until TIME] [--after CURSOR] [--json]`, a read
of the forge's family (`../client/ctl/tools.md`, 2):

- **Its order:** the project's events by number; with `--change` or
  `--landing`, that change's or landing's, through its index (4.1), each
  page's events then loaded by `Numbers`. So a filtered read does
  bounded work.
- **A page:** at most 32 events, the rows' bound in ctl
  (`../client/ctl/README.md`, 9). A page examines at most 256 index or
  event rows; one that reached that bound with fewer events comes back
  cut, with its cursor, as jig's pages do.
- **Its upper bound:** the first page takes the project's last number,
  `next` less one, and the cursor carries it, so later pages read the
  same history; no event past it is listed.
- **Its range:** `--since` and `--until`, as `change show` takes them,
  keep the events whose `at` is in the half-open range; an order by
  number reads from the mark of `--since`'s hour (4.2), and stops at the
  first event at or past `--until`.
- **The cursor:** 17 bytes, a version byte, 1, then the upper bound and
  the last number returned, each eight bytes big-endian, printed as
  lowercase hexadecimal as ctl's cursors are.
- **A cursor past what remains is refused:** a page whose cursor's last
  event has gone (swept since the page before, or the change
  `sweeping`) answers exit 2, `temper ctl: history from event N is gone;
  the first event that remains after it is M`, or `none remains after
  it`, naming the read again from M; nothing is skipped silently.
- **Its answer:** a header line, `project P: events through UPPER`, then
  `, whole since TIME` where the sweep took any; an event a line,
  `NUMBER AT KIND change TASK`, then `landing L round R unit U` where it
  has them, its head and base each as 12 hexadecimal digits, and its
  kind's fields by name; then the next page's command, or `end`. With
  `--json`: `{"events": [...], "after": CURSOR or null, "upper": N,
  "whole_since": TIME or null}`, each event every field of 2.1 by name,
  its kind as 2.2 names it and its fields as its members.
- **On the wire:** a forge read of the client's family, as
  `RevisionSetting` is (`../client/forge/served.md`), its query the
  project, the change or the landing, the range and the cursor, its
  answer the page or the refusal; the client's wire moves its version
  (`../protocol/wires.md`, 8). The root hands the read to the forge,
  which loads (4.3) and answers it.

### 5.2 A change whose row went

- **`change show TASK`,** where today it answers `no change panel for
  task N` (`forge/change/README.md`, 7, *A dropped change reads as an
  unknown task*), reads the change's history, `--change TASK` in its
  task's project, every page to its end, at most 16; prints `change N:
  its record has gone; its last events:`, the last 8 events as 5.1
  prints them, and `all of them: temper ctl landing history --project P
  --change N`; exit 0. With `--json`: `{"task": N, "record": "gone",
  "events": [...], "read": "COMMAND"}`.
- **Its events gone too:** `change N: its record and its history have
  gone`, exit 2; with `--json`, `{"task": N, "record": "gone",
  "history": "gone"}`.
- **A wait** (`change show`'s wait conditions) that finds the record gone
  answers as it does today.

### 5.3 Who reads

- **The project's readers,** as for its changes, whether or not the
  change's task remains: the forge read is admitted by jig's check of the
  party's role in the project (jig's `client/domain/contract/README.md`,
  section 4).
- **The public projection shows none of it** (`../client/forge/README.md`,
  9.3).

## 6. Settings and bounds

- **`forge.history_days`,** stock 14, from 1 to 3,650, and
  **`forge.history_events`,** stock 131,072, at least 1, the same in
  both profiles: deployment-wide limits, each applying to every project
  (`../protocol/configuration/limits.md`). Twenty goals in flight write a
  few thousand events a day, so the count holds about the fourteen days.
- **A record** is bounded by its fields' bounds and a `formed` event's
  members by `forge.landing_members`; the families' largest key and
  value join the store's sizes (`../protocol/store.md`, 4).
- **The demand-loaded families** take no forge memory: only the
  projects' rows and the changes' `history` members are held, each
  bounded by its row's bounds.

## 7. Upgrade

- **A new stored layout,** with its version and patch, in the landing
  that adds it (`../protocol/store.md`, 4; `../protocol/baseline.md`,
  7): temper's patch adds the five families' records and `ChangeRow`'s
  `history`, none for every row kept, and moves `STORE_VERSION`. The
  restart's interval narrowing changes no stored row.
- **No history before it:** a change live at the upgrade has a partial
  history, from its first step after; nothing is rebuilt or backdated.
- **ctl's wire** moves its version for the new read
  (`../protocol/wires.md`, 8); the client's wire carries no bridge.

## 8. Testing

Focused and fuzzy suites alone, each test failing first:

- **The store's step tests:** each family's key against the derived
  order and its decoder; `Range::Forge`'s interval holding no key of
  bytes 128 to 131 and every other forge key; the records' goldens and
  the patch's.
- **The forge's world** (`../testing/README.md`; `landing.md`, 16): each
  kind written by its step, under `queue` and under `turn`, in the
  decision that decides or observes it, with both ends of each interval;
  a restart at each step, a retried job, a run rerun under its id and a
  run relearned after a restart writing no second event; a run's end
  heard before its start was recorded written alone; a change's and a
  landing's rows evicted and their events kept and read; age and count
  sweeping whole changes, oldest first, an idle project's included, and
  a project's deletion sweeping it whole.
- **The read:** `landing history` by project, change, landing and range,
  pages under a fixed upper bound, a cursor past what remains refused;
  `change show` of a change whose row went, and of one whose history
  went too; a reader outside the project refused.
- **Fuzzy:** random step sequences in the forge's world keep each
  change's events ordered and well formed: each interval's start before
  its end, each `asked` answered or open, `landed` or `ended` at most
  once, no two events of a change with one kind and identity.
- **The process tier's queue stories** (`../draft/merge-queue.md`, 12)
  read their landing's history at the end: written and compiled, run
  after landing.

## 9. Later

- **Where the time went,** `landing times`: the draft's section 7, its
  execution's H2, reading only what this document writes.
- **Source times:** a run's start and end and its jobs', and a pull
  request's merge time, read by the forge client and kept as
  `source_at`.
- **A gate's own steps**, the client's view of a timeline, and a
  project's figures over time (`../draft/landing-history.md`, 11).
