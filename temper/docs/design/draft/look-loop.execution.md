# Look requests on walks: execution

The milestone that carries [look-loop.md](look-loop.md) into the design and the code: four goals, with one goal from outside the draft that the third stands on.

| Goal | Executes | Needs | Its walk | State |
|---|---|---|---|---|
| Look requests drawn on walks, landed from a fixed branch: the request drawn on a walk, its commit's walk and author, the word that starts its goals, `docs/look/NAME`, the goals' words, the walk landed as sources, `Look-landed`; `../client/web/look-requests.md`, `../client/web/journeys.md`, 7, `../client/tui/journeys.md`, 10, `../client/journey-writer.md`, 6, and both `AGENTS.md` files brought in line | 2, 3, 4.1, 4.4 | none | none: design text | landed |
| A script compares a look request with what landed: `scripts/look-compare.sh` and the journeys tool's `look-compare` command, its three-way pages, its rules, its rows and exits; `../client/web/look-requests.md`, 3 and 5, and `../testing/tools.md`, 2, citing it | 4.2, 4.3, 7 | the first | the next look request's goal runs it | not started |
| Printed journey pages can't land beside other edits: `tools/pre-pr/design.md`, 4, check 8, built as written | not this draft's | none | none | not started |
| Each landed look request is rendered and checked: skein's driver reads elements, styles and boxes; `temper-client-journeys`' render check; the writer's run, the journeys tool's `writer` command, its timer's rule, checks landed requests and writes `looks.html`; the three gates; the refresh's drop rule; the timer's command | 5.1 to 5.4, 6, 7 | the second and the third | the writer's first run after it lands, on the requests landed since | not started |
| Old journeys that look requests left can be retired: retirement's candidates in `looks.html`; the delivery check's rule for a journey's directory deleted whole | 5.5, 7 | the fourth | none | not started |

## Order

One milestone. The first goal is design text alone and comes first, so that the brief says how a request is drawn and landed as soon as it can. Look requests don't wait for the rest: each is a goal of its own beside the milestone (3.1), from the first request on.

- **Until the second lands,** a request's goal compares by reading, as `../client/web/look-requests.md`, 3, says.
- **Until the fourth lands,** no render check runs. Its first run checks every request landed since that carries `Look-landed` and has its `docs/look/NAME`, so none is skipped.
- **The fifth** is housekeeping, last.
- **The third** has no need, and runs beside the first.

Plans use the focused and fuzzy suites alone: the render check's browser cases are written and compiled in the fourth, and run after landing.

## Acceptance

- **The first:** the design documents say what `look-loop.md`, 2, 3, 4.1 and 4.4, says; the citations pass; and `look-loop.status.md` lists no gap there.
- **The second:** the comparison's focused tests (7) pass.
- **The third:** the delivery check's focused tests pass: a mixture refused, written files alone passed.
- **The fourth:** the run's and the refresh's focused tests (7) pass with a fake check; the browser cases compile; the delivery check refuses `looks.html` beside a source and lets it through alone.
- **The fifth:** the candidates' and the whole-directory rule's focused tests (7) pass.
