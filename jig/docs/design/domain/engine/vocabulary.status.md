# vocabulary: status

2026-11-26. Where the tree's code lags or departs from [vocabulary.md](vocabulary.md).


- **Not yet:** 2.3: temper's root, `temper-engine-domain`, holds `Core` and steps it, times it and asks its quiescence itself, not through the engine [in flight: change 19212].
- **Not yet:** 2.3: temper's tests that build a core (in `temper-engine-service-stock`, `temper-engine-service-vocabulary`, `temper-box` and `temper-client-tools`) build `Core`, not `Engine` [in flight: change 19213].
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
