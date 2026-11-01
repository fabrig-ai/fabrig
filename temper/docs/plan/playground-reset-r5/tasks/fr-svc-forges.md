+++
id = "fr-svc-forges"
part = "forge-records"
kind = "feature"
needs = ["fr-sk-connection", "fr-sk-setup", "fr-machine"]
files = ["temper/crates/temper-service/src/forge.rs", "temper/crates/temper-service/src/forge/**", "temper/crates/temper-service/src/setup.rs", "temper/crates/temper-service/src/setup/settings.rs", "temper/crates/temper-service/src/setup/lifecycle.rs", "temper/crates/temper-service/src/workers.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/translate.rs", "temper/crates/temper-service/src/values.rs", "temper/crates/temper-service/src/tests/runtime_forges.rs"]
effort = "high"
minutes = 120
+++

## Goal

The service configures a forge at runtime, as it configures the
document's at start: every per-forge table sized from `forge.forges`
instead of the configured count, and one entry, `Engine::keep_forge`,
that adds a parsed forge to each of them (`notes/forge-records.md`,
section 5). The request plumbing that calls it (the root's `ForgeKept`,
the shell's runtime answer) is `fr-svc-route`'s; this task makes the
entry and proves it by calling it directly.

Acceptance:

- **Sized from `forge.forges`:** the forge component's limits
  (`forge::shared`/`forge::small`: `forge_component.forges` goes, its
  value `forge.forges`), the setup host's `Limits.forges`, `Tokens::new`,
  `forge::records::Rows::new`, the workers' forge table
  (`workers.rs`), identities and values (`values.rs`'s credential room),
  and the connector's `connection_limits(forges, ..)` call in
  `engine.rs` all take `settings.limits.forges.forges` (the limit
  `fr-machine` added). The document's forges still configure at start
  and count against it (a document naming more than `forge.forges` is
  refused at start, naming the limit).
- `pub(crate) fn Engine::keep_forge(&mut self, parsed: machine::Parsed,
  runtime: ForgeRuntime)`: adds the forge to the forge component
  (`skein_forge_connection::Request::Add`, its access from rows already
  loaded for that number, else none), the setup host
  (`skein_forge_setup::host::Request::Add`, built as `setup::settings`
  builds a configured forge's), tokens, the workers' table and the
  translation's lookups (`translate.rs`, `family.rs`'s by-number reads go
  through the same table, so a kept forge is found as a configured one).
  A second call for a configured number replaces nothing but its setup
  entry's first intent (`setup::lifecycle`'s `Intent`, from the record's
  `first`).
- **Credential rows of numbers not yet kept** (`forge/records.rs`
  `Loader`): held by number until `Engine::restored_forges()` (a new
  call `fr-svc-route` makes when the root reports the connector's
  `Restored`), then reported as today's unconfigured rows; a kept forge
  takes its held rows at `keep_forge`. Today's start-time path is
  unchanged for document forges.
- Tests (a new `src/tests/runtime_forges.rs`, on `src/tests/fixtures.rs`
  and `machine::record_of`):
  `a_forgejo_kept_at_runtime_connects_from_its_records_and_serves_calls`,
  `a_github_forge_kept_at_runtime_mints_from_its_kept_app`,
  `an_owned_forge_kept_at_runtime_takes_its_setup_entry_and_local_starts`,
  `credential_rows_wait_for_their_forge_then_unknown_ones_are_reported`,
  `a_forge_past_forge_forges_is_refused_at_start`,
  `a_kept_forges_repositories_reach_the_worker_with_its_remote_and_identity`.
  Every existing test of the crate passes unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md`,
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 0, 3 and 5).
- `temper/docs/design/protocol/credentials.md`, 2.1 (records keyed by
  forge) and 5; `temper/docs/design/protocol/setup.md`, 6 (the setup
  host).
- Surveys `temper/docs/plan/playground-reset-r5/notes/surveys/s1a-document-forges.md`,
  section 1, "Configuration count still controls" (every table to size),
  and `s1-config-and-connector.md`, section 3, "`temper-service`
  production readers".
- `tasks/fr-sk-connection.md` and `tasks/fr-sk-setup.md` for the skein
  requests you call.

## Owns and may change

- Owns: `temper-service`'s `src/forge.rs`, `src/forge/`, `src/setup.rs`,
  `src/setup/settings.rs`, `src/setup/lifecycle.rs` (the intent from a
  record only), `src/workers.rs`, `src/engine.rs`, `src/system.rs`
  (`prepared`'s sizing), `src/translate.rs`, `src/values.rs`, the new test
  file and its `mod` line.
- May change: `src/family.rs` and `src/assignment.rs` by-number lookups
  only (to read the shared table), `src/startup/checks.rs`'s
  `record_writes`/`records`/`root` sizing only (from `forge.forges`).
- Options: if a table cannot be sized from `forge.forges` without
  changing a skein component's limits beyond `fr-sk-*`'s, size it so and
  ask before changing skein. If `values.rs`'s credential room is priced
  per configured identity at start, price `forge.forges` identities.

## Steps

1. Sizing from `forge.forges`.
2. `keep_forge` across the tables; the loader's held rows.
3. Tests. Narrow: `heavy cargo nextest run -p temper-service --lib -E
   'test(/runtime_forges/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets -- -D
   warnings`; focused tests of `temper-service`; `heavy cargo check -p
   temper -p temper-one-process-world -p temper-real-loop-world -p
   temper-system-world --tests`. Old-forms from `temper/`. Commit and
   push.

## Report

As `notes/common.md` says; each table you resized and from what.
