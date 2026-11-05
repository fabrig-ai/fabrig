# D008: Two memory suites reach their fixtures by `#[path]`

- **Kind:** tests
- **Where:** `tests/builder/tests/suite/memory.rs`, `tests/config/tests/suite/memory.rs`, `tests/builder/src/fixtures/bounds.rs`, `tests/config/src/fixtures/metered.rs`, `tests/config/src/fixtures/memory.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, in smith's `path_attribute` count, which hearth's old forms don't hold
- **Checked:** 2026-11-05 at `e65ce4cfe2`
- **Value:** low (ordinary modules; the tree says what compiles what)
- **Size:** S
- **Status:** open
- **Paid when:** no `#[path]` remains in hearth, or each that does says why.
- **Measure:** `git grep -E '#\[path' -- 'hearth/*.rs' | wc -l`: 3

## What

`tests/builder/tests/suite/memory.rs` compiles `src/fixtures/bounds.rs`,
and `tests/config/tests/suite/memory.rs` `src/fixtures/metered.rs` and
`memory.rs`: files under a library's `src/` that its library never
declares, for a suite that could hold them.

## Why it matters

A reader can't tell from the tree who compiles a file, and a file reached
across a crate's root rebuilds what its own crate doesn't know depends on it.

## What pays it back

Each fixture moves into its suite as a plain module, `suite/memory/`, or
goes with what hearth's memory migration does to these suites. They wait
for it: it holds both files.
