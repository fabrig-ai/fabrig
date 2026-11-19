# redeploy: status

2026-11-19. Where the code lags or departs from [redeploy.md](redeploy.md).

- **Not yet:** 4.1, step 10, and 6.3: a failed verdict before the stop always says that a build without `upgrade --workers` is redeployed with `--together`, even where the staged build has it and refused; no verdict's count of files linked and copied is noted, nor its absence [in flight: change 9154]
