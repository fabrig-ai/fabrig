# landing-settings: status

2026-11-18. Where the tree's code lags or departs from [landing-settings.md](landing-settings.md).

- **Not yet:** 1.1: kind 4 has no `wait` [in flight: change 8587].
- **Not yet:** 1.1: kind 4 has no `flakes` [in flight: change 18911].
- **Not yet:** 1.2: adoption's checks under `turn` of each gate's workflow and its triggers, of a reporting gate's workflow declaring `temper_tests` (left with the others by the user's word, goal 4045), and of the forge's autodetect and allowance of a manual merge.
- **Departs:** 1.1: `turn` is refused for every repository on GitHub
  (`temper-engine-domain-forge/src/landing.rs`, `project_landing`), not only
  where temper cannot fast-forward main, because GitHub's turn is not built.
