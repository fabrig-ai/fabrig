# hooks: execution

2026-11-27. How [hooks.md](hooks.md) gets built. It designs nothing: each goal writes its own design from the draft as it runs, and lands whole, with its own tests. Plans use the focused and fuzzy suites alone; walks come after landing, on the user's word.

## 1. The goals

| Goal | Draft parts | Needs | State |
|---|---|---|---|
| H1, hooks: `.temper/hooks/deliver`, `finish`, `merge` and `close`, each run from the version its job checks out, a link to a hook beside it taken as that hook; temper finding `merge` and `close` by a read of `.temper/hooks/` at the job's commit, and smith finding `deliver` and `finish`, a check for each moment, with `.temper/pre-pr` beside them; the rule by repository, today's convention and settings kept for repositories without hooks; older agents sent `.temper/pre-pr` alone; fabrig's scripts moved to hooks, the old paths kept as wrappers, and the reference profile naming neither setting | 1 to 4, 6 | — | not started |
| H2, the old ones removed: `.temper/pre-pr`, `merge_job` and `close_job` no longer read, a stored setting reading as none and `temper ctl` refusing either; the rule by repository gone; fabrig's wrappers removed; `goal-pre-pr` and `goal-child-pre-pr` moved to `tools/pre-pr/`, with the gate's scope, pre-PR's list and their tests | 5, 6 | H1; dev.fabrig.ai, its engine and its workers' agents running a build that carries H1; CI's runner wrapper moved to `goal-pre-pr`'s new path, by ops | not started |

## 2. Order

- H1 on the user's approval. H2 once its needs hold, on the user's word.
