# checkpoints: execution

2026-11-15. How [checkpoints.md](checkpoints.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Each is a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| C1, taking one: the cut, a barrier in the component's ordered work recording both stores' positions, the transcripts' committed places and the logs' extents; skein-kv's holds on a selected snapshot's name and segments, and the component's on transcript and log removals; links for what nothing writes again and bounded copies of the prefixes still appended to; the record, a fresh generation-1 manifest, the temporary, its rename and its sweep; a unique name; the newest `store.checkpoints.keep` kept, a held lock skipped; `temper-engine checkpoint --data`, `--hold NAME -- COMMAND` and `temper ctl DIR deployment checkpoint`. Its tests: commits and drops racing the cut, a snapshot finishing during it, a refused link, a vanished source, power loss at each step of 3.3 | 2, 3, 4, 5.1 | store-shape's S7 on main (`logs/` in the store) | after landing, on a staging deployment serving a goal: a checkpoint passes `temper-engine upgrade --check`, its files match its record, and its figures (links, bytes copied, time, bytes held) are reported | not started |
| C2, restore: `temper-engine restore NAME` with the safety checkpoint, the staged generation and transcript tree, the restore intent and its roll-forward at every open, the restored manifest refusing `--back`. Its tests: power loss and failure at each rename and at the intent reopen one matching pair, and restoring the safety checkpoint brings the later state back | 5.3 | C1 | after landing, on a staging deployment: a restored checkpoint opens with the state at C by the referee, and restoring its safety checkpoint brings the later state back | not started |
| C3, the redeploy: the full check and its worker verdict over a checkpoint while the old build serves, the drain before the stop for a verdict of no, the stop's checkpoint as the backup, its inputs checked again over it, going back after the first commit by restore (`../redeploy.md`, 4.2 to 4.5; `staged-upgrades.md`, 5 and 7) | 6 | C1, C2 | dev.fabrig.ai's next redeploy after landing: the check runs while the old build serves, and no deployment is copied whole at the stop | not started |

## 2. Order

C1 first, once S7 has landed, since a checkpoint links S7's task logs and C1 changes the same store component. C2 and C3 follow in that order. C3 changes how dev.fabrig.ai is redeployed, so it reaches dev with the redeploy that follows its landing, on the user's yes.
