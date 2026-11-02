# Reference: landing

Draft, 2026-11-02. How the reference lands what code production made: a
goal branch into main as one change, the few small changes around it,
and their verification after landing. It supersedes `landing-model.md`
(section 9 says where each of its items goes). Code production is
`reference-code-production.md`; `reference-case.md` walks both;
`reference-notes.md` holds the evidence and the design changes, cited
by number.

Unqualified, `forge.md`, `landing.md`, `after-landing.md` and
`findings.md` are temper's `../domain/` documents; jig's are named as
jig's.

## 1. In one page

- **What lands is few and large.** One change per goal, whose branch is
  the goal branch, closed by its coordinator (N3); beside it, small
  changes: a repair after landing, the retire, a chat's small fix.
  Nothing is batched.
- **By queue, as temper has it** (`forge.md`, 8.2, 9 and 10): checked at
  its head, queued by its goal's priority, the first brought up to date
  with main if it lacks main's tip and checked again, merged at exactly
  that head. One change at a time owns main's queue.
- **The goal is checked whole before it queues.** The goal change's CI is
  the project's full gate, run once at its head and again only if main
  moved.
- **A red at the gate is the goal's.** The goal change writes its branch
  only through changes into it (N3): a repair change for a red, a
  catch-up change (the merge job, then an agent) for a conflict. Past
  their bounds it is held for its coordinator: the plan stops there.
- **Verified after landing** (`after-landing.md`): the slow tiers follow
  main; a red, rerun alone and blamed on its landing, reaches the goal as
  news; the coordinator triages it and repairs it with a change into
  main that names it.
- **The goal ends at cleared,** its last act the retire: the status files
  and the designs' Later sections, landed as a small change.
- **Dropped:** batches, trains, keys per package, provers, the
  serializer, leases, lent workspaces. Landing by turn stays, built, an
  option this reference does not take.

## 2. What lands

| Change | Made by | Its branch | Lands into |
|---|---|---|---|
| the goal's change, B | the coordinator, in the cut's first batch | `temper/<root>/g<B>`, a landing branch while it collects (N3) | main |
| a repair after landing | the coordinator, on a red | its own, `temper/<root>/c<task>` | main |
| the retire | the coordinator, from words; its producer under the designer's charter (N15) | its own | main |
| a chat's small fix | a chat, handing over the branch it pushed (`forge.md`, 8.1) | the branch it is handed | main |

Each is a change under the project's queue, by the same rules; only B
is large. The changes into the goal branch are code production's
(`reference-code-production.md`, section 6).

## 3. The goal's change

```
collecting ─► closed, drained ─► opening ─► checking ─► queued ─► first ─► landing ─► landed ─► verifying ─► cleared
                                              ▲   │                  │
                                              │   └─► a repair ──────┤
                                              │       change into    ├─ lacks main's tip ─► updating
                                              │       its branch     │                        │
                                              └──────────────────────┤                        │
                                                                     └─ a catch-up change ◄───┘
                                                                        into its branch, on a conflict
```

1. **Closed and drained** (`reference-code-production.md`, section 12):
   its coordinator has closed it and no change into its branch is live.
2. **Opening:** its pull request into main, its title the goal's, its
   body naming the goal's issue and the draft.
3. **Checking:** CI on its head, the project's full gate, its slowest
   suites included: the check of the whole goal. A failure makes a
   repair change into its branch (N3), made by its step, first in its
   branch's queue: its producer starts from the goal branch's tip, its
   brief CI's failing steps and their logs' ends (`forge.md`, 8.4); it
   merges into the goal branch, and the new head is checked again.
4. **Gating:** what main's rules add. The reference adds none beyond CI;
   a design review of the whole at the goal's head is a gate a project
   may add (`forge.md`, 8.3).
5. **Queued,** by its goal's priority, then by when it first became
   ready, aging bounding the wait (`forge.md`, section 9).
6. **First,** it owns main's queue through what follows, so no change
   overtakes it and no CI run is wasted. If its head lacks main's tip it
   is **updated** (`forge.md`, section 10), the forge merging main into
   the goal branch with no LLM: cleanly, and the new head is checked
   again; or in conflict, and it leaves the queue for a catch-up change
   into its branch (N4): the merge job settles what the project's tooling
   knows (N7), a resolution what is left, and the catch-up merges into
   the goal branch. A clean update whose CI fails while main's tip passes
   is a semantic conflict: a repair change, its brief naming what landed
   in main.
7. **Landing:** merged at exactly its head, as a merge commit, so the
   goal branch's history (each change's merge) stays readable on main.
   The merge is refused while the change is kept (`forge.md`, 8.2).
8. **Landed, then verifying,** until main's tiers clear it (section 6).
   Its result, given at cleared, wakes its coordinator.

**Bounds** (`forge.md`, 8.4): repairs (three in the reference),
catch-ups, updates, and each wait's stall. Past one, the change is held,
saying why, and the hold escalates to its coordinator (jig's
`domain/tasks.md`, section 9): the plan's stop. The coordinator releases
it with an amendment (more repairs, a budget), reopens it to make
changes into its branch first (N3), cancels it, dropping the goal's
landing, or passes it up to the person.

## 4. Main's queue

- **One queue, derived** from the ready changes into main
  (`forge.md`, section 9): the goal's change and the small changes,
  ordered by priority, then by when each became ready.
- **One at a time is enough.** Few changes land into main, and most are
  large and checked whole. The evidence agrees: of 131 batches through
  the queue it records, 129 carried one branch, and none was ever
  bisected (`reference-notes.md`, 2.2).
- **Main's own CI red** pauses the queue and makes one repair, a change
  into main, first in the queue (`forge.md`, section 9). Where tiers
  judge main, they do instead, and the queue never pauses for them.
- **A forge's own merge queue,** where main's rules require one, is
  entered as `forge.md`, 9.1 says: temper's queue decides what enters
  and when.

## 5. Catching up

The goal branch meets main twice: along the way, while it collects, by
catch-up changes into its branch (N4, `reference-code-production.md`,
section 11), and at the end, when it is first and lacks main's tip, by
the forge's update, a conflict going to a catch-up change as along the
way. The first keeps the second small: the evidence's one catch-up left
to the end met 23 conflicts and took 56 minutes by hand
(`reference-notes.md`, 2.2).

## 6. After landing

As `after-landing.md` has it, designed and mostly not built, with one
change: each tier names the branches it follows (N5), and main's follow
main alone.

- **Main's tiers** are the suites too slow for the gate (a browser
  suite, exhaustive profiles). Each runs on main's tip as it moves, one
  run per tier at a time, the next at the newest tip, so one run covers
  every landing since the last (`after-landing.md`, section 3).
- **A verdict** (`after-landing.md`, section 5): green clears every
  landing in the run's range. Red reruns the failed tests alone: those
  that pass are a flake. What failed in the tier's previous run too is
  inherited and blamed on no one new. A new failure is the one landing
  in the range's, or the first landing whose commit fails it, found by
  bisecting over the range's landings.
- **The goal's change lives to cleared** (`after-landing.md`, section
  6): landed, verifying, cleared, its result given then. Past the
  project's verification bound (two hours) with no verdict, it ends
  landed and uncleared, saying so, and a later verdict still counts.
- **A red goes to the goal as news** (`after-landing.md`, section 7):
  the tier, the failing tests blamed on its landing, the run and the end
  of each failed job's log. The coordinator wakes on it.
- **The coordinator triages it,** reading the log, the plan's notes
  (which change owned the files) and the goal branch's history; or asks
  a triage delegate, read-only, for a verdict. Then it makes **a repair
  change into main**, small, naming the red landing (`repairs`, N21),
  its words the failing tests and what the triage found. It lands by the
  queue, and the next tier run that passes the failures clears the goal's
  landing, repaired by it (`after-landing.md`, 5.3). A failure the triage
  finds inherited or a flake gets no repair: the coordinator says so in
  its notes, and the verdict stands as the tier gave it.
- **Others go on.** Other changes keep landing into main, and their
  verdicts do not count the red's failures. A project may pause its
  queue while one of its landings is red (`after-landing.md`, 7).
- **A red of no live goal** (a goal that ended, a landing temper did not
  make) is filed as a finding (`findings.md`, section 4) and goes to the
  project's triager.

## 7. The retire

When the goal's change gives its result (cleared, or landed and
uncleared past the bound), the coordinator decides what the goal leaves
for the next one: the status files beside the designs it touched (where
the code lags or departs), the designs' Later sections, and the draft's
fate (kept, folded into a design, or removed). It writes no file itself
(N15): it makes the retire, a change into main whose words say what to
write, the longer text in a note, and whose producer runs under the
designer's charter at xhigh (N21). The retire lands by the queue as any
change and is cleared at once where every tier ignores what it touches.
Its result wakes the coordinator, which reports; the goal is done.

## 8. A small change

A repair after landing, the retire and a chat's small fix go the way
every change into main goes (`forge.md`, 8.2): CI at their head, queued,
updated if they lack main's tip, merged at their head, verified. A
repair after landing is first among equals only by its goal's priority;
a repair of main's own CI is first in the queue (`forge.md`, section 9).

## 9. The landing model's items, here

`landing-model.md` (2026-10-30) planned a landing model for many
landings at once. Under the reference, each item is either already
temper's or jig's, still needed, or not needed:

| Items | Here |
|---|---|
| L1 batches, L2 trains, L3 candidates by a merge chain, L5 keys per package, L6 gate classes, L7 provers and runners, L9 the serializer's durability, L18 the reservation | not needed: one change at a time owns main's queue; landing by turn keeps what is built of them (`landing.md`) |
| L4 the project's tooling as a job, no model | needed, as the merge job (N7) |
| L8 a measured failure rerun once before red | Later, for the gate (N16); tiers have it (`after-landing.md`, 5.2) |
| L10 a repair pinned ahead of every landing | temper's branch repair, first in its queue (`forge.md`, section 9) |
| L11 a change's own checks before ready | the producer's checks at delivery (`.temper/pre-pr`) |
| L12 leases on paths across goals | not needed: one goal branch per goal; overlaps between goals Later |
| L13 tiers following main, L14 cleared as a setting | needed, as designed (`after-landing.md`), each tier naming the branches it follows (N5) |
| L15 findings, L16 QA walks | `findings.md`; the reference uses only a red of no goal's |
| L17 trailers in landing news | as designed (`after-landing.md`, section 8); the reference does not need them |
| L19 views and acts | the client's, for what remains: the queue, tiers, verdicts, holds |
| L20 one change in one repository | kept: the reference is one repository |
| J1 owned holds | built, unused here |
| J2 to J6, J9, J12: drains, inherited holds, parks until an event, moves, successors, holds over admission, lent workspaces | not needed by the reference; jig's designs keep them |
| J7, J8 a conversation's questions | not needed: no chat stands between the goal and its person; jig's `domain/tasks.md`, 8.5 keeps them |
| J10 an address by role | not needed: questions and escalations go up the tree |
| J11 a fresh run at a context's bound | the coordinator starts fresh on each wake (jig's `domain/engine.md`, 7.4); a long run on Claude Code is the program's to compact |
| F1 skein's artifact read | needed by tiers (`after-landing.md`, section 4) |
| F2 to F5 GitHub's turn and permissions | not needed: the reference lands by queue |

## 10. What has not yet run unattended

From the evidence (`reference-notes.md`, 2.2): a catch-up with main, a
plan stopped by its bounds, a goal branch landed with a red after
landing routed to a repair. Each was designed and tested in pieces, and
done by hand when it happened. They are this reference's first stories:

- a goal branch whose update into main conflicts, settled by a catch-up
  change, the merge job and an agent;
- a goal change held past its repairs, released by its coordinator with
  an amendment;
- a red after landing, triaged and repaired by a change naming it, the
  goal cleared by the repair.

## 11. What is missing

| Gap | Where | Note | Status |
|---|---|---|---|
| the collecting change: its branch shared, its repairs and conflicts as changes into it, opening into main once closed and drained, reopened by amendment | temper | N3 | missing; contradicts `forge.md`, section 2, 8.4 and 8.5 |
| catch-up changes, the merge job before a resolution | jig, temper | N4, N7 | jobs designed, not built |
| tiers naming the branches they follow | temper | N5 | missing |
| tiers following main, verdicts, a change living to cleared, reds to goals | temper | `after-landing.md` | designed; no status file; little code found |
| the coordinator's `change` with `repairs` and a producer's charter | temper | N21 | missing |
| a flake at the gate rerun before a repair | temper | N16 | Later |

The queue itself, the update, the resolution, repairs, bounds and holds
are built (`forge.status.md`).

## 12. Later

- Two goals landing near each other, their branches overlapping: a
  report to both coordinators before the second lands.
- A red after landing bisected within a goal's landing, over its
  changes' merges, to find the change that brought it.
- Several repositories, one goal: changes ordered by their dependencies
  (`forge.md`, 8.1), skein's before temper's.

## 13. Open questions

- **A whole-goal review as a gate,** at xhigh, before the goal's change
  queues? The evidence had none; a project may add one by its rules.
- **Pause main's queue while a goal's landing is red?** The default
  lets others land; a project that lands one goal at a time may prefer
  to stop until the repair lands.
