+++
id = "restore-forge"
part = "restore"
kind = "feature"
needs = ["restore-surface", "forge-replaced"]
files = ["temper/crates/temper-engine-domain-forge/src/kinds.rs", "temper/crates/temper-engine-domain-forge/src/settings.rs", "temper/crates/temper-engine-domain-forge/src/tests/kinds.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/tests/kinds.rs"]
effort = "high"
minutes = 120
+++

# restore-forge: the forge answers a restore's items of temper's kinds and forgets what is erased

## Goal

temper's forge answers the two asks `restore-surface` declared (contract B3) for temper's
kinds: a restore's items, kept as the proposal's from the rules and charters it kept as
replaced (`forge-replaced`) and answered with their sizes and charters, and `ForgetReplaced`,
erasing the replaced rows of revisions the history no longer keeps. The root's
`route/kinds.rs` routes both (jig's `domain/connectors.md`, 17; temper's
`protocol/configuration.md`, 13.2).

Acceptance:

- `connector::Ask::RestoreSettings { owner, proposal, items }` reaches the forge through
  `route/kinds.rs`; for each item the forge keeps a `SettingRow` at `SettingSlot::Proposal(
  proposal)`: a `Set { revision }` with the value of its `Replaced(revision)` row, `Remove` and
  `Reset` as their edits; and the root answers `connector::Event::SettingsRestored { owner,
  judged }`, one `Judged` per item in order: `Accept { size, charters }` (a charter's number
  for kind 3, as `judge_settings` names it), or `Refuse { why }` where no replaced row is kept
  or the value is past its kind's bounds (the forge's words, as its judge's refusals are).
- `connector::Ask::ForgetReplaced { before }` erases every `Replaced(revision)` row with
  `revision < before`.
- `forge_route.rs`' and `route/kinds.rs`' `unreachable!("a restore: restore-forge")` arms are
  gone.
- The unit tests below pass with the crates' focused tests.

## Inputs

- jig's `jig/docs/design/domain/connectors.md`, 17; `jig/docs/design/domain/engine.md`, 17.10.
- temper's `temper/docs/design/protocol/configuration.md`, 13.2 ("Judged by the forge",
  "Kept by the forge").
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B3; `notes/common.md`.
- `temper/docs/plan/configuring-p2-build/tasks/forge-replaced.md`, which this builds on.

## Owns and may change

- Owned: `temper-engine-domain-forge/src/kinds.rs` (`restore_settings`, `forget_replaced`),
  `src/settings.rs` (any helper type), `src/tests/kinds.rs`;
  `temper-engine-domain/src/route/kinds.rs` (the two asks and the answer, as
  `JudgeSettings`' is), `src/route/forge_route.rs` (where it sends a connector ask of the
  forge's kinds to `route/kinds.rs`: the two asks join that arm), `src/tests/kinds.rs`.
- Options, pre-approved:
  - If the root keeps the judge's owner to answer `SettingsJudged` and the restore can reuse
    it, reuse it; otherwise keep the restore's owner beside it and say so.
  - If the forge's `keep_proposal_settings` can take the restored items as they are (an
    `Asked` with the replaced value), call it rather than writing the rows again.

## Steps

1. `kinds.rs`: `pub fn restore_settings(&mut self, env, proposal: u64, items: &[Restored],
   out) -> Box<[Judged]>` (the forge's own `Judged`-like answer, as `judge_settings` returns
   one) and `pub fn forget_replaced(&mut self, before: u64, out)`.
2. `route/kinds.rs`: `connector::Ask::RestoreSettings { owner, proposal, items } =>` the
   forge's answer pushed as `Work::Core(Event::EffectConnector(connector::Event::
   SettingsRestored { owner, judged }))`, as `JudgeSettings`' answer is pushed;
   `connector::Ask::ForgetReplaced { before } => domain.forge.forget_replaced(before, &mut
   out)`.
3. Tests, `temper-engine-domain-forge/src/tests/kinds.rs`, with its helpers:
   - `a_restore_keeps_the_replaced_rule_as_the_proposals_item_and_answers_its_size`
   - `a_restore_of_the_forges_charter_names_the_charter`
   - `a_restore_item_with_no_replaced_row_is_refused`
   - `forgetting_erases_replaced_rows_of_older_revisions_only`
   and in `temper-engine-domain/src/tests/kinds.rs`:
   `a_restore_ask_reaches_the_forge_and_its_answer_reaches_the_core`.
4. Narrow tests: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/kinds/)'` (check the count); `heavy cargo nextest run -p temper-engine-domain --lib
   -E 'test(/kinds/)'`. Then `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain --all-targets -- -D warnings`; the focused tests of both packages.
   Commit and push.

## Report

The routing of the owner; the rows a restore writes; the tests and their counts.
