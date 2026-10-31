+++
id = "stage-ui"
part = "stage-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-io", "shell-surfaces", "echo-stage"]
files = ["skein/examples/ui/server/**", "skein/testing/skein-echo-client/**", "skein/tests/ui-web/src/harness_peer.rs", "skein/tests/world/src/bin/terminal_fixture.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++
# Stage sites in the ui example and the echo client, ui-web and world harnesses

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/stage-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.3.

## Sites

- `skein/examples/ui/server/src/main.rs:403`.
- `skein/testing/skein-echo-client/src/lib.rs:366`.
- `skein/tests/ui-web/src/harness_peer.rs:139`.
- `skein/tests/world/src/bin/terminal_fixture.rs:81`.

## Acceptance

- `io_resume` falls by four, less any site left by a default, named.
- The tests of each changed package and of tests/echo (the echo client's
  reader) pass, with counts.
