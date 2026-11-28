# vocabulary: status

2026-11-28. Where the tree's code lags or departs from [vocabulary.md](vocabulary.md).

- **Not yet:** 2.1, 2.2, 2.6: the engine serving the start through its ports: the store's open, its paged loads, the switch, `Write` in the store's port with `Wires` and `Out::Write`, `Fill` and `Census` as the store's inputs, `Resume`, the application's start outputs, a connector's restore by `lifecycle::Ask::Restore` [in flight: change 22485]
- **Not yet:** 2.6: `ops`'s root and its connectors on the engine's start [in flight: change 22494]
- **Not yet:** 2.6: the test root, the test connector, the test store's adapter and service on the engine's start [in flight: change 22499]
- **Not yet:** 2.6: temper's root, its forge, store adapter and service on the engine's start [in flight: change 22501]
- **Departs:**
  1.2: the optional-answer facts departure remains `jig/docs/design/client/domain/contract/answers.status.md`, 1.
  1: the root interface has no text for public `Core::effect_owner` and
  `application_part_request`
  (`jig-domain-core/src/routing/flight.rs`) or `call_room`
  (`jig-domain-core/src/runs/calls.rs`), which the test
  service uses; no reason is recorded.
  1.1: children's timers arrive through `fire`, but the section has no
  text on how many due children a pass fires.
  `jig-domain-core/src/routing/timers.rs`, `due`, returns one `Timer`, the
  first due child in people, tasks, fleet, brief and accounts order;
  `temper-engine-domain/src/domain.rs`, `fire_routed`, fires it once per
  pass, and `jig-test-service/src/engine.rs`, `fire`, also fires one;
  no reason is recorded.
- **Design departs:** 2.4: the user asked, on 2026-11-27, that no root
  receive or name `core::Held::Store`. None receives one, but
  `core::Held` keeps the case, so a root's match over it, which lists
  every case, still names it, in an arm that routes nothing. Why: the
  core's steps queue each load among their other held requests in the
  one queue of requests they share, and a queue of the engine's outputs
  in its place reshapes every step of the core, which is the clean-up's
  (3, *The engine as the API*) and moves no behaviour this fix needs.
  Decided by the goal's designer.
- **Design departs:** 2.6: the application's port says where the start
  is, `application::Out::Start { phase }`, which the draft's list of the
  application's outputs (`temper/docs/design/draft/jig-domain-api.md`,
  5.5) doesn't name. Why: the root's admission holds what waits for the
  start (the draft's 5.1), its reports and its stops name the phase, and
  the engine has no query to ask (the draft's 4.1), so the engine says
  each phase as it begins and the opening as none. Decided by the goal's
  designer, 2026-11-28.
