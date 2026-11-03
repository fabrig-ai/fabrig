# engine: status

2026-11-02. Where jig's code lags or departs from [engine.md](engine.md).

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
  world's `world` and `direct` drivers run no oracle. 17.5, and an
  application's kinds in 17.6 and 17.7: jig's settings are closed
  (`jig-domain-people/src/settings.rs`'s `Setting`,
  `jig-domain-core/src/settings.rs`'s `Value`, settings-v1's `Setting`
  and `Value`), so an application declares no kind of its own, and no
  connector judges or keeps one.
- **Departs:** 6: `Core::new` without a start is ready, because the
  core's unit fixtures need no start; every root starts it. 8: the
  client wire drops the failing charter's number of a
  shortfall's turns, time, waiting and checks
  (`jig-protocol-engine-clients/src/translate/deployment.rs`), because its
  shortfall arms carry no charter; a client wire version adds it.
