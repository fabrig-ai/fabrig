+++
id = "jig-documents-store"
part = "documents-jig"
kind = "feature"
needs = ["jig-documents-surface"]
files = ["jig/crates/jig-protocol-store/schema/settings-v1.schema", "jig/crates/jig-protocol-store/src/generated/settings_v1.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/golden/settings/**", "jig/crates/jig-protocol-store/src/tests/settings.rs", "jig/tests/store-codecs/**", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 75
+++

# jig: the settings store keeps a charter's documents

## Goal

A charter's documents are kept with the charter in the settings family, in every record that
holds a charter's value (its default, its change, its version, a revision's items, a
proposal's items), so a restart and an upgrade find them as they were (jig's
`jig/docs/design/domain/engine.md`, 17.9 and 17.7; `jig/docs/design/protocol/store.md`, 4.4).
The schema changes in place: no new store version (`store.md`, before the baseline).

Acceptance:

- `settings-v1.schema` gains `record Document` and `Charter.documents` as
  `temper/docs/plan/configuring-p1/notes/contracts.md`, section 2 gives them, with the tightest
  ceilings the codec's limits check admits; the generated code, `layouts.rs` and the settings
  goldens are regenerated with
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`,
  then the same run without the variable passes. Every golden that moved is read and listed.
- `encode_charter`/`decode_charter` (`codec/settings.rs`) carry `documents` both ways
  (`Document.name` as its segments); the codec's `limits()` bound them from
  `tasks.charter_documents`, the authority's `segments` and `segment_bytes`, and the path's and
  heading's bytes (`DOCUMENT_PATH_BYTES`, `DOCUMENT_HEADING_BYTES`). A decoded document past a
  bound is refused as the codec refuses any record past its limits.
- Round-trip tests in `src/tests/settings.rs`: a charter with two documents, one with a heading
  and one without, each at its bounds (a name of the authority's most segments, each of its most
  bytes; a 256-byte path; an 80-byte heading), through a default, a change, a charter version
  and a proposal's item; one past each bound refused.
  `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(/settings/)'` passes, its count
  in the report; then `heavy cargo nextest run -p jig-protocol-store` and
  `-p jig-store-codecs-world` (its `drift` and `memory` suites; a memory figure that moves is
  updated where it is pinned, with its old and new value in the report).
- The store's sentence leaves `jig/docs/design/domain/engine.status.md`: delete "17.9: the
  settings store keeps no documents." and nothing else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md` and `notes/contracts.md`, sections 1 and 2.
- `jig/docs/design/domain/engine.md`, 17.7 (in the store) and 17.9;
  `jig/docs/design/protocol/store.md`, 4.4 (the settings family) and its words on changing a
  schema in place before the baseline.
- Old code, for reference: the old jig branch added `record Document { connector: u16 name:
  list 8 Segment path: bytes 256 heading: option bytes 80 }` with `Segment` moved above it, and
  `documents: list 32 Document` on `Charter`; its ceilings were wider than they needed to be,
  which inflated the codec's decoded bounds: keep yours tight.

## Owns and may change

- Owns the files in the header.
- Readers of the generated settings types elsewhere: compiler-forced edits only, listed.
- Not yours: `people-v14` (a settings proposal keeps only a digest there), the client wire.
- The plan beside this one (reference-m1's `settings-wake-charter`) edits this schema and its
  generated files too: change only the `Charter` record, the new `Document` record and the
  codec's charter functions and limits; never reorder the schema.

## Steps

1. The schema; regenerate; the codec both ways and its limits.
2. The round-trip tests; run them alone, then the crate's tests and the store-codecs world.
3. The status sentence.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-protocol-store -p jig-store-codecs-world --all-targets -- -D warnings`;
   `heavy cargo check -p jig-domain-core -p temper-protocol-store --tests`; the citations check.
5. Commit (why: a charter's documents survive a restart with the charter's version); push your
   branch.

## Report

As the implementer's text says, and also: the ceilings chosen, the goldens and figures that
moved, and the tests' count.
