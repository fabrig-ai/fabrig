# branch-reds: execution

2026-11-13. How [branch-reds.md](branch-reds.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| B1, the records and the check reading them: the stored family and its bounds; the coordinator's host tool to record and end a red; the worker reading a goal's reds at each check's start, filtering them by ancestry and writing `temper/reds`; the reference's check reading it beside the tree's list, its cache keyed on its digest; reds ended by the tool and the goal's end; the charters' words; the worker's wire and the host tools moved and bridged | 1 to 5, 6's second and third points, 8's first point, 9 to 11 | temper/dev | none | after landing, on a staging deployment: a coordinator records a red, the change's next check passes it, a later record past the change's base doesn't count | done, by goal 2476 (`../domain/forge/reds.md`; jig's `domain/hosts/hub.md`, 10; `../../../../tools/pre-pr/design.md`, 5 and 7); its walk not yet run, and its redeploy puts the engine first (jig's `protocol/channel/bridges.md`, 6.3) |
| B2, what the checks observe: the check's results file; the worker reporting it at each check's end, refused checks included; passes kept with the change and ending reds when it lands; the process run again after the run, flakes filed as findings | 6, 7, 9 to 11 | temper/dev | B1 | after landing: a red a change fixes ends when the change lands; a test failing under load lands as a flake finding | not started |
| B3, the list goes: `.temper/known-reds.md`, its cap and the merge job's rule removed, the check reading the file alone, the stopgap draft retired, `delivery-checks.md`'s status brought in line | 8 | temper/dev | B2, and every goal started before B1 landed ended | none: the check's and the merge job's tests | not started |

## 2. Order

- **B1 first:** the check reads records only once temper keeps them, and the charters stop writing the list in the same milestone.
- **B2 after B1,** observing what B1's checks run.
- **B3 last,** once no goal started before B1 still writes or carries the list; until then the stopgap (`known-reds-merge.md`) keeps settling it.
- **Section 12's Later items** are no milestone's.
