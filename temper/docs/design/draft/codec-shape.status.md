# codec-shape: status

2026-11-18. Where the design lags or departs from [codec-shape.md](codec-shape.md).

- **Design lags:**
  - 3: `skein/docs/foundation/programming-model.md`, 10.2 and 10.3, gives `skein-lib` alone generic items, and keeps generated code to the subset (`skein/docs/design/data/codec/README.md`, 1). `skein/docs/design/data/codec/README.md`, 4, generates `Parts`, constructors, accessors, bounds' arithmetic and a `Path` variant per field.
  - 4: `jig/docs/design/protocol/store/records.md`, 1, translates every record by hand.
  - 6: `skein/docs/design/data/codec/README.md`, 4 and 6, and `skein/docs/design/data/codec/command.md`, 1, name generated modules by version, give `LIMIT_NAMES` and `Limits::values` counts, number the generated functions' helpers, give the generator's command one schema, and pass the lints with no allowance, where a record of more than three booleans now holds them as named fields under clippy's `struct_excessive_bools` expectation; `skein/docs/design/data/codec/README.md`, 9, commits fingerprints beside the codecs.
- **Design departs:**
  - 5.2: a family's memory test decodes the fixed rule's full value (each list with one item, bytes and text of three bytes), as its golden file held, not the largest value the ceilings allow: that reaches about 4×10⁸ bytes for jig's tasks row and 2.6×10⁹ for jig's client wire. A test that builds a value at the ceilings with the generated constructors keeps it (`skein/docs/design/data/codec/README.md`, 6). The goal K1's designer, 2026-11-18.
  - 5.2: the values a family's tests decode are built by `skein-codegen`'s fixed rule from the schema's text (`goldens`), not by `skein-codec`'s generic values, which build a value only through a patch's machine (`skein/docs/design/data/codec/README.md`, 6 and 8). The goal K1's designer, 2026-11-18.
  - 5.2: a family keeps the fuzzing and memory tests it has; one with none gains none, its bytes being its kinds', which `every` pins. The goal K1's designer, 2026-11-18.
  - 5.1: `every` is completed as a third version, `every-v3.schema`, since its second is frozen by the landed patch that names it (`skein/docs/design/data/codec/command.md`, 2); its first two versions stay for the patch story. The goal K1's designer, 2026-11-18.
  - 5.3: `jig-client-view`'s rendered views stay under its `golden/rendered/`, snapshots of their own subject, which 5.3 leaves aside; its codec goldens beside them go. The goal K1's designer, 2026-11-18.
  - 6.1: a `fixed N` field's value stays `[u8; N]`: N is the field's width, a value's shape that only that field's change touches, not a count of what the schema holds. Every other array whose length counts the schema's items goes. Decided by goal K2's designer, 2026-11-18.
  - 6.1: skein's codec fixtures outside `src/generated/` (`skein/tests/codec/src/every/`, `lints/`, `scalars/`, `secrets/`) keep files named by version, since each holds several versions of one family side by side for its stories (`skein/docs/design/data/codec/command.md`, 1). Decided by goal K2's designer, 2026-11-18.
  - 6.2: the merge job runs the generator's command on a family's schema texts (`temper/docs/design/protocol/tooling.md`, 4.1), not the family's drift test: every drift test's world builds the codec crates whose files are left in conflict, so it cannot build until they are settled, while the command builds `skein-codegen` alone. The drift test checks the result at delivery. Decided by goal K2's designer, 2026-11-18.
- **Open:** none.
