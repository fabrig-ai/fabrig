+++
id = "sv-startup-machine"
part = "fs"
kind = "feature"
needs = ["sv-family", "fe-surface", "bx-steps"]
files = ["temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/startup/**", "temper/crates/temper-service/src/settings.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/limits/**", "temper/crates/temper-service/src/tests/**"]
effort = "high"
minutes = 110
+++

## Goal

temper-service's startup accepts T4's document beside today's (`notes/contracts.md`, T4;
`notes/sv-progress.md`, Contract requests, 1 (b)): `application.machine`, the new shape of
`application.demo`, `forge.forges`, and a document that names no forge, identity or setup
entry. It hands the root `Config.machine` (fe-surface's field). The old members still read
as today; they are refused in `sv-startup-refuse`.

Acceptance:
- `application.forges`, `application.identities` and `application.setup` are optional:
  absent reads as an empty list; the refusal of zero configured forges
  (`startup/application.rs`, `parse`: `forge_ats.is_some() && forge_count == 0`) goes, and
  the identities' count must equal the forges' only when both are present. A document with
  none of them starts: serve with no forge, the engine with no configured forge.
- `application.machine`, optional in this task (absent: `forge::Machine::bare()`):
  `{"owned": null | <entry body>, "github": {"api", "transport", "git", "web"},
  "git_program": null | <data path>}`, read strictly (unknown members refused):
  - `owned`: an entry body read with `temper_forge_setup::setup::Entry::body` (bx-steps)
    at `application.machine.owned`, forge 0; it must be mode owned (else refused `Form`
    naming `application.machine.owned.mode`). Its record: `kind: Forgejo`, `mode: Owned`,
    `api`: the owned `listen` socket as host text (the IP's text) and port, `Plaintext`;
    `git`: `http://` + the `listen` socket's text (as `temper init` writes `$owned_git`);
    `web: None`; `bot`: the body's bot login and email, `name` `b"Temper"`;
    `setup: Some(<the body's JSON text>)`; `number: 0`, `state: Kept`, `first: None`,
    `playground: false`, `read_only: false`, `git_program: None`.
  - `github`: each member null for github.com's, read as today's `forge()` reads a GitHub
    forge's `api`, `transport`, `git` and `web` (their null defaults and checks); its
    record `ForgeRecord::github_com()` with the members given (`api` as host text and port:
    a socket's IP text, or a host name; `transport` `Tls { server_name, trust }` or
    `Plaintext`).
  - `git_program`: null, or a path resolved under the data directory (as setup's
    `fake.git` is: absolute after resolution, no NUL, at most 4096 bytes), the git program
    of every forge the connector makes but the owned one and the playground's: it becomes
    `Machine.github.git_program` (`notes/contracts.md`, T4); the owned and playground
    records keep their own (`None` for owned, the fake's `git` for the playground).
- `application.demo` takes both shapes: today's (`forge` a number, `project` a number) as
  today; T4's: `forge` a fake entry body read with `Entry::body` at
  `application.demo.forge` (forge 0; mode fake, else refused `Form`), `project`
  `{"name": text 1..=64 bytes, "policy": u32 from 1}`. For T4's shape the playground's
  record is `kind: Forgejo`, `mode: Fake`, `playground: true`, `api` the fake `listen`,
  `Plaintext`, `git` `http://` + the listen text, `git_program` the fake's resolved `git`,
  `bot` as for owned, `setup: Some(<body text>)`, number 0, `Kept`; and `settings::Demo`
  holds `project: 0`, `forge: 0` (the playground's start makes both), `owner`, `name`,
  `landing` from the fake's first repository, `cues`, `key_file`, `provider` as today, and
  new members `project_name: Box<str>` and `project_policy: u32` (today's shape: empty and
  0). `Demo::domain` fills fe-surface's `DemoConfig::{project_name, project_policy}` from
  them, and for T4's shape takes the adoption's host from the playground record's `git`.
- `settings::Engine` gains `machine: temper_engine_domain_forge::Machine` (`owned`,
  `github`, `playground` as above; `playground` from T4's demo shape, `None` for today's
  shape or no demo); `system.rs` sets `root::Config.machine` from it.
  `startup::Named` gains `machine: Machine` too (bx-doc may read it).
- `forge.forges`: the override name `forge.forges` (`limits/names.rs`) sets
  `limits.forge.forges` (fe-surface's field, 4 in `FORGE`); at start,
  `forge_component.forges` below `forge.forges` is refused naming both
  (`application.limits.overrides.forge_component.forges` and `...forge.forges`), as the
  existing paired limits are.
- Nothing new is sized by the document's forges; the two counts that are
  (`system.rs::note_bound`, `startup/checks.rs::record_writes`' Origin rows) take at least
  one Origin row for each machine template (owned, GitHub, playground) beside the
  document's forges, so a document with none still sizes them.
- Every existing temper-service, temper-one-process-world and temper (binary) test passes:
  today's documents read as before.

## Inputs

- `temper/docs/design/protocol/configuration.md`, sections 3 and 10 (the application's
  members); `temper/docs/design/protocol/setup.md`, 8 (the setup entry) and 8.4 (the demo).
- Draft `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T1 and T4.
- `temper/docs/plan/playground-reset-r6/notes/fe-surface.md`, 1 (`ForgeRecord`, `Machine`,
  `ForgeRecord::github_com`, `Machine::bare`, `Limits::forges`, `Config.machine`,
  `DemoConfig`'s new fields).
- `temper/docs/plan/playground-reset-r6/tasks/bx-steps.md` (`Entry::body`).

## Owns and may change

- Owns: `temper/crates/temper-service/src/startup.rs`, `src/startup/` (a new module
  `src/startup/application/machine.rs` for the machine's reader), `src/settings.rs`,
  `src/system.rs` (the `Config.machine` line and `note_bound`), `src/limits.rs` and
  `src/limits/names.rs`, and temper-service's tests.
- Fixtures building `settings::Engine` or `settings::Demo` literally gain the new fields
  (compiler-forced, any temper file: rg `settings::Engine {`, `settings::Demo {`).
- Not the binary, not temper-fake (bx-doc reads the binary's side of the new document).
- Pre-approved options:
  - A body's JSON text: if skein-json's `Document` keeps no source span for a value,
    re-emit the value at its position as compact JSON (a function `fn value_text(document:
    &Document, at: u32) -> Option<Box<[u8]>>` in the machine module walking the tokens with
    `skein_json::writer`), bounded by `temper_forge_setup::setup::BODY_BYTES` (a longer body
    refused `Bound` at its path). Default as said.
  - The bot's display name: `b"Temper"`, as `temper init`'s identities name it today. If a
    test of the worlds reads the author of a commit on a machine forge, keep it and say so.
  - An owned body with a `public` origin: `git` stays `http://` + listen in this task;
    say in the report whether `public` should make it.

## Steps

1. `src/startup/application/machine.rs`: `pub(super) fn parse(document, at: Option<u32>,
   limits: &skein_forge_setup::limits::Limits, directories, problems) -> Option<Machine>`
   and the record builders `owned(entry, text)`, `github(...)`, `playground(entry, text)`.
2. `src/startup/application.rs::parse`: the optional lists; `machine` read; `Members`
   gains `machine: Machine`; `demo::parse` gets T4's shape (both shapes in one function,
   by the kind of `forge`'s token).
3. `src/startup.rs`: `Named.machine`; `settings::Engine.machine` filled where the engine
   settings are built (`checked_start_with`).
4. `src/system.rs`: `Config { machine: settings.machine.clone(), .. }`; `note_bound`.
5. `src/limits/names.rs`: `b"forge.forges"`; the pair check where the existing paired
   limits are checked (rg `forge_component.connections` in `startup/checks.rs`).
6. Tests in `src/tests/startup_machine.rs` (registered in `src/tests.rs`), each through the
   startup entry the existing startup tests use (`tests/startup.rs`'s helpers):
   - `a_document_with_no_forge_identity_or_setup_starts`
   - `the_machines_owned_body_becomes_an_owned_record_template`
   - `a_machine_owned_body_of_another_mode_is_refused_at_its_mode`
   - `the_machines_github_nulls_are_github_coms`
   - `the_machines_github_members_replace_the_defaults`
   - `the_machines_git_program_is_the_github_templates_resolved_under_data`
   - `a_machine_member_unknown_is_refused`
   - `a_demo_of_t4s_shape_reads_its_fake_body_and_project`
   - `a_demo_project_name_past_64_bytes_or_policy_0_is_refused`
   - `todays_demo_shape_reads_as_before`
   - `forge_forges_is_an_override_and_must_fit_the_component`
   - `the_root_config_carries_the_machine`
7. Narrow runs, one at a time:
   `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::startup_machine::/)'`
   (12 tests). Then `heavy cargo nextest run -p temper-service`; `heavy cargo nextest run -p
   temper-one-process-world`; `heavy cargo nextest run -p temper --lib` and
   `heavy cargo nextest run -p temper --test suite -E 'test(/startup/)'` (the binary reads
   the startup's checks); `heavy cargo check --workspace --tests`.
8. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets`; old forms (`python3 -I
   -B scripts/old_forms.py` in `temper/`). Commit and push.

## Report

- The members and shapes landed, exactly (bx-doc and bx-init read them): `Named.machine`,
  `settings::Demo`'s new members, the refusal paths.
- Whether `public` should make an owned record's `git`.
- Under `## Left`: what still reads the document's forges (for `sv-startup-refuse`).
