# skein-world-costs: what every world pays to count its heap and keep its wire's trace

The request: `inputs/request.md`. `skein/docs/foundation/testing-strategy.md`, 8.1's second question for two things that belong: make them cheaper at the cause. No budget, timeout or build setting changes.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `counting` | `tasks/counting.md` | | `Counting` with the same counts, one thread-local reached once per call; the six tests read before and after |

More parts follow in later pushes: `wire-trace` and `world-helpers`.

Critical path as it stands: `counting`.
