# journeys: status

2026-11-05. Where temper's code lags or departs from [journeys.md](journeys.md).

- **Not yet:** 9: `project-landing` has no Home recording: its kept
  recording starts at `/welcome`, holds no act and prints Welcome. The
  fresh walk in Chromium reaches Home, on a driver that is not kept; the
  journey is still to be walked again and recorded by temper's UI owner
  (`journeys/project-landing/README.md`). Until then no test walks the
  ChatGPT sign-in in a browser: the process tier holds it with a native
  client beside a browser's idle sockets on the issuer
  (`crates/temper/tests/fake_backends/walk/sign_in.rs`), and no render check
  compares a recorded Home: `project-home-onboarding` and
  `to-begin-more-accounts` are compared with the gallery's
  `home-two-left` and `home-after-skip`, which draw Home from hand-made
  deliveries and do not observe sign-in on a fresh walk.
