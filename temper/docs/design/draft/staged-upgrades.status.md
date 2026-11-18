# staged-upgrades: status

2026-11-18. Where the design lags or departs from [staged-upgrades.md](staged-upgrades.md).

- **Design lags:**
  - 5, 7: `../protocol/binaries.md` still speaks of patch lines, decoding by version's kept text and a pending fresh patch in the worker verdict. Jig's `protocol/store/opening.md`, 5 decodes registered values from the registry's layout texts and other records from their current formats; starts and rehearsals report jobs and their counts.
  - 5 to 7: `../redeploy.md`, sections 4 and 6 design the redeploy and its worker/task referee, but not staging's whole workload, its identity/sign-in/account/history assertions, comparison of job counts, and restore drill.
  - 1, 2, 8: `first-release.md`, 5.4 still has dev.fabrig.ai start afresh when a stored format changes until the baseline. Dev's store is kept at the floor; staging alone is made again at dev's build for each drill.
- **Design departs:**
  - 5, 7, step 2: the rehearsal on dev.fabrig.ai's data runs over a checkpoint taken while the old build serves, and the backup is the settled stop's checkpoint, taken at every redeploy, rather than a copy taken with the engine stopped (`../redeploy.md`, 4.1 and 4.2). Because the user's `checkpoints.md`, 6 (2026-11-15) moves the check before the stop and makes the backup a checkpoint, so that no deployment is copied whole.
  - 7, step 5: going back after the new generation's first commit restores the stop's checkpoint with the new build, then starts the previous build, rather than restoring with the previous build (`../redeploy.md`, 4.6). Because the restore's safety checkpoint opens the deployment as it stands, which a build behind its levels refuses (jig's `protocol/store/restoring.md`, 4.3; goal 7514's designer, 2026-11-18).
- **Open:** none.
