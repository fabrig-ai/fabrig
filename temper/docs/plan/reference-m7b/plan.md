# reference-m7b: the plan

M7b of `temper/docs/design/draft/reference-case.execution.md`, section 3 (`inputs/request.md`):
the merge job (N7) as a kind of temper's jobs, and its uses: a merge into a goal's branch
refused for a conflict (`reference-code-production.md`, section 6, step 5), a catch-up change
into the goal's branch (section 11, N6), and an update the forge refused, before a resolution
(`reference-landing.md`, section 3, step 6, and 5); and a delegation naming a job kind in jig's
configuration, which M7a's retire found missing. Written by the cut as it releases; each batch
brings it up to date.

The standing rules are `notes/common.md`; the shapes that cross tasks, `notes/contracts.md`
(C1 to C6).

## Design this cut wrote

- `jig/docs/design/protocol/configuration.md, 3`: a delegation's kinds, `job` among them.
- `temper/docs/design/protocol/tooling.md`: sections 1, 2, 6 and 7, and **4.1, `merge`** (new):
  the command, run from the merge in progress in the repository it merges, its request and
  result, exits 0 and 3.
- `temper/docs/design/domain/worker.md, 4.4`: kind 3, `merge`, delivering; its checkout, its
  bounds; the spec written by the engine's root (a stated departure: the root encodes the
  worker wire's `JobSpec`, since jig takes a job's spec as bytes when the root makes it).
- `temper/docs/design/domain/forge.md`: 8.1 (a catch-up's commit, parameter 16), 8.2 steps 1
  and 6, **8.4 "The merge job"**, section 10 ("A catch-up"), 11.1, 18's stories.
- `temper/docs/design/domain/agent.md, 4.3.1`: `change`'s `merge`; the merge kind in a change's
  authority.
- `temper/docs/design/protocol/configuration.md, 3` (`merge_job`) and 13.9 (the reference's
  `.temper/merge`, charter 5's catch-ups, the merge kind in the rules and policy 1);
  `temper/docs/design/domain/landing.md, 13.1` (kind 4's `merge_job`).
- Status: Not yet entries in each document's status; the drafts' Design lags for the merge job
  (N7), catching up (N6) and the merge job before a resolution closed.

## Parts and tasks so far

- **config-jobs** (`jig/docs/design/protocol/configuration.md, 3`), slice 1.
  - `config-job-kinds`, medium, ~45 min, needs nothing.
- **merge-kind** (`temper/docs/design/domain/worker.md, 4.4`), slice 2.
  - `merge-settings`, medium, ~60 min, needs nothing: `merge_job` parsed and kept by the forge.
  - `merge-kind`, high, ~120 min, needs nothing: kind 3 on the worker and in the core; the
    worker's world stories.
- **change-merge** (`temper/docs/design/domain/forge.md, 8.4`), slice 3.
  - `change-merge-surface`, medium, ~110 min, needs nothing: `Delegate::Merge`, the facts, the
    row's fields, the store's records, the job's item; inert arms.

## Critical path, as it stands

`change-merge-surface` (~110), then the change steps and the root's route (next batch).

## Overlaps with other plans

- `reference-m2`, `-m3`, `-m5`, `-m6` (cuts in flight) all reach the change procedure:
  `forge_route.rs` (`change_batch`, `chat_change`, `start_change`), the forge-change crate,
  `store-v11.schema`'s change records, `calls.rs`'s `change` schema, `agent.md` 4.3.1,
  `temper/docs/help/change.md` and the reference's `settings.json`. Unavoidable: the merge job
  is a step of that procedure. This plan's early tasks stay out of them but for
  `change-merge-surface` (the store's change records and the forge-change boundary, appended
  arms and fields only); `change`'s new member takes parameter 16, clear of the numbers the
  siblings are likeliest to take next (11 to 15).
- `withdrawn-forms` touches `temper-service/src/tests/fixtures.rs` and `tests/one-process`;
  `merge-kind` keeps to `temper-service`'s `limits.rs`, `worker.rs`, `system.rs` and
  `startup/checks.rs`.
