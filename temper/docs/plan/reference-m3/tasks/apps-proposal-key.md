+++
id = "apps-proposal-key"
part = "appsettings"
kind = "feature"
needs = ["apps-proposals", "apps-store", "apps-wire"]
files = ["jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/restart.rs", "jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/crates/jig-domain-core/src/documents/settings_review.rs", "jig/crates/jig-domain-core/src/tests/settings/application_proposals.rs", "jig/crates/jig-protocol-store/schema/settings-v1.schema", "jig/crates/jig-protocol-store/src/generated/settings_v1.rs", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/golden/settings/**", "jig/crates/jig-protocol-store/src/tests/settings.rs", "jig/crates/jig-client-protocol-wire/schema/client-v19.schema", "jig/crates/jig-client-protocol-wire/src/generated/v19.rs", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-protocol-engine-clients/src/translate/settings.rs", "jig/crates/jig-protocol-engine-clients/src/tests/settings.rs", "jig/crates/jig-client-protocol-link/src/translate/settings_proposals.rs", "jig/crates/jig-client-protocol-link/src/tests/settings_proposals.rs", "jig/crates/jig-client-domain/src/contract/settings.rs", "jig/crates/jig-domain-core/src/tests/inbox/inbox_selection.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "temper/crates/temper-client-tools/src/tests/jig/settings.rs"]
effort = "high"
minutes = 90
+++

# apps-proposal-key: a deployment-unique key for a proposal of an application's settings

## Goal

A settings proposal's number is unique only per proposer (the core keeps sets by
`(people::Proposer, u32)`), so a connector keeping a proposal's items under that number
could mix two proposers' sets. The core gives each settings proposal that names an
application's items a deployment-unique `u64` **key** as it is made, from a counter in its
settings state, stored and restored with the set; every ask that names a proposal to a
connector sends the key; and the settings review document and its client-wire record show
it, so a client can ask the application for the proposal's items by it.

This task **revises released specs** (they stay as written; this task changes their code):

- `apps-proposals`: its option "send the connector the number the tasks child gave the
  proposal as it is (`u64::from(number)`)" is replaced: the `proposal` of
  `KeepProposalSettings`, `DropProposalSettings`, `JudgeSettings` and `KeepSettings` is the
  key from now on.
- `apps-store`: the settings family's `Proposal` record and its extent row gain the fields
  below (settings-v1, in place).
- `apps-wire`: the client wire's `SettingsReview` record gains the key (client-v19, in
  place).
- contracts.md, C2: `proposal: u64` in those asks means this key (the lead's note).

Acceptance:

- Two proposers' pending sets with the same number, each naming an application's items,
  reach their connector under two distinct keys; a restart keeps each set's key and the
  counter, and a set made after it never reuses a key, not even once every set before it
  was decided.
- A proposal naming no application item draws no key and sends no ask (as now).
- `SettingsReview` (core, wire and client domain) carries `key: Option<u64>`, `Some` for a
  set naming an application's items.
- The tests below pass; `jig-domain-core`'s, `jig-protocol-store`'s,
  `jig-protocol-engine-clients`' and `jig-client-protocol-link`'s focused tests pass, and
  `jig/tests/core`'s settings-proposal stories and, if `apps-world` has merged, its
  `application_settings` story (a change of meaning of `proposal` in the asks: run its
  readers' tests).

## Inputs

- `jig/docs/design/domain/engine.md`, 17.5 (**Kept in the same decision**), 17.6 (**The
  owner keeps the value**, **Reviewed against its own revision**), 17.7 (**What the store
  keeps**).
- `jig/docs/design/protocol/store.md`, 4.4 (the settings family's row), 5.3 (**The
  levels**: no version moves before the first release).
- `jig/docs/design/client/domain/contract.md`, 5.1 (a pending proposal's review).
- `temper/docs/plan/reference-m3/notes/contracts.md`, C2 (the asks' `proposal`).
- `temper/docs/plan/reference-m3/notes/common.md`.

## Owns and may change

- Owned: the `files` above, as `apps-proposals`, `apps-store` and `apps-wire` left them.
- Neighbour, compiler-forced fields and arms only: every `settings::Record::Extent` and
  `settings::Proposal` literal or match in jig (`jig-test-domain` `src/lib.rs`,
  `jig/tests/core`'s settings suites and `src/conformance/naming.rs`, the core's
  `tests/start/settings.rs`) and temper's `temper-engine-domain/src/store/settings_bytes.rs`
  (the field alone, the stated cross-tree allowance); `jig/testing/jig-client-test-records`
  and `jig/testing/jig-client-scripted-engine` where they build a `SettingsReview`.
- **Versions:** `settings-v1` and `client-v19` change in place, generated code and goldens
  made again for the changed records only (store.md, 5.3; `protocol/client.md`, section 9).
  If a test or layout check requires a version to move, ask.
- Options, pre-approved:
  - The counter lives in the settings extent row (default: `Record::Extent { oldest,
    keys }`, `keys` the next key, from 1), written in each decision that draws a key, as
    in a revision's commit. If the extent row cannot be written outside a revision's
    commit within the settings room, keep the counter in its own settings record under a
    new `Key` placed after `Extent`, and say so.
  - If the client domain's review type is shared with jig's own kinds' proposals, the
    field is `None` there (default); no other client change.

## Steps

1. `settings.rs`: `Proposal` gains `key: Option<u64>`; `Current` gains the counter
   (`pub(crate) keys: u64`, the next key, 1 at a first start); the extent record carries
   it (option above).
2. `dispatch.rs`: in the decision that keeps a set naming an application's items, draw
   `key = keys`, advance the counter (refuse the proposal with
   `Refusal::Limit { which: LimitKind::Arithmetic }` on overflow), store both, and send
   `KeepProposalSettings { proposal: key, .. }`; `DropProposalSettings`, and an acceptance's
   `JudgeSettings`/`KeepSettings`, send the set's key. A set without a key sends nothing to a
   connector.
3. `restart.rs`: restore the counter and each set's key; a set's key at or past the
   restored counter is a link that cannot be made: `panic!` naming the proposal, as
   engine.md, 17.7 has for a revision with no record.
4. `documents/settings_review.rs`: `SettingsReview` gains `key: Option<u64>`, the set's.
5. Store (`settings-v1.schema`, codec): the `Proposal` record's `key: option u64`; the
   extent's counter. Wire (`client-v19.schema`, engine translation, link, client domain):
   `SettingsReview`'s `key: option u64`.
6. Tests: in `src/tests/settings/application_proposals.rs`, two proposers' sets with one
   number each get distinct keys in their `KeepProposalSettings`; a drop and an
   acceptance send the set's key; after a restart (the module's restart helper) a new set
   gets the next key, also after every earlier set was decided; a set naming only jig's
   kinds has no key. The store's round trip of a keyed `Proposal` and the extent; the
   wire's of a keyed review; goldens remade for those records.
7. Narrow tests: `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/settings/)'`,
   `-p jig-protocol-store --lib`, `-p jig-protocol-engine-clients --lib`,
   `-p jig-client-protocol-link --lib`; `heavy cargo nextest run -p jig-core-world --test
   suite -E 'test(/settings_proposal|application_settings/)'` (check the counts).
   `cargo fmt`; `heavy cargo clippy --workspace --all-targets`. Commit and push.

## Report

Where the counter lives (the option taken); the records and goldens changed; whether
`apps-world`'s story ran.

## Answers

2026-11-03. **Q:** may this task change `jig-domain-core`'s `settings/application.rs`, which
builds the `JudgeSettings`, `KeepProposalSettings`, `DropProposalSettings` and `KeepSettings`
asks?

**A:** Yes; it is now in `files`, for the key's propagation alone: `proposal_number`,
`keep_proposal`, `drop_proposal` and the `start`/`keep` callers' proposal argument take and
send the set's `u64` key (`keep_proposal` given the drawn key by `dispatch.rs`; an
acceptance passes the set's key, a set without one sends no ask). No other behaviour
changes there.

2026-11-03. **Q:** may this task update `inbox_selection.rs`'s fixed memory expectation and
the `settings` row of the store's generated `layouts.rs`, both outside `files`?

**A:** Yes; both are in `files` now, for these repairs alone. Both follow from this task's
own fields, and its acceptance (`jig-domain-core`'s and `jig-protocol-store`'s focused
tests) cannot hold without them.

- `jig/crates/jig-domain-core/src/tests/inbox/inbox_selection.rs`:
  `inbox_context_memory_prices_every_reserved_container_and_complete_document_envelope`
  adds `SettingsReview`'s 16 bytes for `key: Option<u64>` to its expected sum, with one
  comment line saying so. The pricing code stays as it is.
- `jig/crates/jig-protocol-store/src/generated/layouts.rs`: the `settings` row takes the
  changed settings-v1 schema's fingerprint, written by the generator the file's header
  names (`SKEIN_CODEGEN_WRITE=1`) if it runs, otherwise by hand from the fingerprint the
  format test computes; `tests::format::current_layouts_are_the_eight_distinct_runtime_schema_fingerprints`
  checks it. No other row and no version moves.
- Also `temper/crates/temper-client-tools/src/tests/jig/settings.rs`: the three
  `SettingsReview` literals the run left (`:1134`, `:1198`, `:1518`) take `key: None`, the
  field alone, a cross-tree allowance like `settings_bytes.rs`'s; the workspace Clippy in
  step 7 must build.

Then run the two failing tests once each, the four packages' `--lib` runs whole, and
step 7's workspace Clippy.
