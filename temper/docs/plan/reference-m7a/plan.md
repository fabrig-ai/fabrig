# reference-m7a: the plan

M7a of `temper/docs/design/draft/reference-case.execution.md`, section 3 (`inputs/request.md`):
jobs in jig (`jig/docs/design/domain/core.md, 3.2`; `domain/tasks.md, 5.3.1`;
`domain/engine.md, 7.5`; `domain/hosts.md, 6.9`; `client/domain/contract.md, 5.7`) and in
temper's worker (`temper/docs/design/domain/worker.md, 4.4`), as designed and not built. The
merge job, its uses, and the `prepare` and `key` kinds are M7b's.

The shapes every task builds to are `notes/contracts.md`; the standing rules
`notes/common.md`.

## Design this cut wrote

- `jig/docs/design/protocol/channel.md, 5.4` (new) and the job rows of 4.3, 5.1 and 5.2: a
  job's records on the worker channel. Status: Not yet.
- `jig/docs/design/protocol/store.md, 5`: what the families store of a job task and a
  delegation's job kinds. Status: Not yet.
- `jig/docs/design/domain/engine.md, 7.5`: the application declares its kinds with their
  bounds; a batch past them is refused. `domain/hosts.md`, section 4: the job capability beside
  the agent and the workspace.
- `temper/docs/design/domain/worker.md, 4.4`: kinds as entries (number, verb, delivers); one
  spec (command, request) and one result (the result file's bytes) for every kind; a delivering
  kind's push; section 12: the merge job's entry left to M7b.

## Parts and tasks

- **worker-channel** (`jig/docs/design/domain/hosts.md, 6.9`), slice 1.
  - `tasks/worker-wire-jobs.md`, medium, ~60 min, needs nothing: the worker wire's schema in
    place: the hello's `jobs`, `Job`, `JobAnswer`, their kinds.
  - `tasks/hub-jobs.md`, high, ~160 min, needs nothing: `JobKind`/`JobBound`/`Hello::jobs`
    and the hub's hosted job, with forced arms elsewhere.
  - `tasks/link-jobs.md`, high, ~120 min, needs `worker-wire-jobs`, `hub-jobs`: both links
    translate the job's records.
  - `tasks/fleet-jobs.md`, high, ~170 min, needs `hub-jobs`: the fleet places a job on a host
    declaring its kind; hub-link's `StartJob`/`AssignedJob`; a job's answer as the tasks
    child's `End`; restart.
- **worker-jobs** (`temper/docs/design/domain/worker.md, 4.4`), slice 1.
  - `tasks/tooling-capability.md`, high, ~150 min, needs nothing: the new crate
    `temper-worker-domain-tooling`, sans-io, uncomposed.
  - `tasks/worker-jobs.md`, high, ~170 min, needs `hub-jobs`, `tooling-capability`: the root
    composes it; hello's kinds; routing; delivery by the checkout's push.
  - `tasks/service-jobs.md`, high, ~160 min, needs `worker-jobs`, `link-jobs`: the spec's
    record and decoding, the operations on skein's io, the profile's job limits.
- **client-jobs** (`jig/docs/design/client/domain/contract.md, 5.7`), slice 2.
  - `tasks/client-jobs.md`, medium, ~120 min, needs nothing: the client wire, domain, link,
    view, temper's client arms; Create refuses a job.
- **job-tasks** (`jig/docs/design/domain/tasks.md, 5.3.1`), slice 3.
  - `tasks/tasks-jobs-surface.md`, medium, ~180 min, needs `client-jobs`: the variants in tasks,
    authority, the store's three families, the core's and temper's forced arms, inert (the
    batch refuses a job); the engine-clients mappings.
  - Next batch (a designer's): `tasks-jobs-lifecycle` and `core-jobs-admit` side by side after
    the surface, then `core-jobs-run` (needs both and `fleet-jobs`).

## Critical path, as it stands

`client-jobs` (~120) → `tasks-jobs-surface` (~180) → `core-jobs-admit` / `tasks-jobs-lifecycle`
→ `core-jobs-run`, then fab's end check: about nine hours of implementer time, eighteen with
the buffer. The worker side (`hub-jobs` → `link-jobs`, `fleet-jobs`, `worker-jobs` →
`service-jobs`) runs beside it and joins at `core-jobs-run` through `fleet-jobs`.

## Overlaps

- **Within the plan, crates shared by tasks that may run at once,** in distinct files or hunks:
  `fleet-jobs` adds forced arms in `jig-domain-core`'s routing and in the engine roots'
  `link.rs` while `tasks-jobs-surface` adds executor arms in other core files;
  `worker-jobs` adds inert arms in `temper-service`'s worker files, which `service-jobs`
  (after it) owns.
- Within the plan, `worker-wire-jobs` and `hub-jobs` both touch
  `jig/crates/jig-protocol-worker-link/src/translate.rs` (the hello's wire literal; the job
  answer's arm in `up`), distinct functions; `hub-jobs`'s forced `Hello::jobs` literals touch
  `jig-domain-core` and `jig/tests/core` fixtures, which `tasks-jobs-surface` later reaches for
  executor arms: different hunks, and the surface starts after `client-jobs`, likely after them.
- `reference-m8` (cut in flight) builds temper's tiers and skein's reads; its files are not
  known to this cut. Its likeliest meeting point is `temper-client-tools` and
  `temper-client-view`, where `client-jobs` adds arms only.
- `live-smith-reds-2` changes smith alone; no task here touches smith.

## Calls made

- **No version moves.** The store, the worker wire, the client wire and temper's payload codec
  change in place, by `jig/docs/design/protocol/README.md, 7` and `protocol/store.md, 5`: no
  stored format or protocol question for the user.
- **A job's own refusal and failure types** (`JobRefusal`, `JobInvalid`, `JobFailure`) beside
  a run's, so no run type, record or arm changes.
- **temper's job capability takes kinds as data** (number, verb, delivers), so M7b adds the
  merge job as one more entry; production declares none until then. Its spec is decoded by the
  service (the domain links no protocol crate); its result is the result file's bytes.
- **`Rewrite`** stays with `prepare` (M7b or later): only a prepare job's candidate uses it.
