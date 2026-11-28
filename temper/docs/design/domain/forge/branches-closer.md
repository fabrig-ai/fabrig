# A goal's closer

## 1. A goal's closer

A goal from a template that names it is ended by temper's goal closer
(jig's `domain/tasks/lifecycle-endings.md`, 3): when nothing is left for its agent to
judge, jig makes the closer as the goal's own delegate, and the closer
does with no model what a goal's run did once its plan's changes had
ended: it settles the goal's branch, lands it, and reports. The goal's
agent wakes for judgement alone (`../../draft/archive/goal-runs-itself.md`,
section 6). A goal's run that calls `finish` rather than `wait` parks
all the same, its result kept on the goal's record, so its closer
comes and lands its branch, and the report carries what the run said
(jig's `domain/tasks/lifecycle-endings.md`, 3, *A finish parks it*).

- **The forge's procedure 6,** the goal closer, beside the change (2),
  the landing (3), the after-landing (4) and the status (5) procedures.
  Its task's words are jig's fixed words and it takes no parameters: it
  reads its goal, its requester, from the store, and the goal's
  changes, its delegates and theirs. A template names it as `ended_by`
  `{"closer": {"connector": 1, "code": 6}}` (`../../protocol/configuration/stock.md` and `../../protocol/configuration/reference.md`
  ). Its state is a row of the forge's, kept under its
  task's number as a change's is, and restored with it (`effects.md`, section 2).
- **Its steps, in order,** each a change of its own, made as its
  delegate and waited for:
  1. **Close,** where the goal holds a goal branch and the engine
     declares the close kind: once nothing is pending and the goal's
     landed changes have not been closed, the closer makes the step,
     reading nothing of the repository first; its job finds the close
     command in its checkout of the branch's tip, its `.temper/hooks/close`,
     or, where the tip holds no `.temper/hooks/`, its project's
     `close_job`, none there ending the step with no change
     (`../../protocol/hooks.md`, 4.2 and 5;
     `../../protocol/configuration/engine.md`). The step is a change
     into the goal's branch whose producer is the close job, kind 4
     (`../worker/jobs.md`), as a catch-up's is the merge job: its bytes
     parameter 17 the close request (`../../protocol/tooling.md`, 4.2;
     `change/README.md`, 1),
     naming the goal and each change of the goal's tree whose landing
     branch is the goal's branch, with how each ended, the closer's own
     changes left out; its words `Settle goal G's status.` The change
     makes its branch at the goal's branch's tip, its producer the job,
     no agent, and lands as any change into the branch does; nothing
     staged ends it with no change. A refusal of the close's check holds
     it at once, a check that broke holds it once its job's tries are
     spent, and a check that found the goal outdated ends it with no
     change (`change/repairs-releases.md`, 1), for the closer to catch
     the goal up from main and close again (*A close found outdated*,
     below). Where the engine declares no close kind, or the goal holds no
     goal branch, this step is skipped. A close step superseded by more
     landed changes is made again for its new change.
  2. **Land,** where the goal holds a goal branch whose tip its base
     does not hold: the goal's landing, a change on the goal's held
     branch into its base, as the goal's `"branch": "mine"` would make
     it (`change/README.md`, 1), made by the closer on the goal's behalf: its branch the
     goal's, held by the goal and not by the closer or the change; its
     landing branch the repository's (`adoption.md`, 12); its words the
     goal's title, the first line of the goal's words, then a blank
     line and `Draft: PATH`, where the goal's words hold a line
     `Draft: PATH`, as a goal started from a draft's do, and nothing
     more where they hold none. CI, the queue, taking the base,
     a conflict's resolution, a CI repair and the after-landing
     verification are what they are for any change, and the step waits
     for the landing's result: at cleared, repaired or not, or, where
     the project's kind 4 says `wait: landed`, at landed, so that the
     goal ends as its branch lands into its base
     (`../after-landing.md`, section 6). A goal with no goal branch, or whose base
     already holds its tip, skips it.
  3. **Report:** the closer's result, below, and its end: done, which
     ends the goal (jig's `domain/tasks/lifecycle-endings.md`, 3).
- **What its goal's report leaves holds it first.** A goal's own
  record of what it has not done is its report's `Left:` line
  (`../agent/agents.md`, 3, *A goal's report*), kept as its charter
  says: each part it planned and has not cut into changes, and what it
  leaves undone, with why, and no line where nothing is left. A goal
  that ends done while that line stands ends with its work missing and
  its design describing it as built, so the closer reads the line
  before it makes its first step, or reports where it makes none, and
  again before any step it makes once new work has ended (*New work
  waits for, and reopens, the steps*, below):
  - *The read:* the newest note in the goal's scope whose description
    opens `report: `, by `changed` then number, and its `Left:` line,
    each as the client takes them (`../agent/agents.md`, 3): the last
    line whose start, past spaces, is `Left: `, its words past that,
    trimmed. No such note, or none with a `Left:` line, passes, so a
    charter that asks for no report meets no check. The same read takes
    the note's `Asked:` block (`../agent/agents.md`, 3, *What its request
    asked for in its result*), for the report alone: it holds nothing. A read that fails
    is made again after a wait, as an unknown containment is (below).
  - *Held:* a `Left:` line no release has passed holds the closer,
    before it makes any step, its words `the goal's report leaves work
    undone: WORDS`, WORDS the line's words, cut from their end where
    they pass what a hold's words hold and marked `…`. It escalates to
    the goal as any of its holds does (above). The goal cuts what the
    line names, its changes the closer's new work, and takes it off the
    line; or, for a part it drops, has its design's status record it as
    not built and gives why on the line; then releases.
  - *Released:* the release is the goal's word that what the line still
    names stays undone. The closer goes on to its steps, waiting first
    for any new work as ever, and holds on the line no more in its
    life; it still reads the line before each step new work makes, for
    its report. One release, final, keeps the goal's word its own and
    the closer out of a loop over wording.
  - *On its row:* the line and the `Asked:` block it last read, each
    none where it read none, and
    where its check stands, unread, held or released, so that a restart
    neither holds again past a release nor loses what its report
    names. They are added fields of the closer's row in `temper-store`,
    a registered family (jig's `protocol/store/layouts.md`, 1), read as
    unread and none in a row stored before.
  - *Why the close refuses, not the result alone:* the hold reaches the
    goal while its branch is unlanded and its judgement is to be had,
    so an uncut part is cut into the same landing, or dropped with its
    design's status saying so, before its base takes a design that
    describes it as built. A result naming the line alone would end the
    goal done with that design landed, the repair left to its
    requester, and a goal outside a milestone has no one above it to
    catch it. temper reads no word of a plan: what is left is the
    goal's own line, written as its charter says.
- **What wakes the goal meanwhile** is what wakes it for any delegate:
  a step's change held past its bounds, its resolution or repair
  failing, escalates past the closer, a procedure, to the goal (jig's
  `domain/tasks/proposals.md`); under `wait: cleared`, a red after
  landing, while the landing is verified, is news for the goal, whose
  tree the landing is in (`../after-landing.md`, section 7), and under
  `landed` none comes, the goal having ended at its landing. A step's change that ends without
  landing (cancelled, failed) is made again, or holds the closer,
  `ClosingStep`, by its attempts (below), its words naming the step,
  the change and why, which escalates to the goal: its release makes
  the step's change again, and its cancel ends the closer, the goal's
  branch kept. A question any of its open steps' changes passes up
  reaches the goal through the closer (below).
- **Its changes' questions go up to the goal.** The closer makes each
  step's change with questions `up` (`change/questions.md`, 1; `"questions":
  "up"` in the change it makes, whatever the project's questions), so that
  the change's agents, the land step's CI repair and conflict resolution
  and either step's repair, are delegated with the Message family and
  may ask; their change passes each question up to its requester, the
  closer, one at a time. The closer, a procedure, decides nothing: it
  relays as a change does, one level up, and its goal answers as it
  answers its own changes.
  - *Its open steps:* the step whose change it runs, and its landing
    while the landing's change is live. A landing that new work
    interrupts with a status settlement stays open as its change goes
    on (*New work waits for, and reopens, the steps*, below), so while
    that settlement runs both steps are open, and once it has ended
    the landing alone, until its change ends. A step whose change has
    ended is open no longer. Each open step's change is the closer's
    to relay for, whichever of them the closer runs.
  - *Taken:* at a step, a question in its inbox from an open step's
    change is taken and kept on its row: its asker, the asker's number
    for it, the asker's step, its words as they came, and where it
    stands (taken, or asked with the number of the closer's own
    question). One question stands at a time, the one the closer
    passes up; the others wait on its row in the order taken, and the
    first waiting stands once the one before is settled, its answer
    relayed down or dropped (below). Each open step's change passes up
    one at a time, so with two steps open at most one waits; a second
    from a change whose first still stands waits in turn as any does. A question from any other task, which no open step is
    left to relay for, is answered, in the answer form to its number,
    `This question reached no one: the goal's closer decides nothing
    and relays only the questions its open steps ask.` Any other
    task's words, and an answer that matches no standing question, are
    answered in words naming what the closer takes and what is open
    (`change/notices.md`, 3).
  - *Why one at a time:* the closer matches the question it passes up
    to the credit jig admits it with (*Room and admission*, below), and
    one asked of its goal at a time keeps that match whole. Two
    options were turned down: asking the goal several at once, which
    needs a match per question, and answering a second open step's
    question with the canned words, which leaves a landing's repairs
    unanswered whenever a settlement runs during it.
  - *Passed up:* the closer asks its goal in the question form, its
    words `Change C, the goal's STEP, asks:`, a blank line, then the
    question's words whole, cut from their end to the message bound
    and marked `…`; C the change's task number and STEP the asker's
    step, `landing` or `status settlement`. The goal is woken by it as
    by any question (jig's `domain/tasks/messages.md`, 3) and reads it
    labelled with how to answer, the closer as its target.
  - *The answer relayed down:* the goal's answer, in the answer form to
    the closer's question, is taken at the step that reads it and
    relayed to the change in the answer form to the asker's number, its
    words whole, cut from their end to the message bound and marked
    `…`, as the change cuts what it relays down
    (`change/questions.status.md`, Departs). The change relays it to its
    own asker, whose run goes on from where it parked; the standing
    question clears, and the first waiting one stands, passed up at
    the closer's next step, which its standing makes due at once.
    Words from the goal answer nothing.
  - *Its open steps' words,* their changes' notices of an asker that
    ended (`change/questions.md`, 1), are passed up to the goal as
    words, `Change C: ` then the words, cut and marked as above.
  - *A change that ends first,* with its question on the closer's
    record, ends it there: the closer drops it and tells its goal once,
    in words, `Change C ended with its question open; it reached no
    one: LINE` where the goal was not yet asked, a waiting question
    told so as its turn to stand comes, and, where it was asked, takes
    and drops the goal's answer when it comes, saying `Change C
    ended; your answer to question Q goes no further: LINE`, Q the
    closer's question and LINE the question's first line; the first
    waiting stands after either. A question is its asker's, not the
    step's: a step's change that ends drops its own alone, and another
    open step's question stays where it stands or waits.
  - *Room and admission:* these messages share the step's `tasks.batch`
    with the step's other messages; what finds no room stays on the
    record and makes the next step due at once. The record is saved
    only once jig admits the step, as a change's relay is
    (`change/questions.md`, 1), and the question passed up at a step is
    asked with the number of the credit jig admits that step's
    question with. The goal's answer held here for want of room tells
    the goal so, and again once it is relayed down; the stored relay
    keeps since when (`change/notices.md`, 1 and 2).
  - *Reporting waits* while a question stands or waits, or a notice is
    still to send, as for any of the closer's retained messages.
  - *Kept across a restart:* the questions, standing and waiting, with
    where each stands, are stored with the closer's row (`effects.md`,
    section 2) and restored with it, the waiting ones a field of the
    stored relay that defaults to none (`ClosingRelay`'s `waiting`, a
    field of the store's registered family, so no version moves); a
    row stored before reads as none standing and none waiting, and one
    stored with a standing question alone as none waiting.
  - The closer has no question allowance of its own: the change counts
    each question against `forge.change_policy.questions` and holds
    past it (`change/questions.md`, 1), and that hold escalates past
    the closer to the goal as any of its holds does (above). The
    waiting list has one slot per possible open step, two in all, and
    the store bounds it at that count. A procedure step consumes its
    offered inbox prefix, so a question beyond the waiting bound gets
    the same canned answer. The inbox still bounds retained replies.
- **A landing that ended is never the end.** When the land step's
  change ends cancelled or failed, the closer first reads afresh
  whether the base holds the goal's branch (the land step's own test,
  above): where it does, someone or something landed it meanwhile, and
  the closer goes on to report, its landing part `The landing: none,
  its base already held the goal's branch.`; where it does not, it
  makes the next landing or holds, by its attempts (below). A release
  reads that again, and either reports or makes a new landing change,
  never holding again on the landing that ended: a hold names a change
  still to be made, never one that has ended.
- **Attempts, and the project's bound.** Each change the closer makes
  for a step is an attempt at it, and one that ends cancelled or failed
  counts against the project's `landing_attempts`
  (`../../protocol/configuration/engine.md`), 3 when absent: the close
  step's and the land step's each counted alone, over the closer's
  life. A step's change that ends so:
  - *for its branch's reason* holds the closer, whatever its count,
    since another attempt at the same branch would meet the same: its
    state, as it became `Ended`, was held for repairs past their bound
    (`Repairs`), conflict resolutions past theirs (`Resolutions`), a
    person's rejection of a required gate (`Rejected`), its pull
    request closed without a merge (`PullClosed`) or its branch missing
    (`BranchMissing`), the change's holds (`change/repairs.md`). The
    change keeps the hold's cause its state was in as that state becomes
    `Ended`, as `Change.ended_held`, an `Option<Cause>`
    (`change/repairs-releases.md`, 2), none where it
    was not held, so that its reason outlives the hold. The store keeps
    it in the stored state it already has, an ended state with that
    one hold (`ChangeState`'s `holds`), so that its format stays and
    every row stored before reads as none. An ended change takes no
    hold after its end (a relay refused, a bound passed): its state
    stays `Ended`, so that stored form is its ending hold's alone, and
    a row stored before with a hold over its ended state reads as
    ended in that hold.
  - *for any other reason* (a cancel while it went on, a lost or failed
    run, a stall, any other hold, or none) is made again with no hold,
    as a release makes it, waiting for new work first and, for a
    landing, reading containment first, while fewer than
    `landing_attempts` of the step's changes have ended so; at the
    bound, it holds the closer.
  - A release makes one attempt more, past the bound too, and its end
    counts as any. A `landing_attempts` of 1 holds on every ended
    change.
  - A close step's change the closer superseded for new work (*New
    work waits for, and reopens, the steps*, below) is no attempt: the
    closer keeps the changes it superseded with its row, as `Closer`'s
    `superseded`, a field of the store's registered family that
    defaults to none, so no version moves, and leaves them out of the
    count, and, as its own, out of the goal's changes.
  - The hold's words name why, after the change's ending:
    `, held for its branch: WHAT`, WHAT `repairs`, `conflict
    resolutions`, `a rejection`, `its pull request closed` or `its
    branch missing`; or `; attempts ended without landing: K, the
    project's bound L`. So `the landing T41 ended cancelled; attempts
    ended without landing: 3, the project's bound 3`.
- **Its authority across attempts.** The closer's budget and task
  count (jig's `domain/tasks/lifecycle-endings.md`, 3) fund each attempt up to
  the bound:
  - *Spend:* each step's change is given half of what the closer has
    left, its budget less what it spent, what its ended delegates spent
    and what it holds reserved. An ended change's unspent share goes
    back to the closer as it ends (jig's `domain/authority-budgets.md`, section
    1), so an attempt costs the closer what it spent, never its share.
  - *Lent to its goal's run:* while a run of its goal holds the
    closer's funds (jig's `domain/tasks/lifecycle-endings.md`, 3), a step the
    closer cannot fund waits, with no hold, keeping the step to make;
    that run's answer gives the funds back and makes the closer due,
    and the step is made then. So a release the goal's run makes takes
    effect once that run has answered, and never holds the closer for
    funds the run holds.
  - *Tasks:* a change's subtree makes at most its share of the room,
    sized for the bound. The room is the fewer of the tasks the closer
    and its goal have left to make (jig's `domain/authority.md`,
    section 3: a lifetime count, which an ended attempt never gives
    back). The step's attempts left are `landing_attempts` less the
    step's changes ended without landing, and at least 1. The change's
    `delegation.tasks` is the fewer of what the forge's change policy
    gives a change (`forge.change_policy.tasks`, with its `questions`
    beside where a foreman answers them,
    `../../protocol/configuration/limits.md`) and the room divided by
    one more than the attempts left, less one for the change itself;
    and at least 1.
  - *Short:* a step the closer cannot fund or fit while no run of its
    goal holds its funds holds the closer, its words `the goal closer
    cannot delegate its next change within its authority: ` and what it
    lacks: `spend: A left, its change's half must pass M`, A what it
    has left and M the deployment's minimum run spend; `tasks: R left,
    its change needs 2`; `depth`; or `its change's terms`, for any
    other refusal. Its release reads afresh and makes the step where it
    now fits.
- **New work waits for, and reopens, the steps.** A change the goal
  makes while its closer lives is the closer's dependency (jig's
  `domain/tasks/lifecycle-endings.md`, 3). The closer, kept, makes no new step while it
  is pending, and a step's change already made goes on: a landing into
  whose branch the new work lands takes it as its new head (`change/drift.md`, 1). Once
  the work has ended, the closer reads afresh, its goal's report first
  (*What its goal's report leaves holds it first*, above): a change of the goal that
  landed into the goal's branch since its close step makes the close
  step again, beside a landing whose change is still live, which stays
  open meanwhile, its questions relayed (*Its open steps*, above), and
  a landing whose result is in while the goal's branch
  has moved past what it landed makes the land step again; then it
  reports.
  - *A close step's change that runs no close job,* held on its job's
    failure or released and waiting (`change/repairs-releases.md`, 1),
    makes none while the work is pending, whoever releases it. Once the
    work has ended, where a change of the goal landed into the goal's
    branch since that close step, the closer supersedes it: it cancels
    it, its reason `superseded by the goal's new work: changes N, M
    landed into its branch`, N and M those changes' task numbers,
    ascending, and makes the close step again at once, its request
    naming them. Where none landed, it wakes one that waits, which
    decides afresh, and leaves one held for its goal's release, the
    escalation the goal already holds.
  - *Why* (2026-11-24): a goal that makes a repair or a catch-up and
    then releases its held close, as its charter says, or leaves it
    held, has its close made once, after the work has landed, with a
    request that names it. A held close step counted as pending before,
    so nothing reopened it once the repair landed, and a release made a
    fresh job at once with the old request, at a branch the repair had
    not reached.
- **A close found outdated** (`change/repairs-releases.md`, 1,
  *Outdated*) is no attempt. With no new work pending or landed since
  its close step (else the close step is made again for it, as ever),
  the closer catches the goal up from main, once per close request:
  - *The catch-up:* a change into the goal's branch made as a goal's
    catch-up is (`updates.md`, 1), its commit main's tip as the close's
    change read it (`CloseAt`'s landing commit), its words `Catch goal G
    up from main at M.`, questions `up`, funded as the other steps are,
    one attempt left. The merge job looks for its hook in its checkout
    (`../../protocol/hooks.md`, 4.2 and 4.3); the closer does not read
    the hook first. Landed, or ended with no change, the catch-up makes
    the close step again at once, with the same request.
  - *Held at once,* `ClosingStep` on the step and its change, escalating
    to the goal, its words cut to the hold's bytes, G and M 12 hex
    digits, F the close's finding as its words give it:
    - found outdated again after the catch-up, same request: `the close
      step's change TN found the goal's branch at G outdated again, after
      its catch-up from main at M, change TK, landed: F. Catch the goal up
      from main with a change naming main's tip and release the closer,
      or release it to close again as it stands.`;
    - the catch-up ended without landing: `the catch-up from main at M,
      change TK, ended WHY, after the close step's change TN found the
      goal's branch at G outdated: F. Catch the goal up from main with a
      change naming main's tip and release the closer, or release it to
      catch up again.`, WHY as the other holds word an ending (*Attempts,
      and the project's bound*, above);
    - none it can make: `the close step's change TN found the goal's
      branch at G outdated: F; the closer can't catch it up from main:
      WHY. Merge main into the goal's branch and release the closer to
      close again.`, WHY `the engine declares no merge job kind` or `its
      authority does not delegate to the merge job`.
  - *Released,* the held step's change is made again, after any new
    work: the close, which holds at once again if outdated for a request
    already caught up, or the catch-up.
  - *Kept* on its row: the request it last caught up for (`Closer`'s
    `caught_up`) and its catch-up changes, its own and out of the goal's
    changes; fields added with defaults, so no version moves. Per
    request, not per tip, since the catch-up moves the tip itself.
  - *Why* (2026-11-27, `../../draft/test-budgets.md`, 3): the update the
    check asks for is one the closer can make on a goal's own catch-up
    path, so no one waits for it; a second finding, or a catch-up that
    can't land, is past what another update mends.
- **A catch-up only for its close:** the closer makes one only for a
  close found outdated (above); a goal it ends makes one only for a
  change that needs it (`branches.md`, 1).
- **Whether the base holds the goal's tip** is read afresh before a land
  step is made and before the closer reports: the goal branch's tip, or
  the merge a squash landing recorded for it, against the base's tip,
  by the ancestry range (`keeping-up.md`, 3, every containment reader), so a goal
  branch far ahead of and behind its base still lands. Contained, the
  land step is skipped; not contained, it is made. Unknown, the closer
  makes no step and reads again after a wait: `forge.client.backoff`
  first, doubled after each unknown in a row, capped at
  `forge.client.poll_max`, as the client's own reads and polls back off
  (skein's `forge/client.md`, sections 4 and 5). A known answer ends the
  run of waits, and a restart starts it again from `backoff`.

**The report.** The closer's result, a report in the words of the
goal's contract, from what the store and the forge already hold, each
part in this order, each line as written here:

```
Goal TG: TITLE
Landed:
- TN TITLE: pull request #P into BRANCH
  Left: LINE
  LINE
Ended without landing:
- TN TITLE: WHY
Asked: WORDS
  LINE
Left by the goal: WORDS
The landing: pull request #P, merged into BASE as COMMIT; repairs TN, TN; resolutions TN.
CI selection:
- pull request #P: LINE
The goal's finish:
  LINE
  LINE
```

- **The heading:** the goal's task number and title, the first line of
  its words.
- **Each change of the goal's tree that landed,** the closer's own
  left out, by task number, ascending: its title (its result's title,
  or else its words' first line), its pull request's number (its
  result's resource), the branch it landed into, and its
  result's `Left` section word for word, each of its lines indented two
  spaces: the lines of its result's words from the first line whose
  first word, after any `#`, `*` or spaces, is `Left` followed by a
  colon or the line's end, to the line before the next line opening
  with `#` or the words' end; `Left: none given` where it has none.
  The part is left out where no change landed.
- **Each change of the goal's tree that ended without landing,** the
  closer's own left out, ascending: why, `cancelled`,
  `failed: ` then its failure's first line, or `no change: ` then its
  result's first line. The part is left out where none did.
- **What the goal's request asked for in its result:** the `Asked:`
  block its report last read (*What its goal's report leaves holds it
  first*, above): its first line from `Asked: `, then each later line
  as written, indented two spaces, as a change's `Left` section's are.
  The part is left out where it read none, so a goal whose report
  answers nothing reports as it would without the part.
- **What the goal left:** the `Left:` line its report last read
  (*What its goal's report leaves holds it first*, above), its words
  whole, on one line; left out where it read none.
- **The landing:** its pull request, the base, its merge commit (the
  first line of its result's words), and the task numbers of the
  repairs and the resolutions its change made, each list left out where
  empty; `The landing: none, its base already held the goal's branch.`,
  `The landing: none, no goal branch.`, or `The landing: none, PULL
  ended WHY.` otherwise.
- **Each pull into the base's CI selection:** for each pull request of
  the goal's tree that landed into the base, the repository's default
  branch (the landing, each repair into the base its goal made, and for
  a goal with no goal branch each of its changes), the selection line
  its CI's jobs'
  logs begin with: the first line of a job's log, read by its start
  (skein's `forge/vocabulary.md`, 1.2, `Job`'s `Start`, its first 4,096 bytes),
  when it begins `selection: `, the line whole, cut to 256 bytes; the
  first job of the pull's merged head, in the forge's order, that has
  one gives it; `unknown` where no job's log reads one, a log not read,
  a job with no identity, or a forge that names no job among them. The
  part is left out where no pull landed into the base.
- **What the goal's run finished with:** the goal's parked result
  (jig's `domain/tasks/lifecycle-endings.md`, 3, *Its parked result is
  its closer's to report*), read from the goal's record as the closer
  reports, so a later finish's stands: its words, each line indented
  two spaces as a change's `Left` section's are; a failure's reason
  after `failed: `; an approval's words where it has any, and a job's
  none. The part is left out where the goal's record keeps none, as
  for a goal whose runs only parked, or its words are empty.
- **Cut to the contract's bound** as any report is: where the whole
  passes the contract's words bound, the changes' `Left` sections are
  cut first, from the last change's back, each cut one replaced by `  Left: [N
  bytes cut]`, the goal's own line and its `Asked:` block never among them; then, if it still passes, the report keeps its first
  bytes that fit, at a UTF-8 boundary, and ends with `[N bytes cut]`,
  N what it dropped, so that the goal's finish, the last part, loses
  its end first, and the `Asked:` block, behind only the heading and
  the changes, loses its end only once every part after it is gone.
