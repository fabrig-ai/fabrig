# fleet: status

2026-11-25. Where the tree's code lags or departs from [fleet.md](fleet.md).

- **Not yet:** 2, *A job goes only where a host runs it*, *A job is claimed only where a host can take it now* and *A job that waits says why*: `job_short` is the inline cover check of `choose_job`; the core claims a job before asking the fleet for a host, so a job no host covers sits claimed in the fleet's queue; and a job task has no wait. [in flight: change 17715]



- **Not yet:** the whole document: runs on the engine's own slots under
  `smith-inline-agent`; the engine hub's hello before placement; the
  two-claim placement balance.  the point-load fences remain `vocabulary.status.md`, 1.2, and `decisions.status.md`, 1.3.
   the brief's missing sections remain `briefs.status.md`, 1.  turn-call objects remain `executors-transcripts.status.md`, 1.  the missing tools remain `executors-tools.status.md`, 1.
   the restart-oracle gaps remain `../testing.status.md`, 9.
   the pending-reasons continuation gap remains `settings-proposals.status.md`, 1.
   the settings-holder gap remains `settings-proposals.status.md`, 1.
   the ignored restore stories remain `settings.status.md`, 10.
