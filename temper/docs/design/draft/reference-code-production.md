# Reference: code production

Draft, 2026-11-02. The reference way a project produces code on temper,
from a person's draft to a goal branch ready to land: the execution
model, each of its roles in jig's primitives and temper's mechanisms,
the configuration that sets it up, and what is missing. It is the way a
project develops itself through temper: one repository, a person who
writes design drafts, a designer that plans them, coding agents that
fill the plan. How the goal branch lands is `reference-landing.md`;
`reference-case.md` walks both on one small goal; `reference-notes.md`
holds the evidence and the changes to existing designs, N1 to N20, cited
here by number.

Unqualified, `forge.md`, `landing.md`, `after-landing.md`, `agent.md`
and `worker.md` are temper's `../domain/` documents; jig's and smith's
are named as theirs.

## 1. In one page

- **One goal per draft.** A person writes a design draft and starts a
  goal from it: a tracked agent task under the designer's charter, the
  coordinator, at Opus 5.5 and effort xhigh, its sub-agents at Opus 5.5
  and high. It is called, not resident: each wake is a fresh run from
  its brief and inbox.
- **The cut is its first run.** It surveys with read-only sub-agents,
  writes the shared notes and contracts, asks designer sub-agents for a
  family of specs each, and releases the plan as batches of changes as
  soon as each part of it is ready. The plan is the tree of tasks it
  makes.
- **A goal branch collects the work.** The cut's first batch makes the
  goal's change into main, collecting (N3): its branch is the landing
  branch of every other change of the goal, merged into it under light
  rules (N6), and lands whole once the coordinator closes it.
- **Each spec is a change.** Its words are the spec; its producer is a
  coding agent starting from the goal branch's tip; it pushes as it goes
  and finishes with what it did and what it left. A change whose
  dependency is still being produced starts kept, its producer working
  on the dependency's pushed head (N2); its merge waits.
- **Producers ask.** A question goes to the change, which asks a foreman
  first; what the change's words do not settle goes to the coordinator,
  and what the coordinator cannot settle, to the person (N1). Answers are
  the store's, never files'.
- **Holds escalate to the coordinator.** A producer past its tries, a
  change past its bounds or its questions, is held, and the coordinator
  decides: release, amend, cancel, or pass it to the person.
- **Checked along the way.** A tier of the goal branch's own runs at its
  newest tip; a red makes one repair change into it while the other
  changes keep merging (N5). The goal branch catches up with main along
  the way (N4).
- **Mostly configuration.** Charters, a goal template with its LLM
  choice, landing rules, tiers and a few project settings. What must be
  built is section 15: chiefly the change's question route, its kept
  start, the collecting change, the resolver job, and smith's sub-agents
  on Claude Code.

## 2. Roles

| Role | Does | Executor | LLM in the reference |
|---|---|---|---|
| the person | writes the draft, starts the goal, answers what reaches them, decides what is held past the coordinator | a party | |
| the coordinator | cuts the plan, answers questions, decides holds, closes the goal branch, triages reds after landing, retires the goal | an agent task, tracked, the designer's charter | Opus 5.5, xhigh |
| a survey | reads the code and designs for one question, answers with a report | a sub-agent of the coordinator's run, inspect only | Opus 5.5, high |
| a designer | writes one family's specs against the contracts | a sub-agent of the coordinator's run, inspect only | Opus 5.5, xhigh |
| the goal branch | collects the goal's changes, catches up with main, lands whole | the forge's change, collecting (N3) | |
| a change | one spec, landed into the goal branch | the forge's change | |
| a producer | writes the code and its tests, runs the checks, pushes | an agent task, the coding agent's charter | gpt-6.1-sol, high or medium by spec (N18, N19) |
| a foreman | settles a question from the change's words, or passes it up | an agent task, the foreman's charter, read-only | gpt-6.1-sol, medium |
| the resolver | settles known conflicts mechanically (a counted baseline) | a job (N7) | none |
| a resolution | resolves a conflict in a merge in progress | an agent task, the coding agent's charter | gpt-6.1-sol, high |
| the goal branch's tier | checks the goal branch along the way | the forge's tier (N5) | none |
| a repair | repairs a red of the goal branch's tier | the forge's change into the goal branch | as a producer |

Two levels of delegation, as jig has them (jig's `domain/core.md`,
7.4): surveys and designers are sub-agents, cheap and gone with the run
that asked for them; changes and their agents are tasks, durable, each
with its own workspace and authority.

## 3. The tree

```
P, a person
└─ G   implement the draft                 agent, the designer's charter; tracked
   ├─ B   the goal branch                  the forge's change into main, collecting (N3)
   ├─ N   the plan's notes                 the forge's change into B's branch, handed G's run branch
   ├─ C1…Cn  the plan's changes            the forge's change into B's branch, after N and each other
   │     ├─ producer                       agent, the coding agent's charter
   │     ├─ foreman, one per question      agent, the foreman's charter (N1)
   │     ├─ resolver                       job (N7)
   │     └─ resolution                     agent, the coding agent's charter
   ├─ Rk  a repair of B's branch           the forge's change into B's branch, made by B (N5)
   └─ after B lands: repairs into main, the retire   (reference-landing.md)
```

- **G is the goal** (jig's `domain/core.md`, 6.2): its priority orders
  the queues its changes join; its projection is the goal's issue; its
  scope holds the plan's notes; it hears its subtree's landings as news.
- **The plan is G's subtree** (jig's `domain/core.md`, 7.1): its
  structure is the tasks and their dependencies, which the engine runs
  and the store keeps as the plan's history; its strategy is G's notes
  and the issue; its authority is G's.
- **Every change is G's delegate,** made by G's `delegate` calls, so
  G is the requester each question and escalation of the plan reaches.
  Rk is B's delegate, made by B's step, and its questions reach G through
  B (N1).

## 4. A draft becomes a goal

- **The draft** is a design document on main (`<tree>/docs/design/
  draft/<name>.md`), written by the person, perhaps with a chat.
- **The person starts the goal** from the project's goal template for
  drafts (section 14), with words naming the draft and what to build
  from it. Its words may say the goal does not land (a rehearsal): then
  G never closes B, and B's branch stays for people to read.
- **The template makes it whole:** the designer's charter, the LLM
  choice (`{"main": {"model": "claude-opus-5-5", "effort": "xhigh"},
  "sub_agents": [{"model": "claude-opus-5-5", "effort": "xhigh"},
  {"model": "claude-opus-5-5", "effort": "high"}]}`), tracked at the
  project's priority, a report's contract, the coordinator's wake
  policy (section 13), and an authority sized for a plan of up to a
  hundred changes (N20). jig checks it as any batch (jig's
  `domain/tasks.md`, section 4).
- **G becomes active** at once and its first run is the cut.

## 5. The cut

G's first run, in a workspace of its own: a checkout of main at its tip,
which it may write and pushes as its run branch, `temper/<root>/r<G>`
(`forge.md`, section 11). Its instructions are the designer charter's;
the project's rules reach it as the charter's documents (`AGENTS.md`,
the code's rules: temper's `../protocol/configuration.md`, 13.7).

1. **Read.** The draft, the guides, the designs the draft touches and
   their status files.
2. **Survey, if it judges it worth it.** Inspect-only sub-agents at
   high, one question each, side by side (smith's `domain/run.md`,
   5.3), naming the LLM in each call; their answers are their reports.
   What the code does today goes into a status note.
3. **Contracts first.** The shapes several changes share, each name
   exact, and the rules every change follows (what it may touch without
   asking, what is not its own, how it reports): files under the plan's
   directory in G's workspace (`<tree>/docs/plan/<goal>/`), written
   before any spec (N12). The strategy (families, the critical path,
   what was dropped and why) goes into G's notes, in the goal's scope,
   and the issue shows it.
4. **Release what needs no designer.** One batch: B, the goal branch
   (collecting, into main); N, a change into B's branch handed G's run
   branch, which lands the notes; and the changes the cut can specify
   itself, each depending on N. They start at once.
5. **Design by family.** One designer sub-agent per family of work (one
   writer per crate), at xhigh, inspect-only, side by side, each briefed
   with the contracts and the surveys' reports; each answers with its
   family's specs and their dependencies, and asks for a contract it
   lacks in its answer. G settles contract requests, amends the
   contracts before N lands or in the next batch's words, and briefs
   again.
6. **Release each family as it is ready.** One batch per answer: its
   changes, each with its dependencies on changes already made or in the
   same batch. A change whose dependency is not yet made waits for the
   batch that makes it; a dependency found later is added by amendment
   (jig's `domain/tasks.md`, section 7).
7. **Park.** Its run ends with the plan released; a coordinator parks at
   once (jig's `domain/engine.md`, 7.4), and wakes as its policy says.

**A spec** is a change's words, whole (`forge.md`, 8.1): its goal and
acceptance, its inputs (the notes and the paths it reads), what it owns
and may change, its steps and checks, and its report. Its parameters
carry the rest: the repository, B's branch as its landing branch, its
dependencies, its producer's LLM choice (N18), and that its questions
are relayed (N1). No spec is a file and none is edited after release:
what changes it is an amendment of its words, visible in its history.

## 6. A change into the goal branch

Its way is the change procedure's (`forge.md`, 8.2), under the light
rules of goal branches:

1. **Producing.** A task for an agent under the coding agent's charter,
   at the change's LLM choice, whose workspace starts from B's branch at
   its tip (with its dependencies' pushed heads, if it starts kept: N2),
   so the plan's notes are in its checkout. It runs the project's checks
   at delivery (`.temper/pre-pr`: smith's `domain/run.md`, 8.1), pushes
   as it goes (N13), and finishes with the head it pushed and its report:
   what it did, the checks it ran, and a Left list of what it found and
   did not own (N14).
2. **Opening.** Its pull request into B's branch.
3. **Checking and gating:** none. The rules covering goal branches
   require no CI, no up to date and no gate (N6).
4. **Queued, then first,** in B's branch's queue, ordered by priority
   (one goal's, so by when each became ready), never while kept by a
   dependency (`forge.md`, section 9).
5. **Merged** at exactly its head, as a merge commit, once the forge
   finds it mergeable. A conflict with B's tip is an update in conflict:
   the project's resolver job first (N7), then a resolution, an agent in
   a merge in progress (`forge.md`, 8.4). The merge takes B's branch's
   writer slot, so it never races B's own updates (N3).
6. **Landed,** and done: a change into a goal branch has no verification
   (N5). Its result reaches G and waits in G's inbox until G next wakes.

## 7. Early starts

A change that depends on another change into the same goal branch
starts kept (jig's `domain/connectors.md`, 6.1): it steps while its
dependency is produced, and its merge waits for it (jig's
`domain/tasks.md`, 5.7). Its producer starts early under N2's rules:

- once each pending dependency has a pushed head and none of them is
  kept itself;
- from B's tip with those heads merged in, in dependency order; a
  conflict there refuses the early start, and the change waits for its
  dependencies to land;
- once: a producer that asks, or fails for want of its dependencies'
  work, does not start early again: the next waits for them to land;
- a producer continuing from saved work merges B's tip into it first.

Done while kept, the producer's work waits: the change is not ready
until its last dependency has landed, then merges as any change. Its
head is what it pushed, built on its dependencies' heads as they were;
the merge into B's branch is where a difference shows, as a conflict.

## 8. Questions

The route the reference needs and temper today refuses (N1): a change
relays its agents' questions.

1. **The producer asks** its requester, the change, in the question
   form, with its options and its recommendation, after pushing what it
   has. It is kept by its own question (jig's `domain/tasks.md`, 5.7):
   its run parks, no try spent.
2. **The change asks a foreman:** a delegate under the foreman's
   charter, read-only, its brief the change's words, the question and
   the producer's report. Its verdict is one of: *answer*, where the
   words, the notes or the draft settle it; *finish*, where the work
   done holds and the rest belongs to the producer's Left list; or
   *escalate*. On the first two, the change answers its producer's
   question with the foreman's words.
3. **Escalated, the change asks G** the same question. G wakes, reads it
   with the producer's report and the foreman's reasons, and does one of:
   - answers it, in the answer form;
   - amends the change (its words widened to a file another change does
     not own, a Left item accepted) and answers with what changed;
   - makes a new change, adds it as a dependency, and answers that the
     work waits for it: the producer's next run waits for the dependency
     to land and merges B's tip first (N2);
   - asks the person, P, a question of its own; P answers in the inbox;
     G answers the change.
4. **The change relays the answer down;** the producer's next run goes
   on from where it parked with the answer to read.

- **Bounded.** A change takes a number of questions (four in the
  reference); past it, the next is not given a foreman: the change holds
  itself, escalated to G, who decides it as a hold (section 9).
- **Answers stay in the store:** the asked task's question positions and
  the change's history (jig's `domain/tasks.md`, 8.2). Nothing is
  committed to the repository to carry an answer.
- **A repair's questions** reach B, which relays them to G as its own
  change relays its producer's (N1).
- **When G answers:** on each question, at once (N8).

## 9. Failures, holds and escalations

jig's rules, unchanged (jig's `domain/tasks.md`, 5.5 and section 9):

- **A producer's failures** are tried again within their class's tries:
  a peer gone, an agent crashed, a host lost. Its own failure (the task
  cannot be done as asked) holds the change at producing, with its words
  (`forge.md`, 8.4).
- **A change's bounds** (repairs, resolutions, updates, each wait's
  stall, its questions) hold it, saying why.
- **A budget spent** holds the task that spent it: it is the bound on a
  producer that loops.
- **Every hold escalates** to the nearest holder above that may decide
  it, skipping procedures and kept tasks: G, for the plan's changes and
  their agents. G releases it, amends then releases it (more budget, a
  wider scope, a dependency), cancels it, or passes it up to P.
- **A failed change's dependents are held** until G amends their
  dependencies away or cancels them.

## 10. Checked along the way

The goal branch is checked by a tier of its own (N5): the project's
gate without its slowest suites, defined as a tier is
(`after-landing.md`, section 2), naming goal branches.

- **One run at a time,** at B's newest tip, at most once per span; a
  run covers every merge since the last.
- **Green** says nothing to anyone.
- **Red,** rerun alone once (`after-landing.md`, 5.2); what still fails
  makes one repair change into B's branch, made by B's step: its words
  are the failing tests, the log's end, and the changes not yet landed
  with what they own, to be left alone; it repairs the rest. No new run
  starts while it is live. The queue into B's branch is not paused:
  other changes keep merging.
- **A repair that finds everything left to changes not yet landed**
  finds nothing to change: it holds at producing, escalated to G, which
  cancels it; the next run judges again.
- **Reds of this tier never hold B,** and its changes never wait for a
  verdict; B's own CI into main judges the whole (`reference-landing.md`).

## 11. Catching up with main

While B collects, main moves with other landings. B takes them along
the way (N4): when main has moved and the span has passed since its last
update, B updates, the forge merging main into B's branch. A conflict is
B's own resolution, the resolver job first, then an agent in a merge in
progress on B's branch; its writer slot keeps the other changes' merges
waiting meanwhile. Three resolutions in a row that fail to bring main in
hold B, escalated to G.

## 12. Ready to land

- **B tells G when it is drained:** once no change into its branch is
  live, B's step messages G (N3).
- **G closes B** when its plan is done: every change landed or cancelled,
  every Left list either made into a change or accepted as left, the
  last tier run green or its red owned by a live repair. Closing is an
  amendment of B's `collect` parameter.
- **Closed, B goes its own way into main** (`reference-landing.md`):
  opens its pull request, CI at its head, queue, update, merge, tiers.
  No change into its branch is made after.
- **A rehearsal** is never closed: G reports with B's branch and the
  last tier verdict, and cancels B.

## 13. What wakes G, and what it does

G's wake policy (jig's `domain/tasks.md`, 8.3): questions, escalations,
failures, words from its delegates, its person's words and news its
connectors class as waking; delegates' results are kept, read at the
next wake. Each wake is a fresh run, its brief the goal, the plan's
state, its notes and the messages that woke it (jig's
`domain/engine.md`, 7.4).

| Wake | G reads | G decides |
|---|---|---|
| activation | the draft, the code | the cut (section 5) |
| a question a change relays | the question, the producer's report, the foreman's reasons, the plan | answer, amend, new change, ask P (section 8) |
| an escalation | the held task and why | release, amend, cancel, pass up (section 9) |
| B's words: drained | results and Left lists since the last wake | new changes for what is left, or close B (section 12) |
| a red after landing, as news | the tier, the failing tests, the log | triage and a repair into main (`reference-landing.md`) |
| B's result: cleared | the landing | the retire (`reference-landing.md`) |

Each wake also reads the results waiting in its inbox: a Left list that
names a fault in code another change owns goes to that change's words by
amendment while it lives, or to a new change once it has landed.

## 14. Configuration

What the reference sets, as settings (jig's `domain/engine.md`, section
17; temper's `../protocol/configuration.md`, section 13):

| Setting | The reference | Exists |
|---|---|---|
| the designer's charter | instructions for the cut, answers, holds, closing, triage and the retire; Claude Code; tools: `delegate`, `amend`, `cancel`, `release`, `decide`, `decide_escalation`, `propose`, `subscribe_forge`, `note`, `message`, `wait`, the workspace's inspect, modify and shell, sub-agents; starts fresh; waiting 0; documents: the project's `AGENTS.md` and code rules | as a charter; sub-agents on Claude Code no (N10) |
| the goal template for drafts | the designer's charter; the LLM choice of section 4; tracked; a report; section 13's wake policy | yes (jig's `../protocol/configuration.md`, 3.1) |
| the coding agent's charter | charter 2 with instructions to push early, finish with a Left list, and never fix what it does not own (N14) | yes, its text new |
| the foreman's charter | read-only (inspect and the forge's reads); a verdict of answer, finish or escalate | as a charter; its use by the change no (N1) |
| the forge's charters (kind 3) | 1, the change's: the coding agent's; 2, the reviews': unused; 3, the foreman's | number 3 no (N1) |
| each change's LLM choices | producer, repairs and resolutions: gpt-6.1-sol at the spec's effort; foreman: gpt-6.1-sol, medium | no (N18) |
| landing rule for main | CI passed, up to date | yes |
| landing rule for goal branches | `temper/*/g*`: nothing required; a merge commit | the rule yes; skipping checking no (N6) |
| the goal branch's tier | the gate without its slowest suites; at most once per 20 minutes | no (N5) |
| main's tiers | the slow suites | designed (`after-landing.md`) |
| questions | relayed; four per change | no (N1) |
| catch-up span | 30 minutes | no (N4) |
| resolver | the project's job that recounts its counted baselines | no (N7) |
| the profile's limits | sized for a hundred changes under one goal | to check (N20) |

## 15. What is missing

| Gap | Where | Note | Status |
|---|---|---|---|
| a change relays its agents' questions through a foreman, then up; a question bound | temper | N1 | contradicts `forge.md`, 8.1 and `agent.md`, 8.2 |
| a change that starts kept | jig, temper | N2 | designed, not built |
| a producer's checkout at the landing tip with its dependencies' heads; the four early-start rules | temper | N2 | missing |
| the collecting change: its branch a landing branch, closed by its requester, drained to its requester | temper | N3 | in words only (`forge.md`, 11.1) |
| catching up along the way | temper | N4 | contradicts `forge.md`, 10 |
| a goal branch's tier, its repair change, the queue not paused, no verification inside | temper | N5 | missing |
| no checking step where no CI is required | temper | N6 | not built (`forge.status.md`, 8.7) |
| jobs; the resolver before a resolution | jig, temper | N7 | jobs designed, not built |
| sub-agents on a program backend, each naming its LLM; an answer that holds a family's specs | smith | N10 | open question (smith's `protocol/programs.md`, section 9) |
| shared notes as files landed first; the spec bound | configuration | N12 | to check |
| `deliver` mid-run for a producer | temper | N13 | to check against the coding agent's tools |
| a change names its agents' LLM choices | temper | N18 | missing |
| implementers on gpt-6.1-sol | smith | N19 | the direct backend has it; Codex as a program is Later |
| limits for a goal of this size | configuration | N20 | to check |

jig's own primitives cover the rest: goals, coordinators called fresh,
batches with dependencies, dependencies added later, kept tasks,
questions and answers, holds and escalations, wake policies, notes,
the LLM choice with sub-agents, and templates.

## 16. Later

- Several goals at once, their branches overlapping: a report of the
  overlap to both coordinators.
- The order of work within one goal when capacity is short: repairs and
  resolutions before new producers (N9).
- An LLM choice per kind of wake, so answers cost less than the cut
  (N11).
- What the reference leaves off, each a charter and a delegate or a
  gate when wanted: reviews of a change before it merges, a critique of
  the cut, skeleton changes for shared shapes, test writers, a structure
  review of the plan.
- Changes into a goal branch without a pull request each.
- Where builds run, and how many at once.

## 17. Open questions

- **Designers as sub-agents or as tasks.** Sub-agents keep the cut one
  run and its specs in G's hands, and need N10 and an answer large
  enough for a family's specs. Tasks would each push their specs to a
  branch of their own for G to read, durable and parallel, at the cost
  of G waking once per family. The reference takes sub-agents.
- **Should the goal branch's tier blame merges,** bisecting to the change
  whose merge brought a failure and sending it to that change's
  producer, rather than one repair for all? One repair matched the
  evidence; blame would need the tier's verdict machinery inside a goal.
- **Should a kept change start before its dependency's first push,**
  from its contracts alone? The rule waits for a push, so the producer
  builds on code rather than on a description of it.
