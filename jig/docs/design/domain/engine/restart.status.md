# restart: status

2026-11-13. Where the tree's code lags or departs from [restart.md](restart.md).

- **Departs:** the whole document: `Core::new` without a start is ready, because the
  core's unit fixtures need no start; every root starts it.
   the optional-answer facts departure remains `jig/docs/design/client/domain/contract/README.status.md`, 3.
   the undocumented root methods remain `README.status.md`, 4.
   the child-timer departure remains `README.status.md`, 4.1.
   the acceptance-capacity question remains `settings.status.md`, 2, 6 and 8.

- **Not yet:** 1, *Settling before the stop*: the core has no `Event::Settling` and no `Core::settling()`; a root that settles before its stop still has its fleet place waiting attempts and jobs, and its timers activate tasks [in flight: change 2291]
