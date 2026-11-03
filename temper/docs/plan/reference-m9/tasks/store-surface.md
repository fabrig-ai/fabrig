+++
id = "store-surface"
part = "surface"
kind = "interface"
needs = []
files = ["temper/crates/temper-protocol-store/schema/store-v11.schema", "temper/crates/temper-protocol-store/src/generated/v11.rs", "temper/crates/temper-protocol-store/golden/v11/**", "temper/crates/temper-protocol-store/src/records/change.rs", "temper/crates/temper-protocol-store/src/records/verification.rs", "temper/crates/temper-protocol-store/src/records/tiers.rs", "temper/crates/temper-protocol-store/src/records.rs", "temper/crates/temper-protocol-store/src/tests/**", "temper/tests/codecs/**", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/verification.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests/verification.rs"]
effort = "high"
minutes = 150
+++

# The store keeps a change's `repairs`, a repair's landing and a tier's goal branches

## Goal

Add the fields this plan's later tasks build on, with nothing reading them yet: a change's
`repairs`, a repair's `by`, and a tier row's goal branches, its run's goal and its last start;
and the forge's read `Domain::landing_red`, which the root's check of `repairs` will call.
Nothing changes meaning: every new field is `None` or empty where nothing sets it.

Acceptance:

- `notes/contracts.md`, section 1, holds exactly: the schema's appended fields and new record
  `TierGoal`, version 11 kept; the forge's `ChangeRow.repairs`, `TierRepair.by`, `TierGoal`,
  `TierRow.goals`, `TierRow.goal`, `TierRow.started`; `LandingRed` and
  `Domain::landing_red`, exported from the forge crate's root as its other public types are.
- The store translates each new field both ways (`records/change.rs`, `records/verification.rs`,
  `records/tiers.rs`); a decoded `TierRow.goals` that is not sorted and unique by `goal`, or a
  `TierGoal` whose `branch` is empty, is `Undecodable`, as the record's other lists are checked.
- The forge's byte accounting counts the new fields where it counts their records' others
  (`lib.rs`'s change bytes; `tiers.rs`'s `row_bytes` is `goal-tier-forge`'s: leave it, and
  say so in the report if its tests need the goals counted).
- `landing_red` answers from the live `Verification` rows: `Unknown` with no row of
  (repository, commit); `Elsewhere { branch }` when the row's first live change's `base` is not
  `branch`; `NotRed` when no `TierVerification` has a verdict whose blame is `Owns` or
  `Suspect` without a `repaired`; `Red` otherwise.
- Tests pass: the codec round trips carry each new field set (`src/tests/records.rs`,
  `src/tests/tiers.rs`, the fixtures); the drift check; four unit stories of `landing_red` in
  `temper-engine-domain-forge/src/tests/verification.rs` (unknown, elsewhere, not red after a
  green, red after a red verdict), built with that file's existing helpers.

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, section 1.
- `temper/docs/design/domain/after-landing.md`, sections 7 and 10;
  `temper/docs/design/domain/agent.md`, 4.3.1 (`repairs`);
  `temper/docs/design/protocol/store.md`, section 4 (v11 edited in place until the first release).

## Owns and may change

- Owns the files of this spec's header.
- Reader files, literals only (each new field `None`, empty or `Box::new([])`): every
  `ChangeRow {` literal (`temper-engine-domain-forge/src/{domain.rs,tests/fixtures.rs,tests/panel.rs}`,
  `temper-engine-domain/src/route/forge_route.rs` in `start_change`,
  `temper-engine-domain/src/route/forge_route/tests.rs`, `temper-engine-domain/src/tests/documents.rs`,
  `temper-fake/src/tests/check.rs`, `temper/tests/engine/domain/tests/suite/forge_routes.rs`,
  `temper/tests/engine/forge/src/changes.rs`, `temper/tests/engine/protocol-forge/tests/suite/protocol.rs`);
  every `TierRepair {` literal (`verification.rs`'s `judged` sets `by: None`); every `TierRow {`
  literal (`temper-engine-domain-forge/src/tiers.rs`'s `empty` and the tests'). `rg -n 'ChangeRow \{|TierRow \{|TierRepair \{' temper`
  lists them; a literal the compiler asks for elsewhere in temper is allowed the same way.
- `start_change` (`forge_route.rs`) sets `repairs: None` alone: reading parameter 12 is
  `repairs-call`'s.
- If another plan's field lands in the same records first (reference-m3, reference-m5,
  reference-m6 and first-run-projects append to `ChangeRow` and others in v11), append after
  theirs and regenerate from the combined schema.

## Steps

1. The schema, as contracts.md, 1 writes it; regenerate the code and goldens with
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`,
   then run it again without the variable to confirm the committed output matches.
2. The forge's types and their translations; the literals the compiler asks for.
3. `landing_red` in `verification.rs`, beside `change_verification`, and its four stories.
4. Tests: `heavy cargo nextest run -p temper-protocol-store`;
   `-p temper-codecs-world --test suite -E 'test(drift::)'`;
   `-p temper-engine-domain-forge --lib`; `heavy cargo check -p temper-engine-domain --tests`,
   `-p temper-engine-domain-world --tests`, `-p temper-engine-forge-world --tests`,
   `-p temper-engine-protocol-forge-world --tests`, `-p temper-fake --tests` (the readers'
   literals).
5. `cargo fmt`; `heavy cargo clippy -p temper-protocol-store -p temper-engine-domain-forge
   -p temper-engine-domain --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks; name each reader file you touched and the regeneration you ran.

## Answers

2026-11-03. Q: `goals: list 65536 TierGoal` makes `TierRow`'s encoded worst case pass u32, so
the schema does not generate: lower the ceiling or change the layout? A: lower the ceiling, the
layout kept. `notes/contracts.md`, section 1, now writes `goals: list 256 TierGoal` (with
`TierRow`'s other lists, about 2.2 GiB worst case, within u32); use it as written. In
`records.rs`'s `tier_limits` (now in this spec's files) set `tier_row_goals: forge.holds`, so
`bounds` refuses a forge whose `holds` passes the ceiling at start, as `tier_row_pending`
bounds `changes`; add one assertion beside `tests/tiers.rs`'s other limit checks that
`tier_row_goals` is the fixture's `holds`. The pushed schema on `reference-m9/store-surface`
is the start: change its 65536 to 256 and regenerate.
