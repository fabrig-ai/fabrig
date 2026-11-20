# restart: status

2026-11-20. Where the tree's code lags or departs from [restart.md](restart.md).

- **Not yet:** 1: a stop during the start: told `Stopping` before decisions open, the core asks no further phase but never asks `Held::Drained`, which only `finish_decision` reaches once the start's runtime inputs run (`routing/stopping.rs`, `drained`) [in flight: change 10969].

- **Departs:** the whole document: `Core::new` without a start is ready, because the
  core's unit fixtures need no start; every root starts it.
   the optional-answer facts departure remains `jig/docs/design/client/domain/contract/README.status.md`, 3.
   the undocumented root methods remain `README.status.md`, 4.
   the child-timer departure remains `README.status.md`, 4.1.
   the acceptance-capacity question remains `settings.status.md`, 2, 6 and 8.

