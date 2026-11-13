# squash-in: status

2026-11-13. Where the design lags or departs from [squash-in.md](squash-in.md).

Sections 2, 3 and 6 are designed: a change into a goal's branch squashes by its train's `combine` (`../domain/landing.md`, 5.2), its commit's message is `../protocol/tooling.md`, 4's, and a member that brings main's commits is merged. Its mechanism, `combine`, is `merge-queue.md`'s M1.

- **Design lags:**
  - 4: `../domain/forge/change/repairs.md` makes a resolution, but a goal has no way to ask for main's tip without a change, and a catch-up change lands as any change does.
  - 5: `../domain/agent/charter.md`, 4 says nothing of a change's commits, of its title and description becoming the history, or of catching up only when needed.
  - 7: `../testing/process-tier.md` has no queue story with goals whose changes squash in; `merge-queue.md`'s M2 writes it (`merge-queue.md`, 12).
- **Design departs:** none yet.
- **Open:** none.
