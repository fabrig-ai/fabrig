# restart: status

2026-11-28. Where the tree's code lags or departs from [restart.md](restart.md).

- **Not yet:** 2, 4: the engine serving each phase through its ports, the switch asked and the wires written by it [in flight: change 22485]
- **Not yet:** 1, step 2: `ops`'s connectors loading their own records when asked [in flight: change 22494]
- **Not yet:** 1, step 2: the test connector loading its own records when asked [in flight: change 22499]
- **Not yet:** 1, step 2: temper's forge loading its own records when asked [in flight: change 22501]

- **Departs:** the whole document: `Core::new` without a start is ready, because the
  core's unit fixtures need no start; every root starts it.
   the optional-answer facts departure remains `jig/docs/design/client/domain/contract/answers.status.md`, 1.
   the undocumented root methods remain `vocabulary.status.md`, 1.
   the child-timer departure remains `vocabulary.status.md`, 1.1.
   the acceptance-capacity question remains `settings.status.md`, 2, and `settings-proposals.status.md`, 1, and `settings.status.md`, 8.
