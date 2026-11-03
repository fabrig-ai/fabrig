+++
id = "apps-wire"
part = "appsettings"
kind = "feature"
needs = ["apps-proposals"]
files = ["jig/crates/jig-client-protocol-wire/schema/client-v19*.schema", "jig/crates/jig-client-protocol-wire/src/generated/v19.rs", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-protocol-engine-clients/src/translate/settings.rs", "jig/crates/jig-protocol-engine-clients/src/translate/settings/**", "jig/crates/jig-protocol-engine-clients/src/translate/refusals.rs", "jig/crates/jig-protocol-engine-clients/src/tests/settings*.rs", "jig/crates/jig-client-protocol-link/src/translate/settings.rs", "jig/crates/jig-client-protocol-link/src/tests/settings*.rs", "jig/crates/jig-client-domain/src/contract/settings.rs", "jig/crates/jig-domain-core/src/documents/settings.rs", "jig/crates/jig-domain-core/src/reads/settings.rs", "jig/docs/design/client/domain/contract.status.md", "jig/crates/jig-domain-core/src/documents/settings_review.rs", "jig/crates/jig-domain-core/src/reads/vocabulary.rs", "jig/crates/jig-client-domain/src/contract/refusals.rs", "jig/crates/jig-client-protocol-link/src/translate/answers.rs", "jig/examples/ops/domain/src/route.rs", "jig/tests/core/src/peers.rs", "temper/crates/temper-service/src/startup/problems.rs", "temper/crates/temper-engine-domain/src/store/settings_bytes.rs", "jig/testing/jig-fake-parties/src/facts.rs", "jig/crates/jig-domain-core/src/documents.rs", "jig/crates/jig-domain-core/src/reads.rs", "jig/crates/jig-domain-core/src/limits.rs"]
effort = "high"
minutes = 140
+++

# apps-wire: what a client sends and reads of an application's settings

## Goal

jig's client wire carries an application's settings as the client contract has them:
`ChangeSettings` and a settings `Propose` carry items of an application's kind (kind, scope,
number; set to bytes, removed or reset); a *judged* refusal names the item and carries the
connector's words; the settings document, a setting's read, a revision's read and a
proposal's review say an application setting's state and size, never its value.

Acceptance:

- An application item round-trips client → link → wire → engine translation → the core's
  `settings::Item { setting: Setting::Application { .. }, edit: Edit::Set(
  Value::Application(bytes)) | Removed | Reset }`, and its documents back; a
  `Refusal::Judged` reaches the client domain with its setting and words.
- Every `unreachable!("..: apps-wire")` arm `apps-surface` and `apps-change` left is gone,
  in jig and in temper.
- The crates' tests and goldens pass, goldens remade for the changed records only; the
  `jig-engine-clients-world` and `jig-client-wire-world` settings stories pass (a change of
  meaning of the wire's settings records: run them).
- The **Not yet** "3.2, 4, 5.1, 6: an application's settings ..." of
  `jig/docs/design/client/domain/contract.status.md` is removed.

## Inputs

- `jig/docs/design/client/domain/contract.md`, 3.2 (`ChangeSettings`, `Propose`'s settings),
  section 4 (*judged*), 5.1 (*the settings*, the review's items), section 6 (`setting{..}`,
  `revision{n}`).
- `jig/docs/design/protocol/client.md`, section 9 (**Until the first release, versions stay
  fixed**).
- `jig/docs/design/domain/engine.md`, 17.5 (**The core keeps the rest**).
- `temper/docs/plan/reference-m3/notes/common.md`.

## Owns and may change

- Owned: the `files` above.
- Neighbour, arms and fields only: `jig/crates/jig-client-view/src/words/**` (a judged
  refusal worded "refused by its owner: {words}", an application setting named
  "application kind {kind}, {scope}, {number}"; follow the file's phrase tables);
  `jig/testing/jig-client-scripted-engine/**`, `jig/testing/jig-client-test-records/**`;
  `jig/tests/client-wire/**`, `jig/tests/engine-clients/**` fixtures; temper's readers of
  `jig_client_domain`'s settings types (`temper-client-tools` `src/jig/settings_adapter.rs`
  and `render/`, `temper-settings-form`, `temper-engine-smith` `src/settings.rs`): the
  compiler-forced arm alone, an application setting shown by its numbers and a judged
  refusal by its words (stated cross-tree allowance).
- **Version:** the client wire is `client-v19` (with `client-v19-part1`) on this trunk;
  change it in place, its generated code and goldens made again with the crate's tooling
  (`protocol/client.md`, section 9: versions stay fixed before the first release). If a
  test or a hello check needs the version to move, ask.
- Options, pre-approved:
  - If the wire's `Setting` is a union of jig's four kinds, add `application { kind, scope,
    number }` (default); an item's value `application: bytes` within the request's message
    bytes.
  - If `contract.md` 5.1's per-setting name has no source for an application's kind, leave
    the name out (the application names its kinds).

## Steps

1. Schema (`client-v19*.schema`): the setting union's application arm; the value's
   application bytes; the settings document's per-setting state gains `size: u32` and
   `charters: [u32]` for an application's; a revision's and a proposal's application items
   (`SizedEdit`); the refusal's `judged { at, setting, why }`. Regenerate.
2. Engine side (`jig-protocol-engine-clients`): `ChangeSettings`/`Propose` translate an
   application item into the core's `settings::Item` (`translate/settings/change.rs`);
   documents and reads from the core's `Current.applications`, `Revision.applications`,
   `Proposal.applications` (`translate/settings.rs`); `Refusal::Judged` in
   `translate/refusals.rs`. The core's `documents/settings.rs` and `reads/settings.rs` give
   the application states their places in the document and the reads.
3. Client side (`jig-client-protocol-link`, `jig-client-domain`): `contract::Setting`
   gains `Application { kind, scope, number }`, the value its bytes, the document's state
   its size and charters, the refusal its judged arm.
4. Tests: the crates' translate tests (an application item each way, a document with
   one changed and one default-following application setting, a judged refusal); the
   wire's goldens for the changed records.
5. Narrow tests: `heavy cargo nextest run -p jig-protocol-engine-clients --lib`,
   `-p jig-client-protocol-link --lib`, `-p jig-client-protocol-wire --lib`;
   `heavy cargo nextest run -p jig-engine-clients-world -E 'test(/settings/)'` and
   `-p jig-client-wire-world -E 'test(/settings/)'` (check the counts). `cargo fmt`;
   `heavy cargo clippy --workspace --all-targets`. Edit the status file. Commit and push.

## Report

The schema records changed; the goldens remade; temper's arms, by file.

## Answers

2026-11-03. **Q:** may this task change the core's proposal review builder and the other
files that hold its arms, outside its files?

**A:** Yes, option A: all eight are now in `files`. `apps-proposals` took its pre-approved
option and left the review's application items to this task, so
`documents/settings_review.rs` is this task's to build.

- `documents/settings_review.rs`: a review item of an application's setting from the set's
  `applications` (its `SizedEdit`) and `Current.applications`: the setting, the edit, its
  state now (`Follows`) and its size now and after, never a value; `document_bound` counts
  the application items. Change no other behaviour there (`apps-proposal-key` adds its `key`
  later).
- `reads/vocabulary.rs`: `SettingDocument`'s per-setting state gains an application's size
  and charters, no value.
- `jig-client-domain` `contract/refusals.rs`: the judged arm; `jig-client-protocol-link`
  `translate/answers.rs`: its decoding.
- `jig/examples/ops/domain/src/route.rs`, `jig/tests/core/src/peers.rs`, temper's
  `startup/problems.rs` and `store/settings_bytes.rs`: each `apps-wire` arm replaced by its
  real handling (an application's setting named by its numbers, a judged refusal by its
  words, an application's value as its bytes), the arm alone.

2026-11-03. **Q:** a reset's size after the edit has no source when the setting is changed or
removed now: show it as unknown (A), or have the core keep each default's size apart (B)?

**A:** A, in the design's own terms; the skeleton is unchanged. The core keeps an
application setting's state and its current value's size, never a default's apart
(`jig/docs/design/domain/engine.md`, 17.5, **The core keeps the rest**), and a revision's
record already gives a reset no size. So a review item of an application's setting carries
the set's `SizedEdit` as it is and the setting's state now (`Follows`, size, charters):
the size after the edit is a `Set`'s own size, none for a removal, and a reset says only
that the setting returns to its default, with no size; the application's own reads serve
the default and its size (`jig/docs/design/client/domain/contract.md`, 5.1, "the
application serving its values"). No separate "unknown" marker, no zero for a size, no ask
to a connector during review, no change to `State` or the store. Keeping defaults' sizes in
the core is not this milestone's; leave it.

2026-11-03. **Q:** may this task give `jig-fake-parties` a judged refusal, and price the
settings document's application settings from the declared kinds through the core's bound
callers (A), or leave both to other tasks (B)?

**A:** A, both; acceptance cannot hold without them. The skeleton is otherwise unchanged.

- **The fake's judged refusal.** `jig/testing/jig-fake-parties/src/facts.rs` is now in
  `files`. Its `Refusal` gains `Judged { at: u32, setting: Setting, why: Box<[u8]> }`, and
  the file its own `Setting` (`Charter(u32)`, `Template(u32)`, `Policy(u32)`, `Rules`,
  `Application { kind, scope, number }`), mirroring `jig_domain_people`'s without naming it:
  the fake stays independent. `jig/tests/core/src/peers.rs` maps the judged arm to it, field
  for field. Any other consumer the compiler names takes the arm alone.
- **The settings document's price.** The design has the core's worst case price an
  application's kinds from their declared bounds (`jig/docs/design/domain/engine.md`, 17.5,
  **Declared at the start**; 17.7, **The current settings**), and the kinds are the
  configuration's, beside `Limits`, not in it: so the bound takes them, no new limit and no
  fixed count. `documents/settings.rs::document_bound(limits, kinds: &[Kind])` adds, for
  `settings::application::capacity(kinds, limits)` lines, a line each and each line's
  charters priced at its kind's `charters`. `documents.rs::document_with_context_bound` and
  `reads.rs::answer_bound` take `kinds` and pass it on (both now in `files`). A caller that
  holds the kinds passes them (the core's own, `core.kinds`); `limits.rs::valid`, which
  checks `Limits` alone, passes `&[]`, and so do the kind-free callers outside the core
  (`jig-test-domain` `src/limits.rs`, temper-engine-domain `src/limits.rs` and its
  `decision.rs` comment): the argument alone, nothing else changed there (stated
  cross-tree allowance). A test: the bound with one declared kind exceeds the bound with
  none by at least that kind's lines.
- **Not this task's:** checking a configuration's declared kinds against its limits when
  it is loaded (`Current::new` still `expect`s the capacity), and temper passing its own
  declared kinds into its held-output price. Report them under `## Left`; do not build
  them.

