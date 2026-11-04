+++
id = "temper-server-part"
part = "service-clients"
kind = "migration"
needs = ["jig-server-part", "temper-workers-host"]
files = ["temper/crates/temper-service/src/server.rs", "temper/crates/temper-service/src/lib.rs", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/clients/policy.rs", "temper/crates/temper-service/src/tests/server.rs"]
effort = "medium"
minutes = 45
+++

# temper: compose jig's server

## Goal

`T/server.rs` (`T` and `J` are the paths `notes/service-parts.md` gives) goes. `temper-service` composes
`jig_protocol_engine_clients_http::server`, which `jig-server-part` shipped, under the path `crate::server`, so nothing
that reads it changes. temper's `clients/policy.rs` keeps the playground's fields and the cache values, and drops what
jig now ships.

Acceptance:

- `T/server.rs` is gone. `crate::server::{Server, Routed, Limits, valid}` still resolve.
- `temper-service`'s tests pass, with their assertions unchanged; the one test that lived in `T/server.rs` lives in
  `T/tests/server.rs`.
- temper's behaviour holds: `peer_end` is `PeerEnd::Connection`, so a peer's end during a call ends the connection, as
  it does today. Queues change from `with_capacity(8)` to uncapped, by `notes/service-parts.md`: an assert goes, nothing else.
- **Size cap:** the branch's net lines are at most -340. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.4.
- `jig/crates/jig-protocol-engine-clients-http/src/server.rs`, from `jig-server-part`.

## Owns and may change

- Owns: the deletion of `T/server.rs`; `T/clients/policy.rs`.
- May change, in the way listed:

  | File | The edit |
  | --- | --- |
  | `T/lib.rs` | `pub mod server;` becomes `pub use jig_protocol_engine_clients_http::server;` |
  | `T/limits.rs` | `peer_end: crate::server::PeerEnd::Connection` in `server`'s literal |
  | `T/tests/server.rs` | the moved test; `the_policy_keeps_a_field_the_answer_already_holds`'s `add` path |

- Checked by reading, needing no edit: `T/clients.rs` (`crate::server::Server`, `Routed`, `valid`; 7 sites), `T/clients/google.rs`
  (`hold`, `release`, now `pub`), `T/tests/fixtures/server.rs`, `T/tests/server.rs` but for the above, `T/clients/bundle.rs`
  (`policy::FILES_CACHE`, `PAGE_CACHE`). Nothing outside `temper-service` reads `temper_service::server`.
- Not yours: anything in jig. If the shipped server can't meet something temper needs, it's a `Question:`.

## What differs, and which holds

The shipped server is temper's form. The two lines where it isn't:

- **A peer's end during a call:** temper passes `PeerEnd::Connection`, which is what `T/server.rs` does unconditionally.
- **The queues:** uncapped. Nothing but the assert in `Queue::push` can see it.

`Routed`'s and `Server`'s `Debug` show counts now. No temper test reads either.

## Steps

1. `T/lib.rs`, `T/limits.rs`.
2. `T/clients/policy.rs`: `add` goes, and `FIELDS` becomes the shipped one, with `fields` built on it:

   ```rust
   pub(crate) use jig_protocol_engine_clients_http::server::policy::FIELDS;

   pub const fn fields(page: Page) -> [(&'static [u8], &'static [u8]); 5] {
       let [(name, _), nosniff, opener, resource, referrer] = FIELDS;
       match page {
           Page::Service => FIELDS,
           Page::Playground => [(name, PLAYGROUND_POLICY), nosniff, opener, resource, referrer],
       }
   }
   ```

   `PLAYGROUND_POLICY` is the `Page::Playground` arm's bytes, as they stand. `the_playground_policy_adds_only_its_worker_source`
   pins that it stays the service's policy plus `worker-src`.
3. `T/tests/server.rs`: `crate::clients::policy::add` becomes `jig_protocol_engine_clients_http::server::policy::add`.
   Move `listener_limits_accept_stock_and_refuse_zero_connections_or_stall` here from `T/server.rs`; its assertions don't
   change, `valid` is `crate::server::valid`, `crate::tests::fixtures` is `fixtures`.
4. Delete `T/server.rs`.
5. Checks, one at a time: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service
   --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-service`; `heavy cargo check -p temper --tests`; the
   citations check. Read `temper-service`'s direct dependents by `heavy cargo check --tests -p temper-client-world -p
   temper-engine-domain-world -p temper-one-process-world -p temper-system-world -p temper-playground -p temper-fake`.
6. **The browser tier.** temper's browser tier runs this server and never runs in the gate. Run `TIERS_WRAPPER=heavy
   tools/tiers/run temper browser`, the pair `browser-temper`, once, after step 5, and report its counts. It's a check you
   run and report: red is yours to read and fix, as any test of yours; if the tier can't start in your slot, say so in the
   report, and it doesn't stop your acceptance.
7. D001's measure, per file, as `notes/service-parts.md` says. `T/server.rs` is gone, so `sed` says so on stderr and
   `server` reads 0.
8. Commit and push.

## Report

The common note's items, the measure by file, the browser tier's counts, and net lines against the cap. No test is added;
the one that moves needs no break.
