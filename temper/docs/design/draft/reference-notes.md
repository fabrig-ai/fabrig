# Notes behind the reference drafts

Draft, 2026-11-02. What `reference-code-production.md`,
`reference-landing.md` and `reference-case.md` rest on: the run they are
drawn from, the landings on record, today's pipeline in jig's and
temper's terms, and every change to an existing design the drafts need,
N1 to N24, each with a recommendation and a draft rule. Unlike the
three drafts, these notes name today's pipeline (fab, in the agents
repository) and its runs, since they are the evidence. The decisions
are the user's; the user's answers of 2026-11-02 are in 1.

Unqualified, `forge.md`, `landing.md`, `after-landing.md`, `agent.md`,
`worker.md` and `findings.md` are temper's `../domain/` documents; jig's
and smith's are named as theirs.

## 1. In one page

- **The reference is fabrig's own way of developing itself,** as fab runs
  it today, moved onto temper: a person's draft turned into design and a
  plan by designers, Codex implementers filling it, a goal branch checked
  along the way and landed whole, then verified after landing.
- **Code production maps onto jig with configuration, and onto temper
  with a few changes.** jig's primitives cover the structure (a goal, a
  coordinator called fresh on each wake, batches with dependencies,
  holds and writer slots, questions, person tasks, escalations, notes,
  jobs). temper's change procedure and its after landing cover most of
  the mechanics; what temper offers a coordinator over them is narrower
  than a plan needs (N22), and its goal branch is a change where the
  reference needs it to be the goal's (N3). What is missing is section 4.
- **Landing is the queue temper has,** one change per goal into main:
  update, CI at the exact head, merge at that head, then tiers. The
  landing model's batches, trains, keys per package, provers and
  serializer are not needed; the landing by turn stays as built, an
  option the reference does not use.
- **The user's answers (2026-11-02):** implementers ask, through a
  foreman and then the coordinator (N1); a goal's changes land into a
  branch of the goal's, which lands whole (N3); one coordinator per
  goal, at xhigh, mostly configuration (N10, N11); these notes hold the
  design changes. Then, on the plan's text: a goal branch everybody
  writes, designers included, who put their output in `docs/design` for
  this project while other projects do as they choose; notes only where
  they are the right tool; and the ideal shape before the mechanisms at
  hand (N3, N12, N13).
- **Notes, judged** (N12): the first revision of these drafts put the
  plan's text in notes. That spared the coordinator any writing, but it
  used jig's hints ("where a note and the facts disagree, the facts are
  right") as binding contracts, notified no one of a revision, and had
  the coordinator copy each spec into a change's words. The plan's text
  is the project's design, so it belongs in the repository, on the
  goal's branch, versioned and landed with the code it governs. Notes
  keep jig's use: what runs learn.
- **Changed from fab, on purpose:** the goal's branch held by the goal,
  every write of the goal landing into it by change; designers as
  changes, their design landed into the goal's branch, not specs on a
  trunk; specs as words citing the design; no answers or repair sections
  committed into specs (the store holds them); no specs rewritten by an
  answerer (an amendment, visible, instead); reds along the way go to the
  coordinator as news, as reds after landing do; the close-out before
  landing, not a retire after it; designers at xhigh with surveys as
  Claude sub-agents at high (Opus 5.5).

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
- **Mechanisms that exist only because coordination state is files:**
  answers and repair sections committed into specs; wakes through a
  shared file, of which 3 of the cut's 9 reached the tick (the backstop
  found the rest, 24 lost event notes); answer calls that widened
  `files`, added needs, edited another task's spec and made a task,
  against "a pushed spec never changes"; designers' contract requests
  through progress files and the cut's messages.

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

## 3. Today's pipeline in jig's and temper's terms

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
  settles what the change's own words and the design settle; the rest
  goes up.
- **Draft rule:** a change's `questions` parameter, from its maker or the
  project's setting: `none` (today), `foreman` or `up`. With either of
  the last two, its producer, repairs and resolutions keep the Message
  family, and a question one asks reaches the change.
  - With `foreman`, its step delegates a foreman (an agent task under
    the foreman's charter, read-only, its brief the change's words, the
    question and the change's pushed head, its checkout the landing
    branch's design) whose verdict is `answer` (with the answer),
    `finish` (the work done holds; what is left goes to the asker's Left
    list) or `escalate`. On `answer` and `finish` the change answers its
    delegate's question with the foreman's words; on `escalate` it asks
    its requester.
  - With `up`, it asks its requester at once: a designer's change, whose
    questions are about the draft.
  - Asking its requester, the change uses the question form and is kept
    by it (jig's `domain/tasks.md`, 5.7), which is harmless: a
    procedure's step goes on while kept, and escalations skip it anyway.
    It passes the answer down when it comes. One foreman per question.
    The change's own page lists its open questions. A procedure decides
    nothing (jig's `domain/connectors.md`, 6.3): it relays, and the
    foreman and the requester decide.
- **Bounded:** a change's `questions` bound (four in the reference);
  past it, the next question gets no foreman: the change holds itself,
  saying so, and the hold escalates to its requester (the run's parks at
  its run limit). Released, the change relays the question that waits to
  its requester with no foreman.
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
  says. Designers' changes start early on the shared design by the same
  rule.
- **A dependency added while the producer waits on a question** (an
  answer that makes the work wait for another change, a design revision
  among them) holds the answer at the change until the dependency has
  landed; the producer's next run then merges the landing branch's tip
  first, by the rule above.
- **A dropped dependency takes its early dependent with it:** the
  dependent's head carries the dependency's unlanded work. The forge
  refuses an amendment removing a dependency whose pushed head the
  change's producer started from, naming it, and the coordinator cancels
  and remakes the dependent instead.

### N3. The goal's own branch

- **Conflict:** `forge.md`, 11.1 makes a goal's own branch "a change of
  its own, landed by the change procedure like any other", and
  `forge.md`, 11 has that change create it at its base's tip, as an
  effect. Such a change is at once the landing branch of its goal's
  changes and the branch of its own change, and three rules stand
  against it: a branch under temper's prefix takes an exclusive hold
  (`forge.md`, section 2), only its holder's delegates write under a hold
  (jig's `domain/tasks.md`, 6.1), and a forward move by anyone but the
  change's own writers is drift (`forge.md`, 8.5). A landing branch must
  also exist when named (`forge.md`, 11), so the goal's changes cannot be
  made in the batch that makes it. The first revision of these drafts
  patched them with a "collecting change" that held nothing, made repair
  and catch-up changes into its own branch, and was closed and reopened
  by amendment: a change carrying a goal's lifetime.
- **Recommendation:** the branch is the goal's, not a change's. The goal
  holds it for its whole life; everything of the goal writes it under
  that hold, one writer at a time, as jig's writer slot already orders
  (jig's `domain/tasks.md`, 6.3: "whether its task holds the resource or
  writes under a task above it that does"); its landing is an ordinary
  change into main whose branch is its requester's.
- **Draft rules:**
  - **Chosen by template.** A goal's own branch is part of the shape of
    work, so the template for that shape names it. jig's templates name
    no resources (jig's `../protocol/configuration.md`, 3.1), since a
    concrete resource depends on who makes the task; they may name the
    symbolic resources a connector names after a task, which depend on
    nothing but the task, resolved as a batch's are once the task has
    its number (jig's `domain/authority.md`, section 4: "its own
    environment", "its runs' branches"). The forge offers one, *its own
    goal branch*: `temper/<root>/g<root>` in the project's home
    repository, on a tracked task only. The reference's template for
    drafts names it; a small goal's template does not, and that goal's
    changes land into main each, as `forge.md`, 11.1 advises for most
    work. A project offers both templates through its policy, and a
    person chooses the shape by choosing the template; a goal made by
    hand may name it too, within its creator's authority. The choice is
    made with the goal, since a goal cannot take a hold once active
    (jig's `domain/tasks.md`, 6.2). The jig change is small and not
    temper's alone: an incident template could name its own
    environment the same way.
  - **Made as the hold is taken.** In jig, a connector says what taking a
    hold does, as it says what a release does (jig's
    `domain/connectors.md`, 3.3; `domain/tasks.md`, 6.5). The forge makes
    a goal's branch at its base's tip (the repository's default) when its
    goal takes the hold; failing, it holds the goal, saying why.
  - **A landing branch, held.** It has a queue as any landing branch
    (`forge.md`, section 9) and is the one landing branch that takes a
    hold: its goal's. A change into it must be in its holder's subtree;
    any other is refused at its batch, naming the holder. Its merges are
    the connector's effects on a held resource and take its writer slot
    (jig's `domain/tasks.md`, 6.3), one at a time, in its queue's order.
  - **Writer slots go to writers.** A run takes a writer slot only for a
    resource it may write; the goal's own runs inspect alone (`agent.md`,
    4.3) and take none, so a coordinator's run at xhigh never stops a
    merge.
  - **A change on its requester's branch.** A change's branch may be its
    requester's held branch, written under that hold; it has no
    producing step, as a handed branch has none (`forge.md`, 8.2). Its
    repairs, its resolution and the update it asks for take the writer
    slot as they write. A forward move by temper's identity under its
    requester's hold (a merge of another of its requester's delegates)
    is its new head, read and checked again, gates asked again where they
    count; never drift (`forge.md`, 8.5). A move by anyone else is drift,
    as 8.5 says. Cancelled, such a change closes its pull request and
    leaves the branch, whose hold is its requester's (`forge.md`, 8.6).
  - **Released as the goal closes.** Its branch is deleted if a change
    landed it (its tip reachable from that change's landing branch), and
    otherwise kept for a person to read, until deleted by hand or by the
    deployment's retention; `forge.md`, 11's "until its tree's root
    closes" does not serve a root's own branch.
  - **Everyone writes it by change.** Each writer pushes its own branch
    and lands it into the goal's branch by its queue. Direct pushes by
    many producers would race, and a run holding the branch's writer
    slot for its whole length would stop every other writer. Only the
    landing's own repairs and resolution push it directly, when nothing
    else should land.
- **What it removes:** the collecting change's open and closed states,
  its own repairs and catch-ups, its relaying of questions, and its
  holding nothing. `forge.md`, 11.1 changes from "a change of its own"
  to "the goal's own, held by it, landed by a change on it".

### N4. Light rules for changes into a goal's branch

- **Conflict:** a change always has a checking step, CI on its head
  (`forge.md`, 8.2, step 3), is updated when first if it lacks its base's
  tip (8.2, step 6; section 10), and waits on its own for CI and its
  base's tip whatever the rules say (8.7). A merge the forge refuses for
  a conflict has no path: resolutions come from refused updates (section
  10).
- **Recommendation:** checking and updating follow the landing rules,
  and the rule for goal branches asks neither. One rule on `… branch
  temper`, its last segment open, covers every goal's branch (jig's
  `domain/authority.md`, section 4: an open segment covers every name
  beneath), since a change's own branches and saved work are never
  landing branches.
- **Draft rule:** a change's checking step runs only when its landing
  branch's rule requires CI passed, and its update when first only when
  the rule requires it up to date. The reference's rule for goal
  branches requires neither: a change into one merges at its head once
  the forge finds it mergeable. A merge the forge refuses for a conflict
  delegates the merge job (N7) on the change's branch, the landing
  branch's tip as its base; done, the change's new head is the merge;
  left with conflicts, a resolution follows in that merge in progress,
  as `forge.md`, 8.4 has it; then it merges. Its resolutions' bound
  holds it as ever. The repository's merge style is a merge commit (a
  setting of the repository, `forge.md`, 4.4).

### N5. Tiers that name their branches; a goal branch's tier

- **Conflict:** each of a repository's tiers runs on every landing
  branch's moves, and the landings it covers live to cleared
  (`after-landing.md`, sections 1, 3 and 6), so main's slow tiers would
  also follow every goal's branch, and each change into one would wait
  for their verdict. A landing branch's own CI pauses its queue while red
  and makes one repair (`forge.md`, section 9); the run kept merging
  while a repair ran.
- **Recommendation:** a tier names the branches it follows. A goal's
  branch is followed by a tier of its own, which judges no landing; its
  reds go to the goal as news, as reds after landing do
  (`after-landing.md`, section 7), and the coordinator triages them and
  decides the repair, one mechanism for reds along the way and after.
  Merges go on.
- **Draft rule:** a tier's settings name the branches it follows: a
  landing branch by name, or `goal branches`, every goal's own branch;
  main's tiers name main. A tier following goal branches runs at a
  branch's newest tip at most once per its `span` (20 minutes in the
  reference), judges no landing, and blames no one: a red, rerun alone
  as `after-landing.md`, 5.2 says, is news to the branch's goal: the
  tier, the failing tests, the run and the end of each failed job's log.
  A branch no tier judging landings follows clears its landings as they
  land (`after-landing.md`, section 2), so changes into a goal's branch
  end at landed. Its reds hold nothing; the goal's landing's CI into main
  judges the whole (`reference-landing.md`).

### N6. Catching up along the way

- **Conflict:** "It never updates only because its base moved"
  (`forge.md`, 10) against "it takes the base's landings that touch its
  area as they come, by an update" (`forge.md`, 11.1); and the forge's
  update is a pull request's (kind 11), which a goal's branch has not
  until its landing is made. The run's predecessors left catching up to
  the end, by hand (2.2).
- **Recommendation:** the goal catches up by a change into its branch,
  its producer the merge job, made by its coordinator when main's
  landings touch its open work (`forge.md`, 7.3), paced by its wake
  policy (N23).
- **Draft rule:** a catch-up is a change into the goal's branch whose
  producer is the merge job (N7) given main's tip: its own branch made at
  the goal branch's tip, main merged in, a resolution after the job for
  what it leaves, its words naming main's commit. It lands under the
  light rules (N4). `forge.md`, 11.1's "by an update" becomes "by a
  catch-up change".

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
  serves three places: a merge into a goal's branch refused for a
  conflict (N4); a catch-up's producing (N6); and, before a resolution,
  an update the forge refused (`forge.md`, 8.4 and 10), the goal's
  landing's among them. Until jobs are built, a resolution agent with
  the repository's rule in `AGENTS.md`.

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
  "claude-opus-5-5", "effort": "high"}]}`, and the designers' changes G
  makes give their producers the same (N19); smith serves `sub_agent` on
  Claude Code under it, inspect-only children side by side (smith's
  `domain/run.md`, 5.3). Designers are changes' producers, not
  sub-agents (N13), so no child writes and no child's answer carries a
  family's design.

### N11. One coordinator, several kinds of wake

- **Conflict:** none; a cost. One task has one LLM choice, so the
  coordinator answers questions, reads each change's result, triages
  each red of its branch's tier and decides each catch-up at xhigh
  (N23), where fab's answer calls ran at medium (6.7M of the run's 11.0M
  Claude weight). In the run's terms, about 170 wakes, fewer where
  messages reach a run already going.
- **Recommendation:** accept it for now (the user, 2026-11-02). If the
  numbers ask, an LLM choice per kind of wake (a question, an
  escalation, a delegate's result, news) on the charter, which jig would
  add to the run's check; or a rule for catch-ups and repairs along the
  way, made by a procedure rather than judged
  (`reference-code-production.md`, section 17).

### N12. The plan lives in the goal's branch; notes stay hints

- **The choice:** where the plan's text lives (the shared shapes, each
  family's design, the status of the code, each spec). Three homes were
  weighed:
  - **files on a trunk,** fab's: the coordinator writes and pushes them,
    which a goal's runs may not (`agent.md`, 4.3), and fab put its
    coordination state there too (2.1);
  - **notes in the goal's scope,** this draft's first revision: no
    writing by the coordinator, revisions fenced, readable by every task
    of the project (jig's `domain/engine.md`, section 10). But notes are
    hints, "where a note and the facts disagree, the facts are right",
    and the plan's contracts are binding; a revision notifies no one, so
    the coordinator amended live changes by hand; a spec was copied from
    its note into a change's words; and the plan's text outlived the goal
    only by a retire that copied it again into the design;
  - **the project's design in the goal's branch:** written by designers'
    changes (N13), versioned, merged and landed with the code it
    governs; read in each producer's checkout; revised by a change, whose
    landing is a fact every later checkout has.
- **Recommendation:** the third. The plan's text is design, and design
  lives in the repository. Coordination state stays in the store: tasks,
  words, questions, answers, holds. Notes keep jig's meaning: what runs
  learn and pass on.
- **Draft rule:**
  - the coordinator writes no file and needs none: the plan's shape is
    its tasks and their dependencies;
  - shared shapes and each family's design are designers' changes into
    the goal's branch (N13); a spec is a change's words citing the design
    by path and section, and the status entries it closes;
  - a revision is a designer's change into the goal's branch; once it
    lands, the coordinator amends the words of the live changes citing
    it, naming the revision, and their producers merge the branch's tip
    first (N2);
  - where a project keeps its design is its designer's charter's: here
    `docs/design`, a status file beside each document and its Later
    section; another project says its own;
  - notes hold what runs learn (a flaky test, a slow service, a dead end,
    a trap in the code) in the goal's or the repository's scope; the
    coordinator may keep its reasoning between wakes in a note in the
    goal's scope, a hint its tree overrules.
- **What it removes:** the plan's notes and their bounds, the copying of
  specs, the amendments that stood in for notifications, and the retire's
  second copy (N16).

### N13. Designers are changes

- **Conflict:** a change's producer runs under the forge's charter for
  changes (kind 3, number 1: `forge.md`, 8.9, "a change keeps its
  charter"; its procedure's authority grants that charter's delegation
  alone), `change` names none (`agent.md`, 4.3.1), and a change strips
  the Message family from its agents (N1).
- **Recommendation:** a change names its producer's charter among those
  its maker may delegate to; a designer's change is one whose producer
  runs under the designer's charter. It lands into the goal's branch as
  any change does, with no rule of its own.
- **Draft rule:** `change`'s `producer` (N22): a charter's number (the
  designer's, the coding agent's) or the merge job (N6). A designer's
  change relays its producer's questions straight to its requester
  (`questions: up`, N1), and its producer's report reaches the
  coordinator in its result (N15). The designer's charter: the workspace
  written and delivered, sub-agents at its LLM choice, `message` for
  questions; its documents the project's design rules; its report lists
  its family's changes, each with a name, the design sections and status
  entries it closes, what it owns, its dependencies and its effort.
- **Why tasks, not sub-agents:** a design is a commit, and lands as code
  does; a sub-agent's answer would carry a family's design through the
  coordinator's context, and could not be asked about later.

### N14. A producer delivers more than once

- **Conflict:** none in design: `agent.md`, 4.3 grants `deliver` mid-run
  to a run with a repository it may write (jig's `domain/hosts.md`,
  section 8; smith's `domain/run.md`, 8.4). Early starts need it, since
  a dependent starts from its dependency's pushed head (2.1).
- **To build:** each delivery pushes the change's branch, the head is
  provisional under the writer slot (jig's `domain/connectors.md`,
  section 11), and a kept dependent's start reads it.

### N15. A change's result carries its producer's report

- **Conflict:** a change's result is its landing's facts (`forge.md`,
  8.2, step 7: the merge, the pull request, "Landed: …", then the checks,
  repairs and reviews the landing passed), and its producer's report
  stops at the change; the coordinator needs it, for a Left list or a
  designer's list of changes. A change's contract also requires its
  checks to pass at delivery (jig's `domain/hosts.md`, section 8;
  `agent.md`, 4.4), and in the run the test-world tasks found failures in
  code they did not own (2.1).
- **Draft rule:** a change's result words carry, after its landing's
  three lines, its producer's last report, then what the landing passed;
  cut from the end to the result's bound, and marked. The coding agent's
  charter says how to finish with what is outside the task: push what
  holds; mark a test that fails for code the change does not own as
  ignored, the fault its reason; list it, and anything else found, under
  Left in its report; never fix what it does not own. The change that
  fixes the fault removes the mark. The coordinator reads each result as
  it wakes it.
- **To check:** the project's `.temper/pre-pr` exists and runs what the
  change owns (format, lints, its package's focused tests, the counted
  baselines); `tasks.result_bytes` holds a report and a Left list, and a
  designer's list of eleven changes (512 in `small`, too few: N21).

### N16. The close-out lands with the goal

- **Conflict:** none in jig; a choice. fab retires after the landing and
  its verdict, as a second landing through the queue (2.2: phase-case's
  retire); this draft's first revision did the same, as a change from
  words into main.
- **Recommendation:** what the goal leaves for the next one (the status
  files, the designs' Later sections, the draft's fate) is written into
  the goal's branch before its landing, by a designer's change, so it
  lands with the code it describes, in one landing checked whole, and a
  goal that never lands leaves nothing half-said on main. A repair after
  landing that changes it carries the edit itself.
- **Draft rule:** the coordinator's charter: once the plan is done, make
  the close-out, a change into the goal's branch under the designer's
  charter, its words the status files, the Later sections and the
  draft's fate; once it lands, make the goal's landing.

### N17. A red at the gate that passes alone

- **Conflict:** fab retries a red test alone before rejecting
  (collateral). temper reruns failed tests alone for tiers
  (`after-landing.md`, 5.2), and, under the turn, reruns a gate's failed
  jobs once when every failing test matches an open flake finding
  (`findings.md`, section 5); under the queue a CI red is a repair.
- **Recommendation:** Later: with one change in the queue there is no
  collateral, and a repair that finds nothing to change says so. Revisit
  if flakes cost repairs.

### N18. The landing model is superseded

- **Recommendation:** `landing-model.md` is superseded by
  `reference-landing.md`. Its items that jig and temper already carry
  stay where they are (placed holds, a conversation's questions, jobs);
  its batches, trains, keys per package, provers, serializer, leases and
  workspaces lent are dropped; landing by turn stays as built, an option
  (`reference-landing.md`, section 8, item by item).
- **The citation sweep** that deletes it must move its citations in
  jig's `domain/tasks.md`, 7 (J5) and 8.5 (J7, J8); temper's
  `forge.md`, 4.4 (F3) and 12.1; `agent.md`, 8.2 and 8.3;
  `landing.md`, 13.1; `testing.md`; `../protocol/setup.md`, 17.1 (F3);
  and `v0.1.0.md`.

### N19. A change names its agents' LLM choices

- **Conflict:** a change's parameters name its producer's spec and
  charter (`forge.md`, 8.1) but no LLM choice, so every producer runs at
  its charter's default; the run's specs chose an effort each (high for
  52, medium for 12), its foremen ran at medium, and the reference's
  designers run at xhigh with sub-agents at high.
- **Draft rule:** a change's parameters may carry an LLM choice for each
  of its agents' roles (its producer, repairs and resolutions; its
  foremen; its reviews), each checked against its charter's models and
  the change's authority as a batch's is (jig's `domain/tasks.md`,
  section 4); a role without one takes its charter's default.

### N20. Implementers on gpt-6.1-sol

- **Conflict:** smith's only program backend is Claude Code (smith's
  `protocol/programs.md`); the run's implementers were Codex, signed in
  to ChatGPT.
- **Recommendation:** the reference's producers run gpt-6.1-sol on
  smith's direct backend, through an OpenAI account with a key, which
  smith and jig have; Codex as a program backend is Later, and a login
  is never copied to make it so. Its umask, its logs' mode and its
  closed stdin (`AGENTS.md`, 3) are the program backend's to keep when it
  comes.

### N21. Limits for a goal of this size

- **Conflict:** none in design; the profile's numbers. The run's goal had
  64 changes, 185 runs and 54 questions under one coordinator.
- **To check, against the profile the reference deploys with:**
  - G's delegates: about a hundred changes over the goal's life (the
    designers', the plan's, catch-ups, repairs along the way, the
    close-out, the landing, repairs after landing), and person tasks;
  - a change's tasks: a producer, up to four foremen, merge jobs,
    resolutions, repairs, about twelve, where `change` from a chat gives
    six (`agent.md`, 4.3.1) and a change past its allowance holds
    (`forge.md`, 8.4);
  - tasks per tree, about eight hundred, and depth: G at the root, a
    change 1 below it, its producer, foreman or resolution 2; the
    stock's 18 tasks 4 deep (temper's `../protocol/configuration.md`,
    13.4) are a chat's;
  - G's inbox (one result's room per live delegate: jig's
    `domain/tasks.md`, 8.2) and open questions per task;
  - the words of a change and the bytes of a result (N15), the batch
    size, and a call's input bound (`core.call_answer_bytes`, temper's
    `../protocol/configuration.md`, section 7), which a `change` call
    carrying a family's changes must pass, one change per call if it
    must;
  - a charter's documents (two per charter in `small`).

### N22. A coordinator makes changes with all their parameters

- **Conflict:** a run gets a change from `change` alone, which takes the
  words, makes one delegate per call and lands it into the repository's
  default branch; a `delegate` of the change procedure is refused,
  naming `change` (`agent.md`, 4.3.1). So no run can make the plan's
  changes: into the goal's branch, on its own held branch, with
  dependencies, as one batch, relaying questions, with LLM choices, a
  producer's charter or the merge job, or naming the red it repairs
  (`repairs`, `after-landing.md`, section 7).
- **Recommendation:** extend `change` rather than open the procedure's
  `delegate`: the root keeps building the delegate from what it knows,
  and the run names only what a plan decides.
- **Draft rule:** `change` takes, beside the words, optional fields: the
  landing branch, `goal` for the goal's own branch or a branch's name;
  the branch, `mine` for its requester's held branch (N3); dependencies,
  by task number or by a name given in the same call; several changes in
  one call, made as one batch (jig's `domain/tasks.md`, section 4);
  `questions` (N1); the LLM choices of its agents (N19); the producer, a
  charter among those the run may delegate to, or the merge job (N13,
  N6); and `repairs`, a landing it repairs. Each field is checked by the
  root as the delegate is decided; a charter's authority says whether
  its runs may set them (the coordinator's charter may; a chat's sets
  none).

### N23. A charter's default wake policy

- **Conflict:** a charter brings a default wake policy (jig's
  `domain/tasks.md`, section 3 and 8.3), but the configuration's
  charters member names none, and a template "names no wake policy"
  (jig's `../protocol/configuration.md`, 3.1 and section 3), so a
  coordinator's policy cannot be configured.
- **Draft rule:** the charters member carries a default wake policy, in
  jig's closed vocabulary (jig's `domain/tasks.md`, 8.3). The
  coordinator's charter's: questions, escalations, failures, every
  delegate's result, words from its delegates, its person's words, at
  once; news its connectors class as waking (a red of a tier, a landing
  that touches the goal's open work), batched by age, at most once in 30
  minutes. A task made from a template takes its charter's; its creator
  may still narrow or widen it.

### N24. The case's third edition is replaced

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
  goal (N6); flakes at the gate (N17); the coordinator's wakes and their
  cost at xhigh (N11); the size of designers' reports (N15, N21).
- **Several goals at once.** A mechanical probe merges each pair of live
  goal branches, and each with main, without pushing (`git merge-tree`),
  and sends their conflicts to both goals as news (fab's overlap probe,
  built and never logged: 2.2). A goal that meets one may land a
  checkpoint: a change into main on a branch made at a commit of its own
  branch, checked whole by the full gate, which the other goal then takes
  by its ordinary catch-up, while its own branch goes on. Goals never
  merge each other's unlanded branches: a branch carrying another goal's
  unlanded work cannot land without it, which makes two goals one landing
  (the trains the landing model dropped) and a dropped goal a hole in the
  other (N2's rule for dependencies, across goals). A semantic probe, the
  gate on a speculative merge, costs a CI run per pair and waits for
  numbers. Needs: the probe and its news kind, `change` from a commit of
  the goal's branch (N22), and the coordinator's judgement of when a
  commit is a coherent checkpoint.
