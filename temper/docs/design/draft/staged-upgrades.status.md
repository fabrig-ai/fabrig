# staged-upgrades: status

2026-11-22. Where the design lags or departs from [staged-upgrades.md](staged-upgrades.md).

- **Design lags:**
  - 5, 7: `../protocol/binaries.md` still speaks of patch lines, decoding by version's kept text and a pending fresh patch in the worker verdict. Jig's `protocol/store/opening-rehearsal.md`, 1 decodes registered values from the registry's layout texts and other records from their current formats; starts and rehearsals report jobs and their counts.
  - 1, 2, 8: `first-release.md`, 5.4 still has dev.fabrig.ai start afresh when a stored format changes until the baseline. Dev's store is kept at the floor; staging alone is made again at dev's build for each drill.
- **Design departs:**
  - 5 to 7: the drill's script, its workload, its referee and the restore drill are the deployments' operator's, built outside this repository, which designs and builds only the steps they run on each process: the rehearsal over a checkpoint, `upgrade --check` (`../protocol/binaries.md`, section 2), the backup, a checkpoint (`../protocol/checkpoints.md`, 2.1 and 3), and the restore (`../protocol/checkpoints.md`, 2.5). Because the user decided, 2026-11-20, that the cross-host redeploy and its drills are internal tooling that belongs outside fabrig.
  - 5, 7, step 2: the rehearsal on dev.fabrig.ai's data runs over a checkpoint taken while the old build serves, and the backup is a checkpoint taken at the settled stop of every redeploy, rather than a copy taken with the engine stopped (`../protocol/checkpoints.md`, 2.1 and 3; `../protocol/binaries.md`, section 2). Because the user's `archive/checkpoints.md`, 6 (2026-11-15) moves the check before the stop and makes the backup a checkpoint, so that no deployment is copied whole.
  - 7, step 5: going back after the new generation's first commit restores the stop's checkpoint with the new build, then starts the previous build, rather than restoring with the previous build (`../protocol/checkpoints.md`, 2.5). Because the restore's safety checkpoint opens the deployment as it stands, which a build behind its levels refuses (jig's `protocol/store/restoring.md`, 4.3; goal 7514's designer, 2026-11-18).
- **Open:** none.
