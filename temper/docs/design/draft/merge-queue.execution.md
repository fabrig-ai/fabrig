# merge-queue: execution

2026-11-13. How [merge-queue.md](merge-queue.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone; a process-tier story is written and compiled in its milestone, and runs in the process tier after it lands. Each milestone is tried on a staging deployment after it lands.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| M1, one at a time on `turn`: trains, one landing each, waiting in the queue's order, formed level-triggered; one unit of one change per landing, a larger unit or a closure across repositories refused, saying which milestone brings it; the job's contract, payload, result, delivery to one branch the landing holds, and recovery; `combine`, both a merge and a squash, a squashed member's pull request recorded merged; the landing's pull request, fast-forward only, members landed by containment; judgement gates at the candidate; a member leaving and a landing ending empty; a red repaired; holds past a step's bound; news and the view | 1 to 3, 4.1 to 4.4 for one change, 5.1 and 5.2 for one member, 5.3, 6, 7.1, 8, 9, 10, 12 | temper/migration | a dispatch-only gate workflow, given the base (11); the forge's fast-forward merge, manual-merge detection and main's rules on the staging deployment | after landing, on the staging deployment: goals land one at a time through it, their changes squashed into their branches; a red repaired; a conflict resolved; a goal delivering mid-proof; a restart in each phase recovered | landed with goal 683 (main 6d3aa97b7d); its staging walk follows |
| M2, batches: units whole, forming up to `batch`, sealed; the job merging several members, a branch per prefix, partial pushes found again, cleanup; leaving with the unit and preparing again from the first changed prefix; conflicts told apart; rounds; a red round of several units ending, and the recorded fallback taking its units one at a time; the process story of a queue that lands green (12), written and compiled | 4, 5, 7.2's last bullet, 7.3, 8, 10, 12 | temper/migration | M1 | after landing: the process story runs in the process tier; on the staging deployment with `batch` above 1: several goals ready at once land together; two members in conflict; a member whose head moves mid-proof; a red round falling back | landed with goal 2175; its process story and staging walk follow, its red round's walked as M3's search, which replaced the fallback |
| M3, the search by prefixes: probes at unit boundaries, the blamed unit repaired, the green prefix landed in its round's turn, the rest in the next round, late outcomes kept to their keys; the process story of a red narrowed (12), written and compiled | 7.2, 7.3, 7.4, 9, 12 | temper/migration | M2 | after landing: the process story runs in the process tier; on the staging deployment: a batch with one breaking member lands the members below it and repairs that member | landed with goal 2336; its process story and staging walk follow |

## 2. Order

- **M1 first:** `turn`'s first use on a real forge, and enough for a project to move from landing by queue.
- **M2 builds on M1's job, landing pull request and trains;** a project raises `batch` above 1 once it lands and its staging walk passes.
- **M3 replaces M2's fallback** for a red round.
- **Section 13's Later items** are no milestone's.
