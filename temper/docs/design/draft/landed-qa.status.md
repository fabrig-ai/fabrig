# landed-qa: status

2026-11-12. Where the design lags or departs from [landed-qa.md](landed-qa.md).

Milestone Q2, what the first passes need of temper, is designed and
built: the drive session (`../client/drive.md`), recipes and their walk
(`../client/recipe.md`), the fake model's given scripts and strict
switch (`../protocol/setup/box.md`, 5; skein's `testing/fakes/llm.md`,
2.5), a box started again with a newer build (`../protocol/setup/box.md`,
1) and a store refused by name (`../protocol/store.md`, 5). The draft
leaves `../domain/findings.md`, 8, temper's own pass, as it is (section
2).

- **Design lags:**
  - 2, 7, 9: `../../development/qa.md`, 3 starts each pass from a fresh box, and its section 8 leaves driving the pages to a later design; no design has a pass's watcher, triage or ledger.
  - 5: no design has a setup's copy, its place, its archive or its making again, which `../client/recipe.md`, 4's hash names; no recipe is written yet.
  - 6: a scripted call cannot take a value from the conversation (skein's `testing/fakes/llm.md`, 2.4), and `../protocol/setup/box.md`, 5's fake forges start from their seed at every restart.
  - 10: its later items, as 6.
- **Design departs:**
  - 3, 10: the client a build keeps is the TUI's recording build (`temper-client` with `temper-client-shell-tui/record`, `../client/tui/journeys.md`, 2), beside the bundle's recording build: the drive session reads the TUI's frames from its saves, replayed natively, since nothing in the tree reads a pseudo-terminal's screen (`../client/drive.md`, 2.1 and 3.3). The goal's designer, 2026-11-12.
  - 10: the journeys' `again` keeps building from the checkout and recording into it, a person's command (`../client/web/journeys.md`, 8.1); the drive session and the recipe walk are what walks and drives with a given build in a given scratch directory (`../client/drive.md`, 2; `../client/recipe.md`, 5), since `again` writes into the checkout, which a QA session never touches. The goal's designer, 2026-11-12.
  - 7: the session signs the TUI in itself, by the browser's approval, when a line first needs it, so that no line and no recipe approves and a recipe walks the same from a fresh box or a copy (`../client/drive.md`, 3.3). The goal's designer, 2026-11-12.
- **Open:** 13.
