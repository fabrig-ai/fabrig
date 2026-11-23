# testing: status

2026-11-23. Where the tree's code lags or departs from [testing.md](testing.md).

- **Not yet:** 2: the harness fails a step's expiry at once, before the starvation rule reads it, reads a kit `Timeout` in a find as an absent target, and keeps no bound on a red story's capture and close after its failure. [in flight: change 13987]
