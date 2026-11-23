# A party's amendment proposal can be read, decided and withdrawn

Draft, 2026-11-17, on the user's word. On 11-17 a project's owner amended a task belonging to another party of the project. ctl answered `proposed: p1/202878, to role 0 in project 2`. Reading or deciding that proposal then answered `malformed document`, for its maker and for the task's party alike. Its maker had no way to withdraw it. The inbox listed no proposal while a party's needs counted one. Nothing cleared it until its target task ended.

## 1. What happens

- **A party's amendment proposal has no client form:** the client wire's translation refuses every proposal whose proposer is a party and whose action is an amendment (`jig/crates/jig-protocol-engine-clients/src/translate/objects/proposals.rs`, `proposal`). Yet the domain makes such proposals and the store keeps them (`jig/crates/jig-domain-core/src/documents/objects/party.rs`; `jig/crates/jig-protocol-store-records/src/codec/tasks/proposals.rs`). A read of one answers `malformed document` (`jig/crates/jig-protocol-engine-clients/src/reads.rs`).
  - ctl's `proposal decide` reads the proposal first, so it stops there and never sends the decision (`temper/crates/temper-client-tools-jig/src/jig/runner/acts.rs`).
  - The web's proposal cards hit the same refusal (`temper/crates/temper-client-domain-pages/src/pages/inbox.rs`).
- **A proposal and its target share one inbox entry:** a party's amendment proposal is projected under its target task's number, where the task's own needs also sit. Each save replaces the whole entry (`jig/crates/jig-domain-core/src/routing/task/save.rs`; `jig/crates/jig-domain-people/src/domain/inbox.rs`). So a later save of the task can erase the proposal's need, and saving the proposal can erase the task's other needs.
- **No client withdraws:** the domain lets a maker withdraw its own pending proposal, and the engine offers it (`jig/docs/design/domain/tasks/proposals.md`; `jig/crates/jig-domain-core/src/routing/party/propose.rs`). ctl has no `proposal withdraw` (`temper/docs/help/proposal.md`; `temper/docs/design/client/ctl/areas.status.md`). The web's proposal controls have no Withdraw (`temper/crates/temper-client-view-parts/src/confirm.rs`). A pending proposal never expires, so one that nobody can decide waits until its target ends.
- `jig/docs/design/domain/tasks/proposals.status.md` says "No gap".

## 2. Wanted

- **Every proposal the domain makes has a client form:** a party's amendment proposal is read, listed and decided as a task's proposal is, showing its target and the authority before and after. This holds on the client wire, in ctl and on the web. If the client wire can't carry it today, the wire's version moves.
- **A proposal's need keeps its own identity:** in an inbox, a proposal's entry and its target task's needs never replace each other. Both are counted and listed, whichever is saved first.
- **A maker withdraws its pending proposal:** ctl gets `proposal withdraw P`, and the web a Withdraw control. Both are offered to the maker alone and are sent without reading the proposal first.
- Revise `proposals.status.md`, ctl's `areas.status.md`, and `temper/docs/help/proposal.md`, together with whichever designs the client form touches and their status files.

## 3. Out

- Who may decide a proposal, and its routing to policy roles.
- How a widening is funded (`jig/docs/design/domain/authority-budgets.md`, 1).
- Pending proposals' expiry.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The client wire's translation and read tests (`jig/crates/jig-protocol-engine-clients/src/tests/objects.rs`, `jig/crates/jig-protocol-engine-clients/src/tests/reads.rs`): proposals from a party and from a task, with an amendment and with other actions, each read whole by its maker and by an eligible holder.
- The inbox tests (`jig/crates/jig-domain-core/src/tests/inbox/inbox_selection.rs`, `jig/tests/core/tests/suite/start_inbox.rs`): a proposal and its target, saved in both orders, keep both needs, their counts and their pages.
- ctl's and the web's tests:
  - `temper/crates/temper-client-tools/src/tests/jig/amend_controls.rs` and `temper/crates/temper-client-tools/src/tests/jig/settings.rs`;
  - `temper/crates/temper-client-domain/src/tests/pages/inbox.rs`;
  - `temper/tests/client/tests/suite/inbox_observation.rs`.

  Between them they cover: a party's amendment proposal is shown and decided; its maker alone withdraws it; a withdrawal sent again is answered once; its entry is gone afterwards.

Run only the new and affected tests, by name.
