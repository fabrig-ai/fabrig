# A change's records and room

This document holds a change's records and room, split from [`README.md`](README.md).

## 1. Its room

The forge keeps a record of each change, `forge.changes` of them at
most, live and ended alike: in memory and as the store's `Change` row
(`Key::Change`, `../../../protocol/store.md`, 2). A record outlives its
task (`panel.md`, section 1), and leaves only when a new change needs its room, so
`forge.changes` bounds the changes kept at once, and through them the
changes live at once, never every change made.

- **Settled:** a change is over for good once its task has ended (its
  ending recorded and its release finished, so no release row of its
  task is left) and its state is `Ended`, `Cleared` or `NoChange`. A
  live change never goes, nor a landed one whose task lives.
- **Dropped last:** an `Uncleared` change, or a `Landed` or `Red` one,
  whose task has ended (its ending recorded, its release finished) is
  never stepped again (`../../after-landing.md`, 6). It goes only when
  no settled change can, oldest first, pinned as any settled change is,
  so a tier runner down for long refuses no new change while such a
  change can go (the user, 2026-11-11).
- **Pinned while read:** a settled change still stays while
  - **its goal lives:** the tracked goal it served (its row's `goal`),
    or else the tracked ancestor its goal's closer finds for it, is a
    live task. The closer reads every original change of its goal for
    the landed set, its close request and its report (`../branches.md`,
    2), and the close command settles each `[in flight: change N]` tag
    by it (`../../../protocol/tooling.md`, 4.2): a goal's change goes
    only once its goal has ended;
  - **a landing names it:** a landing row of `../../landing.md`, ended
    or not, holds it among its members, whose steps and document read
    each member's record (`../../landing.md`, 4 and 14).
- **Dropped when room is needed, oldest first:** a new change finding
  the table full drops, in the same decision that registers it, the
  settled, unpinned change with the lowest task number; task numbers
  are jig's, deployment-wide and never reused, so the lowest was made
  first. Its record leaves memory and its `Change` row is erased. While
  the table has room nothing is dropped for room, so a quiet deployment
  keeps each panel until its task's record goes (below).
- **Dropped with its task's record:** when jig erases the full record
  of the change's task (`Unname`, jig's `domain/connectors.md`, 3.4),
  30 days after it ended by jig's retention
  (jig's `protocol/store/retention.md`, 6.2), the root drops the change
  in that decision, full table or not, if it is settled or dropped
  last and unpinned, by the rules above, with what goes with it (below).
  A change pinned then stays, and goes when room is needed, as before.
  So a quiet deployment's table holds no change past its task.
- **The root chooses:** only the root knows which goals live, so it
  picks the change to drop, a settled one first and one dropped last
  only when no settled one can go, as `start_change` makes the registration,
  and asks the forge to drop it first (`forge::Event::DropChange { task
  }`, then `forge::Event::Change`). The forge drops it only if its
  table is full, the change is still settled or dropped last, and no
  landing names it; otherwise it drops nothing. The root alone checks
  whether another settled change can go, since only it knows which
  goals pin them. Drop and registration run synchronously, in
  `start_change`'s own decision and before anything else of the change
  is queued: only an accepted registration queues the change's names,
  its branch hold and its step, and a refused one leaves its named
  refusal (*Refused, named*) alone.
- **What goes with it:** every account the forge keeps by its task: its
  panel's CI, its settling, its start conflicts, its wakes and its
  deadline, its pending steps and goal-file reads (a late answer to one
  then finds nothing and is dropped), its goal files, and its `landed`
  commits, each erased where it is stored; and the queue repairs it
  made leave its landing branch's count of them. Its landing log entries stay
  (`../../after-landing-storage.md`, 1), so its project's landed history and
  status, which read the log, keep it.
- **Its landed commits are bounded alike:** the forge's `landed` table,
  each merge temper made and its task (the store's `Landed` row,
  `Key::Landed`), holds `max(forge.landings, forge.changes)` entries
  at most. A merge finding it full makes room in the same decision,
  erasing one entry where it is stored: one whose task has no change
  record first (a dropped change's, or a record a deployment removed by
  hand), else one whose change is settled, else the oldest; lowest task
  number first, the merging change's own last. Every reader of the table
  asks about a change it keeps (its release, its tier's ownership of a
  landing, a subscription's own landing), so an entry without a record
  is read by no one. The merge's own effects never wait on that room:
  its landing branch's head moves to the merge, is saved and pushed
  (`Pushed`), and its tiers observe it, whether or not its entry is kept
  (the operator, 2026-11-11: a full table held goals on their own merges
  into g).
- **A dropped change reads as an unknown task:** its panel is Gone
  (`panel.md`, section 1), and `temper ctl change show` prints its last events from
  its landing history, or that its history has gone too
  (`../../landing-history.md`, 5.2); a subscription naming it reads it as over, as it read
  its ended record, and wakes no one; a public merge one of whose
  members is gone shows that member from the landing log's own entry,
  its task, pull request and title; a tier reads its landing as one
  not its own (`../../after-landing-runs.md`, 1).
- **Refused, named:** a new change that finds the table full and
  nothing to drop is held before it registers, its words naming the
  bound, as every limit's refusal does
  (`../../../protocol/configuration/limits.md`):
  `forge.changes = 128; required usage 129; reduce the named work or
  raise this limit in the deployment configuration, restart, then
  release`. A release registers it once a change can go.
- **Why no bound of its own for kept panels:** a second limit, the
  ended changes kept, would need its own room in the store's sizing,
  and could refuse a live change beside room held for panels. Dropping
  on demand gives every row to live changes first and keeps the most
  history the table can hold.

## 2. Open questions

- **Ten busy goals:** pinning each live goal's changes keeps every
  record its closer reads, but a deployment whose live goals hold more
  than `forge.changes` changes between them refuses the next, naming
  it, until one ends or the limit is raised
  (`../../../protocol/configuration/limits.md`). A
  closer reading a compact account of its goal's changes, kept beside
  its own row, would lift the pin.
