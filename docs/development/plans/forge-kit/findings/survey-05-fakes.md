# Survey 05: the fakes (temper's fake forge, ForgeHost, throwaway, conformance, fake git)

Base: skein b4cc6eb, temper 62d50e0b8 (origin/main), 2026-10-19. Read only. S = skein, T = temper.
Counts: prod = non-test `.rs` lines; test = lines in `tests.rs`, `*/tests/*`; `#[test]` counted by rg.
Graph names were checked with `rg` before use; the paths below are all rg-verified.

```
$ rg -l ForgeHost T            # T/tests/one-process/src/{forge,world}.rs, two docs; nowhere else
$ ls T/testing T/tests T/tools T/crates S/testing S/tests
$ /tmp/count.sh <dir>          # my wc/rg script: prod, test, #[test], Cargo deps
$ rg -l '^<crate>\b' --glob Cargo.toml T     # dependents of each fake crate
$ rg -c '\btemper_fake_forge_domain\b' --glob '*.rs' T   # use sites (likewise _protocol, temper_fake_git, throwaway)
$ grep -n '^#' T/docs/design/{tools,testing}.md; sed -n <11.3, 2.4, 4.2, 4.3, 7, 9>
```

## (a) What temper holds (path; prod / test / #[test]; deps)

| Part | Path | prod | test | #[test] | Cargo deps |
|---|---|---|---|---|---|
| fake forge domain | T/testing/temper-fake-forge-domain | 5527 | 2793 | 96 | skein-lib only |
| fake forge protocol | T/testing/temper-fake-forge-protocol | 3724 | 1585 | 55 | skein-lib, skein-http, skein-json, temper-forge-forgejo, domain; dev: temper-engine-forge-world |
| fake git program | T/testing/temper-fake-git | 1669 | 506 | 14 | skein-lib/io/shell/world, temper-fake-checkout, fake-forge-domain+protocol, temper-worker-{protocol-git, domain-checkout, checkout-world, protocol-git-world}; dev: skein-fake-machine |
| throwaway Forgejo | T/testing/temper-throwaway-forgejo | 728 | 168 | 14 (1 ignored) | temper-forge-forgejo (pin only), skein-io (digest), serde_json |
| conformance | T/tools/forgejo-conformance | 4161 | 324 | 25 | temper-throwaway-forgejo, serde_json, temper-engine-protocol-forge, temper-engine-domain-forge-client, skein-{shell,io,lib,tls} |
| domain memory tests | T/tests/fake-forge (temper-fake-forge-tests) | 6 | 451 | 3 | skein-world, skein-lib, domain |
| ForgeHost | T/tests/one-process/src/forge.rs (pub(crate)) | 268 | - | - | the crate's (13 tests overall; 30 jig/smith/temper/skein deps) |
| fake git, as a Remote | T/tests/worker/checkout/src/forge.rs (crate temper-worker-checkout-world) | 385 | - | 33 in crate | domain, temper-fake-checkout |

- Domain files: actions, api, boundary, ci, domain, faults, git, hooks, issues, limits, next, observe, pulls, reads, scenario, store, wiki. "temper" appears only in two doc lines and the fixture repository `ai/temper`.
- Protocol files: answers, delivery, errors, response, routes, server, service, setup (+accounts, hooks, orgs, tokens), tally. It uses `skein_lib::sha256::Hmac`.
- Fake git: `transport.rs` holds `Forge {domain, config, now, pending, serial}` (`Rc<RefCell<..>>`) and `Transport` (`impl Remote`); `program.rs`, `check.rs`, `answer.rs`, `fixtures.rs` are the program.
- tests/fake-checkout (21 lines) re-exports skein-fake-checkout; not part of the move.

**Blockers (deps that do not move):**
- Domain: none. Clean move.
- Protocol: `temper-forge-forgejo` (document, failure, request, server, delivery, id, limits; becomes `skein-forge`, a move not a blocker), and a test-only dev-dep: `tests/fixtures.rs:96` calls `temper_engine_forge_world::fake_config()` (T/tests/engine/forge/src/world.rs:623; that crate needs jig-domain-people and temper-engine-domain-forge*). Move the config into the protocol's own fixtures.
- Throwaway: `temper_forge_forgejo::pin::{VERSION, RUNNER_VERSION, names_runner, names_forgejo}` (-> skein-forge pin, forge.md 4.3); `serde_json` in instance.rs; hand-rolled `TcpStream` HTTP and base64 in http.rs.
- Conformance: `temper_engine_protocol_forge::{Component, Event, Request, Stopped, Answered, Limits}` and `temper_engine_domain_forge_client::{api, Limits}` in component/hosted.rs (-> skein-forge-connection, skein-forge-client); `serde_json` in about 15 files; the `TEMPER_FORGEJO` variables.
- Fake git: stays; it needs temper-worker-protocol-git (Identity, Command, invocation) and temper-worker-domain-checkout::git. Its `table()`/`scenario()` fixtures build `temper_fake_forge_protocol::Config` (tokens "fake-git-identity-token", user "worker").
- ForgeHost: names `temper_engine_protocol_forge_world::fixtures::fake_limits()`, `temper_fake_git::{Forge, Transport, fixtures::table}` and the world's `SEED`.

## (b) Target in skein, and what stays temper's (forge.md 2.1; fake-forge.md 2 to 8)

| From | To | Note |
|---|---|---|
| fake-forge-domain | S/testing/skein-fake-forge-domain | rename only; manifest as skein-fake-llm-domain (skein-lib only) |
| fake-forge-protocol | S/testing/skein-fake-forge-protocol | deps skein-lib, skein-http, skein-json, skein-forge; tests need a local `config()` |
| tests/fake-forge (memory) | S/tests/forge/machine or the domain crate | design section 8: memory binaries under the counting allocator |
| ForgeHost + `temper_fake_git::Forge` | `skein_fake_peers::forge::Peer` in S/testing/skein-fake-peers | the shared handle (domain, config, now, pending) is the peer's input; not built anywhere in skein |
| throwaway | S/testing/skein-throwaway-forgejo | rewrite: skein-http client under `skein_shell::drive`, skein-json, `from_path` only |
| conformance | S/tests/forge/conformance | rewrite off serde_json; hosts skein-forge-connection |
| protocol-forge, forge-setup/reconcile, engine/forge/client, engine/forge worlds | S/tests/forge/{protocol,setup,client,machine} | worlds listed in forge.md section 11; temper keeps their stories and ceilings |

Stays temper's: `testing/temper-fake-git` (program.rs, check.rs, answer.rs, fixtures.rs) and the `Remote` impl `Transport`; `tests/worker/checkout/src/forge.rs` (the git component's direct fake); temper's `scenario`/`fixtures` tables; the stories and cost ceilings in tests/one-process, tests/system, tests/conformance, tests/engine/*. `Transport` holds the shared handle that will be skein's type, so the handle's type is the seam.

## (c) skein-fake-peers: how LLM and OAuth peers are hosted

```
$ wc -l S/testing/skein-fake-peers/src/*.rs      # lib 21, face 152, llm 488, oauth 661, transport 325
$ rg -n 'pub (fn|struct|enum)|impl Host' S/testing/skein-fake-peers/src
```
- `lib.rs`: `mod face; pub mod llm; pub mod oauth; mod transport; pub use face::{Error, Limits}; pub use transport::Transport;`. A forge peer adds `pub mod forge;`.
- `face.rs`: `Limits {io, connections, queue (>=64), plaintext, ciphertext, observations, observation_bytes}`; `Face::new(address, limits) -> Result<_, Error::Limits>` (loopback only, queues `Listen`), `up`, `down`, `shutdown`, `close_listener`, `work_pending`, `is_empty`, `face::worst_case`.
- `llm.rs`: `llm::Peer::new(address, transport, limits, config, credential, protocol_limits, domain, domain_limits)` owns the domain; per connection `Connection{owner, wire: Wire, server: provider::Server, above, below, plain, closing}`; `impl Host for Peer` (iterate: `face.up`, pop events, `close_listener`, `progress`, `face.down`, reclaim); `llm::worst_case(limits, peer, domain, transport)`; `Observation {Accepted, Head, Query, Answered, Closed}`; `address()`, `observations()`, `shutdown()`.
- `oauth.rs`: same shape, `Peer::new(address, transport, limits, config, issuer_limits, http_limits)`, `queue(plan)`.
- `transport.rs`: `Transport {Plaintext, Tls}`, `Wire`, `validate`, `worst_case`.
- Deps: skein-lib/io/world/llm/http/oauth, the fake domain+protocol crates, rustls. Dev: skein-sim, skein-llm-world, skein-heap.
- Forge peer differences: plaintext only; the domain is borrowed from a shared handle per pass instead of owned (`llm::Peer.domain`); it fires the domain once per pass and routes `pending` with `Service::target`; it also WRITES deliveries as a client (llm and oauth only serve), so it needs an io client socket path and a delivery observation. ForgeHost today is a bespoke `Host` with fixed limits (worst_case 128 MiB, 256 operations) and no deliveries.
- skein has no `tests/forge`, no `testing/skein-fake-forge-*`, no throwaway: all of fake-forge.md is unbuilt (STATUS rows: "not planned").

## (d) Audit against fake-forge.md (V verified in code, N not met)

- V Domain boundary: `Event::Call`, `Request::Reply|Hook`, `MAX_OUT = 1`, `step`, `fire`, `next_deadline`.
- V Config: latency, late, unavailable, timeouts, landing, rate, resolution, skew, status_updates, edit_updates, hook timing; 21 limits; `Skew::{None,Ahead,Behind}`; `Tally`.
- V CI by content: `ci.rs:82` `Cue{path, green}` passes a commit whose file at the path holds the text (the 2.4 "content marker").
- V Faults at bytes (server.rs): `reset` closes before the next request is read (`next`, :305); `drop` closes and `server_error` answers 500 after the domain answered (:156-160); 429 with `Retry-After` rounded up (errors.rs:25); 401 for an unknown token; tally counts total, routes (cap 64), users, unrouted, unauthorized (tally.rs).
- V Deliveries: `Service::delivery(&hook,&limits) -> Option<Post>`, POST `/hooks/<n>`, event header, lowercase-hex HMAC-SHA256, RFC 4231 test; only the configured endpoint; empty host or secret refused; a CI status goes out as a job end.
- V Quirks tested: `leastupdate`/`max_response_items`, limit without page, 200 with no body on merge, 500 on a missing branch delete, 201 on an empty PR, comparisons unpaged, newest-created-first.
- V Throwaway: `Settings::small()`; scratch refused under /tmp (cli.rs:120) or not empty; environment cleared, HOME in scratch, fixed PATH and TZ; secrets masked in exchanges; conformance has a secret scan (`write::check`, `Leak`) and masking (mask.rs).
- N Throwaway `Forgejo::from_env()` reads `TEMPER_FORGEJO` (design 6: path only, no environment); conformance reads it too (component.rs) and `TEMPER_FORGEJO_RUNNER`.
- N Throwaway http.rs is a std `TcpStream` client with a hand chunk parser and base64 (design: skein-http client driven by `drive`).
- N serde_json in the throwaway (instance.rs) and conformance (about 15 files); the design forbids serde in skein.
- N `Instance::attach(child, port, scratch, admin, &Settings)` is in no design section; temper's process tier needs it (crates/temper/tests/support/owned.rs:399-402). Admin login "fixture", app name "temper throwaway", token name "temper-api" and logins "temper"/"temper-admin" are hard-coded.
- N `Change::Review` is delivered only as `ReviewEvent::Approved`; other review events are not rendered.
- N No forge peer, no webhook delivery to a listener: ForgeHost routes `Hook` nowhere (`pending` is left unrouted), so webhooks have never reached the engine in a world (fixtures.rs:119: `"webhooks": null`).
- N Protocol tests depend on a temper-only fixture (above).
- N Scenario configs are scattered over six places: worker/checkout forge.rs `CONFIG`, engine/forge world.rs `fake_config()`, engine/forge/client world.rs `config(faults)`, fake-forge memory.rs `CONFIG`, protocol tests/fixtures.rs, protocol-forge fixtures.rs `fake_limits()`. One shared fixture in the kit would remove the copies.
- Not checked: git-store id layout (`routes::object`, count in the first 8 bytes), `Limits::worst_case` arithmetic, and the transcripts' mask completeness (needs a binary).

## (e) Users (Cargo dependents; `.rs` use sites)

```
$ rg -l '^temper-fake-forge-domain' --glob Cargo.toml T     # 11 manifests
$ rg -c '\btemper_fake_forge_domain\b' --glob '*.rs' T      # 117 sites
```
- fake-forge-domain: tests/engine/forge (+ client) 30 sites in 9 files (simulation.rs 12, client/simulation.rs 9); tests/worker/checkout 23 (src/forge.rs, lib.rs); tests/one-process 22 (stories.rs 20, forge.rs, referee/forge.rs); testing/temper-fake-forge-protocol 14; tests/fake-forge 12; tests/engine/protocol-forge 8; tests/engine/domain 3; tests/system 3 (composed.rs, forge.rs); tests/conformance 3 (stories.rs, systems.rs); temper-fake-git 2.
- fake-forge-protocol: tests/engine/protocol-forge 14 (fixtures.rs 6, hooks.rs 3, joint, protocol, tests fake_protocol/protocol/fuzzy_protocol); tests/forge-setup/reconcile 8 (fixtures 3, world 2, exchange, tests reconcile/fuzzy_reconcile); temper-fake-git 5; tests/one-process 1 (forge.rs).
- temper-fake-git: only tests/one-process (manifest + 20 sites: stories.rs 7, forge.rs 5, fixtures.rs 3, referee/forge.rs 3, world.rs 2; world.rs:503 spawns `Proc::Git(Git::new(.., forge::transport()))`).
- ForgeHost: world.rs :131 `Proc::Forge`, :605 and :676/:683 (spawns at start and restart), :647-648 `forge::set(Some(transport))`, :706 `forge::set(None)`; stories drive it through `Transport`. No other crate.
- throwaway: tools/forgejo-conformance 30 sites in 18 files; crates/temper 9 (tests/process.rs, tests/support/{forgejo,owned}.rs; dev-dep) using `VERSION`, `RUNNER_VERSION`, `Instance::attach`, `Outgoing`, `Token`, `Settings::small()`; tools/git-transcripts 2.
- Scripts: none runs the conformance (T/scripts holds only check.sh, proof-live.sh, proof-one-process.sh). The README of tools/forgejo-conformance gives `TEMPER_FORGEJO=... cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance`. `TEMPER_FORGEJO[_RUNNER]` is read in crates/temper/tests/support/owned.rs:289-290, 460, 472-473, crates/temper-box tests, tools/git-transcripts README; proof-one-process.sh runs `cargo nextest run -p temper --profile process`.
- Data that moves with the conformance: facts.txt (221 lines), transcripts/ (152), deliveries/ (24).

## (f) The tools plan: what its design would also change

State today: the tools crates (`temper-client-tools`, `temper-client-shell-cli`), `tests/real-loop` and `tests/client`'s tools do not exist (rg of T/crates: 24 crates, none of them). So the tools plan is all new code beside the forge kit, with these touch points (client/tools.md 4, 5.3, 11.3; domain/testing.md 2.4, 4.2, 4.3, 7, 9):
- **Forge host / peer.** client/tools.md 11.3 and domain/testing.md 2.4 want "the forge peer" (skein's, on `fake-forge.md` 5) in the real loop and in the one-process world, sharing one forge with the fake git. So tools needs the kit's `skein_fake_peers::forge::Peer` first. Files it would change: T/tests/one-process/src/forge.rs (deleted), world.rs (:131, :605, :647-648, :676, :683, :706), fixtures.rs (:119-122 `webhooks` and `webhook_secret` from null to a listener and a secret), referee/forge.rs, stories.rs (7 fake-git sites, 20 fake-domain sites); T/testing/temper-fake-git/src/transport.rs (`Forge` becomes the peer's handle type; `Transport` keeps `Remote`); a new T/tests/real-loop crate that depends on skein-fake-peers (forge), temper-fake-git, temper-service, jig native transport, smith agents.
- **Webhook delivery to the engine.** The engine's listener is `crates/temper-engine-protocol-forge/src/hooks.rs` (`Webhooks {listener, secrets}`; goes to skein-forge-connection), configured by `temper-service/src/forge.rs:240` (`application.webhooks.listener`, `webhook_secret`) and `settings.rs:69,77`. A story that delivers must configure a loopback listener and secret per forge, which the one-process fixtures leave null today.
- **CI.** Content-marker CI is built in the domain (`ci.rs` cue); shell-subset CI stays later work (domain/testing.md 9). Real git stays later (the real-git bridge, S fake-forge.md 3), so the real loop uses fake git only.
- **`temper ctl adopt` flags** (`--forge --owner --name --landing --role --prefix --no-home --no-ci --check`): parsed by the new `temper-client-tools`; the fields and defaults are `temper-client-domain` `forge/form.rs` (243 lines), `vocabulary.rs`, `me.rs`, `family.rs`, and the records in `temper-client-protocol-wire`; judged by `temper-engine-domain-forge/src/adoption.rs`. temper-client-domain depends on jig-client, skein-lib, the wire and jig links only: none of the forge-kit crates, so the flags do not touch the kit. The reasons (`Missing`, `Forbidden`, `Refused`, ...) come from `skein-forge::Error` into `temper-engine-domain-forge`'s `reason`.
- **Throwaway.** The process tier (domain/testing.md 2.6) and the box harness (client/tools.md 10) need the owned Forgejo and read `TEMPER_FORGEJO[_RUNNER]` in temper's own test support; with the kit's `from_path` only, the reading of those two variables moves into crates/temper/tests/support/{forgejo,owned}.rs and `Instance::attach` needs a decision (design silent).
- **Not changed by tools:** the fake forge domain and protocol layer, the forge crates `temper-forge-forgejo`, `temper-forge-setup*`, `temper-engine-domain-forge-{client,change,issues}`, `temper-engine-domain-forge` beyond what the real-loop stories call.
- **Collision to settle.** If both plans run, both rewrite T/tests/one-process/src/{forge,world,fixtures}.rs and testing/temper-fake-git/src/transport.rs. The forge-kit plan should do the swap to the peer (with a webhook story if it likes), and the tools plan should start from the peer.
