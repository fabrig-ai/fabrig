+++
id = "public-wire"
part = "jig-public"
kind = "interface"
needs = []
files = ["jig/crates/jig-client-protocol-wire/schema/public-v1.schema", "jig/crates/jig-client-protocol-wire/schema/public-v1.fields", "jig/crates/jig-client-protocol-wire/src/generated/public_v1.rs", "jig/crates/jig-client-protocol-wire/src/public.rs", "jig/crates/jig-client-protocol-wire/src/lib.rs", "jig/crates/jig-client-protocol-wire/src/tests/public.rs", "jig/crates/jig-client-protocol-wire/src/tests/public/**", "jig/crates/jig-client-protocol-wire/golden/public/**", "jig/crates/jig-client-protocol-wire/Cargo.toml", "jig/tests/codec-families.rs"]
effort = "high"
minutes = 120
+++

# jig's public records: a schema of their own, the golden of their reachable shape, their bounds

## Goal

`jig-client-protocol-wire` gains jig's public records as a schema apart, versioned apart from
the client wire, naming nothing outside it, with the golden of the whole wire shape a visitor
can receive and the tests that make a field's addition visible
(`jig/docs/design/client/domain/contract.md`, section 12;
`jig/docs/design/protocol/client.md`, section 19). Nothing produces or reads them yet.

Acceptance:

- `schema/public-v1.schema`, family `jig-public-wire 1`, holds exactly the records and enums
  of `notes/contracts.md`, section 3 (`PublicProjects`, `PublicBoard`, `PublicGoal`,
  `PublicCard`, `PublicLine`, `PublicCounts`, `PublicApplication`, `PublicExecutor`,
  `PublicPhase`, `PublicHold`), in skein-codegen's syntax as `client-v19.schema` uses it:
  identifiers `u64`, times `u64` seconds, no text field anywhere. `PublicPhase` has one unit
  variant per `jig_client_domain::contract::Phase` variant, in its order; `PublicHold` one
  per kind of hold the client contract's task line names for a held task (contract.md, 5.1),
  never a reason.
- Generated into `src/generated/public_v1.rs` by skein-codegen, declared in `lib.rs` as
  `pub mod public_v1` (beside `v19`, not glob re-exported), with
  `pub const PUBLIC_VERSION: u16 = 1` and the kind numbers `PUBLIC_PROJECTS: u16 = 1`,
  `PUBLIC_BOARD: u16 = 2`, `PUBLIC_CARD: u16 = 3`; registered as a family of its own in
  `jig/tests/codec-families.rs` (root the crate, golden `golden/public`), its goldens written
  by the drift test.
- `src/public.rs`: `pub struct PublicIndexBounds { pub goals: u16, pub ended: u16,
  pub lines: u16, pub depth: u16, pub projects: u32 }` and
  `pub fn public_bounds(bounds: &PublicIndexBounds) -> [u32; 3]`, the largest encoded answer
  of *projects*, *board* and *card* at those bounds (each list at its bound, each option
  present, each integer at its widest encoding), from the generated code's own bounds where
  it gives them; and the generated codec's limits for the lists filled from the same bounds
  (`pub fn public_limits(bounds: &PublicIndexBounds) -> public_v1::Limits`, or the generated
  equivalent's name).
- `schema/public-v1.fields`, the text golden: one line per record field (record, field, type,
  tag, default) and per enum variant (enum, variant, tag, payload), for every record and enum
  reachable from `PublicProjects`, `PublicBoard`, `PublicCard` and `PublicApplication`, in the
  schema's order.
- Step tests in `src/tests/public.rs` (and `src/tests/public/` if they grow):
  - `the_public_schema_names_nothing_from_elsewhere`: every type the schema names is defined
    in it or is a scalar; it names no record of `client-v19.schema` or `client-v19-part1.schema`;
  - `the_fields_golden_is_the_reachable_wire_shape`: the listing made from the schema equals
    `public-v1.fields` byte for byte, and lists every record and enum the four roots reach and
    no other;
  - `a_nested_field_added_changes_the_golden`: a copy of the schema text with one field
    added to `PublicLine` (and, separately, a variant added to `PublicPhase`) gives a listing
    that differs from the golden, so the comparison test would fail;
  - `a_name_from_elsewhere_breaks_the_closure`: a copy naming `MeProject` in a field fails the
    closure check;
  - `no_public_record_carries_text`: no field of the schema is text or bytes, but
    `PublicApplication.record`;
  - `public_bounds_hold_the_largest_answers`: a board, a card and a projects list built at
    exactly the bounds encode within `public_bounds`, and one entry more is refused by the
    codec's limits.
  The listing and the closure check read the schema with skein-codegen's parser
  (`skein/testing/skein-codegen`, a dev-dependency) if its library exposes one, otherwise a
  small reader of the schema's lines in the test module.
- `heavy cargo nextest run -p jig-client-protocol-wire --lib` passes, and the drift test
  (`heavy cargo nextest run -p jig-client-wire-world -E 'test(/drift::/)'`) with the new
  family.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, section 3.
- `jig/docs/design/client/domain/contract.md`, section 12 (the records, their sources, the
  golden and its tests); `jig/docs/design/protocol/client.md`, section 19.
- As they stand: `jig-client-protocol-wire`'s `lib.rs`, `schema/client-v19.schema` (syntax),
  `src/tests.rs`; `jig/tests/codec-families.rs` (`Family`, `check()`); skein-codegen's
  `src/lib.rs`, `parse.rs`, `emit_bounds.rs`, `emit_limits.rs`;
  `jig_client_domain::contract::Phase` (`contract/common.rs`) and the task line's hold.

## Owns and may change

- Owns the files in the header. `lib.rs`: only the new module's declaration, the constants and
  `mod public`; `v19` and its re-export stay as they are (`visibility-wire` edits the v19
  schema at the same time, in other files).
- `jig/tests/codec-families.rs`: one `Family` entry appended; nothing else.
- `Cargo.toml`: a dev-dependency on `skein-codegen` if the tests use its parser; nothing else.
- Pre-approved: if skein-codegen cannot express an empty-payload enum as written, use its form
  for unit variants and say which. If the generated code gives no worst-case size, compute
  `public_bounds` by hand from the encoding the generated code writes, with a test that
  encodes at the bounds as above, and say so. If the client contract's line names no hold kind
  for a held task, `PublicHold` has the one variant `held`, and the report says so.

## Steps

1. The schema; register the family; generate with
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -E 'test(/drift::/)'`;
   the declarations in `lib.rs`.
2. `public.rs`'s bounds; the fields golden; the step tests; run them alone
   (`heavy cargo nextest run -p jig-client-protocol-wire --lib -E 'test(/public/)'`).
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-client-protocol-wire --all-targets -- -D warnings`; the crate's
   tests and the drift test; the citations check.
4. Commit (why: jig's public records as a schema apart with the golden of what a visitor can
   receive, which the component and the clients build on); push your branch.

## Report

As the implementer's text says, and also: the schema as written (its records and enums, in a
few lines), how the listing reads the schema, and `public_bounds`' three values at the
figures proposed for dev.fabrig.ai (`temper/docs/design/draft/first-release.status.md`: 20
live and 20 ended goals, 50 lines, depth 16, 64 projects).
