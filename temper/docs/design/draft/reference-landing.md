# Reference: landing

Draft, 2026-11-02. How the reference lands what code production made: a
goal's branch into main as one change, the few small changes around it,
and their verification after landing. Code production is
`reference-code-production.md`; `reference-case.md` walks both;
`notes.md` holds the evidence and the reasons, cited by section and by
number (N1 to N24), and where each item of the landing model it drops
goes (`notes.md`, section 6).

Unqualified, `forge.md`, `landing.md`, `after-landing.md` and
`findings.md` are temper's `../domain/` documents; jig's are named as
jig's.

## 1. In one page

- **What lands is few and large.** One change per goal, L, whose branch
  is the goal's own branch g, which the goal holds (N3); beside it, small
  changes: a repair after landing, a chat's small fix. What the goal
  leaves for the next one (status files, Later sections, the draft's
  fate) is already in g, written by the close-out (N16). Nothing is
  batched.
- **By queue, as temper has it** (`forge.md`, 8.2, 9 and 10): checked at
  its head, queued by its goal's priority, the first brought up to date
  with main if it lacks main's tip and checked again, merged at exactly
  that head. One change at a time owns main's queue.
- **The goal is checked whole before it queues.** L's CI is the
  project's full gate, run once at its head and again only if its head
  moves.
- **L is an ordinary change.** It has no producing step, its branch
  being pushed already; its repairs, its update and its resolution write
  g as any change's write its branch, under the goal's hold (jig's
  `domain/tasks.md`, 6.3). Past its bounds it is held, escalated to its
  coordinator, which may still make changes into g: the goal reopens
  without a new mechanism.
- **Verified after landing** (`after-landing.md`): the slow tiers follow
  main; a red, rerun alone and blamed on its landing, reaches the goal as
  news; the coordinator triages it and repairs it with a change into
  main that names it.
- **The goal ends at cleared.** L's result, given at cleared, is the
  coordinator's last wake; it reports, and the goal closes, g deleted
  with its hold.
- **Dropped:** batches, trains, keys per package, provers, the
  serializer, leases, lent workspaces, the retire after landing. Landing
  by turn stays, built, an option this reference does not take.

## 2. What lands

| Change | Made by | Its branch | Lands into |
|---|---|---|---|
| the goal's landing, L | the coordinator, once the close-out has landed | g, `temper/<root>/g<root>`, its requester's held branch (N3) | main |
| a repair after landing | the coordinator, on a red | its own, `temper/<root>/c<task>` | main |
| a chat's small fix | a chat, handing over the branch it pushed (`forge.md`, 8.1) | the branch it is handed | main |

Each is a change under the project's queue, by the same rules; only L is
large. The changes into g are code production's
(`reference-code-production.md`, section 6).

## 3. The goal's landing

```
opening ─► checking ─► gating ─► queued ─► first ─► landing ─► landed ─► verifying ─► cleared
              ▲   │                           │
              │   └──► repairing ─────────────┤
              │                               ├─ lacks main's tip ─► updating
              └───────────────────────────────┤                        │
                                              └──── resolving ◄────────┘
                                                    (a conflict)
```

The change procedure's way (`forge.md`, 8.2), from opening:

1. **Made** by the coordinator with g as its branch: no producing step,
   as for a branch handed over.
2. **Opening:** its pull request into main, its title the goal's, its
   body naming the goal's issue and the draft.
3. **Checking:** CI on its head, the project's full gate, its slowest
   suites included: the check of the whole goal. A failure is a repair
   (`forge.md`, 8.4): an agent at g's tip given CI's failing steps and
   their logs' ends, pushing g under the goal's hold; the new head is
   checked again.
4. **Gating:** what main's rules add. The reference adds none beyond CI;
   a design review of the whole at L's head is a gate a project may add
   (`forge.md`, 8.3).
5. **Queued,** by its goal's priority, then by when it first became
   ready, aging bounding the wait (`forge.md`, section 9).
6. **First,** it owns main's queue through what follows, so no change
   overtakes it and no CI run is wasted. If its head lacks main's tip it
   is **updated** (`forge.md`, section 10), the forge merging main into g
   with no LLM, under the goal's hold: cleanly, and the new head is
   checked again; or in conflict, and it leaves the queue to be
   **resolved**: the merge job settles what the project's tooling knows
   (N7), a resolution agent what is left, in the merge in progress on g;
   the new head is checked again. A clean update whose CI fails while
   main's tip passes is a semantic conflict: a repair, its brief naming
   what landed in main.
7. **Landing:** merged at exactly its head, as a merge commit, so g's
   history (each change's merge) stays readable on main. The merge is
   refused while L is kept (`forge.md`, 8.2).
8. **Landed, then verifying,** until main's tiers clear it (section 6).
   Its result, given at cleared, wakes its coordinator.

**What moves g while L lives:** L's own delegates (its repairs and its
resolution) and the forge's update it asked for, each taking g's writer
slot as it writes; and the changes the coordinator makes into g, whose
merges take it too. A move by the coordinator's other delegates is L's
new head, checked again as after a repair, never drift (N3). A move by
anyone else is drift (`forge.md`, 8.5).

**Bounds** (`forge.md`, 8.4): repairs (three in the reference),
resolutions, updates, and each wait's stall. Past one, L is held, saying
why, and the hold escalates to its coordinator (jig's
`domain/tasks.md`, section 9). The coordinator:

- releases it with an amendment: more repairs, a budget, words naming
  the cause;
- makes changes into g first, where the red is a family's to rework,
  leaving L held until they land, then releases it: L checks the new
  head;
- cancels it, dropping the goal's landing. g stays the goal's, its hold
  unreleased, so a later L may land it; the goal's close releases it, and
  g is kept for a person to read (N3);
- or passes it up to the person.

## 4. Main's queue

- **One queue, derived** from the ready changes into main (`forge.md`,
  section 9): L and the small changes, ordered by priority, then by when
  each became ready.
- **One at a time is enough.** Few changes land into main, and most are
  large and checked whole. The evidence agrees: of 131 batches through
  the queue it records, 129 carried one branch, and none was ever
  bisected (`notes.md`, 2.2).
- **Main's own CI red** pauses the queue and makes one repair, a change
  into main, first in the queue (`forge.md`, section 9). Where tiers
  judge main, they do instead, and the queue never pauses for them.
- **A forge's own merge queue,** where main's rules require one, is
  entered as `forge.md`, 9.1 says: temper's queue decides what enters and
  when.

## 5. Catching up

g meets main twice: along the way, by catch-up changes into it that its
coordinator makes as main's landings touch the goal's open work (N6,
`reference-code-production.md`, section 11); and at the end, when L is
first and lacks main's tip, by the forge's update, a conflict going to
the merge job and a resolution. The first keeps the second small: the
evidence's one catch-up left to the end met 23 conflicts and took 56
minutes by hand (`notes.md`, 2.2).

## 6. After landing

As `after-landing.md` has it, designed and mostly not built, with one
change: each tier names the branches it follows (N5), and main's follow
main alone.

- **Main's tiers** are the suites too slow for the gate (a browser suite,
  exhaustive profiles). Each runs on main's tip as it moves, one run per
  tier at a time, the next at the newest tip, so one run covers every
  landing since the last (`after-landing.md`, section 3).
- **A verdict** (`after-landing.md`, section 5): green clears every
  landing in the run's range. Red reruns the failed tests alone: those
  that pass are a flake. What failed in the tier's previous run too is
  inherited and blamed on no one new. A new failure is the one landing
  in the range's, or the first landing whose commit fails it, found by
  bisecting over the range's landings.
- **L lives to cleared** (`after-landing.md`, section 6): landed,
  verifying, cleared, its result given then. Past the project's
  verification bound (two hours) with no verdict, it ends landed and
  uncleared, saying so, and a later verdict still counts.
- **A red goes to the goal as news** (`after-landing.md`, section 7): the
  tier, the failing tests blamed on its landing, the run and the end of
  each failed job's log. The coordinator wakes on it, as it does on a red
  of g's tier.
- **The coordinator triages it,** reading the log, the design and the
  tree (which change owned the failing files) and g's history, now on
  main; or asks a triage delegate, read-only, for a verdict. Then it
  makes **a repair change into main**, small, naming the red landing
  (`repairs`, N22), its words the failing tests and what the triage
  found. It lands by the queue, and the next tier run that passes the
  failures clears L's landing, repaired by it (`after-landing.md`, 5.3).
  A failure the triage finds inherited or a flake gets no repair: the
  coordinator says so in a note on the tier's tests, where the next run's
  triage finds it, and the verdict stands as the tier gave it.
- **Others go on.** Other changes keep landing into main, and their
  verdicts do not count the red's failures. A project may pause its queue
  while one of its landings is red (`after-landing.md`, 7).
- **A red of no live goal** (a goal that ended, a landing temper did not
  make) is filed as a finding (`findings.md`, section 4) and goes to the
  project's triager.

## 7. A small change

A repair after landing and a chat's small fix go the way every change
into main goes (`forge.md`, 8.2): CI at their head, queued, updated if
they lack main's tip, merged at their head, verified. A repair after
landing is first among equals only by its goal's priority; a repair of
main's own CI is first in the queue (`forge.md`, section 9). A repair
after landing that should change what g's close-out wrote (a status
entry, a Later section) carries that edit itself.

## 9. What has not yet run unattended

From the evidence (`notes.md`, 2.2): a catch-up with main, a
plan stopped by its bounds, a goal's branch landed with a red after
landing routed to a repair. Each was designed and tested in pieces, and
done by hand when it happened. They are this reference's first stories:

- a goal's landing whose update into main conflicts, settled by the
  merge job and a resolution on g;
- a goal's landing held past its repairs, released by its coordinator
  after a change into g;
- a red after landing, triaged and repaired by a change naming it, the
  goal cleared by the repair.

## 11. Later

- **Several goals at once,** their branches overlapping. A mechanical
  probe merges each pair of live goal branches, and each with main,
  without pushing (`git merge-tree`), and sends their conflicts to both
  goals as news. A goal that meets one may land a checkpoint: a change
  into main on a branch made at a commit of its own branch, checked
  whole by the full gate, which the other goal then takes by its
  ordinary catch-up, while its own branch goes on. Goals never merge
  each other's unlanded branches: a branch carrying another goal's
  unlanded work cannot land without it, which makes two goals one
  landing, and a dropped goal a hole in the other (N2's rule for
  dependencies, across goals). A semantic probe, the gate on a
  speculative merge, costs a CI run per pair and waits for numbers.
  Needs: the probe and its news kind, `change` from a commit of the
  goal's branch (N22), and the coordinator's judgement of when a commit
  is a coherent checkpoint.
- A test that fails at the gate rerun alone before a repair, as tiers
  rerun theirs (N17).
- A red after landing bisected within a goal's landing, over its
  changes' merges, to find the change that brought it.
- Several repositories, one goal: a branch per repository, each held by
  the goal, their landings ordered by their dependencies (`forge.md`,
  8.1), skein's before temper's.

## 12. Open questions

- **A whole-goal review as a gate,** at xhigh, before L queues? The
  evidence had none; a project may add one by its rules.
- **Pause main's queue while a goal's landing is red?** The default lets
  others land; a project that lands one goal at a time may prefer to
  stop until the repair lands.
