# Look requests on walks: execution

The milestone that carries [look-loop.md](look-loop.md) into the design and the code: three goals, in order.

| Goal | Executes | Needs | Its walk | State |
|---|---|---|---|---|
| A look request carries its walk and lands from a fixed branch: the request drawn on a walk, its commit's walk and author, the word that starts its goals, `docs/look/NAME`, the goals' words, the walk landed as sources; `../client/web/look-requests.md`, `../client/web/journeys.md`, 7, `../client/tui/journeys.md`, 10, `../client/journey-writer.md`, 6, and both `AGENTS.md` files brought in line | 1, 2, 3, 4.1, 6 | none | none: design text | not started |
| A script compares a look request with its landing: `scripts/look-compare.sh`, its three-way pages, its rules, its account rows and exits, and `../client/web/look-requests.md`, 3 and 5, citing it | 4.2, 4.3, 7 | the first | the next look request's goal runs it | not started |
| Each landed look request is rendered in Chromium after landing: skein's driver reads a computed style; `temper-client-journeys`' render check; the writer's run checks newly landed requests and writes `looks.html`, a written file to the delivery check and the merge job; the timer's command | 5, 7 | the second | the writer's first run after it lands, on the requests landed since | not started |

## Order

One milestone, its goals in the table's order, each after the one before. The first is design text alone, so that a request drawn while the others run is drawn on a walk already. Plans use the focused and fuzzy suites alone: the render check's browser story is written and compiled in the third, and runs after landing.

## Acceptance

- **The first:** the design documents say what `look-loop.md`, 1 to 4.1 and 6, says; the citations pass; and `look-loop.status.md` lists no gap there.
- **The second:** the comparison's focused tests (7) pass; a run on a scratch repository holding one request and its landing prints the account's rows.
- **The third:** the run's focused tests (7) pass with a fake check; the browser story compiles; the delivery check refuses `looks.html` beside another file and lets it through alone.
