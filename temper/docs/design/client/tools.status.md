# tools: status

2026-11-06. Remaining gaps between temper's code and [tools.md](tools.md).
Unlisted sections have no known gap.

- **Not yet:** 7: `--key` with no name or a name outside the rule is refused with
  the bare usage line, and `temper help ctl` doesn't say the rule.
- **Not yet:** 5.2: `task show`'s text says `deployment show` for a host's wait
  (`temper-client-tools`' `render/document/llm.rs`, `wait_explanation`), and ctl has
  no such tool until area 3's is built (5.5), so that line names a command that
  isn't there yet.
- **Not yet:** 5.5: area 2 cannot distinguish a refused creating offer's
  capacity cause from its role cause: jig's `me` carries absence alone
  (`jig/docs/design/client/domain/contract.md`, 5.3). Its diagnostic names
  the role and `documents.lines` as checks, without asserting either cause.
- **Not yet:** 6.1: a task no account serves prints `Due to run` and `--json`'s
  `waits_on` `nothing`, against `waiting on: an account`.
- **Not yet:** 5.5: `temper help task`'s Limits still says temper does not
  let ctl take a task or hand it back, where area 3's `task take` and
  `task hand-back` do so for a person task. The `plainer-panels` request of
  2026-11-06 left the line, saying nothing else changes, so the page's
  `taken by:` stands beside it.
- **Not yet:** 5.5: `forge setup` ends with the line on protection it could not
  read.
- **Not yet:** 5.1: the groups `finding`, `qa`, `party` and
  `deployment`, absent from `temper-client-tools`' `table.rs` `GROUPS`.
  5.2: `repository receipts`, `finding list`, `finding show`, `qa marks`,
  `qa passes` and `hold list`. 5.3: `task move
  --under`, `task succeed --by`, `finding route`, `finding backlog`,
  `finding drop`, `qa choose`, `task hold` and `project hold`.
  5.5: area 2's procedure executor choice for `goal start`,
  `proposal withdraw`, `seen`, and `task amend`'s `--budget`; area 3's `party make`, `sign-in --service`, `deployment
  show`, `task move`, and the project's
  `policy`, `role` and `pool`; area 4's `create`, `propose` and
  `compound`; area 6's `forge forget`, with no request on the wire.
  6.1: a task wait's refusal by a worker's hello
  prints its stop bound as jig's nanoseconds, `allowed grace N ns`, not
  whole seconds (`render/document/llm.rs`, `HelloRefusal::StopBound`).
  8.1: `GROUP VERB --help`; `temper ctl` parses a
  leading `help` alone. 11.2:
  the joined story of a model's amendment (`tests/client`'s
  `amend_tools.rs` parses `--model` alone); the joined stories of
  `playground end`, `forge remove`, and a set up or a connect without
  FORGE, which `temper-client-tools`' own step tests hold alone; the
  joined story of a settings file of each kind proposed, shown, accepted
  and restored (`tests/client`'s `settings_tools.rs`,
  `each_kind_through_ctl_proposed_shown_accepted_and_restored`),
  ignored pending the restore request path; the joint component correlates
  forge setting reads, including historical revision values. 11.3:
  the real loop's connected Forgejo, its tokens connect, a replace of
  its git token and another user's tokens stopped.
- **Not yet:** 6.1: a hold for failed runs says nothing of where its failures
  are, and `temper help hold` doesn't say where the engine's log is.
- **Departs:** 5.1: `project` is area 1's (`jig/rows.rs`
  `PROJECT_SHOW`, `PROJECT_FUND`), not area 3's; the placeholders add
  `HOST:PORT` and `URL`. 5.2: `settings show template|policy` takes
  `--revision` too; `landing show` takes `TASK [--project PROJECT]`, not
  `LANDING`; `landing list` takes no `--newer-than` or `--after`, and
  its `--ended` adds the ended to the live. 5.3: `task stop` places a
  hold over the task's subtree (`jig/rows.rs` `STOP`), as jig's `Stop`
  does (jig's `domain/people.md`, 5.1), where its row holds the task;
  `hold lift` is keyed optionally (`Keyed::Optional`). 6.1: a party's wait
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
  so these lines can still say opposite things (section 14). A person task's
  `task amend` prints `task amend 41 --key KEY`, where that request has the act
  and its arguments, as an agent task's `--rev 8 CHANGE... REASON` does:
  `person_call` (`render/document.rs`) gives an answer alone its arguments. 12: an
  authority's entries are 14 (`limits.rs`), and the client's `projects` bound, 2, is
  not in the table.
