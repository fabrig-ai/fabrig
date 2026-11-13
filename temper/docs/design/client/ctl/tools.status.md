# tools: status

2026-11-13. Where the tree's code lags or departs from [tools.md](tools.md).

- **Not yet:** 2: `landing history` [in flight: change 2508].



- **Not yet:** 2 and 3: `draft list`, `draft ready` and `draft not-ready`,
  absent from the table [in flight: change 665].

- **Not yet:** 2: `role list` and `role show`; 3: `role start` (`../../domain/agent/standing.md`, section 6).

- **Not yet:** 3: `project delete PROJECT`: the table's tool is `project remove`, its ask built through `me` by `shapes::remove_project` with the chat template's grace, so a number `me` no longer lists sends nothing and a rerun after the deletion never reaches jig's kept answer [in flight: change 634].
- **Not yet:** 2: `project show` of a deleted project prints `unknown` or `gone`, never `project 3 was deleted` [in flight: change 634].

- **Not yet:** 3: a settings file's landing rule has no `sets`, in
  `temper-settings-form` or `temper-client-tools`' `render_rule`.

- **Not yet:** 1, 3 and 5: the group `party`, `party make`, and the host's `sign-in --service` [in flight: change 724].

- **Not yet:** 1, 2 and 3: the groups `finding` and `need`, FINDING's form and the placeholders `AREA` and `PARTY`; `finding list` and `finding show`; `finding file`, `finding hand`, `finding route`, `finding backlog`, `finding drop` and `need ask` [in flight: change 749].

- **Not yet:** 1: the groups `qa` and
  `deployment`, absent from `temper-client-tools`' `table.rs` `GROUPS`.
  2: `repository receipts`, `qa marks`,
  `qa passes` and `hold list`. 3: `task move
  --under`, `task succeed --by`, `qa choose`, `task hold` and `project hold`.
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
