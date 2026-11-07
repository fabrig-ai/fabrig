# tools: status

2026-11-07. Remaining gaps between temper's code and [tools.md](tools.md).

Sections 5.2 and 6.1 have no known gap in next-run bounds explanations,
hello figures, charter-bearing shortfalls or their JSON. The waiting
tables check each named ctl call through the parser, each help page by
lookup and the program-login command against its configured prefix.
Host waits name no command. The settings reference's non-waiting help
pointer remains deferred in section 14.

Unlisted sections have no known gap, including 6.3's watch-capacity
diagnostic and deadline words, and 8.2's retry spans. Sections 7's key
refusals, 6.1's unpinned account wait and failure-log directions, 5.5's
setup ending and 5.3's effort check against `me` have no known gap.
Sections 5.3 and 6.2's publication words are in `project amend`'s help
and successful answer's stderr, as text or a `may_show` JSON object.
The help walk gives them before its command. Ctl asks nothing before
acting: an owner who skips the help can first meet them after publication.
The preceding acknowledgement remains deferred in section 14.

Section 7 has no known gap in distinguishing same-key replay, a retained
decision's `decided first:` at exit 0, and an unknown proposal's exit 2
with "this proposal isn't waiting; it may have been decided or replaced."
The tools' settings table checks both accepted-decision words and the
unknown-proposal explanation; the generated help states the same limit.

- **Departs:** 7, from `ui-goal-asks-2`'s requested `decided first:` on a
  replaced proposal: jig retains no decision for an automatically
  replaced task proposal. The answer of 2026-11-07 keeps `Unknown` and
  asks for these plainer words, declining an archive or new decider
  for this ctl edge because spend binds. Section 14 carries that later work.

- **Not yet:** 8.2, 11.2: the user's `ctl-waits` request asks for the
  long-chat wait with a fake model refusing its resume, concurrent reads and
  a same-key retry, with the outcome captured at both sides. The joined
  `tests/client` story `kept_message.rs` extends a conversation and fills its
  inbox, captures Busy and the client's retry, then frees room and checks
  the answer and same-key replay. It disables run answers and drives no
  model's resume refusal. The bounded retry fix covers that full-inbox
  cause, but the reported 23.2 s success and repeated 30 s waits have no
  reproduced cause on record; the settings-effort tests likewise guard an
  existing final refusal, without reproducing that walk's silence. The next
  QA walk remains the requested proof (section 14).
- **Not yet:** 5.3: in text, `settings show charter N` prints no line per part with its `on` and bytes, and `settings show template N` no line for a later choice.
- **Not yet:** 5.5: area 2 cannot distinguish a refused creating offer's
  capacity cause from its role cause: jig's `me` carries absence alone
  (`jig/docs/design/client/domain/contract.md`, 5.3). Its diagnostic names
  the role and `documents.lines` as checks, without asserting either cause.
- **Not yet:** 5.5: `temper help task`'s Limits still says temper does not
  let ctl take a task or hand it back, where area 3's `task take` and
  `task hand-back` do so for a person task. The `plainer-panels` request of
  2026-11-06 left the line, saying nothing else changes, so the page's
  `taken by:` stands beside it.
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
- **Departs:** 5.5: the `ctl-words` request asks that the owned forge's
  protection warning either prove false and go, or temper do what it asks.
  The warning is true: the bot is a writer and cannot read protection.
  The plan suppresses it at `forge setup` instead, retaining JSON's unknown
  protection and `repository adopt`'s warning. Giving the bot admin would
  let its tokens change protection; reading through the site admin remains
  deferred in section 14.
- **Departs:** 5.3: the `ctl-words` request asks that an unsupported effort's
  refusal name both effort and model. Where `me` exposes no such model,
  the item still goes to the engine and its refusal names the model alone:
  the wire's `Refusal::Llm` carries no effort. The plan limits the local
  check to models `me` shows to avoid changing that protocol; extending the
  refusal is deferred in section 14.
- **Departs:** 5.1: `project` is area 1's (`jig/rows.rs`
  `PROJECT_SHOW`, `PROJECT_FUND`, `PROJECT_MAKE`), not area 3's; the placeholders add
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
