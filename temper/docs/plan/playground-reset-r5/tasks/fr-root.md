+++
id = "fr-root"
part = "forge-records"
kind = "interface"
needs = ["fr-family"]
files = ["temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/tests/forge_records.rs", "temper/crates/temper-engine-domain/src/tests.rs"]
effort = "high"
minutes = 50
+++

## Goal

The root carries the forge records' traffic between the connector and the
service: the connector's `ForgeKept` becomes the root's request, and the
playground's template reaches the connector from the root's
configuration (`notes/forge-records.md`, section 2). Nothing in the root
reads a record.

Acceptance:

- `temper_engine_domain::Request::ForgeKept { record:
  temper_engine_domain_forge::ForgeRecord }` beside `SetUpGroup`, emitted
  for each connector `forge::Request::ForgeKept` (`route/forge_route.rs`'s
  output arm, replacing `fr-family`'s dropping arm). Its door's room:
  `Limits.forge.forges` at once (a restart tells every record in one
  stage), reserved as the setup doors are (`super::now`, `domain.rs`).
- `Config.playground: Option<forge::Template>`, passed to
  `forge.configure_playground` beside `configure_demo` in `Domain::new`
  (`domain.rs`, about line 284); the validation construction in
  `refused` passes it too. Every `Config` literal in temper gains
  `playground: None` (forced).
- Every exhaustive match over the root's `Request` in temper gets its
  forced arm: the service's `route.rs` drops it with a comment `//
  fr-svc-route configures it`; the worlds' collectors
  (`temper/tests/engine/domain/src/{direct,protocol,walking,roles,forge_routes,escalation}.rs`,
  `temper/tests/client/src/{front,translate}.rs` and
  `translate/forge.rs`, `temper/tests/system/src/{forge,composed}.rs`)
  ignore it as they ignore `MarkedForges`.
- Test (root library tests, a new `src/tests/forge_records.rs`):
  `a_kept_forge_reaches_the_service_as_the_roots_request`, driving the
  root with a connector record made through a restored `Stored::Forge`
  row (the restore path `fr-family` gave; the connector's `Restored`
  telling it arrives with `fr-domain`: if `fr-domain` has not merged, the
  test asserts the arm by a connector request fed through the route's
  output function, and says so).

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md`,
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`, section 2.
- `temper/docs/design/domain/root.md`, the section on the forge's
  requests and doors (`rg -n 'door' temper/docs/design/domain/root.md`).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s2-box.md`,
  section 3 (the root's request variants and their readers).

## Owns and may change

- Owns: `temper-engine-domain`'s `src/boundary.rs`, `src/translate.rs`,
  `src/domain.rs` (the door and the configure call), `src/route/forge_route.rs`
  (the one arm), the new test file and its `mod` line.
- May change, forced only: the arms and `Config` literals listed above.
- Runs at once with `fr-domain` (which owns `temper-engine-domain-forge`):
  touch nothing of that crate.

## Steps

1. The request, its door, the `Config` field.
2. Forced arms (`heavy cargo check -p PKG --tests` while errors last).
3. Test. Narrow: `heavy cargo nextest run -p temper-engine-domain --lib -E
   'test(/forge_records/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain --all-targets
   -- -D warnings`; focused tests of `temper-engine-domain`; `heavy cargo
   check --tests` for `temper-service`, `temper-engine-domain-world`,
   `temper-client-world`, `temper-system-world`. Old-forms from
   `temper/`. Commit and push.

## Report

As `notes/common.md` says; every file outside the owned list you edited.
