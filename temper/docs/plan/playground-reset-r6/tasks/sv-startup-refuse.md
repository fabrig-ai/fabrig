+++
id = "sv-startup-refuse"
part = "fs"
kind = "migration"
needs = ["sv-service-fixtures", "sv-world-one-process", "sv-world-real-loop", "sv-world-client-browser", "bx-init", "bx-fixtures", "bx-walks"]
files = ["temper/crates/temper-service/**", "temper/crates/temper/src/lib.rs", "temper/crates/temper/src/setup.rs"]
effort = "high"
minutes = 120
+++

## Goal

The document's forges go (`notes/contracts.md`, T4; `notes/sv-progress.md`, request 1 (d)):
`application.forges`, `application.identities` and `application.setup` are refused,
naming the member; `application.machine` is required and `application.demo` takes T4's
shape alone; and every path in serve that the document's forges fed is deleted, so serve's
forges are its registry's alone, from records.

Acceptance:
- Startup refuses a document naming `application.forges`, `application.identities` or
  `application.setup` (one problem each, at the member's path, `What::Jig(config::What::
  Unknown)` or the startup's word for an unknown member: as `read::finish` names one), and
  one with no `application.machine` (`Required` at its path); `application.demo` of
  today's shape (a number `forge` or `project`) is refused `Form` at the member.
- Deleted from temper-service (with their tests, or those tests turned into refusal tests):
  `startup::application::{Forge, Identity, User, Transport, forge, identity, references,
  resolve_forge, resolve_identity, asked_owned, files' forge and identity files,
  git_programs, missing_asked_files, configured}` and `Members.forges`/`identities`/
  `asked_owned`; `settings::Engine.{forges, identities, setup, records_only}`;
  `settings::Setup`; the document seeding of `sv-registry` (the registry starts empty;
  `Loader::new` seeds nothing) and the "number the document configures" branches of
  `sv-registry`, `sv-registry-setup` and `sv-playground`; `forge::{settings, prepare,
  file_access, indexed}`'s document halves and `ForgeRuntime`'s start-time list
  (`Service::engine`'s `forges` parameter goes); `setup/settings.rs`'s start-time forges
  in `fixed`, `local_starts` and the file-era fake adoption and `file_api` tokens;
  `values.rs::fill`'s identities (the table starts empty); `Tokens::new`'s document forges;
  the root `Config`'s transitional `seed` filling, if `fe-records` added one (the field
  itself is fe's: leave it empty and name it in the report).
- The start-time checks sized by the document's forges take the machine's templates:
  `startup/checks.rs::{credentials, webhooks, longest_remote, channel, identities,
  record_writes}`; `system.rs::note_bound`. The landing's gates (`landing.rs`) no longer
  check a gate's forge against configured forges (a gate naming a forge not kept applies to
  no repository) nor GitHub's `turn` refusal (report it as a check lost); the sign-in's
  forge rows (`sign_in/forges.rs::check`) join `application.machine.owned`'s public origin
  instead of a configured Forgejo; the bundle's forge metas (`clients/bundle.rs`) are built
  from `application.machine.owned` the same way.
- The binary: `temper/crates/temper/src/lib.rs`'s `forges()` and its call in
  `prepare_with_limits` go (call sites only, as the service's signatures change);
  `temper/crates/temper/src/setup.rs` loses its reads of `Named.forges`/`identities`
  (compiler-forced; bx-doc's reading of the machine stays).
- `python3 -I -B scripts/old_forms.py --write` lowers the counts the deleted file-era
  forms held (temper's `scripts/old-forms.txt`), never raising one.
- Every test of temper-service, temper-one-process-world, temper-real-loop-world,
  temper-client-world, temper-system-world and temper (default profile) passes.

## Inputs

- `temper/docs/design/protocol/configuration.md`, sections 3 and 10.
- Draft `temper/docs/design/draft/playground-reset.md`, sections 3 and 10.
- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md`, T4.
- `temper/docs/plan/playground-reset-r6/notes/sv-progress.md`, requests 1 and 5.
- The `## Left` lists of `sv-registry`, `sv-registry-setup`, `sv-resolve`,
  `sv-startup-machine`, `sv-service-fixtures` and the worlds' tasks.

## Owns and may change

- Owns: `temper/crates/temper-service/`.
- `temper/crates/temper/src/lib.rs` and `src/setup.rs`: the call sites the service's
  changed signatures force, alone.
- Any other temper file: compiler-forced edits only (a removed field or type), each listed.
- Pre-approved options:
  - A check lost with the document (landing's GitHub `turn` refusal, a configured forge's
    reference) that the engine can make at runtime: leave it unmade and list it for the
    lead; do not move it into the engine here.
  - If a world or process fixture still names a removed member (a task before this one
    left it), move it here only when the edit is the member's removal; else stop and ask.

## Steps

1. The refusals; then the deletions, compiling as you go (`heavy cargo check -p
   temper-service --tests`).
2. Tests: the startup's refusal tests (`a_document_naming_forges_is_refused`,
   `a_document_naming_identities_is_refused`, `a_document_naming_setup_is_refused`,
   `a_document_without_machine_is_refused`, `a_demo_of_todays_shape_is_refused`) in
   `src/tests/startup_machine.rs`; `heavy cargo nextest run -p temper-service --lib -E
   'test(/tests::startup_machine::/)'`.
3. Whole runs, one at a time: `-p temper-service`, `-p temper-one-process-world`, `-p
   temper-real-loop-world`, `-p temper-client-world`, `-p temper-system-world`, `-p
   temper`; `heavy cargo check --workspace --all-targets`.
4. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper --all-targets`; old forms
   with `--write`. Commit and push.

## Report

- The checks lost with the document, each with one line.
- The old-forms counts lowered.
