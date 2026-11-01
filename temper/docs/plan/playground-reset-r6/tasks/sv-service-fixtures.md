+++
id = "sv-service-fixtures"
part = "fs"
kind = "migration"
needs = ["sv-playground"]
files = ["temper/crates/temper-service/src/tests/**", "temper/crates/temper-service/src/**/tests.rs", "temper/crates/temper-service/src/clients/demo_tests.rs", "temper/crates/temper-service/src/forge/records.rs", "temper/crates/temper-service/src/route.rs", "temper/crates/temper-service/src/workers.rs", "temper/crates/temper-service/src/setup/lifecycle.rs"]
effort = "high"
minutes = 120
+++

## Goal

temper-service's own tests stop building their forges from the document: the shared
fixtures (`src/tests/fixtures.rs` and `src/tests/fixtures/*.rs`) give the service no
`application.forges`, `identities` or `setup`, and register each forge they need as a
record (`root::Request::Register`, `sv-registry`), with its credential records where a test
connected it from files. So `sv-startup-refuse` can delete the document's path without
moving a test.

Acceptance:
- `fixtures::engine() -> settings::Engine` and the startup JSON fixtures
  (`src/tests/fixtures/startup.rs`) carry no document forge, identity or setup entry
  (`settings::Engine.forges`, `identities`, `setup`, `records_only` empty), and an
  `application.machine` where a test needs a template.
- A fixture helper `fixtures::with_forges(records: &[ForgeRecord]) -> Service` (or on the
  existing `started()`) registers each record through the root's request path, as the
  restore would, and puts the credential records the test needs into the loader's rows
  (as `tests/forge.rs` seeds records today); `fixtures::record(..)` (from `sv-registry`)
  builds them, `forge_settings`-style constructors (`fixtures/forge.rs`,
  `fixtures/worker.rs`, `fixtures/http.rs`) take records.
- Every test in the files named by `rg -n 'settings::Forge \{|settings::Identity \{|"forges"|"identities"|"setup"|owned_setup\(' temper/crates/temper-service/src`
  builds its forges as records, except those that test the document's members themselves
  (startup's refusals and resolutions of `application.forges`/`identities`/`setup`, which
  `sv-startup-refuse` turns into refusal tests): list those in the report.
- Every temper-service test passes (`heavy cargo nextest run -p temper-service`).

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md`, T1, T4, T5.
- `temper/docs/plan/playground-reset-r6/tasks/sv-registry.md`, `sv-registry-setup.md`,
  `sv-resolve.md` (how a record joins serve; a test record with an IP literal plaintext API
  needs no resolve).
- The inventory of test readers: `temper/crates/temper-service/src/tests/` (about 30 files:
  `fixtures.rs`, `fixtures/{startup,forge,worker,http}.rs`, `startup.rs`, `root_checks.rs`,
  `checks.rs`, `limits.rs`, `family.rs`, `forge.rs`, `translate.rs`, `worker_parts.rs`,
  `workers.rs`, `git_tokens.rs`, `system.rs`, `clients.rs`, `sign_in_configuration.rs`,
  `welcome_policy.rs`, `setup*.rs`) and `src/clients/demo_tests.rs`.

## Owns and may change

- Owns: temper-service's tests and fixtures (`files`). Production code only where a test
  seam is missing (a `#[cfg(test)]` helper), and `forge/records.rs`, `route.rs`,
  `workers.rs` for such seams alone.
- Pre-approved options:
  - A test whose subject is a file-era path that `sv-startup-refuse` deletes (a forge's
    `api_token` file, the fake's start-time adoption with files, `records_only` entries)
    stays as it is and is listed: `sv-startup-refuse` deletes it with its path.
  - Where a test's expectation named a document forge's identity number other than its
    forge number, it takes the forge's number (`notes/sv-progress.md`, request 5).

## Steps

1. The fixtures and the helper; then each test file in the inventory.
2. `heavy cargo nextest run -p temper-service` (whole), reading each failure's log.
3. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets`; old forms. Commit and
   push.

## Report

- The tests left on the document's members, by file and name, and why.

## Answers

2026-11-01, run 1: may the task fix `setup/lifecycle.rs::connected`, which sends a
registered Forgejo without a setup entry `Connected { id: config.user }`, and so `0` from
`forges::settings`, and panics the root's `same forge writer`? Yes: this is the task's.
Its owners (`sv-registry`, `sv-registry-setup`) have landed, and the task's acceptance
(every temper-service test passing on records) cannot hold without the fix, which a
records-only deployment needs anyway. `setup/lifecycle.rs` is added to `files` for this
fix alone. In `connected`'s last branch (a Forgejo with no local setup), take the id from
the forge's credential rows, `Item::User` decoded by `skein_forge_setup::item::user` as
the other two branches do, when the rows hold one, and `config.user` otherwise (the
document's forges until `sv-startup-refuse` deletes them). The login stays the git
identity's, which `forges::identity` gives a record from its bot's login. No fixture
workaround that patches the projection's user. A unit test in the service's tests (the
failing `tests::forge::a_stopped_forge_is_named_and_only_its_calls_are_not_connected` is
enough if it covers it) shows a registered Forgejo connecting with its `Item::User`.

