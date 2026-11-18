# redeploy: status

2026-11-18. Where the code lags or departs from [redeploy.md](redeploy.md).

- **Not yet:** 4.1, step 10, with 4.6, 6.3 and 8: the staged build's verdict is read only at the stop, not once over the check's checkpoint before it, so a build without `upgrade --workers` is refused only after the engine has stopped [in flight: change 8413]
- **Not yet:** 4.2: the stop's verdict copies the checkpoint's snapshot, segments and secrets byte by byte into its scratch, rather than linking the snapshot and sealed segments [in flight: change 8414]
