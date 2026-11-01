# The code pipeline

Draft, 2026-11-01. Nothing here is adopted until the user approves it.
How work goes from the person's word to a branch ready to land, written
as jig's primitives and temper's configuration and mechanisms on them.
What follows ready is `landing-pipeline.md`. Together they replace
`landing-model.md` revision 5. `reference-case.md` is one story that
runs both, step by step, and lists what jig and temper lack for it.

The source is the code pipeline fab runs today (the agents repository,
`docs/design/` and `fab/` at 1dc102c); section 5 is fab's way as
configuration, section 6 where fab departs from this design. Machines,
slots and sign-ins are left out for now. Unqualified, `forge.md`,
`landing.md` and `agent.md` are temper's `../domain/` documents; jig's
are named as jig's.

## 1. In one page

- **jig's primitives carry it:** a task (spec, result contract,
  requester, dependencies, executor, wake policy), made in batches; an
  executor (an agent under a charter, a procedure, a job, a person);
  messages, the inbox and wakes; authority, proposals and escalations;
  connectors (jig `core.md` 3). Nothing here needs a new primitive; a
  few knobs are missing (section 7).
- **temper adds configuration and its forge connector:** charters,
  templates and settings for each role, and the forge's procedures and
  records: the change procedure, a goal's branch, updates and
  resolutions, repairs, checks (`forge.md`).
- **An LLM for judgement, code for mechanisms:** every agent task
  judges; every procedure and job is mechanics. Each section says which.
- **The shape:** the person talks to a chat; the chat starts a goal; the
  goal's agent plans and delegates changes in batches; each change's
  procedure runs a producer, answers what it can, and lands the change
  into the goal's branch; the goal's branch is checked, kept up with
  main, repaired, and made ready (`forge.md` 15.1 is this, in small).

## 2. Intake (LLM)

- **The person talks to a chat** through `temper ctl`: jig's chat, an
  agent task made from a template (jig `people.md` 7) whose charter the
  person chose: a director, or a department such as QA.
- **The chat starts a goal** with temper's `goal` tool (`agent.md`
  4.3.2): a tracked agent task (jig `core.md` 6.2) whose spec names the
  draft or the request ("fix this"), and whose settings come from the
  chat's template (section 5).
- **A small request** may need no goal: the chat makes one change
  (`forge.md` 15.1).

## 3. A goal, as tasks

### 3.1 Planning (LLM; the batches are jig's)

- **The goal's agent plans,** in a run under a planning charter: parts,
  slices (stories that run end to end), the person's questions, and each
  change's spec (words, and typed parameters: what it owns, its kind,
  its options with defaults) with its dependencies (jig `tasks.md` 3).
  It may delegate read-only surveys and design help first.
- **It delegates the plan in batches** (jig `tasks.md` 4): the changes
  that depend on nothing first, then each batch that holds together,
  while it plans the rest. Each batch's tasks start at once; the plan is
  the goal's subtree (jig `core.md` 7.1). Nothing of it is written to a
  repository.
- **Questions for the person** are person tasks (jig `people.md` 8); the
  parts that wait for an answer depend on them.
- **Scaffolds:** a part may first get a change whose producer writes its
  types and empty bodies, its other changes depending on it, started
  beside it before it lands (jig `connectors.md` 6.1, a procedure that
  starts kept beside its dependency).

### 3.2 A change (code; its producer is LLM)

- **The change procedure** (`forge.md` 8) takes each change from
  produced to landed in its landing branch: here the goal's branch
  (3.4). It delegates the producer, an agent task in a workspace on a
  worker, under the producer's charter (model, effort, tools, what it
  may edit beyond its paths).
- **The producer** edits, tests and commits; the procedure pushes. Its
  run finishes, parks on a question (the change is kept, jig `tasks.md`
  5.7), or fails. A failure's class (transient, the agent's own, lost)
  is code's, from the run and the tools, and the tries per class are
  jig's (jig `tasks.md` 5.5). Past them the change is held and escalated
  to the goal.
- **Before it is ready,** the procedure runs the change's own checks
  (`landing-pipeline.md` 2).

### 3.3 Questions and answers (LLM; routing is jig's)

- **A change's question goes to its requester,** the goal (jig
  `tasks.md` 5.7), whose agent answers it: from the spec's options when
  they settle it, else by judgement, preferring to leave what is not the
  change's to its owner. What is the person's (budgets, accounts, what
  is hard to reverse, a change of direction) the goal asks the person,
  in their inbox.
- **When the goal wakes for questions** is its wake policy (jig
  `tasks.md` 8.3): at once when one holds up other work, else in a batch
  by count or age (a knob jig lacks: section 7).
- **A cheaper first pass** is configuration: the goal may delegate a
  verdict task under a cheap charter, which answers from the options or
  passes the question on.

### 3.4 The goal's branch (code; conflicts are LLM)

- **A goal may land through a branch of its own** (`forge.md` 11.1): its
  changes land into it; it is a change of its own, landed into main by
  the change procedure once the goal's changes have (the landing
  pipeline). Or each change lands into main: a goal setting (section 5).
- **Dependents start from it:** a change that depends on another starts
  once that one has landed into the goal's branch (jig dependencies), or
  earlier, beside it (3.1's started kept, `early_start`).
- **A conflict** landing into the goal's branch is a resolution
  (`forge.md` 8.4): the workspace starts from a merge in progress, an
  agent settles the conflicting files with no git writes of its own, the
  procedure commits. A conflict the project's tooling knows (a counted
  baseline, recounted) is settled by its tooling (`landing.md` 5.3).
- **Checks of the goal's branch:** a job (jig `tasks.md` 5.3.1) runs the
  project's gates on its tip on a cadence while changes land (every
  `interim_every` landings or `interim_after` minutes), and in full once
  all have landed. A red is a repair (`forge.md` 8.4), a change of the
  goal's branch.
- **Updates:** the goal's branch takes main as it moves, by an update
  (`forge.md` 10) on a timer (`update_after`), a conflict being a
  resolution.
- **Bounds:** past `update_refusals`, `final_reds` or `queue_refusals`
  the goal's branch change is held and escalated to the goal (jig
  `tasks.md` 5.5, 9), whose agent decides: a fresh resolution, a repair,
  or a question to the person.
- **Ready:** green in full, the goal's branch change is ready, and
  `landing-pipeline.md` takes it.

### 3.5 Review (LLM; counts are code)

- **A change's review** is a gate of the change procedure under the
  reviewer's charter (`forge.md` 8.3); its remarks are a repair
  (`forge.md` 8.4).
- **A plan's review** is a verdict task the goal delegates on each batch
  before it starts (`review`).
- **Past the bound,** the change or the plan is held and escalated to
  the goal, or above it to the chat (jig `tasks.md` 9): the holder
  decides, as an arbiter would.
- **A structure review,** with `review` on: a recurring verdict task
  reads what landed since the last one and writes consolidation changes,
  which start first, and examples for the producers' charter.

### 3.6 Goals beside each other (code)

- **Overlaps are news:** a goal hears that another goal's branch touches
  the same paths, by merging both with main in memory each round; its
  planning brief lists the goals beside it (`forge.md` 7.3 has the
  landing half of this).

### 3.7 The goal's end (LLM; the wait is code)

- **The goal closes** once its changes have landed and its verdict after
  landing has come (`landing-pipeline.md` 4.3); in its closing run, its
  agent does the project's duty at the end (the designs' status files
  current) as one last change (jig `tasks.md` 5.1, closing).

## 4. The person, alerts and measures

- **The inbox** (jig `people.md` 6): questions, person tasks, results,
  read and answered in `temper ctl`.
- **Alerts are news:** what no goal owns (a red no open goal owns, a
  tier that cannot run, a finding not the code's) is news on a topic the
  chat subscribes to (jig `tasks.md` 8.4); the chat's agent decides:
  start a goal, wait, or ask the person.
- **Questions on the forge,** optionally (`questions_on_forge`): each as
  an issue, its comments read by a reply task (`forge.md` 12.1).
- **Measures:** views of the tasks (jig `engine.md` 11); a daily report
  by a recurring task (jig `tasks.md` 10): runs, results, questions and
  time to answer, idle gaps, usage and weights per goal.

## 5. fabrig's configuration

| Choice | fabrig's value |
|---|---|
| the chat | one, a director template |
| a draft's goal | lands through its own branch; `early_start` on; `review` off; no scaffolds |
| a small fix | a goal of one change, landing into main |
| charters | producers Codex (gpt-6.1-sol) at high; verdict tasks Codex at medium; the goal's agent and the chat Claude (Opus) at high, answering at medium |
| a change's tries | 4 runs; 2 for a repair of a red or a finding |
| the producer may | change, without asking, the arms, fields and literals a type it changed forces anywhere in its tree; add a scoped lint exception on its own item |
| answering | a verdict task on the options first; the goal at once for a goal branch, else at 4 questions or after 30 minutes |
| checks of the goal's branch | every 10 landings or 45 minutes, without the fuzzy tests; in full at the end |
| updates | every 30 minutes; held after 3 refusals |
| bounds | 3 reds of the final check, 3 rejections by the queue |
| an agent's context, timeout | about 600k measured (the user, 10-30); 60 minutes for Claude |
| reports | daily |

## 6. Where fab departs (fab's gaps)

fab does these by hand, by an agent where this design has code, by code
where it has an agent, or with temper's data in the repository; temper
does not copy them:

- **By hand:** the cut started by a person; answers written into
  plan.toml; hand-pushed ready refs; a hand merge of main after a stop;
  the daily reports.
- **Data in the repository:** plans, specs, answers and repair sections
  as files (`docs/plan/<draft>/plan.toml`, `tasks/*.md`).
- **An agent doing mechanics:** the cut pushes its plan and wakes the
  tick; implementers push, run the check before ready and merge their
  dependencies again; rebase runs and catch-up repairs rebase, merge and
  push themselves; a run's transient class read from its own words.
- **Code doing judgement:** a stop is a note, not a question.
- **Its own engine and roles:** a tick with a backstop timer; a separate
  cut call (here the goal's agent); an answer call (here the goal's
  agent); a foreman (here a verdict task); a settle call (here the
  holder an escalation reaches); an amender (here a repair); a merge
  rebased, not resolved; a fault: the rebase onto an amended skeleton
  repeats in an integration plan, and a retire's branch never reaches
  main.

## 7. What it needs

**jig: knobs, no new primitive.**

- A chat started from any template in `temper ctl` (today template 1).
- Application setting kinds (jig `engine.md` 17.5, designed) and a scope
  under a project, so a goal carries temper's settings.
- Waking on questions by count and age (the wake policy wakes on
  questions or not).
- Jobs (designed, not built), for checks.
- Starting a task beside a dependency (jig `connectors.md` 6.1,
  designed).
- `goal` choosing its template, `change` taking typed parameters and
  dependencies within a batch (temper's tools).

**temper: its forge connector.**

- A goal's own branch (`forge.md` 11.1, designed), with checks on a
  cadence, timed updates and its bounds.
- The overlap news between goals' branches.
- Questions on the forge (`forge.md` 12.1, designed).
- Charters and templates for the roles above; a daily report.

## 8. Future ideas

**Not to be carried out now.** Surveys as a stage of their own;
test-writers and acceptance runs; a self-critique before ready; probes
and their decisions; a slice complete when its story test passes; a pull
buffer of ready changes driving planning; machines, slots and sign-ins
(left out of this draft); the web UI, a track of its own.
