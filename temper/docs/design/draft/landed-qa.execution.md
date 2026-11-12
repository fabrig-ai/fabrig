# landed-qa: execution

2026-11-11. How [landed-qa.md](landed-qa.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone; the process-tier story of a box at its place is written and compiled in its milestone, and runs in the process tier after it lands.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| Q1, landings and triage: the watcher on main's ref; the deployment keeping builds at landed tips, with the web bundle, its recording build and a manifest; triage one landing at a time in main's order, its cases merged; the ledger's landings, cases and builds, and its commands; a build deleted on no QA or when a newer triaged one is kept; a case waiting past its bound for a build | 2, 3, 4, 9, 11 | temper/migration | nothing | after landing: landings triaged as they come, a build deleted at once on no QA | not started |
| Q2, what the first passes need of temper: drivers from built binaries, a line at a time with `look`, for the browser and the terminal; the recipe's format and walk; a box at its place started with a newer build, with its process-tier story; the fake model's scripts given and its strict switch | 5, 6, 10, 11 | temper/migration | nothing | after landing: the process-tier story runs; a copy of a box set up as `project-landing`'s, restored with a newer build | not started |
| Q3, passes on the web client: places, copies and copies made again; the first recipes, from the journeys' setups; a pass's session, its cases taken in one step, outcomes recorded as it goes, its report and evidence; a recipe that no longer walks followed or filed; findings as one request and one goal; waiting, resuming on a build holding the goal's landing, two rounds; builds deleted on a pass's end and resume | 2, 5, 7, 8, 9, 11 | temper/migration | Q1 and Q2 | after landing: the first passes on real landings, their findings' goals and their resumes | not started |
| Q4, the TUI and deeper states: TUI cases through the terminal driver; fake forges keeping their state across a restart; scripts taking values from the conversation; twins as setups, from B0's and B1's; bounds and places tuned from the ledger | 5, 6, 7, 10 | temper/migration | Q3 | after landing: passes with TUI cases and with a goal's states | not started |

## 2. Order

- **Q1 and Q2 together:** neither needs the other.
- **Q3 as soon as both have landed** (the user, 2026-11-11), with no wait for triage's first verdicts.
- **Q4 after Q3,** whose passes it widens.
- **Section 12's items** are no milestone's.
