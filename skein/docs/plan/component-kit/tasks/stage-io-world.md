+++
id = "stage-io-world"
part = "stage-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-io", "shell-surfaces", "echo-stage"]
files = ["skein/tests/io/src/append.rs", "skein/tests/io/src/output.rs", "skein/tests/io/src/processes.rs", "skein/tests/io/src/usage.rs", "skein/tests/io/src/world.rs", "skein/tests/io/tests/suite/memory.rs", "skein/tests/io/tests/suite/process_memory.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 45
+++
# Stage sites in io's own world

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/stage-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.3, for `skein/tests/io`'s eight sites.

## Sites

- `src/append.rs:136`, `src/output.rs:285`, `src/processes.rs:218`,
  `src/usage.rs:151`, `src/world.rs:78`.
- `tests/suite/memory.rs:169`, `tests/suite/process_memory.rs:159` and
  `:389`.

## Pre-approved defaults

- io's world tests io itself: a test that pins one resume step keeps its
  call and is named. `memory.rs` and `process_memory.rs` count
  allocations; if the stage changes a pinned count, keep the site and
  name it with the count.
- Leave the referees in these files to referee-defaults.

## Acceptance

- `io_resume` falls by the sites moved, each kept one named.
- `heavy cargo nextest run -p <tests/io's package>`, default then
  `--profile fuzzy`, with counts.
