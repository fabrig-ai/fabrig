# Notes behind the reference drafts

Draft, 2026-11-02. What `reference-code-production.md`,
`reference-landing.md` and `reference-case.md` rest on: the run they are
drawn from, the landings on record, today's pipeline in jig's and
temper's terms, and every change to an existing design the drafts need,
N1 to N23, each with a recommendation and a draft rule. Unlike the
three drafts, these notes name today's pipeline (fab, in the agents
repository) and its runs, since they are the evidence. The decisions
are the user's; the user's answers of 2026-11-02 are in 1.

Unqualified, `forge.md`, `landing.md`, `after-landing.md`, `agent.md`,
`worker.md` and `findings.md` are temper's `../domain/` documents; jig's
and smith's are named as theirs.

## 1. In one page

- **The reference is fabrig's own way of developing itself,** as fab runs
  it today, moved onto temper: a person's draft cut into a plan by a
  designer, Codex implementers filling it, a goal branch checked along
  the way and landed whole, then verified after landing.
- **Code production maps onto jig with configuration, and onto temper
  with a few changes.** jig's primitives cover the structure (a goal, a
  coordinator called fresh on each wake, designers as tasks, batches
  with dependencies, notes, questions, person tasks, holds and
  escalations, jobs). temper's change procedure, its goal branch
  (`forge.md`, 11.1) and its after landing cover most of the mechanics;
  what temper offers a coordinator over them is narrower than a plan
  needs (N21). What is missing is section 4.
- **Landing is the queue temper has,** one change per goal into main:
  update, CI at the exact head, merge at that head, then tiers. The
  landing model's batches, trains, keys per package, provers and
  serializer are not needed; the landing by turn stays as built, an
  option the reference does not use.
- **The user's answers (2026-11-02):** implementers ask, through a
  foreman and then the coordinator (N1); a goal's changes land into a
  branch of the goal's, which lands whole (N3); one coordinator per
  goal, its designer's runs at xhigh, mostly configuration (N10, N11);
  these notes hold the design changes. The plan's shared text goes in
  notes rather than files (N12), the user's suggestion, taken.
- **Changed from fab, on purpose:** the plan's text as notes, not files
  on the trunk; designers as tasks, not sub-agents of the cut; no answers
  or repair sections committed into specs (the store holds them); no
  specs rewritten by an answerer (an amendment, visible, instead);
  designers at xhigh with surveys as Claude sub-agents at high (Opus
  5.5).

## 2. The evidence

### 2.1 Replay 6 of playground-kit

`playground-reset-r6`, 2026-11-01, the cut started 09:18:21Z, the trunk's
end check green at 15:41:43Z (T+6:23), first try; never landed
(`ready = false`). Sources: fab's event log (`log.jsonl`), the runs'
briefs and reports, the operator's log and report, the trunk
`playground-reset-r6/integration` at `fd2869ff5d`. The draft it
implements is `playground-reset.md` in this directory.

- **The cut:** one Claude call (Opus 5.5, high), 53 minutes, 4.3M weight;
  6 read-only Codex surveys and 5 Claude designer sub-agents; 63 task
  specs in 8 pushed batches, the first merged at T+27; `plan.md`,
  `notes/contracts.md` and shared notes per family. A 64th task was
  added later by an answer call.
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
- **What the operator fixed during the run** (cherry-picked into the
  replay's install): one early start per task, and an answered task
  waits for its unmerged needs; a failed early run waits for its needs;
  a rerun continued from its pushed branch merges the trunk first; a
  failed run with no question parks for the answerer.
- **Mechanisms that exist only because state is files:** answers and
  repair sections committed into specs; wakes through a shared file, of
  which 3 of the cut's 9 reached the tick (the backstop found the rest,
  24 lost event notes); answer calls that widened `files`, added needs,
  edited another task's spec and made a task, against "a pushed spec
  never changes"; designers' contract requests through progress files
  and the cut's messages.

### 2.2 Landings on record

Sources: fab's event log, the merge queue's `queue.jsonl`, the after
landing record, fabrig's first-parent history.

- **One trunk has ever landed through fab:** `phase-case/integration`, a
  plan of one task, 27 minutes from its cut to main, after landing
  green, its retire through the queue.
- **Never in the records:** a catch-up or catch-up repair, a plan stop, a
  queue check of a trunk again, an overlap between trunks, any after
  landing red routed to a foreman. They exist in fab's code and tests
  only.
- **The queue, 131 batches:** 122 green, 3 red, 2 conflict, 1 collateral,
  3 error; 129 of one branch; no bisection, ever; receipts reused 4
  times of 10 delivered. 88 green batches merged nothing: answer
  branches with no commit.
- **By hand, on 10-31, with the queue stopped:** playground-kit's merge
  with main (23 conflicts, 3 repairs, 56 minutes) and the slow tiers'
  reds (about 11 repair tasks); 79 of the 118 first-parent commits on
  main since the cutover came by hand that day.
- **One end-to-end refusal handled by fab:** `v0.1.0-ladder/finding-17`,
  a queue conflict, its foreman, a rebase run, a repair run, landed in
  38 minutes.
- **A trunk was checked twice:** it carries no receipt, so the queue ran
  the full checks again right after the end check.

So the landing side of the reference is written from the designs and
from what people did by hand, and says so: its catch-ups, its stops and
its reds after landing have never run unattended.

### 2.3 The case, against the run

`reference-case.md`'s steps, each with what it condenses (replay 6
unless named):

| Step | From |
|---|---|
| 1 to 6 | the cut: six surveys, five designers, contracts first, contract requests through progress files, eight batches, the first merge at T+27 |
| 7 | sv-skein-setup, the plain path: done in 9.4 minutes, merged 53 seconds later |
| 8, 10 | bx-init and fe-make-git: early starts, fe-make-git done at 15:23 and held until fe-config-drop merged at 15:36 |
| 9 | bx-doc: its foreman failed, an answer call widened `files` by `problem.rs` |
| 11 | fe-remove: an answer call made sv-skein-retire and added it to fe-remove's needs |
| 12 | the old-forms recount at a merge, which never triggered in replay 6 and settled 17 of 24 conflicts in its predecessor |
| 13, 14 | fe-world-remove: a failed early run, then ten runs fixing code it did not own; the case finishes it at its first Left list |
| 15 | fe-demo-gone's nine-file conflict and its two rebases |
| 16 | fe-world-reset's refused early start on fe-demo-gone |
| 17 | fe-world-reset's question on project isolation, which its foreman settled from the draft; the case passes a hold on it up to the person, a route the run never took |
| 18 | fe-world-remove's widening to jig's `run.rs` (a stale credential), made a change of its own |
| 19, 20 | the checks along the way and repair-5: its question, its answer "you own it", the stall repaired |
| 21 | the catch-up with main, done by hand for playground-kit (23 conflicts) |
| 22 to 24 | the end check, then the landing that never came (`ready = false`); phase-case's landing |
| 25 to 27 | the slow tiers' reds of 10-31, repaired by hand |
| 28 | phase-case's retire through the queue |

## 3. Today's pipeline in jig's and temper's terms

| fab | jig / temper | Where |
|---|---|---|
| the user starts a draft (`fab trunk make`, `fab call cut`) | a person makes a goal from a template, tracked, with the draft's path | jig's `domain/core.md`, 6.2 |
| the cut, answer calls, settle, retire | the goal's coordinator, one agent task, fresh on each wake | jig's `domain/core.md`, 7.2; `domain/engine.md`, 7.4 |
| the cut's surveys (Codex, read-only) | sub-agents at high | smith's `domain/run.md`, 5.3; N10 |
| the cut's designer sub-agents | designer tasks at xhigh, writing specs as notes | jig's `domain/tasks.md`, section 4; N12 |
| `plan.md`, `status.md`, `notes/common.md`, `notes/contracts.md` | notes in the goal's scope | jig's `domain/engine.md`, section 10; N12 |
| `plan.toml` parts and needs | batches of changes with dependencies | jig's `domain/tasks.md`, section 4; N21 |
| the cut's early release | one batch per wake, as each family is ready | the same |
| task spec | a change's words, copied from its note | `forge.md`, 8.1 |
| implementer (Codex, high) | the change's producer, the coding agent's charter | `forge.md`, 8.2; N18, N19 |
| precheck and readers' tests in the run | `.temper/pre-pr` before delivery | `agent.md`, 4.5 |
| `Done:` with `## Left` | a result's words | jig's `domain/tasks.md`, 5.6; N14 |
| `Question:` → foreman → answer call → user | the producer asks; foreman delegate; the coordinator; a person task | N1 |
| `Infra:` | a transient failure, or a wait for an account | jig's `domain/tasks.md`, 5.5 |
| park at the run limit | the question bound, tries or budget past: held, escalated | jig's `domain/tasks.md`, 5.5 and 9; N1 |
| `<draft>/integration` | the goal branch, a collecting change's | `forge.md`, 11.1; N3 |
| merge into the trunk, light | a change landed into the goal branch under its light rules | N6 |
| old-forms recount at a merge | the merge job | N7 |
| merge conflict → refusal foreman → rebase run | the merge job, then a resolution in a merge in progress | `forge.md`, 8.4; N6, N7 |
| early start | a change that starts kept | jig's `domain/connectors.md`, 6.1; N2 |
| check along the way → `repair-<n>` | the goal branch's tier, red → one repair change | N5 |
| end check, catch-up, ready, queue | the goal's change into main; catch-up changes | `reference-landing.md`; N4 |
| plan stop (three reds) | a bound past: held, escalated to the coordinator | jig's `domain/tasks.md`, 5.5 |
| after-landing red → red foreman → red task | a red as news to the goal → triage → a repair change | `after-landing.md`, section 7 |
| retire waits for main and its verdict | the goal's change lives to cleared; the retire, a change from words | `after-landing.md`, section 6; N15 |
| the user's page, forge issues | the person's inbox, questions on the forge | jig's `domain/people.md`; `forge.md`, 12.1 |
| flow target, places, slots, builders | out of the reference | |
| skeletons, call reviews, test-writers, critique, structure review | off in this reference; each a charter and a delegate when wanted | `reference-code-production.md`, section 16 |

## 4. Changes to existing designs

Each item says what an existing design says, why the reference needs
otherwise, the recommendation and a draft rule. They are the input for
changing designs and code; the drafts cite them by number.

### N1. A change's agents may ask

- **Conflict:** "The work that waits asks; a change never does"
  (`agent.md`, 8.2; `forge.md`, 8.1): the change delegates its agents
  without the Message family and answers any question with "a change
  takes no questions". In the run, 54 questions were asked and every one
  was settled by an LLM, most of them about scope (files outside the
  task, a neighbour's fixture, a red the task did not cause).
- **Recommendation:** keep "a change never asks" as the default, and give
  the change a setting that relays its agents' questions. A foreman
  settles what the change's own words settle; the rest goes up.
- **Draft rule:** a change's `questions` parameter, from its maker or the
  project's setting: `none` (today) or `relay`. With `relay`, its
  producer, repairs and resolutions keep the Message family. A question
  one asks reaches the change; its step delegates a foreman (an agent
  task under the foreman's charter, read-only, its brief the change's
  words, the question and the change's pushed head, reading the goal's
  notes) whose verdict is `answer` (with the answer), `finish` (the
  work done holds; what is left goes to the asker's Left list) or
  `escalate`. On `answer` and `finish` the change answers its
  delegate's question with the foreman's words; on `escalate` it asks
  its requester the same question in the question form, and is kept by
  it (jig's `domain/tasks.md`, 5.7), which is harmless: a procedure's
  step goes on while kept, and escalations skip it anyway. It passes
  the answer down when it comes. One foreman per question. The change's
  own page lists its open questions. A procedure decides nothing (jig's
  `domain/connectors.md`, 6.3): it relays, and the foreman and the
  requester decide.
- **Bounded:** a change's `questions` bound (four in the reference);
  past it, the next question gets no foreman: the change holds itself,
  saying so, and the hold escalates to its requester (the run's parks at
  its run limit). Released, the change relays the question that waits
  to its requester with no foreman.
- **Relayed through a procedure above:** a change whose requester is a
  procedure (a goal branch's repair or catch-up, N4, N5) asks that
  procedure, which, relaying too, asks its own requester the same
  question, with no second foreman.
- **The foreman's charter** is the forge's kind 3, number 3, beside the
  change's and the reviews' (temper's `../protocol/configuration.md`,
  13.2); kind 3 is itself designed, not built.

### N2. Early starts: a kept change and its producer

- **Conflict:** the change starting kept is designed (jig's
  `domain/connectors.md`, 6.1; `landing.md`, 4.1) but not built
  (`landing.status.md`), and no checkout start gives its producer its
  dependencies' work (`worker.md`, 4.1 and section 5: a landing branch,
  a branch, saved work, or a merge in progress).
- **Recommendation:** build the kept start, and a checkout start that
  merges the dependencies' pushed heads onto the landing branch's tip.
  Carry over the four rules the run had to learn, and two more.
- **Draft rule:** a change made with a dependency on another change into
  the same landing branch starts kept. Its producer starts once each
  pending dependency's change has a pushed head and none of them is
  kept by a dependency of its own (one level; a dependency kept only by
  its own relayed question does not block), from a checkout at the
  landing branch's tip with those heads merged in, in dependency order;
  a conflict there refuses the start, and the change waits for the
  dependencies to land. A change starts early once: once its producer
  has asked, or failed for want of its dependencies' work, the next
  producer waits for them to land. A producer continuing from saved
  work or its pushed branch merges the landing branch's tip into it
  first; a conflict is left in the checkout as a merge in progress. The
  kept change's merge waits for its dependencies, as jig's rule already
  says.
- **A dependency added while the producer waits on a question** (an
  answer that makes the work wait for another change) holds the answer
  at the change until the dependency has landed; the producer's next
  run then merges the landing branch's tip first, by the rule above.
- **A dropped dependency takes its early dependent with it:** the
  dependent's head carries the dependency's unlanded work. The forge
  refuses an amendment removing a dependency whose pushed head the
  change's producer started from, naming it, and the coordinator cancels
  and remakes the dependent instead.

### N3. The collecting change

- **Conflict:** `forge.md`, 11.1 allows a goal's own branch in words,
  and `forge.md`, 11 says the change that lands it creates it at its
  base's tip, as an effect. Nothing says when its change becomes ready,
  and three rules stand against its double role, landing branch of the
  goal's changes and branch of its own change: a branch under temper's
  prefix takes an exclusive hold (`forge.md`, section 2), and only its
  holder's delegates write under a hold (jig's `domain/tasks.md`, 6.1);
  a forward move by anyone but the change's delegates, a run that handed
  it the branch or an update it asked for is drift (`forge.md`, 8.5); a
  repair is an agent pushing to the change's branch (`forge.md`, 8.4). A
  landing branch must also exist when named (`forge.md`, 11), so the
  goal's changes cannot be made in the batch that makes it.
- **Recommendation:** a collecting change holds nothing. Its branch is a
  landing branch, shared, written only by merges through its queue and
  by the change's own update into its base; it writes nothing else
  itself. Its repairs and conflicts become changes into its own branch.
- **Draft rule:** a change may be made `collect: open`. Its first step
  creates its branch, `temper/<root>/g<task>`, at its landing branch's
  tip, and tells its requester so in words. Its branch takes no hold: it
  is a landing branch (`forge.md`, section 9), and the merges of its
  queue are its own moves, never drift. While open, it is not ready,
  opens no pull request, and makes repair changes (N5) and catch-up
  changes (N4) into its branch, under an authority that grants it the
  change procedure's delegation and the merge job (N20). Its requester
  closes it by amending `collect` to `closed`; once no change into its
  branch is live, it opens its pull request and goes its way into its
  landing branch as any change, but that a CI failure or a gate's
  request makes a repair change into its branch, and a conflicting
  update a catch-up change into it, never an agent on its branch. Its
  update into its base, the forge's pull-request update, is made only
  then, while nothing else writes its branch. Closed, it refuses a new
  change into its branch at the batch, naming itself; its requester may
  reopen it by the same amendment, which takes it out of the queue until
  it is closed again.

### N4. A goal branch catches up with main along the way

- **Conflict:** "It never updates only because its base moved"
  (`forge.md`, 10) against "it takes the base's landings that touch its
  area as they come, by an update" (`forge.md`, 11.1); and the forge's
  update is a pull request's (kind 11), which an open collecting change
  has not. The run's predecessors left catching up to the end, by hand
  (2.2).
- **Recommendation:** an open collecting change catches up by a change
  into its own branch, made at most once per span, never while a repair
  or a catch-up into its branch is live.
- **Draft rule:** a collecting change's `catch_up` span (the project's
  setting, 30 minutes by default; none turns it off). While open, when
  its landing branch has moved, the span has passed since its last
  catch-up and no repair or catch-up into its branch is live, it makes a
  catch-up change into its branch: its own branch made at the goal
  branch's tip, its producer the merge job (N7) merging the landing
  branch's tip in, a resolution after it for what the job leaves, its
  words naming the base's sha; it merges into the goal branch through
  its queue under the light rules (N6). Three catch-ups in a row that do
  not bring the base in hold the collecting change, escalated to its
  requester.

### N5. Tiers that name their branches; a goal branch's tier

- **Conflict:** each of a repository's tiers runs on every landing
  branch's moves, and the landings it covers live to cleared
  (`after-landing.md`, sections 1, 3 and 6), so main's slow tiers would
  also follow every goal branch, and each change into one would wait for
  their verdict. A landing branch's own CI pauses its queue while red
  and makes one repair (`forge.md`, section 9); the run kept merging
  while a repair ran.
- **Recommendation:** a tier names the branches it follows. A goal
  branch is followed by a tier of its own, at most once per span, which
  judges no landing; a red makes one repair change into the goal branch,
  mechanically; merges go on.
- **Draft rule:** a tier's settings name the branches it follows: a
  landing branch by name, or `collecting`, every open collecting
  change's branch; main's tiers name main. A tier following collecting
  branches runs at a branch's newest tip at most once per its `span`
  (20 minutes in the reference), judges no landing, and blames no one: a
  red, rerun alone as `after-landing.md`, 5.2 says, makes one repair
  change into the goal branch, made by the collecting change's step, its
  words the failing tests and the log's end, and the changes not yet
  landed into the branch with the files they name, to leave alone. No
  new run starts while that repair is live; the queue into the goal
  branch is not paused. A branch no tier judging landings follows clears
  its landings as they land (`after-landing.md`, section 2), so changes
  into a goal branch end at landed. Its reds never hold the collecting
  change; its own CI into main does (`reference-landing.md`).

### N6. Light rules for changes into a collecting change's branch

- **Conflict:** a change always has a checking step, CI on its head
  (`forge.md`, 8.2, step 3), is updated when first if it lacks its
  base's tip (8.2, step 6; section 10), and waits on its own for CI and
  its base's tip whatever the rules say (8.7). A merge the forge refuses
  for a conflict has no path: resolutions come from refused updates
  (section 10). Landing rules cannot single out goal branches: a jig
  pattern is exact segments then one last segment exact or open
  (jig's `domain/authority.md`, section 4), so `temper/*/g*` is none.
- **Recommendation:** the light rules come from the collecting change,
  not from a landing rule: a change whose landing branch is an open or
  closing collecting change's needs nothing but a mergeable head.
- **Draft rule:** such a change has no checking and no gating step and
  is never updated when first: it merges at its head once the forge
  finds it mergeable. A merge the forge refuses for a conflict delegates
  the merge job (N7) on the change's branch, the goal branch's tip as
  its base; done, the change's new head is the merge; left with
  conflicts, a resolution follows in that merge in progress, as
  `forge.md`, 8.4 has it; then it merges. Its resolutions' bound holds
  it as ever. The repository's merge style is a merge commit (a setting
  of the repository, `forge.md`, 4.4).

### N7. The merge job

- **Conflict:** under the queue a conflict goes to a resolution, an
  agent (`forge.md`, 8.4 and 10); the project's tooling that settles
  known conflicts is the landing by turn's only (`landing.md`, 5.3), and
  jobs are designed, not built (jig's `domain/hosts.md`, 6.9). In the
  run's predecessor 17 of 24 merges met conflicts in counted baselines,
  which a recount settles.
- **Recommendation:** build jobs, and one job kind a project declares:
  given a branch and a base, merge the base in, settle the conflicts the
  project's tooling knows, push the merge if none remain, and otherwise
  leave the merge in progress for a resolution.
- **Draft rule:** a project's `merge_job` setting names a job kind. It
  serves three places: a merge into a goal branch refused for a conflict
  (N6); a catch-up change's producing (N4); and, before a resolution, an
  update the forge refused (`forge.md`, 8.4 and 10). Until jobs are
  built, a resolution agent with the repository's rule in `AGENTS.md`.

### N8. Answers at once

- **Conflict:** none in design: jig's wake policy batches by count or age
  (jig's `domain/tasks.md`, 8.3). fab answers at once when a question
  holds up other work, otherwise at four or after 30 minutes; an
  integration plan answers at once.
- **Recommendation:** the coordinator's wake policy wakes on each
  question (no batching); revisit with the numbers. "At once when it
  blocks" is not a wake policy's, and is left out.

### N9. Order of work within a goal

- **Conflict:** jig orders runs waiting for room by their goal's priority
  then by when they began to wait; fab runs consolidation, repairs and
  rebases before new tasks.
- **Recommendation:** Later; it matters only when capacity is short,
  which the reference leaves out.

### N10. Surveys as sub-agents on Claude Code

- **Conflict:** the LLM choice carries a main model and effort and its
  sub-agents' (jig's `domain/tasks.md`, section 3), but smith offers no
  sub-agents on a program backend (smith's `protocol/programs.md`,
  section 2; its open question in section 9), and the coordinator and
  designers run on Claude Code.
- **Recommendation:** answer smith's open question: serve smith's
  `sub_agent` to the program through its MCP server, each child a
  program session of its own with the model and effort the LLM choice
  names. Until then a coordinator or designer that wants a survey reads
  for itself.
- **Draft rule:** the goal template's LLM choice is `{"main": {"model":
  "claude-opus-5-5", "effort": "xhigh"}, "sub_agents": [{"model":
  "claude-opus-5-5", "effort": "high"}]}`, and the designers G makes
  take the same; smith serves `sub_agent` on Claude Code under it,
  inspect-only children side by side (smith's `domain/run.md`, 5.3).
  Designers are tasks, not sub-agents (N12), so no child writes and no
  child's answer carries a family's specs.

### N11. One coordinator, several kinds of wake

- **Conflict:** none; a cost. One task has one LLM choice, so the
  coordinator answers questions and reads each change's result at xhigh
  (N22), where fab's answer calls ran at medium (6.7M of the run's 11.0M
  Claude weight). In the run's terms, about 150 wakes, fewer where
  messages reach a run already going.
- **Recommendation:** accept it for now (the user, 2026-11-02). If the
  numbers ask, an LLM choice per kind of wake (a question, an
  escalation, a delegate's result) on the charter, which jig would add to
  the run's check.

### N12. The plan's text as notes

- **Conflict:** none in jig, which has notes for this: entries with a
  one-line description and a body, in a goal's scope, written by a tool
  under the task's note scopes, revised under a fence, read in two steps
  (the index in every brief, the entry by `recall`), and readable by
  every task of the project (jig's `domain/engine.md`, section 10;
  `domain/authority.md`, section 3). fab kept the plan's text in files on
  the trunk, landed first; on temper that would make the coordinator
  write and push, which its runs may not (`agent.md`, 4.3: a goal's runs
  inspect alone), and hand its run branch to a change mid-cut, which
  freezes that change until the run answers (jig's `domain/tasks.md`,
  6.4; `forge.md`, 8.5).
- **Recommendation:** the plan's shared text is notes in the goal's
  scope; the coordinator writes no file. Code still moves only through
  git: pushed heads, merges and changes.
- **Draft rule:** the coordinator writes `plan/strategy`, `plan/status`,
  `plan/common` and one `contract/<name>` per shared shape, and the specs
  it writes itself as `spec/<id>`; designers, granted the goal's scope,
  write their families' specs as `spec/<id>`. Each spec becomes a
  change's words, copied whole by the coordinator, so the words, never
  the note, are what the change's history keeps; a change's words name
  the notes it reads, which makes them binding for it whatever "notes
  are hints" says. A revised contract notifies nobody: the coordinator
  amends the words of the live changes it concerns. What should outlive
  the goal (the status files, the designs' Later sections) lands with
  the retire (N15).
- **To check:** jig names no bound on an entry's bytes and bounds entries
  per scope; the reference needs an entry of 8 KB (the run's largest
  spec) and about a hundred entries per goal (N20).

### N13. A producer delivers more than once

- **Conflict:** none in design: `agent.md`, 4.3 grants `deliver` mid-run
  to a run with a repository it may write (jig's `domain/hosts.md`,
  section 8; smith's `domain/run.md`, 8.4). Early starts need it, since
  a dependent starts from its dependency's pushed head (2.1).
- **To build:** each delivery pushes the change's branch, the head is
  provisional under the writer slot (jig's `domain/connectors.md`,
  section 11), and a kept dependent's start reads it.

### N14. A finished producer's Left list

- **Conflict:** none in jig; one rule to keep. A change's contract
  requires its checks to pass at delivery (jig's `domain/hosts.md`,
  section 8; `agent.md`, 4.4), and in the run the test-world tasks found
  failures in code they did not own (2.1).
- **Draft rule:** the coding agent's charter says how to finish with what
  is outside the task: push what holds; mark a test that fails for code
  the change does not own as ignored, the fault its reason; list it, and
  anything else found, under Left in its report; never fix what it does
  not own. The change that fixes the fault removes the mark. The
  coordinator reads Left lists as each result wakes it.
- **To check:** the project's `.temper/pre-pr` exists and runs what the
  change owns (format, lints, its package's focused tests, the counted
  baselines); `tasks.result_bytes` holds a summary and a Left list (512
  in `small`, too few).

### N15. The retire, a change from words

- **Conflict:** the coordinator writes no file (N12), and a task takes
  its holds at once when it becomes active (jig's `domain/tasks.md`,
  6.2), so it cannot take a run branch late for the goal's last edits.
- **Recommendation:** once the goal's change has given its result, the
  coordinator makes the retire: a change into main from words, its
  producer under the designer's charter at xhigh (N21), its words the
  status files and Later sections to write and the draft's fate, longer
  text in a note. The coordinator reports once the retire's result
  comes.

### N16. A red at the gate that passes alone

- **Conflict:** fab retries a red test alone before rejecting
  (collateral). temper reruns failed tests alone for tiers
  (`after-landing.md`, 5.2), and, under the turn, reruns a gate's failed
  jobs once when every failing test matches an open flake finding
  (`findings.md`, section 5); under the queue a CI red is a repair.
- **Recommendation:** Later: with one change in the queue there is no
  collateral, and a repair that finds nothing to change says so. Revisit
  if flakes cost repairs.

### N17. The landing model is superseded

- **Recommendation:** `landing-model.md` is superseded by
  `reference-landing.md`. Its items that jig and temper already carry
  stay where they are (placed holds, a conversation's questions, jobs);
  its batches, trains, keys per package, provers, serializer, leases and
  workspaces lent are dropped; landing by turn stays as built, an
  option (`reference-landing.md`, section 9, item by item).
- **The citation sweep** that deletes it must move its citations in
  jig's `domain/tasks.md`, 7 (J5) and 8.5 (J7, J8); temper's
  `forge.md`, 4.4 (F3) and 12.1; `agent.md`, 8.2 and 8.3;
  `landing.md`, 13.1; `testing.md`; `../protocol/setup.md`, 17.1 (F3);
  and `v0.1.0.md`.

### N18. A change names its agents' LLM choices

- **Conflict:** a change's parameters name its producer's spec and
  charter (`forge.md`, 8.1) but no LLM choice, so every producer runs at
  its charter's default; the run's specs chose an effort each (high for
  52, medium for 12), and its foremen ran at medium.
- **Draft rule:** a change's parameters may carry an LLM choice for each
  of its agents' roles (its producer, repairs and resolutions; its
  foremen; its reviews), each checked against its charter's models and
  the change's authority as a batch's is (jig's `domain/tasks.md`,
  section 4); a role without one takes its charter's default.

### N19. Implementers on gpt-6.1-sol

- **Conflict:** smith's only program backend is Claude Code (smith's
  `protocol/programs.md`); the run's implementers were Codex, signed in
  to ChatGPT.
- **Recommendation:** the reference's producers run gpt-6.1-sol on
  smith's direct backend, through an OpenAI account with a key, which
  smith and jig have; Codex as a program backend is Later, and a login
  is never copied to make it so. Its umask, its logs' mode and its
  closed stdin (`AGENTS.md`, 3) are the program backend's to keep when it
  comes.

### N20. Limits for a goal of this size

- **Conflict:** none in design; the profile's numbers. The run's goal had
  64 changes, 185 runs and 54 questions under one coordinator.
- **To check, against the profile the reference deploys with:**
  - G's delegates: about a hundred changes, the designers, person tasks;
  - a change's tasks: a producer, up to four foremen, merge jobs,
    resolutions, repairs, about twelve, where `change` from a chat gives
    six (`agent.md`, 4.3.1) and a change past its allowance holds
    (`forge.md`, 8.4); a collecting change's: its repairs and catch-ups
    over the goal's life, about twenty, each a change with its own;
  - tasks per tree, about eight hundred, and depth: G at the root, a
    change 1 below it, its foreman 2; a repair under B 2, its producer
    and foreman 3; the stock's 18 tasks 4 deep (temper's
    `../protocol/configuration.md`, 13.4) are a chat's;
  - G's inbox (one result's room per live delegate: jig's
    `domain/tasks.md`, 8.2) and open questions per task;
  - the spec and result bytes (N12, N14), the batch size, and a call's
    input bound (`core.call_answer_bytes`, temper's
    `../protocol/configuration.md`, section 7), which a `change` call
    carrying specs must pass, one change per call if it must;
  - notes: an entry's bytes and entries per scope (N12); a charter's
    documents (two per charter in `small`).

### N21. A coordinator makes changes with all their parameters

- **Conflict:** a run gets a change from `change` alone, which takes the
  words, makes one delegate per call and lands it into the repository's
  default branch; a `delegate` of the change procedure is refused,
  naming `change` (`agent.md`, 4.3.1). So no run can make the plan's
  changes: into a goal branch, with dependencies, as one batch, handed a
  branch, collecting, relaying questions, with LLM choices, a producer's
  charter, or naming the red it repairs (`repairs`, `after-landing.md`,
  section 7).
- **Recommendation:** extend `change` rather than open the procedure's
  `delegate`: the root keeps building the delegate from what it knows,
  and the run names only what a plan decides.
- **Draft rule:** `change` takes, beside the words, optional fields: the
  landing branch, by a collecting change's task number or a branch name;
  dependencies, by task number or by a name given in the same call;
  several changes in one call, made as one batch (jig's
  `domain/tasks.md`, section 4); `collect` (N3); `questions` (N1); the
  LLM choices of its agents (N18); a producer's charter among those the
  run may delegate to; and `repairs`, a landing it repairs. Each field
  is checked by the root as the delegate is decided; a charter's
  authority says whether its runs may set them (the designer's charter
  may; a chat's sets none).

### N22. A charter's default wake policy

- **Conflict:** a charter brings a default wake policy (jig's
  `domain/tasks.md`, section 3 and 8.3), but the configuration's
  charters member names none, and a template "names no wake policy"
  (jig's `../protocol/configuration.md`, 3.1 and section 3), so a
  coordinator's policy cannot be configured.
- **Draft rule:** the charters member carries a default wake policy, in
  jig's closed vocabulary (jig's `domain/tasks.md`, 8.3). The designer's
  charter's: questions, escalations, failures, every delegate's result,
  words from its delegates, its person's words, and news its connectors
  class as waking. A task made from a template takes its charter's;
  its creator may still narrow or widen it.

### N23. The case's third edition is replaced

- **Recommendation:** `reference-case.md` is now the condensed case of
  the reference, as the user asked; its third edition (an invented goal
  in six acts) is gone from the draft, readable in history at
  `88d6cb0c2f`.
- **The citation sweep** must move what cited its sections: temper's
  `agent.md`, 8.2 and 8.3 (the golden, frozen with the case);
  `after-landing.md`, section 12; `findings.md`, section 11;
  `landing.md`, section 16; `landing.status.md`; `testing.md`;
  `../client/README.md`; `../protocol/README.md`; and the drafts
  `v0.1.0.md` (rungs B0 to B2, C and D), `qa-live.md` (C1 to C6) and
  `temper-on-temper.md`, deciding for each rung and walk what in the new
  case stands for it.

## 5. Later

- The numbers that would reopen choices: questions per change and the
  share the foreman settles (N1, N8); catch-ups and their conflicts per
  goal (N4); flakes at the gate (N16); the coordinator's wakes and their
  cost at xhigh (N11); the cost of copying specs into words (a spec by
  reference, `reference-code-production.md`, section 17).
- Overlaps between goal branches beside each other, reported (fab's
  overlap probe), once two goals run at once.
