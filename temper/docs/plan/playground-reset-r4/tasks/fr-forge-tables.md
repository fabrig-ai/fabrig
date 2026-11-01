+++
id = "fr-forge-tables"
part = "forge-records"
kind = "feature"
needs = ["fr-surface", "fr-seed"]
files = [
  "temper/crates/temper-service/src/forge.rs",
  "temper/crates/temper-service/src/forge/table.rs",
  "temper/crates/temper-service/src/forge/records.rs",
  "temper/crates/temper-service/src/forge/tokens.rs",
  "temper/crates/temper-service/src/workers.rs",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/setup/settings.rs",
  "temper/crates/temper-service/src/tests/forge_table.rs",
]
effort = "high"
minutes = 55
+++

## Goal

The service's per-forge state can take a forge and drop one after its
start, and it can build every per-forge setting it holds from a
`Forge` record. Nothing calls the new entry points yet but tests
(`fr-forge-added` does, on the connector's `ForgeKept`); the service
still starts from its document's forges, unchanged.

Acceptance:

- `forge::table` (the module `fr-surface` made) has the conversions:
  `settings_forge(record, files) -> Result<settings::Forge, NotKept>`
  and `identity(record, files) -> Result<settings::Identity, NotKept>`
  (the file form's secrets read from `settings::Files` by the names the
  record keeps, as `startup::application::resolve` reads them; a
  records-only forge's as `resolve` gives them), `endpoint(record,
  shell) -> Result<skein_forge_connection::settings::Forge, NotKept>`
  (resolving the address and the TLS trust through the shell's
  functions, step 2; a host that does not resolve is held as
  `forge::HELD_ADDRESS`, as the binary's `forges` does today) and
  `host_forge(engine, record, endpoint) -> Result<host::Forge, NotKept>`
  (the body of `setup/settings.rs`'s per-forge loop in `fixed`, for one
  forge).
- For each forge of the template's document (`temper-init`'s template),
  the conversions of its seed record (`fr-seed`'s `startup::seed`) equal
  what the service builds from the document today.
- These hold forges by number and can take one more and drop one:
  `forge::Layer.held`, `forge::records::Rows`'s configured list,
  `forge::tokens::Tokens.forges`, `workers::Workers.forges` (a `List` of
  capacity `forge.forges`), `setup::Layer`'s pending host settings and
  its per-forge maps (`served`, `unserved`, `file_api`, the lifecycle's
  per-forge entries): each gains `add`/`remove` (or `insert`/`drop`) for
  one forge, used by nothing but tests yet.
- Every existing service test passes.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D1).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s6-skein-forges.md`,
  section 5 ("Production construction chain", "Temper layers that must
  participate").
- `temper/docs/plan/playground-reset-r4/notes/surveys/s1-forges-document.md`,
  sections 1 and 3 ("Startup wiring").
- The code: `temper-service/src/forge.rs` (`settings`, `Layer::new`,
  `config`), `setup/settings.rs` (`fixed`, `forgejo`), `forge/records.rs`
  (`Rows`, `Loader`), `forge/tokens.rs` (`Tokens::new`), `workers.rs`
  (`Workers::configured`), `temper/crates/temper/src/lib.rs` (`forges`,
  `address`, `trust`: the shell's resolution today).

## Owns and may change

Owned: the files in the header (`tests/forge_table.rs` new, its `mod`
line in `temper-service/src/tests.rs`).

Neighbours, each for this task's change only:

- `temper-service/src/lib.rs` and `system.rs`'s `Service::new` /
  `prepared` signatures: the shell's functions (step 2) passed in and
  kept on the engine; `temper/crates/temper/src/lib.rs`: passing its
  `address` and `trust` functions there. Every other caller of those
  constructors (service tests, `temper/tests/**`): a shell value from
  a test helper (`forge::table::Shell::refusing()` or one resolving
  loopback literals only).
- Readers of `engine.workers.forges` (`engine.rs`, `route.rs`,
  `setup/lifecycle.rs`, `setup/routes.rs`, `clients*`): the iteration's
  form only, if the `List` needs it.

Pre-approved options:

- `NotKept` is a small enum naming why a record could not be held
  (`Unresolved`, `Trust`, `File`, `Bound`); add the variants you need.
- If the setup lifecycle's per-forge entries are made in more places
  than `Lifecycle`'s constructor, give it one `add_forge`/`drop_forge`
  pair and leave the rest.

## Steps

1. Conversions in `forge/table.rs` (module comment cites the draft, 3,
   and D1), signatures as in the Goal, each a total function of its
   inputs.
2. The shell's functions:

   ```rust
   /// What the shell resolves for the service, at its start and for a
   /// forge kept later (the binary's `address` and `trust`).
   #[derive(Clone, Copy)]
   pub struct Shell {
       pub address: fn(&jig_protocol_config::Address) -> Result<core::net::SocketAddr, Box<[u8]>>,
       pub trust: fn(Option<&[u8]>) -> Result<skein_tls::Config, Box<[u8]>>,
   }
   ```

   (the binary's error strings as bytes). Held on the engine for
   `fr-forge-added`.
3. The tables' `add`/`remove`, each keeping its capacity at
   `forge.forges` and refusing past it.
4. **Tests**, `temper-service/src/tests/forge_table.rs`:
   `the_template_records_convert_as_the_document_does` (per forge:
   `settings_forge`, `identity`, `endpoint` with a loopback shell,
   `host_forge`, each equal to today's build from the document),
   `a_file_form_record_reads_its_files`,
   `a_records_only_record_needs_no_file`,
   `an_unresolved_host_is_held`,
   `each_table_takes_one_more_forge_and_drops_it`.
5. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper
   --all-targets -- -D warnings`; `temper-service`'s focused tests;
   `heavy cargo check --tests` for `temper`, `temper-system-world`,
   `temper-one-process-world`, `temper-conformance-world`,
   `temper-real-loop-world`, `temper-client-world`. Commit and push.

## Report

fab's sections, and `## Interfaces`: the conversions, `Shell`, the
service constructors' new parameter, and each table's add/remove.
