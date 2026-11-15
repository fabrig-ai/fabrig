# checkpoints: status

2026-11-15. Where the design lags or departs from [checkpoints.md](checkpoints.md).

- **Design lags:**
  - 2, 3: skein's `data/kv.md` has no checkpoint, and a finished snapshot deletes the segments it covers at once (section 5); jig's `protocol/store/README.md`, section 2 lays out no `checkpoints/`, and sketches an online backup by copying; jig's `protocol/store/opening.md`, section 1 knows no `checkpoints/`.
  - 4: `../client/ctl/tools.md`, section 3 has no `deployment checkpoint` act, and `temper-engine` no `checkpoint` command.
  - 5: `temper-engine` has no `restore`, and a backup is restored by copying it back (jig's `protocol/store/README.md`, section 2); jig's `protocol/store/opening.md`, section 1 resolves no restore intent and sweeps no checkpoint temporary, and its section 5 lets `--back` follow any manifest with a predecessor.
  - 6: `../redeploy.md`, 4.2 to 4.5 check over the stopped deployment, copy it as the backup, and judge the workers from that check alone; `upgrades.md`, 3.3 makes a backup a copy with the deployment stopped and section 15 leaves online backups open; `staged-upgrades.md`, sections 5 and 7 rehearse on a stopped backup.
- **Open:** none.
