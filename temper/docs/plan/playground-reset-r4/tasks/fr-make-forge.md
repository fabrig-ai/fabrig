+++
id = "fr-make-forge"
part = "forge-records"
kind = "feature"
needs = ["fr-seed"]
files = [
  "temper/crates/temper-engine-domain-forge/src/make.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/make.rs",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
]
effort = "high"
minutes = 40
+++

## Goal

The forge connector's one path that makes a forge (the run's
`designers/ids.md`, `fr-make-forge`; draft 3): the next number from
`Forges`, never reused; the `Forge` row and the new `Forges` row saved
in the caller's decision; the forge in the connector's tables at once;
and `Request::ForgeKept` so the root and the service hold it
(`fr-forge-added`: skein's connection component and setup host). Under
the box, the box link writes its group file at its first ask
(`fr-group-file`), so nothing else is owed here. `SetUp` (`fr-setup`),
`Connect` (`fr-connect`) and the playground's start (`pg-start`) call
it.

Acceptance:

- `make::make(d, env, forge, out) -> Result<u16, Refusal>` with the
  record's `number` and `identity.number` ignored: the number is
  `d.next_forge()`; the identity's number is one past the highest kept
  record's (1 with none); the row is saved
  (`Request::Save { record: Stored::Forge(..) }`), then `Stored::Forges
  { next: number + 1 }`; `forge_records`, `configured`
  (`Configured::from_record`) and `identities` hold it; then
  `Request::MarkedForges` and `Request::ForgeKept`.
- Refused, changing nothing: `Refusal::Full` when `l.forges` records
  are kept (removing ones count: their rows are kept); `Refusal::Busy`
  when a kept record not removing is of the same box kind (an owned
  Forgejo, `SetupMode::Owned`; the playground's, `playground` true):
  one owned and one playground forge at a time (the box runs one forge
  per kind: `notes/progress-box.md`); `Refusal::Invalid` when
  `Forge::fits` refuses it; `Refusal::Exhausted` at `next == u16::MAX`.
- A restart restores the made forge from its rows as any record
  (`fr-seed`), and the next make takes the number after it.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 2:
  `Forges { next }` "never lowered; absent means 1").
- `temper/docs/plan/playground-reset-r4/notes/progress-box.md` (one
  forge per kind).
- The merged `store-families`, `fr-surface` (`make::State`,
  `Request::ForgeKept`) and `fr-seed` reports.

## Owns and may change

Owned: `make.rs` (its stub from `fr-surface`), the new `tests/make.rs`
and its `mod` line.

Neighbours: none. `make::State` stays empty unless a field is needed;
`Domain`'s field for it is already there.

Pre-approved options:

- `Refusal` is `pub(crate)` unless a caller in another crate needs it
  (none does: the asks map it to their own answers).

## Steps

1. `make.rs` (module comment cites the draft, 3, and contracts 2):
   `make`, `Refusal { Full, Busy, Invalid, Exhausted }`, and
   `pub(crate) fn kind_busy(d: &Domain, forge: &crate::Forge) -> bool`
   for the asks to test before they decide.
2. **Tests**, `tests/make.rs`: `a_first_make_is_forge_one`,
   `numbers_follow_the_highest_ever_kept` (seeded 1-3, make: 4; a
   restored `Forges { next: 7 }`: 7), `a_made_forge_is_configured_and_told`
   (`configured(n)`, `ForgeKept`, both saves in order),
   `identity_numbers_follow_the_highest_kept`,
   `a_second_owned_or_playground_forge_is_busy`,
   `full_and_invalid_records_change_nothing`,
   `a_made_forge_restores_as_any_record` (feed the saved rows to a new
   domain's restore).
   Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib
   -E 'test(make::)'`.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge
   --all-targets -- -D warnings`; the crate's focused tests. Commit and
   push.

## Report

fab's sections, and `## Interfaces`: `make::make`, `Refusal`,
`kind_busy`, with their signatures.
