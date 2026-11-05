# journeys: status

2026-11-05. Where temper's code lags or departs from [journeys.md](journeys.md).

- **Not yet:** 9: `project-landing` has no Home recording: its kept
  recording starts at `/welcome`, holds no act and prints Welcome. Its
  walk in Chromium last stopped at Ready, after the fake ChatGPT
  sign-in, with "Model none yet", "Try later" and "Create your project"
  disabled; a retry says "OpenAI refused temper’s sign-in. Check the
  configuration’s registration.". Forgejo is connected and `work/hello`
  is made. The cause is unknown: the same sign-in through a native
  client leaves a usable model and Ready's act enabled
  (`crates/temper/tests/fake_backends/walk/sign_in.rs`). The Chromium
  fault and the recording after it are temper's UI owner's
  (`journeys/project-landing/README.md`). Until then no render check
  compares a recorded Home: `project-home-onboarding` and
  `to-begin-more-accounts` are compared with the gallery's
  `home-two-left` and `home-after-skip`, which draw Home from hand-made
  deliveries.
