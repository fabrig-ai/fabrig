# landed-qa: status

2026-11-12. Where the design lags or departs from [landed-qa.md](landed-qa.md).

Milestone Q1 is designed in `../testing/qa/README.md` and its documents: the watcher on main's ref, the deployment's kept builds, triage, and the ledger's landings, cases and builds.

Milestone Q2, what the first passes need of temper, is designed and
built: the drive session (`../client/drive.md`), recipes and their walk
(`../client/recipe.md`), the fake model's given scripts and strict
switch (`../protocol/setup/box.md`, 5; skein's `testing/fakes/llm.md`,
2.5), a box started again with a newer build (`../protocol/setup/box.md`,
1) and a store refused by name (`../protocol/store.md`, 5). The draft
leaves `../domain/findings.md`, 8, temper's own pass, as it is (section
2).

Milestone Q3, passes on the web client, is designed in
`../testing/qa/places.md` (places, copies, their making again and the
drive session held open), `../testing/qa/passes.md` (a pass's start,
session, end, resume and report) and `../testing/qa/findings.md`
(findings, their request and its goal), with the ledger's passes,
findings and case outcomes (`../testing/qa/ledger.md`, 2.7 to 2.9 and 4.8
to 4.16) and the first recipes in `../../development/qa/setups/`.

- **Design lags:**
  - 2, 7: a pass takes the web client's cases alone; the TUI's wait in the queue until Q4 adds `tui` to `qa.json`'s `surfaces` (`../testing/qa/README.md`, 3.2).
  - 6: a scripted call cannot take a value from the conversation (skein's `testing/fakes/llm.md`, 2.4), and `../protocol/setup/box.md`, 5's fake forges start from their seed at every restart.
  - 10: its later items, as 6.
  - 5, 6, 10: the setups and their places run on the box's own Forgejo, with its runner (`../testing/qa/places.md`; `../../development/qa/setups/`), where the draft, since 2026-11-12, has the fakes alone; Q5 moves them.
- **Design departs:**
  - 3, 10: the client a build keeps is the TUI's recording build (`temper` with `TEMPER_INTERNAL_RECORD=1`, `../client/tui/journeys.md`, 2), beside the bundle's recording build: the drive session reads the TUI's frames from its saves, replayed natively, since nothing in the tree reads a pseudo-terminal's screen (`../client/drive.md`, 2.1 and 3.3). The goal's designer, 2026-11-12.
  - 10: the journeys' `again` keeps building from the checkout and recording into it, a person's command (`../client/web/journeys.md`, 8.1); the drive session and the recipe walk are what walks and drives with a given build in a given scratch directory (`../client/drive.md`, 2; `../client/recipe.md`, 5), since `again` writes into the checkout, which a QA session never touches. The goal's designer, 2026-11-12.
  - 7: the session signs the TUI in itself, by the browser's approval, when a line first needs it, so that no line and no recipe approves and a recipe walks the same from a fresh box or a copy (`../client/drive.md`, 3.3). The goal's designer, 2026-11-12.
  - 3: a kept build's `bin/fake-claude`, smith's fake of the coding agent's command line, prints no temper commit as its version; it is held by the build's manifest alone, and every temper executable of the build prints the commit (`../testing/qa/builds.md`, 3). The goal's designer, 2026-11-12.
  - 5, 7: a recipe's step that broke with no landing meaning it to fails the cases on that setup, with its finding, as section 7 has it, where section 5 says they are blocked by it: failed, they come back at the pass's resume (`../testing/qa/places.md`, 7). The goal's designer, 2026-11-12.
  - 8: a pass's request is a report, `temper/docs/development/qa/requests/pass-<P>-<R>.md`, rather than a draft in `draft/`: it lists findings, not a vision, and needs no status file (`DESIGN.md`, section 4). The deployment's command puts it on main and starts its goal, since how a request reaches main and how a goal starts are the deployment's (`../testing/qa/findings.md`, 4 and 5). The goal's designer, 2026-11-12.
  - 9: a pass session that records nothing counts its cases as blocked, so that a session that does nothing is not taken again without end; cases with no outcome go back uncounted beside a session that recorded something (`../testing/qa/ledger.md`, 4.12). The goal's designer, 2026-11-12.
- **Open:** 13.
