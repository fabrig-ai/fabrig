+++
id = "sv-resolve"
part = "fs"
kind = "feature"
needs = ["sv-registry-setup"]
files = ["temper/crates/temper-service/**", "temper/crates/temper/src/host.rs"]
effort = "high"
minutes = 80
+++

## Goal

A registered record whose API needs io (a host name to resolve, or TLS trust to read) is
resolved by serve's host, as serve's handovers are read today, and then joins the forge
connection and the setup host (`notes/sv-progress.md`, Contract requests, 2). Until this
task such a record is held (`sv-registry`).

Acceptance:
- `temper_service::Resolve { forge: u16, host: Box<[u8]>, port: u16, tls: Option<Tls> }`
  and `temper_service::Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> }` (a trust
  path, `None` for the machine's roots), public beside `HandoverRead`.
- `Service::take_resolve(&mut self) -> Option<Resolve>` gives, once each, the resolve of
  every registered record that is held for io (host name, or TLS); `Service::resolved(&mut
  self, forge: u16, answer: Result<ForgeRuntime, Box<[u8]>>)` takes the host's answer
  (`ForgeRuntime` as today: number, address, transport with its built TLS config).
- On `Ok`: the forge connection's held forge is replaced (`Request::Remove` then `Add` with
  the resolved address and transport; it holds no access yet), the layer's held entry
  goes, the setup host adds it (`sv-registry-setup`'s `Layer::add`; it leaves `unserved`),
  and when the forge records' loader has passed `Phase::Connect` the forge connects from
  its records as that phase would have (`sv-registry`'s rule), or its local reconcile
  starts as its trigger says. On `Err(words)`: it stays held, noted once
  `Note::ForgeNotConnected { number, why: NotConnected::Unreachable { host } }`; a later
  `Register` of the record (an update) or the next start asks again. An answer for a forge
  not registered, or not held, is ignored.
- At start the loader's `Phase::Connect` passes a held record only once its resolve is
  answered (the host answers between iterations, so this waits one iteration at most).
- An IP literal host with TLS is resolved by the host too (its trust needs a file read); an
  IP literal with plaintext is never asked (`sv-registry` adds it reachable).
- The binary's host (`temper/crates/temper/src/host.rs`, `Host::drain`) answers each
  resolve beside the handover reads: `address(&Address::Host { name, port })` (or the IP)
  and `trust(trust.as_deref())` from `lib.rs`, the server name checked with
  `server_name`, exactly as `lib.rs::forges` resolves a document forge today; an error's
  text is the answer's words.
- Every test that drives a `Service` with its own host loop still passes (none registers a
  held record yet).

## Inputs

- `temper/docs/design/protocol/setup.md`, 8.3 (a forge whose host does not resolve is
  held); `temper/docs/design/protocol/forge.md`, 2.2.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T1 (`Api`).
- `temper/docs/plan/playground-reset-r6/notes/sv-progress.md`, Contract requests, 2.
- The handover's precedent: `temper-service/src/lib.rs` (`HandoverRead`),
  `system.rs::take_handover_read`/`handover_read`, `temper/crates/temper/src/host.rs`
  (`drain`, `read_handover`).

## Owns and may change

- Owns: `temper/crates/temper-service/` (`src/lib.rs` for the public types, `src/system.rs`
  for the two methods, `src/forges.rs`, `src/forge.rs`, `src/forge/records.rs`,
  `src/setup.rs`), and their tests.
- `temper/crates/temper/src/host.rs`: `drain`'s new loop and its `resolve_forge` function,
  alone. It calls the crate root's private `address`, `trust` and `server_name`
  (`temper/crates/temper/src/lib.rs`), which a child module may call as they are; nothing
  of `lib.rs` changes. No bx task touches `host.rs` (the lead's word); nothing else of the
  binary changes here.
- Pre-approved options:
  - The host's lookup blocks the loop for its time, as the handover's file reads do.
    Default as said; say in the report how long a refused lookup took on the builder (a
    `localhost` name and an unknown `.invalid` name, from the unit test below).
  - If a world crate (`temper/tests/*`) drives a `Service` with its own host and a test
    there registers a held record, it answers `take_resolve` as the binary does; none is
    expected in this task.

## Steps

1. `src/lib.rs`: `Resolve`, `Tls`. `src/forges.rs`: a held record queues its resolve
   (`engine.resolves: Queue<Resolve>`, capacity `forge_component.forges`), and `resolved`
   applies the answer as the acceptance says.
2. `src/system.rs`: `take_resolve`, `resolved`.
3. `src/forge/records.rs`: `Phase::Connect`'s wait for an outstanding resolve.
4. The binary: `host.rs::drain` and `fn resolve_forge(resolve: &temper_service::Resolve) ->
   Result<temper_service::ForgeRuntime, Box<[u8]>>`.
5. Tests:
   - temper-service, `src/tests/resolve.rs`: `a_held_record_is_asked_once_and_joins_both_layers_when_resolved`,
     `a_refused_resolve_keeps_it_held_and_names_it_once`,
     `an_answer_for_a_forge_not_held_is_ignored`,
     `the_loader_waits_for_an_outstanding_resolve_then_connects_from_records`,
     `an_ip_literal_with_tls_is_asked_and_a_plaintext_one_is_not`.
   - the binary, in `src/host.rs`'s tests (or `src/tests.rs`):
     `resolve_forge_resolves_localhost_and_reads_machine_trust`,
     `resolve_forge_names_an_unknown_host` (`unknown.invalid`).
6. Narrow runs, one at a time:
   `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::resolve::/)'` (5);
   `heavy cargo nextest run -p temper --lib -E 'test(/resolve_forge/)'` (2). Then
   `heavy cargo nextest run -p temper-service`; `heavy cargo nextest run -p temper --lib`;
   `heavy cargo check --workspace --tests`.
7. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper --all-targets`; old forms.
   Commit and push.

## Report

- The two lookup times measured.
- Under `## Left`: `lib.rs::forges` and `ForgeRuntime`'s start-time list still serve the
  document's forges (they go in `sv-startup-refuse`).
