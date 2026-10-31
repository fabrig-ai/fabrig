+++
id = "stage-browser"
part = "stage-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-io", "shell-surfaces", "echo-stage"]
files = ["skein/testing/skein-fake-person-browser/src/**", "skein/testing/skein-lab-browser/**", "skein/tests/browser/tests/support/real.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 45
+++
# Stage sites in the browser harnesses

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/stage-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.3, for the browser family's seven sites.

## Sites

- `skein/testing/skein-fake-person-browser/src/harness.rs:879`,
  `src/harness/capacity.rs:109`, `src/harness/peers/tests.rs:89`.
- `skein/testing/skein-lab-browser/src/runtime.rs:417`,
  `tests/support/mod.rs:173`.
- `skein/tests/browser/tests/support/real.rs:316` and `:408`.

## Pre-approved defaults

- `capacity.rs` measures the harness's capacity; if the room check
  changes what it pins, leave that site and name it.
- The real-browser support (`tests/browser/tests/support/real.rs`) runs
  only in a slow tier: compile it (`heavy cargo check -p <its package>
  --tests`) and say it was not run.

## Acceptance

- `io_resume` falls by seven, less named sites left by a default.
- `heavy cargo nextest run -p skein-fake-person-browser -p
  skein-lab-browser` and tests/browser's focused tests, with counts.
