# Look requests on walks: execution

The milestone that carries [look-loop.md](look-loop.md) into the design and the code: three goals, in order.

| Goal | Executes | Needs | Its walk | State |
|---|---|---|---|---|
| A look request carries its walk and lands from a fixed branch: the request drawn on a walk, its commit's walk and author, the word that starts its goals, `docs/look/NAME`, the goals' words, the walk landed as sources, `Look-landed`; `../client/web/look-requests.md`, `../client/web/journeys.md`, 7, `../client/tui/journeys.md`, 10, and both `AGENTS.md` files brought in line | 2, 3, 4.1, 4.4 | none | none: design text | not started |
| A script compares a look request with its landing: `scripts/look-compare.sh` and the journeys tool's `look-compare` command, its three-way pages, its rules, its rows and exits; `../client/web/look-requests.md`, 3 and 5, and `../testing/tools.md`, 2, citing it | 4.2, 4.3, 7 | the first | the next look request's goal runs it | not started |
| Each landed look request is rendered in Chromium after landing: skein's driver reads elements, styles and boxes; `temper-client-journeys`' render check; the writer's run, the journeys tool's `writer` command, checks landed requests, writes `looks.html` and skips an unmoved tip; the three gates; the refresh's drop rule; the timer's command | 5, 6, 7 | the second | the writer's first run after it lands, on the requests landed since | not started |

## Order

One milestone, its goals in the table's order, each after the one before. The first is design text alone, so that a request drawn while the others run is drawn on a walk already. Plans use the focused and fuzzy suites alone: the render check's browser story is written and compiled in the third, and runs after landing.

## Acceptance

- **The first:** the design documents say what `look-loop.md`, 2, 3, 4.1 and 4.4, says; the citations pass; and `look-loop.status.md` lists no gap there.
- **The second:** the comparison's focused tests (7) pass; a run on a scratch repository holding one request and its landing prints the account's rows.
- **The third:** the run's and the refresh's focused tests (7) pass with a fake check; the browser story compiles; the delivery check refuses `looks.html` beside a source and lets it through alone.
