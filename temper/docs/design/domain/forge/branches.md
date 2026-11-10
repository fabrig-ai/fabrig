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
  - a landing's own branch, for a pin-only member: `temper/<root>/l<task>`,
    the landing's task, held by the landing (`../landing.md`, 4.1);
  - a goal's own branch: `temper/<root>/g<task>`, the goal's number, in
    its project's home, held by the goal (1);
  - an agent's task's own branch, for runs that push outside a change,
    as a chat making a small fix does and a goal's runs do:
    `temper/<root>/r<task>`, in each repository its runs may write, the
    project's home among them, which the task holds from the commit that
    makes it. A chat hands it to the change task that lands it; a goal
    keeps it, and a change on it lands what it holds (`change/README.md`, 1, `branches.md`, 1).

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
  at a time (jig's `domain/tasks/holds.md`, 3).
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
  goal's own close, and a change on a run branch leaves it to its goal. A
  goal's run branch goes as its goal closes, what it holds that g lacks
  with it (`README.md`, section 11). A release waits for a lost
  writer's reread, and fails only if that reread fails.

A goal's release recognises its branch as landed from a change in the same
repository on that branch, in landed, red, cleared or uncleared state, whose
landed head equals the branch's last known tip. With no known tip or no
matching landed change it keeps the branch; a subsequent unlanded move keeps
it too. This uses the retained change and branch records, with no new
ancestry read (2026-11-04).

A task's saved work is named by the set its task's record keeps of the
repositories a next attempt would resume from (`../worker.md`, section 5). As
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
- **Written by change alone:** each of the goal's changes lands into it
  as its landing branch. An ordinary `change` call from the goal or its
  subtree defaults to the nearest applicable held goal branch; an explicit
  destination outside it refuses the whole call before delegation. This
  also holds for every member of a batch. The deliberate `"branch": "mine"`
  change lands that held branch whole into the base instead (`change/README.md`, 1).
  Each child merge is ordered by the branch's queue (`queues.md`
  ) and takes the branch's writer slot while in flight; the goal's own
  runs read it and take no writer slot of it (jig's `domain/tasks/holds.md`, 3),
  so merges into it go on while they run.
  A change whose landing branch is a goal's own, made by a task that is
  neither that goal nor one of its subtree, is refused at its batch,
  naming the goal.
- **What a goal writes goes by its run branch,** where its charter's
  `modify` is set (`../agent/tools.md`). Its run pushes to its own run branch
  (`branches.md`), taking that branch's writer slot and never g's. A change
  with `"branch": "run"` lands what that branch holds into g, by g's queue,
  like any change (`change/README.md`, 1), and a conflict goes to the merge job and a
  resolution. A change `after` it starts early on the branch's pushed head
  (`change/questions.md`, 1). The first run starts at g's tip; with the run branch at a head, a
  later run starts from g's tip with that head merged in (`../worker.md`,
  section 5), so its delivery fast-forwards the run branch: a head g holds
  is no merge, one that merges cleanly is committed before the run starts,
  and a conflict is left in its checkout for it to settle. A run resuming
  saved work starts from it, as any does. A charter without `modify` leaves
  its goal's run to inspect its confirmed branch, as it starts there.
- **Landed whole** by a change into the base whose branch is the goal's
  own: made by the goal's closer (2) where the goal is ended by one,
  and otherwise by the goal on its held branch (`change/README.md`, 1); and deleted as the
  goal closes once that change has landed it (`branches.md`); otherwise
  kept as the goal closes. A goal its closer ends makes no catch-ups:
  its landing takes the base at the front of the base's queue, as any
  change does (`change/README.md`, 2), and a conflict goes to the merge job and a
  resolution (`change/repairs.md`). A goal with no closer may still take the base's
  landings that touch its area by catch-ups (`README.md`, section 6), which it makes
  as it hears them (`README.md`, 5.3). A project's rule may require or forbid it.

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
     (`../worker.md`, 4.4), as a catch-up's is the merge job: its bytes
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
  landing (cancelled, failed) holds the closer, `ClosingStep`, its
  words naming the step and the change, which escalates to the goal:
  its release makes the step's change again, and its cancel ends the
  closer, the goal's branch kept.
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
- **Never a catch-up:** the closer makes none, and nor does a goal it
  ends (1).

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
