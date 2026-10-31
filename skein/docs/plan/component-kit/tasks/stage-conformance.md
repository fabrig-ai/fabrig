+++
id = "stage-conformance"
part = "stage-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-io", "shell-surfaces", "echo-stage"]
files = ["skein/tests/conformance/real/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++
# Stage sites in the real-kernel conformance tests

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/stage-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.3, for `skein/tests/conformance/real`'s six sites.

## Sites

- `src/bin/signal_sender_fixture.rs:168`.
- `tests/suite/adopted_listener.rs:70` and `:174`.
- `tests/suite/given_listener.rs:51`.
- `tests/support/deadline.rs:101`.
- `tests/support/given_listener.rs:47`.

## Pre-approved defaults

- These run io against the real kernel. A test that drives one resume to
  observe one step of io itself keeps its call (stage-common.md) and is
  named; a loop that is a whole stage moves.
- If a conformance test runs only in a slow tier, compile it and say so.

## Acceptance

- `io_resume` falls by the sites moved; each kept one named with its test.
- `heavy cargo nextest run -p <its package>` (default profile), counts.
