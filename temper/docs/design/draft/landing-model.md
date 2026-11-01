# The landing model

Draft, 2026-11-01, revision 5. Nothing here is adopted until the user
approves it. It describes the landing model that fab runs today for
fabrig: how a change goes from the person's draft to landed on main, and
what follows. fab is the dispatcher in the agents repository (`fab/`,
`bin/fab`); its designs (agents `docs/design/`, at 1dc102c) and its code
are the sources, and where they differ the code is followed and the
departure is said. The model is written in temper's words wherever a fab
term maps onto one (section 2); where none fits, the fab term stays and
is marked *(fab)*. `reference-case.md` is one story that runs every
mechanic here, and ends with what temper and jig have and lack for each.
Section 14 holds what fab does not do but may need later; it is not to
be carried out now. Other designs cite revision 4's sections and gap
ids; they are repointed when this one is adopted.

Unqualified, `landing.md`, `after-landing.md`, `findings.md`, `forge.md`
and `worker.md` are temper's `../domain/` documents; fab's designs are
named `fab:<name>` (`fab:integration.md` is the agents repository's
`docs/design/integration.md`); jig's are named as jig's.

## 1. In one page

- **One dispatcher, one record.** A single dispatcher decides every step
  from files in git (plans, briefs) and one append-only record of events
  that only it writes. Every status (a change, a red, a finding, a
  question) is folded from the record; nothing is kept by hand (section
  3).
- **The person starts a draft; a call cuts it.** A one-shot design call
  writes the plan (parts, slices, questions, every change's brief) and
  releases it in pieces, so producers start while it designs the rest
  (section 4).
- **Producers are one-shot runs in claimed workspaces.** Each change is
  one Codex run on a compiled brief, in a warm workspace on a builder;
  its report's first line is its outcome (section 5). As many run as the
  builders' places allow, from the start.
- **A draft lands as one trunk.** Each finished change is merged into
  the draft's integration trunk *(fab)*; dependents start from it, or
  early from a need's pushed branch. The trunk is checked along the way,
  caught up with main, checked in full at the end, repaired where red,
  and only then made ready (section 6).
- **The queue lands ready branches in batches.** It merges a batch onto
  main, proves it on one exclusive gate machine, reuses a change's own
  check for a batch of one, retries a failed test alone before blaming
  anyone, bisects a red batch, and fast-forwards main (section 7).
- **Tiers follow main.** The slow suites run on each newest main tip; a
  confirmed red is grouped with its suspects and sent to a foreman,
  whose fix is a repair change from main; a red no change owns, and a
  fault, go to the person's inbox (section 8).
- **QA walks feed a findings loop.** A walk of a rung reports findings;
  a triage call routes each; on-path ones are repaired; the rung passes
  when its brief, rerun unchanged, passes (section 9).
- **Judgement comes in calls, not residents.** A foreman settles what a
  change's options settle; the answerer takes the rest in batches; a
  call's output is reviewed by another session, amended once, then
  settled; only what is the person's reaches the person (sections 10 and
  11).
- **Custody and capacity are the dispatcher's:** workspaces are claimed
  and swept, builds wait for disk, sign-ins are lent from one shelf
  (section 12).
- **Measured, not felt:** reports and a public status page are computed
  from the record (section 13).

## 2. Words

| fab | Here | temper's or jig's term |
|---|---|---|
| the user | the person | the person (jig's `domain/people.md`) |
| the director, a Claude Code session | the person's chat | the chat |
| `ops`, a session for the machines | the operator | the operator (no design yet) |
| `fab tick`, the dispatcher | the dispatcher | the engine's procedures; no single term |
| `FAB_HOME/log.jsonl` | the record | temper's store |
| draft (`docs/design/draft/`) | draft | draft |
| plan (`plan.toml`), part, slice | plan, part, slice *(fab)* | a coordinator's design and its parts |
| task and its spec | change and its brief | a change (`forge.md`, 8), a goal's brief |
| a Codex run | run | a producer, a delegate agent under a charter |
| a Claude call | call *(fab)* | a one-shot agent delegate on Claude |
| slot (fabrig), worktree (repositories that build nothing) | workspace | workspace (`worker.md`) |
| builder | builder | a worker's machine |
| `<draft>/integration` | trunk *(fab)* | none |
| `ready/<name>` | ready, a landing | a ready change, a landing of one |
| merge queue (`fab queue`) | queue | the landing turn (`landing.md`, 8) |
| batch | batch | none (a candidate is one landing's members) |
| precheck | the check before ready | none |
| precheck receipt | receipt, by tip and main | receipt, by key (`landing.md`, 6) |
| the gate machine, `heavy --gate` | the prover, its exclusive lane | a runner |
| after-landing tiers | tiers | tiers (`after-landing.md`) |
| foreman, answerer, settle, amender | foreman, answerer, settle, amender *(fab)* | none; nearest: escalations and proposals to a holder (jig's `domain/tasks.md`, 9) |
| parked | parked | a run parked on its own question (jig's `domain/tasks.md`, 5.7) |
| `FAB_HOME/user.md`, an ask, a signal | the inbox page, a question, a signal *(fab)* | the person's inbox |
| issue on the forge per ask | the question's issue | questions projected as issues |
| login shelf | the shelf | custody of a sign-in |
| ladder, rung, walk, finding, triage, backlog | the same | temper's `../qa.md`, `findings.md` |

## 3. The dispatcher and its record

- **One writer.** The dispatcher on hel8 appends every event to the
  record under a lock; a torn last line is ignored. Builders, the queue
  and the tier runner keep records of their own, which the dispatcher
  imports by asking (`fab:fab.md`, 3).
- **State is folded, never stored:** a change's status (waiting,
  runnable, running, parked, done, ready, landed, rejected), a part's, a
  red's, a finding's, a rung's and every open question are pure
  functions of the plans and the record. New kinds of event are
  additive; a reader ignores what it does not know.
- **A tick decides everything,** under one lock, in this order (fab
  `dispatch.py`): it fetches each listed plan's checkout; imports the
  queue's and the tiers' records; closes and opens reds; measures the
  builders; then, plan by plan, merges a running cut's pushes, records
  triage and passes, collects ended runs, merges, lands or readies done
  changes, parks, starts runs, settles foremen, starts calls, and takes
  one step of the trunk's checks; then it probes the trunks' overlaps,
  collects runs no plan owns, and rewrites the inbox page. Several plans
  share one tick and one set of builders.
- **Events wake it** (`fab:tick.md`, 6): a run's end, a call's end, a
  cut's push, a queue cycle that landed or rejected, a tier run with a
  new red or a recovery, a landing, the person's answer, a walk's
  result. A wake is one line in a want file; a path unit starts one tick
  for a burst of them. A timer every 10 minutes is the backstop, and
  notes any lost event it finds. A tick whose own records need another
  round wakes itself, at most five times in a row.

## 4. Drafts, cuts, plans and changes

### 4.1 A draft starts by the person's word

- **The person starts it,** or their chat at their word, with `fab call
  cut` naming the draft and its other sources; the call lists the plan.
  A draft landing on main starts nothing by itself (`fab:tick.md` 7
  names a `fab draft start`, which the code does not have).
- **The plan lives where it lands.** A fabrig plan is a directory in
  fabrig (`<tree>/docs/plan/<draft>/`), listed in the dispatcher's plans
  file. An integration plan's directory lives on its trunk, and the
  dispatcher reads it at the trunk's tip.

### 4.2 The cut

- **One call, about 12-20 minutes, writes the plan:** its parts (with
  `direct` on, each direct: one or two changes, no skeleton), their
  needs, the slices (stories that run end to end, and whether QA walks
  them), the person's questions, and every change's brief, with notes
  several briefs share (`notes/`) and the shapes that cross parts
  written first (`notes/contracts.md`).
- **It releases early** (an integration plan with `direct` on and
  reviews off): it pushes first the changes that need no other, then
  each batch that holds together, waking the dispatcher each time; the
  dispatcher merges each push into the trunk, and those changes start
  while the cut designs the rest. The cut never rewrites a brief it has
  released. With reviews on, the plan waits for its review chain (10.2).
- **It sees the drafts beside it:** its brief lists the other plans'
  changes not yet on main, with the paths each owns, and asks it to keep
  early work off them (6.6).
- **It may split the design** across designer subagents, from shared
  contracts it fixes first; it is still one call, with a 60-minute
  timeout.
- **Questions for the person** are `[[question]]`s naming the parts that
  wait for their answer. A direct part's changes do not wait (14).

### 4.3 A change and its brief

- **A brief** is Markdown with a header: id, part, kind (feature,
  interface, migration, repair, rebase, docs), needs (changes that merge
  first), files (the paths it owns, for the cut and the producer; never
  enforced), effort and the expected minutes. Its body says what to do,
  how it is accepted, and the pre-approved options with their defaults.
- **Some changes have no brief file:** repairs of the trunk
  (`repair-<n>`, `catch-up-<n>`), of a red after landing (`red-<n>`) and
  of a QA finding (`finding-<n>`) are folded from the record; their body
  is the evidence, the foreman's section and their answers.

### 4.4 The plan's options

| Option | Default | What it sets |
|---|---|---|
| `landing` | `integration` | the trunk (section 6), or `queue`: each change readied (or, in the agents repository, landed) on its own |
| `ready` | true | the checked trunk readied for the queue; false leaves it unlanded (a replay) |
| `reviews` | false | every call's branch reviewed (10.2) |
| `direct` | true | every part direct; false allows skeleton and contract parts (10.6) |
| `early` | true | early starts (6.2) |

The agents repository's own plans name `landing = "queue"`. fabrig's QA
ladder is a queue plan with rungs instead of parts (section 9).

## 5. Runs

### 5.1 A run

- **One Codex execution** for one change, in a workspace claimed for it:
  a slot on a builder (fabrig) or a worktree (a repository that builds
  nothing heavy). Its base is the trunk's tip, or the change's own
  pushed branch when it ran before (a rebase run starts afresh from the
  trunk).
- **Its brief is compiled, not gathered:** the role text, the change's
  brief and facts (its branch, its trunk); the producer reads what it
  names in its checkout.
- **It commits and pushes before it stops,** and its report's first line
  is the outcome:
  - `Done:` its branch pushed and its own checks passed; it may carry a
    `## Left` list of what lies outside the change and stays red, for
    its owner or the end check;
  - `Question:` with the options and a recommendation (section 10);
  - `Infra:` a disk, a machine or a tool: started again without asking
    anyone;
  - `Failed:` parked for the answerer.
- **The producer may change, without asking,** the arms, fields and
  literals the compiler requires because of a type the change altered,
  anywhere in its tree, and add a scoped lint exception on an item it
  wrote.
- **A run limit:** a change with four runs (rebases aside), or a repair
  of a red or a finding with two, that has not landed parks for the
  answerer, whose answer grants fresh runs.

### 5.2 Places and the flow

- **Places are a hint, not a cap:** each builder names its places (14 in
  all today); an integration plan keeps its target of runs in flight at
  their sum throughout. A queue plan starts there too, grows by one with
  each first-attempt landing made with the window full, and halves with
  each `Infra:` outcome or rejection (but at a merge or rebase), never
  below one.
- **A calm builder takes one more:** a builder with low load and no
  recent disk or lane waits, measured each tick, takes one run beyond
  its places, never beyond its slots.
- **The order of starts:** consolidations, then repairs of reds and
  findings and the runs that follow a refusal, then contract parts'
  changes, then the parts' changes by slice, part and brief order; the
  trunk's own repairs and catch-ups after them; early starts last.
- **A full builder hands a start on** to the next builder with free
  places, since slots claimed by other work are not the dispatcher's.
- **Builders follow the dispatcher:** before each start, a builder's
  install moves forward to the dispatcher's commit, unless pinned.

## 6. The trunk

### 6.1 Merging

- **A done change is merged, not landed:** the dispatcher resets its
  checkout of the trunk to origin, merges the change's branch with a
  merge commit, and pushes the trunk fast-forward only. No gate runs
  here; what to check and when is the producers' own checks and the
  trunk's checks below.
- **Mechanical conflicts settle themselves:** a conflict confined to a
  counted baseline (`scripts/old-forms.txt`) is settled by taking the
  trunk's side and recounting with the project's tool, which refuses a
  raise. Any other conflict aborts the merge, is recorded as a rejection
  at step `merge`, and a rebase run follows (10.3).
- **A merged change counts as landed for its dependents:** they become
  runnable and start from the trunk's tip. Calls' branches, answers and
  repair sections are merged the same way.

### 6.2 Early starts

- **A waiting change may start early** when each need not yet merged is
  running or done with its branch pushed, and has no unmerged needs of
  its own (one level). Its workspace starts from the trunk with those
  branches merged in, in needs order; a conflict refuses the early start
  and it waits.
- **Its brief says so,** naming the branches to merge again before it
  pushes. It merges into the trunk only after every need has.
- **Once at most:** after a question, or a `Failed:` run whose needs'
  work was unfinished, it waits for its needs to merge, then runs again.

### 6.3 Checks along the way

- **While changes remain,** after 10 merges since the last check, or 45
  minutes with at least one, the dispatcher checks the trunk's tip with
  the end check's steps less fuzzy, in an idle workspace, when no repair
  is open.
- **A red becomes a repair** (`repair-<n>`, kind repair, owning every
  path): its body is the failing steps and the report, and lists the
  changes not merged yet with their paths, to leave what they own. It
  runs from the trunk's tip and is merged like any change.

### 6.4 Catching up with main

- **Main moves under a trunk** each time another draft lands. While
  changes remain, when the trunk lacks main's tip and 30 minutes have
  passed since its last catch-up, the dispatcher merges main into the
  trunk (a counted baseline recounted as in 6.1) and pushes it. It waits
  while a repair is open or the cut is in flight. Once every change has
  merged, it catches up only before a check, so a ready trunk does not
  move under the queue.
- **A conflicting catch-up is repaired:** `catch-up-<n>` merges that
  main into a branch from the trunk's tip, keeping both sides' intent,
  fixes what main's changes break, and is merged like any change; the
  dispatcher then checks the trunk contains that main.
- **Stops for judgement:** a catch-up refused at a main whose repair has
  merged, or a third catch-up refusal since the last clean one, stops
  the plan (6.5); a hand merge of main into the trunk, seen by the next
  tick, records a clean catch-up and clears the stop.

### 6.5 The end check, and ready

- **When every change has merged,** the trunk is caught up with main and
  checked in full in one run (`fab check`): every counted baseline,
  formatting, the workspace's lints, the build, the default and fuzzy
  tests, as the queue's full checks. It takes a place.
- **Red makes a repair** as in 6.3, and the trunk is checked again; a
  third red stops the plan.
- **Green makes it ready:** `ready/<draft>/integration`, one landing for
  the whole draft. A queue rejection catches it up, checks it again and
  readies it again; a third rejection stops the plan.
- **A stop** is a note in the record: the trunk takes no further step,
  and only repairs of reds and findings still start.
  `fab:integration.md` 3 says a third red parks the plan with a
  question; the code asks none, and nothing reaches the inbox (14).

### 6.6 Drafts beside each other

- **An overlap probe, report only:** each tick, for every pair of ready
  plans' trunks not yet on main, the dispatcher merges each trunk with
  main in memory, then the pair, and records the paths that conflict
  when they change (`trunks.overlap`). Results are cached by main's and
  the tips' commits. Nothing waits on an overlap: the second trunk to
  land still meets the conflict at its catch-up and repairs it there.
- **It is shown:** the status page lists each overlapping pair with a
  count, and the cut's brief lists the drafts beside it (4.2).

## 7. Landing

### 7.1 The queue

- **Ready refs, oldest first:** a branch is ready when `ready/<name>` is
  pushed at its tip, by the dispatcher or by hand. Each cycle the queue
  (on the gate machine) takes a batch of up to eight untried tips,
  oldest commit first, merges them onto main in order, and runs the
  selected checks on the result. A member that conflicts is rejected at
  `merge`.
- **Selection** is the packages the batch reaches and their reverse
  dependencies, with lists of paths that always run and paths tests
  read; a documentation batch runs only a touched tree's counted
  baseline. Every fifth batch runs the full selection.
- **The steps** run in the gate lane, which nothing else builds in:
  counted baselines (recounted, refusing a raise), formatting, lints,
  the build, the default and fuzzy tests. CPU is measured per tree and
  reported; starved tests stay green; budgets, when the person sets
  them, are reported, not enforced.
- **Green:** main is pushed, expecting its parent; each `ready/<name>`
  is deleted under a lease on its tip, and its source branch when it
  still names that tip.
- **An error is the gate's fault:** short of disk, a dirty checkout, a
  refused push or a ready ref left at the landed tip; nothing is
  rejected or marked tried, and the batch goes again.
- **The queue keeps its record and little else:** batches, landings,
  rejections (with the failing tests at a test step) and collateral
  reds, which the dispatcher imports; it re-derives the rest from git. A
  stop drains the cycle in progress.

### 7.2 A change's own check, reused

- **Before ready, in its workspace,** a change readied on its own (a
  queue plan's, or a repair after landing) runs the check before ready
  inside its run: its tip merged onto main on a detached checkout, the
  gate's selection in the build lane, the checkout reset after (`fab
  precheck`). Its receipt is kept on the builder. The dispatcher then
  runs the counted-baseline gate on it (a raise is refused, as the
  change's) and readies it.
- **The receipt travels with the ready:** the dispatcher hands it to the
  queue. A batch of one, not due for a full run, whose receipt is green
  for the same tip and the same main lands on it after the counted
  baselines and formatting alone.

### 7.3 Reds classed before anyone hears them

- **A failed test retried alone:** at a test step with at most five
  failed tests, each of which passed in every member's receipt, each is
  run again alone. All passing: the batch is `collateral`, nobody is
  rejected or marked tried, and it goes again. One failing alone: the
  red stands.
- **Bounded:** at most five failed tests are retried; a tip classed
  collateral twice has its next red stand.

### 7.4 Bisection

- **A red batch is halved,** the left half first, each half merged and
  checked again; a green half lands before the right half is checked,
  until each red is one branch's; that branch is rejected. After a red
  full batch, the halves in that cycle are checked in full.

### 7.5 Who hears a rejection

- **A change of a plan:** the record gains `branch.rejected` with the
  step, its log and failing tests, and the change's refusal foreman
  decides (10.3); a retry re-readies the same tip and asks the queue to
  forget it was tried (`fab queue untry`).
- **A trunk:** caught up, checked and readied again (6.5).
- **A branch pushed by hand,** which no plan owns: a signal in the
  person's inbox (11.2), cleared when the branch lands or a newer tip is
  tried.

### 7.6 The agents repository

fab's own repository lands one branch at a time through `fab land`: the
dispatcher first runs fab's code gates on the branch (no changed
signature unless the change is an interface or a migration, no function
over 60 lines added or grown, no banned name, docs on public items, no
repeated block), then a rebase onto main, the whole suite (within 120
seconds), and a fast-forward push. A conflict or a red is a rejection as
in 7.5. It refuses the user's drafts and the legacy kit.

## 8. After landing

### 8.1 Tiers follow main

- **One runner, one slot, its own builder.** It fetches main each cycle;
  a tip that is not its cursor (the last tip run without a fault) gets
  one run of the tier set: the browser suites of jig, temper and skein
  and temper's process stories, then the live suites whose sign-ins the
  shelf holds (only skein's today), borrowed for the run. Tips published
  meanwhile are covered by the next run.
- **Suspects** are the first-parent commits since the cursor; a queue
  merge names its branch.
- **A red is confirmed:** each failed test runs again alone. Failing
  again, it is red; passing, a flake. Against its own history a red is
  **new** (it last passed), **standing** (last red) or **unattributed**
  (never ran); a pass after a red is **recovered**.
- **Faults and starvation:** a pair that cannot run (a missing sign-in,
  disk, a missing report) faults and says nothing about its tests;
  wall-clock expiry is **starved**, counted apart, never red; CPU is
  measured per test and pair. A run is red, partial (a fault) or green;
  the cursor moves only past a run without a fault.
- **Budgets** are the person's to set; with none, CPU and wall are
  reported only.

### 8.2 A red reaches its owner

- **Grouped once:** the dispatcher opens one red per tip and suspect set
  for the new and unattributed reds still red in the newest run
  (`red-<n>`), writes its evidence (the tests, the suspects, the rerun
  logs' ends), and routes it.
- **A red with a plan's change among its suspects goes to a foreman**
  (10.3), which reads the evidence and the suspects' briefs at the red's
  tip and decides: **fix** (what failed, where, what to change, and the
  owner among the suspects), or **not the suspects'** (a flake that
  failed its rerun too, the tier's environment, a hand-pushed change),
  which sends the red to the inbox.
- **The repair is a change of its own,** `red-<n>`, from main, in the
  newest suspect's plan: reproduce in the lowest tier that shows it,
  then fix; readied to the queue like any change. It has two runs, then
  the answerer.
- **Closed by the same checks:** a red closes when every test recovers,
  or is absent from a later run with no fault, starvation or unobserved
  test. The first run after the repair landed that still lists the test
  makes the red **unfixed**, a signal.
- **A red with no plan's change among its suspects,** and every fault,
  is a signal in the inbox; a fault clears only when a run no longer
  faults the pair.
- **No bisection after landing:** the foreman reads the suspects' diffs
  and names the owner.

### 8.3 A plan retires after its verdict

When every part is done, a fabrig plan waits until its last branch (the
trunk) has landed on main and a tier run covering it has come back with
no red of the plan's still open; then the dispatcher starts the retire
call (10.6).

## 9. QA walks and the findings loop

### 9.1 The ladder

- **A ladder is a queue plan with rungs:** each rung is a story on a
  backend (ChatGPT with an owned Forgejo or GitHub, or Claude Code with
  an owned Forgejo), its brief files (the kit, then the rung's own), and
  the rungs that must pass first (`after`). Its `[ladder]` names the
  happy path (a design citation) and the backlog file.
- **A rung's edition is its brief's bytes:** a digest of its brief files
  at main. "Rerun unchanged" is a walk at the same edition; a changed
  brief or kit starts a new edition, whose walks start afresh.

### 9.2 A walk

- **A session walks by hand, and reports through commands:** `fab qa
  walk` records the walk's start at a commit (refusing a commit that
  lacks a landed repair of the rung's findings) and writes its brief:
  the rung's files, how to report, and the open findings. The walker
  exercises the story on a real box, holding its sign-ins.
- **The result** is a verdict (pass, fail, void), a reason, and
  findings: title, severity, area, whether it blocked, reproduction,
  expected, seen, evidence, and `same`, an open finding it met again.
  `fab qa result` records each finding, then the walk's end, and wakes
  the dispatcher.

### 9.3 Findings and triage

- **A finding is a record:** new, then routed, backlog, dropped, asked
  (the person's), merged, or in a repair. The same finding is merged by
  the walker's `same` or by triage, never filed twice; a sighting of one
  whose repair has landed is filed again, `again`, for triage.
- **Triage is a call in the chat's role** (report only, medium), started
  at once for a blocking finding or one from a failed or void walk, else
  at 4 new findings or after 30 minutes. One line per finding: **path**
  (on the happy path; its owner; it may add an action the person must
  take before the rerun), **backlog**, **user** (a question in the
  inbox), **brief** (the rung's brief at fault, a question to the
  ladder's owner), **merge**, or **drop**.
- **The backlog** is a file in the tree, rendered from the record and
  readied to the queue whenever it differs from main; two rejections of
  one render raise a signal.

### 9.4 A repair, and the pass

- **An on-path finding goes to a foreman,** then a repair change
  `finding-<n>` from main, as a red's (8.2): reproduce first in the
  lowest world that shows it (a test red before and green after), then
  fix, design text changed in the same branch.
- **A rung's state:** passed, walking, waiting (a lower rung has not
  passed), holding (a blocking finding, or one from a failed walk, is
  open), or due (a first walk, or a rerun at a main holding its
  repairs). The inbox page shows each rung's line.
- **A pass is recorded** when a walk at the rung's edition passes after
  an earlier walk of that edition that passed or failed, with nothing
  holding the rung and its lower rungs passed. A finding whose repair
  landed is then fixed; one filed again is unfixed. A failed walk after
  a pass reopens the rung.

## 10. Calls and judgement

### 10.1 A call

- **A fresh Claude session on a compiled brief,** run by the dispatcher
  as a unit on hel8, ending by writing files: a branch it pushes, or a
  report only. No call waits, polls or watches a run.
- **Measured, not capped:** its peak context, turns and tokens are
  recorded; the person's target is about 600k of context; it is stopped
  only by its timeout (60 minutes).
- **The roles:** by the dispatcher, answer, amend, settle, retire,
  triage, skeleton and contract (when `direct` is off) and the structure
  review (when reviews are on); by hand, the cut and the decision; by
  the forge pass, the issue reply. One call runs per subject at a time.

### 10.2 Review chains

- **With reviews on,** a call's branch gets a read-only Codex review run
  that reports at most ten lines: approve, or refuse with findings.
  Refused, the findings go back once to an amend call on the same
  branch; refused again (or the amend failed, or three reviews ended
  `Infra:`), the chain is held for a **settle** call, which says land,
  amend (with guidance, once more) or user (only for what is the
  person's: budgets, machines, what is hard to reverse, a change of
  direction).
- **A landing's refusal is a review's refusal:** any rejection of a
  call's branch counts in the chain (a conflict merging it into the
  trunk, the queue's rejection, `fab land`'s): amended (rebased) when
  the chain has had no amend since its last reset, else held for a
  settle. A running cut's merge conflict is only a note.
- **With reviews off** (fabrig's default), a call's branch is merged
  into the trunk as it ends.
- **With reviews on, a structure review** call reads the digests of the
  changes landed since the last one (every ten landings or four hours)
  and writes consolidation changes, which go first among the runnable
  changes, and pairs for the taste file the producers read.

### 10.3 The foreman

A read-only Codex run (medium) that decides one thing and writes no
code; the dispatcher records what it decides.

- **A question** from a run of a change with a brief file (a repair has
  no options to settle by: its questions queue at once, and an answer is
  folded into its body, which its next run reads): when the change's
  pre-approved options answer it, the answer goes into the brief's
  Answers and the change runs again; when the run's work is pushed and
  accepted and what it asks lies outside the change, `Accept:` folds the
  change done (integration plans); otherwise the question queues for the
  answerer.
- **A refusal:** fix (a `## Repair` or `## Rebase` section for the next
  run) or retry (not the change's: the tip goes back unchanged, once).
- **A red after landing and an on-path finding:** fix (a repair change)
  or not the code's (the inbox).
- **One per question, refusal, red or finding;** any other outcome is a
  fallback: the question queues, or the repair starts with the evidence
  alone.

### 10.4 The answerer

- **One call answers the queued questions** of a plan in one pass: at
  once when a question holds up other work (or always, in an integration
  plan); otherwise at 4 queued or after 30 minutes.
- **It prefers leaving what is not the change's** to its owner or the
  end check, widening a change only when its own acceptance cannot hold
  otherwise.
- **What it cannot settle** it ends with a `Question:` of its own, which
  goes to the person (11.1).

### 10.5 Parks

- **Only the change that needs an answer parks;** its dependents wait,
  everything else runs. A park lifts when its question is answered.
- **A call's question parks its subject** until the person answers.

### 10.6 The designer's other calls

- **Retire:** once a plan's parts are done (and, for fabrig, after 8.3),
  one call brings the changed designs' status files current and marks
  the plan retired; its brief quotes each changed design as a diff,
  within a bound. Its branch is delivered as the plan's other calls'
  are: for an integration plan, merged into the trunk, which has already
  landed by then, so its changes do not reach main (a fab fault, untried
  live).
- **Skeleton and contract calls** design a part as code (types,
  signatures, empty bodies, its briefs) when `direct` is off, started
  when the ready changes fall below the plan's band. The dispatcher
  checks the call's tip (gates and tests) and starts the part's changes
  from it before it merges; a part waits for its needs to be designed,
  not done. fabrig's plans keep `direct` on, so they make none.
- **Changes started from a skeleton wait for its chain:** a done change
  merges only once the skeleton has; if the skeleton landed amended,
  each such change gets a rebase run onto it, outside its run limit; a
  held or parked chain parks its changes. In an integration plan fab
  does not see the rebase as done (it looks for a base of main, the
  run's is the trunk), so it repeats it: a fault, untried live.
- **A decision call,** by hand, turns a probe part's report into a
  design section; the probe run itself is not built (14).

## 11. The person's channel

### 11.1 Questions and the inbox page

- **Only what is the person's reaches the person:** budgets, sign-ins,
  machines, what is hard to reverse, a change of direction. Whoever asks
  decides it is the person's: a plan's question, a call's `Question:`, a
  chain settle referred, a triage `user` or `brief` line.
- **One page** lists the open questions with what to read and how to
  answer, the open signals, the reports' signals that are the person's,
  the newest tier verdict, and the ladder's rungs; the dispatcher
  rewrites it each tick. `fab page answer SUBJECT TEXT` records the
  answer; every later call of the subject carries it. A plan's question
  is answered by writing its answer into the plan and landing (or
  merging) that.

### 11.2 Signals

Signals are what has no owner the dispatcher knows; the person's chat
reads them on the same page and acts: a hand-pushed rejection
(`queue:<branch>`), a red no change owns or that stayed unfixed
(`red-<n>`), a tier pair that cannot run (`fault:<profile>`), a finding
its foreman called not the code's (`finding-<n>`), a backlog branch
rejected twice (`backlog:<draft>`). Each opens and clears from the
record, or by `fab page clear` with a reason (but a fault).

### 11.3 On the forge

- **One issue per open question** in the record (calls', parked chains'
  and triage's; never a plan's), labelled `waiting-on-user` on the
  agents repository's Forgejo, opened, conversed and closed by `fab
  issues sync`, which a timer installed by hand runs every 2 minutes.
- **The person answers in a comment:** a reply call reads it with the
  question and its report and either replies (context, options, a
  recommendation) or records it as the answer, then the issue closes.

## 12. Machines, workspaces and sign-ins

### 12.1 Builders and workspaces

- **hel8** runs the dispatcher, the calls, the inbox page, the shelf and
  the status page; it builds nothing. **Builders** run Codex and builds
  in slots; **hetzner8** is also the gate machine and the tier runner's
  builder.
- **A workspace is claimed** for a run (a claim file on the builder),
  released at its end with a sweep that removes incremental data and
  stale test executables, which bounds a slot near 25 GB. A claim whose
  run is gone is stale and released.
- **Builds go through `heavy`:** a build lane of the builder's `builds`
  locks and one gate lane; before a build it needs the disk headroom
  free, reclaims idle slots and waits up to 20 minutes, then exits 75,
  which the run reports as `Infra: disk`.
- **A builder's disk is budgeted:** the cache's cap plus its slots at
  the slot bound plus headroom, and one slot bound more on the gate
  machine for the queue. Live overrides of the machines' file are by
  hand.

### 12.2 The shelf

- **Every sign-in lives on one shelf on hel8,** one entry per login:
  Codex and Claude machine sign-ins, and QA's ChatGPT records, Claude
  Code directories and GitHub fixtures.
- **Custody is the record:** added, lent (to whom, where), returned,
  checked, retired. A lent login's shelf copy is a placeholder until it
  is copied back; tokens refresh, so the copy last used is the true one;
  bytes are moved and hashed, never printed.
- **Lent to machines and to runs:** a builder holds one Claude and one
  Codex login; a live tier pair borrows its logins for its run and
  returns them; a swap gives a machine another account.
- **Checked by the products themselves:** a check reads the account and
  every usage limit (the weekly percent is the signal for a swap), every
  30 minutes.

## 13. What is measured and reported

- **Two daily reports, from records alone,** written by `fab report
  daily` run by hand: the code report per plan (runs and outcomes, runs
  in flight against places, idle gaps, minutes per change, questions and
  time to answer, Infra minutes, disk and lane waits, each part's time
  from start to landed, Claude and Codex usage and their weights, stops
  ranked by time lost) and the landing report (batches and outcomes,
  rejections by step, collateral reds, batches landed on a receipt,
  ready-to-landed latency, bisection runs, each step's durations, CPU
  per tree; reds opened, routed, settled and closed; QA walks, findings,
  triage decisions, repairs and passes). Each figure stands beside the
  day before's; the signals that need no target are raised with their
  owner.
- **Weights:** input 1, cache write 1.25, cache read 0.1, output 5 for
  Claude and 8 for Codex; the Claude to Codex weight per draft, with the
  person's chat's hand sessions shown apart.
- **The status page** (status.fabrig.ai) is public and shows counts
  only: what is in flight, landed and merged today, questions waiting
  and their age, each draft's activity, overlaps, and the subscriptions'
  use and pace, from the record, never a branch, path or text.

## 14. Future ideas

**Not to be carried out now.** These are in the previous edition or in
fab's designs, fab does not do them today, and they may be needed later.
Each waits for the numbers or the person's word that asks for it.

- **From the person's execution model, not built or off:** skeleton
  parts with a test vocabulary as the default; surveys before the cut
  and each skeleton; test-writer and accept runs; a self-critique before
  ready; probe runs; a slice complete when its story test passes on
  main; the ready-change buffer pulling design.
- **The queue:** receipts by key per package (the previous edition's
  keys, gate classes and audits by full proofs); receipts reused for
  batches of more than one; speculative batches (a train) or a second
  gate when utilisation says so; a batch size from measures; budgets
  that turn red; evidence bound to trees and certificates from runners.
- **After landing:** bisection to a culprit when foremen name the wrong
  owner; a second repair round for an unfixed red; the session that
  pushed a hand-pushed branch told of its rejection or red; QA passes on
  landings marked for QA.
- **The trunk:** a stop raised as a question in the inbox, as
  `fab:integration.md` 3 has it; a build of overlapping trunks' merge;
  landing a trunk at slice boundaries; overlaps that reorder work; a
  lighter lint of the trunk after each merge.
- **QA:** walks run by the dispatcher (a walk call, a box on a QA
  builder, sign-ins borrowed per backend, a 3-hour cap); a passed rung's
  story joining the live tier, its fake twin in the worlds; findings
  matched by a normalised key; findings of other finders (flakes,
  reviews' follow-ups); findings and questions on the forge as temper's
  own records.
- **Reports:** the daily reports on a timer, sent to the person's chat.
- **Calls:** a cap on a call's context or turns once the person sets
  one; a plan question that holds a direct part's changes; the cut
  started when a draft lands; needs across plans.
- **Machines:** elastic builders from the disk budget's numbers; a new
  builder lent its sign-ins when it joins.
- **Landings across repositories,** and a forge's native merge queue
  (GitHub's) as an accelerator.


