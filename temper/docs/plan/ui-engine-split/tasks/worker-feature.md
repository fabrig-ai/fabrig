+++
id = "worker-feature"
part = "service-split"
kind = "feature"
needs = ["engine-limits"]
files = ["temper/crates/temper-service/**", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 120
+++

## Goal

`temper-service`'s worker half becomes its `worker` feature, so that an
engine alone's service composes without the worker's crates (the
request's first item; `temper/docs/design/protocol/README.md`, 4.1, "A
binary's features are its package's", and section 5's
`crates/temper-service` row). The box's binary and every world keep the
worker; `temper-engine` (task `engine-binary`) leaves it off. Behaviour
with the feature on is unchanged.

Acceptance:

- **The feature** (`notes/contracts.md`, 5, "After `worker-feature`"):
  `[features] default = ["worker"]`, `worker = [...]` listing as
  `dep:` every dependency only the worker's half names (after
  `engine-limits`: `temper-worker-domain`,
  `temper-worker-domain-checkout`, `temper-worker-protocol-git`,
  `jig-protocol-worker-link`, `jig-protocol-worker-agents`,
  `smith-protocol-program`, `smith-host-protocol`, `smith-host-domain`,
  `smith-channel`, `skein-channel`, `smith-protocol-channel` if it is
  still named, and any other whose every use is in the worker's
  modules), each `optional = true`. The root's
  `[workspace.dependencies]` entry becomes `temper-service = { path =
  "temper/crates/temper-service", default-features = false }`.
- **Without it, the same names compile.** The worker's modules
  (`worker.rs`, `worker/`, `worker_route.rs`, `worker_git.rs`,
  `worker_settings.rs`, `identities.rs`, `program_sources.rs`,
  `startup/worker.rs`) are `#[cfg(feature = "worker")]`; a stand-in
  module under `#[cfg(not(feature = "worker"))]` gives `Worker`,
  `WorkerRuntime` and `settings::Agent` as uninhabited types (`enum X
  {}`) with the methods the rest of the crate calls, each `match *self
  {}`, so that `system.rs`, `startup.rs` and the engine's modules keep
  their code, with a `cfg` only where a call has no stand-in.
  `startup::with_setup_given` and the other startup functions keep their
  signatures; `Service::reuse_workspaces` and `dump_llm` exist both
  ways. Only with the feature: `startup::{worker_part, worker_alone,
  WorkerPart}`, `program_sources`, `Service::configure_program_sources`
  and `Service::worker_domain`.
- **The backstop.** Startup refuses, before any other check, a document
  whose `startup::nearest` shape isn't an engine alone when the worker
  isn't built, with a new `What` variant whose line says `this build
  serves an engine alone`, at the document's `workers` path. It is one
  function, `startup::unbuilt(shape: Shape, worker: bool) -> Option<Problem>`
  (or a name of that kind), called with `cfg!(feature = "worker")`, so
  that a table test with the feature on holds it: rows `(Standalone,
  false)`, `(WorkerAlone, false)` refused; `(EngineAlone, false)` and
  every shape with `true` passing. Break it once (pass every shape), see
  the refused rows fail, restore.
- **The dependents keep the worker:** every manifest naming
  `temper-service` by the workspace's entry, normal or dev, says
  `temper-service = { workspace = true, features = ["worker"] }`:
  `temper/crates/temper`, `temper-playground`, `temper-fake` (dev),
  `temper/testing/temper-github-app`, and the worlds under
  `temper/tests/` (`engine/domain`, `playground`, `system`, `real-loop`,
  `client`, `client-browser`, `one-process`); `tests/engine/protocol-forge`'s
  path dependency keeps its default. If `temper-engine-shell` exists on
  your base, its manifest stays as it is, with no feature.
- **The closure:** `cargo tree -p temper-service --no-default-features -e
  normal --locked --offline` holds none of `temper-worker-*`,
  `jig-protocol-worker-agents`, `jig-protocol-worker-link`,
  `smith-host-*` or `smith-protocol-program`; if `temper-engine-shell`
  exists on your base, `cargo tree -p temper-engine-shell -e normal
  --locked --offline` holds none either, and `heavy cargo check -p
  temper-engine-shell` passes (it builds the service without the
  feature); a compile repair there is yours, its call sites alone.
  `cargo tree` builds nothing: run it directly. The report quotes the
  counts.
- **Both builds are clean:** `heavy cargo clippy -p temper-service
  --no-default-features --lib -- -D warnings` and the usual
  `--all-targets` run with the default both pass, with no `#[expect]` for
  an unused item but where a `cfg` can't reach it, said in the report.
- Every test of `temper-service` and of the dependents above passes
  unchanged.
- The status entry closes: in `temper/docs/design/protocol/README.status.md`,
  the **Not yet** 4.1, 5 entry "`temper-service` has no `worker` feature:
  …", its lines alone. This departs from the implementer's text, which
  closes `docs/design/` to you: the entry is this task's.

## Inputs

- `temper/docs/design/protocol/README.md`, 4.1 (features), section 5
  (`crates/temper-service`) and 15.1 (the shapes, the nearest).
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 5 and 6 (what
  `temper-engine-shell` calls, both ways).
- `temper/crates/temper-protocol-records/Cargo.toml` and its `forge`
  feature: the workspace's pattern of a feature and an entry with
  `default-features = false`.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`.
- The root `Cargo.toml`: `temper-service`'s `[workspace.dependencies]`
  line alone. `Cargo.lock` as cargo writes it.
- Each dependent's manifest named above: its `temper-service` line
  alone.
- Readers: a dependent that names a worker-only item compiles only with
  the feature, which it now asks for; if one that shouldn't need the
  worker (`temper-engine-shell`) fails to build without it, repair its
  call site to the both-ways names and name it.
- Pre-approved: if an engine module still names a worker crate that
  `engine-limits` left (its report says which), cut it here the same way,
  or, where it needs a change outside `temper-service`, keep that
  dependency non-optional and name it in the report: `engine-closure`
  then names it as an exception or fails on it.
- Not yours: `temper-protocol-records`' feature (the engine's half needs
  it; M3 takes it off the worker alone's build); the engine's half
  becoming optional (M3).

## Steps

1. The backstop's rows first; see them fail; then the function.
2. The manifests, then the `cfg`s and the stand-in; `heavy cargo check -p
   temper-service --no-default-features --lib` and `heavy cargo check -p
   temper-service --tests` while errors last.
3. The `cargo tree` checks; break one (drop an `optional`), see a worker
   crate appear, restore.
4. The status entry.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-service --all-targets -- -D warnings`; the
   `--no-default-features --lib` clippy; `heavy cargo nextest run -p
   temper-service`; `heavy cargo check -p temper -p temper-playground -p
   temper-fake -p temper-github-app -p temper-engine-domain-world -p
   temper-playground-world -p temper-system-world -p
   temper-real-loop-world -p temper-client-world -p
   temper-client-browser-world -p temper-one-process-world -p
   temper-engine-protocol-forge-world --tests`.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with the `cargo tree` counts, each
`cfg` that has no stand-in, and anything left.
