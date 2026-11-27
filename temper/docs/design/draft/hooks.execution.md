# hooks: execution

2026-11-27. How [hooks.md](hooks.md) gets built. It designs nothing: each goal writes its own design from the draft as it runs, and lands whole, with its own tests. Plans use the focused and fuzzy suites alone; walks come after landing, on the user's word.

## 1. The goals

| Goal | Draft parts | Needs | State |
|---|---|---|---|
| H1, hooks: `.temper/hooks/deliver`, `finish`, `merge` and `close`, each run from the version its job checks out, a link to a hook beside it taken as that hook; temper finding `merge` and `close` by a read of `.temper/hooks/` at the job's commit, and smith finding `deliver` and `finish`, a check for each moment, with `.temper/pre-pr` beside them; the rule by repository, today's convention and settings kept for repositories without hooks; older agents sent `.temper/pre-pr` alone; fabrig's scripts moved to hooks, the old paths kept as wrappers, and the reference profile naming neither setting | 1, 2, 3.1–3.3, 4, 6 | — | landed, by goal 20277: its design `../protocol/hooks.md`, smith's `domain/run-delivery.md`, 1.1, and `protocol/charter.md`, section 2 |
| H3, how long hooks may run: `.temper/hooks/config.toml`'s `timeout_seconds` per moment, read by the worker in the checkout where the hook runs; today's bounds where it's silent; the project's `hook_timeout_max_seconds` ceiling, 3,600 by default, a value above it run at the ceiling and said; the close job's deadline following the ceiling; a file that doesn't parse, a 0 or an unknown key the hook's failure, with its line; fabrig's `close` at 1,800 seconds | 3.4, 6 | H1 | not started |
| H2, the old ones removed: `.temper/pre-pr`, `merge_job`, `close_job` and `check_timeout_ms` no longer read, a stored setting reading as none and `temper ctl` refusing all three; the rule by repository gone; fabrig's wrappers removed; `goal-pre-pr` and `goal-child-pre-pr` moved to `tools/pre-pr/`, with the gate's scope, pre-PR's list and their tests | 5, 6 | H1 and H3; dev.fabrig.ai, its engine and its workers' agents running a build that carries both; CI's runner wrapper moved to `goal-pre-pr`'s new path, by ops | not started |

## 2. Order

- H1 on the user's approval (2026-11-27). H3 after H1, on the user's approval. H2 once its needs hold, on the user's word.
