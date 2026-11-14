# branch-reds: status

2026-11-13. Where the design lags or departs from [branch-reds.md](branch-reds.md).

Milestone B1 is designed, by goal 2476. `../domain/forge/reds.md` covers a goal's reds, the `red` tool (`../domain/agent/tools.md`), their end by the tool and with the goal, and the worker's read at each check's start with the files it writes. The worker's read goes through an application's ask on jig's channel (jig's `domain/hosts/hub.md`, 10; `../protocol/channel.md`; `../protocol/wires.md`, 6). The families are in `../protocol/store.md`, 2, and the bounds in `../protocol/configuration/limits.md`. The reference's check reads the file beside the tree's list, its cache keyed on the file's digest (`../../../../tools/pre-pr/design.md`, 5 and 7), and fabrig's charter texts record reds with the tool.

- **Design lags:**
  - 5 and 6: smith's `domain/run.md`, 8.1 tells the host only that checks started and ended, and `../domain/agent/README.md`, 5 returns a refused check to the agent alone. The worker reports nothing at a check's end, and no change keeps observations of its checks, so no red ends by what lands (`../domain/forge/reds.md`, 4).
  - 6 and 7: `../../../../tools/pre-pr/design.md`, 4 and 6 run no failing test's process again after the run, and write no `temper/check-results`.
  - 7: `../domain/findings.md`, 4 files flakes from after-landing's tiers and a gate's run, not from a delivery's check.
  - 8: `.temper/known-reds.md`, its cap and the merge job's rule for it stay (`../../../../tools/pre-pr/design.md`, 5; `known-reds-merge.md`), and `delivery-checks.status.md` defines inheritance by that list.
  - 9: `../protocol/configuration/limits.md` has no `rerun_processes`.
- **Design departs:**
  - 3: a red's generation is the deployment's count of records, kept in one row, `RedNumber`, not the goal's (`../domain/forge/reds.md`, 2 and 6). It still orders a goal's records as the draft asks, with no counter per goal to keep and erase (goal 2476's designer, 2026-11-13).
  - 9: `reds_read_ms` is the checkout's, `checkout.reds_read_ms`, not among the forge's limits, since the worker is what waits for the engine's answer (`../protocol/configuration/limits.md`; goal 2476's designer, 2026-11-13).
  - 10: a hold of a goal with reds is placed on any worker, as every hold is. A worker of the previous versions writes no reds file, so its checks judge without them, where the draft places such a hold on a worker of the new version. Placement by agreed version keeps a task off a channel only for an assignment that cannot go down (`redeploys.status.md`, 4.3), and a worker of the previous versions is gone after the next workers-in-turn redeploy (goal 2476's designer, 2026-11-13).
  - 10: the read crosses as an application's ask, new kinds of `jig-worker-wire` with temper's bytes in `temper-worker-wire`, both moving with their bridges, where the draft names the worker's wire alone. A worker can ask the engine nothing outside a run's own calls, and the user ruled out riding those (the user, 2026-11-13). A worker of the new versions closes against an engine of the previous ones, so the engine goes first (jig's `protocol/channel/bridges.md`, 6.3).
- **Open:** none.
