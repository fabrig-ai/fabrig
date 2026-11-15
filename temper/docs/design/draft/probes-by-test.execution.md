# probes-by-test: execution

2026-11-15. How [probes-by-test.md](probes-by-test.md) gets carried out. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. The figure is read after landing, and is no plan's.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| T1, probes by test: a gate's `report` and a failed receipt's ids; a search by test, its selection and its verdicts, a search gone whole; probes dispatched with `temper_tests`, a gate green at the top left out; one run per workflow and commit, each run's inputs kept; no turn at a green prefix by test, the round ended unlanded and the next proving it whole; the repair's words, the news, the next round's pull request, `landing show`, the changes view and adoption; the store's patch; `gate.yml` declaring and passing `temper_tests` and uploading the failed ids | 3 to 10 | a goal temper/queue starts | goal 3821's `temper_probe` landed; skein's `forge/forgejo.md`, section 2's dispatch inputs and artifact reads recorded (below) | the probes of the next searches in fabrig's own landings, against section 2's run 976 | landed, goal 4045; its figure after landing not yet read |

## 2. Order

- **After goal 3821,** whose probe input T1 extends.
- **The forge's facts first:** skein's `forge/forgejo.md`, section 2 still lists, before a plan relies on them, a dispatch's inputs reaching the run and a run's artifact listed by name and downloaded. T1 relies on both: a declared input with a value of several lines beside the others, an undeclared one ignored, and the artifact's read. They are recorded on the pinned Forgejo first, by T1's first change where no earlier plan has.
- **The project's gate command runs a selection before its gate gets a report** (`probes-by-test.md`, 9): only the cases named, by package, absent cases passing, the failed ids uploaded, and no tree's record kept from a run by test. That command is the runner host's, changed by whoever runs the project's runners, and is no plan's. It is done when a run given `temper_tests` runs only those; the project then configures the report. Until then temper probes whole, as today, so T1 lands with nothing changed for the project.
