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
said. Where fab does a step by hand, or mixes code and judgement, the
model has a mechanism of its own, and section 14.2 says so.
`reference-case.md` is one story that runs every mechanism, and ends
with what temper and jig have and lack for each. Section 15 holds what
fab does not do but may need later; it is not to be carried out now.
Other designs cite revision 4's sections and gap ids; they are repointed
when this one is adopted.

Words are jig's, then temper's; section 2 names the few that are new.
Unqualified, `landing.md`, `after-landing.md`, `findings.md`, `forge.md`
and `worker.md` are temper's `../domain/` documents; fab's designs are
named `fab:<name>`; jig's are named as jig's.

## 1. In one page

- **An LLM for judgement, code for mechanisms.** Every mechanism here is
  code: scheduling, merging, rebasing, queueing, checks, bisecting,
  retries, holds, lending, reports. Every judgement is an agent task:
  planning, producing, resolving a conflict, answering, examining,
  reviewing, arbitrating, triaging, walking. Nothing mixes them; each
  heading says which it is.
- **The person talks to a chat through `temper ctl`; everything else is
  automatic.** The chat, made from a template of the person's choosing,
  has tools that create tasks. Given a draft, or "fix this", it creates
  a goal; the goal's agent decides the sub-tasks; mechanisms take each
  change to landed; only questions reach the person, in their inbox
  (sections 4, 11).
- **The engine decides each mechanical step** from its store, which it
  alone writes; tasks, plans, specs, answers and findings live there,
  never in a repository, which holds only the project's own code and
  documents (section 3).
- **A planner plans a goal** and hands it over in pieces, so producers
  start while it plans the rest (section 4).
- **Producers are runs in workspaces,** as many as the workers take;
  each ends finished, with a question, or failed (section 5).
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
- **Judgement is in agent tasks,** a fresh one for each decision; only
  the chat and a goal's agent run long (section 10); custody lends
  accounts and workers take runs (section 12); reports come from the
  store (section 13).

## 2. Two layers, and words

- **jig is generic:** tasks and their delegation, specs, questions and
  the inbox, holds, wakes and timers, the store, a settings store with
  revisions, templates and charters, accounts, workers, slots and jobs.
  It knows nothing of code, branches or landing, and would serve any
  application that uses an LLM for judgement and code for mechanisms.
- **temper is the software-delivery application on it:** changes, goal
  branches, the landing queue, gates and receipts, tiers, findings,
  walks and custody of forge credentials are temper's mechanisms, built
  from jig's primitives. Its settings (4.4, 14.1) are temper's own kinds
  of setting, declared to jig's settings store as an application's kinds
  (jig `engine.md` 17.5, designed) and scoped to a goal or a project.
- **Words:** most are jig's (task, goal, chat, template, charter, plan,
  spec, brief, run, try, kept, held, wake, timer, inbox, worker, slot,
  workspace, job, account, grant) or temper's (change, producer, landing
  queue, candidate, gate, receipt, tier, red, flake, fault, repair,
  resolution, update, finding, triage, and the walk, ladder and rung of
  `draft/v0.1.0.md`, cited from `testing.md` 2.4). jig's tries count
  failed attempts per class; here a change also has `tries`, its runs
  not landed. These are new, for what neither names:

| Word | What it is |
|---|---|
| planner | the agent task that turns a draft into a goal's plan: its tasks and their specs, in the store |
| part, slice | a unit of the plan; a story that runs end to end |
| goal branch | the branch a goal's changes merge into before it lands whole (temper's "a branch per goal", `forge.md` 11.1, designed) |
| early start | a change started on a dependency's pushed, unmerged branch |
| interim check, final check | the goal branch checked whole along the way, and before ready |
| check job, tier job | a gate's or a tier's commands run in a workspace with no agent (jig's job) |
| counted baseline | a file of counts of old forms that may only fall, recounted by the project's tooling |
| lane | a worker's queue for builds: a build lane of a few at once, a gate lane of one |
| run target | a worker's hint of how many runs it takes; its slots are the hard count |
| examiner | a read-only agent task that decides one thing: a question, a rejection, a red, a finding |
| answerer | the agent task that answers a goal's queued questions in one pass (the party jig's question credit names as answerer) |
| reviser, arbiter | the agent task that revises a reviewed output once; the one that decides after a second refusal |
| alert | an inbox item for a chat: something no goal owns |
| edition | a rung's brief, identified by its bytes |
| scaffold | a planning task that writes a part as code before its changes fill it |
| wrap-up | the agent task that ends a goal: the project's duty at the end, such as status files current |
| custody | the accounts and the application's credentials, held centrally and lent |

## 3. The engine and its store (code)

- **One writer.** The engine decides each event once, commits its
  decision to its store, which it alone writes, and only then acts (jig
  `README.md` 6.1). The landing queue's and the tier runner's records
  are temper's too, imported into the store (jig keeps such connector
  records, `engine.md` 5.4); none lives in a repository.
- **The store is temper's only data:** goals, plans, specs, answers,
  findings, rungs and every record live there. A repository holds the
  project's code and its own documents (its designs, its drafts),
  nothing temper-specific; temper reads it and lands changes into it.
- **Status is derived, never stored:** a change's (waiting, runnable,
  running, kept, held, finished, ready, landed, rejected), a part's, a
  red's, a finding's, a rung's and every open question are functions of
  the specs and the records.
- **Events wake it:** a run's or an agent task's end, a planner's piece,
  a candidate landed or rejected, a tier run with a new red or a
  recovery, an answer, a walk's result. A run whose worker is lost ends
  as lost, and runs again (jig `tasks.md` 5.5).
- **One engine serves every goal** and one set of workers.

## 4. Intake, goals, plans and changes

### 4.1 The chat and a goal (LLM)

- **The person talks to a chat** through `temper ctl`: a free-form,
  possibly long-running agent task made from a template of their
  choosing, such as a director, or a department such as QA. Its charter
  sets its tools; which alerts and reports reach it is a setting
  (`alerts_to`).
- **The chat's tools create tasks.** Given a draft, or a request such as
  "fix this", its agent creates a goal with the settings its template
  allows (4.4), naming the draft or the request in the goal's spec.
- **The goal's agent decides its sub-tasks:** for a draft, a planner
  (4.2); for a small fix, one change; later, whatever its records ask
  for: a fresh resolution after a hold (6.4); and it revises a rung's
  brief (9.3). From there every step to landed is a mechanism or another
  agent task's judgement; no one works by hand.
- **The plan is the goal's, in the store:** jig's plan, the subtree of
  tasks the goal delegates (jig `core.md` 7.1), with each task's spec;
  nothing of it is written to a repository.

### 4.2 The planner (LLM; its hand-over is code)

- **One agent task writes the plan:** its parts, their dependencies, the
  slices (and whether QA walks each), the person's questions, and every
  change's spec, with notes several specs share and the shapes that
  cross parts written first. It may survey the code with read-only runs
  of its own, and split the design across agents of its own, from shared
  shapes it fixes first; it is still one task, under one timeout. With
  `scaffold` off, every part is direct: one or two changes, no scaffold.
- **It hands the plan over in pieces** (`scaffold` and `review` off):
  first the changes that depend on nothing, then each batch that holds
  together; code makes each piece, on the planner's word, as one batch
  of the goal's delegates (jig `tasks.md` 4) with their specs, and the
  engine starts them at once. It never rewrites a spec it has handed
  over. With `review` on, the plan waits for its review (10.2).
- **It sees the goals beside it:** its brief lists the other goals'
  unlanded changes with the paths each owns, and asks it to keep early
  work off them, naming any overlap it cannot avoid (6.6).
- **The person's questions** name the parts that wait for their answer.

### 4.3 A change and its spec (LLM writes it; code reads it)

- **A change's spec** is words with typed parameters: its part, its kind
  (from the project's list, `change_kinds`), the paths it owns (for the
  planner and the producer; never enforced) and its expected time;
  beside it, as jig's task fields, its dependencies and its LLM choice
  (effort). Its words say what to do, how it is accepted, and its
  **options**, each with a default, so that a question they answer needs
  no one.
- **Repairs have no written spec:** of the goal branch, of a red after
  landing, of a finding; the engine makes them from records, their words
  the evidence, the examiner's section and their answers, all in the
  store.

### 4.4 A goal's settings (code; temper's)

temper's settings for a goal's landing, an application kind on jig's
settings store, scoped to the goal; jig only stores and scopes them.


| Setting | Values | What it sets |
|---|---|---|
| `landing` | `goal_branch` or `each` | the goal branch (section 6), or each change readied on its own |
| `ready` | yes or no | the checked goal branch readied, or left unlanded (a replay) |
| `review` | on or off | planning and judgement outputs reviewed (10.2) |
| `scaffold` | on or off | parts designed as code first (10.6), or every part direct |
| `early_start` | on or off | early starts (6.2) |

A ladder is a goal of `each` with rungs instead of parts (section 9).

## 5. Runs

### 5.1 A run (LLM; its frame is code)

- **One run of a producer** for one change, in a workspace the worker
  prepares (jig `hosts.md` 7). Its base is the goal branch's tip, or its
  own pushed branch when it ran before.
- **Its brief** is assembled by the engine (jig `engine.md` 9): the
  producer's charter text, the spec and its facts; the producer reads
  what they name.
- **The producer edits, tests and commits;** the engine pushes its
  branch at the run's end. It ends:
  - **finished,** perhaps with a list of what it **left**: outside the
    change and still red, for its owner or the final check;
  - **with a question,** the options and a recommendation: the change is
    kept until it is answered (section 10);
  - **failed:** held, and escalated to the answerer.
- **A transient failure** (a build short of disk, a lost worker) is
  classed by code from the tool's exit or the worker's state, never from
  the agent's words; the engine runs the change again without asking,
  from its pushed branch. It counts toward `tries` and narrows an
  adaptive flow. A turn refused at the model's capacity is resumed in
  the same run after waits (`capacity_waits`).
- **What it may do without asking** is its charter's
  (`producer_allowances`): edits outside its paths that its own change
  forces, within limits the project names.
- **Tries:** a change not landed after `tries` runs (resolutions aside),
  or a repair of a red or a finding after `fix_tries`, is held and
  escalated to the answerer, whose answer grants fresh tries.

### 5.2 Slots, run targets and the flow (code)

- **A worker has slots,** the runs it can host at once, **and a run
  target,** a hint (`run_target`). The target of runs in flight is the
  sum of the workers' run targets. With `flow` fixed it stays there;
  with `flow` adaptive it grows by one with each first-attempt landing
  made with the target full, and halves with each transient failure or
  rejection (not a conflict), never below one.
- **Placement:** a run goes to the worker furthest under its run target;
  a worker under its calm thresholds (`calm`: load, recent lane and disk
  waits, refused builds) takes one run beyond its target, never beyond
  its slots. A start whose worker's slots are all taken hands on to
  another worker under its target.
- **The order of starts:** consolidations, then repairs of reds and
  findings and the runs that follow a rejection, then shared-shape
  parts' changes, then the rest by slice, part and spec order; the goal
  branch's own repairs after them; early starts last.
- **Workers follow the engine:** before a start, a worker's code moves
  forward to the engine's version, unless pinned.

## 6. The goal branch (code; conflicts and repairs are LLM)

### 6.1 Merging

- **A finished change merges, not lands:** the engine merges its branch
  into the goal branch with a merge commit and pushes it fast-forward
  only. No gate runs here; the check before ready and the goal branch's
  checks do.
- **Mechanical conflicts settle themselves:** a conflict confined to a
  counted baseline is settled by the project's tooling, taking the goal
  branch's side and recounting, which refuses a raise. Any other
  conflict is a rejection at the merge, which goes to an examiner
  (10.3); the next run starts with a resolution (below).
- **Conflicts are resolved, never rebased:** the engine merges the base
  (the goal branch, main, or a revised scaffold) into the branch in a
  workspace; where it conflicts, an agent task resolves the conflicting
  files only, with no git writes of its own, and the engine commits the
  merge and pushes (temper's resolution, `forge.md` 8.4). A branch only
  moves forward (`forge.md` 10).
- **A merged change counts as landed for its dependents:** they start
  from the goal branch's tip. Scaffolds' and the wrap-up's branches,
  which hold code and documents only, merge the same way.

### 6.2 Early starts

- **A waiting change starts early** when each unmerged dependency is
  running or finished with its branch pushed, and has no unmerged
  dependencies of its own. Its workspace starts from the goal branch
  with those branches merged in, with no recount; any conflict refuses
  the early start.
- **Before it merges,** the engine merges its dependencies' final
  branches into it again; it merges only after they have.
- **Once at most:** after a question, or a failure because its
  dependencies were unfinished, it waits for them to merge.

### 6.3 Interim checks

- **While changes remain,** after `interim_every` merges or
  `interim_after` minutes with one, the engine runs a check job on the
  goal branch's tip with the final check's gates less the slowest
  (`interim_skip`), in an idle workspace, when no repair is open.
- **A red becomes a repair** of the goal branch, owning every path: its
  words the failing gates and the report, and the unmerged changes'
  paths, to leave what they own. It merges like any change.

### 6.4 Updating from main

- **Main moves under a goal branch** as other goals land. For a goal
  with `ready` yes, while changes remain, when the goal branch lacks
  main's tip and `update_after` minutes have passed since its last
  update, the engine merges main into it (counted baselines recounted).
  It waits while a repair is open. Once every change has merged, it
  updates only before a check, so a ready branch does not move under the
  queue.
- **A conflicting update is a resolution** (6.1): the engine starts the
  merge of that main in a workspace from the goal branch; an agent
  resolves the conflicts, keeping both sides' intent; the engine commits
  it and merges it. What main's changes break beyond the conflicts, the
  next check finds and a repair fixes.
- **Stopped after `update_refusals`:** that many refused updates since
  the last clean one stop the goal branch's steps, which escalate to the
  goal (jig `tasks.md` 5.5, 9); its agent reads the record and decides:
  a fresh resolution with what failed, or a question to the person.

### 6.5 The final check, and ready

- **When every change has merged,** the goal branch is updated and
  checked in full by one check job: every gate the queue runs (`gates`),
  in its full selection.
- **Red makes a repair,** and the check runs again; the `final_reds`th
  red stops the goal branch.
- **Green makes it ready,** one landing for the whole goal. A queue
  rejection updates it, checks it again and readies it again; the
  `queue_refusals`th stops it.
- **A stopped goal branch** goes to the goal's agent as in 6.4;
  meanwhile only repairs of reds and findings start.

### 6.6 Goals beside each other

- **An overlap probe, shown only:** each round of the engine, for each
  pair of goals with `ready` yes whose goal branches are not yet on
  main, it merges each with main in memory, then the pair, and records
  the conflicting paths when they change, cached by the commits. Nothing
  waits on it; the second to land meets the conflict at its update.
- **It is shown** in `temper ctl`'s status, and the planner's brief
  lists the goals beside it (4.2).

## 7. Landing (code; examiners are LLM)

### 7.1 The landing queue

- **Ready changes, oldest tip first:** readiness is a record in the
  store (the change, its branch and tip), never a ref in the repository;
  each cycle the queue takes up to `candidate_size` untried ready
  changes from the store, by their commits' time, prepares them onto
  main in order (`prepare`, a setting: merge, or rebase for a candidate
  of one) into one candidate, and runs its gates. A member that
  conflicts, or touches a path the repository refuses (`refused_paths`),
  is rejected.
- **Selection** (`selection`) is the packages the candidate reaches and
  their reverse dependencies, with paths that always run and paths tests
  read, or the whole suite. A candidate the selection finds
  documentation-only runs only `docs_gates`. Every `full_every`th
  candidate runs the full selection.
- **The gates** (`gates`) run in a gate lane nothing else builds in; a
  counted baseline is recounted, refusing a raise. Gates of a change
  alone (`gate_set`) run before it is readied (7.2). CPU is measured;
  starved tests stay green; budgets, when the person sets them, are
  reported.
- **Green:** main is pushed, expecting its parent (`push_attempts`);
  each change's branch is deleted when it still names the landed tip,
  and its readiness recorded as landed.
- **A fault of the gate** (short of disk, a dirty checkout, a refused
  push) rejects nobody and marks nobody tried; the candidate goes again.
- **The queue reads git only for code** (tips, main's ancestry); what it
  tried, landed and rejected are its records, in the store.

### 7.2 A change's own check, reused

- **Before ready, in its workspace** (`check_before_ready`), the engine
  runs the check before ready on a change readied on its own, after its
  run: a check job on its tip merged onto main, the queue's selection,
  in the build lane. Then the change's own gates (`gate_set`) run. A red
  is the change's, back to its producer. The engine carries the receipt
  to the gate lane with the change's readiness.
- **The receipt is reused:** a candidate of one, not due for a full run,
  whose receipt is green for the same tip and the same main lands on it
  after `receipt_gates` alone.

### 7.3 A flake at the gate

- **A failed test rerun alone:** at a test gate with at most
  `gate_reruns` failed tests, each of which passed in every member's
  receipt, each runs again alone. All passing, it is a flake at the
  gate: nobody is rejected or marked tried, and the candidate goes
  again. One failing alone: the red stands.
- **Bounded:** a tip with `gate_flakes` such flakes has its next red
  stand.

### 7.4 Bisecting

- **A red candidate is halved,** the left half first, each half prepared
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

- **One tier runner, one workspace.** Each cycle, a main tip that is not
  its cursor (the last tip run without a fault) gets one tier job of the
  tier set, then the live tiers whose accounts custody holds, lent for
  the run. Tips landed meanwhile are covered by the next run.
- **Suspects** are the first-parent commits since the test last passed
  (since the cursor for one that never ran); a queue merge names its
  branch.
- **A red is confirmed:** each failed test runs again alone. Failing
  again, it is red; passing, a flake. Against its history a red is
  **new**, **standing** or **unattributed** (never ran); a pass after a
  red is **recovered**.
- **Faults and starvation:** a tier that cannot run (no account, disk, a
  missing report) faults and says nothing about its tests; wall-clock
  expiry is **starved**, counted apart, never red; CPU is measured.
- **A run's verdict:** red (any confirmed red, or over a set budget),
  else partial (a fault), else green. A partial run's tip is not run
  again: the next tip's run covers its suspects. A red run with a fault
  runs its tip again.
- **Budgets** are the person's to set; with none, CPU and wall are
  reported only.

### 8.2 A red reaches its owner (code; the examiner is LLM)

- **Grouped once:** one red per tip and suspect set, for the new and
  unattributed reds still red in the newest run, with its evidence (the
  tests, the suspects, the reruns' log ends).
- **A red with an open goal's change among its suspects goes to an
  examiner** (10.3), which reads the evidence and the suspects' specs at
  the red's tip: **fix** (what, where, and the owner among the
  suspects), or **not the suspects'** (a flake that failed its rerun
  too, the tier's environment), which makes it an alert.
- **The repair is a change of its own,** from main, in the owner's goal:
  reproduce in the lowest tier that shows it, then fix; readied like any
  change; `fix_tries`, then the answerer.
- **Closed by the same checks:** when every test recovers, or is absent
  from a later run with no fault, starvation or unobserved test. A
  covering run that still lists the test makes it **unfixed**, an alert.
- **A red with no open goal among its suspects,** and every fault, is an
  alert (11.2); a fault clears only when a run no longer faults.
- **No bisecting after landing:** the examiner names the owner.

### 8.3 A goal ends after its verdict (code; the wrap-up is LLM)

When every part is done, a goal waits until its last branch has landed
on main and a tier run covers it, and while any red of its own is being
decided or repaired (an alert does not hold it); then the wrap-up runs
(10.6).

## 9. QA walks and the findings loop

### 9.1 The ladder (code)

- **A ladder is a goal of rungs,** in the store: each rung is a story on
  a backend (`backends`: a model and a forge), its brief (a shared kit,
  then its own), and the rungs that must pass first. The ladder names
  its happy path (a design citation).
- **A rung's edition is its brief's bytes:** "rerun unchanged" is a walk
  at the same edition; a changed brief or kit starts a new edition.
- **A rung's state:** passed, walking, waiting (a lower rung has not
  passed), holding (a blocking finding, or one from a failed or void
  walk, is open, or an action of the person's it needs), or due (a first
  walk, or a rerun at a main holding its repairs).

### 9.2 A walk (code starts it; the walker is LLM)

- **A due rung starts a walk:** the engine records its start at main's
  tip (never a commit lacking a landed repair of the rung's findings),
  builds what the walk deploys at that commit in a workspace, lends the
  backend's accounts from custody (at most `walk_boxes` walks at once),
  and starts the walker, an agent task on the rung's brief, the open
  findings and how to report, under a time cap (`walk_cap`). It returns
  the accounts at the end.
- **The result** is a verdict (pass, fail, void), a reason, and
  findings: title, severity, area, whether it blocked, reproduction,
  expected, seen, evidence, and `same`, an open finding met again. The
  engine records each finding, then the walk's end.

### 9.3 Findings and triage (code; the triager is LLM)

- **A finding is a record:** new, routed, backlog, dropped, asked,
  merged, left (its examiner's "not the code's"), or in repair. The same
  finding is merged by the walker's `same` or by triage, never filed
  twice; a sighting of one that cannot take a merge (its repair landed,
  fixed, unfixed, left or dropped) is filed again, `again`, for triage.
- **The triager is an agent task in the chat's role,** started at once
  for a blocking finding or one from a failed or void walk, else at
  `triage_at` new findings or after `triage_after` minutes. One line per
  finding: **path** (its owner; perhaps an action the person must take
  before the rerun), **backlog**, **user** (a question to the person),
  **brief** (the rung's brief at fault: the ladder goal's agent revises
  it in the store, a new edition, or asks the person), **merge**, or
  **drop**.
- **The backlog** is the findings in state backlog, in the store, shown
  through `temper ctl`. A project may also have them rendered into a
  document of its own and landed as a change (`backlog_document`),
  rendered only and never read back; `backlog_rejections` of one render
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

- **Each is a fresh run on a brief the engine assembles,** ending with
  its result or files the engine pushes. None waits, polls or watches a
  run; each runs under its charter's model and its template's effort.
- **Measured, not capped:** peak context, turns and tokens are recorded
  against the person's target (`context_target`); a timeout
  (`agent_timeout`, per model) is the only stop.
- **Who starts them:** the chat starts goals; a goal's agent starts its
  planner, a small fix's change, a fresh resolution; the engine starts
  producers, examiners, reviewers, resolvers, the answerer, revisers,
  arbiters, scaffolds, the structure review, the wrap-up, the triager,
  walkers and the forge's reply. Which run on workers and which on the
  engine's machine is a setting (`agent_hosts`). One runs per subject at
  a time.

### 10.2 Review (LLM; the counts are code)

- **What is reviewed:** the outputs of planning and judgement tasks (the
  planner's plan, the answerer's answers, the structure review's
  changes, a scaffold's and the wrap-up's branches), never a producer's.
- **With `review` on,** such an output gets a read-only reviewer:
  approve, or refuse with remarks, in a few lines its charter bounds.
  Refused, the remarks go once to a reviser on the same output; refused
  again (or the reviser failed, or `review_faults` reviews faulted), it
  is held for an **arbiter**, which says take (land, for a branch),
  revise (once more, with guidance) or the person's (budgets, machines,
  what is hard to reverse, a change of direction). An arbiter gets two
  attempts; a second that fails holds the subject for the person.
- **A landing's rejection is a review's refusal,** whatever `review`
  says: a rejection of a scaffold's or the wrap-up's branch (its merge
  into the goal branch, the queue) counts: revised (a resolution, 6.1)
  when there has been no revision since its last reset, else held for
  the arbiter.
- **With `review` on, a structure review** reads the digests of the
  changes landed since the last (every `review_every` landings or
  `review_after` hours) and writes consolidation changes, which start
  first, and examples for the producers' charter.
- **With `review` off,** such an output is taken as it ends.

### 10.3 The examiner (LLM; what it decides is recorded by code)

A read-only agent task that decides one thing and writes no code. One
per question, rejection, red or finding; any other end is a fallback:
the question queues, or the repair starts with the evidence alone.

- **A question** from a change with a written spec (a repair's questions
  queue at once, and their answers join its words): when the spec's
  options answer it, the answer joins the spec and the change runs
  again; when its work is pushed and accepted and what it asks lies
  outside it, **accept** finishes the change as it stands (a goal
  branch's); otherwise the question queues for the answerer.
- **A rejection:** fix (a section recorded in the store, joining the
  next run's brief) or retry (not the change's: the same tip goes back,
  once). Whether the next run is a resolution or a repair, code decides
  from the gate that refused (a merge conflict is a resolution, 6.1).
- **A red after landing, an on-path finding:** fix (a repair change), or
  not the code's (an alert).

### 10.4 The answerer (LLM; when it runs is code)

- **One task answers a goal's queued questions** in one pass: at once
  when one holds up other work, or always with `answer_at_once`;
  otherwise at `answer_at` queued or after `answer_after` minutes. jig
  routes a change's question to its requester, the goal; a fresh
  answerer instead carries only what the questions need, where the
  goal's agent may run long. Batching by count and age is a wake setting
  jig lacks (its wake policy wakes on questions or not).
- **It prefers leaving what is not the change's** to its owner or the
  final check, widening a change only when its acceptance needs it.
- **What it cannot settle** it asks the person (11.1).

### 10.5 Kept and held (code)

- **Only the change that needs an answer is kept;** its dependents wait,
  everything else runs. An answer releases it. A change past its tries,
  or failed, is held (5.1).
- **A part named by an open question** of the person's is not designed.
- **An agent task's question keeps its subject** until the person
  answers.

### 10.6 Scaffolds and the wrap-up (LLM; their frames are code)

- **A scaffold** designs a part as code (types, signatures, empty bodies
  on its branch; the part's specs recorded in the store as its tasks)
  when `scaffold` is on, started when the ready changes fall below
  `scaffold_band` times the target; a part shared by others is
  scaffolded first. The engine checks the scaffold's tip (gates and
  tests) and starts the part's changes from it before it merges; a part
  waits for its dependencies to be designed, not done.
- **Changes started from a scaffold wait for its review:** a finished
  change merges only once the scaffold has; if it was revised, the
  engine merges it into each such change once, as a resolution, outside
  its tries; a held scaffold holds its changes.
- **The wrap-up:** once a goal's parts are done (and after 8.3), one
  task does the project's duty at the end (`wrap_up`); its brief quotes
  each design the goal changed as its diff or whole, the shorter, within
  a bound. Its branch lands, and the goal ends.

## 11. The person, the chats and the forge

### 11.1 Questions and the inbox (code; the questions are LLM)

- **Only what is the person's reaches the person:** budgets, accounts,
  machines, what is hard to reverse, a change of direction. Whoever asks
  decides: a planner's question, an agent task's, an arbiter's referral,
  a triager's `user` or `brief`, a goal's agent after a hold.
- **The person's inbox,** read and answered through `temper ctl`, lists
  the open questions with what to read and how to answer; an answer is
  recorded once in the store and carried in every later brief of its
  subject.

### 11.2 Alerts (code; acting on them is LLM)

- **An alert is what no goal owns,** sent to the chat `alerts_to` names
  (by default the person's): a red no open goal owns or one left
  unfixed, a tier that cannot run, a finding its examiner called not the
  code's, a backlog document's change rejected `backlog_rejections`
  times.
- **The chat's agent acts:** it starts a goal for it, waits for it to
  clear, or asks the person. Each alert opens and clears from the
  records, or when the chat clears it with a reason (but a fault).

### 11.3 On the forge (code; the reply is LLM)

- **With `questions_on_forge` on,** one issue per open question of an
  agent task, a held review or a triager (never a plan's), labelled on
  the forge, opened, conversed and closed by a pass on a timer
  (`forge_sync`). Answering in `temper ctl` alone is enough.
- **The person may answer in a comment:** a reply task reads it with the
  question and its report, and either replies (context, options, a
  recommendation) or records the answer, and the issue closes.

## 12. Workers, workspaces and custody (code)

### 12.1 Workers and workspaces

- **The engine's machine** runs the engine, custody and the agent tasks
  `agent_hosts` puts there. **Workers** run the rest and the builds;
  which worker holds the gate lane and the tier runner is configuration.
- **A workspace** is prepared by the worker for a run and swept at its
  end of what the project names as build waste, which bounds it
  (`workspace_bound`); one whose run is gone is stale and released, the
  run lost (5.1).
- **Builds wait for disk:** in a build lane of `builds` builds at once
  per worker, or the gate lane; short of headroom, a build reclaims idle
  workspaces and waits up to `disk_wait`, then fails as transient.
- **A worker's disk is budgeted:** the cache's cap (`cache_cap`), its
  workspaces at their bound, headroom (`headroom`), and one more on the
  gate's worker.

### 12.2 Custody

- **Every account and credential is held centrally,** one entry each:
  model accounts, and the application's credentials its walks and live
  tiers use.
- **Custody is records:** added, lent (to whom, where), returned,
  checked, retired. A lent account's central copy is a placeholder until
  returned; tokens refresh, so the copy last used is the true one; bytes
  are moved and hashed, never printed.
- **Lent to workers and to runs:** each machine holds the accounts its
  role needs (`machine_accounts`); a live tier or a walk borrows for its
  run. Moving a program's sign-in between machines conflicts with jig's
  design, where a program's login is per host (jig `accounts.md` 6): an
  open question for the person.
- **Checked on a timer** (`account_check`): the account and every usage
  limit. A machine whose account passes `swap_at` of its weekly limit is
  swapped onto a free account of the same model.

## 13. What is measured and reported (code)

- **Two reports, from records alone,** each day on a daily timer
  (`reports_at`), each figure beside the day before's, sent to the chat
  `alerts_to` names: a code report per goal and one landing report.
- **The code report:** runs and results, runs in flight against run
  targets, idle gaps, time per change, questions and time to answer,
  transient minutes, disk and lane waits, each part's time to landed,
  usage and weights, stops ranked by time lost.
- **The landing report:** candidates and outcomes, rejections by gate,
  flakes at the gate, candidates landed on a receipt, ready-to-landed
  latency, bisecting, gate durations; reds opened, routed, decided and
  closed; walks, findings, triage, repairs and passes.
- **Weights** (`weights`) per kind of token and model; the ratio of the
  models' weights per goal, with the chats' own runs shown apart.
- **Status through `temper ctl`:** what is in flight, landed and merged
  today, questions waiting and their age, each goal's activity,
  overlaps, and the accounts' use and pace.

## 14. fabrig's model as configuration

### 14.1 The choices

fabrig's way of running, as fab runs it, is these choices. Values are
fab's (`fab/` at 1dc102c) unless marked; "none" means fab has no such
mechanism and works by hand (14.2).

| Choice | fabrig's value |
|---|---|
| the chat | one, a director template; `alerts_to` it |
| a draft's goal (4.4) | `landing = goal_branch`, `ready` yes, `review` off, `scaffold` off, `early_start` on |
| a small fix's goal | `landing = each` |
| fabrig's queue (7.1) | `prepare` merge, `selection` packages, `push_attempts` 1, no `refused_paths`; `gates`: the counted baselines (`<tree>/scripts/old-forms.txt`, recounted by `old_forms.py --write`), formatting, lints, the build, the `default` and `fuzzy` tests; `docs_gates` the touched trees' counted baselines; `receipt_gates` the counted baselines and formatting; `gate_set` the counted-baseline gate; `check_before_ready` on |
| fab's own repository | `landing = each`, `candidate_size` 1, `prepare` rebase, `selection` whole (its suite, 30 s, under a 120 s timeout), `push_attempts` 3, `refused_paths` the legacy kit and the person's drafts, `check_before_ready` off, `gate_reruns` 0, `gate_set` code style (no changed signature but for an interface or migration, no function over 60 lines added or grown, no banned name, docs on public items, no repeated block of 8 lines); its `agents-rebuild` goal with `review` on and `scaffold` on |
| QA | one ladder goal, `landing = each`; `backends`: ChatGPT with an owned Forgejo or GitHub, Claude Code with an owned Forgejo; `backlog_document` on (`temper/docs/design/debt/v0.1.0-ladder.md`) |
| `change_kinds` | feature, interface, migration, repair, rebase, docs |
| `tries`, `fix_tries` | 4, 2 |
| `flow` | fixed for a goal branch, adaptive for `each` |
| workers | three; run targets 6, 4 and 4, 7 slots each, `builds` 4, `cache_cap` 50 GB, `headroom` 10 GB; the third holds the gate lane and the tier runner (its seventh workspace) |
| `calm` | load per CPU under 0.7; under 5 minutes of lane and disk waits in the last 30; no refused build |
| `scaffold_band` | 1.0 times the target; 2 scaffolds at once at first |
| `interim_every`, `interim_after`, `interim_skip` | 10 merges, 45 minutes, the fuzzy tests |
| `update_after`, `update_refusals` | 30 minutes, 3 |
| `final_reds`, `queue_refusals` | 3, 3 |
| `candidate_size`, `full_every` | 8, 5 |
| `gate_reruns`, `gate_flakes` | 5, 2 |
| the tier set | the browser tiers of jig, temper and skein, temper's process tier; live: skein's |
| `answer_at_once`, `answer_at`, `answer_after` | on for a goal branch; 4, 30 minutes |
| `triage_at`, `triage_after`, `backlog_rejections` | 4, 30 minutes, 2 |
| `review_every`, `review_after`, `review_faults` | 10, 4 hours, 3 |
| `context_target`, `agent_timeout` | about 600k (the user, 10-30); 60 minutes for Claude tasks, none for Codex |
| `capacity_waits` | 120, 300 and 900 seconds, then a failure |
| models | producers Codex (gpt-6.1-sol) at high; examiners and reviewers Codex at medium; the others Claude (Opus) at high, the answerer, arbiter, triager and reply at medium |
| `agent_hosts` | Codex tasks on workers, Claude tasks on the engine's machine |
| `producer_allowances` | the arms, fields and literals the compiler requires because of a type the change altered, anywhere in its tree; a scoped lint exception on an item it wrote |
| `wrap_up` | the status files of the designs the goal changed brought current |
| `machine_accounts` | one Codex and one Claude account per machine (lent by hand) |
| `walk_boxes`, `walk_cap` | none (designed: a QA worker's `boxes`); none enforced (3 hours designed, `fab:qa.md` 2) |
| `swap_at` | none (by hand) |
| `questions_on_forge` | on |
| `forge_sync`, `account_check`, `reports_at` | 2 minutes, 30 minutes (timers installed by hand); none (by hand) |
| `workspace_bound`, `disk_wait` | 25 GB, 20 minutes |
| `weights` | input 1, cache write 1.25, cache read 0.1; output 5 for Claude, 8 for Codex |

### 14.2 fab's gaps

Where fab today works by hand, has an agent do a mechanism, has code
make a judgement, or keeps temper's data in the repository, the model
does not copy it:

- **temper's data in the repository:** fab keeps its plans, specs,
  answers, repair sections and rung briefs as files in the project's
  repository (`docs/plan/<draft>/plan.toml`, `tasks/*.md`, landed or
  merged on branches), and its queue's work list as
  `ready/<draft>/<task>` refs; temper keeps them in its store.
- **By hand:** the person or a session starts the cut (no chat creates a
  goal); answers to a plan's questions written into plan.toml; ready
  refs pushed by hand, which no plan owns (their rejections and reds
  become signals); the hand merge of main after a stop; walks run by a
  session, with its own build and its own account loans (`fab qa walk`,
  `fab qa result`); a rung's brief edited after a triage `brief`
  question; stale slot claims released by hand (designed in
  `fab:machines.md` 2, not built); accounts lent to machines and swapped
  (`fab logins lend`, `swap`); the daily reports (`fab report daily`);
  the timers of the forge sync and the account checks, installed by
  hand; signals read and cleared by a session; a run lost with its
  builder, found and run again by hand.
- **An agent doing a mechanism:** the cut pushes its pieces and runs
  `fab wake cut`; the implementer pushes its branch, runs the check
  before ready (`fab precheck`) and merges its early-start dependencies
  again before pushing; rebase runs reset, rebase and push themselves
  (`run._rebase_brief`), and fab rebases where temper merges; catch-up
  repairs merge main themselves; the amender rebases and force-pushes a
  refused branch; a run's transient class is read from the agent's own
  `Infra:` line.
- **Code doing a judgement:** a goal's stop is a note, not a question to
  anyone (`fab:integration.md` 3 designs the question); a red's repair
  goes to the newest suspect's plan whatever owner its examiner names.
- **fab's engine:** a tick, woken by events into rounds, with a backstop
  timer (10 minutes) for lost wakes and up to 5 self-wakes; temper's
  engine decides each event as it comes (section 3).
- **Faults:** an integration plan's retire branch merges into a trunk
  that has already landed, so it never reaches main; the rebase onto an
  amended skeleton repeats without end in an integration plan
  (`call_review.task_rebase` looks for a base of main); the queue never
  takes a ladder's backlog branch (`queue.candidates` takes ready names
  of two segments only).

## 15. Future ideas

**Not to be carried out now.** These are in the previous edition or in
fab's designs, fab does not do them today, and they may be needed later.
Each waits for the numbers or the person's word that asks for it.

- **From the person's execution model:** scaffolds with a test
  vocabulary by default; surveys as a stage of their own before the
  planner and each scaffold; test-writer and acceptance runs; a
  self-critique before ready; probe runs and the decision task that
  turns a probe's report into design; a slice complete when its story
  test passes on main; a pull buffer of ready changes driving design.
- **Landing:** receipts by key per package (the previous edition's keys,
  gate classes and audits); receipts reused for candidates of several;
  speculative candidates (a train) or a second gate lane; the candidate
  size from measures; budgets that turn the gate red; evidence bound to
  trees; CPU per tree in the landing report.
- **After landing:** bisecting to a culprit when examiners name the
  wrong owner; a second repair of an unfixed red; QA passes on landings
  marked for QA.
- **The goal branch:** a build of overlapping goal branches' merge;
  landing at slice boundaries; overlaps that reorder work; a light lint
  after each merge.
- **QA:** a passed rung's story joining the live tier, with its fake
  twin in the worlds; findings matched by a normalised key; findings of
  other finders; findings projected as forge issues.
- **Agent tasks:** a cap on context or turns once the person sets one; a
  question that holds a direct part's changes; dependencies across
  goals.
- **Workers:** added and retired by demand.
- **Landings across repositories,** and a forge's native merge queue as
  an accelerator.
- **The web UI:** the chat, the inbox and a public status page in a
  browser; a track of its own (the user, 11-01). Here everything goes
  through `temper ctl`.
