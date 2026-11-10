# A goal's tree runs itself

Draft, 2026-11-10, from a session with the user that day. It finishes what
`goal-as-designer.md` began: its section 7 still wakes the goal's agent to
make the landing, make the close-out, finish, and catch the goal's branch up
with main, and its section 10 leaves "the landing and the close-out made up
front" open. Those are mechanisms, and code does them here. The user's rule:
a model for judgement, code for mechanisms. This draft wins over
`goal-as-designer.md`, sections 2, 7 and 10, and over
`reference-code-production.md` where they differ, until its plan folds it in.

## 1. In one page

- **The goal's agent wakes for judgement alone:** a question its changes
  pass up, an escalation or a failure, a red, a person's words, and a
  designer's result where there is one (section 6).
- **Code closes the goal.** When the goal's tree has nothing left to await,
  a closer runs in place of a wake: the project's `close` command settles
  the goal's branch, a landing takes it into main whole, and the goal ends
  with a report temper writes (sections 2 to 5).
- **No catch-ups.** The landing takes main at the front of main's queue, as
  any change does (`../domain/forge.md`, 8.2), and a conflict goes to the
  merge job and a resolution. A goal no longer chases main while it runs.
- **No close-out run.** The goal's design writes the status files up front,
  each entry it will close tagged with the change that closes it, and the
  `close` command settles the tags (section 4). Nothing reads the changes'
  reports to write status afterwards.
- **A lean agent:** the coordinator's charter loses every instruction for
  the steps above, and its report note keeps decisions, risks and open
  items, not a narrative (section 7).

## 2. A closer, in jig

- **Today** a result-ended agent task that parks with nothing left to await
  is woken with the `NothingAwaited` notice, "Finish with your result"
  (jig's `domain/tasks/lifecycle.md`, 2). For a goal, that run has nothing to judge:
  it makes the landing, later the close-out, and finishes.
- **Then** a task may name a **closer** when it is made: a procedure
  executor (jig's `domain/tasks/lifecycle.md`, 3) that jig runs, as the task's own
  delegate, where it would have numbered that notice. jig decides when, by
  the same rule as today; the application's procedure decides what. A task
  with no closer keeps today's notice.
- **Its success ends the task** with the result the closer supplies, by the
  task's own `ended_by` (5.8 there): no agent run claims it. Its failure, or
  a hold it can't clear, is an escalation to the task, which wakes for it
  as for any.
- **A new delegate reopens the tree.** If the task, woken by an escalation,
  makes new work, the closer is cancelled or waits, and runs again once
  nothing is left to await. jig owns this rule; the application only names
  the procedure.

## 3. temper's goal closer

- **Named by the goal's template:** a goal template's work, kind 11
  (`../protocol/configuration.md`, 13.2), gains `closes`, which makes the
  goal with temper's closer. Template 4, the reference's and the stock's,
  sets it. A goal without it keeps today's wakes.
- **In order,** each a step of one procedure:
  1. **Close:** where the project names a `close` command (section 4), a
     change into the goal's branch whose producer is that command's job,
     as a catch-up's is the merge job (`../domain/forge.md`, 8.4). No
     model. Where it names none, this step is skipped.
  2. **Land:** the goal's landing, the change `"branch": "mine"` the goal's
     agent makes today (`../domain/forge.md`, 8.1 and 11.1), made by the
     closer on the goal's behalf, with the goal's title and draft path as
     its words. CI, the queue, taking main, a conflict's resolution and a
     CI repair are what they are for any change.
  3. **Report:** the closer's result, section 5.
- **What still wakes the goal:** the landing held past its bounds, a
  resolution or a repair that fails, a red after landing: each an
  escalation or news, as section 6 lists.

## 4. Status written up front, settled by code

- **The project's `close` command,** beside `merge` in
  `../protocol/tooling.md`, section 4, named like the merge job's: a path in
  the repository, run in a job's directory with no model and no
  credential. Its request names the goal and each of its changes by task
  number, with how each ended: landed into the goal's branch, or not. It
  commits its edits on the goal's branch; exit 0 with nothing to commit is
  success.
- **fabrig's close command** settles in-flight status entries. `DESIGN.md`,
  section 2 gains a tag: a **Not yet** entry the goal's design writes for a
  section one of its changes will build ends `[in flight: change N]`. The
  command deletes each entry whose change landed, and drops the tag from
  each whose change ended without landing, so the gap stays recorded. An
  entry with a tag naming no change of the goal is the command's failure.
  The citation checker (`tools/citations/`) accepts the tag.
- **Written by the design, up front:** the goal's first run writes the
  design, its status entries tagged, the draft's status (**Design lags**,
  **Design departs**, **Open**), the execution file's state, and
  `STATUS.md`'s summaries, as they will stand once the goal lands. A
  draft the design now carries in full is deleted or archived in the same
  delivery (`DESIGN.md`, section 4).
- **Departures:** a coding agent never departs from the design without
  asking. Where an answer departs, the goal's agent writes the **Departs**
  entry in the design revision it delivers with that answer.
- **No close-out** of charter 7's kind runs for a goal whose template
  closes it.

## 5. The goal's report, written by temper

The closer's result, a report in the goal's contract's words, from what the
store and the forge already hold:

- **Each change that landed into the goal's branch:** task, title, pull
  request, and its result's `Left` section, word for word.
- **Each change that ended without landing,** and why, from its result.
- **The landing:** its pull request, merge commit, and each repair and
  resolution that cleared it.
- **Each pull into main's CI selection:** the selection line a CI job's log
  begins with, read by code; a log without one says unknown.
- **Cut to the contract's bound** as any report is, the changes' `Left`
  sections last.

## 6. The goal's wakes

| Wake | The goal's agent |
|---|---|
| a question a change passes up | settles it, amends, makes a change, or asks its person |
| an escalation, a failure, the closer's included | releases, amends, cancels, or passes it up |
| a red, as news, on its branch's tier | a repair change, or a note naming a flake |
| a designer's result, where it subscribed | makes the family's changes |
| a person's words, an amendment | as today |

- **Words from a task wake it and are read.** Every kind its wake policy
  lets through is offered to the run it wakes: the coordinator's parts
  cover `words` (jig's `domain/tasks/messages.md`, 3 and 6).
- **No subscription to main's landings** for catch-ups. Reds still reach the
  goal through its branch's tiers (`../domain/forge.md`, section 10).

## 7. The coordinator's charter

Charter 5, in the stock (`../protocol/configuration.md`, 13.4) and the
reference (13.9 there):

- **Deleted:** the last result's landing, the landing result's close-out,
  pasting reports into it, the close-out result's finish, reading CI logs
  for their selection, the check of every report against its acceptance
  before landing, and catching up with main.
- **Kept:** the first run (survey, design with tagged status, the tree),
  questions, escalations, reds, designers.
- **The report note:** one line of headline, then decisions with their
  times, risks and open items, one line each. The tree and the closer's
  report hold the rest.
- **Charter 7,** the reference's close-out, goes from the reference.

## 8. Tests

- **Focused:** a goal whose last change lands runs its closer with no agent
  run: the close job's commit, the landing, and the goal ended with temper's
  report; a close command that fails, and a landing held, each wake the goal
  once; a goal woken by an escalation that makes a change has its closer
  run again after it; a task with no closer keeps the `NothingAwaited`
  notice; fabrig's close command over landed, cancelled and unknown tags;
  a task's words to a coordinator are offered to the run they wake.
- **Fuzzy:** the close command over random status files and change ends:
  every landed change's entries gone, every other entry kept, nothing else
  changed.

## 9. Out

- A goal across repositories (`whole-story.md`, 5.2).
- Repairs of reds without the goal's agent.
- Batching the answers to questions, as the pipeline batches them.

## 10. Open questions

- **A closer's landing on a goal with no changes:** none, and the goal ends
  with a report saying so. Default: as written.
- **Where the selection line comes from** when a project's CI isn't the
  reference's: unknown, said so. Default: as written.
