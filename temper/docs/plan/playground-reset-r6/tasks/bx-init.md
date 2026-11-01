+++
id = "bx-init"
part = "fs"
kind = "migration"
needs = ["bx-doc"]
files = ["temper/crates/temper-init/**", "temper/crates/temper/src/init.rs", "temper/crates/temper/src/init/**", "temper/crates/temper/src/fake_backends.rs", "temper/crates/temper/src/fake_backends/**", "temper/crates/temper-fake/src/backends.rs", "temper/crates/temper-fake/src/backends/**", "temper/crates/temper/tests/suite/startup.rs"]
effort = "high"
minutes = 120
+++

## Goal

`temper init` writes T4's document, and `temper box --fake-backends` imports and rewrites
it: no forge in either (no `application.forges`, `identities` or `setup`), the machine's own
in `application.machine`, the playground's fake forge as an entry body in
`application.demo.forge` and its project as `{"name": "Playground", "policy": 1}`, the
reset's grants in `engine.acts`. A first run's store then holds no forge; the fake
backends' GitHub is the machine's (`application.machine.github`) and their managed Forgejo is
made at runtime, as a person makes a Forgejo elsewhere.

Acceptance:
- The template (`temper/crates/temper-init/template/temper.json`), in place of
  `application.forges`, `identities` and `setup`:
  - `application.machine`: `{"owned": <forge 3's entry today, without "forge",
    "directory" and "start">, "github": {"api": null, "transport": null, "git": null,
    "web": null}}` (the owned body keeps its `bot`, `"forgejo": "fetched"`, its fetched
    runner, `listen` `$owned` and `person` `$login`/`$email`), plus any member
    `sv-startup-machine` made required (its report);
  - `application.demo`: `llm` as today; `forge`: forge 1's entry today without `forge` and
    `directory` (`bot`, `mode.fake` with `listen` `$forge`, `git` `demo/bin/git`, owner
    `playground`, repository `widget`/`main`); `project`: `{"name": "Playground",
    "policy": 1}`;
  - `engine.acts`: `[{"connector": 1, "may": ["make_project", "add_key", "adopt", "cancel",
    "remove_project", "remove_account"]}]`.
  Placeholders nothing names leave (`$forge_git`, `$owned_git` if unused); the five ports
  `temper init` reserves are unchanged.
- temper-init's tests check the new document: five distinct ports at
  `/application/clients/listener/socket`, `/application/machine/owned/mode/owned/listen/socket`,
  `/application/demo/forge/mode/fake/listen/socket`, `/application/demo/llm/listen/socket`,
  `/application/webhooks/listener/socket`; no `forges`, `identities` or `setup` member;
  `the_default_is_the_owned_example_so_edited` gains the edits that turn the owned
  example's forges, identities and setup into this shape (`docs/examples/owned.json`
  itself is unchanged here).
- The binary's `init/tests.rs` checks that the written document passes
  `crate::setup::check` with no setup entry, `Deployment.ports` holding the owned, fake
  forge and LLM ports, and the demo of T4's shape.
- `temper box` on a fresh `temper init` reaches serve's link with nothing running but serve:
  `a_box_on_a_fresh_init_reaches_serves_link_with_forge_3_dormant` becomes
  `a_box_on_a_fresh_init_reaches_serves_link_with_no_forge` (no `forge-*` directory, no
  fetched binary).
- Fake backends (`temper/crates/temper/src/fake_backends/import.rs`, `fake_backends.rs`,
  `temper/crates/temper-fake/src/backends.rs`): the import takes the new init output (its
  resources: `application/demo/forge/mode/fake/git` for `demo/bin/git`, no setup
  directories; the login at `application/machine/owned/mode/owned/person/login`); the
  rewrite sets `application.machine.github`'s `api`, `transport`, `git` and `web` to the
  fake GitHub as it set forge 2's, adds no forge, identity or setup entry (no managed forge
  4: its fake Forgejo peer still runs, at its held port, for a person or a walk to make it as
  a Forgejo elsewhere), and keeps its other edits; the fake GitHub's seeded App's setup URL
  names forge 1 (`/forges/1/github/setup`), the first forge a fresh sandbox makes; the
  binary's checks of the effective document (`validate`, the excluded ports) read the
  machine's ports instead of the setup entries.
- Every test of the default profile in `temper-init`, `temper-fake` and `temper` passes.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.5 (the welcome's fake backends), 8.2 and 8.3.
- `temper/docs/design/draft/fake-backends.md` (what the sandbox holds and rewrites).
- Draft `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.1.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T3, T4 and J8 (the grants).
- bx-doc's report; `sv-startup-machine`'s report; the answer to bx's contract request 2 in
  `temper/docs/plan/playground-reset-r6/notes/bx-progress.md` (the fakes' git program),
  if answered.

## Owns and may change

- Owns: the files in `files`.
- May change, data only: `temper/crates/temper-service/src/tests/stock.rs` and
  `src/tests/welcome_policy.rs`, the lines that read init's forges (they call
  `temper_init::files`); nothing else of temper-service.
- Not here: `temper/docs/examples/owned.json`; the process-tier tests (binary `process`,
  `tests/fake_backends/` walks) that make forges at runtime: they move in their own tasks.
- Options, pre-approved:
  - The fakes' git program. If bx's contract request 2 is answered with a document member
    (`application.machine.git_program`), the rewrite sets it to `<data>/demo/bin/git`.
    Otherwise (default) the rewrite sets no git program; say so in the report (the walks
    then need the answer).
  - A test of the default profile that asserts init's forge 1, 3 or `application.setup`
    (`rg -n 'forge-3|forges\]\[|"setup"' temper/crates/temper/tests/suite temper/crates/temper/src temper/crates/temper-fake/src`)
    changes to the new members; list each in the report.
  - `temper init`'s fallback words (`forge 3's person is person`) name the owned Forgejo
    instead: `the owned Forgejo's person is person`.

## Steps

1. The template and `src/template.rs`; temper-init's tests:
   `heavy cargo nextest run -p temper-init`.
2. `src/init.rs`, `src/init/tests.rs`:
   `heavy cargo nextest run -p temper --lib -E 'test(=init::tests::generated_configuration_passes_the_same_checks_as_serve)'`.
3. temper-fake's rewrite and its tests (`src/backends/tests.rs`: `fixture()` from the new
   init; `managed_four_and_adapter_are_ordinary_and_preserve_existing_entries` becomes
   `no_forge_is_added_and_the_adapter_is_ordinary`; `init_targets_change_together_and_preserve_registration_terms`
   reads `application.machine.github`): `heavy cargo nextest run -p temper-fake`.
4. The binary's import and checks, and `src/fake_backends/tests.rs`
   (`four_forges_pass_ordinary_admission_with_a_directoryless_managed_entry` becomes
   `the_effective_document_passes_ordinary_admission_with_no_forge`): each changed test
   alone, `heavy cargo nextest run -p temper --lib -E 'test(=fake_backends::tests::NAME)'`.
5. `heavy cargo nextest run -p temper --test suite -E 'test(=startup::a_box_on_a_fresh_init_reaches_serves_link_with_no_forge)'`.
6. Then `heavy cargo nextest run -p temper` (default profile), `-p temper-service --lib`;
   `cargo fmt`; `heavy cargo clippy -p temper-init -p temper-fake -p temper --all-targets`.
   Old forms never raised. Commit and push.

## Report

- Whether the fakes' git program is set, and how.
- Under `## Left`: every process or qa test you saw reading init's forges or the fake
  backends' forge 2 or 4, with file and line.
