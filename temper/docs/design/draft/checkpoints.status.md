# checkpoints: status

2026-11-16. Where the design lags or departs from [checkpoints.md](checkpoints.md).

- **Design lags:**
  - 5.3: `temper-engine` has no `restore`, and a backup is restored by copying it back (jig's `protocol/store/README.md`, section 2); jig's `protocol/store/opening.md`, section 1 resolves no restore intent, and its section 5 lets `--back` follow any manifest with a predecessor.
  - 6: `../redeploy.md`, 4.2 to 4.5 check over the stopped deployment, copy it as the backup, and judge the workers from that check alone; `upgrades.md`, 3.3 makes a backup a copy with the deployment stopped and section 15 leaves online backups open; `staged-upgrades.md`, sections 5 and 7 rehearse on a stopped backup.
- **Design departs:**
  - 3.2: a transcript whose file is missing at C is left out, its record saying so, rather than failing the checkpoint: a drop a crash cut short leaves the same, and the live store reads it as dropped (jig's `protocol/store/checkpoints.md`, 4.3). Decided by goal 5099's designer, 2026-11-16.
  - 4: the answer's held bytes are those only the checkpoint names, its copies and each linked file whose link count is one, rather than those the live store doesn't name: io's stat gives no file's identity to compare with the live store's, and the figure says what removing the checkpoint frees (jig's `protocol/store/checkpoints.md`, 4.6). Decided by goal 5099's designer, 2026-11-16.
- **Open:** none.
