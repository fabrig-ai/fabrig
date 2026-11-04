# D008: Test modules are reached by `#[path]`

- **Kind:** tests
- **Where:** `tests/builder/tests/real.rs`, `tests/builder/tests/real/reserved.rs`, `tests/builder/tests/suite/memory.rs`, `tests/builder/tests/suite/domain.rs`, `tests/config/tests/suite/memory.rs`, `tests/config/tests/suite/fuzzy/goldens.rs`, `tests/template/tests/suite/fuzzy/goldens.rs`, `tests/render/tests/suite/oracle.rs`, `hearth-domain-render/src/tests/packages.rs`, `hearth-protocol-builder/src/tests/translate.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, in smith's `path_attribute` count, which hearth's old forms don't hold
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** low (ordinary modules; the tree says what compiles what)
- **Size:** S
- **Status:** open
- **Paid when:** no `#[path]` remains in hearth, or each that does says why.
- **Measure:** `git grep -E '#\[path' -- 'hearth/*.rs' | wc -l`: 25

## What

- 16 of the 25 are in `tests/builder/tests/real.rs` and `real/reserved.rs`:
  a binary rooted at `real.rs` can't find `real/*.rs` as plain modules.
- `tests/config/tests/suite/fuzzy/goldens.rs` and
  `tests/template/tests/suite/fuzzy/goldens.rs` compile
  `tests/render/src/fixtures/decoders/golden.rs`, across crate roots.
- `tests/builder/tests/suite/memory.rs` compiles `src/fixtures/bounds.rs`,
  and `tests/config/tests/suite/memory.rs` `src/fixtures/metered.rs` and
  `memory.rs`: files under a library's `src/` that its library never
  declares, for a suite that could hold them.

## Why it matters

A reader can't tell from the tree who compiles a file, and a file reached
across a crate's root rebuilds what its own crate doesn't know depends on it.

## What pays it back

`tests/builder/tests/real/main.rs` removes most of them with no other
change. The rest become plain modules, or move into a kit (D009).
