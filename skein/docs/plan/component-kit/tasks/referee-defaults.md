+++
id = "referee-defaults"
part = "world-surfaces"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "shell-surfaces", "stage-io-world"]
files = ["skein/testing/skein-world/src/real.rs", "skein/tests/world/src/hosted.rs", "skein/tests/world/src/lib.rs", "skein/tests/world/tests/suite/hosted.rs", "skein/tests/world/tests/suite/real.rs", "skein/tests/world/tests/suite/world.rs", "skein/tests/world/tests/suite/stop_continue.rs", "skein/tests/world/tests/suite/interrupt.rs", "skein/tests/world/tests/suite/signals.rs", "skein/tests/io/src/pages.rs", "skein/tests/io/src/files.rs", "skein/tests/io/src/append.rs", "skein/tests/io/src/usage.rs", "skein/tests/io/src/private.rs", "skein/tests/io/src/processes.rs", "skein/tests/io/src/renames.rs", "skein/tests/io/src/cuts.rs", "skein/tests/io/src/locks.rs"]
effort = "medium"
minutes = 25
+++
# Referees take the defaults

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.5 (`Referee`). With shell-surfaces' defaults landed, each
referee's `act` that does nothing, `next_deadline` that returns `None`
and `overdue` that returns `None` goes.

## Sites (the survey's; each method only where it is the default)

- `skein/testing/skein-world/src/real.rs:1302` (Quiet: act, overdue),
  `:1396` (Observations: next_deadline, overdue).
- `skein/tests/world/src/hosted.rs:339`; `src/lib.rs:375` (act), `:425`.
- `skein/tests/world/tests/suite/`: `hosted.rs:252`, `real.rs:78`,
  `:122`, `:281`, `world.rs:232`, `stop_continue.rs:14`,
  `interrupt.rs:16`, `signals.rs:18`.
- `skein/tests/io/src/`: `pages.rs:174`, `files.rs:355`, `append.rs:278`,
  `usage.rs:213`, `private.rs:317`, `processes.rs:290`, `renames.rs:253`,
  `cuts.rs:306`, `locks.rs:251`.

The referees in `tests/world/tests/suite/{end_to_end,real_conformance,
writer_pipe}.rs` are forwarding-sites'.

## Acceptance

- `heavy cargo nextest run -p <tests/world's package> -p <tests/io's
  package> -p skein-world`, with counts unchanged from before.
- Report the lines removed.
