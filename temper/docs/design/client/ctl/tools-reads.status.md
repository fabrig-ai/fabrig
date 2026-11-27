# tools-reads: status

2026-11-26. Where the tree's code lags or departs from [tools-reads.md](tools-reads.md).


- **Not yet:** 1: `draft list`, `draft ready` and `draft not-ready`,
  absent from the table [in flight: change 665].


- **Not yet:** 1: `role list` and `role show`; 3: `role start` (`../../domain/agent/standing.md`, section 6).

- **Not yet:** 1: `landing times` [in flight: change 2671].

- **Not yet:** 1: `settings upgrade FILE`, absent from the table [in flight: change 21449].


- **Not yet:** 1: `project show` of a deleted project prints `unknown` or `gone`, never `project 3 was deleted` [in flight: change 634].

- **Not yet:**  the groups `qa` and
  `deployment`, absent from `temper-client-tools`' `table.rs` `GROUPS`.
  1: `repository receipts`, `qa marks`,
  `qa passes` and `hold list`.  `task move
  --under`, `task succeed --by`, `qa choose`, `task hold` and `project hold`.
   the missing area operations remain `areas.status.md` (Not yet, the whole document).
   the hello stop-bound units remain `results.status.md`, 1.
   the trailing-help command syntax remains `command-line.status.md`, 1.1.
  the missing joined ctl stories remain `testing.status.md`, 2.
  the real-loop connected-forge stories remain `testing.status.md`, 3.
- **Departs:**  `project` is area 1's (`jig/rows.rs`
  `PROJECT_SHOW`, `PROJECT_FUND`, `PROJECT_MAKE`), not area 3's; the placeholders add
  `HOST:PORT` and `URL`. 1: `settings show template|policy` takes
  `--revision` too; `landing show` takes `TASK [--project PROJECT]`, not
  `LANDING`; `landing list` takes no `--newer-than` or `--after`, and
  its `--ended` adds the ended to the live.  `task stop` places a
  hold over the task's subtree (`jig/rows.rs` `STOP`), as jig's `Stop`
  does (jig's `domain/people/acts.md`, 1), where its row holds the task;
  `hold lift` is keyed optionally (`Keyed::Optional`).  the question-number omission remains `results.status.md`, 1. the budget-print scaling remains `results.status.md`, 1. the pinned-account wait contradiction remains `results.status.md`, 1. the person-task amendment's missing arguments remain `results.status.md`, 1.  the authority-entry bound remains `bounds.status.md`, 1.
