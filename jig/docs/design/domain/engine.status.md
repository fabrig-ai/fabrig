# engine: status

2026-11-03. Where jig's code lags or departs from [engine.md](engine.md).

- **Not yet:** 2, 8: runs on the engine's own slots under
  `smith-inline-agent`; the engine hub's hello before placement; the
  two-claim placement balance. 4.2, 5.3: the remaining point-load fences.
  7.1, 9: the brief's lineage, calls and waiting sections. 7.2: what a
  turn's calls made, written beside the turn. 7.3: `move`, `succeed`,
  `hold` and `lift`. 11.1: `me`'s
  `resolved_default`. 15: core-world stories that opt out of the restart
  oracle with a named finding (`broken()` in `jig/tests/core`), most for
  a claim the start settles (6, step 3) changing a task's `chip`, `acts`
  or run `settings`, which the projection keeps durable, others for a
  frame's `needs` with a person proposal left pending and
  `Deployment.messages` advancing on a retried deadline; the core
  world's `world` and `direct` drivers run no oracle. 17.6: a
  proposal of settings waits for the deployment's owners whatever it
  names, its holder always the deployment's policy
  (`jig-domain-core/src/admission/proposals.rs`, `holder`): one naming
  an application's items of a project's scope is accepted by any of the
  deployment's owners, not by that project's; one naming a project's
  policy is admitted only where one party owns both the deployment and
  the project, and accepted only by such a party, even with no item of
  the deployment's beside it, where 17.6 has that project's owners
  decide it (`routing/settings/dispatch/project_policy.rs`, `owns`).
  17.10: the core world's eleven restore stories
  (`jig/tests/core/tests/suite/settings_restore.rs`) are all ignored,
  each naming `restore-request`'s `Restore` admission or its
  `ForgetReplaced`, both on main since (`routing/party/people.rs`,
  `settings/restart.rs`, `forget_replaced`), and the story of a deleted
  project's items also naming a project-scoped default that survives
  the project's deletion (`connectors.md`, 17); whether they pass
  unignored is unknown. The core's own tests of a restore run
  (`jig-domain-core/src/tests/settings/restore.rs`).
- **Departs:** 6: `Core::new` without a start is ready, because the
  core's unit fixtures need no start; every root starts it. 8: the
  client wire drops the failing charter's number of a
  shortfall's turns, time, waiting and checks
  (`jig-protocol-engine-clients/src/translate/deployment.rs`), because its
  shortfall arms carry no charter; a client wire version adds it.
