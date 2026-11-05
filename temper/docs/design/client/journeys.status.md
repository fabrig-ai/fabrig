# journeys: status

2026-11-05. Where temper's code lags or departs from [journeys.md](journeys.md).

- **Not yet:** 9: `project-landing` has no Home recording. Its kept
  recording starts at `/welcome`, holds no act and prints Welcome; the
  gallery's `home-*` states draw Home from hand-made deliveries, not
  that walk. The fresh walk in Chromium reaches Home; the journey is
  still to be walked again and recorded by temper's UI owner
  (`journeys/project-landing/README.md`). The kept recording stays as it
  is until that walk is recorded.
  The `project-home-onboarding` render check compares the request's
  `project-landing/01.html` with the gallery's `home-two-left`, and
  its hand-made `project-after-skip/01.html` with `home-after-skip`.
  These states use hand-made deliveries; this render check does not
  observe sign-in on a fresh walk. The `to-begin-more-accounts`
  comparison also uses `home-two-left`, not a Home recording. Neither
  `project-at-work` nor `project-after-skip` is a committed journey;
  their request pages are hand-made and no recording reaches them.
