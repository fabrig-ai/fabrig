+++
id = "replace-own"
part = "replace"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-tasks/src/proposals.rs",
  "jig/crates/jig-domain-tasks/src/tests/**",
  "jig/crates/jig-protocol-tools/src/render.rs",
  "jig/tests/core/tests/suite/settings_proposals.rs",
  "jig/docs/design/domain/tasks.status.md",
  "jig/docs/design/domain/engine.status.md",
]
effort = "high"
minutes = 70
+++

# A run's fresh proposal of settings replaces its own

## Goal

Today a task's second `propose_settings` is refused as busy while its first waits. Make it replace the first: admitted, it withdraws the task's own pending proposal of settings in the same step. Refused, nothing changes. jig alone: this plan's next task changes temper's words on this.

Acceptance:

- A task whose proposal of settings waits proposes settings again: `CallPart::Proposed` names the new one, the old is `Withdrawn` with a `Change::ProposalWithdrawn` history row by `Party::Task(proposer)`, `core.settings_proposals` holds the new set alone, and the task's inbox holds no message of it.
- A fresh proposal refused for any reason (a stale `against`, a bad item, the reason's bound, `settings.proposals` full) leaves the old pending, its set kept, and no new history row.
- An owner's decision of the replaced proposal, afterwards, answers `people::Outcome::Refused(people::Refusal::Unknown)`, as `decision_record` writes no record for a task's withdrawn proposal, and makes no revision; accepting the new one makes revision N+1.
- Only settings over settings: a pending batch or effect refuses `propose_settings` as busy; a pending proposal of settings refuses `propose` of any other kind as busy; another task's pending proposal and a party's are untouched.
- After a restart, the new proposal alone is pending and its set is linked (`start/links.rs` passes).
- A task at `revision` one short of `u64::MAX` is refused, not panicked: a replacement takes two revisions.
- The busy refusal in `jig-protocol-tools/src/render.rs` ends `wait for its decision`.
- `jig/docs/design/domain/tasks.status.md`'s and `engine.status.md`'s **Not yet** entries for this are gone.

## Inputs

- `temper/docs/plan/ui-proposal-2/notes/contracts.md`, *The mechanism* and *The busy refusal's words*: binding.
- `temper/docs/plan/ui-proposal-2/notes/common.md`.
- `jig/docs/design/domain/tasks.md`, section 9, *A run's fresh proposal of settings replaces its own*.
- `jig/docs/design/domain/engine.md`, 17.6, *A run's fresh one replaces its own*.

## Owns and may change

Owns the files in the header. The next task, `chat-revises`, owns every temper file.

Readers, as found by survey:

- `jig/crates/jig-domain-core/src/routing/settings/dispatch.rs`'s `propose_run` preflights through `core.tasks.proposal_problem`, which is `admission_problem`, so it follows the change without an edit. `proposal_saved` and `keep_set` handle the withdrawn row, then the new pending row, in that order. Change them only if a test shows the order or an assertion is wrong, and say so.
- `jig/crates/jig-domain-tasks/src/tests/settings_proposals.rs`'s `settings_proposals_keep_the_one_pending_proposal_per_party_rule` is about a party's proposal and stays as it is.
- Any `jig` test that expects a run's second proposal of settings to be busy: change it to this rule. None was found by reading `jig/tests/core/tests/suite/settings_proposals.rs` and `settings_proposal_edges.rs`, but run them.

Options, each decided:

- **Keep out of `jig-domain-tasks`' `boundary.rs`, `control.rs`, `domain.rs`, `compound/staged.rs` and the stored schema:** another plan is changing them, and this needs none. If admission needs a line in `compound/staged.rs`, stop and find another way: a step never proposes settings (`jig/docs/design/domain/engine.md`, 17.6).
- **`admission_problem_staged` is shared with procedure steps.** The carve-out is only for an action of settings over a pending one of settings, so steps are unaffected.
- **If a store write bound trips** on the two history rows in one step, raise nothing: read the bound, and if it's a real limit report it as a `Question:` with the figure.

## Steps

The shape, in `jig/crates/jig-domain-tasks/src/proposals.rs`:

```rust
/// The pending proposal of settings a task's fresh one replaces (tasks.md, section 9).
fn replaced(task: &TaskRecord, proposal: &Proposal) -> Option<u64> { /* both of settings, the prior pending */ }

// admission_problem_staged: a task's pending proposal refuses unless `replaced` names it,
// and a replacement needs room for two revisions (`checked_add(2)`), where one needs one.

// propose: after admission, before apply_proposal.
fn replace_own(domain: &mut Domain, env: &Env<Limits>, proposal: &Proposal, out: &mut Queue<Request>) {
    // take the prior, state = Withdrawn, positions::history(.., Party::Task(proposer), Change::ProposalWithdrawn, out),
    // proposal_alarms.cancel(proposer). install publishes once, after both.
}
```

`apply_proposal` and `check_result`, used by steps and results, stay as they are.

Tests, in the worlds that hold them:

- `jig/crates/jig-domain-tasks/src/tests/` (a new `replaced.rs`, with its `mod` line): the kinds matrix, settings over settings admitted and every other pair busy; the revision bound; a closing task's refused; two tasks, one replaces, the other's stands.
- `jig/tests/core/tests/suite/settings_proposals.rs`, built on `run_propose`:
  - `a_runs_fresh_proposal_replaces_its_own_that_waits`
  - `a_refused_fresh_proposal_leaves_its_own_waiting`
  - `an_owners_decision_of_a_replaced_proposal_changes_nothing`
  - `a_replaced_proposal_stays_gone_after_a_restart`
- `jig-protocol-tools/src/render.rs`'s own test of the words, at its `Refused: busy` line.

Bite each new rule once by hand, as the implementer's text says.

Then: `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-tasks -p jig-protocol-tools -p jig-core-world --all-targets -- -D warnings`.

**This changes what a shared item means, so run its readers:** `heavy cargo nextest run -p jig-domain-tasks -p jig-domain-core -p jig-protocol-tools -p jig-core-world -p jig-tasks-world`, then `heavy cargo nextest run -p temper-engine-domain-world -p temper-client-world -E 'test(configuring) or test(settings)'`.

Expected red, and not yours: `a_pending_proposal_holds_the_next_until_an_owner_rejects_it` in `temper-engine-domain-world`, which `chat-revises` rewrites. Report it under `## Left`. Any other red is yours to read first.

Commit and push.

## Report

What ran and how many; the rule each new test bit on; any change to `dispatch.rs`; what's under `## Left`; your implementation and check minutes apart.
