# merge-queue: status

2026-11-13. Where the design lags or departs from [merge-queue.md](merge-queue.md).

Its execution's M1 and M2 are designed: `../domain/landing.md` and the documents it cites carry trains, batches of units stacked by the merge train's job, a branch per member, proof at the top, the landing's pull request fast-forwarded, and a red round of several units falling back. M3, the search by prefixes, is not designed.

- **Design lags:**
  - 7.2: `../domain/landing.md`, 7.3 falls back on a red round of several units, the draft's own interim (7.2's last bullet): no probe at a unit boundary, no blamed unit repaired, no green prefix landed with the units above it going on in the next round.
  - 7.3: `../domain/landing.md` keeps no search, so it has no search's identity, and a probe's late outcome is none of its; only a run's receipt by key outlives its round (7.1).
  - 7.4: no blamed unit's brief naming the red prefix, the green one below it and the members that landed.
  - 8: `../domain/landing.md`, 8 has no empty turn: a round lands its whole stack or nothing.
  - 9: `../domain/landing.md`, 4.2 has no probing news, and 10 and 14 show no search on the round's pull request or in the changes view.
  - 10: `../domain/landing.md`, 12 keeps no search's bounds, and `../protocol/configuration/limits.md` names no probe's bound.
  - 12: `../testing/process-tier.md` has no story of a red narrowed.
- **Design departs:**
  - 5.1: a member's branch in the landing is named per landing and member, `temper/<root>/l<task>-<n>`, the first member's the landing's own (`../domain/landing.md`, 5.1), not per round: the landing holds every branch it writes from its making, its members being sealed, as its one branch was held before batches. A round that ends unlanded deletes the branches from its first changed member up and keeps those below, rather than making the next round's branches beside them. The goal's designer, 2026-11-13.
  - 7.2's last bullet, 10: the fallback is a mark on each change that left the red round, to land alone (`../domain/landing.md`, 7.3 and 12), not a record on the landing: a landing's row may be retired once it ends, and the mark must last until its change has been through a landing alone. The landing's result still names each member that left and why. The goal's designer, 2026-11-13.
  - 12: the queue's process story runs on the process tier's owned Forgejo and its runner (`../testing/process-tier.md`), not the fake forge: the fake forge serves no git over HTTP (skein's `testing/fakes/forge.md`), and the story needs the worker's merge train on real git. The goal's designer, 2026-11-13.
- **Open:**
  - 5.2's last bullet: whether the project's `merge_job` settles the conflicts it knows inside the prepare job, asked of the user 2026-11-13. Until it is answered, a conflict met while stacking goes to its change's resolution, where the merge job runs (`../domain/forge/change/repairs.md`).
