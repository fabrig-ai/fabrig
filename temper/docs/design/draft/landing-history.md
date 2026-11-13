# landing-history: temper keeps how each landing went

temper keeps what a change and a landing are now, not how they got there. A change's step and head are current, and the time beside them moves with each step; a landing keeps its current round, the next overwriting it; once either has ended, its row goes as soon as another needs the room (`../domain/forge/change/README.md`, 7; `../domain/landing.md`, 12), and a change landing by `queue` makes no landing row at all (`../domain/landing.md`, 14). So what a project asks of its landings, where the time from ready to landed went and what held it, is answered today only by rebuilding each timeline by hand from the forge's pull request timelines, the runner's logs and the agents' tasks, with the queue's own steps inferred between them. This draft has temper keep each change's and each landing's steps as events, each interval with an event at both ends, written by the commit that decides or observes each step, kept by change for a bounded time after the change and the landing are gone, and read through `ctl`, with the split of each change's time computed by code. temper/dev builds it.

## 1. In one page

- **An event per step, at both ends of each interval** (section 3): readiness and membership kept apart; a queue's selection; updates and jobs requested and settled; conflicts; a member leaving and why; resolutions and repairs asked and answered; deliveries pushed and accepted; gate runs, proofs and the search; rounds formed and ended; the turn; the change landed, its pull request recorded, its terminal outcome; holds; the verdict after landing.
- **Written where the step is decided or observed,** in the same commit, each with an identity, so a restart or a retry neither loses nor repeats one (section 3).
- **temper's time, and a source's where the source already gives one** (section 4). A time temper never saw is never invented.
- **Both ways of landing** (section 3): `queue` and `turn` write the same kinds where they share a step.
- **Kept by change** (section 5): a change's events go together, once its last is older than `history_days`, or the oldest changes first past `history_events`. They outlive the change and landing rows, and are read on demand, never restored into memory.
- **Read by `ctl`** (sections 6 and 7): `landing history` lists events; `landing times` splits each change's time from ready to its end into states that don't overlap, by code, from the events alone, and marks a change whose history is incomplete.
- **A mechanism:** code writes and reads it. No agent writes the history, and nothing in it is judged.

## 2. Where it starts

- **Kept today, as state:**
  - a change's first ready time, its step, its head and the time of its last step, replaced at the next (`../domain/forge/change/README.md`, 2);
  - its updates, repairs and resolutions as counts, not times;
  - a landing's members, their heads and why each left, without when, and its current round only (`../domain/landing.md`, 12);
  - each gate's receipts by key, with the time temper recorded the verdict, a few per gate (`../domain/landing.md`, 6.2);
  - the landing log, the recent landings per repository, a ring (`../domain/after-landing.md`, 10).
- **Kept elsewhere:** jig keeps each task's activity, attempts, milestones and holds with their times after the task ends: the agents' side of resolutions and repairs, not the change's or the landing's steps. The forge keeps pushes, merges and runs with its own times; the runner keeps a gate's inner steps in its logs.
- **What goes with the rows:** the forge's limits evict ended changes and ended landings as room is needed (`../protocol/configuration/limits.md`): a change read after that has no panel, and an ended landing no row.
- **Never kept:** when a change became ready again, joined a queue or a landing, was taken by it, was updated, left it and why; when a job ran; when a delivery was accepted; when a gate's outcome was seen; when the turn was taken. These are the boundaries a timeline rebuilt by hand can only infer.
- **What the forge client reads:** a pull request without its merge time, a run with its creation time alone, a run's jobs without their start and end.

## 3. The events

- **An event** is a row: its project, its number in the project's order (a counter that never goes back, even after every event has gone), its time (4), the change's task, its landing's task, round and unit where it has them, the heads and base it concerns, its kind and the kind's fields, and its **identity**: the operation it records, as the step that writes it names it (a run's id and attempt, a job's task and attempt, a round's number, a delegate's task), with its phase.
- **Its kinds,** each interval's start and end both written:

| Kind | Written when | Its fields |
|---|---|---|
| `ready`, `unready` | a change becomes ready (`../domain/landing.md`, 3), or ceases to be: held, its head moved, a gate refused, a dependency not kept, an approval lapsed | head; why |
| `joined`, `departed` | it joins its queue or a landing, or leaves either, under `queue` and `turn` (`../domain/forge/queues.md`; `../draft/merge-queue.md`, 4.2 and 4.3); a forge's own merge queue admitting or ejecting it | its place and priority; cause |
| `selected` | under `queue`, it is taken as the first to land | — |
| `update`, `updated` | an update of its branch from the base is asked, and settled | base; the new head, or the conflict |
| `job`, `job-ended` | a prepare or merge job is dispatched, and its result heard, under either way | job's task, kind and attempt; outcome |
| `conflict` | a conflict is found | base; with the branch, or with an earlier member; how many files |
| `asked`, `answered` | a resolution, repair or judgement is asked of the change, and ends | the task, the reason; its outcome, the head it pushed |
| `pushed`, `delivered` | a head reaches the change's branch, and temper accepts it | head; checked or an unchecked save |
| `paused`, `resumed` | its CI is paused or resumed (`../domain/forge/change/README.md`, 2, step 3) | why |
| `formed`, `round-ended` | a landing forms or a round starts; a round ends | landing, round, base, its units and their members in order with their heads; the cause: turned, a member left, stale, cancelled |
| `gate`, `gate-ended` | a gate's run is dispatched or seen running; its outcome is heard | gate, key, head or prefix, run and attempt; verdict, a runner's fault and its retry, or superseded |
| `proved` | a candidate or prefix is proved, by runs or by receipts already held (`../domain/landing.md`, 7.2) | the prefix; each gate's receipt or run |
| `search`, `searched` | the search by prefixes starts, and holds its boundary (`../domain/landing-search.md`, 2 to 4) | round; the green and red prefixes, the blamed unit |
| `probe`, `probe-ended` | a probe runs at a prefix, and its outcome is heard | prefix; outcome |
| `turn` | the branch moves to a proved prefix, or a `queue` change's merge is made by temper | prefix or head, members landed |
| `landed` | the change has landed: its candidate is the branch's, by containment or by merge | commit |
| `recorded` | the forge records its pull request merged, or closed for a squash | the pull request's state |
| `ended` | the change ends otherwise: cancelled, no change, failed for good | outcome |
| `landing-ended` | a landing ends, done, cancelled or held | outcome |
| `held`, `released` | a hold on the change or the landing is placed, or lifted | reason, by whom |
| `after` | after landing: verified, red, cleared, uncleared (`../domain/after-landing.md`, 6) | verdict |

- **Written by the step that decides or observes it,** in the same commit as the state it records, never by a reader.
- **Once per identity:** a step that writes an event whose identity is already written writes nothing, so a retry, a run's rerun under its id, or a restart relearning a run's state appends no second event. Recovering an effect already made, at a restart, is not a new step (`../domain/landing.md`, 15).
- **Only what changed:** a poll or a hint that finds nothing new writes nothing.
- **Both ways of landing:** `queue` writes every kind but `formed`, `round-ended`, `proved`, `search`, `searched`, `probe`, `probe-ended` and `landing-ended`; `turn` writes them all but `selected` and `paused`.

## 4. Times

- **`at`, temper's time,** is the commit's that wrote the event: when temper decided the step, or observed one it doesn't make, such as a push or a run's end. Every split is computed on it.
- **`source_at`,** where the source already gives the deciding step a time of its own (a run's creation), is kept beside it, never in its place. A source time temper doesn't read today (a run's start and end, a pull request's merge time) is Later (section 11).
- **Seen late is recorded late:** a run whose end is heard without its start having been seen writes its end with no start of its own, and no event is backdated or invented.

## 5. Kept

- **By change, apart from the rows:** a change's events, and its landing's events naming it, stay after the change and landing rows go.
- **By age and by count:** once a change's last event is older than the project's `history_days`, its events go together; past `history_events`, the changes whose last event is oldest go first, whole. A history is never cut in its middle, so what remains of a change is all of it, from its first event.
- **Swept in bounded steps,** on the engine's own tick as well as on writes, so an idle project's events still go.
- **On demand:** the history is read from the store when asked, as jig reads a task's activity, and is not part of what a restart restores (`../protocol/store.md`, 5).
- **With the project:** a project's deletion deletes its events.
- **Bounded records:** each field is a name, a number, a hash or a short reason within the forge's existing bounds; a `formed` event's members within `landing_members`; no event carries a log, a body or words.

## 6. Reading it

- **`landing history --project PROJECT [--change TASK | --landing TASK] [--since TIME] [--until TIME] [--after CURSOR] [--json]`:** the events in the project's order, at most 32 a page within ctl's document bound (`../client/ctl/README.md`, 9). The first page fixes the project's last number as the read's upper bound and the cursor carries it, so later pages read the same history. A change's or a landing's events are read through their own index, and a project's by its order, so a filtered read does bounded work. A cursor past the events that remain is refused, naming the first that does.
- **A change whose row went** answers `change show` by naming its last events and the read that lists them all; once its events have gone too, it says so.
- **Who may read it:** the project's readers, as for its changes, whether or not the change's task remains. The public projection shows none of it (`../client/forge/README.md`, 9.3).

## 7. Where the time went

- **`landing times --project PROJECT [--since TIME] [--until TIME] [--after CURSOR] [--json]`:** for each change any of whose intervals meets the range, its time from its first `ready` to `landed` or `ended`, or to the read's upper bound, marked open, in half-open intervals on `at`, clipped to the range, split into states that don't overlap:
  - **waiting:** ready and joined, not yet selected or in a formed round;
  - **preparing:** an update or a job asked and not settled;
  - **resolving**, **repairing**, **judging:** a resolution, a repair or a judgement asked and not answered;
  - **delivering:** a head pushed and not yet accepted;
  - **proving:** a gate's run at its head or its round's top, dispatched and not heard;
  - **probing:** its round in a search, from `search` to `searched`, every member of the round included;
  - **turning:** proved, until landed;
  - **paused**, **held:** CI paused; a hold on it or its landing;
  - **other:** none of these.
- **One order for overlaps,** stated with the read: held, paused, resolving, repairing, judging, probing, proving, preparing, delivering, turning, waiting; equal times go by the events' numbers. An interval tied to a head or a round ends when the head moves or the round ends: a run superseded keeps nothing proving.
- **Partial, never guessed:** a change whose first remaining event isn't its `ready`, because the history began after it was ready, is marked partial, its split from its first event on.
- **Paged by change:** at most 32 changes a page, each whole; the cursor carries the upper bound and the running totals, so the last page gives the range's totals per state. Under `turn`, each landing's rounds, members, probes and its proof's time follow its changes.
- **Computed, not stored:** from the events alone, so the same events and the same upper bound give the same answer at every read.

## 8. Settings and bounds

- **`history_days`** (stock 14) and **`history_events`** (stock 131,072) per project, among the forge's limits (`../protocol/configuration/limits.md`). At twenty goals in flight, each landing several changes a day into its branch and its own into main, with roughly ten events for a clean landing and twenty to thirty for one through a resolution or a red round, a project writes a few thousand events a day, so the count holds about the fourteen days; past that volume the count, not the age, sets how far back the history reaches, and `landing history` says from when it holds a project's changes whole.
- **A record** is bounded by its fields' bounds and its members by `landing_members`; the family's rows are demand-loaded and outside the restored state's memory bound.

## 9. Upgrades

- **A new stored family,** demand-loaded, with its version and patch, in the landing that adds it (`../protocol/store.md`, 2 and 4). There is no history before it: a change live at the upgrade has a partial history, and nothing is rebuilt or backdated.
- **`ctl`'s wire** moves its version for the new reads (`../protocol/wires.md`, 8).

## 10. Acceptance

Focused and fuzzy suites alone, each failing first:

- each kind written by its step, under `queue` and under `turn`, in the commit that decides or observes it, with both ends of each interval;
- a restart at each step, a retried job, a run rerun under its id, and a run's state relearned after a restart each write no second event;
- a run whose end is heard before its start was seen writes its end alone;
- a change and a landing whose rows were evicted keep their events, and `landing history` reads them; age and count bounds evict whole changes, oldest first, idle projects included;
- `landing history` filters by change, landing and range, pages under a fixed upper bound, and refuses a cursor past what remains;
- `landing times` for a change landed green at once, one through a resolution, one held, one repaired, one cancelled while ready, one ready across the range's start, one live at the upgrade (partial), a superseded run, equal times, and a `turn` landing whose red round was searched: each state's intervals as stated, the totals across pages;
- fuzzy: random step sequences in the forge's world keep each change's events ordered and well formed (each interval's start before its end, each `asked` answered or open, `landed` or `ended` at most once), and the split's states sum to the time from ready to the end for complete histories.

The process tier's queue stories (`../draft/merge-queue.md`, 12) read their landing's history at the end.

## 11. Later

- **Source times:** a run's start and end and its jobs', and a pull request's merge time, read by the forge client and kept as `source_at`.
- **A gate's own steps** (its wait for a runner, its build, each suite), reported by the project's gate and kept beside its run.
- **The client's view** of a change's and a landing's timeline.
- **A project's figures over time:** landings per hour, the share of time in each state, by day.
