# A task's lifecycle: dependencies and endings

## 1. Kept by a dependency

A task its requester ends is not kept: it may start and run as ever,
and only its ending differs (9).

An active task with a pending dependency is kept: it stays active, with
its holds, its delegates and what its executor has done, and goes no
further until each is done. The tasks child keeps the promise, not the
executor: a charter or a procedure may also behave, but the refusals
hold if they don't. While kept, a task:

- **starts no run.** An agent task is never due. A run already live
  hears the amendment that kept it (`changes.md`), and may finish its turn.
  Its last pending dependency done or removed is a notice that wakes it
  (`messages.md`, 1), and its next run goes on from what the last one left, its brief
  saying what it waited for (../engine/briefs.md, section 1).
- **asks no effect that waits for dependencies.** Its connector says
  which kinds of effect do (../connectors-effects.md, 1.1). One asked while a
  dependency is pending is refused, naming it.
- **ends no way but failed or cancelled.** An agent's run that finishes
  done, with a result its contract takes, parks instead: its saved state
  is kept, no try is spent, and the task's activity names what it waits
  for (../engine/executors.md, 4). A procedure's step that ends it done is refused
  whole (../connectors-procedures.md, 1.2). A person's answer is refused, naming the
  dependencies (../people/README.md, section 7). A failure or a cancel ends it as
  ever.

What reaches it:

- **Messages wait in its inbox,** as for a waiting task, and wake it, as
  its policy says, once it is no longer kept. A live run hears what its
  policy lets through, as ever, and a procedure steps as ever (3).
- **Decisions wait above it.** A proposal or an escalation that would
  wait at a kept task waits at the nearest holder above it instead,
  since a kept task starts no run to decide it (`proposals.md`).
- **A cancel's grace** reaches a live run, as ever. A kept task with no
  live run, a kept person task, and one whose run parks during the grace
  cannot finish on their own, and are cancelled at once (`changes.md`).

What it keeps:

- **Its holds and pool slots.** Releasing them would run their
  connectors' release effects, and taking them again would mean queueing
  whole. So a kept task keeps them, and the waits they make for others
  are checked to close no circle (`holds.md`, 7).
- **One rule for going on.** Whether an active task may go on, and why
  not, is decided in one place of the tasks child, its reason typed: a
  pending dependency is one. A further reason to keep a live task from
  going on, its state and holds kept, is one more reason there, not
  another rule.

**Owing a question keeps it too.** A task its requester made owing it
a question (its spec's `asks`, set at creation and never amended, as
an application marks work to be agreed before it is done) owes one until
its first question-form message to its current requester is admitted.
Admission sets the durable `TaskRecord.asked_requester` latch, which is
never cleared: answering or retiring that question does not make the
obligation owed again. Before it has asked, its run that finishes done
parks with its saved state, consumes a run try, and hears a notice
naming what it owes that wakes it at once. Its next run goes on from
what the last one left with that notice to read. Exhausting its run
tries ends it failed, saying so (7); each run also spends within the task's budget.
Once it has asked, its own question keeps it, as below.

**Its own question keeps it too.** A task that asked its requester a
question, in the question form, still unanswered, is kept as by a
pending dependency, the reason naming the question: it ends no way but
failed or cancelled, and its run that finishes parks without spending a
try, whatever the form of its finish, `failure` among them, its activity
naming the question it waits for. A run that has asked has no reason to
finish until it is answered, so its finish is not taken: its result goes
with its attempt, and its next run hears the answer. This is a rule of
any task with its own question open, and of the run's own finish alone:
a run that fails without one (its tries, its budget or its account
spent), a cancel and a procedure's step that fails end it as ever. Only an
answer in that question's number releases this keep and wakes it (`messages.md`, 1),
letting its next run go on from what the last one left. An instruction
amendment, a note, or the person's answer to someone else is not that
answer. Its delegates, its holds and its other messages go on as for
any kept task. A question that ends unanswered lifts the keep: its asked
task ending with it open, or its cancel, wakes the asker with a notice
naming it. A move of the asker to a new requester (`changes.md`) carries its
open questions to that requester in the move's decision, each keeping
its number and activity line, as a successor's transfer does, so a goal
moved to its party asks that party directly and stays kept. A question
it asked through a person task, in words, is that task's, and detaches
as `changes.md` says; it keeps nothing.

**A credit names the question's parties and line.** Each open question
keeps a `QuestionCredit { number, answerer, asked, line }` in its asker's
record: `answerer` is the party allowed to use its reserved answer room,
`asked` the logical addressed party, each a `Party`; `line` is the
position of the question's activity line in the asker's record. Words
and time are read from that line, never copied into the credit; the
line does not name the addressed party.

**Question notices say which words they carry.** `QuestionEnded` carries
empty words, its kind naming the question's number. `QuestionAnswered`
names the asker in its word's `from` party and carries the person's
answer words within 256 bytes, including a
`[N bytes cut]` line when cut, with the prefix ending at a UTF-8 boundary.
Each received question reserves those 256 bytes beside its notice slot.
Stored notices with empty words still restore. `QuestionOwed` carries the
instruction: "You owe your requester a question: ask your plan with message, as a question, and wait."
Unanswered questions retired together produce one `QuestionEndedDue`
batch per asker; the core numbers each notice once.

## 2. Ended by its requester

A task is ended by its result, by its requester or by its closer (11),
or it stands, ended only by a cancel (`standing.md`, section 2):
its `ended_by`, fixed at its making. A result ends it by default. Its requester may end it
only if it is an agent's conversation with a person, its requester, or
a gathering a person requested (`gatherings.md`);
making it refuses any other shape, and a restart takes what was made
(../engine/restart.md). A gathering has no run, so what follows is an
agent's: its requester's `Conclude` is its word, and it ends done once
that is said and its delegates have ended (`gatherings.md`, section 3). It changes no right
to cancel. It travels with the task from the client's making, through
storage and preparation, to its run's charter (../hosts/charters.md, 4),
whatever the charter the task's agent shares with others.

A task its requester ends:

- **starts and resumes as ever.** Only a pending dependency keeps it
  from a run (8).
- **takes no run's finish.** A run that finishes, with any result,
  a failure among them, or with follow-ups, parks before its result is
  judged. The result and its follow-ups are discarded, and no try is
  spent. What the run spent, saved and committed stays, with its turns,
  its read fences, its delegates and its history. The discard writes no
  line of its own: it is the run's ordinary turn end, and the committed
  reply is the history's line. The fences of saved work, of charge and
  of messages are checked first, as for any finish.
- **ends as its requester says,** or by cancel. While it closes, a
  cancel keeps its partial result. A narrowing run goes Due, as ever,
  and a held task stays held.

The kept rule of 8 holds both facts, the dependency and this one,
apart: a task that is held from active keeps its requester's ending,
and one that is held from closing does not. A failure that is not a
finish, the run's own, keeps its retry policy whoever ends the task.
`End::Exceeded` is such a failure, not a finish: it holds at once by
6, retaining the requester ending through the hold.

## 3. Ended by its closer

A closer ends a task when nothing is left for its agent to judge: jig
decides when, by 2's rule, and the application's procedure decides
what, so that a model wakes for judgement alone and code does what is
mechanical once the task's work is done (temper's `domain/forge/branches.md`,
2, a goal's landing).

- **Named at its making:** `ended_by` `Closer { connector, code }`, a
  procedure executor (3), given by its batch entry or its template
  (`../../protocol/configuration-templates.md`, 2), and fixed: no amendment changes
  it. Only an agent task that is no conversation takes one; any other
  shape is refused as `ended_by` is (`README.md`, section 4). Its making is refused,
  beyond authority, where the task's own delegation does not name the
  procedure (../authority.md, section 3), as a delegate's would be.
- **A finish parks it.** Its run is offered `finish`, as a run of a
  task its result ends is (`../hosts/charters.md`, 4), and a run that
  answers finished, done or failed, with follow-ups or none, is judged
  as any finish: the fences of saved work, of charge and of messages
  first, then its result against the task's contract
  (`lifecycle.md`, 7), a result the contract refuses failing the
  attempt as ever. A result the contract takes is committed as parked,
  as a park by `wait` is: its turns first, then its saved state, no try
  spent but where the task owes its requester a question (1); its
  follow-ups and its ask to cancel its delegates discarded, so that its
  delegates are still awaited; and its result kept, its parked result
  (below), in place of any kept before. The rule of 2 then finds what
  it finds after any park: nothing left to await makes its closer, at
  once or when the last awaited thing ends (below), and a closer
  already live goes on. A narrowing run's finish is not taken, as ever,
  and a failure that is not a finish keeps its retry policy. So no run
  ends the task past its closer: the closer's success, its failures
  past the task's run tries, a run's own failures past its tries, or a
  cancel do.
- **Its parked result is its closer's to report.** The task's record
  keeps it, `TaskRecord.parked_result: Option<TaskResult>`, none until
  a run's finish is parked, while the task lives and through restart
  (`../../protocol/store/records.md`, 1); jig reads no word of it. The
  closer's procedure reads it from its task's record when it reports,
  so that what the run said reaches the closer's result, and so its
  requester (temper's `domain/forge/branches.md`, 2, *The report*).
- **Why a finish parks, not a run offered none.** A run offered no
  `finish` would need a mode of smith's beside `Finite` and `Open`, a
  change to the worker's wire bridged both ways, and would lose what
  the run would have said, where a model's slip calls `finish` all the
  same. A finish parked is the rule 2 already holds for a task no
  result ends, judged first and its result kept, so the closer starts
  as it does after a `wait` and reports what the run wrote.
- **Made where the notice would be.** When the nothing-left-to-await
  rule (2) finds the task due again, and no closer of it is live, the
  tasks child asks the core to make its closer instead: one procedure
  task, the task's own delegate, made by the task as one batch with no
  run: its executor the named procedure, its words the fixed `Close
  your requester: nothing more will come.`, no parameters or inputs,
  its contract the task's own, its authority the task's own whole, one
  level shallower in delegation, and its budget `AsAllowed` within the
  task's (../authority-budgets.md, section 1). The task keeps the closer's number
  while it lives, stays idle, numbers no notice and spends no run try.
  A refused making (a budget or a limit reached) holds the task, its
  words naming the closer's refusal (6).
- **Also when its last awaited thing ends while it is idle.** A
  delegate's end, an answer or a watch's end that leaves an idle task
  nothing to await, by the same conditions, makes its closer too, with
  no run between: a last result under `last_or_failure` (`messages.md`, 3) does not
  wake it, where a failure still does, and the result stays in its
  inbox, offered to its next run if one comes.
- **Its funds follow its task's runs.** The closer's budget grows by
  what its task has left whenever the task is idle or waiting and the
  closer is not closing: after a delegate the task funds ends, after a
  run of the task answers, and after an amendment of the task. A run of
  the task, as it is claimed, takes back what the closer holds
  unreserved, the closer's budget becoming what it and its ended
  delegates spent and what it holds reserved, so that the task's
  judgement is never starved by its closer. Each of those growths makes
  the closer due (`Changed`, as a procedure's changed inputs do) where
  it is idle, whether it gave anything or not, so that a step the
  closer could not fund while a run held its funds is decided again
  once that run has answered (temper's `domain/forge/branches.md`, 2).
- **Its success ends the task:** the closer ending done, its result is
  the task's: checked against the task's contract as a run's finish is
  (7), and the task closes with it (1), no run claiming it. A
  result the contract refuses holds the task, `Contract`, its words
  naming the closer. Its requester hears the task's result, never the
  closer's.
- **Its failure is the task's to judge.** A closer held, by a failure
  past its tries or a hold of its own (6), escalates to the task as
  any delegate does (`proposals.md`), and the escalations of the closer's
  own delegates pass the closer, a procedure, to the task. A closer
  that ends failed or cancelled is a delegate's result, which wakes the
  task by its policy; one that ends failed spends a run try, as a park
  with nothing to await does, so that a closer failing every time
  ends the task failed, saying so (7). The task's next park with nothing to await
  makes a closer again.
- **New work keeps it waiting.** A delegate the task makes while its
  closer lives (by a run, an accepted proposal or a batch) becomes a
  dependency of the closer in the same commit, added by jig as an
  amendment adds one (`changes.md`), so the closer is kept (8) until
  that work has ended: it steps, and cannot end done, nor so end the
  task, before the work. Its procedure reads the work as any kept
  procedure reads its dependencies, and decides what its own steps need
  again. A batch that would make a circle of waits through the closer is
  refused as any is.
- **Closed with the task:** a task that closes otherwise (a cancel, a
  failure past its tries) closes its closer as any delegate (1). Restart takes the closer as any task, and the task's record its
  number.
