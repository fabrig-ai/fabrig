+++
id = "pg-config"
part = "playground"
kind = "interface"
needs = ["bx-machine", "forge-asks", "fr-seed"]
files = [
  "temper/crates/temper-service/src/settings.rs",
  "temper/crates/temper-service/src/startup/application/demo.rs",
  "temper/crates/temper-fake/src/demo.rs",
  "temper/crates/temper-fake/src/tests/demo.rs",
  "temper/crates/temper-service/src/clients.rs",
  "temper/crates/temper-service/src/clients/demo_tests.rs",
  "temper/crates/temper-service/src/tests/playground_config.rs",
]
effort = "high"
minutes = 45
+++

## Goal

What the playground's start (`pg-start`) reads, ready before it: the
engine's `DemoConfig` carries the playground's fake forge as a `Forge`
record (less its number), made from `application.demo.fake`
(`bx-machine`), so the start makes the forge through `make::make`
(`fr-make-forge`); `application.demo.forge` becomes optional, so a
first run's document can name no forge (`bx-init`); and the service
carries the scripted key to the root's admission for `StartPlayground`
as it does for `StartDemo` (draft
`temper/docs/design/draft/playground-reset.md`, 6.1; contracts, 6).

Acceptance:

- `temper_engine_domain_forge::DemoConfig` gains `pub forge:
  Option<crate::Forge>`: with `application.demo.fake`, a record whose
  `number` is 0, `kind` Forgejo, `api` the fake's `listen` socket,
  `transport` plaintext, `git` `http://LISTEN` (as `temper init`'s
  `$forge_git` is written), `web` none, `read_only` false, `identity`
  number 0 with the entry's bot login, the bot's login as name and its
  email, `files` none, `setup` the fake entry (`Template::entry(0)`,
  typed as `record::Setup` with `SetupMode::Fake`), `first` none,
  `playground` true, `removing` false; `Forge::fits` holds for the
  template's; none without `fake`;
- `application.demo.forge` is optional: absent, `fake` is required
  (refused at `application.demo` naming `fake` otherwise);
  `settings::Demo.forge` and `temper_fake::demo::Demo`'s forge are
  `Option<u16>`; present, it is checked as today and the seed
  (`fr-seed`) marks that forge `playground` as today;
- `DemoConfig.adoption`'s `forge` and `project` are the document's
  (0 when absent): `pg-start` replaces both with the `Playground` row's;
- a `StartPlayground` from the wire is parked and its key file read and
  carried before admission exactly as `StartDemo`'s is
  (`clients.rs`), so the root's admission sees `carried`;
- every document that passes today passes unchanged; the tests below
  pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 2
  and 6).
- `temper/docs/design/protocol/setup.md`, 8 (the setup entry) and 8.4
  (`application.demo`).
- `temper/docs/plan/playground-reset-r4/tasks/bx-machine.md`
  (`Template`, `settings::Demo::fake`, `temper_fake::demo::Demo::fake`)
  and `tasks/store-families.md` (`record::Forge` and its parts).

## Owns and may change

Owned: the `files` above (`tests/playground_config.rs` is new).

Neighbour files, edits allowed:

- `temper-engine-domain-forge/src/boundary.rs`: `DemoConfig`'s one new
  field, with a doc line; `DemoConfig::within` (in `demo.rs`) checks the
  record with `Forge::fits` when present;
- every literal of `DemoConfig`, `settings::Demo` and
  `temper_fake::demo::Demo` in the temper tree (tests, worlds): the new
  field (`forge: None`) or the option (`Some(n)`), nothing else;
- every reader of `settings::Demo.forge` / the fake demo's forge
  (`temper-fake/src/steps.rs`, `temper-service/src/startup/seed.rs` or
  wherever `fr-seed` reads it, `settings.rs`' `domain`): read the
  option; with none, the reader behaves as with no demo forge;
- `temper-service/src/tests.rs`: one `mod` line.

Pre-approved:

- the record is built in `settings.rs` (`Demo::domain`), or in a
  function of `startup/application/demo.rs` that `domain` calls;
  default: a `fn playground_forge(fake: &temper_forge_setup::setup::Template)
  -> Option<temper_engine_domain_forge::Forge>` in `demo.rs`;
- the parked start's `HandoverRead.forge` is the document's demo forge,
  or 0 when it names none, if the read's answer is matched by its `run`
  alone (check `system.rs`' `handover_read` and `clients.rs`); else say
  what it needs.

## Steps

1. `DemoConfig.forge` and the record of the acceptance.
2. The optional `forge` in both readers (`startup/application/demo.rs`,
   `temper-fake/src/demo.rs`), and the readers' option.
3. `clients.rs`: the parked start for `StartPlayground` beside
   `StartDemo`'s (one match on either ask).
4. **Tests**:
   - `temper-service/src/tests/playground_config.rs`:
     `the_templates_fake_makes_the_playgrounds_record` (the template's
     document with an `application.demo.fake` equal to its forge 1 entry
     less `forge`: the record of the acceptance, `fits`);
     `a_demo_without_forge_needs_its_fake`;
     `a_demo_without_fake_has_no_record`.
   - `temper-fake/src/tests/demo.rs`: `a_demo_with_fake_and_no_forge_reads`.
   - `clients/demo_tests.rs`:
     `start_playground_reads_each_time_and_carries_the_key_before_admission`,
     after its `StartDemo` twin.
   - Narrow runs: `-p temper-service --lib -E 'test(playground_config::)
     | test(demo_tests::)'`, `-p temper-fake --lib -E 'test(demo::)'`.
5. `cargo fmt`; clippy on `temper-service`, `temper-fake`,
   `temper-engine-domain-forge`; their focused tests; `heavy cargo check
   --tests` for `temper`, `temper-engine-domain`,
   `temper-engine-domain-world`, `temper-system-world`,
   `temper-one-process-world`, `temper-client-world`, one at a time.
   Commit, push.

## Report

fab's sections; `## Interfaces`: `DemoConfig.forge`, the builder's
path and signature, `settings::Demo.forge`'s type; each literal and
reader edited.
