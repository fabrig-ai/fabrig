+++
id = "jig-documents-surface"
part = "documents-jig"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-core/src/settings/charters.rs", "jig/crates/jig-domain-core/src/limits.rs", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-domain-core/src/routing/run/preparation.rs", "jig/crates/jig-domain-tasks/src/limits.rs", "jig/crates/jig-domain-authority/src/limits.rs", "jig/crates/jig-domain-authority/src/lib.rs", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 75
+++

# jig: a charter's documents, the surface: `Document`, `Charter.documents`, the plan's field

## Goal

Give jig's charter its documents' type and fields, and the core's plan its `documents`
field, emitted empty, so the store, the core's bodies, the configuration, the client wire and
temper's root build on them in parallel (jig's `jig/docs/design/domain/engine.md`, 17.9). This
task carries the surface and the forced one-line readers only: no check, no plan body, no
codec, no wire.

Acceptance:

- Everything `temper/docs/plan/configuring-p1/notes/contracts.md`, section 1 names exists, as
  it names it: `Document`, `Charter.documents`, `DOCUMENT_PATH_BYTES`,
  `DOCUMENT_HEADING_BYTES`, `CORE_SECTIONS`, `brief_holds_documents`, `charter_owned_bytes`,
  `DOCUMENT_KIND`, `DOCUMENT_PRIORITY`, `PlannedDocument`, `Now::BriefCorePlanned`'s
  `documents`, `tasks::Limits.charter_documents`, and `jig_domain_authority::name_within`
  public. Each item's doc comment cites `domain/engine.md, 17.9` (module or type level).
- `plan_brief` (`routing/run/preparation.rs`) emits `documents: Box::new([])` and plans no
  document section yet (`jig-documents-core` does).
- Every forced reader compiles with the empty or default value: each `Charter` literal gains
  `documents: Box::new([])`; each `tasks::Limits` literal gains `charter_documents: 2` (Small's
  and Standard's profiles, fixtures alike); each reader of `BriefCorePlanned` either names
  `documents` or uses `..` (temper's `route/now.rs` uses `..` here: `root-document-binding`
  binds them).
- Unit tests in `jig-domain-core` (beside `limits.rs`'s tests, or in `src/tests/`):
  `brief_holds_documents` true at `CORE_SECTIONS + n`, false one below; `charter_owned_bytes`
  of a charter with no documents equals `policy_owned_bytes` of its policy, and grows by each
  document's segments, path and heading bytes.
- Memory figures and pins that move only because `Charter` or `tasks::Limits` grew are updated
  where they are pinned (regenerated with `SKEIN_CODEGEN_WRITE=1` where the test offers it),
  each listed in the report with its old and new value.
- `heavy cargo check --tests` passes for every package named under Owns, and
  `heavy cargo nextest run -p jig-domain-core --lib`, `-p jig-domain-tasks --lib`,
  `-p jig-domain-authority --lib`, `-p jig-protocol-config`, `-p jig-protocol-store` and
  `-p temper-engine-domain --lib` pass.
- The first 17.9 sentence leaves `jig/docs/design/domain/engine.status.md`: delete "17.9:
  `Charter` carries no documents (`jig-domain-core/src/settings/charters.rs`)." and nothing
  else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md` and `notes/contracts.md`, section 1.
- `jig/docs/design/domain/engine.md`, 17.9 (a document; planned with the brief) and 17.1 (a
  charter's value); `jig/docs/design/protocol/configuration.md`, section 6 (a charter's
  documents' limit).
- Old code, for reference: the old jig branch added `Document { connector: u16, name:
  Box<[Box<[u8]>]>, path, heading: Option }` beside `Charter`; this plan types `name` as
  `jig_domain_authority::Name` instead, so the core's check can use `name_within`.

## Owns and may change

- Owns the files in the header.
- Forced readers, each edited only by the field's name and its empty or default value (no
  other line):
  - `Charter` literals: `jig/crates/jig-protocol-config/src/charters.rs` (about :95),
    `jig/crates/jig-protocol-store/src/codec/settings.rs` (`decode_charter`, about :620),
    `jig/crates/jig-protocol-store/src/tests/settings.rs`,
    `jig/crates/jig-protocol-engine-clients/src/translate/settings/change.rs` (about :106),
    `jig/crates/jig-protocol-engine-clients/src/tests/settings.rs`,
    `jig/crates/jig-domain-core/src/tests.rs`, `jig/testing/jig-test-domain/src/tests/fixtures.rs`,
    `jig/testing/jig-test-profile/src/lib.rs`, `jig/tests/engine-clients/src/fixtures/receiving.rs`,
    `temper/crates/temper-service/src/stock.rs` (its one `charter()`).
  - `tasks::Limits` literals: `jig/crates/jig-domain-core/src/tests.rs`,
    `jig/crates/jig-domain-tasks/src/tests/revisions.rs`,
    `jig/crates/jig-protocol-config/src/profile/small.rs` (and `standard.rs` if it spells the
    field), `jig/crates/jig-protocol-config/src/overrides.rs` (its destructure only),
    `jig/crates/jig-protocol-store/src/tests/fixtures.rs`, `jig/testing/jig-test-profile/src/lib.rs`,
    `jig/tests/store/src/root.rs`, `jig/tests/tasks/src/world.rs`,
    `temper/crates/temper-engine-domain-forge/src/tests/reconcile.rs`,
    `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`,
    `temper/tests/engine/domain/src/walking.rs`.
  - `BriefCorePlanned` readers: `temper/crates/temper-engine-domain/src/route/now.rs` (`..`),
    `jig/testing/jig-test-domain/src/tests/link.rs` (the variant it builds gains
    `documents: Box::new([])`).
- Pre-approved: any other `Charter`, `tasks::Limits` or `BriefCorePlanned` site the compiler
  names, in any tree, edited the same way and listed; a memory pin that moves, updated as above.
  If `name_within` cannot be made public without exporting `Limits` paths it uses, export the
  least that compiles and say what.
- Not yours: the store's schema and codec bodies, the core's check and plan bodies, the
  configuration's member and start check, the client wire: their tasks build them.

## Steps

1. The types, constants and functions of contracts.md, section 1; the plan's empty field.
2. The forced readers, then `heavy cargo check -p <each package> --tests` until clean.
3. The unit tests; run them alone, then the lib tests above; update moved pins.
4. The status sentence.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and temper's;
   `heavy cargo clippy -p jig-domain-core -p jig-domain-tasks -p jig-domain-authority
   -p jig-protocol-config -p jig-protocol-store -p jig-protocol-engine-clients
   -p temper-engine-domain -p temper-service --all-targets -- -D warnings`; the focused tests of
   those packages; the citations check.
6. Commit (why: a charter's documents get their type, so their store, check, plan, wire and
   temper's gather build in parallel); push your branch.

## Report

As the implementer's text says, and also: each forced site edited, by file; each moved pin with
its values; how `charter_owned_bytes` counts a `Name`.
