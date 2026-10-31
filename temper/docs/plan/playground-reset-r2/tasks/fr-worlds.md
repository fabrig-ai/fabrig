+++
id = "fr-worlds"
part = "forge-records"
kind = "migration"
needs = ["fr-service"]
files = [
  "temper/tests/one-process/**",
  "temper/tests/real-loop/**",
  "temper/tests/system/**",
  "temper/tests/conformance/**",
]
effort = "medium"
minutes = 60
+++

## Goal

The in-process worlds stop naming forges in their deployment documents
(`temper/docs/design/draft/playground-reset.md, 3`): each gives its
forges as seeds through `startup::with_setup_given`'s `forge_seeds`
(fr-document), the forge's members moved unchanged into a seed's
definition, so `fr-retire` can refuse the document's forges. No test
changes meaning.

Acceptance: `temper-one-process-world`, `temper-real-loop-world`,
`temper-system-world` and `temper-conformance-world` pass, their
documents naming no `application.forges`, `identities` or `setup`
(`rg -n '"forges"|"identities"|"setup"' temper/tests/{one-process,real-loop}`
finds no deployment-document member; fake-forge seed documents such
as `real-loop/tests/fixtures/b0/forge.json` are not deployment
members and stay).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 2
  (the definition).
- `tasks/fr-document.md` (seeds, `ForgeSeed`), `tasks/fr-service.md`.

## Owns and may change

Owns the four worlds (`temper/tests/one-process`, `real-loop`,
`system`, `conformance`) for this task; `fr-` only now, D4's `rm-`/`pg-`
next. May change anything `notes/scope.md` lists as forced.

## Steps

1. **one-process** (`src/fixtures.rs:110` `world_document`, `:190`
   `files`, `src/stories.rs:310-322`, `src/lower/github.rs:40`,
   `src/restart_control.rs:44-78`, `src/world.rs:665-690`): the forge,
   identity and setup entries become one `settings::ForgeSeed { number:
   1, definition }` (`{"forge": F, "identity": I, "setup": S}` less
   their numbers), passed to `with_setup_given`; the credential files
   stay where they are and the definition names them as the document
   did. `restart_control.rs`'s setup check reads the seed's entry.
2. **real-loop** (`src/fixtures.rs:16` `documents`, `:147-163`
   `bind_documents`, `src/forge.rs:45-60`, `src/github.rs:152`): the
   same, `bind_documents` binding the seed's definition instead of
   `/application/forges`.
3. **system, conformance:** typed (`walking::config`), nothing to move
   unless the compiler names a site (`tests/system/src/composed/link.rs:30`
   builds a `settings::Forge` by hand: unchanged).
4. `cargo fmt`; `heavy cargo clippy -p temper-one-process-world -p
   temper-real-loop-world --all-targets -- -D warnings`; one at a time:
   `heavy cargo nextest run -p temper-one-process-world`, `-p
   temper-real-loop-world`, `-p temper-system-world -p
   temper-conformance-world`; old forms.
5. Commit and push.

## Report

The standard report, plus: any world that needed a stock definition
rather than its own.
