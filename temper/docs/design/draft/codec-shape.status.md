# codec-shape: status

2026-11-18. Where the design lags or departs from [codec-shape.md](codec-shape.md).

- **Design lags:**
  - 3: `skein/docs/foundation/programming-model.md`, 10.2 and 10.3, gives `skein-lib` alone generic items, and keeps generated code to the subset (`skein/docs/design/data/codec/README.md`, 1). `skein/docs/design/data/codec/README.md`, 4, generates `Parts`, constructors, accessors, bounds' arithmetic and a `Path` variant per field.
  - 4: `jig/docs/design/protocol/store/records.md`, 1, translates every record by hand.
  - 5: `skein/docs/design/data/codec/README.md`, 6, keeps golden values and generated tests for every family.
  - 6: `skein/docs/design/data/codec/README.md`, 4 and 6, and `skein/docs/design/data/codec/command.md`, 1, name generated modules by version, give `LIMIT_NAMES` and `Limits::values` counts, number the generated functions' helpers, give the generator's command one schema, and pass the lints with no allowance, where a record of more than three booleans now holds them as named fields under clippy's `struct_excessive_bools` expectation; `skein/docs/design/data/codec/README.md`, 9, commits fingerprints beside the codecs.
- **Design departs:**
  - 6.1: a `fixed N` field's value stays `[u8; N]`: N is the field's width, a value's shape that only that field's change touches, not a count of what the schema holds. Every other array whose length counts the schema's items goes. Decided by goal K2's designer, 2026-11-18.
  - 6.1: skein's codec fixtures outside `src/generated/` (`skein/tests/codec/src/every/`, `lints/`, `scalars/`, `secrets/`) keep files named by version, since each holds several versions of one family side by side for its stories (`skein/docs/design/data/codec/command.md`, 1). Decided by goal K2's designer, 2026-11-18.
  - 6.2: the merge job runs the generator's command on a family's schema texts (`temper/docs/design/protocol/tooling.md`, 4.1), not the family's drift test: every drift test's world builds the codec crates whose files are left in conflict, so it cannot build until they are settled, while the command builds `skein-codegen` alone. The drift test checks the result at delivery. Decided by goal K2's designer, 2026-11-18.
- **Open:** none.
