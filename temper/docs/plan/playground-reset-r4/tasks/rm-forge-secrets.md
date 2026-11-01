+++
id = "rm-forge-secrets"
part = "forge-removal"
kind = "feature"
needs = ["fr-forge-added", "rm-box-remove"]
files = [
  "temper/crates/temper-service/src/forge/table.rs",
  "temper/crates/temper-service/src/forge.rs",
  "temper/crates/temper-service/src/forge/records.rs",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/setup/public.rs",
  "temper/crates/temper-service/src/tests/forge_secrets.rs",
]
effort = "high"
minutes = 45
+++

## Goal

The service's half of a forge's removal, at the connector's
`Request::ForgeRemoved` (draft 4: "its secrets at once, the admin's
token, the bot's tokens, the webhooks' secret and the App's key among
them"): every credential record of the forge is erased from jig's
secrets store, the setup layer's work for it ends, and every service
path treats the forge as gone.

Acceptance:

- `forge::table::removed` (from `fr-forge-added`) also erases each
  credential record of the forge: every `temper_protocol_records::Item`
  under `records::number(forge, item)`, by
  `Secrets::Erase { owner: SecretOwner::Application, number }` as
  `setup/public.rs`' `erase` does, one at a time through the store's
  secrets door, resumed if the store is busy (the service's existing
  erase pattern); a record already absent counts as erased.
- The setup layer drops the forge's staged plans, runs, public callers,
  pastes, waiting GitHub visits and held installs; each public caller
  waiting on the forge gets its one answer as for a forge not
  connected (`ConnectAnswer::Refused(ForgeRefusal::NotConnected)`, or
  the matching answer of its request); the GitHub token table forgets
  the forge's installation tokens.
- `forge::ended(engine, forge)` is true for a forge the service does not
  hold (as well as the demo's ended forge, until the playground part
  removes the demo): no connect, setup or adoption path reaches skein
  for it.
- A removal cut by a restart before every record is erased finishes at
  the next start: a forge no record of the connector names has its
  credential records swept by the forge records' loader, which today
  only names unconfigured forges (`forge/records.rs`, `Phase::Name`):
  it erases them instead, once the connector's kept forges are told.
- The tests below pass; `temper-service`'s focused tests pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D3).
- The draft, section 4 ("Its records erased").
- `temper/docs/design/protocol/credentials.md` (the records and their
  numbers).
- The merged `fr-forge-added` and `rm-box-remove` reports.

## Owns and may change

New test files are declared from the module they test, at its end
(`#[cfg(test)] #[path = "tests/NAME.rs"] mod tests;`, the path relative to
that module, as `temper-protocol-store/src/records.rs` declares `holds`),
never in a crate's `src/tests.rs`: tasks running at once would all edit it.
Where this spec says "its `mod` line in `tests.rs`", read that.

Owned: the files in the header (`tests/forge_secrets.rs` new), and
`forge/records.rs`'s `Phase::Name` (erase in place of naming, for a
forge the connector does not keep).

Neighbours: `temper-service/src/tests/*`: a test that expects a
`Note::ForgeUnconfigured` for leftover records now expects their
erasure; list each.

Pre-approved options:

- If the loader's sweep erasing unkept forges' records would erase a
  seeded forge's records before the connector told its kept forges at
  the first start, gate the sweep on that tell (`fr-forge-added`'s
  flag) and say so.

## Steps

1. `table::removed`'s erase pass and the setup layer's drop
   (`setup::Layer::drop_forge`, new).
2. `forge::ended`'s widening; `records.rs`' sweep.
3. **Tests**, `tests/forge_secrets.rs`:
   `a_removed_forge_has_every_credential_record_erased`,
   `a_removed_forges_public_connect_is_answered_not_connected`,
   `a_removed_forge_is_never_connected_again`,
   `leftover_records_of_an_unkept_forge_are_erased_at_the_next_start`.
4. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets --
   -D warnings`; `temper-service`'s focused tests; `heavy cargo check
   --tests -p temper -p temper-system-world`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `setup::Layer::drop_forge`, the
erase pass, the widened `forge::ended`, and the sweep's rule.
