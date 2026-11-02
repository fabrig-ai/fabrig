# Notes behind the drafts

2026-11-02. The reasons behind temper's drafts (`AGENTS.md`, section 9):
the evidence they rest on and the choices made on it, each with its
reasons. Sections 1 to 6 are the reference's: `reference-code-production.md`,
`reference-landing.md` and `reference-case.md` cite its choices, N1 to
N24, by number, and its evidence by section. Unlike the drafts, these
notes name the pipeline the reference is drawn from (fab, in the agents
repository) and its runs, since they are the evidence. Where the design
still lags a draft is that draft's status, not these notes.

Unqualified, `forge.md`, `landing.md`, `after-landing.md`, `agent.md`,
`worker.md` and `findings.md` are temper's `../domain/` documents; jig's
and smith's are named as theirs.

## 1. In one page

- **The reference is fabrig's own way of developing itself,** as fab runs
  it today, moved onto temper: a person's draft turned into design and a
  plan by designers, coding agents filling it, a goal branch checked
  along the way and landed whole, then verified after landing.
- **Code production maps onto jig with configuration, and onto temper
  with a few changes.** jig's primitives cover the structure (a goal, a
  coordinator called fresh on each wake, batches with dependencies,
  holds and writer slots, questions, person tasks, escalations, notes,
  jobs). temper's change procedure and its after landing cover most of
  the mechanics; what temper offers a coordinator over them is narrower
  than a plan needs (N22), and a goal's own branch is a change's where
  the reference needs it to be the goal's (N3).
- **Landing is the queue temper has,** one change per goal into main:
  update, CI at the exact head, merge at that head, then tiers. The
  landing model's batches, trains, keys per package, provers and
  serializer are not needed (section 6); landing by turn stays, an
  option the reference does not use.
- **The user's answers (2026-11-02):** implementers ask, through a
  foreman and then the coordinator (N1); a goal's changes land into a
  branch of the goal's, which lands whole (N3); one coordinator per
  goal, at xhigh, mostly configuration (N10, N11). On the plan's text: a
  goal branch everybody writes, designers included, who put their
  output in `docs/design` for this project while other projects do as
  they choose; notes only where they are the right tool; and the ideal
  shape before the mechanisms at hand (N3, N12, N13).
- **Notes stay hints** (N12): the plan's text is the project's design,
  so it belongs in the repository, on the goal's branch, versioned and
  landed with the code it governs. Notes keep jig's use: what runs
  learn.
- **Where the reference differs from fab, on purpose:** the goal's
  branch held by the goal, every write of the goal landing into it by
  change; designers as changes, their design landed into the goal's
  branch, not specs on a trunk; specs as words citing the design; no
  answers or repair sections committed into specs (the store holds
  them); no specs rewritten by an answerer (an amendment, visible,
  instead); reds along the way go to the coordinator as news, as reds
  after landing do; the close-out before landing, not a retire after
  it; designers at xhigh with surveys as Claude sub-agents at high
  (Opus 5.5).

## 2. The evidence

### 2.1 Replay 6 of playground-kit

`playground-reset-r6`, 2026-11-01: the cut started at 09:18:21Z, the
trunk's end check green at 15:41:43Z (T+6:23), first try; never landed
(`ready = false`). The draft it implements is `playground-reset.md` in
this directory.

- **The cut:** one Claude call (Opus 5.5, high), 53 minutes, 4.3M weight;
  6 read-only Codex surveys and 5 Claude designer sub-agents; 63 task
  specs in 8 pushed batches, the first merged at T+27; `plan.md`,
  `notes/contracts.md` and shared notes per family. An answer call added
  a 64th task.
- **Runs:** 185: 121 implement, 5 rebase, 49 foreman, 10 check. 54 early
  starts, 2 refused for a conflict. 135 merges into the trunk, 5 of them
  conflicts.
- **Questions:** 54, 9 of them parks. Foremen settled 14, 35 answer calls
  settled 40, the person none. Every one of the 77 `Done:` reports had a
  `## Left` list; the foreman never answered `Accept:`.
- **Checks along the way:** 9, 8 red (1, 2, 29, 38, 47, 37, 7 and 1
  failing tests), each making one repair; the end check green at once.
- **What cost time:** a serial tail of four tasks after 13:52, behind one
  test-world task (fe-world-remove: 10 runs, 5 foremen, 5 answer calls,
  4 parks) that kept finding faults in code it did not own; early
  starts whose needs pushed late; a rebase that lost the race to the
  trunk twice.
- **The early-start rules the run had to learn** (N2): one early start
  per task, and an answered task waits for its unmerged needs; a failed
  early run waits for its needs; a rerun continued from its pushed
  branch merges the trunk first; a failed run with no question parks
  for the answerer.
- **Mechanisms that exist only because coordination state is files:**
  answers and repair sections committed into specs; wakes through a
  shared file, of which 3 of the cut's 9 reached the tick (the backstop
  found the rest, 24 lost event notes); answer calls that widened
  `files`, added needs, edited another task's spec and made a task,
  against "a pushed spec never changes"; designers' contract requests
  through progress files and the cut's messages.

### 2.2 Landings on record

From fab's event log, the merge queue's `queue.jsonl`, the after-landing
record and fabrig's first-parent history:

- **One trunk has landed through fab:** `phase-case/integration`, a plan
  of one task, 27 minutes from its cut to main, after landing green, its
  retire through the queue.
- **Never in the records:** a catch-up or catch-up repair, a plan stop, a
  queue check of a trunk again, an overlap between trunks, any after
  landing red routed to a foreman. They exist in fab's code and tests
  only.
- **The queue, 131 batches:** 122 green, 3 red, 2 conflict, 1 collateral,
  3 error; 129 of one branch; no bisection; receipts reused 4 times of
  10 delivered. 88 green batches merged nothing: answer branches with no
  commit.
- **By hand, on 10-31, with the queue stopped:** playground-kit's merge
  with main (23 conflicts, 3 repairs, 56 minutes) and the slow tiers'
  reds (about 11 repair tasks); 79 of the 118 first-parent commits on
  main since the cutover came by hand that day.
- **One end-to-end refusal handled by fab:** `v0.1.0-ladder/finding-17`,
  a queue conflict, its foreman, a rebase run, a repair run, landed in
  38 minutes.
- **A trunk checked twice:** it carries no receipt, so the queue runs the
  full checks again right after the end check.

So the landing side of the reference is written from the designs and
from what people did by hand: its catch-ups, its stops and its reds
after landing have never run unattended (`reference-landing.md`,
section 9).

### 2.3 The case, against the run

`reference-case.md`'s steps, each with what it condenses (replay 6
unless named):

| Step | From |
|---|---|
| 1 to 3 | the cut: the trunk made, six surveys, the plan's shape, contracts first, eight batches |
| 4 | `notes/contracts.md`, written before any family's specs |
| 5 | designers' contract requests through progress files and the cut's messages |
| 6 | five designer sub-agents' specs, released per family; the first merge at T+27 |
| 7 | sv-skein-setup, the plain path: done in 9.4 minutes, merged 53 seconds later |
| 8, 10 | bx-init and fe-make-git: early starts, fe-make-git done at 15:23 and held until fe-config-drop merged at 15:36 |
| 9 | bx-doc: its foreman failed, an answer call widened `files` by `problem.rs` |
| 11 | fe-remove: an answer call made sv-skein-retire and added it to fe-remove's needs |
| 12 | the old-forms recount at a merge, which never triggered in replay 6 and settled 17 of 24 conflicts in its predecessor |
| 13, 14 | fe-world-remove: a failed early run, then ten runs fixing code it did not own; the case finishes it at its first Left list |
| 15 | answer calls that edited another task's spec where a shared shape changed |
| 16 | fe-demo-gone's nine-file conflict and its two rebases |
| 17 | fe-world-reset's refused early start on fe-demo-gone |
| 18 | fe-world-reset's question on project isolation, which its foreman settled from the draft; the case passes a hold on it up to the person, a route the run never took |
| 19 | fe-world-remove's widening to jig's `run.rs` (a stale credential), made a change of its own |
| 20 | the checks along the way and repair-5: its question, its answer "you own it", the stall repaired; here the coordinator's triage gives that answer before the repair starts |
| 21 | the catch-up with main, done by hand for playground-kit (23 conflicts) |
| 22 | phase-case's retire through the queue, moved before the landing |
| 23, 24 | the end check, then the landing that never came (`ready = false`); phase-case's landing |
| 25 to 27 | the slow tiers' reds of 10-31, repaired by hand |
| 28 | phase-case's end |

## 3. fab's pipeline in jig's and temper's terms

| fab | jig / temper | Where |
|---|---|---|
| the user starts a draft (`fab trunk make`, `fab call cut`) | a person makes a goal from a template, tracked, with the draft's path; the goal's own branch named and held | jig's `domain/core.md`, 6.2; N3 |
| `<draft>/integration`, the trunk | the goal's own branch, held by the goal | N3 |
| the cut, answer calls, settle, retire | the goal's coordinator, one agent task, fresh on each wake | jig's `domain/core.md`, 7.2; `domain/engine.md`, 7.4 |
| the cut's surveys (Codex, read-only) | sub-agents at high | smith's `domain/run.md`, 5.3; N10 |
| the cut's designer sub-agents | designers' changes into the goal's branch, at xhigh | N13 |
| `notes/contracts.md` | the shared design, a designer's change landed first | N12, N13 |
| `plan.md`, `status.md`, `notes/common.md` | the tree of tasks; the design and status files in the goal's branch | N12 |
| `plan.toml` parts and needs | batches of changes with dependencies | jig's `domain/tasks.md`, section 4; N22 |
| the cut's early release | one batch per family, as its design lands | the same |
| task spec | a change's words, citing the design in the goal's branch | `forge.md`, 8.1; N12 |
| implementer (Codex, high) | the change's producer, the coding agent's charter | `forge.md`, 8.2; N19, N20 |
| precheck and readers' tests in the run | `.temper/pre-pr` before delivery | `agent.md`, 4.5 |
| `Done:` with `## Left` | the producer's report, carried in the change's result | N15 |
| `Question:` → foreman → answer call → user | the producer asks; foreman delegate; the coordinator; a person task | N1 |
| `Infra:` | a transient failure, or a wait for an account | jig's `domain/tasks.md`, 5.5 |
| park at the run limit | the question bound, tries or budget past: held, escalated | jig's `domain/tasks.md`, 5.5 and 9; N1 |
| merge into the trunk, light | a change landed into the goal's branch under the goal branches' landing rule | N4 |
| old-forms recount at a merge | the merge job | N7 |
| merge conflict → refusal foreman → rebase run | the merge job, then a resolution in a merge in progress | `forge.md`, 8.4; N4, N7 |
| early start | a change that starts kept | jig's `domain/connectors.md`, 6.1; N2 |
| check along the way → `repair-<n>` | the goal branch's tier, its red as news to the coordinator, which makes a repair change | N5 |
| catch-up | a catch-up change into the goal's branch, the merge job its producer | N6 |
| retire | the close-out, into the goal's branch before landing | N16 |
| end check, ready, queue | the goal's landing, a change into main on the goal's branch | `reference-landing.md`; N3 |
| plan stop (three reds) | a bound past: the landing held, escalated to the coordinator | jig's `domain/tasks.md`, 5.5 |
| after-landing red → red foreman → red task | a red as news to the goal → triage → a repair change | `after-landing.md`, section 7 |
| retire waits for main and its verdict | the goal's landing lives to cleared; the goal ends with its result | `after-landing.md`, section 6 |
| the user's page, forge issues | the person's inbox, questions on the forge | jig's `domain/people.md`; `forge.md`, 12.1 |
| flow target, places, slots, builders | out of the reference | |
| skeletons, call reviews, test-writers, critique, structure review | off in this reference; each a charter and a delegate when wanted | `reference-code-production.md`, section 16 |

## 4. The choices, N1 to N24

Each item is a choice the reference drafts make where the existing
design says otherwise or says nothing, and why. The drafts hold the
rules; their statuses say where the design still lags.

### N1. A change's agents may ask

- **The choice:** "a change never asks" stays the default, and a change
  takes a setting that relays its agents' questions: through a foreman,
  which settles what the change's words and the design settle, or
  straight to its requester (`reference-code-production.md`, section 8).
- **Why:** in the run, 54 questions were asked and an LLM settled every
  one, most of them about scope: files outside the task, a neighbour's
  fixture, a red the task did not cause (2.1). A change that may not ask
  leaves its producer to guess or fail. A foreman first keeps the
  coordinator's wakes for what the words and the design do not settle.
  A designer's questions are about the draft, which no foreman can read
  better than the coordinator.
- **The change kept by its question** (jig's `domain/tasks.md`, 5.7)
  still steps. It merges nothing while its producer works, and a
  repair's or a resolution's question takes it out of its queue until
  answered, which suits a change that cannot merge yet. A procedure
  decides nothing (jig's `domain/tasks.md`, section 9): it relays, and
  the foreman and the requester decide.
- **The bound** (four in the reference) stands for the run's parks at its
  run limit.

### N2. Early starts: a kept change and its producer

- **The choice:** a change depending on another change into the same
  landing branch starts kept, and its producer starts early from the
  landing branch's tip with its dependencies' pushed heads merged in
  (`reference-code-production.md`, section 7).
- **Why:** the run made 54 early starts, and learned four rules for them
  (2.1); the reference keeps those and adds two: a conflict between the
  heads refuses the start, and an answer that adds a dependency waits
  for it to land. A dropped dependency takes its early dependent with
  it, since the dependent's head carries the dependency's unlanded work,
  so the coordinator remakes the dependent rather than amending the
  dependency away.

### N3. The goal's own branch

- **The choice:** the branch is the goal's, not a change's. The goal
  holds it for its whole life; everything of the goal writes it under
  that hold, by change, one writer at a time; its landing is an ordinary
  change into main whose branch is its requester's
  (`reference-code-production.md`, sections 3 and 4;
  `reference-landing.md`, section 3).
- **Why not a change's:** a goal's own branch made "a change of its own"
  (`forge.md`, 11.1) is at once the landing branch of its goal's changes
  and the branch of its own change, and three rules stand against that:
  a branch under temper's prefix takes an exclusive hold (`forge.md`,
  section 2), only the holder's delegates write under a hold (jig's
  `domain/tasks.md`, 6.1), and a forward move by anyone but the change's
  own writers is drift (`forge.md`, 8.5). A landing branch must also
  exist when named (`forge.md`, 11), so the goal's changes could not be
  made in the batch that makes it. A "collecting change" that held
  nothing, made repairs and catch-ups into its own branch, and was
  closed and reopened by amendment was weighed: it carries a goal's
  lifetime in a change.
- **Why by template:** a goal's own branch is part of the shape of work.
  A small goal names none, and its changes land into main each, as
  `forge.md`, 11.1 advises for most work. A goal cannot take a hold once
  active (jig's `domain/tasks.md`, 6.2), so the choice is made with the
  goal. The change to jig is not temper's alone: an incident template
  could name its own environment the same way.
- **Why everyone writes by change:** direct pushes by many producers
  would race, and a run holding the branch's writer slot for its whole
  length would stop every other writer. Only the landing's own repairs
  and resolution push it directly, when nothing else should land.

### N4. Light rules for changes into a goal's branch

- **The choice:** a change's checking and updating follow its landing
  branch's rule, and the rule for goal branches asks neither: a change
  into one merges at its head once the forge finds it mergeable, and a
  merge refused for a conflict goes to the merge job
  (`reference-code-production.md`, section 6).
- **Why:** the goal's landing into main checks the whole
  (`reference-landing.md`, section 1), and g's tier checks along the way
  (N5); CI and an update on every change into g would cost a gate per
  change for a branch nothing ships from. The run merged into its trunk
  lightly and checked along the way (2.1). One rule on `… branch temper`,
  its last segment open, covers every goal's branch (jig's
  `domain/authority.md`, section 4), since a change's own branches and
  saved work are never landing branches. A merge commit keeps g's
  history, each change's merge, readable on main.

### N5. Tiers that name their branches; a goal branch's tier

- **The choice:** a tier names the branches it follows. A goal's branch
  has a tier of its own, which judges no landing; its reds go to the
  goal as news and the coordinator triages them; merges go on
  (`reference-code-production.md`, section 10).
- **Why:** main's slow tiers following every goal's branch would make each
  change into one wait for their verdict. The run kept merging while a
  repair ran (2.1: 9 checks, 8 red, one repair each). Reds along the way
  and after landing reach the coordinator the same way, one mechanism.

### N6. Catching up along the way

- **The choice:** the goal catches up by a change into its branch, its
  producer the merge job, made by its coordinator when main's landings
  touch its open work, paced by its wake policy (N23)
  (`reference-code-production.md`, section 11).
- **Why:** the run's predecessors left catching up to the end, by hand:
  23 conflicts and 56 minutes (2.2). The forge's update is a pull
  request's, which a goal's branch has not until its landing is made.

### N7. The merge job

- **The choice:** a job kind the project declares, its `merge_job`: given
  a branch and a base, it merges the base in, settles the conflicts the
  project's tooling knows, pushes the merge if none remain, and
  otherwise leaves the merge in progress for a resolution. It serves a
  merge into a goal's branch refused for a conflict (N4), a catch-up's
  producing (N6), and an update the forge refused, the goal's landing's
  among them, before a resolution.
- **Why:** in the run's predecessor 17 of 24 merges met conflicts in
  counted baselines, which a recount settles with no model (2.3, step
  12).

### N8. Answers at once

- **The choice:** the coordinator's wake policy wakes on each question,
  with no batching; revisit with the numbers (section 5).
- **Why:** fab answers at once when a question holds up other work,
  otherwise at four or after 30 minutes; an integration plan answers at
  once. "At once when it blocks" is no wake policy's word (jig's
  `domain/tasks.md`, 8.3), and is left out.

### N9. Order of work within a goal

- **The choice:** Later (`reference-code-production.md`, section 16).
- **Why:** jig orders runs waiting for room by their goal's priority,
  then by when they began to wait; fab runs consolidation, repairs and
  rebases before new tasks. The order matters only when capacity is
  short, which the reference leaves out.

### N10. Surveys as sub-agents on Claude Code

- **The choice:** smith serves its `sub_agent` to a program through its
  MCP server, each child a program session of its own at the model and
  effort the LLM choice names; inspect-only children run side by side
  (`reference-code-production.md`, section 5).
- **Why:** the coordinator and the designers run on Claude Code, and the
  run's surveys were read-only questions asked side by side (2.1).
  Designers are changes' producers, not sub-agents (N13), so no child
  writes and no child's answer carries a family's design. A survey is a
  saving, not a need: a run without sub-agents reads for itself.

### N11. One coordinator, several kinds of wake

- **The choice:** one task has one LLM choice, so the coordinator answers
  questions, reads each change's result, triages each red of its
  branch's tier and decides each catch-up at xhigh. Accepted (the user,
  2026-11-02).
- **Why:** fab's answer calls ran at medium (6.7M of the run's 11.0M
  Claude weight); in the run's terms the coordinator wakes about 170
  times, fewer where messages reach a run already going. If the numbers
  ask: an LLM choice per kind of wake on the charter, or catch-ups and
  repairs along the way made by a rule rather than judged
  (`reference-code-production.md`, sections 16 and 17).

### N12. The plan lives in the goal's branch; notes stay hints

- **The choice:** the plan's text (the shared shapes, each family's
  design, the status of the code, each spec) is design, in the goal's
  branch; coordination state stays in the store (tasks, words,
  questions, answers, holds); notes keep jig's meaning, what runs learn
  and pass on (`reference-code-production.md`, section 5).
- **Three homes were weighed:**
  - **files on a trunk,** fab's: the coordinator writes and pushes them,
    which a goal's runs may not (`agent.md`, 4.3), and fab keeps its
    coordination state there too (2.1);
  - **notes in the goal's scope:** no writing by the coordinator,
    revisions fenced, readable by every task of the project (jig's
    `domain/engine.md`, section 10). But notes are hints, "where a note
    and the facts disagree, the facts are right", and the plan's
    contracts bind; a revision notifies no one, so the coordinator
    amends live changes by hand; a spec is copied from its note into a
    change's words; and the plan's text outlives the goal only by a
    second copy into the design;
  - **the project's design in the goal's branch:** written by designers'
    changes (N13), versioned, merged and landed with the code it
    governs; read in each producer's checkout; revised by a change,
    whose landing is a fact every later checkout has. The reference
    takes this one.

### N13. Designers are changes

- **The choice:** a change names its producer's charter among those its
  maker may delegate to, and a designer's change is one whose producer
  runs under the designer's charter. It lands into the goal's branch as
  any change does, with no rule of its own
  (`reference-code-production.md`, sections 2 and 5).
- **Why tasks, not sub-agents:** a design is a commit, and lands as code
  does; a sub-agent's answer would carry a family's design through the
  coordinator's context, and could not be asked about later.

### N14. A producer delivers more than once

- **The choice:** a producer pushes as it goes, each delivery pushing the
  change's branch (`reference-code-production.md`, section 6).
- **Why:** a kept dependent starts from its dependency's pushed head (N2).

### N15. A change's result carries its producer's report

- **The choice:** a change's result words carry its producer's last
  report after its landing's lines, and the coding agent's charter says
  how to finish with what is outside the task
  (`reference-code-production.md`, sections 6 and 14).
- **Why:** the coordinator needs the report, for a Left list or a
  designer's list of changes; every `Done:` report of the run had a Left
  list (2.1). A change's contract requires its checks to pass at
  delivery (jig's `domain/hosts.md`, section 8; `agent.md`, 4.4), and in
  the run the test-world tasks met failures in code they did not own:
  marking such a test ignored, the fault its reason, keeps the checks
  green without fixing what the change does not own.

### N16. The close-out lands with the goal

- **The choice:** what the goal leaves for the next one (the status
  files, the designs' Later sections, the draft's fate) is written into
  the goal's branch before its landing, by a designer's change
  (`reference-code-production.md`, section 12).
- **Why:** it lands with the code it describes, in one landing checked
  whole, and a goal that never lands leaves nothing half-said on main.
  fab retires after the landing and its verdict, as a second landing
  through the queue (2.2: phase-case's retire). A repair after landing
  that changes what the close-out wrote carries the edit itself.

### N17. A red at the gate that passes alone

- **The choice:** Later (`reference-landing.md`, section 11).
- **Why:** fab retries a red test alone before rejecting (collateral).
  temper reruns failed tests alone for tiers (`after-landing.md`, 5.2),
  and, under the turn, reruns a gate's failed jobs once when every
  failing test matches an open flake finding (`findings.md`, section 5);
  under the queue a CI red is a repair. With one change in the queue
  there is no collateral, and a repair that finds nothing to change says
  so. Revisit if flakes cost repairs.

### N19. A change names its agents' LLM choices

- **The choice:** a change's parameters may carry an LLM choice for each
  of its agents' roles (`reference-code-production.md`, section 5).
- **Why:** the run's specs chose an effort each (high for 52, medium for
  12), its foremen ran at medium, and the reference's designers run at
  xhigh with sub-agents at high.

### N20. Implementers on gpt-6.1-sol

- **The choice:** the reference's producers run gpt-6.1-sol on smith's
  direct backend, through an OpenAI account with a key; Codex as a
  program backend is Later (`reference-code-production.md`, section 16).
- **Why:** the run's implementers were Codex, signed in to ChatGPT, and
  smith's one program backend is Claude Code. A key is what smith and
  jig already carry; a login is never copied to make Codex a program
  (`AGENTS.md`, 3). Its umask, its logs' mode and its closed stdin are
  the program backend's to keep when it comes.

### N21. Limits for a goal of this size

- **The choice:** the reference deploys with a profile sized for its goal
  (`reference-code-production.md`, section 4).
- **Why, and the sizes:** the run's goal had 64 changes, 185 runs and 54
  questions under one coordinator (2.1). The profile holds:
  - G's delegates: about a hundred changes over the goal's life (the
    designers', the plan's, catch-ups, repairs along the way, the
    close-out, the landing, repairs after landing), and person tasks;
  - a change's tasks: a producer, up to four foremen, merge jobs,
    resolutions, repairs, about twelve;
  - tasks per tree, about eight hundred, and depth: G at the root, a
    change 1 below it, its producer, foreman or resolution 2;
  - G's inbox (one result's room per live delegate: jig's
    `domain/tasks.md`, 8.2) and open questions per task;
  - the words of a change and the bytes of a result (N15): a designer's
    report lists up to eleven changes; the batch size; and a call's
    input bound, which a `change` call carrying a family's changes must
    pass, one change per call if it must;
  - a charter's documents.

### N22. A coordinator makes changes with all their parameters

- **The choice:** `change` takes, beside the words, what a plan decides
  (`reference-code-production.md`, section 3), rather than a run being
  given the change procedure's `delegate`: the root keeps building the
  delegate from what it knows, and the run names only what the plan
  decides.
- **Why:** the plan's changes go into the goal's branch, on its own held
  branch, with dependencies, as one batch, relaying questions, with LLM
  choices, a producer's charter or the merge job, or naming the red they
  repair (`repairs`, `after-landing.md`, section 7). A run may delegate
  to the change procedure under the stock rules, but cannot know what
  the procedure's delegate needs (`agent.md`, 4.3.1); `change` builds it.

### N23. A charter's default wake policy

- **The choice:** a charter in configuration carries a default wake
  policy, in jig's closed vocabulary (jig's `domain/tasks.md`, 8.3); the
  coordinator's is `reference-code-production.md`, section 13.
- **Why:** every delegate's result, not only the last: the coordinator
  reads each result for its Left list, and makes a family's batch from
  its designer's report (N15). News batched: a catch-up or a red of g's
  tier needs no answer within minutes, and each wake is a run at xhigh
  (N11).

## 5. What would reopen a choice

The numbers: questions per change and the share the foreman settles (N1,
N8); catch-ups and their conflicts per goal (N6); flakes at the gate
(N17); the coordinator's wakes and their cost at xhigh (N11); the size of
designers' reports (N15, N21).

## 6. The landing model's items

The reference drops the landing model (`landing-model.md`, 2026-10-30,
readable in history at `8a4c5bcd9f`), a plan for many landings at once,
whose items other documents cite by number. Each item, under the
reference:

| Items | Under the reference |
|---|---|
| L1 batches, L2 trains, L3 candidates by a merge chain, L5 keys per package, L6 gate classes, L7 provers and runners, L9 the serializer's durability, L18 the reservation | not needed: one change at a time owns main's queue; landing by turn is `landing.md`'s |
| L4 the project's tooling as a job, no model | needed, as the merge job (N7) |
| L8 a measured failure rerun once before red | Later, for the gate (N17); tiers have it (`after-landing.md`, 5.2) |
| L10 a repair pinned ahead of every landing | temper's branch repair, first in its queue (`forge.md`, section 9) |
| L11 a change's own checks before ready | the producer's checks at delivery (`.temper/pre-pr`) |
| L12 leases on paths across goals | not needed: one branch per goal, held by it (N3); overlaps between goals Later (`reference-landing.md`, section 11) |
| L13 tiers following main, L14 cleared as a setting | needed, as `after-landing.md` has them, each tier naming the branches it follows (N5) |
| L15 findings, L16 QA walks | `findings.md`; the reference uses only a red of no goal's |
| L17 trailers in landing news | `after-landing.md`, section 8; the reference does not need them |
| L19 views and acts | the client's, for what remains: the queue, tiers, verdicts, holds |
| L20 one change in one repository | kept: the reference is one repository |
| J1 owned holds | jig's; the goal's hold on g is one (N3) |
| J2 to J6, J9, J12: drains, inherited holds, parks until an event, moves, successors, holds over admission, lent workspaces | not needed by the reference; jig's designs keep them |
| J7, J8 a conversation's questions | not needed: no chat stands between the goal and its person; jig's `domain/tasks.md`, 8.5 keeps them |
| J10 an address by role | not needed: questions and escalations go up the tree |
| J11 a fresh run at a context's bound | the coordinator starts fresh on each wake (jig's `domain/engine.md`, 7.4); a long run on Claude Code is the program's to compact |
| F1 skein's artifact read | needed by tiers (`after-landing.md`, section 4) |
| F2 to F5 GitHub's turn and permissions | not needed: the reference lands by queue |
