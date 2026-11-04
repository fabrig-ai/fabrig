# D047: Role sets are written as slice `contains` checks

- **Kind:** primitives
- **Where:** `skein-ui/src/builder.rs`, `skein-ui-web/src/page.rs`, `skein-ui-web/src/mapping.rs`, `skein-ui-text/src/render.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (A11) and `reviews/2026-10-31-skein-areas.md` (A11)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** each set of roles the UI crates test against is a named predicate beside `Role` with an exhaustive match, so a new role breaks the build where it must be placed.
- **Measure:** `rg -c 'Role::\w+(, Role::\w+)+\]\.contains' skein/crates skein/testing | awk -F: '{s+=$2} END{print s}'`: 18 at base

## What

The UI crates test a role's class with `[Role::A, Role::B, ..].contains`
18 times (`page.rs` 7, `builder.rs` 4, `render.rs` 3, and others). The
review found two copies of one set already drifted.

## Why it matters

A slice check compiles whatever roles are added, so a new role silently
falls outside every class; exhaustive matches are the rule this tree
relies on.

## What pays it back

Named predicates on `Role` (`is_landmark`, `is_field`, ...) defined once
with exhaustive matches, and the slice checks replaced.
