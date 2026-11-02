# The reference workflow: code production

Draft, 2026-11-01. Nothing here is adopted. How a project develops
itself on temper, from a person's draft to a branch ready to land.
`reference-landing.md` goes on from there, and `reference-case.md`
walks both once. This is one way of using temper, reached mostly by
configuration; other workflows use the same primitives differently.
Each mechanism says whether it is built, designed or missing, and
section 13 lists what is missing, as the input for building it. The
numbers come from a real run of this workflow: 64 tasks, 185 agent runs,
6h23 from the draft to a green branch.

## 1. In one page

- **What it is:** everything between a person's draft and a goal's
  branch ready to land: a designer cuts the draft into parts, designers
  turn each part into tasks while implementers fill them, and every
  finished task lands into the goal's own branch.
- **The pipeline grows:** a small first intake starts as quickly as
  possible and implementers start at once; while they build, design runs
  just ahead of them, part by part; the work grows as parallel as the
  work and the capacity allow, and it never stops while work remains
  (section 4).
- **LLMs judge, code runs the mechanisms** (section 2). An LLM designs,
  writes code, answers a question, resolves a conflict and reviews.
  Starting tasks in order, landing, rerunning a failure, making a repair
  for a red, keeping up with main and stopping at a bound are code, and
  wake no LLM.
- **Called, not resident:** no designer stays alive between jobs. Each
  is a coordinator that parks at once and starts fresh from its brief
  when something needs its judgement (jig's `domain/engine.md`, 7.4).
- **The goal's tree is the plan** (section 3): a goal, its parts as
  delegates under a designer's charter, one change per task, and one
  change that lands the goal's branch once every part has ended.
  Nothing is kept beside the tree.
- **One landing mechanism at two levels:** a task lands into the goal's
  branch and that branch lands into main by the same turn; the rule on
  the goal's branch has no gates, so its landings prove nothing and take
  seconds (`reference-landing.md`, section 3).
- **A question parks one task** (section 7): the change is held, the
  hold escalates to its part's designer, and the answer is an amendment
  and a release. A change still asks no one.
- **A red on the branch is the branch's** (section 8): a check follows
  the branch, and a red makes a repair by code.
- **Measured, not felt** (section 11): the numbers come from what the
  store records.

## 2. Judgement and mechanism

Everything that can be decided from facts is code; an LLM is called
where the decision needs reading and judging. The test for each step: a
wrong answer from a mechanism is a bug to fix once, and a step repeated
the same way every time is a mechanism whoever does it today.

| Step | Code | An LLM |
|---|---|---|
| Intake | makes the goal from a draft | judges the plan's shape: its mode, its parts, whether it needs a branch of its own |
| Design | starts a part when what it needs has landed; delivers and lands what a designer writes | designs the part as tasks; writes specs, and the shapes that cross tasks as code |
| Starting tasks | dependencies, kept, early starts, capacity | none |
| Implementing | saves work, pushes, runs the repository's checks | writes the code and its tests |
| Questions | holds the change, escalates, makes the next producer | answers, in a spec amendment |
| Waiting for a dependency's work | keeps the change and starts it again | none |
| Landing into the branch | prepares, fast-forwards, deletes the branch | none |
| A conflict | settles what the project's tooling knows; makes the resolution | resolves the rest, keeping both sides' intent |
| The branch's check | runs it on its cadence, reruns failures, tells new from inherited | none |
| A red | makes one repair, with the failing output and the list of what to leave alone | fixes it |
| Keeping up with main | merges main in on its cadence | resolves a conflict |
| A bound passed | holds, and escalates | judges what next |

## 3. The tree

```
a person
└─ G  the goal                       coordinator, a designer's charter; tracked
   ├─ c0  shared contracts           change into B, handed G's run branch
   ├─ P1  a part                     coordinator, a designer's charter
   │  ├─ c1…cn  a task               change into B, dependencies between them
   │  │    ├─ a producer             agent, a coding charter
   │  │    └─ a resolution           agent, a coding charter
   │  └─ …
   ├─ P2…Pm  the other parts
   └─ L   the landing change         change of B into main; needs c0, P1…Pm
      └─ r1…  a repair, a resolution  change into B, made by L's step
```

- **G is a goal,** as `forge.md`, 15.1's T1 is: tracked, with a budget,
  an issue that shows the plan, and a priority. It holds the judgement
  that spans parts: the plan's shape, a question a part passes up, a
  hold on L, and the goal's last report. It wakes on an escalation, and
  on the last of its delegates done.
- **A part** is one concept, and small enough for one designer to hold:
  2 to 20 tasks, by the measured run. It is a delegate of G under the
  same charter. It holds the judgement of its tasks: their specs, their
  questions, what is left when they end.
- **A task** is one module or concept that one implementer finishes in
  one run, production code and its tests together: a change (`forge.md`,
  section 8) whose landing branch is B. Longer runs are better than more
  round trips.
- **B, the goal's branch,** is `forge.md`, 11.1's: made at main's tip by
  the change that lands it, written only by landings into it, deleted
  when it lands. Its name is a new family beside `forge.md`, section
  11's, `temper/<root>/g<task>`, so a landing rule's pattern can cover
  every goal's branch.
- **L** is the change that lands B. It is made in G's first batch, kept
  by its dependencies from the start, and it needs no producer: its
  head is what has landed into B. While kept it steps as any procedure
  does (jig's `domain/tasks.md`, 5.3): it keeps B up with main and makes
  B's repairs.
- **L needs the parts, not the tasks.** A task ends after its delegates
  (jig's `domain/tasks.md`, 5.6), so a dependency on a part waits for
  its whole subtree, however it grows, with no amendment; and every
  part exists from G's first batch, so L cannot go ready between one
  batch of tasks and the next. A part G adds later is an amendment of
  L's dependencies. A part that fails holds L (jig's `domain/tasks.md`,
  5.5), and G decides.
- **A plan with one part** is a tree with one P. A small fix has no tree:
  section 10's Fix mode is one change into main.

Dependencies say the order of work. One writer per crate is a chain of
`needs`; no lease or lock is asked for. What must land together is the
goal: nothing reaches main before B does.

## 4. The growing pipeline

### 4.1 Intake

- **The goal is made from a draft** by a person, in a chat with `goal`
  or through a client: the draft's path at its commit is its spec, with
  anything the person adds.
- **G's first run is the thin cut,** about 20 minutes: it reads the
  draft, chooses the plan's shape and names the parts, their
  dependencies and the shared contracts: the shapes that cross parts,
  fixed before anyone builds to them. It writes no task specs.
- **The contracts land first.** G's run writes them, as code where they
  can be and as files where not, delivers them to its own run branch and
  hands that branch to c0 (`forge.md`, 8.1 and 11; jig's
  `domain/tasks.md`, 6.4). Structural decisions are a designer's, and
  code carries them to implementers better than prose. Only their
  readers wait for them: parts that touch no contract start beside c0.
- **A designer writes only what it delivers.** `agent.md`, 4.3's rail
  holds: `modify` comes with mid-run delivery, so no run leaves edits
  that nothing delivers. What it delivers lands through a change it
  hands its branch to, under the rule of the branch it lands into.
- **G's first batch** is L, c0 and every part, whole or nothing (jig's
  `domain/tasks.md`, section 4). A person's question that no work waits
  for goes in G's report, and nothing parks.

### 4.2 Design runs just ahead

- **A part's first run** designs its tasks and delegates them batch by
  batch as they hold together, the first batch at once. One run may
  delegate many times (jig's `domain/engine.md`, 7.3), and tasks delegated
  early are running while it writes the next: in the measured run the
  first implementer started at T+0:27, and the plan was all delegated by
  T+0:54, in nine pushes.
- **Surveys are the designer's to judge.** Where a part needs one (what
  reads what changes, the primitives in reach), it is a sub-agent of the
  designer's run, read-only.
- **A released spec is settled:** only an answer changes it.
- **A task's spec settles up front** what would otherwise be a question:
  the paths it owns, the readers it may change, the likely options with
  their defaults, and its acceptance. An implementer may do without
  asking what the compiler requires of a type the task changed, in any
  file of its tree.
- **Pulled, later:** a part starts when the ready tasks run low, so that
  it is designed with what earlier parts taught (section 14). In the
  first version every part starts when its dependencies allow.

### 4.3 Growth

| Measured | |
|---|---|
| first implementer | T+0:27; the target is about an hour after the draft |
| peak | 14 implementers, T+1:30 to T+2:00 |
| tasks landed into B per half hour | 5, 13, 12, 12, 8, 4, 5, 3, 4, 1, 2, 3 |
| green at the end | T+6:23, against a critical path of 18 hours as estimated |

Capacity decides how many run; nothing else holds them. A pause of a
designer or a question parks only the task that needs the answer. Where
runs happen, and how many at once, is out of this draft's scope.

### 4.4 What keeps the state

Nothing lives only in a session.

- **The tree** holds the plan: each task's spec and revisions, its
  dependencies, its activity. The `Plan` section gives a designer's
  fresh run its delegates and what came since (jig's
  `domain/engine.md`, section 9).
- **Specs are task words,** kept in the store, not files. An answer is
  an amendment, not a commit: more than two fifths of the measured run's
  merges into its branch only carried specs and answers.
- **Shared notes are files on B:** c0's contracts, and what a part's
  designer adds the same way. Implementers read them in their checkout,
  as specs cite them, and a designer reads them at B's tip. They land
  with the code.
- **Notes for the next designer** are jig's notes in the goal's scope
  (jig's `domain/engine.md`, section 10), read by a designer's fresh run.

## 5. A task's way

Each step is code unless it says otherwise.

```
kept ─► producing ─► opening ─► ready ─► landing into B ─► landed
  ▲         │  ▲                             │
  │         │  └── held: a question          └── a conflict ─► resolving ─► ready
  └─────────┘      (section 7)
    started early, its dependency unfinished
```

1. **Made by its part's designer,** in a batch, with its spec, its
   `needs`, B as its landing branch, its charter and its LLM choice.
2. **With nothing pending,** its producer starts from B's tip
   (`forge.md`, 8.2, step 1), so it sees everything landed so far.
3. **With a dependency pending,** the change starts kept (jig's
   `domain/connectors.md`, 6.1) and may start early: once each pending
   dependency has pushed a head and has no pending dependency of its
   own, its producer starts from B's tip with those heads merged in, and
   its brief names them. 54 of the measured run's 121 implementer runs
   started so. A conflict there starts nothing: it waits as if not early.
4. **A change starts early once.** A producer started early that ends
   with a question, or fails because what it needs is unfinished, asks
   no one: the change waits for its dependencies and then runs again.
   Most of the judges' answers in the measured run said no more than
   "run again once it lands"; this needs no LLM.
5. **Producing** ends with a pushed head, its checks passed: smith's
   `deliver`, then `finish` with a change, its title and its body
   (`agent.md`, 4.4). An implementer commits and pushes before it stops
   for any reason, so the next producer starts from its work.
6. **What is not its own** it leaves. When its acceptance holds and its
   work is pushed but something outside it stays red or open, it
   finishes with a list of what is left in its result's body, and the
   change lands. The owning task, the branch's check or its part's
   designer takes the rest.
7. **Opening:** a pull request into B, keyed by its branches.
8. **Ready** once its head is pushed and it is no longer kept: a kept
   change is coded beside its dependency and lands after it
   (`forge.md`, 8.2, step 7).
9. **Landing into B** is a landing of one by the project's turn, with
   nothing to prove (`reference-landing.md`, section 3). Dependents
   start from the new tip.

## 6. Conflicts

- **Mechanical first.** What a conflict's rules decide is settled by
  the project's tooling, with no LLM: a counted file that many tasks
  lower is recounted, as 17 of 24 merges of one measured run needed.
  This is the project's `prepare` on a job (`landing.md`, 5.3;
  `../protocol/tooling.md`).
- **The rest is a resolution** (`forge.md`, 8.4): the change, under
  its own hold, in a workspace with B's tip merged into its branch and
  the files in conflict marked, keeping both sides' intent. Its merged
  head contains B's tip and lands as it is. In the measured run this
  took a judgement run and a rebase run each; here it is one resolution.

## 7. Questions

- **Asked as a result.** An implementer stops when something beyond its
  spec needs deciding: it pushes what it has and ends with a typed
  result saying it needs a decision, the question in its words, with
  its options and what it recommends.
- **The change is held** (jig's `domain/tasks.md`, 5.5: "its procedure
  says it needs a decision"), as a producer that fails for good holds
  it today (`forge.md`, 8.4). No agent of a change messages anyone, so
  `forge.md`, 8.1 stands: a change asks no one.
- **The hold escalates** to the nearest task above that may decide,
  skipping procedures: the part's designer, which an escalation always
  wakes (jig's `domain/tasks.md`, 8.3). Only that task parks; the rest
  of the tree runs on. Several waiting at once are read by one run.
- **A fresh designer run answers,** in minutes, from its brief: the
  question, the spec, the plan, its notes (jig's `domain/engine.md`,
  section 9). 35 runs of 1 to 4.5 minutes answered 40 questions in the
  measured run. Its answer is one or more of:
  - an **amendment** of the spec with the answer, then a **release**
    (jig's `domain/tasks.md`, section 7);
  - a **new task** and a dependency on it, then a release: the plan
    grows in place, and the change is kept until the new task lands;
  - a **cancel;**
  - a **pass up** to G, which may pass it to its person.
- **Its rule of thumb:** leave what is another task's to its owner or to
  the branch's check, and widen a task only when its own acceptance
  cannot hold otherwise. In a run that answered nearly every question by
  widening, each became a longer rerun.
- **Released,** the change decides afresh: a fresh producer starts from
  its pushed head, B's tip merged in as `forge.md`, 8.2 does for a repair,
  its brief carrying the amended spec and its earlier attempts.
- **The person hears only what is theirs:** budgets, sign-ins and what
  is hard to reverse, when G passes it up and they answer in their
  inbox.
- **Bounded:** a change whose runs reach their bound without landing
  (four, measured) is held the same way, for judgement.

45 of the measured run's 121 implementer runs ended in a question: scope
("may this task edit X?"), a missing prerequisite, a dependency not yet
published, an acceptance that could not hold. Sections 4.2 and 5 remove
the third kind and most of the first by mechanism; the metric is
section 11's.

## 8. Along the way

### 8.1 The branch's check

Reds collect if nothing looks: a run with no check along the way ended
with 101 of 24,207 tests red.

- **A tier follows B** (`after-landing.md`, section 3): the repository's
  check, less its slowest profile, as a dispatch-only workflow on a
  branch of temper's own. Its cadence is a setting: after 10 landings
  into B, or 45 minutes with one landing since. Not while a repair is
  open.
- **A red run reruns its failed tests once,** alone. Those that pass are
  a flake. Failures an open repair already covers are inherited, and
  make nothing new. B's tier blames no landing: it has one goal.
- **A red makes a repair, by code.** L's step makes one change into B,
  `forge.md`, section 9's shape: its words open with what it repairs, its
  brief carries the failing output, says the red is its own whatever
  caused it, and lists the tasks not yet landed with their paths, to
  leave what they own. It starts from B's tip and lands like any task.
  A question beyond that escalates to G, past L.
- **Reds are ordinary.** 8 of the measured run's 9 checks were red, each
  fixed by one repair, and none stopped the plan.

### 8.2 Keeping up with main

Main moves while a goal runs. Taken at the end, one such merge was 23
conflicts, 4 repairs and 56 minutes; taken as they come they are small.

- **L updates B on a cadence** while its dependencies are pending:
  after 30 minutes with main moved, before the next check, and not while
  a repair is open. The update is mechanical, by a merge, as `forge.md`,
  section 10's is under a queue, and it waits for no dependency.
- **Clean,** the next check reads what main brought. A semantic conflict
  shows as a red, and is a repair.
- **A conflict** is settled by the project's tooling, or makes a
  resolution as a change into B, starting from a merge of main into B's
  tip, so that landings into B never wait for an agent. L then checks
  that B contains the tip it asked for.
- **A bound:** the third refusal since a clean update holds L, and G
  judges.

## 9. Ready

- **A part ends** when its designer, woken by the last of its delegates
  or told nothing more will come (jig's `domain/tasks.md`, 5.2), reads
  what its tasks left and finishes with a report, or delegates more.
  That last read is judgement: it is where a plan grows at the end.
- **When every part has ended, L is no longer kept.** That moment is
  the boundary: no LLM marks it, and B is ready to land. Nothing has
  gated it yet.
- **After that** B is written only by L's own delegates. Back-and-forth
  is `reference-landing.md`, section 6: a repair from the gate, or a new
  part G adds, which keeps L again.

## 10. Configuration

### 10.1 Charters

| Role | Task | Charter | LLM choice | Tools |
|---|---|---|---|---|
| Designer | G and each part | a coordinator's: starts fresh, parks at once | Opus 5.5 at xhigh; sub-agents, for surveys, Opus 5.5 at high | reads; a checkout at B with the shell, written only with mid-run delivery to its run branch; `delegate`, `amend`, `cancel`, `release`, `decide_escalation`, `note`, `message` |
| Implementer | a change's producer, resolution, repair | the coding charter (charter 2) | the coding model at high | the workspace's; no messages |
| Reviewer | a gate, when on | the review charter (charter 4) | by lens | reads |

- **A designer is a task, a survey a sub-agent.** A run's sub-agents take
  one effort per model (smith's `protocol/charter.md`), so what must
  run at xhigh and delegate is a part, and a survey is what the run
  opens for itself.
- **Its instructions** carry what the workflow teaches: the task's size,
  writing specs that settle scope, one writer per crate, leaving what
  is another's.
- **The documents it reads:** the repository's guide, and a charter's
  documents (jig's `domain/engine.md`, 17.9).

### 10.2 Landing rules and the check

| Setting | On main | On a goal's branch |
|---|---|---|
| landing rule's pattern | `main`, the stock's rule narrowed to it | `temper/*/g*` |
| gates | the full gate | none |
| tiers | the slow tiers | the check, on its cadence |
| keep up | none | every 30 minutes |
| repairs | `reference-landing.md`, section 4 | one at a time |

### 10.3 Limits

The stock's limits fit a chat and two changes. This workflow's profile
raises what it measures: live tasks; tasks per tree (several hundred:
each producer and resolution is a task, a landing being a root of its
own); delegates per task, batch size and dependencies per task; live
landings (`forge.landings`, 2 in the stock); a spec's bytes and the brief
(the measured specs ran to 21 KB against a brief of 14,592 bytes), with
its plan and notes sections; a result's bytes, 512 in the stock, which a
list of what is left outgrows; and a designer's run budget (the longest
run took 53 minutes). All are settings but one: a change's 6 tasks and
depth of 2 are fixed in the code, and each question costs a producer
(CP-1).

### 10.4 What turns on later

The pipeline's other stages are options of the same mechanism, off in
the first version, and none needs a new primitive beyond section 13's.

| Stage | What it is here |
|---|---|
| Skeleton | a part's first change, written by its designer as c0 is: modules, types, signatures and test vocabulary as compiling code, built in its own run, handed to a change and landed alone; its tasks need it |
| A contract's review | a review gate on c0 |
| Self-critique | a review gate with a lens, blocking, on each task: what it says is a repair |
| Test-writer | a task of its own, beside the task that implements |
| API baseline | a check gate comparing a task's public items with its skeleton's |
| Structure review | a designer's delegate on a cadence (a wake policy's batching: jig's `domain/tasks.md`, 8.3), whose findings become tasks at the head of the plan |
| Slices | stories named in the plan; a story's test on B is a tier |
| QA walk | a delegate after a slice, reporting findings as tasks |
| Probe | a read-only part that reports; G decides |
| Migration | a part whose tasks are mechanical, under a ratchet in the gate |
| Fix | a goal with one change into main, no branch, no designer: `reference-landing.md`, section 5 |
| Modes | G's choice at intake: the same tree, with these on or off |

## 11. Measured

The numbers come from records, never by hand, per goal and per part:

| Metric | Read from |
|---|---|
| start-up: to first task, to first landing | jig's activity; the landing log |
| flow: implementers in flight, tasks landed per hour, idle gaps with their cause | jig's activity; the landing log |
| task size: lines per landing, runs per task | the forge; jig's runs |
| questions: per run, by class, time to answer, reruns they cost | holds and amendments |
| repairs: checks red, repairs per check, conflicts per landing | tiers; changes |
| cost: each role's spend and its ratio, each run's peak context | jig's accounting |

A metric off its target is a signal with a first action: a question
rate above target goes to the spec's template; flat flow below capacity
goes to what the parts wait for. Targets are the person's, set from the
first numbers. A stop seen twice gets a tool, not another rule, and
`Taught:` trailers carry the lesson (`after-landing.md`, section 8).

## 12. What it asks of jig

Little, as expected. Tasks, batches, dependencies, kept, holds,
escalations, amendments, wake policies, fresh runs, briefs and notes are
jig's and built, but for the rows section 13 marks. Everything specific
to branches, landing and tiers is temper's, and jig names none of it.

## 13. What is missing

Each row is what stands between this draft and running it. *Mechanism*
is code to build, *setting* is configuration to add.

| # | Owner | Kind | What | Where |
|---|---|---|---|---|
| CP-1 | temper | mechanism | A typed end "needs a decision" for a producer: no tries spent, the question in the hold's words; a release with a head pushed makes a fresh producer at that head, its base taken first, which `forge.md`, 8.2 says of repairs alone; a bound on a change's producers, and its 6 tasks and depth of 2 as a setting; a hold's words reaching whoever its escalation reaches, since today they go to the immediate requester and a task past a procedure reads only that a decision is needed | `forge.md`, 8.1, 8.2, 8.4; `agent.md`, 4.3.1, 4.4 |
| CP-2 | temper | mechanism | A goal's branch: its name's family; a change with no producer and no handed branch, which creates B at its first step and opens its pull request once B differs from main; a change into a branch not yet there waits for it; a landing into B is not drift on L | `forge.md`, 8.1, 8.5, 11, 11.1 |
| CP-3 | temper | mechanism | The code lands on a change's base and proves no gates at once, so what is left is `landing.md`'s words, which say main, and one bound: every prepare counts toward `updates` and holds at three, so a busy branch would wake a designer for going stale. A re-prepare from a stale base counts toward nothing | `landing.md`, sections 3 to 8 |
| CP-4 | temper | setting | Gates and tiers per pattern of landing branches; kind 5 holds them per repository. The stock's rule covers everywhere and a project's rule never replaces it, so it narrows to `main` | `landing.md`, 13.1; `after-landing.md`, section 11; `forge.md`, 8.7 |
| CP-5 | jig, temper | mechanism | The change procedure started kept: designed in jig, listed as not yet in temper's status | jig's `domain/connectors.md`, 6.1; `landing.md`, 4.1 |
| CP-6 | temper | mechanism | The early start: when it may start, once only, the wait after an unfinished dependency, and a workspace start of a branch with heads merged in | `worker.md`, 4.1, section 5 |
| CP-7 | temper | mechanism | Tiers themselves, of which only `Check::Tier` exists, and on a goal's branch: a cadence, no blame, a red that L's step turns into one repair change. A change's authority names no procedure, and a branch's repair is refused for any requester but the deployment; L needs both | `after-landing.md`, sections 3, 5, 7; `forge.md`, section 9 |
| CP-8 | temper | mechanism | Keeping up: an update under the turn, where `forge.md`, section 10 applies to the queue alone, on a cadence, against its "never only because its base moved", and while kept; a conflict's resolution as a change into B, the merge in progress built for a change's own branch; the check that the tip came in | `forge.md`, 8.4, 10 |
| CP-9 | jig, temper | mechanism | Job tasks, and the project's `prepare` on them, for a landing and for L's update, which is no landing: the only way a repository-specific rule runs without an LLM. Until then a counted file's conflict costs a resolution | jig's `domain/tasks.md`, 5.3.1; `hosts.md`, 6.9; `worker.md`, 4.4; `landing.md`, 5.3 |
| CP-10 | temper | mechanism | Delegating changes: `change` takes words alone, into the default branch, one at a time; `delegate` of the change procedure takes a base, a batch and dependencies, but no charter or LLM choice | `agent.md`, 4.3.1 |
| CP-11 | temper | setting | A designer's charter in the stock: its tools hold no control, decide, subscribe or note today; `goal` takes its caller's charter, gives no notes and, from a tracked run, starts advisors alone; a designer's checkout at B, since a run outside a change starts at the default branch | `agent.md`, 4.3.2, section 8; `worker.md`, 4.1; `../protocol/configuration.md` |
| CP-12 | temper | setting | A profile for this workflow's limits (10.3) | `../protocol/configuration.md` |
| CP-13 | temper | mechanism | The standing report (section 11): no view gathers it | `client/tools.md` |
| CP-14 | temper | mechanism | A designer that writes: mid-run delivery to its own run branch, with `modify`, as a chat's small fix has. A change already takes a handed branch; what is missing is a goal's or a part's run that may write, since the rail lets a goal's runs inspect alone | `agent.md`, 4.3; `forge.md`, 8.1 |

## 14. Later

- **The pull buffer:** a part starts when ready tasks fall below a band,
  by code. Today's wake vocabulary has batching by count, not a band.
- **Questions that resume:** a producer asking in the question form and
  kept by it (jig's `domain/tasks.md`, 5.7), resuming its transcript,
  with the question routed past its change. It keeps more context for
  more mechanism.
- **Landing a slice on main** before its goal ends.
- **Several drafts seeing each other's overlaps:** a landing that
  overlaps B is news already (`forge.md`, 7.3); G keeps it from waking.
- **Comparing models and efforts** per role by section 11's numbers.

## 15. Open questions

- **The tier's cadence and the update's:** 10 landings, 45 and 30
  minutes are what one run used.
- **A part's size** and whether a one-part plan keeps its P.
- **Who answers a repair's question:** G here, past L; a part's designer
  may know more.
