# landed-qa: status

2026-11-12. Where the design lags or departs from [landed-qa.md](landed-qa.md).

Milestone Q1 is designed in `../testing/qa/README.md` and its documents: the watcher on main's ref, the deployment's kept builds, triage, and the ledger's landings, cases and builds. It leaves `../domain/findings.md`, 8, temper's own pass, as it is (section 2).

- **Design lags:**
  - 2, 7, 9: `../testing/qa/README.md` has no pass: no place, no pass's session, report or evidence, no waiting or resuming, and `../testing/qa/ledger.md` records no pass, finding or case's outcome; `../../development/qa.md`, 3 starts each pass from a fresh box, and its section 8 leaves driving the pages to a later design.
  - 5: no design has recipes or copies; `../client/web/journeys.md`, 8.2's setup takes one parent, ends at a `reload` and has no `ctl` record, and the TUI's keys are `../client/tui/journeys.md`, 8.1's walk, a format of their own.
  - 6: skein's `testing/fakes/llm.md`, 2.4 answers a turn no script covers with a seeded answer or `done`, and a scripted call cannot take a value from the conversation; `../protocol/setup/box.md`, 5's fake backends generate their model's scripts and take none, and their fake forges start from their seed at every restart.
  - 8: no design files a pass's findings as one goal's request.
  - 10: `../client/web/journeys.md`, 8.1 builds from the checkout and scratches under `/tmp`, and nothing drives a page or the TUI a line at a time; `../protocol/setup/box.md`, 5 restarts a box in place with its own build, and no design starts its data with a newer one.
- **Design departs:** none yet.
- **Open:** 13.
