# model-check: status

2026-11-11. Where the design lags or departs from [model-check.md](model-check.md).

- **Design departs:**
  - 2, *Leaving*: `../domain/trial.md`, 3.1 and 5: leaving a page cancels nothing, but a restart cancels every live trial before any run can be claimed and restores no row. This keeps a page's departure from spending or stopping work, and meets the request that a restart spend no second turn; its cost is losing a failed trial's line. Decided by the model-check plan's cut.
  - 2: `../domain/trial.md`, 2 and 4: a later account's trial names its account and the charter's first model offered with it, else the first it serves, where 2 has every trial name no `llm`. The first usable account's still names none, for 2's reason, and is the one a new chat takes; a later one would otherwise answer as an earlier account, giving no proof of the account just connected (the user, 2026-11-07, the request `ui-trial-account`). Each retry judges the offer again, so it stays on the account retried even when another has since become usable. Decided by the ui-trial-account plan's cut.
  - 2: `../domain/trial.md`, 8: a deployment's accounts aren't tried, since only the accounts page watches their document and the trigger reads the person's accounts on the welcome or Connect a model. Decided by the model-check plan's cut.
  - 2, *Its end*, and 3: `../domain/trial.md`, 3.2: "first committed text" is the first committed turn, since a trial's run has no tools and the root reads turns without their bytes. Decided by the model-check plan's cut.
  - 2: `../domain/trial.md`, 6: the client starts the trial, since the engine sees no "became usable" and ctl must not spend a turn. Decided by the model-check plan's cut.
  - 3: `../client/pages/README.md`, *A way's page*: a key goes on to Ready once it is added and a model usable, where 3 has nothing on the way's page wait first, because a key can be refused and its page says why. Decided by the cut of that request's landing, under the user's "Decide, don't ask".
- **Open:**
  - 5: an act on Settings' accounts.

`../domain/trial.md`, 2 to 7, and `../client/pages/README.md`, carry `model-check.md`, sections 2 to 4 apart from the departures above; the draft remains in place. The code's concurrent-account and wait-reason gaps are in `../domain/trial.status.md`, 6. No outcome of the request's real-account walk is supplied for retirement.
