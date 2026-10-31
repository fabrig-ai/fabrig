+++
id = "echo-stage"
part = "echo-first"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-io", "shell-surfaces"]
files = ["skein/examples/echo/service/**", "skein/examples/echo/shell/**", "skein/tests/echo/**", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 25
+++
# The echo example's host on the shared stage

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/stage-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.2: the echo example's host runs on the shared stage, and its
story test is green: the first root end to end on the contract.

## Sites

- `skein/examples/echo/service/src/lib.rs`: `io_up` (~322) and `io_down`
  (~403) with `io_room` (~415) become `skein_io::stage::up` and
  `stage::down`; `src/io_stage.rs` and its `mod io_stage;` (~32) go.
- `skein/examples/echo/shell/src/lib.rs`: the `Echo` host's
  `next_deadline`, if it merges io's deadline with its policy deadline by
  hand, becomes `next_mechanism_deadline` (stage-common.md).

## Acceptance

- `io_resume` falls by one (`--write`).
- `heavy cargo nextest run -p <tests/echo's package>`: the story and
  end-to-end tests pass (name them, with counts), and
  `heavy cargo nextest run -p skein-echo-service -p skein-echo-shell`
  (use the packages' names from their `Cargo.toml`).
