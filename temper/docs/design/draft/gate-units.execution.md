# gate-units: execution

2026-11-18. How [gate-units.md](gate-units.md) gets carried out: in small steps, each landing on its own. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. The figure of the draft's section 6 is read after each lands, and is no plan's.

## 1. The milestones

Each is a goal, one change of one concern.

| Milestone | Draft parts | Needs | Its figure after landing | State |
|---|---|---|---|---|
| A1, base checks build their tests' targets: `tools/tiers/one-process --alone` listing each flagged binary's own target, `tools/tiers/references.md`, 5, following | 3 | none | a gate's base-only units against section 2's 2.9 to 24.6 GiB | not started |
| A2, the agent fixtures in a kit of their own: `temper-fixtures`' `agent` module, its dependencies and its users | 4 | none | the tests that link `temper-fixtures` | not started |
| A3, the landing fixtures in a kit of their own: `temper-fixtures`' `landing` module, its dependencies and its users | 4 | A2 | the same | not started |
| A4, the `parent` test's harness: the process guard and its tests in a small crate under `temper-box-kit` | 5 | none | `parent`'s executable against its 412 MiB | not started |

## 2. Order

- **Each milestone is a goal,** started once it's approved.
- **No priority mark:** builds aren't one of v0.1.0's critical pieces. Its plans start when places are free and marked work isn't about to arrive.
- **A1, A2 and A4 side by side;** A3 after A2, since both change `temper-fixtures`.
