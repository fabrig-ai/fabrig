# branch-reds: status

2026-11-13. Where the design lags or departs from [branch-reds.md](branch-reds.md).

Nothing of this draft is designed or built yet: its execution file's milestones write the design as they run.

- **Design lags:**
  - 3 and 4: no stored family keeps a goal's reds (`../protocol/store.md`, 2 and 4); `../domain/agent/tools.md`, 2 gives a goal's coordinator no tool to record or end one, and `../protocol/wires.md`, 6 no host tool for it.
  - 5 and 6: smith's `domain/run.md`, 8.1 tells the host only that checks started and ended, and `../domain/agent/README.md`, 5 returns a refused check to the agent alone; the worker reads no reds from the engine at a check's start and reports nothing at its end (`../domain/worker/checkouts.md`; `../protocol/wires.md`, 2 to 5); no change keeps observations of its checks.
  - 5, 6 and 7: `../../../../tools/pre-pr/design.md`, 5 reads `.temper/known-reds.md` from the tree with its own cap; 7 keys the passed-tree cache by tree and base alone; 4 and 6 run no failing test's process again after the run and write no results for temper.
  - 7: `../domain/findings.md`, 4 files flakes from after-landing's tiers and a gate's run, not from a delivery's check.
  - 8: `stock-charters.md`, 10.1, 10.2, 10.4 and 10.5, and the texts in `../../charters/fabrig/`, have the coordinator and the implementer edit `.temper/known-reds.md`; `delivery-checks.status.md` defines inheritance by that list.
  - 9: `../protocol/configuration/limits.md` has none of `branch_reds`, `red_key_bytes`, `rerun_processes` and `reds_read_ms`.
- **Design departs:** none yet.
- **Open:** none.
