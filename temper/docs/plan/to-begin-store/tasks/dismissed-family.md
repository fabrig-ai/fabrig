+++
id = "dismissed-family"
part = "store"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/dismissed.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/restore.rs", "temper/crates/temper-engine-domain-forge/src/reconcile.rs", "temper/crates/temper-engine-domain-forge/src/limits.rs", "temper/crates/temper-engine-domain-forge/src/tests/dismissed.rs", "temper/crates/temper-protocol-store/**"]
effort = "high"
minutes = 50
+++

# The store family `Dismissed`

## Goal

temper's store holds a row at byte 34 per party and thing dismissed, restored whole at the restart
step and held by the forge connector, read back in key order (contracts K1, K2). Nothing writes a
row yet: `dismiss-request` does.

Acceptance:

- `Dismissal`, `Stored::Dismissed`, `Key::Dismissed`, `RecordKind::Dismissed`, `Limits::dismissed`
  and `Domain::dismissals` exist as K1 has them; the map is sized by `Limits::dismissed`.
- `keys::DISMISSED` is 34, the key and record as K2 has them, with `STORE_VERSION` unchanged, the
  schema's header unchanged, the generated code and goldens from the writer (`notes/common.md`).
- The keys' property test draws `Key::Dismissed`, so the encoder is held to `Key`'s order; its
  decoder's refusals of an unknown kind's tag, a project past `u32` and trailing bytes are tested.
- A restart restores the rows: in a new `temper-engine-domain-forge/src/tests/dismissed.rs`,
  `dismissals_are_the_partys_own_in_key_order`, `dismissals_restore_at_the_bound`,
  `a_dismissal_past_the_bound_is_refused_at_restart`, the last two on the pattern of
  `tests/binding.rs`; and the record round-trips in `temper-protocol-store/src/tests/records.rs`.
- `heavy cargo nextest run -p temper-engine-domain-forge -p temper-protocol-store -p
  temper-codecs-world -p temper-engine-domain` pass; `heavy cargo check --tests -p temper-service -p
  temper-fake -p temper-engine-forge-world -p temper-engine-protocol-forge-world` pass.

## Inputs

- `temper/docs/plan/to-begin-store/notes/common.md`; `notes/contracts.md`, K1 and K2.
- `temper/docs/design/protocol/store.md`, 2 ("A person's dismissals"), 3 and 4.
- `temper/docs/design/domain/forge.md`, 5.5, "Its rows".
- `temper/docs/design/protocol/sign-in.md`, 5, for the bound's members.

## Owns and may change

- The files of the header. `lib.rs`: the exports and the arms of `key` and of its byte count.
- Follow `Binding` through every match: `rg 'Stored::Binding|Key::Binding|RecordKind::Binding'`.
  Reader files, each its arm alone: `temper/crates/temper-fake/src/check.rs`,
  `temper/tests/engine/forge/src/world.rs`, `temper/tests/engine/protocol-forge/src/protocol.rs`,
  `temper/crates/temper-engine-domain/src/**` (a `Stored` or `Key` match).
- `temper/tests/codecs/tests/suite/fuzzy/store.rs`: the family's arm in its generator;
  `temper-protocol-store/src/tests/fixtures.rs`, a new key among its numbered fixtures, keeping their
  order.
- `temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs`: `RecordKind::Dismissed` in
  its tables, which name every family.
- `temper/docs/design/protocol/store.status.md`: remove "family 34 `Dismissed`" and its "(no `Key` or
  `Stored` variant)".
- Pre-approved:
  - `Limits::dismissed` is 0 under a fixture whose `judge_projects` is 0: leave it; that fixture has
    no use for the family.
  - If `reconcile.rs` must judge the row, `Ok(())`, as `Binding`'s, with no check of its project: a
    row is erased in its project's release, in one decision.
  - If a size check of `temper-protocol-store` moves with the new key or value, it follows from the
    code; a test pinning a largest key or value that moves takes the new value.

## Steps

1. `dismissed.rs` with `Dismissal`; the variants at the end of `Stored` and `Key`; `lib.rs`.
2. The domain's map, its restore and capacity; `Limits::dismissed`; `Domain::dismissals`. How the rows
   are held inside is yours, within a linear walk of the party's rows.
3. `temper-protocol-store`: the schema's lines, the writer, `keys.rs`, `records.rs`.
4. The tests, each biting: remove the party's filter in `dismissals`, and pass the bound by one in
   the restore check, each once.
5. The status line; the implementer's checks, packages `temper-engine-domain-forge`,
   `temper-protocol-store`, `temper-engine-domain`; commit and push.

## Report

As the implementer's text asks; the size of the largest key and value if a test pinned them.
