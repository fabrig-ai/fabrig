# The project's checks at each push: the numbers so far

2026-11-03. The evidence behind `../design/protocol/configuration.md`, section 11 ("What the
project's checks run at each push"), gathered for the reference's M10 from fabrig's history:
every repair of a red that reached a plan's shared branch (fab's `<draft>/integration`) in 14
plans, 2026-10-30 to 11-03 (reference-m1 to m9, configuring-p1, first-run-projects,
withdrawn-forms, playground-reset r4 and r6). In those plans each implementer ran `cargo fmt`,
clippy on its changed packages and their focused tests before it pushed.

## Reds that reached the shared branch, by cause

| Cause | Reds | Stopped by the changed packages' lints and focused tests at push |
|---|---|---|
| a test in another package, a dependent, often in another tree | 9 | 0 |
| a merge of main into the shared branch | 7 | 0 |
| clippy (2 from a merge's resolution, 1 in a task still pending) | 4 | 1 |
| a test of a package the red's task changed | 3, and 2 in part | 3, and 2 in part |
| the docs' vocabulary or a citation | 3 | 0 |
| a race or a story's timing | 3 | 0 |
| the old-form counts | 1 | today's `.temper/pre-pr` runs them |
| format | 0 | |
| nothing to change (empty, a false red, owned by a pending task, already fixed) | 5 | |
| unknown | 2 | |

- **Repairs' cost:** most 11 to 25 minutes, from 5 to 54, mostly one commit and one or two
  files; the four a check at push would have stopped took 5, 9, 13 and 18 minutes, each one
  commit and one file.
- **Reds after landing** on main (5) were all in slow tiers, which no check at push runs.

## The check's time at each push

Unknown. `.temper/pre-pr` builds nothing (the format check and the counted baselines). The
workspace's focused suite runs in under 30 s once built (`skein/docs/foundation/testing-strategy.md`,
section 8), but a push's build time in a box's worker, warm or cold, has not been measured.

## What the case's walk measures

- the reds of g's tier per change, by the causes above, with producers that run their own
  lints and focused tests (charter 2 with the shell);
- each repair's tasks and minutes;
- the changed packages' lints and focused tests at a push in the box's worker, warm and cold,
  beside the checks' deadline, the agent's `policy.check_timeout`.
