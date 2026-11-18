# codec-shape: status

2026-11-17. Where the design lags or departs from [codec-shape.md](codec-shape.md).

- **Design lags:**
  - 3: `skein/docs/foundation/programming-model.md`, 10.2 and 10.3, gives `skein-lib` alone generic items, and keeps generated code to the subset (`skein/docs/design/data/codec/README.md`, 1). `skein/docs/design/data/codec/README.md`, 4, generates `Parts`, constructors, accessors, bounds' arithmetic and a `Path` variant per field.
  - 4: `jig/docs/design/protocol/store/records.md`, 1, translates every record by hand.
  - 5: `skein/docs/design/data/codec/README.md`, 6, keeps golden values and generated tests for every family.
  - 6: generated modules are named by version (`skein/docs/design/data/codec/README.md`, 6); `jig/docs/design/protocol/store/records.md`, 3, commits the fingerprints and writes the `Layouts` row; `AGENTS.md`, 2, says nothing of merging generated files.
- **Design departs:** none.
- **Open:** none.
