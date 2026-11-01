# The landing model

Draft, 2026-11-01, revision 5. Nothing here is adopted until the user
approves it. It describes, as generic mechanisms of temper and jig, how
work goes from the person's word to landed on main and what follows,
with no hand work anywhere. Each mechanism has settings; fabrig's way of
running is one configuration of them (section 14), the one fab runs
today. temper must offer these mechanisms to replace fab; it may offer
fabrig's configuration as a ready-made default. The sources are fab's
designs and code (the agents repository, `docs/design/` and `fab/`, at
1dc102c); where they differ the code is followed and the departure is
said. `reference-case.md` is one story that runs every mechanism, and
ends with what temper and jig have and lack for each. Section 15 holds
what fab does not do but may need later; it is not to be carried out
now. Other designs cite revision 4's sections and gap ids; they are
repointed when this one is adopted.

Words are jig's, then temper's; section 2 names the few that are new.
Unqualified, `landing.md`, `after-landing.md`, `findings.md`, `forge.md`
and `worker.md` are temper's `../domain/` documents; fab's designs are
named `fab:<name>`; jig's are named as jig's.

## 1. In one page

- **An LLM for judgement, code for mechanisms.** Every mechanism here is
  code: scheduling, merging, queueing, checks, bisecting, retries,
  keeping, lending, reports. Every judgement is an agent task: planning,
  producing, answering, examining, reviewing, arbitrating, triaging,
  walking. Nothing mixes them; each heading says which it is. Where fab
  today uses an agent for a mechanism, or code for a judgement, section
  14.2 calls it a fab gap.
- **The person talks to a chat; everything else is automatic.** The
  chat, made from a template of the person's choosing, has tools that
  create tasks. Given a draft, or "fix this", it creates a goal; the
  goal's agent decides the sub-tasks; mechanisms take each change to
  landed; only questions reach the person, in their inbox (sections 4,
  11).
- **One engine, one store.** The engine decides every mechanical step
  from specs in git and records in its store, which it alone writes;
  every status is derived (section 3).
- **A planner plans a goal** and releases it in pieces, so producers
  start while it plans the rest (section 4).
- **Producers are runs in claimed workspaces,** as many as the workers'
  slots allow; each run's result is finished, a question, a transient
  failure or failed (section 5).
- **A goal may land through its own branch:** each finished change
  merges into it; dependents start from it, or early from a dependency's
  pushed branch; it is checked along the way, updated from main, checked
  in full, repaired where red, and only then made ready (section 6). Or
  each change lands on its own.
- **The landing queue lands ready branches in candidates,** reusing a
  change's own check, rerunning a failed test alone before blaming
  anyone, and bisecting a red candidate (section 7).
- **Tiers follow main;** a confirmed red goes to an examiner and a
  repair from main (section 8). **QA walks** feed a findings loop
  (section 9).
- **Judgement is in agent tasks, never residents** (section 10); custody
  lends accounts and workspaces (section 12); reports come from the
  store (section 13).

## 2. Words

Most words are jig's (task, goal, chat, template, plan, spec, brief,
run, try, kept, wake, timer, inbox, worker, slot, workspace, account,
lend, recurring) or temper's (change, producer, landing queue,
candidate, gate, receipt, tier, red, flake, fault, repair, resolution,
update, finding, triage, walk, ladder, rung). These are new, for what
neither names:

| Word | What it is |
|---|---|
| planner | the agent task that turns a draft into a goal's plan |
| part, slice | a unit of the plan; a story that runs end to end |
| goal branch | the branch a goal's changes merge into before it lands whole (temper's "a branch per goal", `forge.md` 11.1) |
| early start | a change started on a dependency's pushed, unmerged branch |
| interim check, final check | the goal branch checked whole along the way, and before ready |
| examiner | a read-only agent task that decides one thing: a question, a rejection, a red, a finding |
| answerer | the agent task that answers a goal's queued questions in one pass |
| reviser, arbiter | the agent task that revises a reviewed output once; the one that decides after a second refusal |
| alert | an inbox item for a chat: something no goal owns |
| edition | a rung's brief, identified by its bytes |
| scaffold | a planning task that writes a part as code before its changes fill it |
| closing task | the agent task that ends a goal: status files current |
| custody | the accounts held centrally and lent to workers and runs |

## 3. The engine and its store (code)

- **One writer.** The engine appends every record to its store; workers,
  the landing queue and the tier runner keep records of their own, which
  the engine reads by asking.
- **Status is derived, never stored:** a change's (waiting, runnable,
  running, kept, finished, ready, landed, rejected), a part's, a red's,
  a finding's, a rung's and every open question are pure functions of
  the specs and the records. A reader ignores kinds it does not know.
- **One round decides everything mechanical,** in a fixed order: read
  each goal's plan and the queue's and the tiers' records; close and
  open reds; measure the workers; then, goal by goal, merge the
  planner's pieces, record triage and passes, take ended runs, merge,
  land or ready finished changes, keep, start runs and agent tasks, and
  take one step of the goal branch; then probe overlaps and rewrite the
  inboxes. Several goals share one round and one set of workers.
- **Events wake it:** a run's or an agent task's end, a planner's piece,
  a candidate landed or rejected, a tier run with a new red or a
  recovery, an answer, a walk's result. A burst of wakes makes one
  round. A backstop timer (`backstop`) finds lost wakes and notes them;
  a round whose own records need another wakes itself, up to a bound
  (`self_wakes`).

## 4. Intake, goals, plans and changes

### 4.1 The chat and a goal (LLM)

- **The person talks to a chat,** a free-form, possibly long-running
  agent task made from a template of their choosing: a director, or a
  department such as QA or the web UI. The template sets its charter,
  its tools and which alerts reach it (11.2).
- **The chat's tools create tasks.** Given a draft, or a request such as
  "fix this", its agent creates a goal with the settings its template
  allows (4.4), naming the draft or the request as the goal's spec.
- **The goal's agent decides its sub-tasks:** for a draft, a planner
  (4.2); for a small fix, one change; later, whatever its records ask
  for: a fresh resolution after a stop (6.4), a goal for an alert. From
  there every step to landed is a mechanism or another agent task's
  judgement; no one works by hand.
- **The plan lives with the goal:** a directory in the repository; a
  goal with a goal branch keeps its plan on that branch.

### 4.2 The planner (LLM; its release is code)

- **One agent task writes the plan:** its parts, their dependencies, the
  slices (and whether QA walks each), the person's questions, and every
  change's spec, with notes several specs share and the shapes that
  cross parts written first. With `scaffold` off, every part is direct:
  one or two changes, no scaffold.
- **It hands the plan over in pieces** (a goal branch, `scaffold` and
  `review` off): it commits first the changes that depend on nothing,
  then each batch that holds together; the engine pushes each piece,
  merges it into the goal branch and starts its changes. It never
  rewrites a spec it has handed over. With `review` on, the plan waits
  for its review (10.2).
- **It sees the goals beside it:** its brief lists the other goals'
  unlanded changes with the paths each owns, and asks it to keep early
  work off them (6.6).
- **It may split the design** across agents of its own, from shared
  shapes it fixes first; it is still one task, under one timeout.
- **The person's questions** name the parts that wait for their answer.

### 4.3 A change and its spec

- **A spec** is words with typed parameters: its part, its kind
  (feature, interface, migration, repair, rebase, docs), its
  dependencies, the paths it owns (for the planner and the producer;
  never enforced), its effort and expected time. Its words say what to
  do, how it is accepted, and its **options**, each with a default, so
  that a question they answer needs no one.
- **Repairs have no written spec:** of the goal branch, of an update, of
  a red after landing, of a finding; the engine makes them from records
  (code), their words the evidence, the examiner's section and their
  answers.

### 4.4 A goal's settings

| Setting | Values | What it sets |
|---|---|---|
| `landing` | `goal_branch` or `each` | the goal branch (section 6), or each change readied on its own |
| `ready` | yes or no | the checked goal branch readied, or left unlanded (a replay) |
| `review` | on or off | agent outputs reviewed (10.2) |
| `scaffold` | on or off | parts designed as code first (10.6), or every part direct |
| `early_start` | on or off | early starts (6.2) |

A ladder is a goal of `each` with rungs instead of parts (section 9).

## 5. Runs

### 5.1 A run (LLM; its frame is code)

- **One run of a producer** for one change, in a workspace the engine
  claims for it on a worker. Its base is the goal branch's tip, or its
  own pushed branch when it ran before (a rebase starts afresh).
- **Its brief is compiled, not gathered:** the producer's charter text,
  the spec and its facts; the producer reads what they name.
- **The producer edits, tests and commits;** the engine pushes its
  branch at the run's end. Its result is one of:
  - **finished,** perhaps with a list of what it **left**: outside the
    change and still red, for its owner or the final check;
  - **a question,** with the options and a recommendation: the change is
    kept until it is answered (section 10);
  - **a transient failure** (disk, a machine, a tool): run again,
    spending no try;
  - **failed:** kept for the answerer.
- **Allowed without asking:** the arms, fields and literals the compiler
  requires because of a type the change altered, anywhere in its tree,
  and a scoped lint exception on an item it wrote.
- **Tries:** a change not landed after `tries` runs (rebases aside), or
  a repair of a red or a finding after `fix_tries`, is kept for the
  answerer, whose answer grants fresh tries.

### 5.2 Slots and the flow (code)

- **A worker's slots are a hint, not a cap.** The target of runs in
  flight is the sum of the workers' slots. With `flow` fixed it stays
  there; with `flow` adaptive it grows by one with each first-attempt
  landing made with the target full, and halves with each transient
  failure or rejection (not at a merge or rebase), never below one.
- **A calm worker takes one more:** a worker with low load and no recent
  disk or lane waits takes one run beyond its slots, never beyond its
  workspaces.
- **The order of starts:** consolidations, then repairs of reds and
  findings and the runs that follow a rejection, then shared-shape
  parts' changes, then the rest by slice, part and spec order; the goal
  branch's own repairs after them; early starts last.
- **A start hands on** to another worker with free slots when its
  workspaces are claimed by other work.
- **Workers follow the engine:** before a start, a worker's code moves
  forward to the engine's version, unless pinned.

## 6. The goal branch (code; resolutions and repairs are LLM)

### 6.1 Merging

- **A finished change merges, not lands:** the engine merges its branch
  into the goal branch with a merge commit and pushes it fast-forward
  only. No gate runs here; the check before ready and the goal branch's
  checks below do.
- **Mechanical conflicts settle themselves:** a conflict confined to a
  counted baseline is settled by the project's tooling, taking the goal
  branch's side and recounting, which refuses a raise. Any other
  conflict is a rejection at the merge, and a rebase follows (10.3).
- **A merged change counts as landed for its dependents:** they start
  from the goal branch's tip. Agent tasks' branches and answers merge
  the same way.

### 6.2 Early starts

- **A waiting change starts early** when each unmerged dependency is
  running or finished with its branch pushed, and has no unmerged
  dependencies of its own. Its workspace starts from the goal branch
  with those branches merged in; a conflict refuses the early start.
- **Before it merges,** the engine merges its dependencies' final
  branches into it again; it merges only after they have.
- **Once at most:** after a question, or a failure because its
  dependencies were unfinished, it waits for them to merge.

### 6.3 Interim checks

- **While changes remain,** after `interim_every` merges or
  `interim_after` minutes with one, the engine checks the goal branch's
  tip with the final check's gates less the slowest (`interim_skip`), in
  an idle workspace, when no repair is open.
- **A red becomes a repair** of the goal branch, owning every path: its
  words the failing gates and the report, and the unmerged changes'
  paths, to leave what they own. It merges like any change.

### 6.4 Updating from main

- **Main moves under a goal branch** as other goals land. While changes
  remain, when it lacks main's tip and `update_after` minutes have
  passed since its last update, the engine merges main into it (counted
  baselines recounted). It waits while a repair is open or the planner
  runs. Once every change has merged, it updates only before a check, so
  a ready branch does not move under the queue.
- **A conflicting update is a resolution:** a change that merges that
  main into a branch from the goal branch, keeping both sides' intent,
  and fixes what main's changes break; the engine then checks the goal
  branch contains that main.
- **A stop wakes the goal's agent:** a refusal at a main whose
  resolution has merged, or the `update_refusals`th since the last clean
  update, stops the goal branch's steps; the goal's agent reads the
  record and decides: a fresh resolution with what failed, or a question
  to the person.

### 6.5 The final check, and ready

- **When every change has merged,** the goal branch is updated and
  checked in full in one run: every counted baseline, formatting, lints,
  the build and every test gate the queue runs.
- **Red makes a repair,** and the check runs again; the `final_reds`th
  red stops it.
- **Green makes it ready,** one landing for the whole goal. A queue
  rejection updates it, checks it again and readies it again; the
  `queue_refusals`th stops it.
- **A stop** wakes the goal's agent as in 6.4; meanwhile only repairs of
  reds and findings start.

### 6.6 Goals beside each other

- **An overlap probe, shown only:** each round, for each pair of ready
  goal branches not yet on main, the engine merges each with main in
  memory, then the pair, and records the conflicting paths when they
  change, cached by the commits. Nothing waits on it; the second to land
  meets the conflict at its update.
- **It is shown** on the status page, and the planner's brief lists the
  goals beside it (4.2).

## 7. Landing (code; examiners are LLM)

### 7.1 The landing queue

- **Ready branches, oldest first:** each cycle the queue takes up to
  `candidate_size` untried ready tips, merges them onto main in order
  (`prepare`: merge, or rebase for a candidate of one) into one
  candidate, and runs its gates. A member that conflicts is rejected.
- **Selection** is the packages the candidate reaches and their reverse
  dependencies, with paths that always run and paths tests read; a
  documentation candidate runs only a touched tree's counted baseline.
  Every `full_every`th candidate runs the full selection. A repository
  may run its whole suite instead (`selection = whole`).
- **The gates** run in a gate lane nothing else builds in: counted
  baselines (recounted), formatting, lints, the build, the test gates,
  and any code-style gates of the repository (`gate_set`). CPU is
  measured per tree; starved tests stay green; budgets, when the person
  sets them, are reported.
- **Green:** main is pushed, expecting its parent; each ready ref is
  deleted under a lease on its tip, and its source branch when it still
  names that tip.
- **An error is the gate's:** short of disk, a dirty checkout, a refused
  push; nobody is rejected or marked tried, and it goes again.
- **The queue keeps its records and little else,** and re-derives the
  rest from git.

### 7.2 A change's own check, reused

- **Before ready, in its workspace,** the engine runs the check before
  ready on a change readied on its own, after its run: its tip merged
  onto main, the queue's selection in the build lane, the checkout reset
  after. A red is the change's, back to its producer. Its receipt stays
  on the worker, and the counted-baseline gate runs before it is
  readied.
- **The receipt travels with it:** a candidate of one, not due for a
  full run, whose receipt is green for the same tip and the same main
  lands on it after the counted baselines and formatting alone.

### 7.3 A flake at the gate

- **A failed test rerun alone:** at a test gate with at most
  `gate_reruns` failed tests, each of which passed in every member's
  receipt, each runs again alone. All passing, it is a flake at the
  gate: nobody is rejected or marked tried, and the candidate goes
  again. One failing alone: the red stands.
- **Bounded:** a tip with `gate_flakes` such flakes has its next red
  stand.

### 7.4 Bisecting

- **A red candidate is halved,** the left half first, each half merged
  and checked again; a green half lands before the right is checked,
  until each red is one branch's, which is rejected. After a red full
  candidate, its halves are checked in full.

### 7.5 Who hears a rejection

- **A change:** a rejection record with the gate, its log and failing
  tests, and an examiner decides (10.3); a retry readies the same tip
  again, the queue told to forget it was tried.
- **A goal branch:** updated, checked and readied again (6.5).
- **Every ready branch belongs to a goal,** so every rejection has an
  owner.

## 8. After landing

### 8.1 Tiers follow main (code)

- **One tier runner, one workspace, its own worker.** Each cycle, a main
  tip that is not its cursor (the last tip run without a fault) gets one
  run of the tier set, then the live tiers whose accounts custody holds,
  lent for the run. Tips landed meanwhile are covered by the next run.
- **Suspects** are the first-parent commits since the cursor; a queue
  merge names its branch.
- **A red is confirmed:** each failed test runs again alone. Failing
  again, it is red; passing, a flake. Against its history a red is
  **new**, **standing** or **unattributed** (never ran); a pass after a
  red is **recovered**.
- **Faults and starvation:** a tier that cannot run (no account, disk, a
  missing report) faults and says nothing about its tests; wall-clock
  expiry is **starved**, counted apart, never red; CPU is measured. A
  run is red, partial or green; the cursor moves only past a run without
  a fault.
- **Budgets** are the person's to set; with none, CPU and wall are
  reported only.

### 8.2 A red reaches its owner (code; the examiner is LLM)

- **Grouped once:** one red per tip and suspect set, for the new and
  unattributed reds still red in the newest run, with its evidence (the
  tests, the suspects, the reruns' log ends).
- **A red with a goal's change among its suspects goes to an examiner**
  (10.3), which reads the evidence and the suspects' specs at the red's
  tip: **fix** (what, where, and the owner among the suspects), or **not
  the suspects'** (a flake that failed its rerun too, the tier's
  environment), which makes it an alert.
- **The repair is a change of its own,** from main, in the owner's goal:
  reproduce in the lowest tier that shows it, then fix; readied like any
  change; `fix_tries`, then the answerer.
- **Closed by the same checks:** when every test recovers, or is absent
  from a later run with no fault, starvation or unobserved test. A
  covering run that still lists the test makes it **unfixed**, an alert.
- **A red with no open goal among its suspects,** and every fault, is an
  alert (11.2); a fault clears only when a run no longer faults.
- **No bisecting after landing:** the examiner names the owner.

### 8.3 A goal closes after its verdict (code; the closing task is LLM)

When every part is done, a goal waits until its last branch has landed
on main and a tier run covering it shows no open red of its own; then
the closing task runs (10.6).

## 9. QA walks and the findings loop

### 9.1 The ladder (code)

- **A ladder is a goal of rungs:** each rung is a story on a backend (a
  model with an owned Forgejo or GitHub), its brief files (a shared kit,
  then its own), and the rungs that must pass first. The ladder names
  its happy path (a design citation) and its backlog file.
- **A rung's edition is its brief's bytes:** "rerun unchanged" is a walk
  at the same edition; a changed brief or kit starts a new edition.
- **A rung's state:** passed, walking, waiting (a lower rung has not
  passed), holding (a blocking finding, or one from a failed walk, is
  open), or due (a first walk, or a rerun at a main holding its
  repairs).

### 9.2 A walk (code starts it; the walker is LLM)

- **A due rung starts a walk:** the engine records its start at main's
  tip (never a commit lacking a landed repair of the rung's findings),
  builds the binary at that commit in a workspace, lends the backend's
  accounts from custody (`walk_boxes` at once), and starts the walker,
  an agent task on the rung's brief, the open findings and how to
  report, under a time cap (`walk_cap`). It returns the accounts at the
  end.
- **The result** is a verdict (pass, fail, void), a reason, and
  findings: title, severity, area, whether it blocked, reproduction,
  expected, seen, evidence, and `same`, an open finding met again. The
  engine records each finding, then the walk's end.

### 9.3 Findings and triage (code; the triager is LLM)

- **A finding is a record:** new, routed, backlog, dropped, asked,
  merged, or in repair. The same finding is merged by the walker's
  `same` or by triage, never filed twice; a sighting of one whose repair
  has landed is filed again, `again`, for triage.
- **The triager is an agent task in the chat's role,** started at once
  for a blocking finding or one from a failed or void walk, else at
  `triage_at` new findings or after `triage_after` minutes. One line per
  finding: **path** (its owner; perhaps an action the person must take
  before the rerun), **backlog**, **user** (a question to the person),
  **brief** (the rung's brief at fault: the ladder goal's agent rewrites
  it, or asks the person), **merge**, or **drop**.
- **The backlog** is a file in the tree, rendered from the records and
  readied whenever it differs from main; two rejections of one render
  raise an alert.

### 9.4 A repair, and the pass (code; the examiner is LLM)

- **An on-path finding goes to an examiner,** then a repair from main,
  as a red's (8.2): reproduce first in the lowest world that shows it,
  red before and green after, then fix, design text changed with it.
- **A pass** is recorded when a walk at the rung's edition passes after
  an earlier walk of that edition that passed or failed, nothing holding
  the rung, its lower rungs passed. A finding whose repair landed is
  then fixed; one filed again is unfixed. A failed walk reopens it.

## 10. Agent tasks and judgement

### 10.1 Agent tasks (LLM; started and measured by code)

- **Each is a fresh run on a compiled brief,** ending with its result or
  files the engine pushes. None waits, polls or watches a run; each runs
  under its charter's model and effort.
- **Measured, not capped:** peak context, turns and tokens are recorded
  against the person's target (`context_target`); a timeout
  (`agent_timeout`) is the only stop.
- **Who starts them:** the chat starts goals; a goal's agent starts its
  planner; the engine starts the producers, examiners, the answerer,
  reviewers, revisers, arbiters, scaffolds, the structure review, the
  closing task, the triager, walkers and the forge's reply. One runs per
  subject at a time.

### 10.2 Review (LLM; the counts are code)

- **With `review` on,** an agent task's branch gets a read-only reviewer
  that reports at most ten lines: approve, or refuse with remarks.
  Refused, the remarks go once to a reviser on the same branch; refused
  again (or the reviser failed, or three reviews faulted), it is held
  for an **arbiter**, which says land, revise (once more, with guidance)
  or the person's (budgets, machines, what is hard to reverse, a change
  of direction).
- **A landing's rejection is a review's refusal:** a rejection of the
  branch (its merge into the goal branch, the queue) counts: revised
  when there has been no revision since its last reset, else held for
  the arbiter.
- **A structure review** reads the digests of the changes landed since
  the last (every `review_every` landings or `review_after` hours) and
  writes consolidation changes, which start first, and examples for the
  producers' charter.
- **With `review` off,** an agent task's branch merges as it ends.

### 10.3 The examiner (LLM; what it decides is recorded by code)

A read-only agent task that decides one thing and writes no code. One
per question, rejection, red or finding; any other end is a fallback:
the question queues, or the repair starts with the evidence alone.

- **A question** from a change with a written spec (a repair's questions
  queue at once, and their answers join its words): when the spec's
  options answer it, the answer joins the spec and the change runs
  again; when its work is pushed and accepted and what it asks lies
  outside it, **accept** finishes the change (a goal branch's);
  otherwise the question queues for the answerer.
- **A rejection:** fix (a repair or rebase section for the next run) or
  retry (not the change's: the same tip goes back, once).
- **A red after landing, an on-path finding:** fix (a repair change), or
  not the code's (an alert).

### 10.4 The answerer (LLM; when it runs is code)

- **One task answers a goal's queued questions** in one pass: at once
  when one holds up other work, or always with `answer_at_once`;
  otherwise at `answer_at` queued or after `answer_after` minutes.
- **It prefers leaving what is not the change's** to its owner or the
  final check, widening a change only when its acceptance needs it.
- **What it cannot settle** it asks the person (11.1).

### 10.5 Kept (code)

- **Only the change that needs an answer is kept;** its dependents wait,
  everything else runs. An answer releases it.
- **A part named by an open question** of the person's is not designed.
- **An agent task's question keeps its subject** until the person
  answers.

### 10.6 Scaffolds and the closing task (LLM; their frames are code)

- **A scaffold** designs a part as code (types, signatures, empty
  bodies, its specs) when `scaffold` is on, started when the ready
  changes fall below a band; a part shared by others is scaffolded
  first. The engine checks the scaffold's tip (gates and tests) and
  starts the part's changes from it before it merges; a part waits for
  its dependencies to be designed, not done.
- **Changes started from a scaffold wait for its review:** a finished
  change merges only once the scaffold has; if it was revised, each such
  change gets one rebase onto it, outside its tries; a held scaffold
  keeps its changes.
- **The closing task:** once a goal's parts are done (and after 8.3),
  one task brings the changed designs' status files current; its branch
  lands, and the goal ends. Its brief quotes each changed design as a
  diff, within a bound.
- **A decision task** turns a probe's report into a design section; the
  probe itself is not built (15).

## 11. The person, the chats and the forge

### 11.1 Questions and the inbox (code; the questions are LLM)

- **Only what is the person's reaches the person:** budgets, accounts,
  machines, what is hard to reverse, a change of direction. Whoever asks
  decides: a planner's question, an agent task's, an arbiter's referral,
  a triager's `user`, a goal's agent after a stop.
- **The person's inbox** lists the open questions with what to read and
  how to answer; an answer is recorded once, carried in every later
  brief of its subject, and written into the plan where it is a plan's.

### 11.2 Alerts (code; acting on them is LLM)

- **An alert is what no goal owns,** sent to the chat its template names
  (by default the person's): a red no open goal owns or one left
  unfixed, a tier that cannot run, a finding its examiner called not the
  code's, a backlog branch rejected twice.
- **The chat's agent acts:** it starts a goal for it, waits for it to
  clear, or asks the person. Each alert opens and clears from the
  records, or when the chat clears it with a reason (but a fault).

### 11.3 On the forge (code; the reply is LLM)

- **One issue per open question** of an agent task, a held review or a
  triager (never a plan's), labelled on the forge, opened, conversed and
  closed by a pass on a timer (`forge_sync`).
- **The person answers in a comment:** a reply task reads it with the
  question and its report, and either replies (context, options, a
  recommendation) or records the answer, and the issue closes.

## 12. Workers, workspaces and accounts (code)

### 12.1 Workers and workspaces

- **The engine's machine** runs the engine, the agent tasks, the
  inboxes, custody and the status page, and builds nothing. **Workers**
  run producers and builds; one also holds the gate lane and the tier
  runner.
- **A workspace is claimed** for a run and swept at its end (incremental
  data, stale test executables), which bounds it (`workspace_bound`); a
  claim whose run is gone is stale and released.
- **Builds wait for disk:** in a build lane of `builds` places per
  worker, or the gate lane; short of headroom, a build reclaims idle
  workspaces and waits up to `disk_wait`, then fails as transient.
- **A worker's disk is budgeted:** the cache's cap, its workspaces at
  their bound, headroom, and one more on the gate's worker.

### 12.2 Custody

- **Every account is held centrally,** one entry each: the workers'
  model accounts, and QA's model records, Claude Code directories and
  forge fixtures.
- **Custody is records:** added, lent (to whom, where), returned,
  checked, retired. A lent account's central copy is a placeholder until
  returned; tokens refresh, so the copy last used is the true one; bytes
  are moved and hashed, never printed.
- **Lent to workers and to runs:** each machine holds one account of
  each model; a live tier or a walk borrows for its run.
- **Checked on a timer** (`account_check`): the account and every usage
  limit. A machine whose account passes `swap_at` of its weekly limit is
  swapped onto a free account of the same model.

## 13. What is measured and reported (code)

- **Two reports a day, from records alone,** by a recurring task: the
  code report per goal (runs and results, runs in flight against slots,
  idle gaps, time per change, questions and time to answer, transient
  minutes, disk and lane waits, each part's time to landed, agent and
  producer usage and their weights, stops ranked by time lost) and the
  landing report (candidates and outcomes, rejections by gate, flakes at
  the gate, candidates landed on a receipt, ready-to-landed latency,
  bisecting, gate durations, CPU per tree; reds opened, routed, settled
  and closed; walks, findings, triage, repairs and passes). Each figure
  stands beside the day before's; they go to the chat its template
  names.
- **Weights:** input 1, cache write 1.25, cache read 0.1, output 5 for
  Claude and 8 for Codex; the ratio per goal, with the chats' own runs
  shown apart.
- **A public status page** shows counts only: what is in flight, landed
  and merged today, questions waiting and their age, each goal's
  activity, overlaps, and the accounts' use and pace; never a branch,
  path or text.

## 14. fabrig's model as configuration

### 14.1 The choices

fabrig's way of running, as fab runs it, is these choices. Values are
fab's (`fab/` at 1dc102c) unless marked.

| Choice | fabrig's value |
|---|---|
| the chat | one, a director template, for the person; alerts and reports to it |
| a draft's goal (4.4) | `landing = goal_branch`, `ready` yes, `review` off, `scaffold` off, `early_start` on |
| a small fix's goal | `landing = each` |
| fab's own repository | `landing = each`, `candidate_size` 1, `prepare` rebase, `selection = whole` (its suite, 120 s), `gate_set` code style (no changed signature but for an interface or migration, no function over 60 lines added, no banned name, docs on public items, no repeated block); `review` on for its rebuild |
| QA | one ladder goal, `landing = each`, three backends (ChatGPT with an owned Forgejo or GitHub, Claude Code with an owned Forgejo) |
| `tries`, `fix_tries` | 4, 2 |
| `flow` | fixed for a goal branch, adaptive for `each` |
| workers' slots | 6, 4 and 4 (the third holds the gate lane and the tier runner) |
| `interim_every`, `interim_after`, `interim_skip` | 10 merges, 45 minutes, the fuzzy tests |
| `update_after`, `update_refusals` | 30 minutes, 3 |
| `final_reds`, `queue_refusals` | 3, 3 |
| `candidate_size`, `full_every` | 8, 5 |
| `gate_reruns`, `gate_flakes` | 5, 2 |
| the tier set | the browser tiers of jig, temper and skein, temper's process tier; live: skein's |
| `answer_at_once`, `answer_at`, `answer_after` | on for a goal branch; 4, 30 minutes |
| `triage_at`, `triage_after` | 4, 30 minutes |
| `review_every`, `review_after` | 10, 4 hours |
| `context_target`, `agent_timeout` | about 600k (the user, 10-30), 60 minutes |
| `backstop`, `self_wakes` | 10 minutes, 5 |
| models | producers Codex at high, examiners Codex at medium, the others Claude (the answerer, arbiter and triager at medium) |
| `walk_cap` | 3 hours (designed in `fab:qa.md` 2) |
| `forge_sync`, `account_check` | 2 minutes, 30 minutes |
| `workspace_bound`, `disk_wait` | 25 GB, 20 minutes |

### 14.2 fab's gaps

Where fab today works by hand, has an agent do a mechanism, or code make
a judgement, the model does not copy it:

- **By hand:** the person or a session starts the cut (no chat creates a
  goal); answers to a plan's questions written into plan.toml; ready
  refs pushed by hand, which no plan owns (their rejections and reds
  become signals); the hand merge of main after a stop; walks run by a
  session (`fab qa walk`, `fab qa result`); account swaps (`fab logins
  swap`); the daily reports (`fab report daily`); signals read and
  cleared by a session.
- **An agent doing a mechanism:** the cut pushes its pieces and runs
  `fab wake cut`; the implementer pushes its branch, runs the check
  before ready (`fab precheck`) and merges its early-start dependencies
  again before pushing; the reviser rebases a branch `fab land` refused,
  as text work.
- **Code doing a judgement:** a goal's stop is a note, not a question to
  anyone (`fab:integration.md` 3 designs the question); a red's repair
  goes to the newest suspect's plan whatever owner its examiner names.
- **Faults:** an integration plan's retire branch merges into a trunk
  that has already landed, so it never reaches main; the rebase onto an
  amended skeleton repeats without end in an integration plan
  (`call_review.task_rebase` looks for a base of main).

## 15. Future ideas

**Not to be carried out now.** These are in the previous edition or in
fab's designs, fab does not do them today, and they may be needed later.
Each waits for the numbers or the person's word that asks for it.

- **From the person's execution model:** scaffolds with a test
  vocabulary by default; surveys before the planner and each scaffold;
  test-writer and acceptance runs; a self-critique before ready; probe
  runs; a slice complete when its story test passes on main; a pull
  buffer of ready changes driving design.
- **Landing:** receipts by key per package (the previous edition's keys,
  gate classes and audits); receipts reused for candidates of several;
  speculative candidates (a train) or a second gate lane; the candidate
  size from measures; budgets that turn red; evidence bound to trees.
- **After landing:** bisecting to a culprit when examiners name the
  wrong owner; a second repair of an unfixed red; QA passes on landings
  marked for QA.
- **The goal branch:** a build of overlapping goal branches' merge;
  landing at slice boundaries; overlaps that reorder work; a light lint
  after each merge.
- **QA:** a passed rung's story joining the live tier, with its fake
  twin in the worlds; findings matched by a normalised key; findings of
  other finders; findings as temper's own records on the forge.
- **Agent tasks:** a cap on context or turns once the person sets one; a
  question that holds a direct part's changes; dependencies across
  goals.
- **Workers:** added and retired by demand; accounts lent to a new
  worker when it joins.
- **Landings across repositories,** and a forge's native merge queue as
  an accelerator.

