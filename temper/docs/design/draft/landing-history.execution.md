# landing-history: execution

2026-11-13. How [landing-history.md](landing-history.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| H1, the events: the stored family, each kind written by the step that decides it under `queue` and `turn`, both times where two exist, kept by age and count apart from the change and landing rows, `landing history` and `change show` of a change whose row went | 1 to 6, 8, 9, 10 | temper/dev | none | after landing, on a staging deployment: a few goals land, one through a resolution; their history read back after their change rows are gone | landed, goal 2474; its walk after landing |
| H2, where the time went: `landing times`, its states, their order and totals, per landing under `turn` | 7, 10 | temper/dev | H1 | after landing, on the staging deployment: `landing times` over a day of landings, set against the changes' own pull requests | not started |

## 2. Order

- **H1 first:** the events are what H2 reads, and are worth having alone.
- **H2 after H1,** reading only what H1 writes.
- **Section 11's Later items** are no milestone's.
