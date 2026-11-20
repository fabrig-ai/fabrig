# restart: status

2026-11-20. Where the tree's code lags or departs from [restart.md](restart.md).

- **Not yet:** steps 4 to 6 and *The start's interface*: the script reads every connector afresh and settles every outbox, as `ReadAfresh` and `SettleOutbox` phases, before it opens; no `Ask::CatchUp`, `catch_up_done` or `caught_up` [in flight: change 11034].

- **Departs:** the whole document: `Core::new` without a start is ready, because the
  core's unit fixtures need no start; every root starts it.
   the optional-answer facts departure remains `jig/docs/design/client/domain/contract/README.status.md`, 3.
   the undocumented root methods remain `README.status.md`, 4.
   the child-timer departure remains `README.status.md`, 4.1.
   the acceptance-capacity question remains `settings.status.md`, 2, 6 and 8.

