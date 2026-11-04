+++
id = "part-leftovers"
part = "tidying"
kind = "feature"
needs = ["bite-tests", "workers-debug", "listener-text"]
files = ["jig/crates/jig-protocol-accounts/src/host.rs", "jig/crates/jig-protocol-accounts/src/host_tests.rs", "jig/crates/jig-protocol-store/src/host.rs", "jig/crates/jig-protocol-engine-clients/src/host.rs", "jig/crates/jig-protocol-engine-workers/src/host.rs", "temper/crates/temper-service/src/clients/policy.rs", "temper/crates/temper-service/src/tests/server.rs", "temper/crates/temper-service/src/tests/public.rs", "temper/crates/temper-service/src/tests/workers.rs"]
effort = "high"
minutes = 45
+++

# B6: leftovers from the move

## Goal

Five leftovers from moving the parts into product code. Behaviour stays unchanged.

Acceptance:

- `provider_templates` takes `Option<Runtime>` by value. `mem::replace` and `mut runtime` are gone from
  `jig/crates/jig-protocol-accounts/src/host.rs`, and `bite-tests`' test passes, changed only in its call.
- `Hosted::close` assigns `Queue::uncapped()` to `above`, `secrets` and `sources`, with no drain loop.
- `Hosted::secrets`' loop has one test of an event, not two, if that holds under the strict subset; see below.
- `place` in `jig_protocol_engine_clients::host` is private, and `Workers::socket` is gone.
- temper's `policy::Page` is gone: `Service` was kept for one test.
- The packages below pass. The only assertions that change are the three named under `Owns and may change`.
- **Size cap:** net at most -25 lines.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11.1, 11.3, 11.5 and 11.6.
- The strict subset: `skein/docs/foundation/programming-model.md`, 10. Every `match` lists every variant, and
  `crate::Event` has about twenty.

## Owns and may change

- Owns the product files in the header; `jig-protocol-accounts/src/host_tests.rs`, the one call.
- temper's tests, three assertions that exist only to read what goes:
  - `tests/public.rs`: drop the `kit::host::place(..)` line. jig's own
    `public_names_are_kept_as_a_key_without_an_expiry` pins that owner and number through the host's request.
  - `tests/workers.rs`: drop the `workers.host.socket(connection)` line. The `Bind` popped just above it came out of that
    table, so it pins the same fact.
  - `tests/server.rs`, `the_playground_policy_adds_only_its_worker_source`: `service` becomes `FIELDS`, and
    `assert_eq!(FIELDS, service)` goes with it. Keep the rest of the test.
- `policy.rs`: `fields(page)` becomes one function for the playground; name it `playground_fields`. `Page` and its
  `#[cfg_attr(.., expect(dead_code ..))]` go. `headers` calls it.
- **`Hosted::secrets`.** A `match` that lists every `crate::Event` costs more lines than the two `if let`s it replaces.
  If one `if let` with a guard reads no worse, take it: a `SecretsNotReady` under `NotReady::Above` stays in `above`
  whoever owns it, as it does now. Otherwise leave the two, and say so.

## Steps

1. Accounts: by value, destructuring `Runtime` once. `heavy cargo nextest run -p jig-protocol-accounts`.
2. Store: `close`, then `secrets`. `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(host)'`, then `-p
   jig-store-world`, and `-p jig-test-service --lib -E 'test(store)'`.
3. Clients and workers: `place`, `socket`. temper's policy and its three test lines.
4. `heavy cargo nextest run -p jig-protocol-engine-clients -p jig-protocol-engine-workers`, `-p temper-service --lib -E
   'test(workers) | test(public) | test(the_playground_policy)'`, and `heavy cargo check -p jig-test-service -p
   temper-service --tests`. fmt, clippy, old forms, citations. Commit, push: one commit per leftover.

## Report

As the implementer's text says, with the net lines. None of this pins a new rule; name the `Hosted::secrets` choice.
