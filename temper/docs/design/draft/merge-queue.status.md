# merge-queue: status

2026-11-13. Where the design lags or departs from [merge-queue.md](merge-queue.md).

Its execution's M1 to M3 are designed: `../domain/landing.md` and the documents it cites carry trains, batches of units stacked by the merge train's job, a branch per member, proof at the top, the landing's pull request fast-forwarded, and `../domain/landing-search.md` a red round of several units narrowed by prefixes, the blamed unit repaired, the green prefix landed in the round's turn and the units above it in the next round.

- **Design departs:**
  - 5.1: a member's branch in the landing is named per landing and member, `temper/<root>/l<task>-<n>`, the first member's the landing's own (`../domain/landing.md`, 5.1), not per round: the landing holds every branch it writes from its making, its members being sealed, as its one branch was held before batches. A round that ends unlanded deletes the branches from its first changed member up and keeps those below, rather than making the next round's branches beside them. The goal's designer, 2026-11-13.
  - 4.4, 7.2: a unit the round's search blames leaves without ending the round, which goes on to its turn at the green prefix (`../domain/landing-search.md`, 4), where 4.4 has any member leaving end its round; 7.2 lands the green prefix in the round's turn, which needs the round to outlive the blame. Its pull request is opened anew from the green prefix's branch, since a round's pull request is from the branch whose head it merges. The goal's designer, 2026-11-13.
  - 7.2: a probe dispatches the train's blocking test gates alone, an advisory gate being the candidate's (`../domain/landing-search.md`, 3): an advisory verdict holds nothing, so a probe's verdict cannot read it. The goal's designer, 2026-11-13.
  - 12: the queue's process stories run on the process tier's owned Forgejo and its runner (`../testing/process-tier.md`), not the fake forge, the red narrowed's gate failing by its `check.sh` reading a change's file rather than by the fake forge's CI following content: the fake forge serves no git over HTTP (skein's `testing/fakes/forge.md`), and the stories need the worker's merge train on real git. The goal's designer, 2026-11-13.
- **Open:**
  - 5.2's last bullet: whether the project's `merge_job` settles the conflicts it knows inside the prepare job, asked of the user 2026-11-13. Until it is answered, a conflict met while stacking goes to its change's resolution, where the merge job runs (`../domain/forge/change/repairs.md`).
