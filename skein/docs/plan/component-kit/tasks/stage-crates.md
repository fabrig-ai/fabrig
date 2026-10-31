+++
id = "stage-crates"
part = "stage-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-io", "shell-surfaces", "echo-stage"]
files = ["skein/crates/skein-fake-peers/src/face.rs", "skein/crates/skein-sim/src/tests/given_listener.rs", "skein/crates/skein-ui-text/src/terminal.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++
# Stage sites in skein's crates: fake-peers, sim, ui-text

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/stage-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.3, for the sites inside `skein/crates/`.

## Sites

- `skein/crates/skein-fake-peers/src/face.rs:71`: `Face::up` (~64), which
  gates on `submissions.room() < 2` by hand.
- `skein/crates/skein-sim/src/tests/given_listener.rs:50`.
- `skein/crates/skein-ui-text/src/terminal.rs:254`.

## Acceptance

- `io_resume` falls by three, less any site left by a default, named.
- `heavy cargo nextest run -p skein-fake-peers -p skein-sim -p
  skein-ui-text`, and the tests/fake-peers and tests/ui-text worlds.
