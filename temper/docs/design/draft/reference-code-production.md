# Reference: code production

Draft, 2026-11-02. The reference way a project produces code on temper,
from a person's draft to a goal branch ready to land: the execution
model, each of its roles in jig's primitives and temper's mechanisms,
the configuration that sets it up, and what is missing. It is the way a
project develops itself through temper: one repository, a person who
writes design drafts, designers that plan them, coding agents that fill
the plan. How the goal branch lands is `reference-landing.md`;
`reference-case.md` walks both on one small goal; `reference-notes.md`
holds the evidence and the changes to existing designs, N1 to N23, cited
here by number.

Unqualified, `forge.md`, `landing.md`, `after-landing.md`, `agent.md`
and `worker.md` are temper's `../domain/` documents; jig's and smith's
are named as theirs.

## 1. In one page

- **One goal per draft.** A person writes a design draft and starts a
  goal from it: a tracked agent task under the designer's charter, the
  coordinator, at Opus 5.5 and effort xhigh, its survey sub-agents at
  Opus 5.5 and high. It is called, not resident: each wake is a fresh
  run from its brief and inbox.
- **The plan's text is notes, its structure is tasks.** The coordinator
  writes the contracts, the common rules and the strategy as notes in
  the goal's scope, which every task under the goal reads (N12); the
  plan itself is the tree of tasks it makes. The coordinator writes no
  file.
- **Designers are tasks.** One per family of work, at xhigh, each
  surveying with sub-agents at high, writing its specs as notes, asking
  the coordinator for what the contracts lack. Each designer's result
  wakes the coordinator, which releases that family as a batch of
  changes: the plan goes out as each part is ready.
- **A goal branch collects the work.** The first batch makes the goal's
  change into main, collecting (N3): its branch is the landing branch of
  every other change of the goal, merged into it under light rules (N6),
  and it lands whole once the coordinator closes it.
- **Each spec is a change.** Its words are the spec; its producer is a
  coding agent starting from the goal branch's tip; it pushes as it goes
  and finishes with what it did and what it left. A change whose
  dependency is still being produced starts kept, its producer working
  on the dependency's pushed head (N2); its merge waits.
- **Producers ask.** A question goes to the change, which asks a foreman
  first; what the change's words do not settle goes to the coordinator,
  and what the coordinator cannot settle, to the person (N1).
- **Holds escalate to the coordinator,** which releases, amends, cancels,
  or passes them up to the person.
- **Checked along the way.** A tier of the goal branch's own runs at its
  newest tip; a red makes one repair change into it while other changes
  keep merging (N5). The goal branch catches up with main along the way
  by catch-up changes into it (N4).
- **Mostly configuration,** with a few things to build (section 15):
  chiefly the coordinator's `change` with all its parameters (N21), the
  change's question route (N1), its kept start (N2), the collecting
  change (N3) and the merge job (N7).

## 2. Roles

| Role | Does | Executor | LLM in the reference |
|---|---|---|---|
| the person | writes the draft, starts the goal, answers what reaches them, decides what is passed up | a party | |
| the coordinator | the plan's notes, designers, batches, answers, holds, closing the goal branch, triage after landing, the retire | an agent task, tracked, the designer's charter | Opus 5.5, xhigh |
| a designer | one family's specs, as notes | an agent task, the designer's charter, narrowed | Opus 5.5, xhigh |
| a survey | reads the code and designs for one question, answers with a report | a sub-agent of a coordinator's or a designer's run, inspect only | Opus 5.5, high |
| the goal branch | collects the goal's changes, catches up with main, lands whole | the forge's change, collecting (N3) | |
| a change | one spec, landed into the goal branch | the forge's change | |
| a producer | writes the code and its tests, runs the checks, pushes | an agent task, the coding agent's charter | gpt-6.1-sol, high or medium by spec (N18, N19) |
| a foreman | settles a question from the change's words and the goal's notes, or passes it up | an agent task, the foreman's charter, read-only | gpt-6.1-sol, medium |
| the merge job | merges a base into a branch, settling the conflicts the project's tooling knows | a job (N7) | none |
| a resolution | resolves what the merge job left, in a merge in progress | an agent task, the coding agent's charter | gpt-6.1-sol, high |
| the goal branch's tier | checks the goal branch along the way | the forge's tier (N5) | none |
| a repair, a catch-up | a change into the goal branch the goal branch makes | the forge's change | as a producer; the merge job |

Two levels of delegation, as jig has them (jig's `domain/core.md`,
7.4): surveys are sub-agents, cheap and gone with their run; designers,
changes and their agents are tasks, durable, each with its own
workspace and authority.

## 3. The tree

```
P, a person
└─ G   implement the draft                 agent, the designer's charter; tracked
   ├─ B   the goal branch                  the forge's change into main, collecting (N3)
   │  ├─ Rk  a repair of B's branch        the forge's change into B's branch (N5)
   │  └─ Kk  a catch-up with main          the forge's change into B's branch (N4)
   ├─ D1…Dm  designers, one per family     agent, the designer's charter, narrowed
   ├─ C1…Cn  the plan's changes            the forge's change into B's branch, after each other
   │     ├─ producer                       agent, the coding agent's charter
   │     ├─ foreman, one per question      agent, the foreman's charter (N1)
   │     ├─ merge job                      job (N7)
   │     └─ resolution                     agent, the coding agent's charter
   └─ after B lands: repairs into main, the retire   (reference-landing.md)
```

- **G is the goal** (jig's `domain/core.md`, 6.2): its priority orders
  the queues its changes join; its projection is the goal's issue; its
  scope holds the plan's notes; it hears its subtree's landings as news.
- **The plan is G's subtree** (jig's `domain/core.md`, 7.1): its
  structure is the tasks and their dependencies, which the engine runs
  and the store keeps as the plan's history; its strategy is G's notes
  and the issue; its authority is G's.
- **Every change is G's delegate,** made by G's `change` calls (N21),
  so G is the requester each question and escalation of the plan
  reaches. Rk and Kk are B's, made by B's step; their questions reach G
  through B (N1).

## 4. A draft becomes a goal

- **The draft** is a design document on main (`<tree>/docs/design/
  draft/<name>.md`), written by the person, perhaps with a chat.
- **The person starts the goal** from the project's goal template for
  drafts (section 14), its words naming the draft and what to build
  from it.
- **The template makes it whole:** the designer's charter, the LLM
  choice (`{"main": {"model": "claude-opus-5-5", "effort": "xhigh"},
  "sub_agents": [{"model": "claude-opus-5-5", "effort": "high"}]}`),
  tracked at the project's priority, a report's contract, and an
  authority sized for a plan of up to a hundred changes (N20); its wake
  policy is the charter's default (N22). jig checks it as any batch
  (jig's `domain/tasks.md`, section 4).
- **G becomes active** at once; its first run begins the cut.

## 5. The cut

G's runs read: a checkout of main at its tip, inspect only, no
delivery (`agent.md`, 4.3), its brief carrying the project's rules as
the charter's documents (`AGENTS.md`, the code's rules: temper's
`../protocol/configuration.md`, 13.7).

1. **Read.** The draft, the guides, the designs it touches and their
   status files.
2. **Survey, if it judges it worth it.** Inspect-only sub-agents at
   high, one question each, side by side (smith's `domain/run.md`, 5.3);
   their answers are their reports.
3. **Write the plan's notes,** in the goal's scope (jig's
   `domain/engine.md`, section 10): `plan/strategy` (the families, the
   critical path, what was dropped and why), `plan/status` (what the
   code does today, from the surveys), `plan/common` (what every change
   may do without asking, what is not its own, how it reports), and one
   `contract/<name>` per shape several changes share, its names exact.
   The specs G writes itself, for work that needs no designer, are notes
   too: `spec/<id>`.
4. **First batch:** B, the goal branch, collecting, into main; and one
   designer per family, each told its family's scope and the notes to
   read. Then G parks (jig's `domain/engine.md`, 7.4).
5. **B makes its branch** at main's tip and tells G so (N3). G wakes and
   makes the changes it specified itself, as a batch into B's branch.
6. **Designers design.** Each reads the draft, the notes and the code,
   surveys with its own sub-agents at high where it judges it worth it,
   and writes each spec of its family as a note `spec/<id>`, naming its
   dependencies. A contract it lacks it asks G for, in the question
   form: G adds or revises the contract's note and answers; the
   designer goes on. It finishes with a report listing its specs.
7. **Each family as it is ready.** Each designer's result wakes G, which
   reads the family's specs and makes them a batch of changes, each with
   its dependencies on changes already made or in the same batch. A
   change whose dependency is not yet made waits for the batch that
   makes it; a dependency found later is added by amendment (jig's
   `domain/tasks.md`, section 7).

**A spec** is a change's words, whole (`forge.md`, 8.1), copied from its
note: its goal and acceptance, its inputs (the notes and the paths it
reads), what it owns and may change, its steps and checks, and its
report. Its parameters carry the rest: B's branch as its landing branch,
its dependencies, its producer's LLM choice (N18), and that its
questions are relayed (N1). What changes a spec after release is an
amendment of its words, kept in its history; what changes a contract is
a revision of its note, after which G amends the words of the live
changes it concerns, since notes notify nobody.

## 6. A change into the goal branch

Its way is the change procedure's (`forge.md`, 8.2), under the light
rules a collecting change gives the changes into its branch (N6):

1. **Producing.** A task for an agent under the coding agent's charter,
   at the change's LLM choice, whose workspace starts from B's branch at
   its tip (with its dependencies' pushed heads, if it starts kept: N2).
   It reads the notes its words name, runs the project's checks at
   delivery (`.temper/pre-pr`: smith's `domain/run.md`, 8.1), pushes as
   it goes (N13), and finishes with the head it pushed and its report:
   what it did, the checks it ran, and a Left list of what it found and
   did not own (N14).
2. **Opening.** Its pull request into B's branch.
3. **No checking, no gating, no update** (N6).
4. **Queued, then first,** in B's branch's queue, ordered by priority
   (one goal's, so by when each became ready), never while kept by a
   dependency (`forge.md`, section 9).
5. **Merged** at exactly its head, as a merge commit, once the forge
   finds it mergeable. A merge the forge refuses for a conflict goes to
   the merge job, which merges B's tip into the change's branch and
   settles what the project's tooling knows (N7); what is left goes to a
   resolution, an agent in that merge in progress (`forge.md`, 8.4);
   the merged head then merges into B's branch.
6. **Landed,** and done: no tier of main follows B's branch, so a
   change into it has no verification (N5). Its result wakes G.

## 7. Early starts

A change that depends on another change into the same goal branch
starts kept (jig's `domain/connectors.md`, 6.1): it steps while its
dependency is produced, and its merge waits for it (jig's
`domain/tasks.md`, 5.7). Its producer starts early under N2's rules:

- once each pending dependency has a pushed head and none of them is
  kept by a dependency of its own;
- from B's tip with those heads merged in, in dependency order; a
  conflict there refuses the early start, and the change waits for its
  dependencies to land;
- once: a producer that asks, or fails for want of its dependencies'
  work, does not start early again: the next waits for them to land;
- a producer continuing from saved work merges B's tip into it first.

Done while kept, the producer's work waits: the change is not ready
until its last dependency has landed, then merges as any change. Its
head carries its dependencies' work as they pushed it, so a dependency
that is dropped takes its dependent with it: G cancels and remakes the
dependent rather than amending the dependency away (N2).

## 8. Questions

The route the reference needs and temper today refuses (N1): a change
relays its agents' questions.

1. **The producer asks** its requester, the change, in the question
   form, with its options and its recommendation, after pushing what it
   has. It is kept by its own question (jig's `domain/tasks.md`, 5.7):
   its run parks, no try spent.
2. **The change asks a foreman:** a delegate under the foreman's
   charter, read-only, its brief the change's words, the question and
   the change's pushed head; it reads the goal's notes. Its verdict is
   one of: *answer*, where the words, the notes or the draft settle it;
   *finish*, where the work done holds and the rest belongs to the
   producer's Left list; or *escalate*. On the first two, the change
   answers its producer's question with the foreman's words.
3. **Escalated, the change asks G** the same question, and is kept by
   it while it waits. G wakes and does one of:
   - answers it, in the answer form;
   - amends the change (its words widened to a file no other change
     owns, a Left item accepted) and answers with what changed;
   - makes a new change, adds it as a dependency, and answers that the
     work waits for it: the change holds the answer until the
     dependency has landed, and the producer's next run merges B's tip
     first (N2);
   - asks P, through a person task with the options as its choices
     (jig's `domain/people.md`, section 8), which keeps G from nothing;
     P answers in the inbox; the person task's result wakes G, and G
     answers the change.
4. **The change relays the answer down;** the producer's next run goes
   on from where it parked with the answer to read.

- **Bounded.** A change takes a number of questions (four in the
  reference); past it, the next is not given a foreman: the change holds
  itself, escalated to G. Released, the change relays that question to
  G with no foreman.
- **Answers stay in the store:** the asked task's question positions and
  the change's history (jig's `domain/tasks.md`, 8.2). A ruling that
  concerns many changes becomes a revision of `plan/common` or a
  contract, and amendments of the live changes it concerns.
- **A repair's or a catch-up's questions** reach B, which relays them to
  G with no second foreman (N1).
- **When G answers:** on each question, at once (N8).

## 9. Failures, holds and escalations

jig's rules, unchanged (jig's `domain/tasks.md`, 5.5 and section 9):

- **A producer's failures** are tried again within their class's tries:
  a peer gone, an agent crashed, a host lost. Its own failure (the task
  cannot be done as asked) holds the change at producing, with its words
  (`forge.md`, 8.4).
- **A change's bounds** (repairs, resolutions, each wait's stall, its
  questions) hold it, saying why; so does its authority spent (its tasks
  made: `forge.md`, 8.4; N20).
- **A budget spent** holds the task that spent it: it is the bound on a
  producer that loops.
- **Every hold escalates** to the nearest holder above that may decide
  it, skipping procedures and kept tasks: G, for the plan's changes and
  their agents. G releases it, amends then releases it (more budget, a
  wider scope, a dependency), cancels it, or passes it up to P with its
  reasons, for P to decide the same way.
- **A failed change's dependents are held** until G cancels them or
  amends their dependencies away (only those that did not start early
  on it: section 7).

## 10. Checked along the way

The goal branch is checked by a tier that follows collecting changes'
branches (N5): the project's gate without its slowest suites, defined as
a tier is (`after-landing.md`, section 2). Main's tiers follow main
only.

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
  finds nothing to change: it holds at producing, escalated past B to
  G, which cancels it; the next run judges again.
- **Reds of this tier never hold B,** and judge no landing; B's own CI
  into main judges the whole (`reference-landing.md`).

## 11. Catching up with main

While B collects, main moves with other landings. B takes them along
the way (N4): when main has moved, the span has passed since its last
catch-up, and no repair or catch-up into its branch is live, B makes a
catch-up change into its branch. Its producer is the merge job, which
merges main's tip into a branch made at B's tip and settles what the
project's tooling knows; what is left goes to a resolution in that merge
in progress; the result merges into B's branch through its queue like
any change. Three catch-ups in a row that fail to bring main in hold B,
escalated to G.

## 12. Ready to land

- **G closes B** when its plan is done: every change landed or cancelled,
  every Left list made into a change or accepted as left, the tier's
  last run green or its red owned by a live repair. Closing is an
  amendment of B's `collect` parameter (N3); G sees the plan done when
  its last change's result wakes it.
- **Closed, B goes its own way into main** once no change into its branch
  is live (`reference-landing.md`): opens its pull request, CI at its
  head, queue, update, merge, tiers.
- **No change into B's branch is made after,** unless G reopens it by
  the same amendment.

## 13. What wakes G, and what it does

G's wake policy, the designer's charter's default (N22): questions,
escalations, failures, every delegate's result, words from its
delegates, its person's words and news its connectors class as waking.
Each wake is a fresh run, its brief the goal, the plan's state, its
notes and the messages that woke it (jig's `domain/engine.md`, 7.4); a
message that arrives while G runs is relayed to that run instead.

| Wake | G reads | G decides |
|---|---|---|
| activation | the draft, the code | the plan's notes, B, the designers (section 5) |
| B's words: its branch made | its own spec notes | the changes it specified (section 5) |
| a designer's question | the question, the contracts | a contract's note, an answer |
| a designer's result | the family's spec notes | the family's batch |
| a question a change relays | the question, the foreman's reasons, the notes | answer, amend, new change, ask P (section 8) |
| an escalation | the held task and why | release, amend, cancel, pass up (section 9) |
| a change's result | its report and Left list | a new change for what is left, an amendment of a live change, or nothing; close B once the plan is done (section 12) |
| a person task's result | P's choice | the answer it was waiting to give |
| a landing into main that overlaps the goal's open work (`forge.md`, 7.3) | the landing | usually nothing: B catches up (section 11) |
| a red after landing, as news | the tier, the failing tests, the log | triage and a repair into main (`reference-landing.md`) |
| B's result: cleared | the landing | the retire (`reference-landing.md`) |
| the retire's result | its landing | the report: the goal is done |

## 14. Configuration

What the reference sets, as settings (jig's `domain/engine.md`, section
17; temper's `../protocol/configuration.md`, section 13):

| Setting | The reference | Exists |
|---|---|---|
| the designer's charter | instructions for the cut, designing, answers, holds, closing, triage and the retire; Claude Code; tools: `change` (N21), `delegate` (designers, person tasks, a triage), `amend`, `cancel`, `release`, `decide`, `decide_escalation`, `propose`, `subscribe_forge`, `subscribe`, `note`, `recall`, `message`, `wait`, the workspace's inspect, sub-agents; starts fresh; waiting 0; its default wake policy (section 13); documents: the project's `AGENTS.md` and code rules | as a charter; `change`'s parameters no (N21); its wake policy no (N22); sub-agents on Claude Code no (N10) |
| a designer | the designer's charter, its authority narrowed by G to reads, notes in the goal's scope and messages | yes |
| the goal template for drafts | the designer's charter; section 4's LLM choice; tracked; a report | yes (jig's `../protocol/configuration.md`, 3.1) |
| the coding agent's charter | charter 2, its instructions telling it to push early, finish with a Left list, mark a story that fails for code it does not own as ignored with the fault as its reason, and never fix what it does not own (N14); its run sized for the work (charter 2's 16 turns and 120 s are a small fix's) | yes, its text and run new |
| the foreman's charter | read-only (inspect and the forge's reads); a verdict of answer, finish or escalate | as a charter; its use by the change no (N1) |
| the forge's charters (kind 3) | 1, the change's: the coding agent's; 2, the reviews': unused; 3, the foreman's | kind 3 designed, not built; number 3 no (N1) |
| each change's LLM choices | producer, repairs and resolutions: gpt-6.1-sol at the spec's effort; foreman: gpt-6.1-sol, medium | no (N18) |
| the project's policy and rules | tool families with notes; delegation to the designer's, the coding agent's and the foreman's charters, the change procedure and the merge job; G's and designers' notes in the goal's scope; tasks and depth for the plan (N20) | as settings; their values new |
| landing rule for main | CI passed, up to date, enforced | yes |
| the repository's merge style | a merge commit | yes (`forge.md`, 4.4) |
| the project's checks | `.temper/pre-pr`: format, lints, the package's focused tests, the counted baselines | the convention yes; the file new |
| the goal branch's tier | the gate without its slowest suites, following collecting changes' branches, at most once per 20 minutes | no (N5) |
| main's tiers | the slow suites, following main | designed (`after-landing.md`); following main only no (N5) |
| questions | relayed; four per change | no (N1) |
| catch-up span | 30 minutes | no (N4) |
| the merge job | the project's job that merges and recounts its counted baselines | no (N7) |
| the profile's limits | sized for a hundred changes under one goal, its notes and its calls | to check (N12, N20) |

## 15. What is missing

| Gap | Where | Note | Status |
|---|---|---|---|
| `change` for a coordinator: several changes in one batch, a landing branch, dependencies, `collect`, `questions`, LLM choices, a producer's charter, `repairs` | temper | N21 | `change` takes words alone (`agent.md`, 4.3.1) |
| a charter's default wake policy in configuration | jig | N22 | missing |
| a change relays its agents' questions through a foreman, then up; a question bound | temper | N1 | contradicts `forge.md`, 8.1 and `agent.md`, 8.2 |
| a change that starts kept | jig, temper | N2 | designed, not built |
| a producer's checkout at the landing tip with its dependencies' heads; the early-start rules | temper | N2 | missing |
| the collecting change: its branch shared, written only by merges and its own update; it makes repairs and catch-ups; closed by its requester | temper | N3 | in words only (`forge.md`, 11.1); contradicts `forge.md`, section 2 and 8.5 |
| catch-up changes along the way | temper | N4 | contradicts `forge.md`, 10 |
| tiers that name the branches they follow; a goal branch's tier, its repair change, the queue not paused | temper | N5 | missing |
| no checking, no update, conflicts to the merge job, for changes into a collecting change's branch | temper | N6 | contradicts `forge.md`, 8.2 and 8.7 |
| jobs; the merge job | jig, temper | N7 | jobs designed, not built |
| sub-agents on a program backend, each naming its LLM | smith | N10 | open question (smith's `protocol/programs.md`, section 9) |
| the plan's text as notes: entry and scope bounds for a plan this size | jig, configuration | N12 | no entry bound named |
| `deliver` mid-run for a producer | temper | N13 | designed (`agent.md`, 4.3), not built |
| a change names its agents' LLM choices | temper | N18 | missing |
| implementers on gpt-6.1-sol | smith | N19 | the direct backend has it; Codex as a program is Later |
| limits for a goal of this size | configuration | N20 | to check |

jig's primitives cover the structure: goals, coordinators called fresh,
batches with dependencies, dependencies added later, kept tasks,
questions and answers, person tasks, holds and escalations, wake
policies, notes, the LLM choice with sub-agents, and templates. What
temper offers a run over them is narrower than the plan needs (N21).

## 16. Later

- Several goals at once, their branches overlapping: a report of the
  overlap to both coordinators.
- The order of work within one goal when capacity is short: repairs and
  resolutions before new producers (N9).
- An LLM choice per kind of wake, so a change's result read costs less
  than the cut (N11).
- A rehearsal: a goal that is never landed, its branch kept for people
  to read after the goal ends.
- What the reference leaves off, each a charter and a delegate or a
  gate when wanted: reviews of a change before it merges, a critique of
  the cut, skeleton changes for shared shapes, test writers, a structure
  review of the plan.
- Changes into a goal branch without a pull request each.
- Where builds run, and how many at once.

## 17. Open questions

- **A spec by reference.** G copies each spec from its note into a
  change's words, which costs the coordinator's output. A change could
  instead name its spec's note, rendered whole into its producer's brief
  at a pinned revision, as a charter's documents are. Worth it once the
  copies' cost is measured.
- **Should the goal branch's tier blame merges,** bisecting to the change
  whose merge brought a failure and sending it to that change's
  producer, rather than one repair for all? One repair matched the
  evidence; blame would need the tier's verdict machinery inside a goal.
- **Should a kept change start before its dependency's first push,**
  from its contracts alone? The rule waits for a push, so the producer
  builds on code rather than on a description of it.
