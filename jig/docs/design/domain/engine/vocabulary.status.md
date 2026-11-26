# vocabulary: status

2026-11-26. Where the tree's code lags or departs from [vocabulary.md](vocabulary.md).

- **Not yet:** 2.1, 2.2, 2.4: the store's port (`In::Store`, `Out::Held`, `Peer::Store` and the
  `engine::store` enums), and parties' and runs' reads through it [in flight: change 19318].
- **Not yet:** 2.4: ended tasks, read positions, decided proposals and escalations, and a task's
  proposal from its history, through the port (`Range::Record`, `Range::Proposal`)
  [in flight: change 19320].
- **Not yet:** 2.4: a transcript's window through the port (`Range::Window`, its pages and cut
  turns) [in flight: change 19321].
- **Not yet:** 2.4: notes' pages through the port (`Range::Notes`) [in flight: change 19322].

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
