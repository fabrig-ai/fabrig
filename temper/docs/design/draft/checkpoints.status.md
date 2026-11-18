# checkpoints: status

2026-11-17. Where the design lags or departs from [checkpoints.md](checkpoints.md).

- **Design lags:**
  - 6: `../redeploy.md`, 4.2 to 4.5 check over the stopped deployment, copy it as the backup, and judge the workers from that check alone; `upgrades.md`, 3.3 makes a backup a copy with the deployment stopped and section 15 leaves online backups open; `staged-upgrades.md`, sections 5 and 7 rehearse on a stopped backup. A restore's safety checkpoint needs a build that opens the deployment as it stands, so 6's going back after the first commit restores with the new build where it has upgraded the store (jig's `protocol/store/restoring.md`, 9).
- **Design departs:**
  - 3.2: a transcript whose file is missing at C is left out, its record saying so, rather than failing the checkpoint: a drop a crash cut short leaves the same, and the live store reads it as dropped (jig's `protocol/store/checkpoints.md`, 4.3). Decided by goal 5099's designer, 2026-11-16.
  - 4: the answer's held bytes are those only the checkpoint names, its copies and each linked file whose link count is one, rather than those the live store doesn't name: io's stat gives no file's identity to compare with the live store's, and the figure says what removing the checkpoint frees (jig's `protocol/store/checkpoints.md`, 4.6). Decided by goal 5099's designer, 2026-11-16.
  - 5.3: the staged generation links only the logs the checkpoint linked whose files end at their extents (ended tasks' turns logs), and copies every other log up to its extent, rather than linking each log whose file ends at its extent. The restored store appends past the extents of a live task's turns log and of every history log, and a linked file would change the checkpoint, whose files never change (2) (jig's `protocol/store/restoring.md`, 4.4). Decided by goal 6983's designer, 2026-11-17.
  - 5.3: the intent names the staged generation, the safety checkpoint and `active`'s digest, and the staged transcript tree by its fixed name, `transcripts.staged/`, rather than in its record: one restore runs at a time under the deployment's lock (jig's `protocol/store/restoring.md`, 2). Decided by goal 6983's designer, 2026-11-17.
- **Open:** none.
