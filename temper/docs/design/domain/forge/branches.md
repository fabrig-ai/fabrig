# Branches

- **Named after the tree.** Every branch temper makes is under the
  repository's prefix (`temper/` by default, one deployment's on a
  repository: `adoption.md`) and its tree's root task:
  - a change's own branch: `temper/<root>/c<task>`;
  - a task's saved work: `<prefix><root>/s<task>` in each writable
    repository, using that repository's own prefix (`temper/` by
    default). The task is the running agent's, even when its workstream
    and delivery branch belong to its parent change. Saved work has its
    own branch, so it starts no CI and moves no pull request;
  - a landing's own branch, where its prepare job pushes each round's
    candidate: `temper/<root>/l<task>`, the landing's task, held by the
    landing from its making, deleted as a round ends unlanded and as
    the landing closes (`../landing.md`, 5.1);
  - a goal's own branch: `temper/<root>/g<task>`, the goal's number, in
    its project's home, held by the goal (1);
  - an agent's task's own branch, for runs that push outside a change,
    as a chat making a small fix does:
    `temper/<root>/r<task>`, in each repository its runs may write, the
    project's home among them, which the task holds from the commit that
    makes it. A chat hands it to the change task that lands it. A goal's
    runs push to its goal branch instead (3): a goal holding a run
    branch has nothing push it, and its close deletes it as below.

  A task made before its project's home was adopted holds no run branch
  there: its runs read the home without pushing, and its brief says why.
  A new task gets the branch.

  A later attempt of the same task reclaims its own writer slot, whether
  running or lost; its own earlier writer never holds it. An earlier
  attempt's reread cannot free the later writer, report drift for it or
  hold the later run. A failed reread holds the task as a resource
  unavailable, naming the task, run, branch, repository and read error.
  Another holder's contention names the resource and holder in a resource
  wait hold. Neither is a procedure hold: an agent task has no procedure.

  They are siblings under `temper/<root>/`, so no name is the prefix of
  another as git refs forbid, and one grant covers a tree's branches;
  a task's own branches are named in its batch symbolically and resolved
  once it has its number (jig's `domain/authority.md`, section 4).
- **Held by the task it is named after,** a change's by the change that
  lands it, a goal's own by its goal, and written by its delegates one
  at a time (jig's `domain/tasks/holds.md`, 3); a goal's own is written by
  its own runs too, beside them (3).
- **Landing branches are never made by a push.** A landing branch must
  exist when it is adopted or named; a goal's own branch (1) is made
  at its base's tip as its goal's hold is taken, as the taking's effect
  (jig's `domain/connectors.md`, 4.7), and a change into it reads it
  once it is made.
- **Deleted as their tasks close,** as the release of their holds (jig's
  `domain/tasks/holds.md`, 5): a landed change's branch, a cancelled one's,
  and a task's saved work. A failed change's branch is kept for a person
  to read, and deleted as its tree's root closes. A goal's own branch is
  deleted as its goal closes if a change landed it (its tip reachable
  from that change's landing branch), and otherwise kept for a person to
  read, until deleted by hand or by the deployment's retention. A change
  on its requester's held branch (`change/README.md`, 1) holds no branch and deletes none
  as it closes: the goal's landing leaves its goal's branch to the
  goal's own close. A goal's run branch goes as its goal closes. A
  release waits for a lost writer's reread, and fails only if that
  reread fails. A branch temper knows no head of once that reread is done (a change's branch before its first push, a run branch nothing pushed) is not deleted: its hold is dropped with no write, as a goal's unused run branch is.

A goal's release recognises its branch as landed from a change in the same
repository on that branch, in landed, red, cleared or uncleared state, whose
landed head equals the branch's last known tip. With no known tip or no
matching landed change it keeps the branch; a subsequent unlanded move keeps
it too. This uses the retained change and branch records, with no new
ancestry read (2026-11-04).

A task's saved work is named by the set its task's record keeps of the
repositories a next attempt would resume from (`../worker/checkouts.md`). As
its task's release begins, each branch of that set becomes a hold of the
closing task, with no writer, and goes as any hold does: deleted, a branch
already gone counting as deleted (skein's `forge/client.md`, section 7);
or, if its task failed and is not its tree's root, kept for a person to
read and deleted as the root closes. Such a hold has no holding in the
hub, and no head is read for it. A branch outside that set stays until
deleted by hand or by the deployment's retention, as does a save whose
terminal the engine never heard, its worker lost, and a branch the forge's
full holds left no room for at the close.

## 1. A branch per goal

Whether a goal's changes land into a branch of its own, which then lands
whole, or each into the shared base, is the plan's choice like any
other: a change's landing branch is its parameter. Landing each into the
base is the usual answer: plans meet early and in small pieces; a goal
that needs another's work has it once that lands; and repositories that
pin each other's commits (temper locks skein's in `Cargo.lock`) pin a
commit of the base, which no later squash of a goal's branch replaces.
A goal's own branch suits work likely to be dropped; a goal
checked whole before it lands (by an end-to-end run, by a review of the
whole); intermediate states that would break the base; and upstream
contributions (`README.md`, section 8). Such a branch is short-lived, and the goal's
own, never a change's:

- **Named by the goal's template,** or by the batch that makes the goal
  by hand, as the symbol *its own goal branch* (`README.md`, section 2), offered on a
  tracked task only, since its goal is the one its changes serve (jig's
  `domain/model.md`, 6.2). Resolved once the goal has its number to
  `temper/<root>/g<task>` in the project's home (`branches.md`). A goal
  cannot take a hold once active (jig's `domain/tasks/holds.md`, 2), so the
  choice is made with the goal.
- **Held by its goal for its whole life,** an exclusive hold that
  refuses, and made at the home's default branch's tip as the hold is
  taken, as the taking's effect. A making that fails for good holds the
  goal, its words naming the branch and what the forge answered.
  A making waits for the outbox's room and never fails for it: with the
  outbox full (`forge.entries`), it holds nothing and polls nothing, as a
  read answered `Busy` waits for the client's room (`keeping-up.md`, 5).
  It gives up its entry number while it waits, so it reserves nothing:
  a kept number would stop every later entry's handoff, and an outbox
  full of those would never free. Once an entry frees, the makings that
  wait go on in the order they began to wait, each taking a fresh number
  and reading the default branch's tip afresh, as a restart's making
  with no entry does; fresh numbers follow that order, so the client's
  entries stay in increasing order. A making that meets a full outbox
  again waits again, keeping its place.
  What no freed entry mends holds the goal at once, each cause in its
  own words after the branch's `could not be made: `: a repository not
  adopted, `the repository is not adopted`; one adopted to be read,
  `the repository is adopted to be read, not written`; one whose
  adoption takes no branch writes, `the repository's adoption takes no
  branch writes` (`adoption.md`, 2); and an entry number already in the
  outbox, `its outbox entry N is already in use`, N that number. Access
  is judged first, so a full outbox never hides a making that could
  never be sent.
- **Made whatever restarts.** temper alone makes and deletes a goal's
  own branch, so its making is conditional (`README.md`, section 4): while the goal
  holds the branch and none of its name exists, it is made, and never
  reset once it exists. A restart reads each held goal branch afresh
  (`README.md`, section 9): there, it is the goal's; missing, with its making's
  entry restored, that entry settles as its class says, its find
  answering not found sending it again at its commit; missing, with no
  entry (the restart cut its making before it committed one), the
  connector asks the root for a fresh entry number, as a taking's effect
  is given one, reads the default branch's tip and makes it there. A
  forge away while it is made is ridden (`keeping-up.md`, 5), never a failure. A goal
  held for its making that fails for good is made again when a person
  releases that hold: the release reads the branch, as a restart does,
  and makes it if it is still missing.
- **Written by change, and by the goal's design:** each of the goal's
  changes lands into it as its landing branch, and the goal's own runs
  push their design and status documents to it (3); code reaches it
  by change alone. An ordinary `change` call from the goal or its
  subtree defaults to the nearest applicable held goal branch; an explicit
  destination outside it refuses the whole call before delegation. This
  also holds for every member of a batch. The deliberate `"branch": "mine"`
  change lands that held branch whole into the base instead (`change/README.md`, 1).
  Each child merge is ordered by the branch's queue (`queues.md`
  ) and takes the branch's writer slot while in flight; the goal's own
  runs take no writer slot of it (jig's `domain/tasks/holds.md`, 3; 3),
  so merges into it go on while they run.
  A change whose landing branch is a goal's own, made by a task that is
  neither that goal nor one of its subtree, is refused at its batch,
  naming the goal.
- **What a goal writes goes straight to g,** where its charter's
  `modify` is set (`../agent/tools.md`): its runs are g's design writers
  (3), and `deliver` pushes their design to g, at any point in a run
  and as often as it likes, with no change to land it. A charter without
  `modify` leaves its goal's run to inspect its confirmed branch, as it
  starts there.
- **Landed whole** by a change into the base whose branch is the goal's
  own: made by the goal's closer (2) where the goal is ended by one,
  and otherwise by the goal on its held branch (`change/README.md`, 1); and deleted as the
  goal closes once that change has landed it (`branches.md`); otherwise
  kept as the goal closes. A goal takes the base only by a catch-up
  (`README.md`, section 6) for one of its changes that needs what the
  base gained since its branch was made, never to stay current
  (`../../draft/squash-in.md`, 4): its landing takes the base at the
  front of the base's queue, as any change does (`change/README.md`,
  2), and a conflict goes to the merge job and a resolution
  (`change/repairs.md`). A landing into the base that touches its open
  work reaches it as news (`README.md`, 5.3), for it to judge whether a
  change needs it. A project's rule may require or forbid a catch-up.

## 2. A goal's closer

A goal from a template that names it is ended by temper's goal closer
(jig's `domain/tasks/lifecycle.md`, 11): when nothing is left for its agent to
judge, jig makes the closer as the goal's own delegate, and the closer
does with no model what a goal's run did once its plan's changes had
ended: it settles the goal's branch, lands it, and reports. The goal's
agent wakes for judgement alone (`../../draft/goal-runs-itself.md`,
section 6).

- **The forge's procedure 6,** the goal closer, beside the change (2),
  the landing (3), the after-landing (4) and the status (5) procedures.
  Its task's words are jig's fixed words and it takes no parameters: it
  reads its goal, its requester, from the store, and the goal's
  changes, its delegates and theirs. A template names it as `ended_by`
  `{"closer": {"connector": 1, "code": 6}}` (`../../protocol/configuration/stock.md` and `../../protocol/configuration/reference.md`
  ). Its state is a row of the forge's, kept under its
  task's number as a change's is, and restored with it (`README.md`, section 9).
- **Its steps, in order,** each a change of its own, made as its
  delegate and waited for:
  1. **Close,** where the goal holds a goal branch and its project names
     a `close_job` (`../../protocol/configuration/engine.md`): a change
     into the goal's branch whose producer is the close job, kind 4
     (`../worker/jobs.md`), as a catch-up's is the merge job: its bytes
     parameter 17 the close request (`../../protocol/tooling.md`, 4.2),
     naming the goal and each change of the goal's tree whose landing
     branch is the goal's branch, with how each ended, the closer's own
     changes left out; its words `Settle goal G's status.` The change
     makes its branch at the goal's branch's tip, its producer the job,
     no agent, and lands as any change into the branch does; nothing
     staged ends it with no change. Where the project names none, or the
     goal holds no goal branch, this step is skipped.
  2. **Land,** where the goal holds a goal branch whose tip its base
     does not hold: the goal's landing, a change on the goal's held
     branch into its base, as the goal's `"branch": "mine"` would make
     it (`change/README.md`, 1), made by the closer on the goal's behalf: its branch the
     goal's, held by the goal and not by the closer or the change; its
     landing branch the repository's default branch; its words the
     goal's title, the first line of the goal's words, then a blank
     line and `Draft: PATH`, where the goal's words hold a line
     `Draft: PATH`, as a goal started from a draft's do, and nothing
     more where they hold none. CI, the queue, taking the base,
     a conflict's resolution, a CI repair and the after-landing
     verification are what they are for any change, and the step waits
     for the landing's result at cleared, repaired or not
     (`../after-landing.md`, section 6). A goal with no goal branch, or whose base
     already holds its tip, skips it.
  3. **Report:** the closer's result, below, and its end: done, which
     ends the goal (jig's `domain/tasks/lifecycle.md`, 11).
- **What wakes the goal meanwhile** is what wakes it for any delegate:
  a step's change held past its bounds, its resolution or repair
  failing, escalates past the closer, a procedure, to the goal (jig's
  `domain/tasks/proposals.md`); a red after landing, while the landing
  is verified, is news for the goal, whose tree the landing is in
  (`../after-landing.md`, section 7). A step's change that ends without
  landing (cancelled, failed) is made again, or holds the closer,
  `ClosingStep`, by its attempts (below), its words naming the step,
  the change and why, which escalates to the goal: its release makes
  the step's change again, and its cancel ends the closer, the goal's
  branch kept. A question its step change passes up reaches the goal
  through the closer (below).
- **Its changes' questions go up to the goal.** The closer makes each
  step's change with questions `up` (`change/questions.md`, 1; `"questions":
  "up"` in the change it makes, whatever the project's questions), so that
  the change's agents, the land step's CI repair and conflict resolution
  and either step's repair, are delegated with the Message family and
  may ask; their change passes each question up to its requester, the
  closer, one at a time. The closer, a procedure, decides nothing: it
  relays as a change does, one level up, and its goal answers as it
  answers its own changes.
  - *Taken:* at a step, a question in its inbox from its live step
    change is taken and kept on its row: its asker, the asker's number
    for it, its words as they came, and where it stands (taken, or
    asked with the number of the closer's own question). At most one
    stands, since its one live step change passes up one at a time; a
    second from that change while one stands, and a question from any
    other task, is answered, in the answer form to its number,
    `The goal's closer decides nothing; it relays its own change's
    questions to its goal.` Any other task's words are taken and
    dropped, as before.
  - *Passed up:* the closer asks its goal in the question form, its
    words `Change C, the goal's STEP, asks:`, a blank line, then the
    question's words whole, cut from their end to the message bound
    and marked `…`; C the change's task number and STEP `landing` or
    `status settlement`. The goal is woken by it as by any question
    (jig's `domain/tasks/messages.md`, 3) and reads it labelled with how
    to answer, the closer as its target.
  - *The answer relayed down:* the goal's answer, in the answer form to
    the closer's question, is taken at the step that reads it and
    relayed to the change in the answer form to the asker's number, its
    words whole, cut from their end to the message bound and marked
    `…`, as the change cuts what it relays down
    (`change/questions.status.md`, Departs). The change relays it to its
    own asker, whose run goes on from where it parked; the closer's
    record clears. Words from the goal answer nothing.
  - *Its step change's words,* its notices of an asker that ended
    (`change/questions.md`, 1), are passed up to the goal as words,
    `Change C: ` then the words, cut and marked as above.
  - *A change that ends first,* with its question on the closer's
    record, ends it there: the closer drops it and tells its goal once,
    in words, `Change C ended with its question open; it reached no
    one: LINE` where the goal was not yet asked, and, where it was,
    takes and drops the goal's answer when it comes, saying `Change C
    ended; your answer to question Q goes no further: LINE`, Q the
    closer's question and LINE the question's first line. The next
    step's change starts with none.
  - *Room and admission:* these messages share the step's `tasks.batch`
    with the step's other messages; what finds no room stays on the
    record and makes the next step due at once. The record is saved
    only once jig admits the step, as a change's relay is
    (`change/questions.md`, 1).
  - *Kept across a restart:* the question, with where it stands, is
    stored with the closer's row (`README.md`, section 9) and restored
    with it; a row stored before reads as none.
  - The closer counts no bound of its own: the change counts each
    question against `forge.change_policy.questions` and holds past it
    (`change/questions.md`, 1), and that hold escalates past the
    closer to the goal as any of its holds does (above).
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
    change keeps the hold its state was in as that state becomes
    `Ended`, as `Change.ended_held`, an `Option<Hold>`, none where it
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
  - The hold's words name why, after the change's ending:
    `, held for its branch: WHAT`, WHAT `repairs`, `conflict
    resolutions`, `a rejection`, `its pull request closed` or `its
    branch missing`; or `; attempts ended without landing: K, the
    project's bound L`. So `the landing T41 ended cancelled; attempts
    ended without landing: 3, the project's bound 3`.
- **Its authority across attempts.** The closer's budget and task
  count (jig's `domain/tasks/lifecycle.md`, 11) fund each attempt up to
  the bound:
  - *Spend:* each step's change is given half of what the closer has
    left, its budget less what it spent, what its ended delegates spent
    and what it holds reserved. An ended change's unspent share goes
    back to the closer as it ends (jig's `domain/authority.md`, section
    7), so an attempt costs the closer what it spent, never its share.
  - *Lent to its goal's run:* while a run of its goal holds the
    closer's funds (jig's `domain/tasks/lifecycle.md`, 11), a step the
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
  `domain/tasks/lifecycle.md`, 11). The closer, kept, makes no new step while it
  is pending, and a step's change already made goes on: a landing into
  whose branch the new work lands takes it as its new head (`change/README.md`, 3). Once
  the work has ended, the closer reads afresh: a change of the goal that
  landed into the goal's branch since its close step makes the close
  step again, and a landing whose result is in while the goal's branch
  has moved past what it landed makes the land step again; then it
  reports.
- **Never a catch-up:** the closer makes none; a goal it ends makes
  one only for a change that needs it (1).
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
The landing: pull request #P, merged into BASE as COMMIT; repairs TN, TN; resolutions TN.
CI selection:
- pull request #P: LINE
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
  (skein's `forge/README.md`, 3.2, `Job`'s `Start`, its first 4,096 bytes),
  when it begins `selection: `, the line whole, cut to 256 bytes; the
  first job of the pull's merged head, in the forge's order, that has
  one gives it; `unknown` where no job's log reads one, a log not read,
  a job with no identity, or a forge that names no job among them. The
  part is left out where no pull landed into the base.
- **Cut to the contract's bound** as any report is: where the whole
  passes the contract's words bound, the `Left` sections are cut first,
  from the last change's back, each cut one replaced by `  Left: [N
  bytes cut]`; then, if it still passes, the report keeps its first
  bytes that fit, at a UTF-8 boundary, and ends with `[N bytes cut]`,
  N what it dropped.

## 3. A goal's design writers

A goal's design goes straight to its goal branch. A run of the goal
whose charter sets `modify`, its goal branch made, confirmed and
undrifted, is a **design writer** of it: its workspace starts at g's tip
(or its saved work, or, where its run branch holds a head g lacks, g's
tip with that head merged in) and names g as both its push branch and
its landing branch, which the worker takes as a design delivery
(`../worker/checkouts.md`): Markdown only, g's moved tip merged first, a
fast-forward never forced. Its repository's other writes are as before.

- **Its first run among them.** A goal's run prepared while its goal
  branch is still being made (the taking's effect in flight, jig's
  `domain/connectors.md`, 4.7) or not yet confirmed (its head not yet
  recorded as temper's own) waits, as a write into a branch still being
  made does, and starts once g is made and confirmed, at g's tip, a
  design writer like any later run. A modifying goal holding g never
  starts on its base or with its workspace's writes read-only, so its
  first run can deliver the first wave's design before it cuts its
  changes. A making that fails for good holds the goal (1); a drifted
  g holds it (`change/README.md`, 3), and its run starts once a release settles it.
- **How it calls it:** smith's `deliver`, with a `title` and a `body`
  as a change's delivery has them, at any point in its run and as often
  as it likes; the project's checks run at it (`../agent/README.md`, section 5).
  It answers a delivery's receipt, `pushed <head> to branch <g>; your
  checkout stays detached, as expected`, or, where g had moved, `pushed
  <head> to branch <g>, merging its tip <tip> first; ...`; or a refusal
  the run settles and delivers again: a path that isn't Markdown, named,
  since code goes through `change`; files in conflict with g's moved
  tip, named, the merge left in its checkout; g kept moving. It is never
  stale, and a `save` there pushes nothing and is refused.
- **No writer slot.** A design writer takes no writer slot of g (jig's
  `domain/tasks/holds.md`, 3): g's hold keeps its design writers beside its
  slot, each a run's attempt, so merges into g, a landing's repairs and
  the goal's other writers go on while it runs. The hold keeps them with
  it in the store, so a restart keeps them.
- **Its pushes are g's own.** While g has a design writer, a forward
  move by temper's identity is that writer's push, never drift, as
  while any writer holds a branch (`change/README.md`, 3); a move by another identity, or
  not forward, is drift as before. When the design writer's run
  answers, or its attempt is lost, g is read afresh, as a lost writer's
  branch is (`branches.md`): a tip that descends from the head temper last
  knew becomes g's recorded head, as the branch's own; one that does not
  is drift. What the run reports pushing is not taken as the head, since
  merges land beside it.
- **Written while any writes it.** The forge's keep counts g as written
  while a design writer lives, a run holds its slot, or a merge into it
  is in flight; each of them ending leaves g written while another
  stands.
- **Changes on g take it.** A change on g (the goal's landing,
  `"branch": "mine"`, the closer's land step) treats a live design
  writer as its requester's own run (`change/README.md`, 3): a forward move is its new
  head, checked again, never "branch moved".
- **Its brief says so**: one line, that `deliver` pushes the
  checkout's Markdown straight to the goal branch, merging its moved tip
  first, and that code goes through `change`.
- **The goal's own runs alone.** A designer the goal makes besides
  itself stays a change (`../../draft/goal-as-designer.md`, section 2): its
  producer delivers its change's branch, and its design reaches g by
  that change's pull request, as any change's does. A push to g from
  anywhere but a design writer holds as drift (`change/README.md`, 3).

## 4. Open questions

- **Saved branches outside the task's saved set** (`README.md`, 11): a terminal the
  engine never heard, a repository no longer in the set, or full holds at
  close can leave a branch on the forge. Finding these needs a branch
  inventory beyond the set a next attempt resumes from. Default: manual
  deletion or deployment retention; a failed descendant's known saved
  work stays with its root until that root closes.
