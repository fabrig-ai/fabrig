+++
id = "fr-retire"
part = "forge-records"
kind = "migration"
needs = ["fr-worlds", "fr-fixtures", "fr-init", "bx-box", "bx-children"]
files = [
  "temper/crates/temper-service/src/startup.rs",
  "temper/crates/temper-service/src/startup/**",
  "temper/crates/temper-service/src/settings.rs",
  "temper/crates/temper-service/src/forge/stock.rs",
  "temper/crates/temper-service/src/tests/retire.rs",
  "temper/crates/temper/src/setup.rs",
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper/src/seeds.rs",
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper/src/forge.rs",
  "temper/crates/temper/src/forge_sign_in.rs",
  "temper/crates/temper/src/fake.rs",
  "temper/crates/temper/src/fake/**",
  "temper/crates/temper/tests/suite/**",
]
effort = "high"
minutes = 75
+++

## Goal

The bridge ends (`temper/docs/design/draft/playground-reset.md, 3`;
`notes/contracts.md`, 4: "`application.forges`, `identities`, `setup`
and `demo` leave the document"): the service refuses those members,
seeds come only from seed files and an in-process given, and the
binary's `Deployment` holds no setup entries or demo, so the box and
the child commands take groups only from `<data>/box/groups/N`
(D3's `bx-box`, `bx-children`). And the machine's GitHub: an optional
`application.github` names where this machine reaches GitHub (the
fake GitHub under `temper box --fake-backends`, `notes/answers.md`, "A
fresh box is empty"); absent, GitHub's own.

Acceptance:

- `temper-service` (`src/tests/retire.rs`):
  `a_document_naming_forges_is_refused` (each of the four members
  refused naming its path, saying forges are records and seeds come
  from `<data>/seed/forges/`); `application_github_gives_the_stock_github`
  (with `application.github` naming a plaintext loopback API, web and
  git, `stock::github`'s definition carries them and its record's
  connection goes there); `no_application_github_is_githubs_own`.
- The binary's `suite` passes with no document naming a setup entry:
  the child-command cases write `groups/N` with
  `temper_forge_setup::group::compose` (bx-steps).
- `temper` lib tests: `box_command`'s machine ports come from
  `application.ports` (`machine_ports`), with no `bridge_groups`.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`,
  `temper/docs/plan/playground-reset-r2/notes/scope.md` and
  `temper/docs/plan/playground-reset-r2/notes/answers.md` ("A fresh box
  is empty").
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 3
  and 4.
- `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/design/protocol/configuration.md`, section 3 (a GitHub
  forge's members: `api`, `transport`, `git`, `web`).
- `tasks/bx-box.md` (`bridge_groups`, `machine_ports`),
  `tasks/bx-children.md` (`forge::group_entry`'s document fallback),
  `tasks/fr-document.md` (seeds, `Machine`), `tasks/fr-service.md`.

## Owns and may change

Owns, after its needs (D4's `pg-` and `rm-` tasks that edit these
files come next and need this task): the service's `startup.rs`,
`startup/**`, `settings.rs`, `forge/stock.rs`; the binary's `setup.rs`,
`lib.rs`, `seeds.rs`, and its `suite` tests.

May change, pre-approved by D3 (`designers/progress-d3.md`): in
`box_command.rs`, delete `bridge_groups` and its call, and make
`machine_ports` read `application.ports`; in `forge.rs`, `fake.rs`,
`fake/check.rs`, `forge_sign_in.rs`, drop the document fallback of
`forge::group_entry`. `bx-process` then finds the bridges gone: say so
in the report. Anything `notes/scope.md` lists as forced.

## Steps

1. **The service:** `application.forges`, `identities`, `setup`,
   `demo` refused (`startup/application.rs`); the document's seeds
   (fr-registry's step 4) go; `settings::Engine.forge_seeds` holds the
   given and the seed files' only. The typed `forges`/`identities`/
   `setup` stay: they are the seeds resolved.
2. **`application.github`:** `null`, absent, or `{"api": A,
   "transport": T, "git": G, "web": W}` as a GitHub forge entry has
   them (read by the forge reader in definition mode, kind GitHub); the
   binary resolves `api` as a seed's and fills `settings::Machine.github`
   with it (endpoint and members: give `Machine.github` the members
   too, e.g. `Machine.github: Option<GitHubMachine { forge:
   Box<[u8]>, endpoint: ForgeRuntime }>`, the type yours);
   `stock::github` writes them as its forge member when given, else
   today's nulls.
3. **The binary:** `Deployment` (`src/setup.rs:12-14`) without
   `checked.setup`'s entries and `demo` (the playground's LLM members
   come from `application.playground`); the D3 call sites above.
4. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p temper-service -p
   temper --all-targets -- -D warnings`; one at a time: `heavy cargo
   nextest run -p temper-service`, `-p temper` (default profile), `-p
   temper-one-process-world -p temper-real-loop-world -p temper-fake`;
   old forms (the document members' old forms, if counted, fall).
5. Commit and push.

## Report

The standard report, plus: the `Machine.github` type; each D3 call site
edited; what `configuration.md` 3 should say of `application.github`
(for its designer).
