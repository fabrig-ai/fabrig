+++
id = "restore-surface"
part = "restore"
kind = "interface"
needs = ["policy-surface"]
files = ["jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/settings/restart.rs", "jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/crates/jig-domain-core/src/effects/connector.rs", "jig/crates/jig-domain-people/src/boundary.rs", "jig/crates/jig-domain-people/src/refusals.rs", "jig/crates/jig-domain-core/src/tests/settings/replaced.rs", "jig/crates/jig-domain-core/src/tests/settings.rs", "jig/crates/jig-protocol-config/src/profile/small.rs", "jig/crates/jig-protocol-config/src/tests/profile.rs", "jig/testing/jig-test-service/src/tests.rs", "temper/crates/temper-service/src/store.rs", "temper/crates/temper-service/src/tests/startup.rs", "temper/crates/temper-service/src/tests/limits.rs", "temper/crates/temper-service/src/tests/store.rs"]
effort = "high"
minutes = 90
+++

# restore-surface: what each item replaced, recorded; a restore's request and asks, as types

## Goal

A revision's record in the core keeps, for each item, what it replaced (contract B2), filled
for real at each place a revision is made; the connector asks and events of a restore (B3)
and the people boundary's `Restore`, its outcome and its refusal (B4) exist as types, nothing
sending or admitting them yet. No behaviour of a change, proposal, start or read moves.

Acceptance:

- B2's, B3's and B4's types exist with those names, fields and variants, each documented in a
  line citing `jig/docs/design/domain/engine.md`, 17.10 (the asks: also
  `jig/docs/design/domain/connectors.md`, 17).
- Each committed revision's `items` carry `replaced` as the setting was before the change, a
  release's revision's items `Replaced::Default`; each revision's `applications` carry the
  state and size they replaced. The unit tests below show it.
- `heavy cargo check --workspace --all-targets` is clean; the focused tests of
  `jig-domain-people` and `jig-domain-core` pass.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.5 ("History"), 17.7, 17.10.
- `jig/docs/design/domain/connectors.md`, 17 ("What its items replaced, for a restore").
- `jig/docs/design/domain/people.md`, 5.1.
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B2, B3 and B4 (binding names);
  `notes/common.md`.
- Old code, from jig's `configuring-revisions/replaced` (unreachable from your slot, quoted):
  `RevisionItem { setting, edit, replaced }` and `enum Replaced { Value(Box<Value>), Default,
  Removed, NotOffered }`, `Revision.items` of `RevisionItem`; `Current::replaced(setting)` and
  `Current::revision_items(items)` took the state before an edit applied;
  `revision_worst_case` priced two sets and a boxed `Value` per item; `settings/restart.rs`'
  `restored_revision_items` recorded a release's items as `Default` or `NotOffered`. This spec
  takes `Default` for every release item (17.10).

## Owns and may change

- Owned: `jig-domain-core/src/settings.rs` (`Replaced`, `RevisionItem`, `Revision`,
  `Current::replaced`, `revision_bound`'s price); `settings/application.rs` (`Replaced`,
  `RevisionRecorded`, `Restored`, `RestoredEdit`, where `keep` builds a revision's
  applications); `settings/restart.rs` (the release's revision items);
  `routing/settings/dispatch.rs` (`decide_with`, where the revision is built);
  `effects/connector.rs` (`Ask::ForgetReplaced`, `Ask::RestoreSettings`,
  `Event::SettingsRestored`); `jig-domain-people/src/boundary.rs` (`Ask::Restore`,
  `Outcome::Restored`); `jig-domain-people/src/refusals.rs` (`Refusal::Forgotten`); the new
  unit test file `jig-domain-core/src/tests/settings/replaced.rs` and its `mod` line.
- Neighbour files, **compiler-forced arms, fields and literals only**, each as this table
  says, in jig and in temper (temper's the stated cross-tree allowance, each edited in that
  arm or field alone):

| Where | The edit |
|---|---|
| `jig-protocol-store`'s `codec/settings.rs` (`encode_revision`, `decode_revision`, the recorded applications) | encode drops `replaced` (the store's `Item` and `Recorded` unchanged); decode fills `Replaced::NotOffered` and `application::Replaced::NotOffered`, with the line `// The store keeps no replaced state until settings-store.` |
| the core's readers of `Revision.items` (documents, reads, `revision_read`, history copies) | read `item.setting` and `item.edit` as before; build nothing new |
| a match on `jig_domain_people::Ask`, `Outcome` or `Refusal` in the people child, the core's routing, the store's people codecs | `Ask::Restore { .. }` refused as an unknown or unsupported request is refused today where one is refused (name the refusal in the report), else `unreachable!("a restore: restore-request")`; `Outcome::Restored { .. }` and `Refusal::Forgotten { .. }`: `unreachable!("a restore: settings-store")` in `jig-protocol-store`, `unreachable!("a restore: settings-wire")` in `jig-protocol-engine-clients` and the client crates, `unreachable!("a restore: restore-request")` in the core |
| a root's routing of `connector::Ask` (`jig-test-domain` `src/routes.rs`, `jig/examples/ops/domain/src/route.rs`, temper's `temper-engine-domain/src/route/kinds.rs` and `forge_route.rs`) | one arm for the two new asks: `unreachable!("a restore: restore-connector")` in jig, `unreachable!("a restore: restore-forge")` in temper; the core's handler of `connector::Event::SettingsRestored`: `unreachable!("a restore: restore-request")` |

  A `_` arm that now catches a new variant silently: leave it and list it in the report.
- Options, pre-approved:
  - If `Revision`'s worst case (`revision_bound` and the writes bounds that price a record)
    grows, price `replaced` as one more `Value` per item (a whole set's worth beside the
    set), and correct a test's fixed byte price that fails only because the record grew, its
    comment naming the term; a production limit or budget that fails: stop and ask.
  - If a `ProjectPolicy` item reaches a revision's construction (it cannot while
    policy-surface refuses it), give its `replaced` `unreachable!("a project's policy:
    policy-change")`.
  - If `Ask` or `Event` derive a trait the new fields' types lack, derive it on the new types.

## Steps

1. `settings.rs`:
   ```rust
   /// What an item replaced (jig/docs/design/domain/engine.md, 17.10).
   #[derive(Clone, PartialEq, Eq, Hash, Debug)]
   pub enum Replaced { Value(Value), Default, Removed, NotOffered }
   /// A revision's item and what it replaced (engine.md, 17.10).
   #[derive(Clone, PartialEq, Eq, Hash, Debug)]
   pub struct RevisionItem { pub setting: Setting, pub edit: Edit, pub replaced: Replaced }
   ```
   `Revision { head, items: Box<[RevisionItem]>, applications: Box<[application::RevisionRecorded]> }`.
   `impl Current { #[must_use] pub fn replaced(&self, setting: Setting) -> Replaced }`: a
   change with a value, `Value(v.clone())`; a change with none, `Removed`; no change and a
   default, `Default`; neither, `NotOffered` (read with `lookup`).
2. `routing/settings/dispatch.rs`, `decide_with`: the items' `replaced` read from
   `core.current` before `s::apply` runs (the record is built after it today: capture first,
   then build). `settings/restart.rs`: the release's revision's items `Replaced::Default`.
3. `settings/application.rs`: `Replaced`, `RevisionRecorded` as B2;
   `#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct Restored { pub kind: u32, pub scope:
   u32, pub number: u32, pub edit: RestoredEdit }`, `pub enum RestoredEdit { Set { revision:
   u64 }, Remove, Reset }`. Where `keep` builds a revision's applications, each one's
   `replaced` from `current.applications`' state before it is updated: `Follows::Default`,
   `Default`; `Changed { .. }`, `Changed { size: state.size }`; `Removed { .. }`, `Removed`;
   absent, `NotOffered`.
4. `effects/connector.rs`: `Ask::ForgetReplaced { before: u64 }`, `Ask::RestoreSettings {
   owner: Token, proposal: u64, items: Box<[settings::application::Restored]> }`,
   `Event::SettingsRestored { owner: Token, judged: Box<[Judged]> }`, documented from B3.
5. `jig-domain-people`: `Ask::Restore { revision: u32, reason: Box<[u8]> }` (doc: "Propose the
   settings as a revision left them (engine.md, 17.10; people.md, 5.1)."), `Outcome::Restored
   { proposal: u64, to: u32, left_out: Box<[crate::Setting]> }`, `Refusal::Forgotten { oldest:
   u32 }`. Where the people child sizes or prices a request or an outcome (its capacity and
   bounds modules), price the new variants as their nearest kin (`Propose` of settings,
   `Proposed`, `Stale`).
6. The compiler-forced edits of the table, found with `heavy cargo check --workspace
   --all-targets`.
7. Tests, `jig-domain-core/src/tests/settings/replaced.rs`, in the style of `application.rs`
   beside it:
   - `a_revision_records_what_each_item_replaced`: from the fixture's start, a change sets
     charter 1 (a default), removes template 2, and sets a charter number the fixture does
     not offer; a second change sets charter 1 again and resets template 2. The first
     revision's items are `Default`, `Default`, `NotOffered`; the second's `Value(<the first
     change's charter 1>)` and `Removed`.
   - `a_release_revision_records_its_items_as_default`: a restart with a changed default (as
     the restart tests beside it make one) records each item `Replaced::Default`.
   - `an_application_item_records_the_state_it_replaced`: with the test kind of
     `application.rs`, a change sets a value of 10 bytes, a second sets one of 20: the second
     revision's application item is `Changed { size: 10 }`, the first's `Default`.
8. Narrow tests: `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/settings::replaced/)'`
   (check the count), then `-E 'test(/settings/)'`; `heavy cargo nextest run -p
   jig-domain-people --lib`; `heavy cargo check --workspace --all-targets`. Then `cargo fmt`;
   `heavy cargo clippy -p jig-domain-core -p jig-domain-people --all-targets -- -D warnings`,
   and for each package given an arm; the focused tests of `jig-domain-core`,
   `jig-domain-people` and `jig-protocol-store` (whose codec you touched). Commit and push.

## Report

The edits outside the owned files, by file (a list); how the request, outcome and refusal are
refused or reached today; any price or expectation corrected and why.

## Answers

2026-11-03. **Raise Small's derived store limits, or keep them and revise the sizing plan?**
Raise them. Small's `STORE_KV` `value`, `chunk` and `page.bytes` are no budget of the
user's but envelopes derived from the largest row (the comment says so; `cf7ee61604` and
`e6d351c2b7` raised them the same way for job specs and dependency marks), and 17.10's record
needs two whole sets per revision in the worst case: the items set at most a whole set, and
what they replaced is at most the settings now, another. Do it this way, on your branch
(pushed head `cfebdbacfb`):

- `jig/crates/jig-protocol-config/src/profile/small.rs`: `value` to the admission need you
  measured (2,039,684), and `page.bytes` and `chunk` derived from it as now (value + 2,079
  key; page + 20): 2,041,763 and 2,041,783 unless your measurement says otherwise. Rewrite
  the page comment to name the largest row as it now is (a revision's record with what its
  items replaced, if that is it). `commit`, `budget`, `segment`, `queued_bytes` and the rest do
  not derive from `value`: leave them. Standard (`standard.rs`) only if a test shows it short,
  derived the same way.
- The literals that pin those figures, and nothing else in their files:
  `jig/crates/jig-protocol-config/src/tests/profile.rs` (81), `jig/testing/jig-test-service/
  src/tests.rs` (25), and temper-service's `store.rs` (476), `tests/startup.rs` (58),
  `tests/limits.rs` (483), `tests/store.rs` (98). The spec's `files` now lists them.
- The three fixed-price literals in `jig/crates/jig-protocol-store/src/tests/format.rs`, as the
  pre-approved option already allows, each comment naming the replaced-state term.
- Then the focused run of the edited packages and their dependents again. A failure the raised
  limit does not explain (the run named "startup/readiness and refusal-precedence" failures
  besides the shortfall): read its log; if your change reaches it, fix it; if not, run it once
  alone and report it under `## Left` with both logs. Any other production limit or budget
  that fails is still a stop and a question. Name the raised figures and the measurement they
  come from in the commit.

The skeleton is unchanged.
