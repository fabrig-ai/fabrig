# Contracts: lean records on jig's tasks store

2026-11-18. Goal K3 of `temper/docs/design/draft/codec-shape.execution.md`:
the shapes its changes share while it runs
(`temper/docs/design/draft/design-economy.md`, 4.3). The design is
`README.md`, sections 1, 2, 4 and 6, from the draft's section 3; this
document goes with the goal's last change.

## 1. Two forms while families move

- **Lean or sealed.** Until every family is lean (the draft's K4), the
  generator writes either form: lean records (`README.md`, 4), or the
  sealed form every other family has now (private fields, `Parts`,
  constructors, accessors, `into_parts`, a `Path` enum, bound
  arithmetic).
- **The owner's mark:** a file `lean` in the owner's `schema/`
  directory, beside `registered`, lists the families generated lean,
  one family name a line as its schema's `family` line spells it
  (`jig-store-tasks`), `#` starting a comment. No file, or a family it
  doesn't list, is sealed.
- **The generator's surface:**

  ```rust
  // skein_codegen; Form derives Clone, Copy, Debug, equality
  pub enum Form { Sealed, Lean }
  // the form's own checks on a parsed schema (README.md, 7):
  // Lean refuses a type named Path, Problem or Record (parse already
  // refuses Path and Problem for both forms); Sealed adds none
  pub fn check_as(schema: &Schema, form: Form) -> Result<(), Error>;
  // generate_as expects check_as(schema, form) to have passed
  pub fn generate_as(schema: &Schema, form: Form) -> Output;
  // generate(schema) is generate_as(schema, Form::Sealed)
  // Lean where schema_dir/lean lists family; an unreadable file is Err
  pub fn form_of(schema_dir: &std::path::Path, family: &str)
      -> std::io::Result<Form>;
  ```

- **The command** (`README.md`, 4) writes the form `form_of` gives for
  its first schema's directory and the parsed family, after `check_as`
  for that form passes (a refusal is reported as `parse`'s are), so the
  merge job
  (`temper/docs/design/protocol/tooling.md`, 4.1) and the drift test
  write the same module. Its arguments don't change.
- **Each drift test** generates its families in the form their mark
  gives. skein's `every` fixture writes its current version both ways
  (2).

## 2. skein's pin, both ways

- **`every-v3` lean:** `skein/tests/codec/src/every/v3_lean.rs`, Rust
  module `every_v3_lean`, generated lean from `every-v3.schema` beside
  the sealed `every_v3`, both checked by the fixture's drift test.
- **The same bytes, bounds and refusals:** every golden of `every-v3`
  decodes, measures and encodes alike through both modules; each
  declaration's encoded and decoded bounds are equal, at the ceilings
  and at small limits; and a refusal names the same path, by name, with
  the same reason.

## 3. A lean module's surface

What hand-written code reads of a lean family's module, beside
`README.md`, 4:

- **Its types** keep their names, fields, Rust field types, variants and
  derives; each field is public. A `boxed` variant keeps its `Box`.
- **`paths`,** a public module of one `skein_codec::Path` constant a
  path, named as the sealed `Path` variant, in capitals:
  `Path::SpecWords` is `paths::SPEC_WORDS`, `Path::RowTail` is
  `paths::ROW_TAIL`, `Path::RowVersion` `paths::ROW_VERSION`,
  `Path::PartKindTag` `paths::PART_KIND_TAG`.
- **`pub fn name(path: Path) -> Option<&'static str>`:** the path's
  name as the sealed `Path::name` gave it; `None` for a path the family
  doesn't have.
- **`pub use skein_codec::{Path, Problem, Record};`** so the module's
  path names them, and a caller brings `Record` into scope from it.
- **Unchanged:** `VERSION`, `Limits` and its fields, `CEILINGS`,
  `LIMIT_NAMES`, `Limits::values`, `check_limits`, and the family's
  `encoded_bound` and `decoded_bound`, but that `check_limits` returns a
  `skein_codec::Problem`.
- **Measuring and encoding take the limits** and refuse with a
  `Problem`; a list is checked by its length, not its capacity.
