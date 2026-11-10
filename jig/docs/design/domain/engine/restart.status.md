# restart: status

2026-11-10. Where the tree's code lags or departs from [restart.md](restart.md).

- **Departs:** the whole document: `Core::new` without a start is ready, because the
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
   accepting proposed settings rechecks the change and
  refuses it as full, leaving it pending, if held revisions or charter
  versions would pass their limits. With one charter slot, the acceptance
  path (`jig-domain-core/src/routing/settings/dispatch.rs`, `accept` and
  `decide`) answers `ProposalDecided { Accepted }` rather than `Full`;
  whether a capacity check is missing or the expectation is wrong, and
  whether this departs, is the designer's decision: it is a bug if the
  design requires `Full` in this case; no reason is recorded.
