+++
id = "forge-public-wire"
part = "temper-public"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-protocol-wire/schema/public-v1.schema", "temper/crates/temper-client-protocol-wire/schema/public-v1.fields", "temper/crates/temper-client-protocol-wire/src/generated/public_v1.rs", "temper/crates/temper-client-protocol-wire/src/public.rs", "temper/crates/temper-client-protocol-wire/src/lib.rs", "temper/crates/temper-client-protocol-wire/src/tests/public.rs", "temper/crates/temper-client-protocol-wire/src/tests/public/**", "temper/crates/temper-client-protocol-wire/golden/public/**", "temper/crates/temper-client-protocol-wire/Cargo.toml", "temper/tests/codecs/tests/suite/drift.rs"]
effort = "high"
minutes = 90
+++

# The forge's public records: a schema of their own, the golden of their reachable shape, their bounds

## Goal

`temper-client-protocol-wire` gains the forge's public records, *change*, *landings* and
*landing*, as a schema apart from the panels', naming nothing outside it, with the golden of
the whole wire shape a visitor can receive and its bounds, so the forge's projections and the
web client's public pages build on them at once (`temper/docs/design/client/forge.md`, 11.1,
11.3 and 11.4; `notes/contracts.md`, section 5). Nothing produces or reads them yet.

Acceptance:

- `schema/public-v1.schema`, family `temper-public-wire 1`, in skein-codegen's syntax as
  `client-wire-v1.schema` uses it, holding `PublicChange`, `PublicLandings`, `PublicLanding`
  and their parts with exactly forge.md 11.1's fields and 11.3's sources:
  - a task (the change's, a member's, a queue entry's and its goal) is a `u16`, its place in
    jig's `PublicApplication.tasks`, never a number;
  - an address (a repository's, a pull's, a CI run's) is `bytes` within `ADDRESS_BYTES`, as the
    panels' `Pull` and `Link` carry one; a commit as the panels carry one;
  - a step, a repair's reason's kind, a hold's kind, a phase, CI's state, a way and a proof's
    state are enums of unit variants (or a variant whose payload is a gate's number for a
    repair by a gate), mirroring the forge's own kinds, never a reason, words, a name or text;
  - a gate is its number, kind, whether it blocks and its verdict (or, in a landing, its
    proof's state and the run's address once run); counts are `u32`; times `u64` seconds;
  - a queue is its repository's address and its changes in order, each its task, its goal and
    when it became ready: no place and no tasks ahead;
  - no field of a branch, a name, a question, a reason or a tier's text, of any type.
- Generated into `src/generated/public_v1.rs`, declared in `lib.rs` as `pub mod public_v1`
  beside `v1`, with `pub const PUBLIC_VERSION: u16 = 1` and the kind numbers
  `PUBLIC_CHANGE: u16 = 1`, `PUBLIC_LANDINGS: u16 = 2`, `PUBLIC_LANDING: u16 = 3`; its goldens
  under `golden/public/`, written by `temper/tests/codecs`' drift test.
- `src/public.rs`: `pub fn public_bounds(bounds: &Bounds) -> [u32; 3]`, the largest encoded
  answer of each kind at the panels' bounds (`Bounds`, `limits`), each list within its panel's
  bound, and the generated codec's limits filled from them
  (`pub fn public_limits(bounds: &Bounds) -> Option<public_v1::Limits>`, as `limits` is).
- `schema/public-v1.fields`, the text golden: one line per field (record, field, type, tag,
  default) and per enum variant (enum, variant, tag, payload), for every record and enum
  reachable from the three answers, in the schema's order.
- Step tests in `src/tests/public.rs`:
  - `the_public_schema_names_nothing_from_elsewhere`: every type it names is its own or a
    scalar, and none is a record of `client-wire-v1.schema`;
  - `the_fields_golden_is_the_reachable_wire_shape`: the listing made from the schema equals
    `public-v1.fields` byte for byte;
  - `a_nested_field_added_changes_the_golden`: a copy of the schema text with a field added to
    a landing's member (and, separately, a variant added to the step) differs from the golden;
  - `a_name_from_elsewhere_breaks_the_closure`: a copy naming a panel's record fails;
  - `only_addresses_are_bytes`: every `bytes` or text field of the schema is an address;
  - `public_bounds_hold_the_largest_answers`: each kind built at its bounds encodes within
    `public_bounds`, and one entry more is refused by the codec's limits.
  The listing and closure checks read the schema with skein-codegen's `parse` (as the drift
  test does), a dev-dependency.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, section 5.
- `temper/docs/design/client/forge.md`, 11.1 (the kinds and their fields), 11.2 (what is
  never given: places, tasks ahead), 11.3 (what crosses, and the source table) and 11.4 (the
  schema and golden).
- As they stand: `temper-client-protocol-wire`'s `lib.rs` (`Bounds`, `limits`,
  `ADDRESS_BYTES`, `GATES`), `schema/client-wire-v1.schema` (the panels' `Pull`, `Link`,
  `Receipted`, the change's step and the landing's records, for the kinds to mirror),
  `src/tests.rs`; `temper/tests/codecs/tests/suite/drift.rs` (`schema_matches`).

## Owns and may change

- Owns the files in the header. `lib.rs`: the module's declaration, the constants and
  `mod public` alone.
- `temper/tests/codecs/tests/suite/drift.rs`: `schema_matches` writes a family's code to
  `src/generated/{version}.rs` and its goldens to `golden/`; generalize it only as far as a
  family named `public` needs (its code at `src/generated/public_v1.rs`, its goldens at
  `golden/public/`), and add the one call for it; the existing families' paths stay as they
  are.
- Pre-approved: where a forge kind (a step, a hold) carries words in a payload, the public
  enum's variant drops the payload, and the report lists each such variant. If a commit is
  carried by the panels as an address rather than a hash, carry the hash as the panels'
  smallest form and say which.

## Steps

1. The schema; the drift test's call; generate with
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`;
   the declarations.
2. `public.rs`'s bounds; the fields golden; the step tests; run them alone
   (`heavy cargo nextest run -p temper-client-protocol-wire --lib -E 'test(/public/)'`).
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-client-protocol-wire -p temper-codecs-world --all-targets -- -D warnings`;
   the crate's tests and the drift test; the citations check.
4. Commit (why: the forge's public records as a schema apart, with the golden of what a
   visitor can receive, which the forge's projections and the public pages build on); push
   your branch.

## Report

As the implementer's text says, and also: the schema's records in a few lines, every enum
variant whose payload you dropped, and `public_bounds`' three values at the panels' default
bounds.
