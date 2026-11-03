+++
id = "goal-tier-config"
part = "tier-config"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/startup/application/landing.rs", "temper/crates/temper-service/src/tests/startup.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/tiers.rs", "temper/crates/temper-engine-domain-forge/src/landing.rs", "temper/crates/temper-engine-domain-forge/src/verification.rs", "temper/crates/temper-engine-domain-forge/src/tests/tiers.rs", "temper/crates/temper-engine-domain-forge/src/tests/landing.rs", "temper/crates/temper-engine-domain-forge/src/tests/verification.rs"]
effort = "high"
minutes = 120
+++

# A tier follows its landing branch, one branch or the goals' own branches, with a span

## Goal

The configuration's tier takes `"branch": "goal"` (the goals' own branches) and `"span_ms"`,
and the forge's `Tier` says what it follows with `Follows` and carries its span. A tier
following goal branches stays inert here: it follows nothing and judges nothing, so a change
into a goal's branch still ends at landed. `goal-tier-forge` makes it run.

Acceptance (`notes/contracts.md`, section 4):

- The forge's `Tier` has `follows: Follows` in place of `branch`, and `span:
  Option<skein_lib::Duration>`, last; `Follows` beside it, exported as `Tier` is.
- `TierCheck` has `follows` in place of `branch` and `span_ms: u64`, last; its translation to
  the forge's `Tier` maps `span_ms` 0 to `None` and any other to that many milliseconds in the
  type `AfterLanding.bound` uses.
- The document's `tier` (`landing.rs` in temper-service) reads `"branch"` absent → `Landing`,
  exactly `"goal"` → `Goals`, another non-empty string → `Branch`, empty refused as today;
  `"span_ms"` a u64, absent → 0, any other type refused with the member's name as the other
  members are.
- `valid_tier` (`temper-engine-domain-forge/src/landing.rs`) checks `Branch`'s name as it
  checks `branch` today; `Landing` and `Goals` need nothing.
- `tiers::followed` answers the landing branch for `Landing`, the name for `Branch`, `None` for
  `Goals`; `verification::landed` counts no `Goals` tier, so a landing into a goal's branch
  makes no `Verification` row and its change ends `Unfollowed`.
- `tiers::definition` writes exactly today's bytes for `Landing` (as `branch: None`) and
  `Branch` (as `branch: Some`), and for `Goals` a marker of its own that no `Landing` or
  `Branch` definition can write; the span is not in it.
- Tests: in `temper-service/src/tests/startup.rs`, a tier with `"branch": "goal"` and
  `"span_ms": 1200000` reads as `Goals` and 1,200,000, one with neither as `Landing` and 0, one
  with a `"span_ms"` string is refused naming it; in the forge's unit tests, a `Goals` tier
  starts no run on a move of main nor of `temper/1/g1` (`tests/tiers.rs`), a change landing into
  `temper/1/g1` with a `Goals` tier and a main tier configured ends unfollowed while one into
  main is verified (`tests/verification.rs`), and the keys of a `Landing`, a `Branch` and a
  `Goals` tier of one definition otherwise are three, the first two equal to what they were
  (pin today's bytes in the test before you change `definition`).

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, section 4.
- `temper/docs/design/domain/after-landing.md`, section 2 (goal branches, a tier's span) and
  section 6 (cleared at once where only tiers following goal branches follow);
  `temper/docs/design/protocol/configuration.md`, section 3 (`branch`, `span_ms`).

## Owns and may change

- Owns the files of this spec's header for what acceptance names.
- Reader files, compiler-forced only (each a `branch:` that becomes `follows:` with the same
  meaning, and `span: None` or `span_ms: 0`): `temper-engine-domain/src/route/forge_route/tests.rs`
  (`result_tier`), `temper/tests/engine/domain/src/tier_runs.rs`,
  `temper/tests/engine/forge/tests/suite/tiers.rs` and `temper/tests/engine/forge/src/world.rs`,
  and any other literal of `Tier` or `TierCheck` the compiler names in temper.
- `store-surface` edits `boundary.rs`, `verification.rs` and `tiers.rs` at the same time, in
  other places (`ChangeRow`, `TierRepair`, `TierRow`, `landing_red`, `judged`, `empty`): touch
  only `Tier`, `Follows`, `landed`, `followed`, `definition` and `valid_tier` there.
- `temper/docs/help/tier.md`'s example is read by `temper/crates/temper/src/tests/help.rs`: it
  must still parse; if it does not, say so in the report (help is not yours).

## Steps

1. Pin today's definition bytes in a test; then the types, the parse, the translation.
2. `followed`, `landed`, `definition`, `valid_tier`.
3. Tests: `heavy cargo nextest run -p temper-service --lib -E 'test(/startup/)'`;
   `-p temper-engine-domain-forge --lib`; `-p temper-engine-domain --lib -E 'test(/tier/)'`;
   `-p temper-engine-domain-world --test suite -E 'test(/tiers/)'`;
   `-p temper-engine-forge-world --test suite -E 'test(/tiers/)'`;
   `-p temper --lib -E 'test(/help/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper-engine-domain
   -p temper-engine-domain-forge --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks; say how `Goals`' marker keeps it apart.
