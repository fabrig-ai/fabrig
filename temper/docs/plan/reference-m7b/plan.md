# reference-m7b: the plan

M7b of `temper/docs/design/draft/reference-case.execution.md`, section 3 (`inputs/request.md`):
the merge job (N7) as a kind of temper's jobs, and its uses: a merge into a goal's branch
refused for a conflict (`reference-code-production.md`, section 6, step 5), a catch-up change
into the goal's branch (section 11, N6), and an update the forge refused, before a resolution
(`reference-landing.md`, section 3, step 6, and 5); and a delegation naming a job kind in jig's
configuration, which M7a's retire found missing. Written by the cut as it releases; each batch
brings it up to date.

The standing rules are `notes/common.md`; the shapes that cross tasks, `notes/contracts.md`.

## Parts and tasks so far

- **config-jobs** (`jig/docs/design/protocol/configuration.md, 3`), slice 1.
  - `tasks/config-job-kinds.md`, medium, ~45 min, needs nothing: `{"kind": "job"}` in a
    delegation's kinds.

## Critical path, as it stands

`config-job-kinds` (~45) alone, until the merge job's family is released.
