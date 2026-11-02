+++
id = "jig-documents-config"
part = "documents-jig"
kind = "feature"
needs = ["jig-documents-surface"]
files = ["jig/crates/jig-protocol-config/src/charters.rs", "jig/crates/jig-protocol-config/src/overrides.rs", "jig/crates/jig-protocol-config/src/profile/small.rs", "jig/crates/jig-protocol-config/src/profile/standard.rs", "jig/crates/jig-protocol-config/src/domain.rs", "jig/crates/jig-protocol-config/src/tests/**", "jig/tests/channel/src/fixtures.rs", "jig/tests/client-protocol/src/native/config.rs", "jig/tests/client-browser/src/configuration.json", "jig/testing/jig-test-service/src/tests.rs", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 75
+++

# jig: the configuration reads a charter's documents; Small's brief holds them

## Goal

An application that lets `jig-protocol-config` read its defaults gets a charter's documents
from its document, `tasks.charter_documents` as a limit with its override, Small's and
Standard's brief sized for the core's own six sections and two documents, and a start refused
when the brief cannot hold them, naming both limits (jig's `jig/docs/design/domain/engine.md`,
17.9; `jig/docs/design/protocol/configuration.md`, section 3's `charters` and section 6).

Acceptance:

- **The member.** A charter's optional `documents` in the document's `engine.charters`, as
  `jig/docs/design/protocol/configuration.md`, section 3 writes it:
  `[{"connector": n, "name": [s, ...], "path": s, "heading": s}]`, `heading` optional, absent
  meaning none, parsed into `Charter.documents` (`charters.rs`). A member out of its form, an
  unknown member of a document, a `connector` that is no number, a `name` that is no list of
  strings, refuse the start at their path, as the charter's other members do. Bounds and the
  connector's registration are the core's one check (`jig-documents-core`), which the start
  runs and maps to the document's path as it does for every setting.
- **The limit.** `tasks.charter_documents` is accepted as an override wherever `tasks.charters`
  is, in the same forms (`overrides.rs`); Small and Standard set it to 2; their brief
  `sections` are 8 (`profile/small.rs`; Standard as it derives from Small).
- **The start check.** `domain.rs`'s `whole` refuses unless
  `jig_domain_core::brief_holds_documents(limits.brief.sections, limits.tasks.charter_documents)`,
  naming `brief.sections` and `tasks.charter_documents` at their override paths, as its other
  bounds name theirs.
- **Fixtures.** The three configuration fixtures setting `"brief.sections": 1`
  (`jig/tests/channel/src/fixtures.rs`, `jig/tests/client-protocol/src/native/config.rs`,
  `jig/tests/client-browser/src/configuration.json`) start again: by default each sets
  `"tasks.charter_documents": 0` and `"brief.sections": 6`, the least the check admits.
- **Figures.** Memory figures and pins that move with Small's brief sections (the brief's
  capacity, journal writes, `jig-test-service`'s value/commit/ops figures, the overrides test's
  `brief.sections` pin) are updated where pinned, regenerated with `SKEIN_CODEGEN_WRITE=1` where
  the test offers it, each with its old and new value in the report.
- Tests in `src/tests/` (`charters.rs`, `overrides.rs`, and the start's tests): a charter with
  two documents, one with a heading, parsed whole; each malformed member refused at its path;
  `tasks.charter_documents` overridden; Small and Standard at 2 and 8; a start with
  `brief.sections` one below `6 + tasks.charter_documents` refused naming both, and one at it
  admitted. `heavy cargo nextest run -p jig-protocol-config` passes, its count in the report;
  then `heavy cargo nextest run -p jig-test-service -p jig-channel-world` and the client-protocol
  world (package names as their `Cargo.toml` give them).
- The configuration's sentence leaves `jig/docs/design/domain/engine.status.md`: delete "17.9:
  the configuration reads no `documents` member and no `tasks.charter_documents`, and Small's
  brief has 4 sections, not 8." and nothing else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md` and `notes/contracts.md`, sections 1 and 2.
- `jig/docs/design/protocol/configuration.md`, section 3 (`charters`), section 6 (a charter's
  documents), section 10 (formats); `jig/docs/design/domain/engine.md`, 17.9 and 17.7 (a start
  over the defaults).
- Old code, for reference: the old jig branch parsed an optional `documents` in
  `charters.rs`, added the start check `brief.sections < 6 + charter_documents` naming both
  limits in `domain.rs`, and accepted `tasks.charter_documents` and its composed
  `core.tasks.charter_documents` override name; its figures are stale.

## Owns and may change

- Owns the files in the header, and the memory-figure pins elsewhere that this task's sizing
  moves (each listed).
- Not yours: `clients.rs`'s wire bounds (`jig-documents-wire` adds the documents' row), the
  core's check, the store.
- Pre-approved: if one of the three fixtures needs documents for its tests, give it
  `"brief.sections": 8` and keep `charter_documents` 2; if Standard spells its own brief
  sections, set them to 8 there; if the composed `core.tasks.*` form is not how `tasks.charters`
  is accepted today, follow how it is.
- The plan beside this one (reference-m1's `settings-wake-charter`) edits `charters.rs` and its
  tests: add the member beside the others; reformat nothing.

## Steps

1. The member; the override; the profiles; the start check.
2. The tests; run them alone, then the crate's; the fixtures; the figures.
3. The status sentence.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-protocol-config --all-targets -- -D warnings`;
   `heavy cargo check -p temper-service --tests`; the focused tests of the crates whose
   fixtures you changed; the citations check.
5. Commit (why: a charter's documents come from the document, within a brief sized for them);
   push your branch.

## Report

As the implementer's text says, and also: each figure moved with its values, and the tests'
count.
