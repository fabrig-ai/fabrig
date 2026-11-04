+++
id = "jig-worker-agents"
part = "service-worker"
kind = "migration"
needs = ["jig-accounts-host"]
files = ["jig/crates/jig-protocol-worker-agents/**", "jig/testing/jig-test-service/src/program_sources.rs", "jig/testing/jig-test-service/src/worker_route.rs", "jig/testing/jig-test-service/src/worker.rs", "jig/testing/jig-test-service/src/tests/program_sources.rs", "jig/testing/jig-test-service/src/tests.rs", "jig/testing/jig-test-service/Cargo.toml", "jig/tests/boundary/src/tests/graph.rs", "jig/docs/design/protocol/service.status.md"]
effort = "high"
minutes = 60
+++

# jig: the worker's agents, program sources and route, as product code

## Goal

`jig-test-service` and temper's service each hold the program sources and the worker route. Create
`jig/crates/jig-protocol-worker-agents` and move `J/program_sources.rs` to its `sources` and `J/worker_route.rs` to its
`route`, in one branch. `J` keeps a thin adapter, and its tests keep passing. `temper-worker-agents` deletes temper's
copies. `T` and `J` are the paths `notes/service-parts.md` gives.

Acceptance:

- `jig_protocol_worker_agents::{sources, route}` exist with the signatures below.
- `J/program_sources.rs` holds the re-export, `PATH_BYTES` and `from_paths`, and nothing else. `J/worker_route.rs` holds
  its three wrappers, the `activity` re-export and its test module, and nothing else.
- `jig-test-service`'s tests and the worlds in step 9 pass, changed only as step 8 says.
- `jig-boundary` passes, with the new crate named in `smith_breaches` and its six smith crates listed exactly.
- `jig/docs/design/protocol/service.status.md` no longer lists these two parts.
- **Size cap:** the branch's net lines are at most +300, tests and manifests included. It rises because the new crate
  takes temper's evidence decoder (about 65 lines jig's copy never had), a manifest, and the wrappers; temper's task
  then falls by about 500. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`,
  `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.2, on the trunk: the contract.
- `jig/docs/design/protocol/channel.md`, 6.1, 6.2 and 6.4; `jig/docs/design/domain/hosts.md`, 5.1 and 6.8; they say what
  these parts are, and nothing in them changes.
- `jig/docs/design/protocol/README.md`, 5: the crate's row and "Which crates link smith", already on the trunk.
- `J/program_sources.rs`, `J/worker_route.rs`. In `T/worker_route.rs`, read-only: `evidence`, `decoded_delivery` and
  `delivery_reason`, which you copy.

## Owns and may change

- Owns: the new `jig/crates/jig-protocol-worker-agents`; `J/program_sources.rs`, `J/worker_route.rs`; the four source
  sites in `J/worker.rs`; `J/tests/program_sources.rs`, which goes, and its `mod` line in `J/tests.rs`.
- The crate's creation pre-approves: its lines in the root `Cargo.toml`'s members and `[workspace.dependencies]`, beside
  `jig-protocol-worker-link`; `Cargo.lock`; the dependency line in `jig/testing/jig-test-service/Cargo.toml`.
- `jig/testing/jig-test-service/Cargo.toml`: `smith-domain-run` goes (`J/worker_route.rs` was its only reader), and
  `smith-channel`, then read by tests alone, moves to `[dev-dependencies]`. Any other dependency left with no reader
  goes the same way; say so in the report.
- Readers, one edit each:
  - `jig/testing/jig-test/src/lib.rs`, `configure_sources`, and `jig/testing/jig-test/src/tests.rs`,
    `a_worker_reads_program_resources_from_the_agent_configuration`: `program_sources::Resources::from_paths(` becomes
    `program_sources::from_paths(`.
  - `jig/tests/channel/src/processes.rs`: `Resources::new` takes
    `jig_test_service::program_sources::PATH_BYTES` as its third argument.
- `jig/tests/boundary/src/tests/graph.rs`: step 7 only.
- `jig/docs/design/protocol/service.status.md`: from **Not yet**, 11, remove "the program sources" and "the worker
  route", and update the date line. `notes/service-parts.md` says why this is yours.
- Not yours: any temper file. `jig_test_worker::translate::grant` stays: `jig-test-worker`'s domain and its test read it.

Options, with their defaults:

- `claude` is smith's field on `Programs`. It stays as it is: `jig/AGENTS.md`'s "no system is named" is about forges and
  repositories, and `jig-protocol-config` already names it.
- If `start`'s `#[expect(clippy::too_many_arguments, ..)]` goes unfulfilled, remove it.
- If `J/worker_route.rs`'s test module needs a `use` the wrappers no longer bring, add it there.

## Steps

The rules the two copies disagree on, which this branch settles:

| Point | What holds |
|---|---|
| Path bound | A value, `Resources::new`'s third argument: `PATH_BYTES`, 1024, in jig; 4096 in temper. |
| Constructor | `from_paths` is jig's free function, temper's `configured` temper's: how settings name a program. |
| Output queue | `start`, `from_io` and `fire` return the answer; the owner pushes it. |
| A name smith refuses | jig's: it answers `Unconfigured`, pinned by `a_stored_name_smith_refuses_answers_unconfigured`. |
| `usize` conversion | jig's `skein_lib::length::usize_of`. |
| Delivery evidence | A value, `Evidence`: jig passes `Text`, the raw use it has today; temper `Reply`. |
| `Invalid` | `route`'s, with `Evidence`. jig's wrapper can't receive that variant: `unreachable!`, with the reason. |
| Limits' arguments | The values: `&Limits`, `&smith::Limits`, `&Endpoints`. |
| A grant's program name | jig's: `?`, as `jig_test_worker::translate::grant` does. |
| `refused`, `activity`, `process`, `exited` | Move. Keep jig's arm order. |

1. The crate. `Cargo.toml` as `jig-protocol-charter`'s, description "A worker's program sources and route between
   smith's host half and the channel", with `[dependencies]` `jig-domain-host`, `skein-io`, `skein-lib`,
   `smith-channel`, `smith-domain-run`, `smith-host-domain`, `smith-host-protocol`, `smith-protocol-channel`,
   `smith-protocol-program`, and `[lints] workspace = true`. `src/lib.rs`, with a module doc citing
   `jig/docs/design/protocol/service.md, 11.2`:

   ```rust
   #![cfg_attr(not(test), no_std)]
   #![forbid(unsafe_code)]
   extern crate alloc;
   pub mod route;
   pub mod sources;
   #[cfg(test)]
   mod tests;
   ```

2. `src/sources.rs`, `J/program_sources.rs` with these changes. Its module doc keeps its words, with each citation by
   its path from the root. `Resources` keeps its hand-written `Debug` and that comment: a hand impl in step code, by
   necessity, as `jig/crates/jig-protocol-accounts/src/translate.rs`'s `Candidate`.

   ```rust
   pub struct Resources { .. }
   impl core::fmt::Debug for Resources { .. }
   impl Resources {
       /// The program's paths, each at most `path_bytes`, and its explicit environment.
       #[must_use]
       pub fn new(programs: Programs, environment: Box<[Box<[u8]>]>, path_bytes: usize) -> Option<Self>;
   }

   #[derive(Debug)]
   pub struct Component { .. }
   impl Component {
       #[must_use]
       pub fn new(detail: u32) -> Self;
       pub fn configure(&mut self, resources: Resources) -> Result<(), Resources>;
       #[must_use]
       pub fn owns(&self, owner: Token) -> bool;
       #[must_use]
       pub fn settled(&self) -> bool;
       pub fn start(
           &mut self,
           owner: Token,
           io_owner: Token,
           name: jig::SourceName,
           operation: sources::Operation,
           root: Fd,
           deadline: Time,
           below: &mut Queue<io::Request>,
       ) -> Option<(Token, sources::Answer)>;
       pub fn from_io(&mut self, event: io::Event, below: &mut Queue<io::Request>) -> Option<(Token, sources::Answer)>;
       pub fn fire(&mut self, now: Time, below: &mut Queue<io::Request>) -> Option<(Token, sources::Answer)>;
       #[must_use]
       pub fn next_deadline(&self) -> Option<Time>;
   }
   ```

   `failed` returns `(Token, sources::Answer)`, and the private `answer` returns the same in an `Option`, clearing
   `command` as it does. `start` returns `Some` wherever it pushed `above`, and `None` once a command runs. `from_paths`
   goes to the adapter. `ENVIRONMENT_ENTRIES` and `ENVIRONMENT_BYTES` stay in this file.

3. `src/route.rs`. Module doc: smith's payloads are translated in the worker layer, citing
   `jig/docs/design/protocol/channel.md, 6.1 and 6.4`. The bodies are `J/worker_route.rs`'s, split by `Encodable`'s
   arms; `evidence`, `decoded_delivery` and `delivery_reason` are `T/worker_route.rs`'s; `delivery` and `grant` stay
   private. `Undecodable` becomes `None`.

   ```rust
   /// What a settled delivery's evidence is (jig/docs/design/protocol/channel.md, 6.2).
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum Evidence {
       /// The bytes are a receipt's or a refusal's words, as they stand.
       Text,
       /// The bytes are smith's `DeliveryReply`, which must decode and agree with the outcome.
       Reply,
   }

   /// What of the hub's start did not decode.
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum Invalid {
       Charter,
       Transcript,
       CallName,
       /// A resumed delivery's evidence, which must decode and agree with its outcome.
       Evidence,
   }

   /// An agent process's event, in the worker root's terms.
   #[derive(Debug)]
   pub enum Processed {
       Live { agent: Token, piece: hub::Piece },
       Process(Box<smith::process::Event>),
   }

   #[must_use]
   pub fn encode_turn(turn: smith::Turn, limits: &Limits, endpoints: &Endpoints) -> Option<hub::Turn>;
   #[must_use]
   pub fn encode_call_name(name: smith::CallName, limits: &Limits) -> Option<Box<[u8]>>;
   #[must_use]
   pub fn encode_result(result: smith::RunResult, limits: &Limits) -> Option<Box<[u8]>>;
   #[must_use]
   pub fn activity(body: &[u8], limits: &Limits) -> Option<hub::AgentActivity>;
   pub fn decode(
       run: Token,
       source: hub::Start,
       limits: &Limits,
       host_limits: &smith::Limits,
       endpoints: &Endpoints,
       evidence: Evidence,
   ) -> Result<Box<smith::Start>, Invalid>;
   /// Decode a resumed delivery's evidence and check its committed outcome.
   pub fn evidence(outcome: hub::DeliveryOutcome, bytes: &[u8]) -> Result<smith::Delivery, Invalid>;
   #[must_use]
   pub fn process(event: ProcessEvent) -> Option<Processed>;
   ```

   `Limits` is `smith_host_protocol::Limits`, `Endpoints` `smith_protocol_channel::Endpoints`, `smith`
   `smith_host_domain`, `hub` `jig_domain_host`. `decode` keeps today's order: charter, transcript, answered calls,
   messages, grants. In its `SettledAnswer::Delivery` arm, `Evidence::Reply` calls `self::evidence(outcome, &bytes)?` and
   `Evidence::Text` calls `delivery(outcome, bytes)?`. `encode_result` keeps `Parked` and `Failed` as an empty body.

4. The adapters. `J/program_sources.rs`, whole:

   ```rust
   //! The reference worker's program sources (jig/docs/design/protocol/service.md, 11.2):
   //! its bound on a path, and its startup's paths.
   use alloc::boxed::Box;
   pub use jig_protocol_worker_agents::sources::{Component, Resources};
   use smith_protocol_program::Programs;

   /// The reference worker's bound on a program path.
   pub const PATH_BYTES: usize = 1024;

   /// Startup's already parsed agent paths and explicit environment.
   #[must_use]
   pub fn from_paths(directory: Box<[u8]>, claude: Box<[u8]>, environment: Box<[Box<[u8]>]>) -> Option<Resources> {
       Resources::new(Programs { directory, claude }, environment, PATH_BYTES)
   }
   ```

   `J/worker_route.rs`, its signatures as today, so `J/worker.rs` and `J/tests/worker.rs` read them unchanged:

   ```rust
   pub(super) use jig_protocol_worker_agents::route::activity;

   pub(super) fn encode(value: Encodable, limits: &Limits, endpoints: &Endpoints) -> Encoded {
       match value {
           Encodable::Turn(turn) => match route::encode_turn(turn, limits, endpoints) {
               Some(turn) => Encoded::Turn(turn),
               None => Encoded::Undecodable,
           },
           // CallName and Result the same way.
       }
   }
   pub(super) fn decode(run: Token, source: hub::Start, agent: &crate::Agent) -> Result<Box<smith::Start>, Invalid> {
       match route::decode(run, source, &agent.limits, &agent.host_limits, &agent.endpoints, route::Evidence::Text) {
           Ok(start) => Ok(start),
           Err(route::Invalid::Charter) => Err(Invalid::Charter),
           Err(route::Invalid::Transcript) => Err(Invalid::Transcript),
           Err(route::Invalid::CallName) => Err(Invalid::CallName),
           Err(route::Invalid::Evidence) => unreachable!("text evidence is never decoded"),
       }
   }
   pub(super) fn process(event: smith_host_protocol::ProcessEvent) -> Option<jig_test_worker::Event> {
       match route::process(event)? {
           route::Processed::Live { agent, piece } => Some(jig_test_worker::Event::Live { agent, piece }),
           route::Processed::Process(event) => Some(jig_test_worker::Event::Process(event)),
       }
   }
   ```

5. `J/worker.rs`: each of the four source sites takes the answer and pushes it on `source_events`, in the same place,
   so its order holds. `start`'s arm, `up`'s `from_io`, `abort`'s `fire` and `iterate`'s `fire`, each so:

   ```rust
   if let Some((owner, answer)) = self.sources.fire(now, &mut self.io) {
       self.source_events.push(jig_domain_host::Event::SourceAnswered { owner, answer });
   }
   ```

6. The readers above.
7. `jig/tests/boundary/src/tests/graph.rs`. `smith_breaches`' `allowed` match gains an arm, as the charter's does:
   `"jig-protocol-worker-agents"`, with exactly `smith-channel`, `smith-domain-run`, `smith-host-domain`,
   `smith-host-protocol`, `smith-protocol-channel` and `smith-protocol-program`. Rename
   `only_the_charter_and_the_transcript_link_smith_even_when_empty` to
   `only_the_charter_the_transcript_and_the_worker_agents_link_smith_even_when_empty`: no document cites it. The rule
   follows `jig/docs/design/protocol/README.md`, 5. The rest of the file doesn't change.

8. Tests:

   - **Move,** to `src/tests/sources.rs` (with `src/tests.rs` declaring `mod route;` and `mod sources;`, as
     `jig-protocol-engine-workers` lays out its own): `J/tests/program_sources.rs`' three,
     `program_resources_debug_shows_only_counts`, `a_configured_component_formats_without_the_environment` and
     `a_stored_name_smith_refuses_answers_unconfigured`. `resources()` calls
     `Resources::new(Programs { .. }, Box::new([entry.finish()]), 1024)`. The third test can't compile as it stands: it
     reads `let (answered, answer) = component.start(.., &mut below).expect("source answer");`, and
     `assert!(above.is_empty())` goes with the queue. Every other assertion stays.
   - **Move,** to `src/tests/route.rs`: `delivery_evidence_disagreeing_with_its_outcome_is_invalid`, as it stands in
     `T/tests/worker_parts.rs`, its golden included, with `route::evidence` and `route::Invalid`. Nothing else in jig
     reads the decoder. `temper-worker-agents` deletes temper's.
   - **Stay:** `J/worker_route.rs`'s test module (`each_live_kind_keeps_every_field`,
     `a_live_kind_disagreeing_with_its_call_is_dropped`, `the_start_passes_each_ask_and_omits_an_empty_one`), which read
     the wrappers, and `J/tests/worker.rs`, unchanged. These are composition tests by `notes/service-parts.md`.
   - None of the tests `notes/service-parts.md` lists as cited is touched.

9. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-worker-agents -p
   jig-test-service -p jig-test -p jig-boundary -p jig-channel-world --all-targets -- -D warnings`; then
   `heavy cargo nextest run -p jig-protocol-worker-agents -p jig-test-service -p jig-test -p jig-boundary -p
   jig-channel-world -p jig-one-process-world -p jig-engine-clients-world -p jig-client-protocol-world`. Check each
   count; read a failure's log before anything reruns. The citations check, then commit and push, one commit per
   coherent step.

## Report

The common note's items. The break: add `smith-charter.workspace = true` to the new crate's manifest, and name what the
renamed test reads. The moved tests need none. Net lines against the cap, and how many tests moved, with each tier's
figure from your run.
