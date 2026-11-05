+++
id = "making-family"
part = "making-records"
kind = "feature"
needs = ["copy-surface"]
files = ["temper/crates/temper-engine-domain-forge/src/making.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/restore.rs", "temper/crates/temper-engine-domain-forge/src/reconcile.rs", "temper/crates/temper-engine-domain-forge/src/limits.rs", "temper/crates/temper-engine-domain-forge/src/tests/making.rs", "temper/crates/temper-protocol-store/**", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/limits/names.rs", "temper/crates/temper-service/src/limits/reference.rs", "temper/docs/help/reference/limits.md"]
effort = "high"
minutes = 55
+++

# The store family `Making`

## Goal

temper's store holds a row per repository being made, at byte 35, restored whole at the restart step and
held by the forge connector, bounded by `forge.makings` (contracts T1, T2). Nothing writes a row yet:
`making-decide` does.

Acceptance:

- `RepositoryMaking` and its types, `Stored::Making`, `Key::Making`, `RecordKind::Making`,
  `Limits::makings`, `Domain::making`, `next_making`, `making_due` and `makings` exist as T1 and T5 have
  them; the map is sized by `Limits::makings`.
- `keys::MAKING` is 35, the key and record as T2 has them, `STORE_VERSION` unchanged, generated code and
  goldens from the writer. `ApplicationOutcome`'s two new variants round-trip.
- The keys' property test draws `Key::Making`, so the encoder is held to `Key`'s order; its decoder's
  refusals of trailing bytes and a forge past `u16` are tested.
- In a new `src/tests/making.rs`: `makings_restore_at_the_bound`,
  `a_making_past_the_bound_is_refused_at_restart`, `a_row_making_at_restart_comes_back_waiting_with_its_mark`,
  `next_making_is_the_oldest_waiting`; the record round-trips in
  `temper-protocol-store/src/tests/records.rs`.
- `forge.makings` parses, defaults to 32, is refused at 0 and past 64, and is in the generated
  `temper/docs/help/reference/limits.md`.
- `heavy cargo nextest run -p temper-engine-domain-forge -p temper-protocol-store -p temper-codecs-world
  -p temper-engine-domain -p temper-docs` pass; `heavy cargo check --tests -p temper-service -p temper-fake
  -p temper-engine-forge-world -p temper-engine-protocol-forge-world` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, T1, T2 and T5.
- `temper/docs/design/protocol/store.md`, 2 ("The repositories being made"), 3 and 5.
- `temper/docs/design/domain/forge.md`, 4.5.
- `temper/docs/design/protocol/configuration.md`, 7.

## Owns and may change

- The files of the header. A departure from the implementer's text: `temper/docs/help/reference/limits.md`
  is generated, and is yours; it is read by tests (`.config/gate/read-by-tests.txt`). So is the status
  line: remove "2, 3, 5: family 35 `Making`." from `temper/docs/design/protocol/store.status.md`, and
  "7: `forge.makings`." from `temper/docs/design/protocol/configuration.status.md`.
- Follow `Binding` through every match: `rg 'Stored::Binding|Key::Binding|RecordKind::Binding'`.
  Reader files, each its arm alone: `temper/crates/temper-fake/src/check.rs`,
  `temper/tests/engine/forge/src/world.rs`, `temper/tests/engine/protocol-forge/src/protocol.rs`,
  `temper/crates/temper-engine-domain/src/**`.
- `temper/tests/codecs/tests/suite/fuzzy/store.rs` and `temper/tests/codecs/tests/support/store.rs`: the
  family's arm; `temper-protocol-store/src/tests/fixtures.rs`, a new key among its numbered fixtures.
- `temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs`: `RecordKind::Making` in its tables.
- **The `Limits` literals, the field alone:** `rg 'adoptions:' temper --type rust`.
- `to-begin-store` is on these same lines. Order per T1. If `Dismissed` is on your base, `Making` follows
  it; if not, byte 35 stands, and the bytes in between stay unused, as 30 to 33 are.
- Pre-approved:
  - If `reconcile.rs` must judge the row, `Ok(())`; a row is erased with its forge in one decision.
  - If a size check of `temper-protocol-store` moves with the longer key, it follows from the code; a test
    pinning a largest key or value takes the new value.
  - `SV/family.rs`'s `12-22` bound assert may gain the family, if it fails to compile otherwise.

## Steps

1. `making.rs`, the variants, `lib.rs`. 2. The domain's map, its restore, `next_making` and `making_due`.
3. The store's schema, writer, `keys.rs`, records. 4. `forge.makings`. 5. The tests, each biting: restore
`Making` as `Making`, and pass the bound by one, each once. 6. The status lines; the implementer's checks;
commit and push.

## Report

As the implementer's text asks; the largest key and value if a test pinned them.
