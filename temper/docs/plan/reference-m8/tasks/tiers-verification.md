+++
id = "tiers-verification"
part = "tiers-forge"
kind = "feature"
needs = ["tiers-runs"]
files = ["temper/crates/temper-engine-domain-forge/src/verification.rs", "temper/crates/temper-engine-domain-forge/src/tests/verification.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs"]
effort = "high"
minutes = 240
+++

# A landing's verification, and the change's facts from it

## Goal

Each landing temper makes into a branch a tier follows has a `Verification` row, made with its
`Landed` row and erased when its last change closes; each tier's verdict is written on it
(cleared, or the failures it owns), a red one cleared by the first later run that passes its
failures; and every change's facts carry its landing's verification and its project's bound, so
a change lives landed, verifying, then cleared, red or uncleared.

Acceptance (`temper/docs/design/domain/after-landing.md`, 5.3, 6 and 10):

- `verification.rs`: a merge temper makes into a branch some tier follows makes a
  `Verification` row (`changes`: the change tasks it landed; `tiers`: one per tier following
  that branch, `verdict: None`) in the same commit as its `Landed` row; a merge into a branch no
  tier follows makes none. `judged(d, repository, tier, &verdict)` (called by `tiers-runs`)
  writes each landing's `Blame` in the run's range on its row (`TierVerdict { run, blame }`),
  and for each earlier red landing of that tier sets `repaired` to the run's tip's landing when
  `repaired(blame, &verdict.failing)` holds (the pure crate). Rows are erased with their last
  change's close, so bounded by `changes`.
- `Domain::change_verification(task)`: `Unfollowed` when the change's landing branch has no
  tier or the change has no row; `Red` when a tier's verdict owns failures not yet repaired;
  `Cleared` when every tier's verdict is `Cleared` or repaired; else `Verifying`.
- The change's facts (`domain.rs`, where `change::Facts` is filled): `verification` from
  `change_verification`, `verification_bound` from `Domain::after_landing(project)` (the
  constant `change-verifying` left goes); a change is stepped again when its row's verdicts
  move (a verdict wakes the change tasks it names, as a delegate's result does), and at its
  bound's deadline.
- A late verdict, after the change ended uncleared: its row is gone, so it changes no change;
  the tier's record keeps it (inherited failures). Nothing more (after-landing.md 6's news and
  findings are not this plan's).
- Unit stories (`src/tests/verification.rs`): two landings in one green run, each change
  finishes `Cleared`; a run blaming the second landing makes its change `Red` and clears the
  first; a later green run passing those failures makes it `Cleared` with `repaired` set; no
  verdict past the bound makes `Uncleared`; a cancel while verifying, `Uncleared`; a change into
  a branch no tier follows (a goal's own branch, in a repository whose tier follows main)
  finishes at `Landed` at once, as today; a cold restart keeps the rows.
- Packages pass: `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
  `-p temper-engine-forge-world --test suite`; `-p temper-engine-domain-world --test suite -E
  'test(/forge_routes/)'` (the readers of a change's end: they must not change, since none of
  their repositories has a tier).

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 1, 2 and 4.
- `temper/docs/design/domain/after-landing.md`, 5.3, section 6, section 10 (`Verification`).
- The old code: `temper/docs/plan/reference-m8/notes/old/temper-2a-forge.diff`
  (`verification.rs`, the change's verification fact) and `temper-t9-8140d6c33.diff`
  (verification from the observed landing, the bound, late verdicts; its news to goals and
  retained recipients are M9's: leave them).

## Owns and may change

- Owns `verification.rs` and its tests; in `domain.rs`, the facts and the change's wakes only;
  in `tiers.rs`, nothing but the call into `verification::judged` if its arguments need to grow.
- Not this task's: `tiers.rs`'s runs, reports and probes (`tiers-reports` runs beside this
  task and owns them); the root; the panel.

Pre-approved options:

- If a change must be woken by an event this crate has no path for, use the path a delegate's
  result takes to wake a change task, and name it.
- `tiers-reports` changes `tiers.rs` beside you: keep your `tiers.rs` hunks to the call into
  `verification::judged`, so the two branches merge cleanly.

## Steps

1. Rows and their life; `judged`; `change_verification`; the facts and wakes.
2. The unit stories; then the worlds.
3. `cargo fmt`; clippy over every package you changed; the tests in Goal, counts reported.
4. Commit (why: a change's result waits for its landing's verdicts); push your branch.

## Report

As the implementer's text says, and also: how a verdict wakes its changes; what a late verdict
does.
