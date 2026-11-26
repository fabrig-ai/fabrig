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

### 1.1 What the milestone carried

All four goals have run, 2026-11-26. Where the design now holds each part of the draft:

| Goal | Draft parts | Design |
|---|---|---|
| L1 | 3, 8 | `../domain/landing-members.md`: the row as the one record, the derived index and the claim check at start (2), the ask queue with its kinds, taking by number and what is dropped (3), the conversion of the previous build's links and asks (4) |
| L2 | 7 | `../domain/forge/releases.md`: a release that finds the resource and its owner first (1), and its account on every path (2); a round's cleanup holding with it (`../domain/landing-prepare.md`, 1.1); P88's scenario active (`../testing/engine-checks.md`, 5.1) |
| L3 | 4 | `../domain/landing-turn.md`, 2: who may join from stored state, and the cohort; `../domain/landing-forming.md`, 3: a refusal that may pass stored on its train |
| L4 | 5, 6, 11 | `../domain/landing-liveness.md`: the check at every step and at start (2, 6), a landing that can't move ended or held (3), a member's hold or end waking its landing (4), every ending's settling (5), and the check as a property (8) |

What still lags, and where the design departs, is the draft's status, [landing-liveness.status.md](landing-liveness.status.md).

## 2. Keeping this file and the status

As in `bounded-waits.execution.md`, 2: only L4, the last, updates this file and the draft's status file, from the milestone's results. L1 to L3 leave both alone, record their design in the design documents they change, with their status files, and say in their results what of the draft they carried.

## 3. After the milestone

- The team retires its landing watches (`landing-liveness.md`, 9) once this milestone and bounded waits' list of every wait have reached the reference deployment.
- The conversion of old asks and the stale-link sweep go once the reference deployment's store has run them (`landing-liveness.md`, 8): a small goal of its own, on the user's word, after the next redeploy.
