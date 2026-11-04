+++
id = "temper-worker-agents"
part = "service-worker"
kind = "migration"
needs = ["jig-worker-agents", "temper-accounts-host"]
files = ["temper/crates/temper-service/src/program_sources.rs", "temper/crates/temper-service/src/worker_route.rs", "temper/crates/temper-service/src/worker.rs", "temper/crates/temper-service/src/tests/worker_parts.rs", "temper/crates/temper-service/Cargo.toml", "temper/crates/temper/src/lib.rs", "temper/crates/temper/src/worker_alone.rs", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 40
+++

# temper: compose the worker's agents, and delete its copies

## Goal

`jig-worker-agents` shipped `jig_protocol_worker_agents::{sources, route}`. Delete temper's copies and compose them.
temper keeps `configured`, its `items`, `job_items`, `job_spec`, `part`, `identity` and `answer_parts`, and the
evidence check, through the shipped decoder. `T` is the path `notes/service-parts.md` gives.

Acceptance:

- `T/program_sources.rs` holds the re-export and `configured`, and nothing else. `T/worker_route.rs` holds its wrappers,
  the `activity` re-export and temper's own functions, and nothing else.
- Every temper test passes, changed only as step 7 says. A worker's visible events don't change.
- `Resources`' `Debug` stays hidden, by the shipped hand-written one: `program_resource_debug_exposes_only_counts` passes.
- **Size cap:** the branch's net lines are at most -420, and it deletes about 500. Measure it, and stop and ask if it's
  passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`,
  `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.2.
- `jig/crates/jig-protocol-worker-agents/src/sources.rs` and `route.rs`, on the trunk: the signatures you compose.
- `jig/docs/design/protocol/channel.md`, 6.2: why the evidence's form is temper's to say.

## Owns and may change

- Owns: `T/program_sources.rs`, `T/worker_route.rs`, the five sites in `T/worker.rs` below, `T/tests/worker_parts.rs`
  as step 7 says.
- `temper/crates/temper-service/Cargo.toml`: the `jig-protocol-worker-agents` line. `smith-channel`, then read by tests
  alone, moves to `[dev-dependencies]`. Any other dependency left with no reader goes the same way; say so in the report.
  `Cargo.lock` follows.
- Readers, one edit each: `temper/crates/temper/src/lib.rs` and `temper/crates/temper/src/worker_alone.rs`:
  `service::program_sources::Resources::configured(` becomes `service::program_sources::configured(`.
- `temper/tests/worker/domain/src/protocol.rs`, its module doc only: it cites jig's `testing/jig-test-service`,
  `worker_route.rs`, for the routing; cite `jig-protocol-worker-agents`, `route`.
- `temper/docs/design/protocol/README.status.md`, **Departs**, by name, and its date line. This departs from the
  implementer's text for this file:
  - from the first "6, 7" sentence's list of drifted copies, remove `program_sources.rs` and `worker_route.rs`;
  - the second becomes: 6, 7: `temper-worker-domain/src/translate.rs` rebuilds smith's `SourceName` with
    `.expect("same source name grammar")`, so a stored name smith refuses aborts the process, where
    `jig/crates/jig-protocol-worker-agents/src/sources.rs` answers its command `Unconfigured`; no reason is recorded.
- Not yours: any jig file. If the shipped part can't meet a need here, stop and ask: the answer is a jig task.
- `settings::PROGRAM_PATH_BYTES` stays where it is: `worker_settings.rs`, `T/tests/worker.rs` and
  `temper/crates/temper/tests/support/owned.rs` read it.

Options, with their defaults:

- If `T/tests/worker_parts.rs` is left with an unused import, remove it.
- If a wrapper's signature would have to change, keep it as today and adapt inside: no test reads `route` directly but
  the three lines in step 7.

## Steps

What this branch changes from temper's copy, each settled by `jig-worker-agents`:

| Point | What holds |
|---|---|
| Path bound | `PROGRAM_PATH_BYTES`, 4096, passed by `configured`. No other value changes. |
| Delivery evidence | temper passes `route::Evidence::Reply`: the same decoder, in the same place in `decode`. |
| `Invalid` | `route::Invalid`'s four variants, each onto temper's. |
| A name smith refuses | The `expect`s in `program_sources.rs` and `smith_grant` go for jig's form. No path a worker can reach changes: a worker's `SourceName` is made only by `SourceName::new` in `jig-protocol-worker-link`'s `translate`, whose grammar is smith's, so both the answer and the `?` are unreachable there. |
| `refused`, `process`, `exited`, `activity` | Identical but for arm order. |

1. `T/program_sources.rs`, whole. `configured` was a method on `Resources`, which is now a foreign type, so it's a
   free function:

   ```rust
   //! temper's program sources (jig/docs/design/protocol/service.md, 11.2): how its settings name a program.
   use alloc::boxed::Box;
   pub use jig_protocol_worker_agents::sources::{Component, Resources};
   use smith_protocol_program::Programs;

   #[must_use]
   pub fn configured(program: &crate::settings::Program, environment: Box<[Box<[u8]>]>) -> Option<Resources> {
       match program.adapter {
           crate::settings::Adapter::ClaudeCode => Resources::new(
               Programs { claude: program.path.clone(), directory: program.directory.clone() },
               environment,
               crate::settings::PROGRAM_PATH_BYTES,
           ),
       }
   }
   ```

2. `T/worker.rs`: `sources.fire` in `abort` and `iterate`, `from_io` in `up` and `start` in `route` each take the answer
   and push it on `self.events`, in the same place, so the order holds:

   ```rust
   if let Some((owner, answer)) = self.sources.fire(now, &mut self.io) {
       self.events.push(root::Event::SourceAnswered { owner, answer });
   }
   ```

3. `T/worker_route.rs`: delete `encode`'s body, `refused`, `activity`, `decode`'s body, `process`, `evidence`,
   `decoded_delivery`, `delivery_reason`, `exited` and `smith_grant`. What stays reads:

   ```rust
   use jig_protocol_worker_agents::route;
   pub use route::activity;

   #[must_use]
   pub fn encode(value: Encodable, agent: &crate::settings::Agent) -> Encoded {
       match value {
           Encodable::Turn(turn) => match route::encode_turn(turn, &agent.limits, &agent.endpoints) {
               Some(turn) => Encoded::Turn(turn),
               None => Encoded::Undecodable,
           },
           // CallName and Result the same way.
       }
   }
   /// Decode the hub start, including its live ask (domain/worker.md, 4.1 and 7).
   pub fn decode(run: Token, source: hub::Start, agent: &crate::settings::Agent) -> Result<Box<smith::Start>, Invalid> {
       match route::decode(run, source, &agent.limits, &agent.host_limits, &agent.endpoints, route::Evidence::Reply) {
           Ok(start) => Ok(start),
           Err(route::Invalid::Charter) => Err(Invalid::Charter),
           Err(route::Invalid::Transcript) => Err(Invalid::Transcript),
           Err(route::Invalid::CallName) => Err(Invalid::CallName),
           Err(route::Invalid::Evidence) => Err(Invalid::Evidence),
       }
   }
   #[must_use]
   pub fn process(event: smith_host_protocol::ProcessEvent) -> Option<temper_worker_domain::Event> {
       match route::process(event)? {
           route::Processed::Live { agent, piece } => Some(temper_worker_domain::Event::Live { agent, piece }),
           route::Processed::Process(event) => Some(temper_worker_domain::Event::Process(event)),
       }
   }
   ```

   `items`, `job_items`, `job_spec`, `part`, `identity` and `answer_parts` don't change.
4. `T/worker.rs`'s `Request::Fact` arm reads `worker_route::activity(&body, &self.agent.limits)`. Its other `worker_route`
   sites don't change.
5. The readers above, the module doc in `temper/tests/worker/domain/src/protocol.rs`, the status file.
6. The manifest.
7. Tests. Every test in `T/tests/worker.rs`, `T/tests/fixtures/worker_peers.rs` and `T/tests/worker_parts.rs` reads a
   wrapper of the same name and stays, as a composition test, with its assertions as they are, but for:

   - `delivery_evidence_disagreeing_with_its_outcome_is_invalid` goes: `jig-worker-agents` moved it to the crate that
     holds the decoder. Its golden stays pinned there. Say so in the report, with no tier's CPU moved that matters: it
     runs a few decodes.
   - `program_resource_debug_exposes_only_counts`: `Resources::new` takes `crate::settings::PROGRAM_PATH_BYTES` as its
     third argument.

   `worker_preparation_refuses_a_program_path_past_capacity` reads `worker_settings::prepare`, not `Resources`, and
   doesn't change. None of the tests `notes/service-parts.md` lists as cited is touched.
8. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`, with `--write` only if a count fell. With
   `TEMPER_BUNDLE=none`: `heavy cargo clippy -p temper-service -p temper -p temper-worker-domain-world --all-targets --
   -D warnings`; then `heavy cargo nextest run` over `temper-service`, its direct dependents and the world whose comment
   changed: `-p temper-service -p temper -p temper-github-app -p temper-playground -p temper-fake -p
   temper-engine-protocol-forge-world -p temper-engine-domain-world -p temper-playground-world -p temper-system-world -p
   temper-real-loop-world -p temper-client-world -p temper-one-process-world -p temper-client-browser-world -p
   temper-worker-domain-world`. Check each count; read a failure's log before anything reruns. The citations check, then
   commit and push, one commit per coherent step.

## Report

The common note's items. D001's **Measure**, with the command as the item gives it, per file, against 1,388. No test is
added, and the one that goes needs no break. Net lines against the cap.
