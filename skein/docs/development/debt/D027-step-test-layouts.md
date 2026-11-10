# D027: Step tests follow several layouts, some through `#[path]`

- **Kind:** tests
- **Where:** `skein-oauth-accounts`, `skein-oauth`, `skein-shell`, `skein-forge-setup`, `skein-llm`, `skein/testing/skein-browser`, `skein/testing/skein-fake-person-browser`, `skein/testing/skein-codegen`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, layout) and `reviews/2026-10-31-skein-areas.md` (A0)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 6 and decision D4)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** M
- **Status:** open
- **Paid when:** every crate's step tests are in `src/tests.rs` and `src/tests/` (or a module's own `tests.rs` where a test needs its private items), with no sibling `*_tests.rs` file and no inline `mod tests {}`.
- **Measure:** `find skein/crates skein/testing -name '*_tests.rs' | wc -l`: 21 at base

## What

- 21 sibling `*_tests.rs` files (6,374 lines) are compiled through
  `#[path]`: 8 in `skein-oauth-accounts`, 4 in `skein-browser`, 2 each
  in `skein-fake-person-browser`, `skein-shell` and `skein-forge-setup`,
  one each in `skein-codegen`, `skein-oauth` and `skein-llm`.
- 98 inline `#[cfg(test)] mod tests {` blocks sit in source files.
- `skein/docs/design/testing/README.md`, 6, gives `src/tests.rs` and
  `src/tests/` as the place.

## Why it matters

A reader looking for a crate's tests checks three places, and each new
test picks whichever its neighbour used.

## What pays it back

Move the sibling files under `src/tests/` crate by crate (mechanical,
names kept), then the inline modules. Agree the rule in `skein/docs/design/testing/README.md`, 6,
first if the module-level `tests.rs` exception is to stay.
