+++
id = "fr-machine"
part = "forge-records"
kind = "feature"
needs = ["fr-family", "fr-wire"]
files = ["temper/crates/temper-service/src/machine.rs", "temper/crates/temper-service/src/machine/**", "temper/crates/temper-service/src/lib.rs", "temper/crates/temper-service/src/startup/application.rs", "temper/crates/temper-service/src/startup/application/**", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/limits/**", "temper/crates/temper-service/src/settings.rs", "temper/crates/temper-service/src/tests/machine.rs", "temper/crates/temper-init/template/temper.json", "temper/crates/temper-init/src/tests/**"]
effort = "high"
minutes = 110
+++

## Goal

The service's pure half of forge records: the document's machine member
(`application.machine`), the fills that turn a person's choice into a
`Template`, the parse of a record's entries into the service's typed
forge, its inverse for fixtures, and the `forge.forges` limit
(`notes/forge-records.md`, sections 1 and 3). No runtime path calls them
yet (`fr-svc-forges`, `fr-svc-route` do). The document may now name no
forge.

Acceptance:

- `application.machine` parsed as the note's section 3 has it, into
  `startup::application::Members.machine: Machine` and
  `settings::Engine.machine: Machine` (each member optional; absent
  `machine` is all `null` but `bot`'s default `temper` / `Temper` /
  `temper@localhost`); `temper init`'s template gains the member with the
  `$owned`, `$forge`, `$login`, `$email` placeholders the template already
  substitutes (`temper-init/src/template.rs`), its forges, identities and
  setup unchanged; `temper-init`'s template tests pass (update the
  expected document in `src/tests/documents.rs` only for the new member).
- The parser accepts a document whose `application.forges`,
  `identities` and `setup` are empty or absent (with `demo` then
  `null`): today's `parse` refuses an empty forge list
  (`startup/application.rs`, about line 812); it no longer does. A
  document that names forges reads as today.
- `temper_service::machine` (new `src/machine.rs`):
  - `pub struct Machine { bot, github, owned: Option<Owned>, fake:
    Option<Fake> }` (fields as the member's; listeners as
    `jig_protocol_config::Address`);
  - `pub fn playground(&Machine) -> Option<Template>` (the fake forge:
    `(Forgejo, Fake)`, `asked: false`, entries from `fake` and `bot`);
  - `pub fn fill(&Machine, &v1::NewForge) -> Result<Template, Problem>`
    for `owned` (`(Forgejo, Owned)`, `asked: true`, the owned details as
    `configured` builds them today: pinned versions,
    `Place::PersonPassword`), `github` (`(GitHub, Connected)`,
    `read_only` the person's) and `forgejo` (`(Forgejo, Managed)`,
    `admin_connect: true`, API, transport and git from the address as the
    note says); refused when the machine lacks the member, the address
    fails today's `forge_endpoint`/`git_base` checks, or an entry exceeds
    its bound (`ENTRY_*_BYTES`);
  - `pub fn parse_record(&ForgeRecord, &Machine, &config::Directories) ->
    Result<Parsed, Problems>`, `Parsed { forge: settings::Forge, identity:
    settings::Identity, setup: Option<settings::Setup> }`, through today's
    per-entry parsers (`forge`, `identity`, `temper_forge_setup`'s entry)
    given the record's number (identity number = forge number; setup
    directory `forge-N`), and today's resolution (`resolve_forge`) but
    reading no file: `api_token`, `webhook_secret` none, the identity's
    token empty (records-only);
  - `pub fn record_of(&settings::Forge, &settings::Identity,
    Option<&settings::Setup>) -> ForgeRecord`, the inverse, for test
    fixtures (doc comment saying so): `parse_record(record_of(x))` gives
    `x` back but for file-read secrets.
  Refactor the per-entry parsers to read an entry without its number
  members when given one; the document's list parse calls them as before.
- `forge.forges` (`Forges.forges` in `src/limits.rs`, its name in
  `src/limits/names.rs`, small profile 4, at most 16), checked in
  `limits::engine` into the connector's `Limits.forges`.
- Tests (a new `src/tests/machine.rs`): `the_templates_machine_member_reads_with_its_listeners`,
  `an_absent_machine_member_offers_no_owned_forge_and_no_playground`,
  `a_document_without_forges_reads`,
  `owned_github_and_forgejo_fills_name_their_choice_by_kind_and_mode`,
  `a_forgejo_address_with_plaintext_off_loopback_is_refused`,
  `a_record_parses_into_the_typed_forge_its_entries_describe`,
  `record_of_round_trips_through_parse_record`,
  `forge_forges_overrides_and_bounds`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 5),
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 1, 3 and 6).
- `temper/docs/design/protocol/configuration.md`, sections 3 (forges,
  identities) and 7 (limits); `temper/docs/design/protocol/setup.md`, 8
  (setup entries) and 8.3 (the default document).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s1a-document-forges.md`,
  sections 1 and 2 (every parser, default and check you reuse).

## Owns and may change

- Owns: `temper-service`'s new `src/machine.rs` (and `src/machine/` if
  split), `src/lib.rs` (the `pub mod`), `src/startup/application.rs` and
  `src/startup/application/`, `src/limits.rs`, `src/limits/`,
  `src/settings.rs` (the `machine` field), the new `src/tests/machine.rs`
  and its `mod` line; `temper-init/template/temper.json` and
  `temper-init/src/tests/`.
- May change, forced only: `settings::Engine` literals (`machine:
  Machine::default()` or the parsed one) in temper-service's tests and in
  the worlds that build one (`rg 'settings::Engine \{'`).
- Options: if `temper_forge_setup`'s entry parser is private and making it
  `pub` is the smallest way, make it `pub` with a doc comment; if
  `Machine` wants a `Default`, derive it.

## Steps

1. The member and its parse; the relaxed list checks; the template.
2. `machine` functions; the per-entry parsers given a number.
3. `forge.forges`.
4. Tests. Narrow: `heavy cargo nextest run -p temper-service --lib -E
   'test(/machine/)'`; `heavy cargo nextest run -p temper-init --lib`.
5. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper-init
   --all-targets -- -D warnings`; focused tests of `temper-service`,
   `temper-init` and `temper-forge-setup`; `heavy cargo check -p temper
   --tests`. Old-forms from `temper/`. Commit and push.

## Report

As `notes/common.md` says; the template's new member as written, and any
check of today's parser you could not reuse.
