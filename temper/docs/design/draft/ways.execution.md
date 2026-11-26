# ways: execution

2026-11-26. How [ways.md](ways.md) gets built: a sketch, since the next pass (its section 12) may reshape it. It designs nothing: each goal writes its own design from the draft as it runs, and lands whole, with its own tests. Plans use the focused and fuzzy suites alone; walks come after landing, on the user's word.

## 1. The goals

| Goal | Draft parts | Needs | State |
|---|---|---|---|
| W1, names: a name on every template, charter and policy, reserved and unique, with the stock's names; `temper ctl` and the served records taking names beside numbers; areas naming a kind by name; a deployment's items named when set | 9 | — | not started |
| W2, `.temper/` guarded, then hooks: the read of `.temper/` whole from one landing tip, kept as the project's repository revision, with its triggers and its last good revision; the landing check and the owner's sign-off; then `.temper/hooks/deliver`, `merge` and `close` by moment, `close` run at the landing tip where it holds a landing, the legacy convention and settings read until a repository has hooks; fabrig's scripts moved, wrappers kept, and the two CI wrappers retired once their callers move | 8.1, 8.4 to 8.6, 10, 11 | W1 | not started |
| W3, the project's words: jig's project-scoped overlay and a charter's identity with the project; each new task taking the newest repository revision; `.temper/charters/` with `+++` front matter, `base` and `replaces`; runs recording their sources; `settings show charter NAME --project`; `charter copy`; fabrig's overlay words moved to `.temper/charters/` and its common rules to `AGENTS.md` | 6, 10, 11 | W2 | not started |
| W4, ways per piece of work: the way and its shapes but `pair`; Hand it over and Milestone; `ways.toml`, `ways/` and `way copy`; ways' names; starts taking `--way`, recording the way and its values in one step with admission; children's projections; work defaults; the work's own rule wherever criteria are computed; a floor's review on kind 3's reviewer; `reaches`, `questions`, `charters` and `budget` by way; kind 7's `way` read as the default until `ways.toml`; the stock ways' project-wide items retired, today's stock rule 1 written out for existing projects; a standing role starting only its shape's default way and setting no value; dev.fabrig.ai's lean goal as fabrig's `milestone` way | 3, 4, 5, 7.3, 8.2, 8.3, 10, 11 | W1, W3 | not started |
| W5, standing roles' grants: ways, shapes and values a role may start and set, checked at each start, delegation and amendment; a way bringing no role | 7 | W4 | not started |

## 2. Order

- After the next pass, and the user's approval of each milestone.
- W1, then W2, then W3, then W4, then W5. The guard and the read land before any hook is found by its moment.
- Pair's shape needs its own draft built first. Until then the stock offers Hand it over and Milestone, and a chat keeps its template.
