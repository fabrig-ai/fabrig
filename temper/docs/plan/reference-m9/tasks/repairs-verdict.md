+++
id = "repairs-verdict"
part = "repairs"
kind = "feature"
needs = ["store-surface"]
files = ["temper/crates/temper-engine-domain-forge/src/verification.rs", "temper/crates/temper-engine-domain-forge/src/tests/verification.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs"]
effort = "high"
minutes = 90
+++

# A red landing cleared by a run is repaired by the landing that names it

## Goal

When a later tier run passes the failures blamed on a red landing, the forge records the
landing in that run's range whose change names the red one (`ChangeRow.repairs`) as its
repairer, and the red landing's change's result says so.

Acceptance (`notes/contracts.md`, section 3; `temper/docs/design/domain/after-landing.md`, 5.3,
6 and 7):

- `verification::judged`: where it sets a tier's `repaired`, `TierRepair.by` is the commit of
  the newest landing of the run's range (the `landings` it is given, oldest first) whose
  `Verification` row's changes include one with `ChangeRow.repairs` equal to the red landing's
  commit; `None` when none does. Nothing else in `judged` changes meaning.
- `change_verification_words` (`forge_route.rs`): a tier whose `repaired.by` is `Some` words
  its line `Cleared by NAME run N, which passed the failures blamed on it; repaired by HEX12.`,
  HEX12 the repairing commit's first twelve digits (the text `commit_text` gives); with `None`
  the line is as today.
- Tests:
  - in `temper-engine-domain-forge/src/tests/verification.rs`: a red landing, then a run whose
    range holds two landings, the newer one's change naming the red landing, passing its
    failures: the red landing's tier is repaired `by` the newer one's commit; the same with no
    change naming it: `by: None`; two naming it in one range: the newest;
  - in `forge_route/tests.rs`: the result words of a cleared change repaired by a landing, and
    of one cleared by the run alone (unchanged).

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, sections 1 and 3.
- `temper/docs/design/domain/after-landing.md`, 5.3 (repaired by the landings that name it), 6
  (the result names the landing that repaired it), 7 (`repairs`).

## Owns and may change

- Owns `judged` in `verification.rs` and its tests, and `change_verification_words` with its
  tests; `repairs-call` edits other functions of `forge_route.rs` at the same time
  (`change_batch`, `chat_change`, `start_change`): touch none of them.
- `ChangeRow.repairs` is set by `repairs-call`; your tests set it in their fixtures directly.
- The ignored stories of `temper/tests/engine/domain/tests/suite/forge_routes/tiers.rs` are
  the stories task's, not yours.

## Steps

1. `judged`'s `by`, with its unit stories.
2. The result words, with theirs.
3. Tests: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/verification/)'`;
   `-p temper-engine-domain --lib -E 'test(/forge_route/)'`;
   `-p temper-engine-domain-world --test suite -E 'test(/forge_routes::tiers/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain
   --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks.

## Answers

2026-11-03. May this task add a public read of the retained `Verification` row for
`change_verification_words`? Yes: add `Domain::landing_verification(&self, repository:
skein_forge::api::Repository, commit: &skein_forge::api::Commit) -> Option<&Verification>` in
`verification.rs`, beside `change_verification`, with a one-line doc citing
`temper/docs/design/domain/after-landing.md`, 6: it returns the live row of (repository,
commit) from `self.verifications`, or `None`, and changes nothing. It is this spec's public
item (`notes/contracts.md`, section 3, now names it); add no other. `change_verification_words`
reads the cleared change's merge row through it (the `merge` it is given and the change row's
`repository`). Test it through the result words in `forge_route/tests.rs`; no story of its own
is needed. `ChangeRow.repairs` and `TierRepair.by` are `store-surface`'s: this task needs it,
so start from a trunk that holds its merge, and define neither field here.

## Rebase (2026-11-03, step merge, foreman reference-m9/repairs-verdict/foreman-2)

rebase repair wording onto main’s retained clearing-run rendering.
The merge conflicts in `temper/crates/temper-engine-domain/src/route/forge_route.rs` and its `forge_route/tests.rs`.
Main commit `09010971a5000722f34d994bf1c11ac346dbbf98` (“temper: retain clearing runs in change results”), integrated through `ba2caaa8aa`, changes the same functions and fixtures.
Preserve main’s retained verdicts, per-tier lines, clearing run numbers, cancellation wording and removal of `tier_landing_run`.
In `change_verification_words`, add `; repaired by HEX12.` when `TierRepair.by` is `Some`; retain main’s existing sentence when it is `None`.
Keep the authorized `landing_verification` lookup using the change’s repository and merge commit; reconcile the function signature and callers accordingly.
In tests, preserve main’s `result_verdicts` fixture and run-number assertions. Adapt the branch’s repair fixtures to retained verdicts instead of restoring configured-tier fallback rendering.
Expect the run-only repair line to be `Cleared by browser run 2, which passed the failures blamed on it.`
Preserve coverage for the repairing commit and multiple tiers, with each retained tier on its own line.
Keep the auto-merged `judged` repairer selection, its tests and the authorized accessor.
Stay within the four task files; leave `change_batch`, `chat_change` and `start_change` untouched.
Run the spec’s focused tests and formatting/clippy checks after resolving the conflicts.
