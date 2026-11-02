+++
id = "worker-wire-jobs"
part = "worker-channel"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-protocol-worker-wire/**",
  "jig/tests/worker-wire/**",
]
effort = "medium"
minutes = 60
+++

# The worker wire carries jobs: the hello's kinds, `Job` and `JobAnswer`

## Goal

Add jobs' records to jig's worker channel schema, version 9 in place
(`jig/docs/design/protocol/channel.md, 5.4`): the hello's `jobs`, a `Job` record down and a
`JobAnswer` record up, each with its kind number. The translations of these records to the
domain are a later task's (`link-jobs`); this task carries only the schema, its generated
code, goldens and kinds, and the one literal field the hello's encoder needs.

Acceptance:

- `schema/worker-wire-v9.schema` holds `notes/contracts.md`, section 3, exactly: the hello's
  `jobs` list of `JobBound { kind: u32, spec: u32, result: u32, deadline: duration }`, and the
  records `Job` and `JobAnswer` with the enums `JobEnding`, `JobRefusal`, `JobInvalid`,
  `JobFailure` and the record wrappers the schema's idiom uses (`...Value` records around enum
  arms, as `RefusalValue` and `PreparationValue` are).
- `src/kinds.rs`'s `Kind` gains `Job` (engine to worker) and `JobAnswer` (worker to engine),
  each with a kind number after the last one used, a direction, and a body bound computed from
  the limits as the assignment's and the answer's are.
- `src/generated/v9.rs` and the goldens are regenerated, never hand-edited; the worker wire's
  drift test passes without `SKEIN_CODEGEN_WRITE`; `VERSION` stays 9.
- `jig-protocol-worker-link`'s `translate::hello` builds the wire hello with `jobs` empty; no
  other line of it changes. Every other constructor of a wire `Hello` the compiler names gets
  the same empty `jobs`.
- `jig-worker-wire-world`'s tests and `jig-protocol-worker-wire`'s own pass; clippy is clean
  for `jig-protocol-worker-wire`, `jig-protocol-worker-link`, `jig-protocol-engine-workers`.

## Inputs

- `temper/docs/plan/reference-m7a/notes/common.md` and `notes/contracts.md`, sections 1 to 3.
- `jig/docs/design/protocol/channel.md, 4.3`, `5.1` (the hello and `job answer` rows), `5.2`
  (the `job` row) and `5.4` (jobs).
- The record shapes to mirror: `record Assignment` (`task`, `attempt`, `workspace`, `grants`)
  and `record Answer`/`enum Ending`/`enum Failure` in the same schema, `record Work`,
  `enum Preparation`, `enum DeliveryOutcome`, `enum CancelReason`, `enum InvalidStart`.

## Owns and may change

Owns: `jig/crates/jig-protocol-worker-wire/**` (schema, generated code, `kinds.rs`, goldens,
its unit tests) and `jig/tests/worker-wire/**`.

May change, for forced literals only: `jig/crates/jig-protocol-worker-link/src/translate.rs`
(the `hello` function's wire `Hello` gains `jobs` empty) and any test fixture in
`jig/tests/worker-link/**`, `jig/tests/engine-workers/**` or the two link crates that builds a
wire `Hello` record (`jobs` empty, nothing else).

Pre-approved options:

- **The list's bound:** if the schema needs a numeric bound on `jobs`, use 256, beside
  `models`' idiom; if the generated `Limits` gains a field for it, its value comes from the
  same place the models' bound's does.
- **A duration in a record:** use the schema's `duration` as `Hello::stop_bound` does.
- **`JobInvalid::start`:** reuse `InvalidStart` if its fields are `why` bytes and a cause;
  if `InvalidStart` carries a cause a job has none of, give `start` its own record with `why`
  alone, and say so in the report.
- If a fuzz target of this family (`jig/tests/worker-wire/tests/suite/fuzzy`) enumerates
  record kinds exhaustively, add the two new kinds there.
- If the kinds table is also checked by `jig-worker-link-world` or `jig-engine-workers-world`
  (a test listing every kind), add the two kinds there and list the file in the report.

## Steps

1. Schema: the hello's `jobs`; then, after the answer's records, the job's records and enums,
   each new enum arm appended last where an enum is extended (none is: every job enum is new).
2. `kinds.rs`: the two kinds and their rules.
3. Regenerate (`notes/common.md`): `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
   jig-worker-wire-world -E 'test(drift::)'`, then again without it; `git diff --stat` shows
   the schema, `v9.rs`, `kinds.rs`, new goldens, and the forced literals alone.
4. Unit tests in `jig-protocol-worker-wire` beside the existing ones: a `Job` and a
   `JobAnswer` (finished, failed `undelivered: stale`, refused `invalid: kind`) encode and
   decode back equal; a hello with two job kinds round-trips; a `Job` whose spec is one byte
   past its bound is refused by the generated constructor.
5. `cargo fmt`; `heavy cargo clippy -p jig-protocol-worker-wire -p jig-protocol-worker-link -p
   jig-protocol-engine-workers --all-targets -- -D warnings`; `heavy cargo nextest run -p
   jig-protocol-worker-wire -p jig-worker-wire-world -p jig-protocol-worker-link -p
   jig-protocol-engine-workers`.
6. One commit; push your branch.

## Report

As the implementer's text says, and also: the new kinds' numbers, the bound chosen for `jobs`,
and the record names the schema's idiom gave each job enum's wrapper (the next task,
`link-jobs`, translates them).
