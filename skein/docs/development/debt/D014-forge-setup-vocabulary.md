# D014: forge-setup's shared vocabulary lives in Forgejo's reconcile, and `Reply` mixes dialects

- **Kind:** coupling
- **Where:** `skein-forge-setup/src/reconcile.rs`, `skein-forge-setup/src/github`, `skein-forge-setup/src/host`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T2, the forge-setup row; A4) and `reviews/2026-10-31-skein-areas.md` (A4, F1)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 3)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** the vocabulary both dialects use (`Need`, `Event`, `Reply`, `Failure`, `Stage`, `Why`) sits in a module of its own; each dialect's plans see a `Reply` of their own dialect only; the admin step and the plans' `stop` are each written once.
- **Measure:** `rg -c 'GithubRows\(' skein/crates/skein-forge-setup/src | awk -F: '{s+=$2} END{print s}'`: 61 at base

## What

- `Need`, `Event`, `Reply`, `Stage` and `Why` are defined in
  `skein-forge-setup/src/reconcile.rs`, Forgejo's reconcile, and used by
  `github/` too.
- `Reply` carries both dialects' bodies, so plans list the other
  dialect's variants to reject them: `GithubRows(` appears 61 times.
- The review found the admin step written five times and `stop` in six
  shapes, and the crate spelling `Github*` beside `GitHub*` (see D037).

## Why it matters

The crate is asymmetric: GitHub's plans depend on Forgejo's module, and
every new reply variant adds an arm to every plan of the other dialect.

## What pays it back

A `plan` vocabulary module, a `forgejo/` directory beside `github/`, a
`Reply` per dialect, and an admin sub-machine. It goes better after
D010.
