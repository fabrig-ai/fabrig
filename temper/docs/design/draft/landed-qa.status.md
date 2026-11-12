# landed-qa: status

2026-11-11. Where the design lags or departs from [landed-qa.md](landed-qa.md).

Nothing of this draft is designed or built yet. It leaves `../domain/findings.md`, 8, temper's own pass, as it is (section 2).

- **Design lags:**
  - 2, 7, 9: `../../development/qa.md`, 3 starts each pass from a fresh box, and its section 8 leaves driving the pages to a later design; no design has a pass's watcher, triage or ledger.
  - 5: no design has recipes or copies; `../client/web/journeys.md`, 8.2's setup takes one parent, ends at a `reload` and has no `ctl` record, and the TUI's keys are `../client/tui/journeys.md`, 8.1's walk, a format of their own.
  - 6: skein's `testing/fakes/llm.md`, 2.4 answers a turn no script covers with a seeded answer or `done`, and a scripted call cannot take a value from the conversation; `../protocol/setup/box.md`, 5's fake backends generate their model's scripts and take none, and their fake forges start from their seed at every restart.
  - 10: `../client/web/journeys.md`, 8.1 builds from the checkout and scratches under `/tmp`, and nothing drives a page or the TUI a line at a time; `../protocol/setup/box.md`, 5 restarts a box in place with its own build, and no design starts its data with a newer one.
- **Design departs:** none yet.
- **Open:** 13.
