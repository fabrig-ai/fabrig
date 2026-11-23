# landing-update-loop: status

2026-11-17. Where the design lags or departs from [landing-update-loop.md](landing-update-loop.md).

- **Design lags:** none.
- **Design departs:** 2: a stale refusal at the check after a send of the update's own, its answer lost, is a lost write: it is recovered against its pin as the draft's lost write is, and its next check, if the base has gone on, refuses it before any send, which then replans. The client cannot tell a copy of its own from another hand there (`../domain/forge/updates.md`, 1; skein's `forge/client-effects.md`, 1.5; the goal's designer, 2026-11-17). 2: where both its head and its base moved, the refusal names the head (`Stale`), since a moved head decides the change's next step (skein's `forge/client-effects.md`, 1.4; the goal's designer, 2026-11-17).
- **Open:** none.
