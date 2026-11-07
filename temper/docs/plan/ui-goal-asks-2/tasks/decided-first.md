+++
id = "decided-first"
part = "decide"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/routing/party/decisions.rs",
  "jig/tests/core/tests/suite/settings_proposal_edges.rs",
]
effort = "high"
minutes = 40
+++

## Goal

`temper ctl proposal decide t2/6 accept` on a proposal a replacement withdrew answers exit 2, `out of shape; running`, with or without `--running`, where the help promises `decided first:`. The walker's flag isn't the cause. ctl reads the ended proposal, sees a settings action (`settings_history` in `temper/crates/temper-client-tools/src/jig/runner/acts.rs`) and sends `running: Some(Stays)` either way. The engine's task-proposer branch then refuses any `running` before it looks for the ended row. Let that request reach the ended row, whose own check already takes `running` on an accepted settings proposal.

Acceptance:

1. Accepting a task's settings proposal that has ended, withdrawn, accepted or rejected, with `running: Some(_)`, answers `people::Outcome::ProposalDecided { repeated: true, choice, by, at, .. }`, as `jig/crates/jig-domain-core/src/routing/party/ended.rs` makes it, and changes nothing: the settings' revision, the proposal's state and the live proposal beside it stand.
2. `running` on a live proposal that isn't settings still answers `Malformed::Running`. On a rejection or a pass, with any proposal, it still does.
3. A proposal that was never made answers `Refusal::Unknown`, with `running` or without.
4. A party's settings proposal that has ended answers as acceptance 1 does. If it already does, the table holds that as a row and `decisions.rs` doesn't change for it.

## Inputs

- `jig/docs/design/client/domain/contract.md`, 3.2, `Decide`: "or decided first, by whom and how".
- `jig/docs/design/domain/tasks.md`, 9.
- `temper/docs/help/reference/ctl-proposal.md`, `proposal decide`: what the fix makes true. No help or design text changes.
- `temper/docs/plan/ui-goal-asks-2/notes/common.md`.

## Owns and may change

**Owns:** the files in the header.

**Reader, read only:** `jig/crates/jig-domain-core/src/routing/party/ended.rs`, whose `running_valid` is the check this request now reaches. It needs no edit. If it does, make the edit and say why in the report.

**Not yours:** `temper/`. ctl already sends the right ask and `repeated_text` in `temper/crates/temper-client-tools/src/jig/render/outcome.rs` already prints `decided first: by …, withdrawn, …`. A branch stays in its tree.

**Options, each pre-approved:**
- If a test in `jig-domain-core` or `jig-domain-people` pins `Malformed::Running` for an ended or missing proposal, it pinned this fault: change it to acceptance 1 or 3 and name it in the report.

## Steps

**1. Failing test first,** a table in `jig/tests/core/tests/suite/settings_proposal_edges.rs`, beside the tests that withdraw a proposal (`proposal_state(..) == Withdrawn`), on their fixture:

```rust
// (proposer, how it ended, running) -> ProposalDecided { repeated: true, choice }
```

Rows: a task's proposal withdrawn, `Some(Stays)` and `Some(Moves)`; a task's proposal accepted; a party's proposal withdrawn. Each asserts the answer, and that the settings document and the replacing proposal are as they were. Add `running` on a live non-settings proposal and on a rejection as rows of what already pins them, if a table holds them; otherwise leave those tests be.

**2. `jig/crates/jig-domain-core/src/routing/party/decisions.rs`,** the `people::Proposer::Task(number)` arm near line 640:

```rust
if let Some(row) = core.tasks.proposal(number, proposal)
    && let tasks::ProposalAction::Settings { .. } = row.action
{
    task_settings_decide(core, env, routed, party, row, (decision, running));
    return;
}
// Only a live proposal refuses `running` here: an ended one reaches `Now::EndedProposal`, which judges it by what it was.
if core.tasks.proposal(number, proposal).is_some() {
    if running.is_some() {
        pool_refused(&mut routed.work, request, Refusal::NotWellFormed { at: 0, why: people::Malformed::Running });
        return;
    }
    person_proposal_decide(core, env, routed, party, number, proposal, decision);
    return;
}
```

Check the arm's other uses of `proposal` and its fall-through to `Now::EndedProposal` as you make it.

**3.** `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-core -p jig-core-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-core -p jig-domain-people -p jig-core-world`. This changes what `Decide` answers, so also run `temper-client-tools`'s tests, which read that answer: `heavy cargo nextest run -p temper-client-tools --lib`. Commit and push.

## Report

Name the rows, the rule broken by hand (restore the early `running` refusal) and what it failed with, and whether acceptance 4 needed a change.
