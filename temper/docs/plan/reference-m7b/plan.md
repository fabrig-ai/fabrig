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

## Parts and tasks

- **config-jobs** (`jig/docs/design/protocol/configuration.md, 3`), slice 1.
  - `config-job-kinds`, medium, ~45 min, needs nothing: `{"kind": "job"}` in a delegation's
    kinds.
- **merge-kind** (`temper/docs/design/domain/worker.md, 4.4`), slice 2.
  - `merge-settings`, medium, ~60 min, needs nothing: `merge_job` parsed, kept by the forge.
  - `merge-kind`, high, ~120 min, needs nothing: kind 3 on the worker and in the core; the
    worker's world stories of a merge in progress settled, left, and left marked.
- **change-merge** (`temper/docs/design/domain/forge.md, 8.4`), slice 3.
  - `change-merge-surface`, medium, ~110 min, needs nothing: `Delegate::Merge`, the facts, the
    row's `catch_up` and `merge_result`, the store's records, the job's item; inert arms.
  - `change-merge-steps`, high, ~120 min, needs `change-merge-surface`: the step decides the
    job, its result, the bound, a catch-up's producing; conflicts under rules asking no update
    are resolved, no longer held.
  - `change-merge-route`, high, ~170 min, needs `change-merge-surface`, `change-merge-steps`,
    `merge-settings`: the root makes the job (C2's spec), reads its result, gives a change the
    merge kind, fills the facts; the engine world's stories.
  - `merge-r1-paths`, medium, ~75 min, needs `change-merge-route`, `merge-kind`,
    `merge-reference`: **revision 1** (`notes/contracts.md`, R1): the merge job's item keeps
    its repository's name (an item's name is its remote's too, so C2's `home` fetched the wrong
    repository); the capability resolves a command against the job's directory; the root's
    command is `<name>/<merge_job>`.
- **catch-up** (`temper/docs/design/domain/forge.md, 10`), slice 4.
  - `catch-up-call`, high, ~140 min, needs `change-merge-route`: `change`'s `merge`, its
    refusals, parameter 16; the catch-up stories.
  - `merge-help`, medium, ~30 min, needs `catch-up-call`: the lead's text in
    `temper/docs/help/change.md`.
- **merge-reference** (`temper/docs/design/protocol/configuration.md, 13.9`), slice 4.
  - `merge-reference`, high, ~100 min, needs `merge-settings`: fabrig's `.temper/merge` and
    `tools/merge/`, `landing.json`'s `merge_job`, charter 5's catch-ups.

Slice 4 is **the walk** (`walk = true`): `reference-case.md` steps 12, 16 and 21. Its brief
needs the box's rules and policy 1 to delegate to the merge kind (job 3) beside charter 5, by
hand as `configuration.md, 13.9` says, and the walk's document to carry `landing.json` with its
`merge_job`.

**Revision 2** (`notes/contracts.md`, R2; `notes/common.md`, "Corrections found after
release"): a review of the released specs against the trunk found the merge kind's bounds past
jig's small task limits (now 4,096 and 512 bytes, the command cutting its result), a job task's
spec taking no words or parameters, the engine world's worker declaring no kind, `chat_change`'s
list capacity, kind 3's bound reaching the root, a catch-up's resolution state, wrong names in
`merge-kind`, `merge-settings` and `change-merge-surface`, and `merge-help`'s text past the help
page's bound (replaced by a shorter text). Each correction is in the notes the affected spec
cites, which its implementer reads as it starts; no pushed spec changed.

## Critical path

`change-merge-surface` (~110) → `change-merge-steps` (~120) → `change-merge-route` (~170) →
`catch-up-call` (~140) → `merge-help` (~30): about 570 implementer-minutes, up to about 1140
with the buffer, then fab's end check. `merge-r1-paths` (~75) runs beside `catch-up-call`
after the route; both may touch the root's merge encoder, `merge-r1-paths` only the command's
bytes. `merge-settings` (~60) joins at `change-merge-route`;
`merge-kind` (~120) and `merge-reference` (~100, after `merge-settings`) run beside it and are
done well before.

## Left for later, by the request

- **L's update when M2 has landed** (`reference-landing.md`, 3 step 6): an update the forge
  refuses goes to the merge job before a resolution for every change under rules asking up to
  date (`change-merge-steps`, `change-merge-route`), L's among them once it exists. What is
  M2's is L itself on its requester's held branch, the merge job then writing g under the
  goal's hold; M2 is not on main, so no task here wires L, and its walk story
  (`reference-landing.md`, section 9's first) waits for M2's.
- **The merge job's limits as configuration** (`tooling.md`, section 7): kept the small
  profile's constants.
- **A resolution from the merge job's settled tree** (`tooling.md`, section 7): a resolution
  starts from a fresh merge in progress, its checks recounting a baseline.

## Calls made

- **The merge command lives in the repository it merges,** run from the merge in progress,
  not in the project's home at a pinned revision as `prepare` and `key`: the baselines and the
  scripts that count them are that repository's (`tooling.md, 4.1`).
- **The engine reads no job result:** whether the merge was pushed is the branch's head, read
  afresh, moved or not; the result's bytes are carried into the resolution's words. The
  domain parses nothing (skein's programming model).
- **The root encodes the job's spec** (the worker wire's `JobSpec`), a stated departure
  (`worker.md, 4.4`): jig takes a job's spec as bytes when the root makes it.
- **No new change state or client step:** the job runs within `resolving` (a conflict) or
  `producing` (a catch-up), so the panel, the client wire and the web client are unchanged.
- **One bound:** a merge job counts one resolution, the resolution after it none.
- **Without a `merge_job`,** a conflict goes to a resolution at once, under rules asking no
  update too (where M1's change held), and `change`'s `merge` is refused.
- **The stock is unchanged:** no stock project names a `merge_job`, and the stock's delegation
  gains no job kind (its six executors fill `temper init`'s six); the reference adds the merge
  kind on the box with charter 5.
- **Parameter 16** for a catch-up's commit, clear of 11 to 15, which the wave's sibling plans
  are likeliest to take next.

## Overlaps with other plans

- `reference-m2`, `-m3`, `-m5`, `-m6` (cuts in flight) all reach the change procedure:
  `forge_route.rs` (`change_batch`, `chat_change`, `start_change`), the forge-change crate,
  `store-v11.schema`'s change records, `calls.rs`'s `change` schema, `agent.md` 4.3.1,
  `temper/docs/help/change.md` and the reference's `settings.json`. Unavoidable: the merge job
  is a step of that procedure. This plan's early tasks stay out of them but for
  `change-merge-surface` (the store's change records and the forge-change boundary, appended
  arms and fields only); `change`'s new member takes parameter 16, clear of the numbers the
  siblings are likeliest to take next (11 to 15).
- Within the plan, tasks that may run at once in one file: `merge-settings`,
  `change-merge-surface` and `change-merge-steps` in the forge's `domain.rs` (a setter, forced
  arms, a `Facts` literal: separate hunks); `catch-up-call` and `merge-r1-paths` in the forge's
  `items.rs` and the root's merge encoder (R1's hunk the command's bytes alone).
- `withdrawn-forms` touches `temper-service/src/tests/fixtures.rs` and `tests/one-process`;
  `merge-kind` keeps to `temper-service`'s `limits.rs`, `worker.rs`, `system.rs` and
  `startup/checks.rs`.
