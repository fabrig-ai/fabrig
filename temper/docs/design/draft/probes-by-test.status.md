# probes-by-test: status

2026-11-15. Where the design lags or departs from [probes-by-test.md](probes-by-test.md).

- **Design lags:**
  - 3: `../domain/landing.md`, 13.1's gates have no `report`, and 6.2's failed receipt keeps no ids; `../domain/after-landing.md`, 4's reading selects errored cases too, which a selection leaves out.
  - 4: `../domain/landing-search.md`, 3 dispatches every blocking gate whole at each probe and takes a verdict from any failed receipt, and 6 makes a receipt of every late outcome; `../domain/landing.md`, 6.2 and 7.2 know only whole runs, and 7.1 gives a gate no `temper_tests`, takes two runs of one key at one commit as one, and keeps no run's inputs, nor does section 15 at a restart.
  - 5: `../domain/landing-search.md`, 4 and 5 turn at every green prefix but the base, and `../domain/landing.md`, 4.3 and 5.1 keep a round going after the unit its search blames leaves.
  - 6 and 7: `../domain/landing-search.md`, 7 and 8 have no words for a search by test, and `../domain/landing.md`, 10 no line from a search on the next round's pull request.
  - 8: `../domain/landing.md`, 12 and 13.2, and `../domain/landing-search.md`, 9, keep no selection or run inputs and check no `temper_tests`.
- **Design departs:** none.
- **Open:** none.
