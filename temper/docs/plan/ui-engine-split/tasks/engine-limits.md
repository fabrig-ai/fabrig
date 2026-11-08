+++
id = "engine-limits"
part = "service-split"
kind = "migration"
needs = ["brief-crate"]
files = ["temper/crates/temper-service/src/**", "temper/crates/temper-service/Cargo.toml", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 120
+++

## Goal

`temper-service`'s engine half stops naming the worker's crates, so that
the next task can make the worker's half an optional feature with the
engine's code untouched (the request's first item: "the engine's limits
read no worker domain or checkout crate";
`temper/docs/design/protocol/README.md`, section 5's
`crates/temper-service` row). Behaviour doesn't change: every test passes
as it is, and a document parses, checks and starts as before.

Acceptance:

- **The rule** (`notes/contracts.md`, 5, "After `engine-limits`"): no
  module of `temper-service` outside `worker.rs`, `worker/`,
  `worker_route.rs`, `worker_git.rs`, `worker_settings.rs`,
  `identities.rs`, `program_sources.rs`, `startup/worker.rs` (new) and
  `tests/` names `temper_worker_domain`, `temper_worker_domain_checkout`,
  `temper_worker_domain_tooling`, `temper_worker_protocol_git`,
  `jig_protocol_worker_agents`, `jig_protocol_worker_link`,
  `smith_host_domain`, `smith_host_protocol`, `smith_protocol_program`,
  `smith_protocol_channel`, `smith_channel` or `skein_channel`. `rg` over
  the other modules finds none; the report shows the command and its
  empty output.
- **The limits mirror the worker's.** `limits::Limits`'s `checkout` and
  `git_component` fields take structs of `limits.rs`'s own,
  `limits::Checkout` and `limits::GitComponent`: the same fields, names
  and values as `temper_worker_domain_checkout::Limits` and
  `temper_worker_protocol_git::Limits` (durations as
  `skein_lib::Duration` or whatever the originals hold, if it is
  skein's), with their override names and defaults unchanged
  (`limits/names.rs`), so a document's `limits` parse the same. The
  worker's half converts them (`impl From<limits::Checkout> for
  checkout::Limits` and the git component's, in `worker_settings.rs` or a
  worker module), field by field, and the engine's uses (`startup.rs`'s
  wire sizing, `translate.rs`, `startup/checks.rs`) read the mirrors.
- **What only the worker needs moves to its half:** `limits::tooling` (the
  reference's use, `startup/reference.rs`, reads a `TOOLING_PATH` const
  of `limits.rs`'s in its place); the git component's rules and the
  worker part's checks of `startup/checks.rs` that read worker crates
  (`worker_payload`, `worker`, the git component's rules,
  `PushDiagnostic::CAPACITY`) into `startup/worker.rs`, which
  `startup/checks.rs` calls; `translate.rs`'s worker-only functions
  (`Assigned`, `workspace`, `work`, `landing`, `push_reason`,
  `preparation`, `value` and what only they use), which only
  `worker_route.rs` calls, into a worker module; `settings::Agent`'s
  definition into `worker_settings.rs`, re-exported as `settings::Agent`.
  `startup.rs`'s worker building (`NamedWorker`'s reads of worker crates,
  `worker_part`, `worker_alone`, `WorkerPart`, and the worker's
  `settings::Worker` and `worker_settings` calls) moves into
  `startup/worker.rs` where it names a worker crate; `startup.rs` keeps
  calling it. `system.rs`'s `Service::worker_domain`, whose type is
  `temper_worker_domain::Domain`, moves into an `impl Service` block of
  `worker.rs`, its callers unchanged.
- **Shared constants become the engine's own.** `values.rs`' and
  `workers.rs`' reads of `worker_settings::credential_bytes` and
  `grant_bytes` take engine-side functions in `values.rs`, over an
  engine-side `SOURCE_NAME_BYTES` (32); the worker's half holds a
  compile-time check that it equals `smith_host_domain::SourceName::BYTES`
  (`const _: () = assert!(…)`), and likewise for any other constant a
  mirror copies. The engine's uses of `smith_protocol_channel`
  (`assignment.rs`, `tools.rs`, `workers.rs`: `encode_charter`,
  `decode_declared`, `Endpoints`) take them from
  `smith_protocol_translate`, which `Cargo.toml` adds.
- **`startup::nearest`** is public, `notes/contracts.md`, 5's signature:
  it parses the bytes as the configuration's document (its existing
  limits) and returns the nearest shape, `None` for bytes that aren't a
  document whose root is an object. The table test
  `each_document_shape_is_read_from_its_members`
  (`src/tests/checks.rs`) gains, for each of its three documents, a row
  for `nearest`, and rows for a worker alone's document with a broken
  `application` (still `Some(WorkerAlone)`), for `b"[]"` and for
  `b"not json"` (`None`). Break `nearest` once (always standalone), see
  its rows fail, restore.
- Every test of `temper-service` passes unchanged but for paths to the
  moved items; `temper` and the worlds build.
- The status entry closes: in `temper/docs/design/protocol/README.status.md`,
  the **Not yet** 5 entry "the engine's half of `temper-service` names
  the worker's crates: …", its lines alone. This departs from the
  implementer's text, which closes `docs/design/` to you: the entry is
  this task's.

## Inputs

- `temper/docs/design/protocol/README.md`, section 5, the
  `crates/temper-service` row; 15.1 (the shapes and "the nearest").
- `temper/docs/design/protocol/configuration.md`, section 7 (the limits),
  for what the override names mean.
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 5.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`: every module of `temper-service` and its
  tests, for this.
- Readers outside the crate: a file whose path to a moved `pub` item
  breaks (`temper/src`, `temper-playground`, `temper/tests/one-process`,
  `temper/tests/system`, `temper/tests/real-loop`): the path alone, named
  in the report. Where a moved item was `pub` at the crate's root or in
  a public module, keep it reachable there by a re-export, so that such
  readers stay unchanged.
- Pre-approved:
  - If a worker crate's type reaches an engine module only as an opaque
    value the engine hands on (a `JobBound`, a diagnostic), give the
    engine its own type and convert in the worker's half, as for the
    limits; if that would need a change outside `temper-service`, stop
    there, keep that one use, and name it in the report as left for
    `worker-feature`.
  - If `limits::Limits`' field types are read by a test of another crate
    (none known), change that read and name it.
- Overlaps: the plan `goal-call-faults` is changing `tools.rs` and
  `src/tests/tools.rs`. Your change in `tools.rs` is the one import path
  (`smith_protocol_channel` → `smith_protocol_translate`).
- Not yours: any `Cargo.toml` feature (task `worker-feature`); the worker
  crates themselves; `temper/src`'s code beyond the paths above.

## Steps

1. `nearest`'s rows first; see them fail; then `nearest`.
2. The mirrors and their conversions; then the moves, one group at a
   time, `heavy cargo check -p temper-service --tests` while errors last.
3. `heavy cargo nextest run -p temper-service --lib -E
   'test(/checks|limits|startup|worker|translate|room|stock_brief/)'`;
   read the counts.
4. The rule's `rg`, its output in the report.
5. The status entry.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-service --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-service`; `heavy cargo check -p temper -p
   temper-playground -p temper-one-process-world -p temper-system-world
   -p temper-real-loop-world -p temper-engine-domain-world --tests`.
7. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with the rule's `rg` and its output,
each opaque type given the engine, and anything left.
