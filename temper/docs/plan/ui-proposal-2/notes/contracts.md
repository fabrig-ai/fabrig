# ui-proposal-2: the binding calls

## The mechanism: a fresh proposal replaces its own

`jig/docs/design/domain/tasks.md`, section 9, *A run's fresh proposal of settings replaces its own*, and `jig/docs/design/domain/engine.md`, 17.6.

- A task's `propose_settings` over that task's own pending proposal of settings is admitted, and withdraws the one that waits in the same step. Chosen over a `withdraw` the chat could call: there is no moment with nothing pending, a refused new one leaves the old standing, no family moves and no run gains a tool, and the model needs no proposal number.
- **No new event, field, record or wire form.** The replaced proposal is an ordinary withdrawal: `ProposalState::Withdrawn`, a `Change::ProposalWithdrawn` history row by `Party::Task(proposer)`, a `Decided { made: false }` activity line. `routing/task/save.rs`'s `proposal_saved` erases its set from that row, as for any withdrawal.
- **Its proposer hears no message of it:** no `SettingsDecision` word. `SettingsOutcome::Stale` stays for a revision made since.
- **The answer is unchanged:** `CallPart::Proposed { proposal, holder }`, naming the new one.
- **What stays refused:** a pending proposal of another kind holds the next as busy; settings over another kind is busy; a party's stays one at a time (`settings_proposals_keep_the_one_pending_proposal_per_party_rule`); and `core`'s `proposal_check` still refuses at `settings.proposals`, the replaced set not counted as room, which changes nothing.
- `withdraw` keeps its decide family, and stays refused to the configuring chat.
- **A decision of the replaced one is refused:** `core`'s `decision_record` writes no record for a task's withdrawn proposal, so an owner's decision reaches `route_ended_proposal` with no row and answers `Refused(Unknown)`, as any task's withdrawn proposal does today. It makes no revision. The client ends the card on that answer (`cards/proposal.rs`, `settings_state`), so the press says so.

## The busy refusal's words

Exactly, in both copies, `jig-protocol-tools/src/render.rs` and `temper-engine-smith/src/answers.rs`: the same line as now, ending at `wait for its decision`. "or withdraw it" goes, since a run with no decide family can't. So:

`Refused: task 2 busy: it has a proposal pending (at most 1 at once): wait for its decision`

## The configuring chat's words

`temper/crates/temper-service/src/stock/configuring.txt`, in the `Propose once` paragraph. These two sentences:

> You can't withdraw or replace it while it waits. Asked to, say so: an owner rejects it under Needs you, with a reason you hear; then propose afresh.

become:

> Asked to change it while it waits, propose again, whole, against the newest revision: yours replaces the one waiting and its card says so, so never send them to Needs you for it. Refused, the one waiting stands: say so.

In the `First:` paragraph, after `say no account can run a model yet, and to connect one.`, add:

> Asked for a model no account can run, say so and, in one line, to connect one: Settings, Accounts, in the browser.

## What a card reads for its application's items

A proposal's heads (`c::ProposalActionSettings.items`) name jig's settings alone: `jig-domain-core`'s `proposal_heads` leaves out each application's item and project's policy, on purpose, as `jig_domain_tasks::Setting` has no such variant. Heads stay as they are. No stored or wire change.

- A live card takes those rows from `settings.review`: each `c::ReviewItem` whose `setting` is `Application { .. }` or `ProjectPolicy { .. }`, by its `setting` and `edit`.
- The forge keeps a proposal's values under `review.key`, not the proposal's number (`keep_proposal`'s `SettingSlot::Proposal(key)`; ctl's `Query::ProposalItems { key }`). `forge/ways.rs` asks under `review.key`, and `Side::proposed` stays looked up by the proposal's number.
- A decided card has no review and no kept values. It keeps what it read, and with none draws its heads alone. A person with no deployment role has no review: the same.
