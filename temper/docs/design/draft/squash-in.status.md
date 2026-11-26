# squash-in: status

2026-11-26. Where the design lags or departs from [squash-in.md](squash-in.md).

Sections 2, 3, 5 and 6 are designed: a change into a goal's branch squashes by its train's `combine` (`../domain/landing-prepare.md`, 1.2), its commit's message is `../protocol/tooling.md`, 4's, and a member that brings main's commits is merged. Its mechanism, `combine`, is `merge-queue.md`'s M1. `merge-queue.md`, section 5's words are the stock charters' (`../domain/agent/charter.md`, 4; `../protocol/configuration/stock.md`'s charter 2 and 5 rows), by S2.

- **Design lags:**
  - 4: `../domain/forge/change/repairs.md` makes a resolution, but a goal has no way to ask for main's tip without a change, and a catch-up change lands as any change does.
  - 7: `../testing/process-tier.md` has no queue story with goals whose changes squash in; `merge-queue.md`'s M2 writes it (`merge-queue.md`, 12).
- **Design departs:**
  - 2 and 5: a goal's own runs write its branch too, as its design writers (`../domain/forge/branches.md`, 3): each delivery is a commit on it as delivered, its moved tip merged first and never forced, where the draft's only writers are its train's turns and merges of main, and its "designer" a change's maker. The stock's designer and coordinator are those runs, so their words write each delivery's title and body for the history and deliver at any point (`../domain/agent/charter.md`, 4), keeping `stock-charters.md`'s design delivered straight to the goal's branch (goal 2231's designer, 2026-11-13).
  - 3: a change's commit carries the change's own trailers too, before its four, from the last paragraph of its producer's words or of its head commit's message, but for those four, and its body is cut first to keep them within the message's bound (`../protocol/tooling.md`, 4, TRAILERS and *The bound*), so that a look request's `Look-request` and `Look-landed` reach main (`../client/web/look-requests.md`, 3). The rest of the change's message stays with its head, in its pull request. The user approved it on 2026-11-25.
- **Open:** none.
