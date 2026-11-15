# held-landing-recovery: status

2026-11-15. Where the design lags or departs from [held-landing-recovery.md](held-landing-recovery.md).

- **Design lags:** none.
- **Design departs:** 4: a push that timed out is named `failed`, class
  `undelivered`, not `timeout` (`../domain/landing-history.md`, 2.2, *A
  job's attempt*): the worker reports a failed delivery to the engine
  with its timeout only in its words, and naming it apart needs a typed
  cause on the workers' wire, which 5 leaves out with why the pushes
  timed out. `timeout` names a run past its deadline. Decided by the
  goal's design, 2026-11-15.
- **Design departs:** 6, the second story, step 4: the forge refuses to
  reopen P by hand while B's own pull request is open (Forgejo, and the
  fake forge as it does), so the story's reopen is refused, and the
  story checks that the refusal, and P's news, change nothing for B.
  Decided by the goal's design, 2026-11-15.
- **Open:** none.
