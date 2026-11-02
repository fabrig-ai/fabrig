+++
id = "jig-documents-core"
part = "documents-jig"
kind = "feature"
needs = ["jig-documents-surface"]
files = ["jig/crates/jig-domain-core/src/settings/check.rs", "jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/routing/run/preparation.rs", "jig/crates/jig-domain-core/src/tests/runs.rs", "jig/crates/jig-domain-core/src/tests/runs/brief_documents.rs", "jig/crates/jig-domain-core/src/tests/settings.rs", "jig/crates/jig-domain-core/src/tests/settings/documents.rs", "jig/crates/jig-domain-core/src/tests/inbox/inbox_selection.rs", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 90
+++

# jig: the core checks a charter's documents and plans them with the brief, at the run's revision

## Goal

The core holds a charter's documents to their bounds and their connector registered, prices
them within a setting's bound, and plans each as an optional connector section of the run's
brief, from the charter version the run is prepared under, so a run reads the documents its
revision's charter names (jig's `jig/docs/design/domain/engine.md`, 17.9, 17.1 *shape*, 17.3).

Acceptance:

- **The check.** A charter's documents are checked wherever a charter's value is: at a start
  over the defaults and current settings, for a change over the settings it would leave, and at
  a restart over stored values (`restore_fit`'s charter arm: at most `tasks.charter_documents`;
  each name within the authority's pattern bounds by `jig_domain_authority::name_within`; a path
  of 1 to `DOCUMENT_PATH_BYTES` bytes; a heading, when present, of 1 to `DOCUMENT_HEADING_BYTES`);
  and each document's connector registered (`check_set`'s charter arm, beside `charter_fault`,
  which has the connectors). Each fails as `Failure { setting: Setting::Charter(n), fault:
  Fault::Shape }`, as any other bound of a charter does. No new fault.
- **The price.** `settings::set_bound` (and whatever prices a charter's value within a
  setting's bound) adds a charter's documents at their bounds: `tasks.charter_documents` of
  them, each at the authority's most segments of its most bytes, a 256-byte path and an 80-byte
  heading. The pinned `SETTINGS_DOCUMENT` figure (`src/tests/inbox/inbox_selection.rs`) and any
  other pin that moves are updated, old and new values in the report.
- **The plan.** `plan_brief` (`routing/run/preparation.rs`), after the core's own sections, for
  each document of `core.charter_of(task)` in order, while `wanted` has room: a
  `brief::Planned::Connector` and a `PlannedDocument` exactly as
  `temper/docs/plan/configuring-p1/notes/contracts.md`, section 1 gives them (connector the
  document's, `DOCUMENT_KIND`, tokens `Token::new(1 + index)`, size 0, limit the brief's
  `read_bytes`, `DOCUMENT_PRIORITY`, not required), in `Now::BriefCorePlanned`'s `documents`. A
  document without room is in neither. A charter with no documents plans as today.
- Tests in `jig-domain-core`:
  - `src/tests/settings/documents.rs` (declared in `src/tests/settings.rs`): a charter with
    documents at their bounds is accepted at a start and by a change; each bound one past is
    refused `Shape` naming the charter (too many documents; a name with one segment too many,
    and one segment one byte too long; a path empty and one of 257 bytes; a heading empty and
    one of 81 bytes); a document of a connector not registered is refused `Shape`; a stored
    charter past a bound is refused at restart (`restore_fit`).
  - `src/tests/runs/brief_documents.rs` (declared in `src/tests/runs.rs`), by the fixtures
    `src/tests/runs/brief_revisions.rs` uses: a run of a charter naming two documents plans two
    optional connector sections after the core's own, and `documents` names them with the same
    tokens; a plan with room for one plans the first alone; **a task made on revision R, whose
    charter's documents a later change replaces with others, plans R's documents at its next
    run, and a task made after the change plans the new ones** (17.3); a conversation, moved at
    its next run, plans the newest's.
  `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/documents/)'` passes, its count in
  the report; then `heavy cargo nextest run -p jig-domain-core --lib` and `-p jig-core-world`.
- The core's sentence leaves `jig/docs/design/domain/engine.status.md`: delete "17.9: the core
  neither checks a charter's documents nor plans their sections." and nothing else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md` and `notes/contracts.md`, section 1.
- `jig/docs/design/domain/engine.md`, 17.9, 17.1 (one check, its faults), 17.3 (`charter_of`;
  a run reads its revision's charter) and section 9 (the brief's plan);
  `jig/docs/design/protocol/configuration.md`, section 6 (a charter's documents priced within a
  charter's part of the worst case).
- Old code, for reference: the old jig branch's `run_preparation::document_plan` pushed, after
  the core's sections, `Planned::Connector { connector, kind: 0, token: index + 1, size: 0,
  limit: read_bytes, priority: 5, required: false }` per document and `.expect`ed room; this
  plan guards room instead (contracts.md). Its check threaded `connectors` through `shape`;
  today's `check_set` already has them, so leave `shape`'s signature alone.

## Owns and may change

- Owns the files in the header.
- Read only: `settings/charters.rs` (`charter_of`, `Document`), `limits.rs`, `vocabulary.rs`
  (`jig-documents-surface`'s), the store and the configuration crates.
- Pre-approved: if `restore_fit` cannot reach the authority's limits, pass them as its other
  limits are passed and say how; if a core-world story (`jig/tests/core`) breaks because a
  fixture's charter now fails a bound, fix the fixture's value, not the bound, and list it.
- The plan beside this one (reference-m1's `settings-wake-charter`) edits `check.rs` too: add
  your arms beside the existing charter checks; reformat nothing.

## Steps

1. The bounds in `restore_fit`, the connector in `check_set`; their tests.
2. The price; update the pins it moves.
3. The plan in `plan_brief`; its tests.
4. The status sentence.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-core-world -p jig-test-domain`;
   `heavy cargo check -p temper-engine-domain --tests`; the citations check.
6. Commit (why: a run's brief plans the documents of the charter version it runs under);
   push your branch.

## Report

As the implementer's text says, and also: each pin moved with its values, and the tests' count.
