# D005: hex, decimal and octal digits are written by hand across the crates

- **Kind:** duplication
- **Where:** `hearth-client-protocol-records/src/form/text.rs`, `hearth-client-protocol-records/src/form/json.rs`, `hearth-client-protocol-records/src/parts/config.rs`, `hearth-client-protocol-records/src/parts/secrets.rs`, `hearth-protocol-builder/src/cache.rs`, `hearth-protocol-builder/src/store/names.rs`, `hearth-protocol-mmdebstrap/src/packages.rs`, `hearth-protocol-git/src/listing.rs`, `hearth-domain-render/src/kinds/software.rs`, `hearth-domain-render/src/kinds/files.rs`, `hearth-domain-render/src/kinds/secrets.rs`, `hearth-lib/src/tests/fixtures.rs`, `hearth-protocol-tree/src/tests/fixtures.rs`, `hearth-domain-render/src/tests/fixtures/digests.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, in skein's `hex_decimal_fn` form (`skein/docs/design/draft/component-kit.md`, 4), which hearth's old forms don't count
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** low (fewer lines; one tested digit rule)
- **Size:** S
- **Status:** open
- **Paid when:** step code calls `bytes::hex_digit`, `bytes::hex_value` and `bytes::decimal` where they fit, a mode's digits are written and read in one place each, and the fixtures' `hex` is one helper.
- **Measure:** `git grep -E '\bfn\s+(hex|decimal)\b' -- 'hearth/*.rs' | wc -l`: 15

## What

skein-lib has `bytes::hex_digit`, `bytes::hex_value` and `bytes::decimal`;
no hearth file calls them.

- A hex digit is written inline in `form/text.rs`, `form/json.rs`,
  `cache.rs` and `store/names.rs`, and read in `store/names.rs`,
  `kinds/software.rs` and `packages.rs`.
- `packages.rs`'s `decimal` is `bytes::decimal`. `listing.rs` has the same
  loop inline.
- A mode's four octal digits are written in three places in the records and
  read in two kinds of render.
- `hex(text: &str)` is one fixture of over 20 lines, copied into three step
  crates' tests.

Of the 15 counted, `Oid::hex`, fake git's `hex`, `Lines::decimal`, which
calls `skein_lib::Decimal`, and the worlds' references are not copies of
lib's: a reference stays independent on purpose. About 8 should remain.

## Why it matters

Each copy is small, but each is a place for a digit bug, and new crates
copy what they see.

## What pays it back

Call lib's. Give the mode's digits one function in each direction, and the
fixture one home each crate's tests can reach. Add `hex_decimal_fn` to
hearth's ratchet (D011).
