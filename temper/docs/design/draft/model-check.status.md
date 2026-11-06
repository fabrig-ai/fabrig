# model-check: status

2026-11-06. Where the design lags or departs from [model-check.md](model-check.md).

- **Design lags:**
  - 2, *Who pays*: `../client/README.md`, 3.5, *The try*, describes the person's cost but gives the pane no "one short message" line once the trial goes. The running line in `temper-client-view/src/pages/model_check.rs` draws "Trying: STEP" alone; the draft's spend notice is absent from both design and code.
  - 3: `../client/README.md`, 3.5: the Model step's region, named "Forge", holds the forge's line alone, and Model's line stands in Ready's pane, named "Forge and model", where 3 has the pane hold both lines from the Model step on. The request `model-check-pane` draws Ready alone.
- **Design departs:**
  - 2, *Leaving*: `../domain/trial.md`, 3.1 and 5: leaving a page cancels nothing, but a restart cancels every live trial before any run can be claimed and restores no row. This keeps a page's departure from spending or stopping work, and meets the request that a restart spend no second turn; its cost is losing a failed trial's line. Decided by the model-check plan's cut.
  - 2: `../domain/trial.md`, 2, 6.2 and 8: a second account's trial may answer as another account, since it names no model and must select as a new chat does. Its `took` names the account that answered; that account's row holds the line. Decided by the model-check plan's cut.
  - 2: `../domain/trial.md`, 8: a deployment's accounts aren't tried, since only the accounts page watches their document and the trigger reads the person's accounts on the welcome or Connect a model. Decided by the model-check plan's cut.
  - 2, *Its end*, and 3: `../domain/trial.md`, 3.2: "first committed text" is the first committed turn, since a trial's run has no tools and the root reads turns without their bytes. Decided by the model-check plan's cut.
  - 2: `../domain/trial.md`, 6: the client starts the trial, since the engine sees no "became usable" and ctl must not spend a turn. Decided by the model-check plan's cut.
  - 3: `../client/README.md`, 3.5, *A way's page*: a key goes on to Ready once it is added and a model usable, where 3 has nothing on the way's page wait first, because a key can be refused and its page says why. Decided by the cut of that request's landing, under the user's "Decide, don't ask".
- **Open:**
  - 5: an act on Settings' accounts.

`../domain/trial.md`, 2 to 7, and `../client/README.md`, 3.5, carry sections 2 to 4 apart from the gaps and departures above; the draft remains in place. The code's concurrent-account and wait-reason gaps are in `../domain/trial.status.md`, 6. No outcome of the request's real-account walk is supplied for retirement.
