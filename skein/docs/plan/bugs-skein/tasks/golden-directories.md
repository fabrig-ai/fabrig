+++
id = "golden-directories"
part = "codec"
kind = "feature"
needs = []
files = ["skein/tests/codec/tests/suite/drift.rs", "skein/tests/codec/tests/suite/patch.rs", "skein/tests/codec/tests/suite/fuzzy/decode.rs", "skein/tests/codec/golden/**", "skein/docs/design/codec.status.md"]
effort = "medium"
minutes = 35
+++

# One codec family to a golden directory

## Goal

`skein/tests/codec/tests/suite/drift.rs` checks five families (`every-v1`, `secrets-v1`, `sample-v1`,
`scalars-v1`, `lints-v1`) against one `golden/v1`, while `every-v2` has `golden/every-v2`. Two
families naming a type alike overwrite each other's goldens, and a golden no family writes is never
seen. Give each family's version its own directory, and check that a directory holds that family's
goldens alone. (Bug 18, its seen half.)

Acceptance:

- `golden/v1` is gone; `golden/every-v1`, `golden/secrets-v1`, `golden/sample-v1`,
  `golden/scalars-v1` and `golden/lints-v1` hold those families' files, each byte as it was
  (`git status` shows renames).
- `drift.rs`'s checks name those directories, and a new test fails on the old layout.
- `skein-codec-tests`' `drift`, `patch` and `fuzzy` suites pass.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/codec.md`, 6 ("Drift", "One family to a golden directory"), 8 ("Goldens").

## Owns and may change

- Owns: `tests/suite/drift.rs`, `skein/tests/codec/golden/**`.
- Readers, one line each, the path only: `tests/suite/patch.rs` (near 347: `golden/every-v1`),
  `tests/suite/fuzzy/decode.rs` (near 76: the family whose goldens it reads; read it to see which).
- `skein/docs/design/codec.status.md`: remove this plan's **Not yet** entry (`notes/common.md`).
- Not changed: `skein-codegen`'s command, which writes `golden/v<N>` for a crate of one family
  (`skein/testing/skein-codegen/src/main.rs`); `skein/examples/ui/view`'s own `golden/v1`;
  `patches/golden/`, a tree of its own.

## Steps

1. In `drift.rs`, gather the six rows into one table and write
   `a_golden_directory_holds_its_family_alone`: for each row, the file names in its directory are
   exactly `generate(&schema).goldens`' names, and no two rows name one directory. Run it against
   the old layout and quote the failure.
2. Split `golden/v1` by the names each family generates, with `git mv`. A name two families share
   is a finding: compare the bytes, copy it to both, and say so in the report.
3. Update the table and the two readers. Every `check(...)` test stays by name.
4. `heavy cargo nextest run -p skein-codec-tests --test suite -E 'test(/^drift::/)'`, then the
   package's focused tests. `cargo fmt`, `heavy cargo clippy -p skein-codec-tests --all-targets --
   -D warnings`.
5. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old layout and which file went where
in counts per family.
