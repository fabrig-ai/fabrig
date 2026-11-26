# landing-liveness: status

2026-11-26. Where the design lags or departs from [landing-liveness.md](landing-liveness.md).

- **Design lags:**
  - 4.3: a train's refusal is stored on its train and named in its changes' waits and the engine's log, but not in `landing list` or a waits view (`../domain/landing-forming.md`, 6), which bounded waits' views carry (`bounded-waits.md`, 9).
  - 5.2: a held member's words say `member change C is held`, not yet naming the party whose hold it is (`../domain/landing-liveness.md`, 9).
- **Design departs:**
  - 5.1: a held landing is not checked, whoever holds it, not only a person: a held task takes no step, its hold names its way out, and the step after its release checks it (`../domain/landing-liveness.md`, 2.2). The goal's designer, 2026-11-26: the hold, a person's or one with its own bound, already says how it ends, and the check never overrides it.
- **Open:** none.
