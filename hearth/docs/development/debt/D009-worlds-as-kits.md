# D009: World crates are used as kits by other test crates

- **Kind:** tests
- **Where:** `tests/builder`, `tests/render`, `tests/config`, `tests/git`, `tests/conformance`, `hearth/crates/hearth/Cargo.toml`
- **Seen:** 2026-11-04 at `66968a30ef`, in skein's D028, the same shape; met in hearth's manifests
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** low (a world's changes stop rebuilding and breaking its neighbours)
- **Size:** M
- **Status:** open
- **Paid when:** no hearth package depends on a world crate under `hearth/tests/`; what a second crate needs lives in a kit under `hearth/testing/`.
- **Measure:** `git grep -E '^hearth-[a-z]+-world\b' -- 'hearth/*Cargo.toml' | wc -l`: 4

## What

Four manifest edges point at worlds: `hearth` on `hearth-builder-world`,
for its end-to-end tests; `hearth-builder-world` on `hearth-render-world`,
for the goldens; `hearth-render-world` on `hearth-config-world`; and
`hearth-conformance-world` on `hearth-git-world`. The goldens' readers also
cross by `#[path]` (D008).

## Why it matters

A tree's layout keeps shared harnesses in `testing/` and worlds in `tests/`
(`skein/docs/design/testing.md`, 6).
These edges are why a change to the render world rebuilds the builder's,
and a world's helpers are public API to its neighbours.

## What pays it back

Move the shared parts, the repository fixtures, the goldens' readers and
the scratch repository helper, into kits under `hearth/testing/`. Needs a
look at what each edge uses first: the builder's is the largest.
