# tools: status

2026-11-11. Where the tree's code lags or departs from [tools.md](tools.md).

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
   area 2's procedure executor choice for `goal start`,
  `proposal withdraw`, `seen`, and `task amend`'s `--budget`; area 3's `party make`, `sign-in --service`, `deployment
  show`, `task move`, and the project's
  `policy`, `role` and `pool`; area 4's `create`, `propose` and
  `compound`; area 6's `forge forget`, with no request on the wire.
   a task wait's refusal by a worker's hello
  prints its stop bound as jig's nanoseconds, `allowed grace N ns`, not
  whole seconds (`render/document/llm.rs`, `HelloRefusal::StopBound`).
   `GROUP VERB --help`; `temper ctl` parses a
  leading `help` alone. 
  the joined story of a model's amendment (`tests/client`'s
  `amend_tools.rs` parses `--model` alone); the joined stories of
  `playground end`, `forge remove`, and a set up or a connect without
  FORGE, which `temper-client-tools`' own step tests hold alone; the joint component correlates
  forge setting reads, including historical revision values. 
  the real loop's connected Forgejo, its tokens connect, a replace of
  its git token and another user's tokens stopped.
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
  `hold lift` is keyed optionally (`Keyed::Optional`).  a party's wait
  prints `you: question of task 44 · 1 waiting`, with no question's number,
  where the section's sample and `temper help task` have `question 3`, and
  the `plainer-panels` request has that page say what the line prints: jig's
  `WaitsOn::Party` carries the asked task, its kind and how many wait, and no
  number (`render/document.rs`, `waiting`). A task's spent and budget print
  divided by 100, `spent 1.20 of 5.00` (`render/document.rs`, `amount`), as
  `temper help budget` says, where the section has them never scaled. A task
  pinned to an account that holds no source for this work waits on that account
  with the why `Empty` (jig's `select_source`), and prints `next run: account 43
  source unknown: empty`, then `no account serving this work yet`
  (`render/document/llm.rs`, `wait_explanation`), then `waiting on: Nothing
  awaited`: the `wait-words` request of 2026-11-06 has `waiting on:` name an
  account when the next run has none to run on. Its plan held "no account" to
  the wait that printed the words the request drops, to keep this one's number,
  so these lines can still say opposite things (`README.md`, section 11). A person task's
  `task amend` prints `task amend 41 --key KEY`, where that request has the act
  and its arguments, as an agent task's `--rev 8 CHANGE... REASON` does:
  `person_call` (`render/document.rs`) gives an answer alone its arguments.  an
  authority's entries are 14 (`limits.rs`).
