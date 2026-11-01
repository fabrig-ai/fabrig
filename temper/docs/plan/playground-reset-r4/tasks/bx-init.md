+++
id = "bx-init"
part = "box-groups"
kind = "migration"
needs = ["bx-wire", "bx-commands", "jig-acts", "fr-make-forge", "fr-setup", "pg-start"]
files = [
  "temper/crates/temper-init/**",
  "temper/crates/temper/src/init.rs",
  "temper/crates/temper/src/init/tests.rs",
  "temper/crates/temper/tests/suite/startup.rs",
  "temper/crates/temper/tests/suite/fake.rs",
  "temper/crates/temper/tests/suite/fake_opening.rs",
  "temper/crates/temper/tests/suite/box_listeners.rs",
  "temper/crates/temper-fake/src/backends.rs",
  "temper/crates/temper-fake/src/backends/tests.rs",
  "temper/crates/temper/src/fake_backends.rs",
  "temper/crates/temper/src/fake_backends/**",
]
effort = "high"
minutes = 60
+++

## Goal

`temper init`'s document names no forge (draft 3: "A first run's document
names no forge"; contracts, D1 and section 6): its template drops
`application.forges`, `identities` and `setup`, gains `application.owned`
(the owned Forgejo it held as forge 3) and `application.demo.fake` (the
playground's fake forge it held as forge 1, with its `listen`), and its
`engine.acts` grants the reset's acts in place of `close`. A first run's box
then holds the owned and playground kinds' groups (bx-wire) and runs none
until serve asks; `SetUp` and the playground's start make their forges as
records. `temper box --fake-backends`' effective document keeps naming the
fake GitHub and Forgejo it names today, as seeded forges (D1).

Acceptance:

- The template (`temper/crates/temper-init/template/temper.json`) has no
  `application.forges`, `identities` or `setup` member (or, if the
  document's check at the base requires them, each an empty list: see
  Options); `application.owned` is the former forge 3 setup entry less
  `forge` and `start`, at the `$owned` port; `application.demo` has `llm`,
  `project` and `fake`, the former forge 1 entry less `forge`, at `$forge`,
  and no `forge`; `engine.acts` is `[{"connector": 1, "may": ["cancel",
  "remove_project", "remove_account", ...]}]`, with every grant `pg-start`'s
  and `pg-reset`'s reports name for the playground's start and reset, and no
  `close`.
- `temper init` writes it; `generated_configuration_passes_the_same_checks_as_serve`
  passes; a box on a fresh init reaches serve's link with no group running
  and the owned and playground kinds held (the suite's
  `a_box_on_a_fresh_init_reaches_serves_link_with_forge_3_dormant`,
  renamed `a_box_on_a_fresh_init_reaches_serves_link_with_its_kinds_held`).
- `temper box --fake-backends`: its rewrite (`temper_fake::backends::rewrite`)
  adds to the effective document the fake GitHub forge 2 and the managed
  fake Forgejo 4 it rewrites today (their `forges`, `identities` and
  `setup` members as the template held them, rewritten as today), so every
  fake-backends story's forge numbers stand; its own tests pass.
- temper-init's, temper-fake's and the binary's focused tests pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D1, sections 4
  and 6).
- `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.1-6.3.
- `temper/docs/design/protocol/setup.md`, 8.2 (`temper init`) and 8.3 (the
  default document), and `7.5` (the welcome's fake backends).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s1-forges-document.md`,
  section 2 (`temper-init`), and s4-box.md, section 4 (fake backends).
- The reports of `bx-machine` (the members' form), `bx-wire` (the kinds'
  groups), `jig-acts` (the grant names), `fr-make-forge` and `fr-setup`
  (`SetUp` from `application.owned`), `pg-start` (the playground's start
  from `application.demo.fake`, its grants).
- The code: `temper/crates/temper-init/src/{lib,template}.rs` and
  `src/tests/documents.rs`, `temper/crates/temper/src/init.rs`,
  `temper/crates/temper-fake/src/backends.rs` (`rewrite`).

## Owns and may change

- Owns the files in the header.
- Neighbours, each for the lines named only:
  - Engine worlds that read the template
    (`temper/tests/engine/domain/tests/fixtures/stock.rs`,
    `temper/tests/engine/domain/src/forge_routes.rs`, near the template's
    path): where a world needs the forges the template held, give it them
    from a fixture of its own tree (the old template's three members, copied
    into the world's fixture), the one change there; a world that reads the
    template for its engine members only reads it as is.
  - Every other test that reads the template (`rg -l temper-init/template`
    at the base: `temper-service/src/tests/stock_brief.rs`,
    `temper-fake/src/tests/demo.rs` and `src/oauth/tests.rs`,
    `temper/tests/real-loop/src/llm.rs`, the binary's
    `tests/support/{github_land,first_repository}.rs`): its expectations of
    the moved members updated, or the old members given from a fixture of
    its own, the one change in each.
  - `temper/docs/examples/owned.json` stays as it is (an example naming
    forges, D1); `temper-init`'s `the_default_is_the_owned_example_so_edited`
    takes the new edits (the members removed and added) in its list.
  - A process-tier test (`tests/process.rs`, `tests/support/**`) whose build
    breaks: fix the build only, and list each expectation made plainly
    wrong by a first run naming no forge (forge 1's playground, forge 3's
    set up) in the report; those stories are the playground's and the
    engine's to rewrite.
- Options:
  - If the base's document check still refuses an absent or empty
    `application.forges` (`startup/application.rs`'s `parse`: `forges` with
    no entry is `Bound`), write each as absent if that is accepted, else as
    `[]`, else stop: `Question:` (the check is the engine part's,
    `fr-make-forge`/the seed's).
  - If `application.demo.forge` is still required at the base (the demo not
    yet removed by the playground part), stop: `Question:`, naming the task
    that would remove it; do not make it optional here.

## Steps

1. The template: the members as in Acceptance; the placeholders (`$owned`,
   `$forge`, `$llm`, `$login`, `$email`, `$owned_git`) as today, each where
   its member moved. `temper-init`'s substitutions follow
   (`template.rs`); `Ports` keeps its five ports (the machine's).
2. temper-init's tests: `the_template_names_no_forge_and_holds_the_machines_members`;
   `the_template_grants_the_resets_acts_and_not_close`; the existing ones
   updated for the moved members (`no_placeholder_is_left`,
   `the_person_keeps_a_forgejo_login_or_falls_back`,
   `the_template_keeps_project_one_without_settings_members`).
3. The binary's: `init/tests.rs` as needed; the suite's startup test
   renamed and asserting the kinds held (the box's terminal names no group;
   serve's link reached; the owned and fake ports held by the box: a
   connect to each is accepted by the kernel's backlog, as
   `a_dormant_listener_queues_connections_and_is_given_again_after_restart`
   shows for a dormant group).
4. `temper-fake/src/backends.rs`: `rewrite` adds the GitHub forge 2 and the
   managed Forgejo 4 with their identities and setup entries to a document
   that names no forge, as it rewrites them today; `backends/tests.rs` and
   `temper/src/fake_backends/tests.rs` follow.
5. `cargo fmt`; `heavy cargo clippy -p temper-init -p temper-fake -p temper
   --all-targets -- -D warnings`; focused tests: `heavy cargo nextest run
   -p temper-init`, `-p temper-fake --lib`, `-p temper --lib`, `-p temper
   --test suite`; dependents build: `heavy cargo check -p temper-service
   -p temper-engine-domain-world --tests` (and every engine world package
   that reads the template; find them with `rg -l temper-init/template`).
   Commit, push.

## Report

Fab's sections; the template's new members as written; the grants and
which report each came from; every process-tier expectation left wrong.
