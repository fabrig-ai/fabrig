+++
id = "lib-forge"
part = "lib-sites"
kind = "migration"
needs = ["kit-lib", "counts"]
files = ["skein/crates/skein-forge/src/forgejo/id.rs", "skein/crates/skein-forge/src/github/id.rs", "skein/crates/skein-forge/src/marker.rs", "skein/tests/forge/client/src/translate.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 25
+++
# skein-forge and the forge-client world on lib's hex

## Goal

Draft, 3.4: skein-forge writes the nibble-to-digit `hex` three times,
and the forge-client world decodes a digit by hand; they use
`skein_lib::hex`, by `notes/lib.md`.

Acceptance:

- `skein-forge`: `src/forgejo/id.rs:64` (`pub(crate) fn hex`),
  `src/github/id.rs:22` (`fn hex`) and `src/marker.rs:64-67`
  (`pub(crate) fn hex`) go; their callers use `hex::digit`.
- `skein/tests/forge/client/src/translate.rs:47` (`fn hex`, a value)
  goes for `hex::value`.
- `hex_decimal` falls by 4 (`--write`).
- `heavy cargo nextest run -p skein-forge -p skein-forge-client-world -p
  github-conformance` passes (counts named; live-only tests skipped as
  today); clippy clean for `skein-forge` and the world.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/lib.md`.
- `skein_lib::hex` on your base.

## Owns and may change

- The files in the header, `skein/scripts/old-forms.txt`; callers of the
  three `pub(crate) hex`s elsewhere in `skein/crates/skein-forge/src/`
  change only at the call (found with `rg 'hex\(' crates/skein-forge/src`);
  name the files.
- The other copies the review lists in skein-forge (LRU, `Header::new`,
  measure-then-write) are later runs: not yours.

## Steps

1. Each site by `lib.md`; an upper-case digit kept by its default is
   named.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`.
