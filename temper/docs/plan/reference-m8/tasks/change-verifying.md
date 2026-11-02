+++
id = "change-verifying"
part = "change-cleared"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain-forge-change/**", "temper/crates/temper-protocol-store/schema/store-v11.schema", "temper/crates/temper-protocol-store/src/generated/**", "temper/crates/temper-protocol-store/golden/v11/**", "temper/crates/temper-protocol-store/src/records/change.rs", "temper/crates/temper-protocol-store/src/tests/**"]
effort = "high"
minutes = 150
+++

# A change lives to cleared: the change procedure's states and facts

## Goal

The change procedure learns after-landing's life: landed means verifying, then red, cleared,
or uncleared past the project's bound or on a cancel, steered by a new fact, the landing's
verification. The forge still says `Unfollowed` for every change (no tier exists yet), so every
landing behaves exactly as today; `tiers-forge` gives the fact its values later. This is the
surface that task, the root's result words and the panel build on.

Acceptance:

- `temper-engine-domain-forge-change` has `notes/contracts.md`, 2 whole: `State::Landed { merge,
  since }`, `Red`, `Cleared`, `Uncleared`, `Verification`, `Facts::{verification,
  verification_bound}`, and the steps listed there, each pinned by a unit test in the crate's
  tests (`src/tests.rs` or a new `src/tests/verifying.rs`): merged with `Unfollowed` finishes at
  once at `Landed`; `Verifying` waits until `since + bound`, then `Uncleared` and finishes;
  `Red` waits with no deadline and stays red on `Verifying`; `Cleared` from `Landed` and from
  `Red` finishes at `Cleared`; a cancel while verifying or red finishes `Uncleared`.
- The store keeps them: `ChangeLive` appends `red`, `cleared`, `uncleared` after `no_change`;
  `LandedChange` gains `since` (the wall time as the schema writes other wall times), and the
  red state carries it too; `records/change.rs` encodes and decodes each; the generated code
  and `golden/v11` regenerated through the drift test (no existing `enum_change_live_*` value's
  number moves). `STORE_VERSION` stays 11 (`temper/docs/design/protocol/store.md`, 4).
- The forge builds the fact: in `temper/crates/temper-engine-domain-forge/src/domain.rs` where
  it fills `change::Facts` (about line 3126), `verification: change::Verification::Unfollowed`
  and `verification_bound` two hours, a named constant beside it saying `tiers-config` and
  `tiers-forge` replace it. Behaviour of every existing test is unchanged.
- Packages pass: `heavy cargo nextest run -p temper-engine-domain-forge-change`;
  `-p temper-protocol-store --lib`; `-p temper-codecs-world --test suite -E 'test(/drift|store/)'`;
  `-p temper-engine-domain-forge --lib`; `-p temper-engine-domain --lib`;
  `-p temper-engine-domain-world --test suite -E 'test(/forge_routes/)'`;
  `-p temper-engine-forge-world --test suite`. These last two are the readers of `Landed`'s
  meaning: run them although nothing should change.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 2.
- `temper/docs/design/domain/after-landing.md`, section 6; `temper/docs/design/protocol/store.md`,
  sections 2 and 4 (the `Change` family; a schema changes in place before the first release; how
  goldens regenerate: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world
  --test suite -E 'test(drift::)'`).
- The old code, for shape only: `temper/docs/plan/reference-m8/notes/old/temper-2a-forge.diff`
  (`temper-engine-domain-forge-change`'s `Red`/`Cleared` and its verification fact) and
  `temper-t9-8140d6c33.diff` (the bound, `uncleared`). The old code put the bound in the change's
  `Limits` and inserted the states mid-enum: do neither (contracts.md, 2).

## Owns and may change

- Owns the change crate and the store files in the header.
- Compiler-forced arms and patterns for the new states and `Landed`'s new field, each mapping a
  new state as `Landed` maps today (no new behaviour), in:
  `temper/crates/temper-engine-domain-forge/src/{boundary.rs,brief.rs,domain.rs,landing.rs,lib.rs,panel.rs,topics.rs}`
  (the panel shows `Red`, `Cleared` and `Uncleared` as its `Landed` step until `panel-steps`);
  `temper/crates/temper-engine-domain/src/route/forge_route.rs` (result words: as `Landed`;
  `tiers-root` gives them their own); `temper/crates/temper-fake/src/check.rs`;
  `temper/tests/system/src/forge/stories.rs`, `temper/tests/conformance/src/stories.rs`, and the
  test files matching `State::Landed { merge }` (`temper/tests/engine/domain/tests/suite/forge_routes.rs`,
  `forge_routes/result.rs` and their siblings): the pattern gains `..`, nothing else.
- The `Facts` literal in the forge's `domain.rs`, as above, and in the change crate's tests.

Pre-approved options:

- If a store test pins the largest row or value size (`src/tests/sizes.rs`) and the new field
  moves it, update the pinned figure to the new measure and give both in the report.
- If a figure under `temper/crates/temper/tests/figures/` moves, regenerate it as its test's
  write mode says, and list it.
- If `since` cannot be the step's wall time where the top first sees the merge (the forge's
  `env.wall` in the step), say which time it is.

## Steps

1. The change crate: states, `Verification`, facts, steps (contracts.md, 2), unit tests.
2. The store: schema, regeneration, `records/change.rs`, its tests; the forge's facts and arms.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge-change -p temper-protocol-store
   -p temper-engine-domain-forge -p temper-engine-domain -p temper-fake --all-targets -- -D
   warnings`; the tests in Goal, each count in the report.
4. Commit (why: a landing is verified before its change ends); push your branch.

## Report

As the implementer's text says, and also: every file you touched for an arm only; the golden
files regenerated (count); any pinned size that moved.
