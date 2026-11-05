# journeys: status

2026-11-05. Where temper's code lags or departs from [journeys.md](journeys.md).

- **Not yet:** 9: `project-landing` has no Home recording: its kept
  recording prints Welcome. `journeys/project-landing/README.md`
  describes the walk blocked at Ready after the fake ChatGPT sign-in,
  with "Model none yet", "Try later" and "Create your project"
  disabled, though Forgejo is connected and `work/hello` is made.
  The `project-home-onboarding` render check compares the request's
  `project-landing/01.html` with the gallery's `home-two-left`, and
  its hand-made `project-after-skip/01.html` with `home-after-skip`.
  These gallery states use hand-made deliveries; the kept recording
  still prints Welcome until `record-landing` walks it again with a
  usable model and records Home. Whether sign-in still blocks a fresh
  walk is unobserved by this render check.
