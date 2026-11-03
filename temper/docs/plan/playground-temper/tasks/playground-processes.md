+++
id = "playground-processes"
part = "composition"
kind = "feature"
needs = ["playground-documents", "hosted-git-moves"]
files = [
  "temper/crates/temper-playground/Cargo.toml",
  "temper/crates/temper-playground/src/lib.rs",
  "temper/crates/temper-playground/src/machine.rs",
  "temper/crates/temper-playground/src/machine/**",
  "temper/crates/temper-playground/src/service.rs",
  "temper/crates/temper-playground/src/service/**",
  "temper/crates/temper-playground/src/programs.rs",
  "temper/crates/temper-playground/src/programs/**",
  "temper/crates/temper-playground/src/story.rs",
  "temper/crates/temper-playground/src/tests/**",
  "Cargo.lock",
]
effort = "high"
minutes = 150
+++

# The composition's processes in `temper-playground`

## Goal

The composition's processes move into `temper-playground` beside its documents: the service, the
engine and one worker in one `iterate` started from configuration bytes as `temper serve` starts
them; jig's store on skein-kv over the fake machine's disk; smith's agents hosted; the fake LLM and
fake forge peers; the fake git program and the scripted shell; the process type and its program
registry (`temper/docs/design/playground.md`, 3 and 4). The harness's seam is C2's, a world's start
C3's. Copied from `temper/tests/one-process`, which keeps its own until `one-process-composed`
rewires it; nothing outside this crate changes.

Acceptance:

- `temper_playground::{machine, service, programs, story}` hold C1 to C3's shapes: `Harness`,
  `Observe`, `Process<H>` with its arm `Other(H::Process)`, `impl Harness for ()` with `Never`,
  `ServiceHost<H>`, `Machine`, `Rooted<H>`, `SignalHost<H>`, `fresh`, `service_roots`,
  `programs::<H>()`, `Start`, `Fakes`, `Links` and `world(start)`.
- The service's host drains handover reads from the disk at their path (C3, "No box") and, with
  `Links::Print`, writes each link as one line to its standard output as
  `temper/crates/temper/src/host.rs` (~98) does; with `Links::Keep` it leaves them queued.
- `world(start)` makes the harness-free world on `skein_sim_world::World`: the simulator's seed
  `start.seed`, its trace off (`skein_sim::Trace::Off`) and no fault (`playground.md`, 4 and 8),
  the fake machine laid, the service, the fake LLM and the fake forge spawned, `programs()`
  hosted, the service's standard output a pipe the world captures (`skein_sim_world::PipeCapture`).
- The crate test below passes; `[dependencies]` stay product crates alone.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md` and `notes/contracts.md`, C1 to C3.
- `temper/docs/design/playground.md`, 3 ("The composition", "A library on every target") and 4.
- `temper/docs/design/domain/testing.md`, 2.5 (what runs).
- `skein/docs/design/world.md`, sections 2 and 7 (the captured pipe; the trace), read only.
- The report of `playground-documents` (its public items).

## Owns and may change

- `temper-playground`: `src/machine.rs`, `src/service.rs`, `src/programs.rs`, `src/story.rs` (and
  their directories), new tests under `src/tests/`; `src/lib.rs` gains their `pub mod` lines and
  re-exports only; `Cargo.toml` gains the product dependencies they need. `documents` and `fakes`
  are read, and changed only where a value they hold must become a parameter (say so).

What is copied, from `temper/tests/one-process/src/`:

- `machine.rs`: `SignalHost<H>` ~11-80, `Roots`/`Binding`/`ROOTS` ~84-93, `agent_roots` ~94,
  `Rooted<H>` ~107-225, `Machine` ~226-352, `fresh` ~353.
- `stories.rs`: `Shell` ~343-462 (the scripted `/bin/sh`).
- `world.rs`: `service_roots` ~654, `service` ~660-716, `agent` ~721, `git` ~755, `shell` ~766,
  `worker_root` ~770, `agent_root` ~773, `llm_peer` ~864, `fake_forge` ~900, `share`/`transport`/
  `SHARED` ~879-885, `peer` ~886, `host_programs` ~903-936 but its ctl registration (~905-913),
  which becomes `programs()`; the handover drain in `ServiceHost::drain` ~103-110; `Proc`'s arms
  Service, Agent, GitRoute, Git, Shell, Peer, Forge (~188-199) as `Process<H>`'s.
- `ServiceHost`'s harness fields (~83-187: capture, store, commits, wire, cut, submitted, reserve,
  codec, families, hooks) become `H::Service: Observe`'s: `ServiceHost<H>` holds the service, its
  in-flight set, its clock and `observer: H::Service`, and calls `Observe`'s hooks where those
  fields act today. Name each hook as its call site needs; each has a default doing nothing where
  `()` needs none.
- The factories' calls into the one-process world's m15 observer (`world.rs` ~663-664, ~722,
  ~756, ~767) become a hook of `Harness` (e.g. `spawned(role, inherited, base)`), default nothing.
- The GitHub layer's switch (`world.rs` ~757, ~774; `machine.rs` ~95, ~240) becomes a field of
  `Start` or a `Harness` constant, the forge's (Forgejo) the default.
- Test kits' types become the product's: `skein_world::{Inherited, HostedProgram, StartupRoot,
  PipeHost, ...}` are `skein_sim_world`'s; `AgentErrors::pipe` is `skein_sim_world::PipeWriter`;
  `temper_fake_checkout::git::Remote` is `temper_fake_git_hosted::Remote`.
- The factories take their parameters (the `Start`'s fakes and options) as the one-process world's
  do today (thread-locals set before the world steps, cleared after); yours to arrange, said in the
  report.

## Steps

1. `machine`, `programs`, `service`, `story`, copied and cut as above; `heavy cargo check -p
   temper-playground`.
2. `world(start)`.
3. The crate test, `src/tests/world.rs`: `the_world_starts_calm_and_prints_its_sign_in_link`: a
   `Start` from `documents` (its bytes), `fakes` (a fake LLM with one script, as the one-process
   world's `scripted_provider` makes, and an empty fake forge), seed 7, `Links::Print`; stepped
   until it waits idle with nothing due (a bounded loop; past it, fail naming the step): the
   captured standard output of the service holds one line, the sign-in link, of the form
   `temper serve` prints; no process ended failed. Run it twice: the two worlds' digests
   (`skein_sim_world::Digests` or `StateDigests`) are equal. `heavy cargo nextest run -p
   temper-playground --lib`.
4. `cargo tree -p temper-playground -e normal`: no test kit.
5. The wasm32 build is `playground-wasm32`'s, not this task's; run `heavy cargo check --target
   wasm32-unknown-unknown -p temper-playground` once and put its first errors (crate and item) in
   the report, whatever they are.
6. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-playground --all-targets -- -D
   warnings`. Commit; push.

## Report

As the implementer's text says, and: the public items of the new modules and the `Observe` and
`Harness` hooks, each with the one-process call site it serves (`one-process-composed` reads this);
how the factories get their parameters; the wasm32 check's first errors.
