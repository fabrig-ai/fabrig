# tools: status

2026-11-27. Where the tree's code lags or departs from [tools.md](tools.md).



- **Not yet:** 1: the table's amount type; 3: `project fund`'s AMOUNT and answer in ctl's amount [in flight: change 7671].



- **Not yet:** 1: the groups `finding` and `need`, FINDING's form and the placeholders `AREA` and `PARTY`; `finding file`, `finding hand`, `finding route`, `finding backlog`, `finding drop` and `need ask` [in flight: change 749].

- **Not yet:** 1: the groups `qa` and
  `deployment`, absent from `temper-client-tools`' `table.rs` `GROUPS`.
   `repository receipts`, `qa marks`,
  `qa passes` and `hold list`.  `task move
  --under`, `task succeed --by`, `qa choose`, `task hold` and `project hold`.
   the missing area operations remain `areas.status.md` (Not yet, the whole document).
   the hello stop-bound units remain `results.status.md`, 1.
   the trailing-help command syntax remains `command-line.status.md`, 1.1.
  the missing joined ctl stories remain `testing.status.md`, 2.
  the real-loop connected-forge stories remain `testing.status.md`, 3.
- **Departs:** 1: `project` is area 1's (`jig/rows.rs`
  `PROJECT_SHOW`, `PROJECT_FUND`, `PROJECT_MAKE`), not area 3's; the placeholders add
  `HOST:PORT` and `URL`.  `settings show template|policy` takes
  `--revision` too; `landing show` takes `TASK [--project PROJECT]`, not
  `LANDING`; `landing list` takes no `--newer-than` or `--after`, and
  its `--ended` adds the ended to the live.  `task stop` places a
  hold over the task's subtree (`jig/rows.rs` `STOP`), as jig's `Stop`
  does (jig's `domain/people/acts.md`, 1), where its row holds the task;
  `hold lift` is keyed optionally (`Keyed::Optional`).  the question-number omission remains `results.status.md`, 1. the budget-print scaling remains `results.status.md`, 1. the pinned-account wait contradiction remains `results.status.md`, 1. the person-task amendment's missing arguments remain `results.status.md`, 1.  the authority-entry bound remains `bounds.status.md`, 1.
