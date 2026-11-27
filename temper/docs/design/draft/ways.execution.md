# ways: execution

2026-11-26, revised 2026-11-27 for the second pass. How [ways.md](ways.md) gets built, in increments that each land whole and can be deployed alone. It designs nothing: each goal writes its own design from the draft as it runs, and lands with its own tests. Plans use the focused and fuzzy suites alone, world tests among them; walks come after landing, on the user's word. Each increment reaches dev.fabrig.ai only after staging has run it, on the user's word, by ops' steps.

## 1. The goals

| Goal | Draft parts | Needs | State |
|---|---|---|---|
| W1, names: a name on every template, charter and policy, reserved and unique, with the stock's names; `temper ctl` and the served records taking names beside numbers; areas naming a kind by name; a deployment's items named when set | 9 | — | not started |
| W2, `.temper/` guarded: the read of `.temper/` whole from one landing tip, kept as the project's repository revision, with its triggers and its last good revision; the landing check; the owner's sign-off as a project setting, off by default; each file's `format`, every shipped format read and a newer one refused by name | 8.1, 8.5, 8.6, 8.8 | W1 | not started |
| W3, the project's words: jig's project-scoped overlay and a charter's identity with the project; each new task taking the newest repository revision; `.temper/charters/` with `+++` front matter, `base`, `replaces` and `format`; runs recording their sources; `settings show charter NAME --project`; `charter copy`; fabrig's overlay words moved to `.temper/charters/` and its common rules to `AGENTS.md` | 6, 10, 11 | W2 | not started |
| W4, ways per piece of work: the way and its shapes but `pair`; Hand it over and Milestone; `ways.toml` all or nothing, `ways/` and `way copy`; starts taking `--way`, recording the way and its values in one step with admission; children's projections; work defaults; the work's own rule wherever criteria are computed; a floor's review on kind 3's reviewer; `charters`, `reaches`, `questions`, `budget` and `landing` but `reviews`, `approvals` and `share`; kind 7's `way` read as the default until `ways.toml`; the stock ways' project-wide items retired, today's stock rule 1 written out for existing projects; a standing role starting only its shape's default way and setting no value; dev.fabrig.ai's lean goal as fabrig's `milestone` way | 3.1 to 3.4, 4, 5, 7.3, 8.2, 8.3, 10, 11 | W1, W3 | not started |
| W5, standing roles' grants: ways, shapes and values a role may start and set, checked at each start, delegation and amendment; a way bringing no role | 7 | W4 | not started |
| W6, models in a way: `models` per agent by name, effort and speed, ignored where the deployment lacks the model and said so in the preview | 3.1, 5.2, 6.4 | W4; the speeds milestone's second goal | not started |
| W7, priority and deadline in a way, within the template's and policy's bounds | 3.1, 5.2 | W4 | not started |
| W8, reviews and approvals added: `landing.reviews`, each a lens and its charter, blocking or advisory; `landing.approvals`; both in the work's own rule, adding only | 3.3, 5.2, 5.4 | W4 | not started |
| W9, the gallery: its entries built in (3.5), each a fixture, a world test running work through it and its stops checked against the path's derivation; How work runs showing them; Ask temper's reference of every primitive; entries whose needs haven't landed held back, named | 3.5, 12 | W4 to W8, for the entries they serve | not started |
| W10, a new project's ways: the gallery and the stock ways shown at creation and adoption; the stock ways run with nothing written; a pick written by a change, "Add .temper/ with your ways"; Hand it over preselected | 8.7 | W9 | not started |
| W11, upgrades reviewed: after an upgrade, Ask temper reads a project's `.temper/` beside what the release changed and proposes a change, edit by edit with reasons | 11 | W9 | not started |
| W12, sharing a landing: `landing.share`, `alone`, `with`, `at_most` and `combine`, in projects that land by turn; the turn formed from the waiting changes whose ways allow one another; the night-batch entry | 3.3, 3.5 | W9 | not started |
| W13, a milestone landing whole, and catch-ups by way: `[milestone] whole`, the milestone's own branch, its goals landing into it and its closer landing it whole; `landing.milestone`; `catch_up`, `after-each-landing` made by temper through the merge hook; the one-landing-milestone entry | 3.2, 3.3, 3.5, 5.2 | W9; [hooks.md](hooks.md)'s merge hook | not started |

## 2. Order

- After the user's approval of each milestone.
- W1, W2, W3, then W4: the core. Then W5 to W8, in any order, each needing W4 alone (W6 also the speeds milestone). Then W9, the gallery, its entries released as their needs land, W10 and W11. W12 and W13, the landing, last.
- Pair's shape needs its own draft first. Until then the stock and the gallery offer Hand it over and Milestone, and a chat keeps its template.
- The hooks are [hooks.md](hooks.md)'s own goals, already running.
