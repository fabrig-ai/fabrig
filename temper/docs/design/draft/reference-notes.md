# Notes behind the reference drafts

Draft, 2026-11-02. What `reference-code-production.md`,
`reference-landing.md` and `reference-case.md` rest on: the run they are
drawn from, the landings on record, today's pipeline in jig's and
temper's terms, and every change to an existing design the drafts need,
N1 to N20, each with a recommendation and a draft rule. Unlike the three
drafts,
these notes name today's pipeline (fab, in the agents repository) and
its runs, since they are the evidence. The decisions are the user's; the
user's answers of 2026-11-02 are in 1.

Unqualified, `forge.md`, `landing.md`, `after-landing.md`, `agent.md`,
`worker.md` and `findings.md` are temper's `../domain/` documents; jig's
and smith's are named as theirs.

## 1. In one page

- **The reference is fabrig's own way of developing itself,** as fab runs
  it today, moved onto temper: a person's draft cut into a plan by a
  designer, Codex implementers filling it, a goal branch checked along
  the way and landed whole, then verified after landing.
- **Code production maps onto jig and temper with configuration and a
  few changes.** jig's primitives cover the structure (a goal, a
  coordinator called fresh on each wake, batches with dependencies,
  questions, holds and escalations, wake batching, jobs). temper's
  change procedure, its goal branch (`forge.md`, 11.1) and its after
  landing cover most of the mechanics. What is missing is section 4.
- **Landing is the queue temper has,** one change per goal into main:
  update, CI at the exact head, merge at that head, then tiers. The
  landing model's batches, trains, keys per package, provers and
  serializer are not needed; the landing by turn stays as built, an
  option the reference does not use.
- **The user's answers (2026-11-02):** implementers ask, through a
  foreman and then the coordinator (N1); a goal's changes land into a
  branch of the goal's, which lands whole (N3); one coordinator per
  goal, its designer's runs at xhigh, mostly configuration (N10, N11);
  these notes hold the design changes.
- **Changed from fab, on purpose:** no answers or repair sections
  committed into specs (the store holds them), no specs rewritten by an
  answerer (an amendment, visible, instead), designers at xhigh with
  surveys as Claude sub-agents at high (Opus 5.5).

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
  which 3 of the cut's 9 reached the tick (the backstop found the rest, 24 lost
  event notes); answer calls that widened `files`, added needs, edited
  another task's spec and made a task, against "a pushed spec never
  changes".

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
| 1 to 5 | the cut: six surveys, five designers, contracts first, eight batches, the first merge at T+27 |
| 6 | sv-skein-setup, the plain path: done in 9.4 minutes, merged 53 seconds later |
| 7, 9 | bx-init and fe-make-git: early starts, fe-make-git done at 15:23 and held until fe-config-drop merged at 15:36 |
| 8 | bx-doc: its foreman failed, an answer call widened `files` by `problem.rs` |
| 10 | fe-remove: an answer call made sv-skein-retire and added it to fe-remove's needs |
| 11 | the old-forms recount at a merge, which never triggered in replay 6 and settled 17 of 24 conflicts in its predecessor |
| 12, 13 | fe-world-remove: a failed early run, then ten runs fixing code it did not own; the case finishes it at its first Left list |
| 14 | fe-demo-gone's nine-file conflict and its two rebases |
| 15 | fe-world-reset's refused early start on fe-demo-gone |
| 16 | fe-world-reset's question on project isolation, which its foreman settled from the draft; the case sends it to the person, a route the run never took |
| 17 | fe-world-remove's widening to jig's `run.rs` (a stale credential), made a change of its own |
| 18, 19 | the checks along the way and repair-5: its question, its answer "you own it", the stall repaired |
| 20 | the catch-up with main, done by hand for playground-kit (23 conflicts) |
| 21 to 23 | the end check, then the landing that never came (`ready = false`); phase-case's landing |
| 24 to 26 | the slow tiers' reds of 10-31, repaired by hand |
| 27 | phase-case's retire through the queue |

## 3. Today's pipeline in jig's and temper's terms

| fab | jig / temper | Where |
|---|---|---|
| the user starts a draft (`fab trunk make`, `fab call cut`) | a person makes a goal, tracked, with the draft's path | jig's `domain/core.md`, 6.2 |
| the cut, answer calls, settle, retire | the goal's coordinator, one agent task, fresh on each wake | jig's `domain/core.md`, 7.2; `domain/engine.md`, 7.4 |
| `plan.toml` parts and needs | batches of changes with dependencies | jig's `domain/tasks.md`, section 4 |
| the cut's early release | several batches | the same |
| `notes/contracts.md`, family notes | files on the goal branch, landed first | N12 |
| task spec | a change's words and parameters | `forge.md`, 8.1 |
| implementer (Codex, high) | the change's producer, the coding agent's charter | `forge.md`, 8.2 |
| precheck and readers' tests in the run | `.temper/pre-pr` before delivery | `agent.md`, 4.5 |
| `Done:` with `## Left` | a result's words | jig's `domain/tasks.md`, 5.6 |
| `Question:` → foreman → answer call → user | the producer asks; foreman delegate; the coordinator; the person | N1 |
| `Infra:` | a transient failure, or a wait for an account | jig's `domain/tasks.md`, 5.5 |
| park at the run limit | tries past their bound: held, escalated | jig's `domain/tasks.md`, 5.5 and 9 |
| `<draft>/integration` | the goal branch | `forge.md`, 11.1; N3 |
| merge into the trunk, light | a change landed into the goal branch under light rules | N6 |
| old-forms recount at a merge | the project's resolver, a job | N7 |
| merge conflict → refusal foreman → rebase run | a resolution, in a merge in progress | `forge.md`, 8.4 |
| early start | a change that starts kept | jig's `domain/connectors.md`, 6.1; N2 |
| check along the way → `repair-<n>` | the goal branch's tier, red → one repair change | N5 |
| end check, catch-up, ready, queue | the goal's change into main | `reference-landing.md`; N4 |
| plan stop (three reds) | a bound past: held, escalated to the coordinator | jig's `domain/tasks.md`, 5.5 |
| after-landing red → red foreman → red task | a red as news to the goal → triage → a repair change | `after-landing.md`, section 7 |
| retire waits for main and its verdict | the goal's change lives to cleared | `after-landing.md`, section 6 |
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
  task under the foreman's charter: read-only, its brief the change's
  words, the question and the asker's report) whose verdict is
  `answer` (with the answer), `finish` (the asker's acceptance holds;
  what is left goes to its report's Left) or `escalate`. On `answer` and
  `finish` the change answers its delegate's question with the
  foreman's words; on `escalate` it asks its requester the same
  question in the question form, and passes the answer down when it
  comes. One foreman per question. The change's own page lists its open
  questions. A procedure decides nothing (jig's `domain/connectors.md`,
  6.3): it relays, and the foreman and the requester decide.
- **Bounded:** a change's `questions` bound (four in the reference);
  past it, the next question gets no foreman: the change holds itself,
  saying so, and the hold escalates to its requester, which decides it
  as any hold (the run's parks at its run limit).
- **Relayed through a procedure above:** a change whose requester is a
  procedure (a goal branch's repair, N5) asks that procedure, which,
  relaying too, asks its own requester the same question, with no second
  foreman.
- **The foreman's charter** is the forge's kind 3, number 3, beside the
  change's and the reviews' (temper's `../protocol/configuration.md`,
  13.2).

### N2. Early starts: a kept change and its producer

- **Conflict:** the change starting kept is designed (jig's
  `domain/connectors.md`, 6.1; `landing.md`, 4.1) but not built
  (`landing.status.md`), and no checkout start gives its producer its
  dependencies' work (`worker.md`, 4.1 and section 5: a landing branch,
  a branch, saved work, or a merge in progress).
- **Recommendation:** build the kept start, and a checkout start that
  merges the dependencies' pushed heads onto the landing branch's tip.
  Carry over the four rules the run had to learn.
- **Draft rule:** a change made with a dependency on another change into
  the same landing branch starts kept. Its producer starts once each
  pending dependency's change has a pushed head and none of them is
  kept itself (one level), from a checkout at the landing branch's tip
  with those heads merged in, in dependency order; a conflict there
  refuses the start, and the change waits for the dependencies to
  land. A change starts early once: once its producer has asked, or
  failed for want of its dependencies' work, the next producer waits
  for them to land. A producer continuing from saved work or its pushed
  branch merges the landing branch's tip into it first; a conflict is
  left in the checkout as a merge in progress. The kept change's merge
  waits for its dependencies, as jig's rule already says. A dependency
  added to a change whose producer waits on a question (an answer that
  makes the work wait for another change) holds the answer at the change
  until the dependency has landed; the producer's next run then merges
  the landing branch's tip first, by the rule above.

### N3. The goal branch

- **Conflict:** `forge.md`, 11.1 allows a goal's own branch in words, but
  nothing says who makes it, when its change becomes ready, or how a
  branch under temper's prefix, which takes an exclusive hold
  (`forge.md`, section 2), is also a landing branch, which is shared
  (`forge.md`, section 9).
- **Recommendation:** the goal branch is the branch of one change, the
  goal's change into main, made in the coordinator's first batch. It
  stays open to the goal's other changes until the coordinator closes
  it.
- **Draft rule:** a change may be made `collecting`, with its own branch
  `temper/<root>/g<task>` created at its landing branch's tip. Its
  branch is a landing branch for the changes of its goal's subtree that
  name it: their merges into it are effects that take its writer slot
  (jig's `domain/tasks.md`, 6.3), ordered by its queue. A collecting
  change is not ready while its requester has not closed it (an
  amendment of the parameter) or while any change landing into its
  branch is live; then it goes its way into its own landing branch as
  any change, opening its pull request only then, so CI does not run on
  every merge into it. Once no change into its branch is live, its step
  tells its requester so, in words, so a coordinator wakes to close it.
  Closed and on its way, it refuses a new change into its branch at the
  batch, naming itself; its requester may reopen it by the same
  amendment, which takes it out of the queue until it is closed again.

### N4. A goal branch catches up with main along the way

- **Conflict:** "It never updates only because its base moved"
  (`forge.md`, 10) against "it takes the base's landings that touch its
  area as they come, by an update" (`forge.md`, 11.1). The run's
  predecessors left catching up to the end, by hand (2.2).
- **Recommendation:** a collecting change updates from its landing branch
  while it collects, at most once per span, never while a resolution or
  a repair into its branch is open.
- **Draft rule:** a collecting change's `catch_up` span (the project's
  setting, 30 minutes by default; none turns it off). While it
  collects, when its landing branch has moved and the span has passed
  since its last update, it updates (the forge's merge of the base, as
  `forge.md`, 10). A conflict is a resolution into the goal branch, its
  brief naming the base's sha and the conflicting files; three
  resolutions in a row that do not bring the base in hold the change,
  escalated to its requester.

### N5. A goal branch is checked along the way

- **Conflict:** a landing branch's own CI pauses its queue while red and
  makes one repair (`forge.md`, section 9), and tiers follow landing
  branches with landings that live to cleared (`after-landing.md`,
  sections 3 and 6). The run kept merging while a repair ran, and its
  inner changes never waited for a verdict.
- **Recommendation:** a goal branch is checked by a tier of its own,
  which coalesces (one run at a time, at the newest tip); a red makes
  one repair change into the goal branch, mechanically; merges go on.
  Inner changes end at landed.
- **Draft rule:** a project's tiers may name goal branches, with their
  own definitions (fabrig: the full gate less its fuzzy suite). A tier
  run on a goal branch judges no landing: a red, rerun alone as
  `after-landing.md`, 5.2 says, makes one repair change into the goal
  branch, made by the collecting change's step, its words the failing
  tests and the log's end, and the changes not yet landed into the
  branch with the files they name, to leave alone. No new run starts
  while that repair is live; the queue into the goal branch is not
  paused. Changes landed into a goal branch have no verification and
  end at landed. Reds of a goal branch's tier never hold the collecting
  change; its own CI into main does (`reference-landing.md`).

### N6. Light landing rules for a goal branch

- **Conflict:** a change always has a checking step, CI on its head
  (`forge.md`, 8.2, step 3), and the no-CI choice is not built
  (`forge.status.md`, 8.7).
- **Recommendation:** landing rules that require nothing make the
  checking step pass at once; the project's rules for goal branches
  require nothing but a mergeable head.
- **Draft rule:** a landing rule covering `temper/*/g*` with no CI, no up
  to date and no gate; a change into a branch whose rules require no CI
  skips checking. The merge style is a merge commit.

### N7. A mechanical resolver before an LLM

- **Conflict:** under the queue an update or merge in conflict goes to a
  resolution, an agent (`forge.md`, 8.4 and 10); the project's tooling
  that settles known conflicts is the landing by turn's only
  (`landing.md`, 5.3), and jobs are designed, not built (jig's
  `domain/hosts.md`, 6.9). In the run's predecessor 17 of 24 merges met
  conflicts in counted baselines, which a recount settles.
- **Recommendation:** build jobs, and a project's resolver job the
  change runs on a conflicting update or merge before a resolution.
- **Draft rule:** a project's `resolver` setting names a job kind. On a
  conflict, the change delegates the job in a merge in progress; done,
  its delivery is the merge; failed or leaving conflicts, a resolution
  follows as today. Until jobs are built, a resolution agent with the
  repository's rule in `AGENTS.md`.

### N8. Answers in batches

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

### N10. Designers at xhigh, surveys as sub-agents at high

- **Conflict:** the LLM choice carries a main model and effort and its
  sub-agents' (jig's `domain/tasks.md`, section 3), but smith offers no
  sub-agents on a program backend (smith's `protocol/programs.md`,
  section 2; its open question in section 9), and the coordinator runs
  on Claude Code.
- **Recommendation:** answer smith's open question: serve smith's
  `sub_agent` to the program through its MCP server, each child a
  program session of its own with the model and effort the LLM choice
  names. Until then surveys are read-only delegate tasks.
- **Draft rule:** the goal template's LLM choice is `{"main": {"model":
  "claude-opus-5-5", "effort": "xhigh"}, "sub_agents": [{"model":
  "claude-opus-5-5", "effort": "xhigh"}, {"model": "claude-opus-5-5",
  "effort": "high"}]}`; smith serves `sub_agent` on Claude Code under
  it, each call naming its LLM among those (smith's `domain/run.md`,
  5.3): designers at xhigh, surveys at high.
- **Designers return text.** A sub-agent that writes holds the run's
  lease and runs alone (smith's `domain/run.md`, 5.3), so designers are
  inspect-only and answer with their family's specs, side by side. A
  family's specs (the run's: 10 to 16 specs of 3 to 8 KB) must fit a
  child's answer cap; check it, and size the designer charter's for it,
  or ask a designer for its family in parts.

### N11. One coordinator, several kinds of wake

- **Conflict:** none; a cost. One task has one LLM choice, so the
  coordinator answers questions at xhigh where fab's answer calls ran at
  medium (6.7M of the run's 11.0M Claude weight).
- **Recommendation:** accept it for now (the user, 2026-11-02). If the
  numbers ask, an LLM choice per kind of wake (a question, an
  escalation, a delegate's end) on the charter, which jig would add to
  the run's check.

### N12. Shared notes and specs

- **Conflict:** a task's spec is words bounded by the deployment's limits
  (jig's `domain/tasks.md`, section 11); fab's specs and shared notes
  are files of several kilobytes, read in the checkout.
- **Recommendation:** shared notes and contracts are files the
  coordinator writes on its own branch (`forge.md`, 11) and hands to the
  first change into the goal branch, which every other change depends
  on; a change's words carry its spec and name the notes it reads.
  Check the spec bound against the run's largest spec before choosing
  a profile.
- **Written once.** The coordinator's run branch is handed to that first
  change (`forge.md`, 11), so the notes are written before it is made;
  a later ruling goes into the words of the changes it concerns, by
  amendment, and into the coordinator's own notes, never into the
  files. The run's contract changes (J8, T4, T8) all came during its cut.
- **Specs are words, never files.** What changes a spec is an amendment,
  kept in the change's history, which replaces the run's answer calls
  editing specs in git.
- **A result holds a Left list.** A producer's result words must hold
  its summary and its Left list; check `tasks.result_bytes` against the
  run's reports (512 in `small`, too few).

### N13. A producer delivers more than once

- **Conflict:** a producer's tools end with `finish` and a change
  (`agent.md`, section 8); early starts need its work pushed as it goes
  (2.1).
- **Draft rule:** producers are granted `deliver` mid-run (jig's
  `domain/hosts.md`, section 8); each delivery pushes the change's
  branch, the head is provisional under the writer slot (jig's
  `domain/connectors.md`, section 11), and a kept dependent's start
  reads it.

### N14. A finished producer's Left list

- **Conflict:** none. The coding agent's charter text says how to finish
  with what is outside the task: push what holds, list the rest under
  Left, and for a task that writes stories over code it does not own,
  report the failures as Left rather than fix them (2.1, the test-world
  tasks). The coordinator reads Left lists when it wakes.

### N15. Where the coordinator's work lands after the goal

- **Conflict:** none in intent. Once the goal's change has landed and
  cleared, the coordinator's last run writes the status files and the
  designs' Later sections as a small change into main, then reports.
- **To check:** the coordinator's run branch was handed to the notes
  change (N12) and deleted when it closed (`forge.md`, 11). The draft
  rule: a task's run branch, once its change has closed, is made again
  by its next push, at the tip of the branch the run started from, and
  held by the task as before.

### N16. A red at the gate that passes alone

- **Conflict:** fab retries a red test alone before rejecting
  (collateral); temper reruns failed tests alone for tiers
  (`after-landing.md`, 5.2) and files a flake at a gate as a finding
  (`findings.md`, section 5), but a queue's CI red goes to a repair.
- **Recommendation:** Later: with one change in the queue there is no
  collateral, and a repair that finds nothing to change says so. Revisit
  if flakes cost repairs.

### N17. The landing model is superseded

- **Recommendation:** `landing-model.md` is superseded by
  `reference-landing.md`. Its items that jig and temper already carry
  stay where they are (placed holds, a conversation's questions, jobs);
  its batches, trains, keys per package, provers, serializer, leases and
  workspaces lent are dropped; landing by turn stays as built, an
  option (`reference-landing.md`, section 9, item by item). Deleting the
  draft waits for the citation sweep, which must also move the
  citations of its J7 and J8 in jig's `domain/tasks.md`, 8.5 and
  temper's `agent.md`, 8.2, and of the old case in `agent.md`, 8.2.

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
  delegates per task (G's: a hundred), tasks per tree (each change with
  a producer, foremen, resolutions and repairs: about six hundred), the
  tree's depth (G, a change, its foreman: three below the root, four
  with a repair's change under B), G's inbox (one result's room per live
  delegate: jig's `domain/tasks.md`, 8.2), open questions per task, and
  the spec and result bytes (N12). The stock's 18 tasks 4 deep
  (temper's `../protocol/configuration.md`, 13.4) are a chat's.

## 5. Later

- The numbers that would reopen choices: questions per change and the
  share the foreman settles (N1, N8); catch-up conflicts per goal (N4);
  flakes at the gate (N16); answers' cost at xhigh (N11).
- Overlaps between goal branches beside each other, reported (fab's
  overlap probe), once two goals run at once.
