# merge-queue: execution

2026-11-11. How [merge-queue.md](merge-queue.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. Each milestone is tried on a staging deployment after it lands, then the project's own deployment takes it.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| M1, one at a time on `turn`: trains, one landing each, waiting in the queue's order, formed level-triggered; one unit of one change per landing, a larger unit or a closure across repositories refused, saying which milestone brings it; the job's contract, payload, result, delivery to one branch the landing holds, and recovery; the landing's pull request, fast-forward only, members landed by containment; judgement gates at the candidate; a member leaving and a landing ending empty; a red repaired; holds past a step's bound; news and the view | 1 to 3, 4.1 to 4.4 for one change, 5.1 and 5.2 for one member, 5.3, 6, 7.1, 8, 9, 10, 12 | temper/migration | the project's dispatch-only gate workflow, given the base (11); the forge's fast-forward merge, manual-merge detection and main's rules on the staging deployment | after landing, on the staging deployment: goals land one at a time through it; a red repaired; a conflict resolved; a goal delivering mid-proof; a restart in each phase recovered; then the project's deployment moves from landing by queue | not started |
| M2, batches: units whole, forming up to `batch`, sealed; the job merging several members, a branch per prefix, partial pushes found again, cleanup; leaving with the unit and preparing again from the first changed prefix; conflicts told apart; rounds; a red round of several units ending, and the recorded fallback taking its units one at a time | 4, 5, 7.2's last bullet, 7.3, 8, 10, 12 | temper/migration | M1 | after landing, on the staging deployment at `batch` 4: several goals ready at once land together; two members in conflict; a member whose head moves mid-proof; a red round falling back | not started |
| M3, the search by prefixes: probes at unit boundaries, the blamed unit repaired, the green prefix landed in its round's turn, the rest in the next round, late outcomes kept to their keys | 7.2, 7.3, 7.4, 9, 12 | temper/migration | M2 | after landing, on the staging deployment: a batch with one breaking member lands the members below it and repairs that member | not started |

## 2. Order

- **M1 first:** it replaces the project's landing by queue, and is `turn`'s first use on a real forge.
- **M2 builds on M1's job, landing pull request and trains;** the project's `batch` goes to 4 once it lands and its staging walk passes.
- **M3 replaces M2's fallback** for a red round.
- **Section 13's Later items** are no milestone's.
