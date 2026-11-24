# landing-liveness: execution

2026-11-24. How [landing-liveness.md](landing-liveness.md) gets built: one milestone of four goals, which start once the bounded-waits milestone's goal for temper's changes, landings and closers has landed (`bounded-waits.execution.md`, 1, W4), since that goal gives the landing's waits their records and bounds in the same code. It designs nothing: each goal writes its design from the draft as it runs. Plans use the focused and fuzzy suites only, and no goal walks.

## 1. The milestone and its goals

One milestone, *a landing train that always moves*. Its goals land into main one by one.

| Goal | Draft parts | Where | Needs |
|---|---|---|---|
| L1, one owner for members: the landing row as the one record, checked at start for a change two landings claim; the derived index from change to live landing; the handed-over ask as a queue, with its kinds, `Landed` first, taken by identity, a superseded command dropped with words; the written conversion of the previous build's asks and links, with its tests | 3, 8 | temper's forge domain: landing rows and their mutations, the index, changes' asks, restore | bounded-waits W4 |
| L2, releases: an account on every release failure, round deletions included, and temper's own causes named; a release that finds the resource and its owner first, and treats one already released by its owner as a success; P88's scenario passing, its ignore and sweep exclusion removed | 7 | temper's forge domain: release and cleanup outcomes | bounded-waits W4 |
| L3, forming: who may join from stored state and today's guards; the cohort a releasing decision stores, left at each candidate's first decision; a refusal that may pass as a wait the train owns, and one that won't as a hold on its unit; the settling and refusal memories retired. It records its waits as wait records and shows nothing of its own | 4 | temper's root forming and forge landing | L1, bounded-waits W5 |
| L4, liveness: the policy's departures first, then the check, in each landing step, on a withdrawn wake or a fired deadline, and as a structural pass at start; uncertain merges settled before anything else; ending or holding a landing that can't move, and the forge's ending for a row with no task; a member's hold, cancel or end waking its landing; the endings' shared settling and their separate steps; the check as a property in the landing policy's fuzzy and the engine forge and root worlds; S7's scenario passing | 5, 6, 11 | temper's forge landing domain, the root's routing and its worlds | L1, L2, L3, bounded-waits W6 |

L1 and L2 run side by side once bounded waits' goal for temper's procedures has landed: L1 owns the landing rows, the index and the asks; L2 owns cleanup's outcomes; L4 owns the closing hook they meet in. L3 follows L1 and bounded waits' goal for held answers and `after`, whose dependency facts forming reads. L4 follows the rest and bounded waits' stuck detection, which owns jig's hold and wake paths that a member's hold reaching its landing may need.

## 2. Keeping this file and the status

As in `bounded-waits.execution.md`, 2: only L4, the last, updates this file and the draft's status file, from the milestone's results. L1 to L3 leave both alone, record their design in the design documents they change, with their status files, and say in their results what of the draft they carried.

## 3. After the milestone

- The team retires its landing watches (`landing-liveness.md`, 9) once this milestone and bounded waits' list of every wait have reached the reference deployment.
- The conversion of old asks and the stale-link sweep go once the reference deployment's store has run them (`landing-liveness.md`, 8): a small goal of its own, on the user's word, after the next redeploy.
