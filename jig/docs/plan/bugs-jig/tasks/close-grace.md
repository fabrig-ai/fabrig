+++
id = "close-grace"
part = "close-grace"
kind = "feature"
needs = []
files = ["jig/examples/ops/domain/src/**", "jig/tests/ops/src/conformance/**", "jig/tests/ops/tests/suite/acts.rs", "jig/docs/design/examples.status.md", "jig/docs/development/debt/D004-ops-root-jig-wiring.md"]
effort = "medium"
minutes = 40
+++

# An environment's close takes its grace from ops's configuration

## Goal

`ops`'s root picks a close's grace from the first offered conversation template, whichever template
made the chat: a decision a root should not make. A task keeps no template
(`jig/docs/design/domain/engine.md`, 11.1), and a close's grace is the request's, so the deployment's
close names its own, from `ops`'s configuration, beside its reason. No identity crosses the wire and
nothing stored changes.

Acceptance:

- `jig_ops_domain::Config` has `environment_close_grace: skein_lib::Duration`; `Domain` keeps it, as
  it keeps `environment_closed_reason`; `Domain::new` asserts it is within `limits.core.tasks.grace`.
- `jig_ops_domain::ENVIRONMENT_CLOSE_GRACE` is 1 second, exported beside `ENVIRONMENT_CLOSED_REASON`.
- The `infrastructure::Request::Close` arm reads no template and sends
  `grace: Some(domain.environment_close_grace)`.
- The new test fails on the old arm, and passes. Every test of `jig-ops-domain` and `jig-ops-world`
  stays green.

## Inputs

- `jig/docs/plan/bugs-jig/notes/common.md`.
- `jig/docs/design/examples.md`, 7.3 ("A chat closed with its environment");
  `jig/docs/design/domain/engine.md`, 11.1 ("Templates are served, never used").
- The code: `jig/examples/ops/domain/src/route.rs`, the `Request::Close` arm;
  `src/domain.rs` (`Config`, `Domain`, `Domain::new`); `src/limits.rs`, `ENVIRONMENT_CLOSED_REASON`.
- The world: `jig/tests/ops/tests/suite/acts.rs`, `provisioning` and
  `infrastructure_closes_the_chat_its_environment_served_once_torn_down`;
  `jig/tests/ops/src/conformance/mod.rs` (`Config`, `without_close_grant`, `build`);
  `src/conformance/config.rs`, `root_for`.

## Owns and may change

- `jig/examples/ops/domain/src/route.rs`: only the `Request::Close` arm.
- `src/domain.rs`: the field in `Config` and `Domain`, the assert, and `Domain::new`'s line.
- `src/limits.rs`, `src/lib.rs`: the constant and its export.
- `src/tests.rs` and `jig/tests/ops/src/conformance/config.rs`: the new `Config` field, each one line.
- `jig/tests/ops/src/conformance/mod.rs`: harness `Config`'s `close_grace: Option<Duration>`, its
  builder `with_close_grace`, and in `build` its overwrite of `configuration.environment_close_grace`.
- `jig/tests/ops/tests/suite/acts.rs`: split `provisioning` so a test can pass its harness `Config`;
  the new test; the old test's `expect` text, which says "its template's grace".
- `jig/docs/design/examples.status.md`: this plan's **Not yet** entry for 7.3.
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`: remove the bullet about the close grace;
  this departs from nothing, the file is no design.
- If the arm's `number` or `core::settings` imports go unused, remove them.

## Steps

1. Add the field, the constant, the harness builder; leave the arm as it is.
2. The test, `a_chat_closed_with_its_environment_takes_the_configured_grace`, modelled on
   `infrastructure_closes_the_chat_its_environment_served_once_torn_down`, with
   `Config::new(915, Story::Environment).with_close_grace(Duration::from_secs(2))`:
   assert `grace.until` is the saved act's `at` plus 2 seconds, `grace.reason` as today, then advance
   2 seconds, drain, and `closed(&world)`. See it fail with 1 second, the template's. Then change the
   arm.
3. `heavy cargo nextest run -p jig-ops-world --test suite -E 'test(=acts::a_chat_closed_with_its_environment_takes_the_configured_grace)'`,
   then `heavy cargo nextest run -p jig-ops-domain -p jig-ops-world`.
4. `cargo fmt`; old forms; `heavy cargo clippy -p jig-ops-domain -p jig-ops-world --all-targets -- -D warnings`;
   the citations check; commit and push.

## Report

As the implementer's text asks, with how the test failed on the old code.
