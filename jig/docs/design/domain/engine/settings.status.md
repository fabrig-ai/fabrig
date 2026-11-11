# settings: status

2026-11-11. Where the tree's code lags or departs from [settings.md](settings.md).

- **Not yet:** 12: a read changing only the displayed omitted count
  does not mark `me`. `routing/lists.rs`, `answered`, compares good-read
  presence and `names`, but not `left_out`; `documents/small.rs`, `frame`,
  adds the visible accounts' largest `left_out` to each project's count.
  Settings can therefore keep the old count until another Frame mark. The field is computed correctly on a fresh read.
  The mark table has no row for this case; this finding is by reading,
  with no runtime result supplied. No reason for the omission is recorded.

- **Not yet:**  engine slots, hello and placement remain `fleet.status.md` (Not yet, the whole document).  the point-load fences remain `README.status.md`, 4.2 and 5.3.
   the brief's missing sections remain `briefs.status.md`, 1.  turn-call objects remain `executors.status.md`, 2.  the missing tools remain `executors.status.md`, 3.
   the restart-oracle gaps remain `../testing.status.md`, 9.
  6: pending reasons admitted past `people.words` are cut at that bound
  in the review, with loss stated; no continuation is served.
  6: a
  proposal of settings waits for the deployment's owners whatever it
  names, its holder always the deployment's policy
  (`jig-domain-core/src/admission/proposals.rs`, `holder`): one naming
  an application's items of a project's scope is accepted by any of the
  deployment's owners, not by that project's; one naming a project's
  policy is admitted only where one party owns both the deployment and
  the project, and accepted only by such a party, even with no item of
  the deployment's beside it, where 6 has that project's owners
  decide it (`routing/settings/dispatch/project_policy.rs`, `owns`). Its
  row goes to the deployment's owners alone
  (`documents/inbox_deployment.rs`), so a project's owner who doesn't own
  the deployment reads none in that project.
  10: the core world's eleven restore stories
  (`jig/tests/core/tests/suite/settings_restore.rs`) are all ignored,
  each naming `restore-request`'s `Restore` admission or its
  `ForgetReplaced`, both on main since (`routing/party/people.rs`,
  `settings/restart.rs`, `forget_replaced`), and the story of a deleted
  project's items also naming a project-scoped default that survives
  the project's deletion (`../connectors.md`, 17); whether they pass
  unignored is unknown. The core's own tests of a restore run
  (`jig-domain-core/src/tests/settings/restore.rs`).
- **Not yet:** 5: validation of declared application kinds against
  configured limits before construction is missing, because
  `jig-domain-core/src/settings.rs`, `Current::new`, expects the capacity to
  be representable instead of returning a named configuration refusal; the
  bounds check takes limits without declared kinds. No reason is recorded.

- **Not yet:** 6: the project-deletion admission price remains `README.status.md`, 5.1.

- **Departs:**  the unstarted-core departure remains `restart.status.md` (Departs, the whole document).
   the optional-answer facts departure remains `jig/docs/design/client/domain/contract/README.status.md`, 3.
   the undocumented root methods remain `README.status.md`, 4.
   the child-timer departure remains `README.status.md`, 4.1.
  2, 6, 8: accepting proposed settings rechecks the change and
  refuses it as full, leaving it pending, if held revisions or charter
  versions would pass their limits. With one charter slot, the acceptance
  path (`jig-domain-core/src/routing/settings/dispatch.rs`, `accept` and
  `decide`) answers `ProposalDecided { Accepted }` rather than `Full`;
  whether a capacity check is missing or the expectation is wrong, and
  whether this departs, is the designer's decision: it is a bug if the
  design requires `Full` in this case; no reason is recorded.

- **Design departs:** 6, from the request `ui-walk-large` of 2026-11-06, decided by
  its plan's cut: that request has a proposal of settings' project set when it is
  made, and the core works its `projects` out at each read, because its items never
  change once it is made, so the value is the same and no stored format moves. That
  request has it show in each project it touches and in every project whose owner can
  decide it; a row names the lowest 32, the bound of that list on the client's wire.
