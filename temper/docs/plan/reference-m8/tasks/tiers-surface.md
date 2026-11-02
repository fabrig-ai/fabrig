+++
id = "tiers-surface"
part = "tiers-forge"
kind = "interface"
needs = ["change-verifying", "tiers-config"]
files = ["temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/purpose.rs", "temper/crates/temper-engine-domain-forge/src/tiers.rs", "temper/crates/temper-engine-domain-forge/src/verification.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/landing.rs", "temper/crates/temper-protocol-store/**", "temper/tests/codecs/**"]
effort = "high"
minutes = 180
+++

# The forge's tier surface: events, requests, purpose tag 8, store families 28 and 29

## Goal

The types every tier task builds on, with no behaviour: the forge's tier events and requests,
the tier task's decision, purpose tag 8, the store families `Tier` (28) and `Verification` (29)
and a receipt's failing tests, and the forge's three read methods as stubs. After it,
`tiers-runs` (the forge's bodies) and `tiers-root` (the root's joins) run side by side.

Acceptance:

- `notes/contracts.md`, 4 exists whole in `temper-engine-domain-forge`: `Request::{TierRun,
  TierDecision}`, `TierDecision`, `Event::{TierTask, TierRefused, StepTier, VetoTier}`,
  `TierHeard`, `EffectPurpose::Tier` (tag 8, 30 bytes with the tag, encoded and decoded beside
  tags 6 and 7 with a round-trip test and a size test), `Stored::{Tier, Verification}`,
  `Key::{Tier, Verification}`, `TierRow`, `Verification`, `TierVerification`, `TierVerdict`,
  `Receipt::failures`. New modules `tiers.rs` and `verification.rs` hold the rows' helpers and
  the stubs: `Domain::tier_task` (None), `Domain::describe_tier_effect` (None),
  `Domain::change_verification` (`Unfollowed`). The new events are accepted and change nothing
  (a test says so); nothing emits the new requests yet.
- The store keeps both families at bytes 28 and 29 (`temper/docs/design/protocol/store.md`, 2):
  `keys.rs` constants, encode and decode (a tier's name checked as a gate's on decode), the
  schema's records (`TierRow` with the pure crate's `Record` in full: `Failures`, `Run`, `Part`,
  `Selection`, `Finding`, `Blame`, `Flake`, `Judged`, `Landing`), `Verification`, the receipt's
  optional failures; `records/tiers.rs` and `records/verification.rs`; `records::limits` from the
  forge's `Limits::tiers` and `changes`; generated code and `golden/v11` regenerated through the
  drift test; key order still sorts as `Key`'s variants (`tests/keys.rs`).
- Restore and reconcile know both families (the forge's `lib.rs` key and byte arms, `restore.rs`,
  `reconcile.rs`, and the restore table `src/tests/restore_fit.rs`): `Tier` bounded by
  repositories times `tiers.tiers`, `Verification` by `changes`; the receipt bound counts tiers
  beside gates (`repositories * (gates + tiers) * receipts_per_gate`).
- Every exhaustive match outside the forge gains its arm only: `temper/crates/temper-fake/src/check.rs`,
  `temper/tests/engine/protocol-forge/src/protocol.rs`, `temper/tests/engine/forge/src/world.rs`,
  `temper/tests/codecs/tests/suite/fuzzy/store.rs` (`forge_rows` makes rows of both families),
  and the root (`temper-engine-domain`): `admission.rs` classifies the new events as the landing's
  are; `route/forge_route.rs` matches the new requests with an arm that does nothing, a comment
  naming `tiers-root`.
- Packages pass: `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
  `-p temper-protocol-store --lib`; `-p temper-codecs-world --test suite`;
  `-p temper-engine-domain --lib`; `heavy cargo check -p temper-engine-forge-world
  -p temper-engine-domain-world -p temper-fake -p temper --tests`.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 1, 3 and 4.
- `temper/docs/design/domain/after-landing.md`, section 10 (a run is a task; effects; the purpose;
  the store); `temper/docs/design/protocol/store.md`, sections 2 to 4 (a new family; key order;
  goldens; in place until the first release).
- How a family was added in place, by a survey: schema record and `Value` variant, regenerate,
  `Stored` and `Key` variants in the forge's `boundary.rs` and the `stored_key` arm in its
  `lib.rs`, `keys.rs` constant and arms, `records.rs` arms and a new `records/<family>.rs`, its
  limits, then the store's tests (`src/tests/{fixtures,records,keys,sizes}.rs`) and the codecs
  world (`tests/codecs/tests/suite/fuzzy/store.rs`). Regenerate with
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E
  'test(drift::)'`.
- The old code, shapes to adapt: `temper/docs/plan/reference-m8/notes/old/temper-2a-store.diff`
  (keys, schema records, `records/{tiers,verification}.rs`, codecs tests; its bytes 25 to 27,
  its `LandingLog` and its mid-enum change states are not taken) and `temper-2a-forge.diff`
  (`boundary.rs`, `purpose.rs`).

## Owns and may change

- Owns the files and packages in the header, and in the forge crate the arms and stubs above in
  `lib.rs`, `restore.rs`, `reconcile.rs`, `domain.rs` (event dispatch arms, and the two families'
  retained storage and restore: see Answers) and `src/tests/restore_fit.rs`; in `landing.rs`,
  the receipt's bound and size only (see Answers).
- The root's `admission.rs` and `route/forge_route.rs`: the arms above only.
- Not this task's: following, runs, verdicts, the change's facts (`tiers-runs`,
  `tiers-verification`); the root's tier task (`tiers-root`).

Pre-approved options:

- If a store test pins the largest key or row (`src/tests/sizes.rs`,
  `the_application_sizes_are_the_largest_key_and_row`) and a new family moves it, update the
  figure to the new measure and give both; if it passes the application's ceiling
  (`largest_forge_record_fits_the_application_ceiling`), shrink the row's bounds (a tier's
  `landings` in a run at most `2 * changes + 1`, the old line's bound) before asking.
- If a figure under `temper/crates/temper/tests/figures/` moves, regenerate it as its test says.

## Steps

1. The forge's types and stubs; the purpose; the arms.
2. The store's families and the receipt's failures; regenerate; the store's and codecs' tests.
3. `cargo fmt`; clippy over every package you changed; the tests in Goal, each count reported.
4. Commit (why: the shapes the tier tasks share); push your branch.

## Report

As the implementer's text says, and also: the sizes that moved; the regenerated goldens (count);
any field you added beyond contracts.md, 4, and why.

## Answers

2026-11-02. May the task add the families' storage in `domain.rs` and the receipt's accounting
in `landing.rs`, which the restore acceptance needs? Yes, at the least the acceptance needs:

- `domain.rs`: `Domain` fields for the `Tier` and `Verification` rows, sized from
  `repositories * tiers.tiers` and `changes`, initialized in construction; the restore dispatch
  inserting their restored rows; their `restored_link_key` arms. Rows are kept and restored only:
  no following, runs, verdicts or change facts (`tiers-runs`, `tiers-verification`), and the three
  read methods stay stubs.
- `landing.rs`: `receipt_slots` counts tiers beside gates
  (`repositories * (gates + tiers) * receipts_per_gate`, checked arithmetic as today);
  `receipt_bytes` counts the receipt's optional failures; construction sites set
  `failures: None`. No other landing change.
