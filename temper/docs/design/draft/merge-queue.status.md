# merge-queue: status

2026-11-11. Where the design lags or departs from [merge-queue.md](merge-queue.md).

Nothing of this draft is designed or built yet: its execution file's milestones write the design as they run.

- **Design lags:**
  - 3: `../domain/landing.md`, 7.2 runs proofs side by side, and 9 gives the turn to the first proved, with a reservation for a landing whose proof went stale; nothing holds a repository and branch for one landing; `../domain/forge/queues.md` orders only a project that lands by queue; a closure across repositories is not refused.
  - 4: `../domain/landing.md`, 4.1 joins members only by `with` or a dependency and never merges two live landings, with no batch, unit or forming; 4.2 ends a landing only when every member landed or was cancelled, with no member leaving and no rounds.
  - 5: `../domain/landing.md`, 5.2 prepares a member of its own by the forge's rebase of its branch, and 5.3 stacks by a rebase onto the member's branches; neither merges onto branches the landing holds or reports prefixes. `../domain/worker.md`, 4.4 and `../protocol/tooling.md` deliver one candidate to one branch with one lease; no prefixes, per-member conflicts or partial pushes. `../domain/forge/branches.md` names one landing branch per repository, not one per round and member.
  - 5.2: `../domain/landing.md`, 5.3 and `../domain/forge/change/repairs.md` resolve every conflict a prepare meets against main, with no conflict between members told apart.
  - 6: `../domain/landing.md`, 3 and `../domain/forge/change/gates.md`, 1 and 2 judge a member's verdicts at its own candidate, not at a prefix on a landing's pull request; 6.1's key has no base; 13.2's adoption doesn't check that the forge's rules on main admit a landing's pull request.
  - 7: `../domain/landing.md`, 7.3 repairs the failing member of a repository, with no search, no green prefix landed, no rounds and no fallback.
  - 8: `../domain/landing.md`, 8 merges each member's own pull request at its candidate and checks only the bases; no landing pull request, no member landed by containment, no member heads read at the turn; 10 shows no round's pull request; 11's effects have no landing pull request or prefix branches per round; 15 recovers no round or partly pushed job.
  - 9: `../domain/forge/change/README.md`, 3 holds a change whose head another identity moved, but nothing takes it out of a landing.
  - 10: `../domain/landing.md`, 13.1 has no `batch`; 12 keeps no rounds, prefixes, outcomes, search or fallback, and its `landing_members` admission says nothing of units; the stock `landing_members` of 2 is a code default, with no sizing rule for batches in `../protocol/configuration/limits.md`.
  - 11: `../domain/after-landing.md`, 2 keeps main's focused and fuzzy tier for a project whose landings run the same suites.
- **Design departs:** none yet.
- **Open:** none. The user's word of 2026-11-11: `batch` 4 for this project once batches are built, a setting to change as needed.
