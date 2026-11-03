+++
id = "jig-documents-wire"
part = "documents-jig"
kind = "feature"
needs = ["jig-documents-surface", "jig-documents-config"]
files = ["jig/crates/jig-client-protocol-wire/schema/client-v19-part1.schema", "jig/crates/jig-client-protocol-wire/src/generated/**", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-client-protocol-wire/src/tests/**", "jig/crates/jig-client-domain/src/contract/settings.rs", "jig/crates/jig-client-protocol-link/src/translate/settings.rs", "jig/crates/jig-client-protocol-link/src/limits.rs", "jig/crates/jig-client-protocol-link/src/tests/**", "jig/crates/jig-protocol-engine-clients/src/translate/settings.rs", "jig/crates/jig-protocol-engine-clients/src/translate/settings/change.rs", "jig/crates/jig-protocol-engine-clients/src/tests/settings.rs", "jig/testing/jig-client-scripted-engine/src/codec/values/settings.rs", "jig/testing/jig-client-test-records/src/settings.rs", "jig/tests/client-wire/**", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 90
+++

# jig: the client wire carries a charter's documents

## Goal

A charter's value crosses to clients whole, its documents included, wherever it crosses today:
a setting's value and document, a change's items, a revision's items and a proposal's review
items; so a client reads them and a change of a charter made through a client keeps or sets
them, never dropping them (jig's `jig/docs/design/domain/engine.md`, 17.9 and 17.1;
`jig/docs/design/client/domain/contract.md`, section 6's settings reads). The wire's schema
changes in place, as the client wire's version 19 parts do before the baseline.

Acceptance:

- `client-v19-part1.schema` gains `record CharterDocument` and `Charter.documents`, the contract
  `contract::CharterDocument` and `contract::Charter.documents`, as
  `temper/docs/plan/configuring-p1/notes/contracts.md`, section 2 gives them; the generated code
  and wire goldens are regenerated with
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`,
  then the same run without the variable passes; every golden that moved is read and listed.
- The link side (`jig-client-protocol-link`'s `translate/settings.rs`, `wire_charter` and
  `restore_charter`) and the engine side (`jig-protocol-engine-clients`' `translate/settings.rs`,
  core to wire, and `translate/settings/change.rs`, wire to core, which builds
  `Charter.documents` from the wire's instead of the empty list `jig-documents-surface` left)
  map documents one to one, a name's segments into `jig_domain_authority::Name`. Their bounds:
  the link's limits (`limits.rs`) and `jig-protocol-config/src/clients.rs`'s start rows gain the
  documents' (`tasks.charter_documents` against the wire's ceiling, as `core.settings.alternatives`
  is against `charter_alternatives`).
- The scripted engine's settings codec and the client test records carry them.
- Tests: the engine-clients translate round-trips a charter with two documents (one with a
  heading) core to wire to core, and a `ChangeSettings` item setting a charter keeps its
  documents (`src/tests/settings.rs`); the link translate round-trips them wire to contract to
  wire; the client-wire world's drift and fuzzy suites pass.
  `heavy cargo nextest run -p jig-protocol-engine-clients -E 'test(/settings/)'` passes, its
  count in the report; then `heavy cargo nextest run -p jig-client-protocol-wire
  -p jig-client-protocol-link -p jig-protocol-engine-clients -p jig-client-wire-world
  -p jig-engine-clients-world`.
- The wire's sentence leaves `jig/docs/design/domain/engine.status.md`: delete "17.9: the client
  wire's charter carries no documents." and nothing else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md` and `notes/contracts.md`, sections 1 and 2.
- `jig/docs/design/domain/engine.md`, 17.9 and 17.2 (a change's items);
  `jig/docs/design/client/domain/contract.md`, section 6 (the settings reads and their bounds).

## Owns and may change

- Owns the files in the header.
- Neighbour: `jig/crates/jig-protocol-config/src/clients.rs`, the documents' bound row alone.
- Forced readers in temper, each edited only by the field (`documents` with an empty list, or
  passed through where the reader copies a charter whole):
  `temper/crates/temper-settings-form/src/wire.rs` (`settings_restore_charter`,
  `settings_wire_charter`), `temper/crates/temper-client-tools/src/jig/settings_adapter.rs`
  (`form_charter`, `client_charter`), `temper/crates/temper-client-tools/src/tests/jig/settings.rs`
  (its `c::Charter` literal). temper's settings file and its rendering of documents are
  `settings-file-documents`'.
- Pre-approved: any other reader of the wire or contract `Charter` the compiler names, in either
  tree, edited the same way and listed; a wire or memory figure that moves, updated where pinned,
  with its values.
- The plan beside this one (reference-m1's `settings-wake-clients`) edits this schema, its
  generated code, goldens and both translates too: change only the `Charter` record, the new
  `CharterDocument` record and the charter's translate functions; never reorder the schema.

## Steps

1. The schema; regenerate; the contract; both translates; the bounds.
2. The scripted engine and test records; the forced temper readers.
3. The tests; run them alone, then the packages above.
4. The status sentence.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and temper's;
   `heavy cargo clippy -p jig-client-protocol-wire -p jig-client-domain -p jig-client-protocol-link
   -p jig-protocol-engine-clients -p jig-client-scripted-engine -p jig-client-test-records
   -p jig-protocol-config -p temper-settings-form -p temper-client-tools --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-settings-form -p temper-client-tools --lib`; the citations
   check.
6. Commit (why: a charter's documents reach clients, and a change made by one keeps them); push
   your branch.

## Report

As the implementer's text says, and also: the goldens and figures that moved, each forced temper
site, and the tests' count.

## Rebase (2026-11-03, step merge, foreman configuring-p1/jig-documents-wire/foreman-1)

resolve engine.status.md by preserving both completed-gap deletions.
The merge conflict is in `jig/docs/design/domain/engine.status.md`.
Task commit `f8854a588d` deletes “17.9: the client wire's charter carries no documents.”
Integration commit `3fb9bc592ab27b6ef87dcf1ef74ff6cca88939b1` deletes the adjacent “17.9: the settings store keeps no documents.”
Resolve the conflict with both sentences absent, preserving every other status entry and the date.
Keep the task's charter wire changes; `jig/crates/jig-client-protocol-wire/src/generated/v19.rs` auto-merged without a reported conflict.
No additional feature changes are needed for this refusal.
