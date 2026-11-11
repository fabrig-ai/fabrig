# tools: status

2026-11-11. Where the tree's code lags or departs from [tools.md](tools.md).

- **Not yet:** 2 and 3: `draft list`, `draft ready` and `draft not-ready`,
  absent from the table [in flight: change 665].
- **Not yet:** 2: `role list` and `role show`; 3: `role start` (`../../domain/agent/standing.md`, section 6) [in flight: change 108].

- **Not yet:** 3: `project remove PROJECT`, absent from `temper-client-tools`' table, so no ctl call makes jig's `RemoveProject` [in flight: change 35].

- **Not yet:** 3: a settings file's landing rule has no `sets`, in
  `temper-settings-form` or `temper-client-tools`' `render_rule`
  [in flight: change 202].

- **Not yet:** 1: the groups `finding`, `qa`, `party` and
  `deployment`, absent from `temper-client-tools`' `table.rs` `GROUPS`.
  2: `repository receipts`, `finding list`, `finding show`, `qa marks`,
  `qa passes` and `hold list`. 3: `task move
  --under`, `task succeed --by`, `finding route`, `finding backlog`,
  `finding drop`, `qa choose`, `task hold` and `project hold`.
   the missing area operations remain `areas.status.md` (Not yet, the whole document).
   the hello stop-bound units remain `results.status.md`, 1.
   the trailing-help command syntax remains `README.status.md`, 6.1.
  the missing joined ctl stories remain `testing.status.md`, 2.
  the real-loop connected-forge stories remain `testing.status.md`, 3.
- **Departs:** 3: the `ctl-words` request asks that an unsupported effort's
  refusal name both effort and model. Where `me` exposes no such model,
  the item still goes to the engine and its refusal names the model alone:
  the wire's `Refusal::Llm` carries no effort. The plan limits the local
  check to models `me` shows to avoid changing that protocol; extending the
  refusal is deferred in `README.md`, section 11.
- **Departs:** 1: `project` is area 1's (`jig/rows.rs`
  `PROJECT_SHOW`, `PROJECT_FUND`, `PROJECT_MAKE`), not area 3's; the placeholders add
  `HOST:PORT` and `URL`. 2: `settings show template|policy` takes
  `--revision` too; `landing show` takes `TASK [--project PROJECT]`, not
  `LANDING`; `landing list` takes no `--newer-than` or `--after`, and
  its `--ended` adds the ended to the live. 3: `task stop` places a
  hold over the task's subtree (`jig/rows.rs` `STOP`), as jig's `Stop`
  does (jig's `domain/people/acts.md`, 1), where its row holds the task;
  `hold lift` is keyed optionally (`Keyed::Optional`).  the question-number omission remains `results.status.md`, 1. the budget-print scaling remains `results.status.md`, 1. the pinned-account wait contradiction remains `results.status.md`, 1. the person-task amendment's missing arguments remain `results.status.md`, 1.  the authority-entry bound remains `README.status.md`, 9.
