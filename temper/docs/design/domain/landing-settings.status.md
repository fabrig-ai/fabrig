# landing-settings: status

2026-11-16. Where the tree's code lags or departs from [landing-settings.md](landing-settings.md).

- **Not yet:** 1.1: kinds 4 and 5 are not declared: each project's way, gates, tiers, merge and close jobs are the document's `application.landing`, read at the start; of kind 4's members, `prepare`, `key`, `home_revision`, `prepare_timeout`, `pins`, `pins_follow_main`, `dependencies_join`, `suspects`, `red_pauses`, `findings` and `questions_on_forge` are not built [in flight: change 5838].
- **Not yet:** 1.2: adoption's checks under `turn` of each gate's workflow and its triggers, of a reporting gate's workflow declaring `temper_tests` (left with the others by the user's word, goal 4045), and of the forge's autodetect and allowance of a manual merge.
- **Not yet:** 1.3: no hold: an engine lands from its start [in flight: change 5838].
- **Departs:** 1.1: `turn` is refused for every repository on GitHub
  (`temper-engine-domain-forge/src/landing.rs`, `project_landing`), not only
  where temper cannot fast-forward main, because GitHub's turn is not built.
