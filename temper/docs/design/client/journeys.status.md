# journeys: status

2026-11-05. Where temper's code lags or departs from [journeys.md](journeys.md).

- **Not yet:** 9: `project-landing` has no Home recording. Its kept
  recording starts at `/welcome`, holds no act and prints Welcome; the
  gallery's `home-*` states draw Home from hand-made deliveries, not
  that walk. The fresh walk in Chromium reaches Ready after the fake
  ChatGPT sign-in, but says "Model none yet" and "Try later", and
  disables "Create your project". Retrying also exposes "OpenAI refused
  temper’s sign-in. Check the configuration’s registration." Forgejo is
  connected and `work/hello` is made. The cause is unknown: the same
  sign-in through a native client leaves a usable model and Ready's act
  enabled (`crates/temper/tests/fake_backends/walk/sign_in.rs`), so
  recording Home waits on the browser's walk passing Ready, then the
  journey must be walked again (`journeys/project-landing/README.md`).
  The `project-home-onboarding` render check compares the request's
  `project-landing/01.html` with the gallery's `home-two-left`, and
  its hand-made `project-after-skip/01.html` with `home-after-skip`.
  These states use hand-made deliveries; whether sign-in still blocks a
  fresh walk is unobserved by this render check. The kept recording
  still prints Welcome until `record-landing` walks it again with a
  usable model and records Home.
