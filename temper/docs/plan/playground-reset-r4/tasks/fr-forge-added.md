+++
id = "fr-forge-added"
part = "forge-records"
kind = "feature"
needs = ["fr-forge-tables", "sk-conn-add", "sk-conn-remove", "sk-setup-forges"]
files = [
  "temper/crates/temper-service/src/forge/table.rs",
  "temper/crates/temper-service/src/forge.rs",
  "temper/crates/temper-service/src/forge/records.rs",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/engine.rs",
  "temper/crates/temper-service/src/tests/forge_added.rs",
  "temper/crates/temper-protocol-records/src/forge.rs",
]
effort = "high"
minutes = 60
+++

## Goal

The service holds exactly the forges the connector keeps: it builds
skein's connection component, its setup host and its own per-forge
tables with no forge, and takes each forge, seeded or made, from the
connector's `Request::ForgeKept` (contracts, D1: "the engine, the box
and the client read forges from records alone"), and drops one on
`Request::ForgeRemoved`, at once.

Acceptance:

- `forge::table::kept(engine, record)`: a forge not held is converted
  (`fr-forge-tables`) and added to the connection component
  (`Component::add`), the setup host (its pending settings before the
  host begins, `Host::add` after), and every per-forge table; once the
  forge records' loader is past its load, it is connected from its
  records as the loader's `Connect` phase connects one forge. A forge
  already held with the same record changes nothing; a conversion or an
  add refused leaves it unheld, with a `Note::ForgeNotConnected` naming
  why (a new `NotConnected` variant if none fits).
- The service keeps each held record whole: `forge::table::record(engine:
  &Engine, forge: u16) -> Option<&temper_engine_domain_forge::Forge>`
  (the box link writes its group file from it: `fr-group-file`).
- `forge::table::removed(engine, forge)`: `Component::remove`,
  `Host::remove`, and every table's `remove`; its held address and
  setup state gone. Its secrets are not erased here (`rm-remove-forge`).
- The service's construction builds the component and host with no
  forge; the document's forges are checked at the start as today (the
  start refuses what it refuses now, at the same paths), but only the
  connector's `ForgeKept` makes the service hold them.
- A start of the template's document holds forges 1, 2, 3 after the
  connector's restore, connects them as today, and every service and
  system-world test passes; a document naming no forge (`fr-seed`)
  starts and serves its clients.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D1).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s6-skein-forges.md`,
  section 5.
- The merged reports of `fr-forge-tables` (conversions, tables,
  `Shell`), `sk-conn-add`, `sk-conn-remove`, `sk-setup-forges`
  (`Component::add`/`remove`, `Host::add`/`remove`), `fr-seed`
  (`ForgeKept` at each start).
- `temper/docs/design/domain/root.md`, 9.1 (the start: restore, the
  durable start mark, then the forge records' load).

## Owns and may change

Owned: the files in the header.

Neighbours: `temper-service/src/tests/*` and `temper/tests/system/**`,
`temper/tests/one-process/**`: a test that read the forge component's
or the setup host's forges right after construction, before the
connector's restore, now drives the start first; list each.

Pre-approved options:

- If the forge records' loader can reach its `Connect` phase before the
  connector's `ForgeKept` requests of the start, the loader waits for
  them: a flag set when the root's restart passes the connector's
  `Restored` stage (the `RestartDone { stage: Restored }` the root
  routes), default; say where it is set.
- The start-time check of the document's forges stays where it is
  (`forge::prepare`, `setup::prepare`) over the seed's endpoints; the
  binary keeps resolving them at its start for that check.
- If the webhooks' listener settings (`hooks::Webhooks`) are needed by
  `Component::new` with no forge, pass them as today.

## Steps

1. `table::kept` and `table::removed` bodies (`fr-surface`'s stubs), and
   `table::box_remove` untouched (`rm-box-remove`'s).
2. Construction: `system.rs` builds `forge::Layer` and `setup::Layer`
   from settings with no forge (capacity `forge.forges`); the tables
   start empty.
3. The loader's per-forge connect as a function `records::connect_one`
   callable from `kept`.
4. **Tests**, `temper-service/src/tests/forge_added.rs`, with the
   service fixtures (`tests/fixtures.rs`' started service):
   `the_template_holds_its_forges_only_once_kept`,
   `a_forge_kept_after_the_start_connects_from_its_records` (a
   records-only Forgejo record kept at runtime with its records in the
   secrets store: the component gets its `Connect`),
   `a_kept_github_forge_takes_its_installations`,
   `a_removed_forge_leaves_the_component_the_host_and_every_table`,
   `an_unresolvable_record_is_noted_and_not_held`,
   `a_document_naming_no_forge_serves_its_clients`.
5. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets --
   -D warnings`; `temper-service`'s focused tests; this changes when the
   service holds its forges, so also `temper-system-world`'s and
   `temper-one-process-world`'s focused tests; `heavy cargo check
   --tests -p temper -p temper-conformance-world -p
   temper-real-loop-world -p temper-client-world`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `table::kept`, `table::removed`,
`table::record`,
`records::connect_one`, any `NotConnected` variant, and where the
loader waits (first option).

## Answers

2026-11-01. May this task change `temper-protocol-records/src/forge.rs`
for a new `NotConnected` variant? Yes (option A): the file is added to
the header's `files`, for the variant and its `Display` arm only, and
any match the compiler then requires is repaired where it stands. Name
it `NotConnected::NotHeld { reason: Box<[u8]> }`, not `NotKept`: the
connector kept the forge, the service could not hold it. Its `Display`
is `the service could not hold it: ` and the reason; the reason is
ASCII the service writes from skein's refusal, naming the layer that
refused and why (`Component::add`'s `Added::Closed` or
`Added::Unusable`, `Host::add`'s `Unusable`), with counts and limits at
most: no host, token or record bytes. Like `Unreachable`, it is an
operator diagnostic, never a wire or stored value; a conversion refused
keeps the existing variants (`Unreachable`, `Unreadable`, ...).
