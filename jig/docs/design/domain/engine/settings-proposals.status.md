# settings-proposals: status

2026-11-23. Where the tree's code lags or departs from [settings-proposals.md](settings-proposals.md).

- **Not yet:** 1: pending reasons admitted past `people.words` are cut at that bound
  in the review, with loss stated; no continuation is served.
  1: a
  proposal of settings waits for the deployment's owners whatever it
  names, its holder always the deployment's policy
  (`jig-domain-core/src/admission/proposals.rs`, `holder`): one naming
  an application's items of a project's scope is accepted by any of the
  deployment's owners, not by that project's; one naming a project's
  policy is admitted only where one party owns both the deployment and
  the project, and accepted only by such a party, even with no item of
  the deployment's beside it, where section 1 has that project's owners
  decide it (`routing/settings/dispatch/project_policy.rs`, `owns`). Its
  row goes to the deployment's owners alone
  (`documents/inbox_deployment.rs`), so a project's owner who doesn't own
  the deployment reads none in that project.
- **Not yet:** 1: the project-deletion admission price remains `decisions.status.md`, 1.1.

- **Departs:** 1: accepting proposed settings rechecks the change and
  refuses it as full, leaving it pending, if held revisions or charter
  versions would pass their limits. With one charter slot, the acceptance
  path (`jig-domain-core/src/routing/settings/dispatch.rs`, `accept` and
  `decide`) answers `ProposalDecided { Accepted }` rather than `Full`;
  whether a capacity check is missing or the expectation is wrong, and
  whether this departs, is the designer's decision: it is a bug if the
  design requires `Full` in this case; no reason is recorded.

- **Design departs:** 1, from the request `ui-walk-large` of 2026-11-06, decided by
  its plan's cut: that request has a proposal of settings' project set when it is
  made, and the core works its `projects` out at each read, because its items never
  change once it is made, so the value is the same and no stored format moves. That
  request has it show in each project it touches and in every project whose owner can
  decide it; a row names the lowest 32, the bound of that list on the client's wire.
