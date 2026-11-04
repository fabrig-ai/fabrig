+++
id = "bite-tests"
part = "tidying"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-accounts/src/host_tests.rs", "jig/crates/jig-protocol-engine-clients/src/host/tests.rs", "jig/testing/jig-test-service/src/tests/stop.rs"]
effort = "high"
minutes = 40
+++

# B5: three tests that can't fail on their rule

## Goal

Three tests of the shipped parts pin less than their names say. Make each able to fail on its rule, and show it.
Product code does not change: if one of these tests fails against the code as it stands, that's a bug to stop and
ask about, not a test to soften.

Acceptance, each bitten as `notes/common.md` asks, with the break in your report:

1. **Trust bounds.** A provider on TLS with empty trust bounds is refused through `provider_templates`, and the
   check in it is what refuses: delete `if untrusted(..)` there and the new test fails. Today only the helper
   `untrusted` is tested.
2. **Endings.** `endings_leave_in_commit_order_behind_their_own_commit` fails if `ending_commit_durable` becomes
   `self.durable > 0`.
3. **A lost listener.** `a_lost_listener_changes_nothing_for_the_reference` fails if `Workers::up` in
   `jig/testing/jig-test-service/src/workers.rs` returns `false` for `ListenerClosed`, through `service.end()`: its
   assertion on `service.end()` can't fail today, since it calls `workers.up` directly and only the service's
   event routing sets `engine.end`.

- **Size cap:** net at most +50 lines. This task adds lines on purpose; part 3 as a whole ends with fewer.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11.1 (`trust`), 11.3 (a lost listener is the owner's), 11.6 (`Endings`).
- `jig/testing/jig-test/src/lib.rs` builds a `Transport::Tls` from roots, for the shape. `jig-protocol-accounts` has
  `skein-tls` as a dev-dependency.

## Owns and may change

- Owns the three test files above. No product file changes.
- May add to `jig/crates/jig-protocol-accounts/src/tests/fixtures.rs` a small fixture for the engine and the provider.
- Keep `trust_bounds_refuse_tls_alone_and_none_checks_nothing`: it covers the helper's table, cheaply.
- **If a `config::Engine` with a `sign_in` provider is too much to build in `host_tests.rs`,** put test 1 in the world
  `jig/tests/protocol-accounts`, through `Accounts::new`, and say so.

## Steps

1. In `jig/crates/jig-protocol-accounts/src/host_tests.rs`, `a_tls_provider_with_empty_trust_bounds_is_refused`: call
   `provider_templates` with one provider whose `sign_in` is set and a `Runtime` holding its endpoint on
   `driver::Transport::Tls`, for `TrustBounds { roots: 0, bytes: 1 }`; expect the `Bound` problem at
   `engine.llm.providers`. The same endpoint with `TrustBounds { roots: 1, bytes: 1 }` and with `None` is past the
   check. Call it as the code does today; a later task of this plan changes that signature and keeps this test.
2. In `jig/crates/jig-protocol-engine-clients/src/host/tests.rs`, between the first `None` and `committed(58)`:
   `endings.committed(57)`, then `41` and `42` leave, then `next()` is `None` with `58`'s still waiting, by name of
   the rule "behind their own commit".
3. In `jig/testing/jig-test-service/src/tests/stop.rs`, drive the listener's `io::Event::Closed` through the service's
   own routing (`route_events` in `jig/testing/jig-test-service/src/lib.rs`, by the queue it reads, or the nearest
   public way in), so `engine.end` can be set. Keep `workers.host.ending()` and `service.end()`; keep the name.
4. Bite each. `heavy cargo nextest run -p jig-protocol-accounts -p jig-protocol-engine-clients -p jig-test-service`,
   narrowed by `-E` to these tests while you work. fmt, clippy, old forms, citations. Commit, push.

## Report

As the implementer's text says, with `notes/common.md`'s table and the net lines.
