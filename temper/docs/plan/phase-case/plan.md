# phase-case: the plan

From `inputs/request.md` (QA finding 42): `--until phase:Running` is refused while
`phase:running` is met. One direct part, one task, as the request asks.

## Parts and tasks

- **phase-case** (`temper/docs/design/client/tools.md, 5.4`), slice 1.
  - `tasks/phase-any-case.md`, medium, ~35 minutes, needs nothing: `subject::phase` matches
    its words ignoring ASCII case (`backing-off` also `BackingOff`, `backing_off`); a
    regression test beside the phase parse tests; the `task show` `--until` help line, the
    `ctl` page's bullet, the reference page and the two help goldens, with the help lines
    given verbatim in the spec.

## Critical path

`phase-any-case`, then fab's end check on the trunk.

## Overlaps

None with `v0.1.0-ladder`'s open branches (`finding-1`, `-2`, `-5`) at the cut; its later
findings may touch `rows.rs`, `ctl.md` or the help goldens, which this task changes in one
line each.

## Calls made

- The help lines are the lead's (`temper/AGENTS.md`), given verbatim in the spec; the task
  writes them with the code so the trunk never says what the code does not do (a stated
  departure).
- `task show` prints phrase-book words, several of which are no phase word (`Retrying`,
  `Due to run`, `Waiting for news`, ...): out of scope, reported as left.
