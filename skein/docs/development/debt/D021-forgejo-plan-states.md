# D021: Forgejo's plans keep a `Stage` beside bags of fields, and measure answers twice

- **Kind:** complexity
- **Where:** `skein-forge/src/forgejo/plans`, `skein-forge/src/forgejo/plans/reads/size.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T4, the Forgejo plans row; A2) and `reviews/2026-10-31-skein-areas.md` (A2, F2 and F3)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** each Forgejo plan's stages are enum variants holding their data, matched per stage with no tuple scrutinee and no `if let` on the crate's own enums; the plans measure an answer with one function.

## What

- The plans under `forgejo/plans` hold a `Stage` beside 10 to 22 fields
  each; the review counted 22 `if let` and 13 `let … else` on own enums
  and tuple scrutinees with `_` arms.
- `plans/reads/size.rs` is a second measure of an answer's allocation,
  which the review found drifted from the first.

## Why it matters

`skein/docs/foundation/programming-model.md`, 5.4 and 10.3: matches that
fall outside the lint's sight do not break the build when a stage is
added.

## What pays it back

One plan per task onto data-carrying stages, and one answer measure. A
rules checker (`skein/docs/foundation/programming-model.md`, 10.3) would keep the forms out.
