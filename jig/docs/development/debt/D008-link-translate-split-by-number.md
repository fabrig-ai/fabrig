# D008: The link's translation is split by number and repeats one loop

- **Kind:** duplication
- **Where:** `jig-client-protocol-link/src/translate`, `jig-client-protocol-link/src/tests/fixtures`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Duplicated code": the piece-decoding loop)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T3 short term, A8 F2, F12)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (T3 in the short term; A8 F12)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (smaller files a reader can navigate by meaning)
- **Size:** M
- **Status:** open
- **Paid when:** no file in jig is named `*_partN`, the piece-decoding loop is one helper, and the link's production code holds no test-only encoders.
- **Measure:** `find jig -name '*_part[0-9]*.rs' | wc -l` (29 at base)

## What

- 29 numbered files: 21 in `jig-client-protocol-link/src/translate` (`documents_part0` to `8`, `pages_part0` to `8`, `facts_part0` to `2`), 8 in `jig-client-protocol-link/src/tests/fixtures`;
- the loop that splits a document's length-prefixed pieces, bounds their sum and decodes each row is written 44 times in production (`rg -o 'piece::split\(&mut reader\)' jig/crates jig/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l`);
- 85 `wire_`/`encode_` functions in `jig-client-protocol-link/src/translate` exist for tests.

## Why it matters

Files split by size say nothing about what is in them, and each copy of the loop is a
place for a bound to be checked differently.

## What pays it back

A codec piece reader in skein (jig's step code defines no generic items), a per-record
`to_bytes`, files grouped by record family, and the test encoders moved under `tests/`.
A step towards D007, and paid by it if D007 lands first.
