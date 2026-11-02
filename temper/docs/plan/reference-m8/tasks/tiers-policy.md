+++
id = "tiers-policy"
part = "tiers-core"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge-tiers/**", "Cargo.toml", "Cargo.lock"]
effort = "high"
minutes = 70
+++

# The pure tier verdict policy, `temper-engine-domain-forge-tiers`

## Goal

A new pure crate holding the report and verdict policy of after-landing's tiers: a run's
report read into failing test ids, and the level-triggered decisions of one tier (due, start,
step through a whole run, a rerun, bisect probes, faults and the hold, the judged record,
repaired). It is the old crate, ported to today's tree; the forge connector builds on it in
a later task.

Acceptance:

- `temper/crates/temper-engine-domain-forge-tiers` is a workspace member with the public
  surface of `temper/docs/plan/reference-m8/notes/contracts.md`, 1, names and signatures
  unchanged, `no_std` with `alloc` outside its tests, depending on `skein-lib` alone.
- Its unit tests cover at least the old crate's cases (the diff's `tests/report.rs` and
  `tests/verdict.rs`: JUnit failed and errored cases, entities, malformed input, lines,
  the count and id bounds, a failed run naming no test is a fault; due, the baseline, green,
  rerun, flake, one landing owns, bisect skipping an unchanged key, inherited failures, a
  later green repairing, tell-all, `Many` in each of its paths, receipts passed and failed,
  faults retried then held, the flake record's bound) and pass:
  `heavy cargo nextest run -p temper-engine-domain-forge-tiers`.
- `heavy cargo clippy -p temper-engine-domain-forge-tiers --all-targets -- -D warnings` is
  clean under `temper/clippy.toml`; `cargo fmt` leaves nothing.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 1.
- `temper/docs/design/domain/after-landing.md`, sections 2 (keys, receipts, the baseline),
  3 (range, faults), 4 (what a run reports) and 5 (the verdict); section 10's limits
  (`failing_tests`, `test_id_bytes`).
- `temper/docs/design/domain/landing.md`, 6.1 and 6.2 (a gate's key and receipts, which a
  tier's are).
- `skein/docs/foundation/programming-model.md`, section 10 (what a step crate may use).
- The old crate: `temper/docs/plan/reference-m8/notes/old/temper-tiers-crate.diff`, which
  `git apply` creates whole at the right path (checked against this plan's base).
- The template beside it: `temper/crates/temper-engine-domain-forge-landing` (its
  `Cargo.toml`, `lib.rs`, its tests' layout).

## Owns and may change

- Owns `temper/crates/temper-engine-domain-forge-tiers/**` (new).
- Root `Cargo.toml`: one line in `[workspace] members` beside
  `"temper/crates/temper-engine-domain-forge-landing"`, and one line in
  `[workspace.dependencies]` beside `temper-engine-domain-forge-landing = { path = ... }`.
  `Cargo.lock` as cargo writes it. Nothing else of either.
- No other crate names the new one in this task.

Pre-approved options:

- Where today's `skein-lib` changed an API the old code uses (`List`, `Queue`, their
  constructors and errors), use today's form and say so.
- Where today's clippy or `temper/clippy.toml` refuses an old form, reshape it inside the
  crate; a scoped `#[expect(clippy::<lint>, reason = "...")]` only where reshaping would
  change the policy, and listed.
- If an old test asserts behaviour that contradicts after-landing.md 2 to 5 as written, the
  design wins: change the test, and name it with the section in the report.

## Steps

1. `git apply temper/docs/plan/reference-m8/notes/old/temper-tiers-crate.diff`; add the
   crate to the workspace (members and dependencies, as above).
2. `heavy cargo check -p temper-engine-domain-forge-tiers --tests` and repair it against
   today's `skein-lib` and lints. Module docs cite after-landing.md by path from the root
   and section (`temper/docs/design/domain/after-landing.md, 5`), as the tree's code does.
3. Read `verdict.rs` against after-landing.md 5.2 and 5.3 and the decisions in
   contracts.md, 1; the tests must name each decision they pin.
4. `cargo fmt`; clippy as above; `heavy cargo nextest run -p temper-engine-domain-forge-tiers`
   (check the count).
5. Commit (say why: the policy tiers judge by, ported from the old line); push your branch.

## Report

As the implementer's text says, and also: what of the old crate you changed and why; the
test count; anything of after-landing.md 2 to 5 the crate cannot decide alone (the top's
part), for the forge task.
