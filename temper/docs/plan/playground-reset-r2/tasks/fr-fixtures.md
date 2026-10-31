+++
id = "fr-fixtures"
part = "forge-records"
kind = "migration"
needs = ["fr-service", "fr-init"]
files = [
  "temper/crates/temper-service/src/tests/**",
  "temper/crates/temper/tests/suite/**",
  "temper/crates/temper/src/tests/**",
  "temper/crates/temper-fake/src/tests/**",
  "temper/crates/temper-fake/src/oauth/tests.rs",
]
effort = "medium"
minutes = 60
+++

## Goal

The focused tests that write a deployment document stop naming forges
in it (`temper/docs/design/draft/playground-reset.md, 3`): temper-service's
JSON fixtures (`src/tests/fixtures/startup.rs`'s `APPLICATION`,
`document()`, `demo_document()` and their users), the binary's `suite`
target and unit tests, and temper-fake's document tests give their
forges as seeds (fr-document's `forge_seeds`, or a seed file under
`<data>/seed/forges/` for a test that runs the binary). `fr-retire`
then refuses the document's forges. No test changes meaning.

Acceptance: `heavy cargo nextest run -p temper-service`, `-p temper`
(default profile: `suite` and the unit tests) and `-p temper-fake`
pass, and no fixture of theirs writes `application.forges`,
`identities` or `setup` (`fr-init`, merged before this task, already
moved the readers of the init template). `tests/suite/box_listeners.rs`,
which reads the owned port from `application.setup[1]`, reads it from
`application.ports.owned`.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 2
  and 4.
- `tasks/fr-document.md`, `tasks/fr-service.md`.

## Owns and may change

Owns the test files above for this task (D4's `rm-`/`pg-` next); runs
at once with `fr-worlds`, whose worlds it does not touch; after
`fr-init`, whose template readers it builds on. Production
code is not this task's: a test that cannot be written without a
production change is a question. May change anything `notes/scope.md`
lists as forced, and the binary's `tests/support/*.rs` helpers that
the `suite` target shares, only in what writes the document's forges
(the process tier's own cases are `fr-qa`'s and `fr-process`'s).

## Steps

1. temper-service: the fixtures move each forge (with its identity and
   setup entry) into a `ForgeSeed`; the 11 files that use the startup
   JSON fixture and the 9 that edit `.forges`/`.identities`/`.setup` on
   the typed `settings::Engine` keep working (the typed fields stay:
   they are the seeds resolved).
2. The binary's `suite` (`forge.rs`, `startup.rs`, `fake.rs`,
   `fake_opening.rs`, `box_listeners.rs`, `configuration.rs`) and unit
   tests: a document fixture that names forges writes them as seed files
   instead (0600 in a 0700 `<data>/seed/forges/`), or drops them when
   the test does not use them.
3. temper-fake: `src/tests/demo.rs`, `src/tests/steps.rs` and
   `src/oauth/tests.rs` the same, where they build a deployment
   document.
4. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper -p
   temper-fake --all-targets -- -D warnings`; the three nextest runs
   one at a time; old forms.
5. Commit and push.

## Report

The standard report, plus: each fixture moved, and any that could not
move without a production change.
