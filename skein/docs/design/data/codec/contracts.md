# Contracts: generated files that merge themselves

2026-11-18. Goal K2 of `temper/docs/design/draft/codec-shape.execution.md`:
the shapes its changes share while it runs
(`temper/docs/design/draft/design-economy.md`, 4.3). The draft's section 6
is the design; this document goes with the goal's last change.

## 1. A generated module's file

- **`src/generated/<stem>.rs`,** in the crate that holds the family's
  records, for every family a build stores, sends or embeds. `<stem>` is
  the family's current schema file's name less its `-v<N>.schema`, each
  `-` an `_`: `tasks-v13.schema` makes `tasks.rs`, `worker-wire-v17.schema`
  `worker_wire.rs`, `store-v25.schema` `store.rs`.
- **A split schema** (`client-v33.schema` with `client-v33-part1.schema`)
  makes one module, named for its first part: `client.rs`.
- **Unique within a crate:** two families of one crate never share a
  stem.
- **The Rust module keeps its name.** The crate includes the file by
  `#[path]` under the name its hand-written code already uses
  (`#[path = "generated/tasks.rs"] pub mod tasks_v13;`); renaming a
  module is its owner's choice, and none is asked here.
- **Not covered:** skein's codec fixtures outside `src/generated/`
  (`skein/tests/codec/src/every/`, `lints/`, `scalars/`, `secrets/`),
  which hold several versions of one family side by side
  (`command.md`, 1), keep their names. The glue files
  `src/generated/layouts.rs` are no family's module.

## 2. The generator's command

```
skein-codegen <schema>... <out-dir>
```

- **Its texts:** the schemas' texts concatenated in the order given,
  nothing between them, as jig's drift check reads a split family's
  parts (`jig/tests/codec-families.rs`, `check`), parsed as one schema.
- **Its output:** `generate`'s records' module, written to
  `<out-dir>/<stem>.rs`, `<stem>` the first schema's (1); `<out-dir>` is
  made where absent. Nothing else is written.
- **Its failure:** a schema that does not read or parse exits nonzero,
  saying why on stderr, and writes nothing.
- **`patch`** as the first argument stays the patch command
  (`command.md`, 8).
- **The same bytes as the drift test:** the command's module for a
  family's schemas is the module that family's drift test writes.
