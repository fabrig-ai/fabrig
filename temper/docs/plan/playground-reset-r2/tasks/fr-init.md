+++
id = "fr-init"
part = "forge-records"
kind = "feature"
needs = ["fr-service", "jig-acts"]
files = [
  "temper/crates/temper-init/**",
  "temper/crates/temper/src/init.rs",
  "temper/crates/temper/src/init/**",
  "temper/tests/engine/domain/tests/fixtures/stock.rs",
  "temper/tests/engine/domain/src/forge_routes.rs",
]
effort = "medium"
minutes = 60
+++

## Goal

`temper init` writes contract 4's document
(`temper/docs/design/draft/playground-reset.md, 3`, "A first run's
document names no forge"): no `application.forges`, `identities` or
`setup`; `application.ports` and `application.playground` (fr-document
put them in the template); `engine.acts` granting the deployment's
connector `close`, `remove_account`, `cancel` and `remove_project`. A
fresh deployment starts with no forge; its playground's fake forge is
fr-service's seed, forge 1.

Acceptance:

- `temper-init` (`src/tests/documents.rs`):
  `the_document_names_no_forge` (the written `temper.json` has none of
  the three members, has `ports` with the reserved owned, forge and llm
  ports, and `playground`); `engine_acts_grant_the_resets_acts` (the
  grant lists exactly `close`, `remove_account`, `cancel`,
  `remove_project`); the existing test that the service reads the
  written document (`rg -n 'temper_service' src/tests`) passes, and the
  service it starts lists no forge.
- The binary's `src/init/tests.rs` passes, its facts unchanged (five
  ports).
- Every reader of the init template passes unchanged in meaning
  (`temper/tests/engine/domain/tests/fixtures/stock.rs:4`,
  `temper/tests/engine/domain/src/forge_routes.rs:1921`,
  `temper-fake/src/tests/demo.rs:414`, `temper-fake/src/oauth/tests.rs:105`,
  the binary's `tests/suite/fake_opening.rs:191` and
  `tests/suite/startup.rs:1736,1771`); the process tier's readers
  (`tests/support/first_repository.rs`, `github_land.rs`) are
  `fr-process`'s.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 1
  (the acts' names) and 4.
- `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.3
  ("What goes": `close` is replaced by the reset's grants, which is
  D4's; this task adds `cancel` and `remove_project` beside it).
- `temper/docs/design/protocol/setup.md`, 8.2 and 8.3.
- `tasks/fr-document.md`, `tasks/fr-service.md`.

## Owns and may change

Owns `temper-init` (its template, `src/template.rs`, `src/lib.rs`,
tests) and the binary's `src/init.rs` and `src/init/` (D4's `pg-` own
them next).

May change:

- The template readers listed in Acceptance, in any crate: what they
  assert about the template's forges.
- A help page under `temper/docs/help/` (`box.md`, `forge.md`,
  `forgejo.md`, `github.md`) that names the default document's forges,
  identities or setup entries: only to drop or rename those names (a
  departure from "help pages are the leads'", allowed by
  `brief-common.md` for a removed name), with temper-docs' tests
  (`heavy cargo nextest run -p temper-docs`) and the binary's help tests.
- Anything `notes/scope.md` lists as forced.

## Steps

1. The template: remove `application.forges`, `identities`, `setup`;
   `engine.acts` as Goal; the limits' overrides that existed for three
   forge slots (`forge.client.repositories`, `forge.repositories`: 3,
   `forge.output`: 82) stay as the deployment's counts (draft 3,
   "Limits are counts"), unless a test shows a default fits: say which.
2. `src/template.rs`: the placeholders no member uses any more
   (`$forge_git`, `$owned_git`, `$login`, `$email`) go; `Facts.login`
   stays if anything else reads it, else say so (the service takes the
   login at serve start, fr-document).
3. The `demo/bin/git` link and the scripted model's key init writes stay
   (the playground's stock definition names them).
4. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p temper-init -p
   temper --all-targets -- -D warnings`; one at a time: `heavy cargo
   nextest run -p temper-init`, `-p temper` (default profile), `-p
   temper-fake`, `-p temper-engine-domain-world`, `-p temper-docs` if a
   help page changed; old forms.
5. Commit and push.

## Report

The standard report, plus: the limits kept and why; each help page
edited; what `setup.md` 8.3 should now list (for its designer).
