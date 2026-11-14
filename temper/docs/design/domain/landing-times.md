# Landing times

Provisional, 2026-11-13. Where each change's time went, from ready to
landed or ended: `temper ctl landing times` splits it into states that
don't overlap, computed by code from the events landing history keeps
(`landing-history.md`) and from nothing else; under `turn`, each
landing's rounds follow its changes. It is the draft's milestone H2
(`../draft/landing-history.md`, sections 7 and 10; its execution's H2),
reading only what H1 writes. Nothing is stored: the same events under
the same bound give the same answer at every read.

## 1. In one page

- **A change's span** (2.1): from its first `ready` to its `landed` or
  `ended`, or to the read's bound, marked open; a change whose history
  began after it was ready is partial, its span from its first event
  (2.4).
- **One state at a time** (2.2, 2.3): at each instant of its span the
  change is in the first state whose interval holds, in the order held,
  paused, resolving, repairing, judging, probing, proving, preparing,
  delivering, turning, waiting, and other where none does; each
  interval half-open on the events' times, the span clipped to the
  read's range.
- **A fold** (2.5): a change's events go through it one at a time in
  number order, and it keeps only its open intervals, so a change of any
  length is split in bounded memory.
- **A landing's rounds** (section 3): each round's members, its proof's
  and its search's times and its probes, and the landing's sums, from
  the landing's own events.
- **The read** (section 4): the changes whose spans meet the range, in
  the order a scan of the project's events first meets them, at most 32
  a page, each whole, then the landings they were members of; the
  running totals per state ride in the cursor, and the last page gives
  the range's.
- **ctl** (section 5): `landing times`, a line per change and per
  landing, the totals, then the next page's command.

## 2. A change's split

### 2.1 Its span

- **Its events:** the change's, in number order, those numbered at most
  the read's upper bound (4.3). An event's time is its `at`, or the
  latest `at` before it in the change's order where that is later, so
  time never goes back within a change.
- **Its start:** its first `ready`'s time; for a partial change (2.4),
  its first event's.
- **Its end:** the time of its first `landed`, or of its first `ended`,
  with that event's `ending`; with neither, the read's upper time (4.3),
  and the change is open. Events after its end (`recorded`, `after`, a
  round's or a landing's end) change no state.
- **No span:** a change with no `ready` that isn't partial was never
  ready in its history: it has no span and is not listed.
- **The range:** `[since, until)` on time, `since` none for a read
  without `--since`, and `until` the earlier of `--until` and the read's
  upper time. The change is listed when its span meets the range: its
  start is before `until` and its end after `since`; a span of no length
  is listed when its start is in the range.
- **What it gives:** `from` and `to`, its start and its end, not clipped;
  how it ended; whether it is partial; and each state's time within the
  span clipped to the range.

### 2.2 Its intervals

Each runs from an event to another, half-open, and every one also ends
at the span's end. A change's *round* is the event's `landing`: the
landing's task and the round.

| Interval | From | Until |
|---|---|---|
| ready | `ready` | the next `unready` |
| selected | `selected` | the next `unready` or `departed` |
| in a round | `formed` | that round's `round-ended`, or a `departed` from the `landing` line |
| held | `held`, on the change or on its landing | the `released` whose `operation` is the `held`'s number |
| paused | `paused` | the `resumed` whose `operation` is the `paused`'s number |
| resolving, repairing, judging | `asked`, by its `ask` | the `answered` with its `operation` |
| probing | `search` | that round's `searched` or `round-ended` |
| proving | `gate` | the `gate-ended` with its `operation` and `attempt`, or its round's `round-ended` |
| preparing | `update`; `job` | the `updated` with its `operation`; the `job-ended` with its `operation` and `attempt`, or, for a job in a round, its round's `round-ended` |
| delivering | `pushed`, of a head not temper's own (below) | the next `delivered`, `pushed` or `ready` |
| turning | `proved`; under `queue`, `turn` | `landed`, or, after `proved`, its round's `round-ended` |
| waiting | while ready, neither selected nor in a round | |

- **Temper's own head:** a `pushed` while an update is open (an `update`
  without its `updated`), or of the head of the change's last `updated`,
  `job-ended` or `delivered`, opens nothing: it is temper's own update
  or job pushing, or a head a delivery already brought, not one waiting
  to be accepted.
- **Waiting under each way:** under `queue`, a ready change is joined to
  its queue and waits until it is `selected`; under `turn`, it is joined
  only with its landing's first `formed` round (`landing-history.md`,
  2.4), so it waits while ready and in no round: before its landing
  forms, and between its rounds.
- **Proving under `queue`:** landing history writes `gate` only for a
  run at a round's top or at a probed prefix (`landing-history.md`,
  2.2), so a `queue` change's checks at its own head make no interval,
  and their time is whatever else holds then, most often other.
- **An end without its start** opens nothing and closes nothing: a run
  seen late writes its `gate-ended` alone (`landing-history.md`, 3.4),
  and nothing is backdated. A start whose end never comes holds until
  the span ends.

### 2.3 One state at a time

- **The order:** held, paused, resolving, repairing, judging, probing,
  proving, preparing, delivering, turning, waiting. At each instant of
  the clipped span, the change is in the first of these whose interval
  holds, and in other where none does.
- **Pieces:** every interval starts and ends at an event, so the
  clipped span is cut at its events, and at `since` and `until`, into
  pieces of one state each; a state's time is the sum of its pieces, in
  nanoseconds.
- **Equal times:** events apply in number order. A piece between two
  events at one time is empty, so the order decides only which
  intervals are open after them.
- **The sum:** the states' times sum to the clipped span's length; a
  whole history read with no range gives states summing to the time
  from its start to its end.

### 2.4 Partial

- **A history begun after the change was ready:** a change live at the
  upgrade (`landing-history.md`, 7) has no `ready` before events only a
  ready change writes. A change whose history holds, before its first
  `ready` or with none, any of `unready`, `selected`, `job`,
  `job-ended`, `formed`, `round-ended`, `gate`, `proved`, `search`,
  `searched`, `probe`, `probe-ended`, `turn`, `landed`, `recorded` or
  `after` is partial.
- **Its span** starts at its first event, and it is taken as ready from
  there until its first `unready` or `ready`: the readiness its own
  events show. Every other interval needs its start in the history, as
  in 2.2.
- **Not partial:** a change whose history begins with `pushed`,
  `delivered`, `held`, `conflict`, `asked` or another step a change
  takes before it is first ready begins whole, and its span starts at
  its first `ready`.
- **In the fold:** until it meets its first `ready` or one of the kinds
  above, the fold (2.5) counts the pieces since the change's first
  event as a partial change's, readiness taken, apart from its states:
  a `ready` first drops them and starts the span there; one of the kinds
  first keeps them and marks the change partial. So the fold needs no
  event ahead of the one it is given.

### 2.5 The fold

`history::times` in `temper-engine-domain-forge-vocabulary`, re-exported
through the forge's `history` module, pure and in the strict subset:

```rust
pub struct Bound { pub since: Option<Wall>, pub until: Option<Wall>, pub upper: Wall }
pub enum State { Held, Paused, Resolving, Repairing, Judging, Probing, Proving, Preparing, Delivering, Turning, Waiting, Other }
pub struct States { pub held: u64, pub paused: u64, pub resolving: u64, pub repairing: u64, pub judging: u64, pub probing: u64,
    pub proving: u64, pub preparing: u64, pub delivering: u64, pub turning: u64, pub waiting: u64, pub other: u64 }
pub enum End { Open, Landed, Ended(Ending) }
pub struct Times { pub change: u64, pub repository: Repository, pub from: Wall, pub to: Wall,
    pub end: End, pub partial: bool, pub states: States }
pub struct Piece { pub state: State, pub from: Wall, pub to: Wall }

pub struct Split { /* its open intervals, bounded */ }
impl Split {
    pub fn new(bound: Bound) -> Self;
    pub fn push(&mut self, event: &Event);
    pub fn finish(self) -> Option<Times>;
}
pub fn timeline(events: &[Event], bound: Bound) -> Box<[Piece]>;
```

- **`Bound`:** `upper` is the read's upper time, where an open span
  ends; the range is `[since, until)` with `until` taken as the earlier
  of `until` and `upper` (2.1).
- **`Split`:** takes one change's events in number order and answers its
  `Times` where its span meets the range, none otherwise. It holds its
  open intervals and the time of its last event, never an event: what
  it keeps is bounded by what landing history can hold open at once (a
  round's runs, a change's delegates and jobs), each in a `List` of a
  fixed capacity; a start past that capacity is dropped, its interval
  not counted.
- **`States`:** a field per state, with `get(State)` and `add(State,
  u64)`, saturating; `STATES` is the order of 2.3, other last.
- **`timeline`:** the same split as pieces of one state, in order, for
  tests and for a reader holding a whole history: the sum of its pieces
  per state is `finish`'s `states` for the same events and bound, and
  it is empty where `finish` answers none.

## 3. A landing's rounds

### 3.1 What it gives

From the landing's events, every member's (`landing-history.md`, 3.3),
each copy of a round's event counted once:

- **The landing:** its task; its outcome and time from its first
  `landing-ended`, none while open; `formed`, its first round's
  `formed`; `rounds`, how many rounds it formed; `proving`, the sum over
  its rounds of each round's time from its `formed` to its `proved`, or
  to its end where it never proved; `probing`, the sum of each search's
  time from `search` to `searched`, or to its round's end; and
  `probes`, how many prefixes it probed.
- **Its last rounds,** at most 8, the latest: the round's number; its
  `formed` and its members from it, each its change and unit, in the
  stack's order; its end, the time of the latest `round-ended` of the
  round among its members, and that event's `cause`, none while open;
  its `proved`, `search` and `searched` times where they were written;
  and its probes, the prefixes probed in it.
- **Copies:** a round's `formed`, `proved`, `search` and `searched` are
  taken from the first copy, its end from the latest (a cut ends some
  members' round before the rest's, `landing-history.md`, 3.3), and a
  probe is counted once per prefix, by its head.
- **Times:** as a change's (2.1), never back within the landing's
  events; a round or a search still open runs to the read's upper time.

### 3.2 The fold

```rust
pub struct RoundMember { pub change: u64, pub unit: u16 }
pub struct Round { pub round: u16, pub formed: Wall, pub members: Box<[RoundMember]>, pub ended: Option<Wall>,
    pub cause: Option<RoundCause>, pub proved: Option<Wall>, pub search: Option<Wall>, pub searched: Option<Wall>,
    pub probes: u16 }
pub struct LandingTimes { pub landing: u64, pub outcome: Option<LandingOutcome>, pub formed: Wall,
    pub ended: Option<Wall>, pub rounds: u32, pub proving: u64, pub probing: u64, pub probes: u32,
    pub last: Box<[Round]> }

pub struct LandingFold { /* the last rounds, bounded */ }
impl LandingFold {
    pub fn new(landing: u64, upper: Wall) -> Self;
    pub fn push(&mut self, event: &Event);
    pub fn finish(self) -> Option<LandingTimes>;
}
```

- **`push`** takes the landing's events in number order and ignores
  those of another landing and the kinds section 3.1 doesn't read;
  `finish` answers none where no `formed` was seen.
- **Bounded:** at most 8 rounds kept, the oldest dropped as a newer one
  forms, its times already in the sums; at most 8 members a round
  (`forge.landing_members` is at most 5); at most 8 prefixes a round
  remembered for counting probes.

## 4. The read

### 4.1 Its query

`history::times::Query { project, since, until, after }`: the project,
`since` and `until` as `Option<Wall>`, and the cursor of 4.5, none for
the first page. A later page repeats the first page's `since` and
`until`.

### 4.2 What it lists, and in what order

- **The changes whose spans meet the range** (2.1), each listed once,
  where a scan of the project's events by number from a lower bound
  first meets one of its events. The order is that scan's: a change at
  its first event at or after the lower bound.
- **The lower bound** is fixed by the first page: the first number of
  `--since`'s hour, from the project's marks as `landing history` reads
  them (`landing-history.md`, 4.2 and 5.1), or 1 without `--since`;
  lowered to the `history.last` of every change of the project whose
  row is live, has `history`, and has neither landed nor ended. So a
  change open across the whole range, with no event in it, is met at
  its last event.
- **The scan runs to the upper bound,** not to `--until`: a change
  ready across a past range that ended after it is met at its end.
- **Each change whole:** a change met at its first event at or after the
  lower bound has its history read through its index
  (`landing-history.md`, 4.1 and 4.3), every event numbered at most the
  upper bound, each page of index rows then its events by `Numbers`,
  through a `Split` (2.5); one met at a later event was met before, and
  is passed over. Whether an event is its change's first at or after
  the lower bound is read from the change's index, its first row at or
  after that number (a load of one row), and remembered within the
  page.
- **A change being swept** (`landing-history.md`, 4.2, `sweeping`) is
  passed over.
- **Its landings:** each landing the listed change's events name
  (`landing`), once a page, read whole through its index up to the
  upper bound and folded by a `LandingFold` (section 3), after the
  page's changes, in the order the page first names them.

### 4.3 The bound

- **The upper bound,** fixed by the first page: `upper`, the project's
  last event number (`next` less one), and `upper_at`, the wall time of
  the decision that begins the first page. No event past `upper` is
  read; an open span ends at `upper_at`; `until` is the earlier of
  `--until` and `upper_at`.
- **Nothing after it moves the answer:** a later page reads the same
  events, so the same events under the same bound give the same
  answer.

### 4.4 A page

- **At most 32 changes and 32 landings,** and at most 4,096 rows
  examined: the scan's, the first-row checks', and the histories'
  index and event rows together. A change or landing being read when
  the rows run out is finished whole first; a page that reaches the row
  bound with fewer changes comes back cut, with its cursor, as
  `landing history`'s pages do.
- **A change whose landings would take the page past 32** ends the page
  before it, the scan's position left at the event before the change's,
  so the next page meets it first.
- **The last page** is the one whose scan reaches the upper bound: it
  has no cursor, and its totals are the range's.
- **Reads at once:** at most 4 of these reads in flight in the forge,
  beside `landing history`'s; one past that, or one whose load the root
  could not take, answers busy, as `landing history` does.

### 4.5 The cursor

145 bytes, each number big-endian: a version byte, 1; `upper` (8);
`upper_at` (8); the lower bound (8); `from` (8), the `at` of the first
event the first page's scan read; the scan's position (8), the number
of the last event it examined; then the running totals: the changes
listed (4), the partial among them (4), and each state's time (8 each,
in the order of 2.3, other last). ctl prints it as lowercase
hexadecimal. A cursor that doesn't decode is refused as a malformed
read.

- **History swept under the read:** the sweep takes whole changes,
  oldest first, and moves the project's `whole_since` to the last event
  of the newest it took (`landing-history.md`, 4.2 and 4.5). A later
  page whose project's `whole_since` is at or after the cursor's `from`
  may have lost a change it would list, and answers gone: the read
  begins again without a cursor. One whose `whole_since` is earlier lost
  nothing it reads.

### 4.6 Its answer

```rust
pub struct Cursor { pub upper: u64, pub upper_at: Wall, pub lower: u64, pub from: Wall, pub position: u64, pub totals: Totals }
pub struct Totals { pub changes: u32, pub partial: u32, pub states: States }
pub struct Page { pub changes: Box<[Times]>, pub landings: Box<[LandingTimes]>, pub totals: Totals,
    pub after: Option<Cursor>, pub upper: u64, pub upper_at: Wall, pub whole_since: Option<Wall> }
pub enum Reply { Page(Page), Gone, Busy }
```

`Cursor::encode` and `Cursor::decode` keep 4.5's bytes. A page's
`totals` are the running totals through it; `whole_since` is the
project's, as `landing history` gives it.

### 4.7 On the wire

A forge read of the client's family, as `History` is
(`landing-history.md`, 5.1), in `client-wire-v1.schema`, the family's
version moving with it (`../protocol/wires.md`, 9):

```
# Landing times (temper/docs/design/domain/landing-times.md, 4).
record TimesRead { project: u32 since: option u64 until: option u64 after: option fixed 145 }
record TimesStates { held: u64 paused: u64 resolving: u64 repairing: u64 judging: u64 probing: u64 proving: u64 preparing: u64 delivering: u64 turning: u64 waiting: u64 other: u64 }
enum TimesEnd { open, landed, ended: HistoryEnded }
record TimesChange { change: u64 repository: HistoryRepository from: u64 to: u64 end: TimesEnd partial: bool states: TimesStates }
record TimesMember { change: u64 unit: u16 }
record TimesRound { round: u16 formed: u64 members: list 8 TimesMember ended: option u64 cause: option HistoryRoundCause proved: option u64 search: option u64 searched: option u64 probes: u16 }
record TimesLanding { landing: u64 outcome: option HistoryLandingOutcome formed: u64 ended: option u64 rounds: u32 proving: u64 probing: u64 probes: u32 last: list 8 TimesRound }
record TimesTotals { changes: u32 partial: u32 states: TimesStates }
record TimesPage { changes: list 32 TimesChange landings: list 32 TimesLanding totals: TimesTotals after: option fixed 145 upper: u64 upper_at: u64 whole_since: option u64 }
enum TimesReply { page: TimesPage, gone, busy }
versioned record TimesAnswer { result: TimesReply }
```

- **`Read`** gains `times: TimesRead`, after `history`; times are
  nanoseconds since the epoch, as `History`'s are.
- **The crate's functions,** `times_answer` and `decode_times_answer`,
  as `history_answer` and `decode_history_answer` are.
- **No bridge:** the client's wire moves its version alone
  (`../draft/redeploys.md`, 8).

### 4.8 In the root and the forge

- **The forge:** `Event::ReadTimes { read, query }` begins a read, and
  `Request::TimesPage { read, answer }` answers it; its loads are
  `Request::LoadHistory`'s, answered as `landing history`'s are
  (`landing-history.md`, 4.3).
- **The root:** `ForgeRead::Times(Query)` and
  `ForgeReadAnswer::Times(Reply)`, admitted and refused as
  `ForgeRead::History` is.
- **The service's family** decodes `TimesRead` into the query and
  encodes the reply, as it does `History`'s.

### 4.9 Who reads

The project's readers, as for `landing history` (`landing-history.md`,
5.3); the public projection shows none of it.

## 5. `landing times`

`temper ctl landing times --project PROJECT [--since TIME] [--until
TIME] [--after CURSOR] [--json]`, a read of the forge's family
(`../client/ctl/tools.md`, 2), TIME as `landing history` takes it:

- **Its header:** `project P: events through UPPER at TIME`, then
  `, whole since TIME` where the sweep took any.
- **A change a line:** `change C: FROM to TO landed`, or `ended ENDING`,
  or `open`; `, partial` where it is; then `; ` and each state with time
  in the range, in the order of 2.3, other last, as `STATE DURATION`
  joined by `, `, or `none in the range`.
- **A landing a line,** after the changes: `landing L OUTCOME: FORMED to
  ENDED`, OUTCOME `done`, `cancelled` or `held`, or `landing L open:
  FORMED`; then `, R rounds, proving DURATION, probing DURATION, N
  probes`. Its last rounds follow, each indented two spaces: `round R:
  FORMED to ENDED CAUSE` or `FORMED open`, `, members C/U C/U`,
  `, proved TIME` or `, not proved`, and, where it searched, `, search
  FROM to TO` (or `FROM open`) `, N probes`.
- **Its totals:** `so far, N changes (P partial): ` and the states as a
  change's, on a page with a next; `totals, N changes (P partial): ` on
  the last.
- **Then** the next page's command, the query repeated with `--after
  CURSOR`, or `end`.
- **A duration** prints in whole seconds, rounded down, as hours,
  minutes and seconds, each part left out when zero (`1h28m5s`, `40m`),
  `0s` for none. A time prints as `landing history` prints one.
- **With `--json`:** `{"changes": [...], "landings": [...], "totals":
  {...}, "after": CURSOR or null, "upper": N, "upper_at": T,
  "whole_since": T or null}`: each change with `change`, `repository`
  (`forge`, `repository`), `from`, `to`, `end` (`landed`, `ended` or
  `open`), `ending` (or null), `partial` and `states`, an object of each
  state by name; each landing with every field of 3.2 by name, its
  rounds in `last`; times and durations in nanoseconds.
- **Gone:** exit 2, `temper ctl: history this read needs was swept after
  its first page; read it again without --after`. **Busy** answers as
  `landing history`'s does.

## 6. Upgrade

- **Nothing stored:** no family, record or patch; the read takes
  whatever events remain.
- **The client's wire** moves its version for the new read (4.7).
- **A change live at H1's upgrade** reads partial (2.4).

## 7. Testing

Focused and fuzzy suites alone, each test failing first:

- **The split** (2.5), from hand-built events, each case's pieces and
  states as stated: a change landed green at once under `queue` and
  under `turn`; one through a resolution; one held; one repaired; one
  cancelled while ready; one ready across the range's start, clipped;
  one live at the upgrade, partial; a run superseded; equal times;
  temper's own head; and a `turn` landing whose red round was searched,
  with its landing's rounds, probes and sums (section 3).
- **The cursor:** its 145 bytes both ways, a short or unknown version
  refused.
- **The read,** in the forge's world: changes listed in the scan's
  order, each once; a page cut at 32 changes, and at 32 landings, and
  the totals across pages equal to the sum of the changes'; events
  written after the first page counted on no later page; the range
  clipping a change; a change open across the range with no event in
  it, and one ready across a past range that ended after it, both
  listed; a sweep under the read answered gone; a fifth read busy; a
  reader outside the project refused.
- **The wire and the client:** the family's version and records; the
  service's decoding and encoding; ctl's lines, its JSON, the next
  page's command, gone and busy.
- **Fuzzy:** the forge's world's random step sequences
  (`landing-history.md`, 8) also split each change: for a whole
  history, with no range, its pieces in order and adjoining, and its
  states summing to its span's length; `timeline`'s sums equal to the
  fold's.

## 8. Later

- **Source times** for the split, once landing history keeps them
  (`landing-history.md`, 9).
- **A queue change's checks at its head** as proving, once landing
  history writes their runs (2.2).
- **A project's figures over time,** landings per hour and the share of
  time in each state by day (`../draft/landing-history.md`, 11).
