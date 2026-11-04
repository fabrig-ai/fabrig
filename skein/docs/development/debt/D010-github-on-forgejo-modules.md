# D010: GitHub's dialect stands on Forgejo's modules

- **Kind:** coupling
- **Where:** `skein-forge/src/github`, `skein-forge/src/forgejo`, `skein-forge-connection`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T2, the skein-forge row) and `reviews/2026-10-31-skein-areas.md` (A1, F1)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 3)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high
- **Size:** M
- **Status:** open
- **Paid when:** no non-test code under `skein-forge/src/github` names `forgejo::`; what both dialects share (time, path, body text, `Malformed`, one `Limits` check, the plan `Failure`, the user table) sits at skein-forge's root, and forge-connection no longer translates twin types field by field.
- **Measure:** `rg -c 'forgejo::' skein/crates/skein-forge/src/github -g '!**/tests/**' -g '!tests.rs' | awk -F: '{s+=$2} END{print s}'`: 44 at base

## What

- Non-test code in `skein-forge/src/github` names `forgejo::` 44 times:
  `document::Body`, `id::unsigned`, `time`, `path`, `decode::Malformed`.
- What it does not borrow it copies: twin `Limits`
  (`forgejo/limits.rs`, `github/limits.rs`), twin plan `Failure`s,
  `Bounds`, the LRU tables (D006) and the field walkers.
- forge-connection maps the twins onto each other (its
  `connection::Failure` mirrors both dialects' identical `Failure`).
- `skein/docs/design/forge.md`, 4.1, gives each dialect its own names,
  times and ids; nothing records the sharing.

## Why it matters

GitHub cannot change without reading Forgejo's modules, and a neutral
fix lands in a dialect's directory. The copies drift (D006). The review
measured about 1.5k lines across the forge family that a neutral root
would remove.

## What pays it back

Move the neutral pieces to skein-forge's root modules (`time`, `path`,
a text `Body`, `decode::Malformed`, one `Limits` with its check, a
`plan` module for `Failure`, `Bounds` and order, the user table), then
point both dialects and forge-connection at them. Splitting skein-forge
into per-dialect crates is not part of it (`skein/docs/design/forge.md`,
12). D005 and D006 are pieces of it; D011 and D013 follow.
