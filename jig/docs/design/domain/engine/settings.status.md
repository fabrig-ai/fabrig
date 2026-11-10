# settings: status

2026-11-10. Where the tree's code lags or departs from [settings.md](settings.md).

- **Not yet:** 12: a read changing only the displayed omitted count
  does not mark `me`. `routing/lists.rs`, `answered`, compares good-read
  presence and `names`, but not `left_out`; `documents/small.rs`, `frame`,
  adds the visible accounts' largest `left_out` to each project's count.
  Settings can therefore keep the old count until another Frame mark.
  The mark table has no row for this case; this finding is by reading,
  with no runtime result supplied. No reason for the omission is recorded.

- **Not yet:**  runs on the engine's own slots under
  `smith-inline-agent`; the engine hub's hello before placement; the
  two-claim placement balance.  the remaining point-load fences.
   the brief's lineage, calls and waiting sections.  what a
  turn's calls made, written beside the turn.  the tools `move`, `succeed`,
  `hold` and `lift`.
   core-world stories that opt out of the restart
  oracle with a named finding (`broken()` in `jig/tests/core`), most for
  a claim the start settles (6, step 3) changing a task's `chip`, `acts`
  or run `settings`, which the projection keeps durable, others for a
  frame's `needs` with a person proposal left pending and
  `Deployment.messages` advancing on a retried deadline; the core
  world's `world` and `direct` drivers run no oracle.
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

- **Not yet:** 6: the whole project-deletion commit has no
  established admission price against `journal.writes`, because the
  established allowance covers settings-proposal withdrawals alone,
  excluding the remaining state writes, closing the project and cancellation
  operations. No reason is recorded.

- **Departs:**  `Core::new` without a start is ready, because the
  core's unit fixtures need no start; every root starts it.
   the core's answers to parties cross the engine's client translation
  with optional refusal and ending facts always empty; the fields and
  the choice to fill them or drop them in a client wire version are in
  `jig/docs/design/client/domain/contract/README.status.md`; no reason is recorded.
   the root interface has no text for public `Core::effect_owner` and
  `application_part_request`
  (`jig-domain-core/src/routing/flight.rs`), `call_room`
  (`jig-domain-core/src/runs/calls.rs`), or `next_finishing_deadline` and
  `finishing_due` (`jig-domain-core/src/routing/timers.rs`), which the test
  service uses; no reason is recorded.
   children's timers arrive through `fire`, but the section has no
  text on how many due children a pass fires.
  `jig-domain-core/src/routing/timers.rs`, `due`, returns one `Timer`, the
  first due child in people, tasks, fleet, brief and accounts order;
  `temper-engine-domain/src/domain.rs`, `fire_routed`, fires it once per
  pass, and `jig-test-service/src/engine.rs`, `fire`, also fires one;
  no reason is recorded.
  2, 6, 8: accepting proposed settings rechecks the change and
  refuses it as full, leaving it pending, if held revisions or charter
  versions would pass their limits. With one charter slot, the acceptance
  path (`jig-domain-core/src/routing/settings/dispatch.rs`, `accept` and
  `decide`) answers `ProposalDecided { Accepted }` rather than `Full`;
  whether a capacity check is missing or the expectation is wrong, and
  whether this departs, is the designer's decision: it is a bug if the
  design requires `Full` in this case; no reason is recorded.
