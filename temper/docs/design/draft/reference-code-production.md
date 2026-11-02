# Reference: code production

Draft, 2026-11-02. The reference way a project produces code on temper,
from a person's draft to a goal ready to land: the execution model, each
of its roles in jig's primitives and temper's mechanisms, and the
configuration that sets it up. It is the way a
project develops itself through temper: one repository, a person who
writes design drafts, designers who turn them into design, coding agents
who make the code meet it. How the goal lands is `reference-landing.md`;
`reference-case.md` walks both on one small goal; `notes.md` holds the
evidence and the reasons for each choice, N1 to N24, cited here by
number.

Unqualified, `forge.md`, `landing.md`, `after-landing.md`, `agent.md`
and `worker.md` are temper's `../domain/` documents; jig's and smith's
are named as theirs.

## 1. In one page

- **One goal per draft.** A person writes a design draft and starts a
  goal from it: a tracked agent task under the coordinator's charter, at
  Opus 5.5 and effort xhigh, its survey sub-agents at Opus 5.5 and high.
  It is called, not resident: each wake is a fresh run from its brief and
  inbox.
- **The goal holds its own branch** (N3), made at main's tip as the goal
  begins. Every change of the goal lands into it, designers' included,
  one merge at a time under the goal's hold; nothing outside the goal
  writes it. It lands into main whole, as one change, once the goal is
  done.
- **The plan lives where the code does** (N12). Designers write the
  design into the goal's branch: in this project, the documents and
  status files under `docs/design`; in another, what its designer's
  charter says. A spec is a change's words citing that design by path and
  section. The plan's structure is the tree of tasks. Notes keep the use
  jig gives them: what runs learn and pass on.
- **Designers are changes** (N13): a change into the goal's branch whose
  producer runs under the designer's charter at xhigh, surveying with
  sub-agents at high. Its design lands into the branch; its report lists
  its family's changes, and the coordinator makes them as one batch.
- **Each spec is a change** into the goal's branch, under the light rules
  goal branches have (N4): its producer is a coding agent starting from
  the branch's tip with the design in its checkout; it pushes as it goes
  and finishes with what it did and what it left. A change whose
  dependency is still being produced starts kept, its producer working on
  the dependency's pushed head (N2).
- **Producers ask.** A question goes to the change, which asks a foreman
  first; what the change's words and the design do not settle goes to
  the coordinator, and what the coordinator cannot settle, to the person
  (N1).
- **Holds escalate to the coordinator,** which releases, amends, cancels,
  or passes them up to the person.
- **Checked along the way.** A tier follows the goal's branch; a red
  reaches the coordinator as news, as a red after landing does, and the
  coordinator makes a repair change into the branch while other changes
  keep landing (N5). Main's landings come in by catch-up changes, whose
  producer is the merge job (N6, N7).
- **Done, closed out, handed to landing.** The close-out, a designer's
  change, writes what the goal leaves into the branch (N16); then the
  coordinator makes the goal's landing, a change into main whose branch
  is the goal's (`reference-landing.md`).
- **Mostly configuration** (section 14), with a few mechanisms the
  design lacks (`reference-code-production.status.md`): chiefly the
  coordinator's `change` with all its parameters (N22), the goal's own
  branch (N3), the change's question route (N1), its kept start (N2) and
  the merge job (N7).

## 2. Roles

| Role | Does | Executor | LLM in the reference |
|---|---|---|---|
| the person | writes the draft, starts the goal, answers what reaches them, decides what is passed up | a party | |
| the coordinator, G | the plan's shape, its changes, answers, holds, catch-ups, repairs along the way, the close-out, the landing, triage after landing | an agent task, tracked, the coordinator's charter; holds the goal's branch | Opus 5.5, xhigh |
| a designer | writes one family's design into the goal's branch, lists the family's changes | the producer of a change, the designer's charter | Opus 5.5, xhigh |
| a survey | reads the code and designs for one question, answers with a report | a sub-agent of a coordinator's or a designer's run, inspect only | Opus 5.5, high |
| the goal's branch, g | where every change of the goal lands; lands into main whole | a landing branch its goal holds (N3) | |
| a change | one spec, landed into g | the forge's change | |
| a producer | writes the code and its tests, runs the checks, pushes | an agent task, the coding agent's charter | gpt-6.1-sol, high or medium by spec (N19, N20) |
| a foreman | settles a question from the change's words and the design, or passes it up | an agent task, the foreman's charter, read-only | gpt-6.1-sol, medium |
| the merge job | merges a base into a branch, settling the conflicts the project's tooling knows | a job (N7) | none |
| a resolution | resolves what the merge job left, in a merge in progress | an agent task, the coding agent's charter | gpt-6.1-sol, high |
| g's tier | checks g along the way | the forge's tier (N5) | none |

Two levels of delegation, as jig has them (jig's `domain/core.md`,
7.4): surveys are sub-agents, cheap and gone with their run; changes and
their agents are tasks, durable, each with its own workspace and
authority. A designer is a task because what it makes is a commit, and
it lands as code does.

## 3. The tree

```
P, a person
└─ G   implement the draft                agent, the coordinator's charter; tracked; holds g
   ├─ S   the shared design               change into g; its producer under the designer's charter
   ├─ D1…Dm  one family's design each     changes into g, after S; producers under the designer's charter
   ├─ C1…Cn  the plan's changes           changes into g, after their family's design and each other
   │     ├─ producer                      agent, the coding agent's charter
   │     ├─ foreman, one per question     agent, the foreman's charter (N1)
   │     ├─ merge job                     job (N7)
   │     └─ resolution                    agent, the coding agent's charter
   ├─ Kk  a catch-up with main            change into g; the merge job its producer (N6)
   ├─ Rk  a repair along the way          change into g (N5)
   ├─ X   the close-out                   change into g; its producer under the designer's charter (N16)
   ├─ L   the goal's landing              change into main, its branch g (reference-landing.md)
   └─ after L lands: repairs into main    (reference-landing.md)
```

- **G is the goal** (jig's `domain/core.md`, 6.2): its priority orders
  the queues its changes join; its projection is the goal's issue; it
  hears its subtree's landings as news.
- **G holds g** (N3). Under its hold, each writer in its subtree writes g
  one at a time (jig's `domain/tasks.md`, 6.3): a merge into g, and, once
  L is made, L's update, repairs and resolution. G's own runs inspect
  alone (`agent.md`, 4.3) and take no writer slot. g is a landing branch
  with a queue, as any (`forge.md`, section 9), and the one that is held:
  a change into it from outside G's subtree is refused at its batch,
  naming G. As G closes, its hold's release deletes g if a change landed
  it (its tip reachable from that change's landing branch), and
  otherwise keeps it for a person to read, until deleted by hand or by
  the deployment's retention.
- **The plan is G's subtree** (jig's `domain/core.md`, 7.1): its
  structure is the tasks and their dependencies, which the engine runs
  and the store keeps as the plan's history; its text is the design in g
  (section 5); its authority is G's.
- **Every change is G's delegate,** made by G's `change` calls (N22), so
  G is the requester each question and escalation of the plan reaches.
  `change` takes, beside the words: the landing branch, `goal` for the
  goal's own branch or a branch's name; the branch, `mine` for its
  requester's held branch; dependencies, by task number or by a name
  given in the same call; several changes in one call, made as one batch
  (jig's `domain/tasks.md`, section 4); `questions` (section 8); its
  agents' LLM choices (N19); its producer, a charter the run may
  delegate to or the merge job (N13, N7); and `repairs`, a landing it
  repairs. The root checks each field as it decides the delegate; a
  charter's authority says whether its runs may set them: the
  coordinator's may, a chat's sets none.

## 4. A draft becomes a goal

- **The draft** is a design document on main (`<tree>/docs/design/
  draft/<name>.md`), written by the person, perhaps with a chat.
- **The person starts the goal** from the project's goal template for
  drafts (section 14), its words naming the draft and what to build from
  it.
- **The template makes it whole:** the coordinator's charter, the LLM
  choice (`{"main": {"model": "claude-opus-5-5", "effort": "xhigh"},
  "sub_agents": [{"model": "claude-opus-5-5", "effort": "high"}]}`),
  tracked at the project's priority, a report's contract, and an
  authority sized for a plan of up to a hundred changes (N21); its wake
  policy is the charter's default (N23).
- **Its own branch, by template** (N3). The template for drafts names
  *its own goal branch*, a symbolic resource the forge offers, resolved
  to `temper/<root>/g<root>` once the goal has its number, as a task's
  run branch is (`forge.md`, section 11). A small goal's template names
  none, and its changes land into main each. jig checks the batch as any
  (jig's `domain/tasks.md`, section 4). A goal made by hand may name it
  too, within its creator's authority. The choice is made with the goal,
  which cannot take a hold once active (jig's `domain/tasks.md`, 6.2).
- **G becomes active** at once, taking its hold, and the forge makes g at
  main's tip as the hold is taken (N3); failing, it holds G, saying why.
  G's first run begins the cut.

## 5. The cut

G's runs read: a checkout of g, inspect only, no delivery (`agent.md`,
4.3), its brief carrying the project's rules as the charter's documents
(`AGENTS.md`, the code's rules: temper's `../protocol/configuration.md`,
13.7).

1. **Read.** The draft, the guides, the designs it touches and their
   status files.
2. **Survey, if it judges it worth it.** Inspect-only sub-agents at
   high, one question each, side by side (smith's `domain/run.md`, 5.3);
   their answers are their reports. On Claude Code, smith serves its
   `sub_agent` through the program's MCP server, each child a program
   session of its own at the model and effort the LLM choice names
   (N10).
3. **Shape the plan:** the families of work, the shapes they share, the
   order between them, what was dropped and why, and the work the design
   already specifies, which needs no designer.
4. **First batch,** all into g:
   - **S, the shared design,** where families share shapes: a change
     whose producer runs under the designer's charter, its words the
     shapes to fix (names exact) and the goal's own rules beyond the
     project's guide, and where in the design each goes.
   - **One designer's change per family,** D1 to Dm, each after S and
     told its family's scope. Each starts kept on S (N2): its producer
     starts from S's pushed head, its merge waits for S's.
   - **The changes G specifies itself,** after S where they use its
     shapes.

   Then G parks (jig's `domain/engine.md`, 7.4).
5. **Designers design.** Each designer reads the draft, g's design and
   the code, and surveys with its own sub-agents at high where it judges
   it worth it. It writes its family's design into g: the sections the
   family's work meets, a status entry for each gap between them and the
   code, and the Later section for what the family leaves out. It pushes
   and finishes. Its report lists the family's changes, a few lines
   each: a name, the design sections and status entries it closes, what
   it owns, its dependencies and its effort. Its change lands into g as
   any change does (section 6), and its result, carrying the report
   (N15), wakes G.
   - **A shared shape it finds lacking** it writes itself, in its own
     design, where no other family uses it. Where another family does,
     or the draft's promise is at stake, it asks G, its question going
     straight up (N1): G answers, or makes a change revising S's design,
     adds it as the designer's dependency and answers that the designer
     waits for it.
6. **Each family as it is ready.** Each designer's result wakes G, which
   makes the family's changes as one batch, each after its dependencies,
   made or in the same batch. A dependency found later is added by
   amendment (jig's `domain/tasks.md`, section 7).

**A spec** is a change's words (`forge.md`, 8.1): what to make, citing
the design in g by path and section (`temper/docs/design/domain/forge.md,
4.2`) and the status entries it closes; what it owns and must leave
alone; its checks; its report. It is short: the design holds the
detail, and the producer's checkout holds the design. Its parameters
carry the rest: g as its landing branch, its dependencies, an LLM choice
for each of its agents' roles (its producer, repairs and resolutions;
its foremen; its reviews), each checked against its charter's models
and the change's authority as a batch's is, a role without one taking
its charter's default (N19), and that its questions are relayed (N1).

**A revision of the design** is a change into g, a designer's. Once it
lands, G amends the words of the live changes it concerns, naming the
revision; their producers' next runs merge g's tip first (N2).

**Where the plan's text lives:**

| The plan's | Lives in | Read by |
|---|---|---|
| families, order, dependencies, owners | G's subtree | G, the client's views |
| shared shapes and the goal's own rules | S's design, in g | every change, in its checkout |
| a family's design, its gaps, its Later | the designs and status files in g | the family's changes |
| a spec | a change's words, citing g | its producer and foremen |
| questions and answers | the store: question positions, histories | G, the asker |
| what runs learn: a slow check, a dead end, a trap in the code | notes, in the goal's or the repository's scope (jig's `domain/engine.md`, section 10) | every run of the project |
| G's reasoning between wakes, if it wants it | a note in the goal's scope, a hint the tree overrules | G's next runs |

## 6. A change into the goal's branch

Its way is the change procedure's (`forge.md`, 8.2), under the landing
rule for goal branches (N4):

1. **Producing.** A task for an agent under the change's producer
   charter, the coding agent's or the designer's, at the change's LLM
   choice, whose workspace starts from g's tip (with its dependencies'
   pushed heads, if it starts kept: N2). It reads the design its words
   cite in its checkout, runs the project's checks at delivery
   (`.temper/pre-pr`: smith's `domain/run.md`, 8.1), pushes as it goes
   (N14), and finishes with the head it pushed and its report: what it
   did, the checks it ran, and a Left list of what it found and did not
   own (N15).
2. **Opening.** Its pull request into g.
3. **No checking, no gating, no update:** the landing rule for goal
   branches requires none (N4).
4. **Queued, then first,** in g's queue, ordered by priority (one
   goal's, so by when each became ready), never while kept by a
   dependency (`forge.md`, section 9).
5. **Merged** at exactly its head, as a merge commit, under G's hold,
   once the forge finds it mergeable. A merge the forge refuses for a
   conflict goes to the merge job, which merges g's tip into the change's
   branch and settles what the project's tooling knows (N7); what is left
   goes to a resolution, an agent in that merge in progress (`forge.md`,
   8.4); the merged head then merges into g.
6. **Landed,** and done: no tier that judges landings follows g, so a
   change into it has no verification (N5). Its result wakes G: its
   result words carry, after its landing's three lines, its producer's
   last report, then what the landing passed, cut from the end to the
   result's bound and marked (N15).

## 7. Early starts

A change that depends on another change into g starts kept (jig's
`domain/connectors.md`, 6.1): it steps while its dependency is produced,
and its merge waits for it (jig's `domain/tasks.md`, 5.7). Its producer
starts early under N2's rules:

- once each pending dependency has a pushed head and none of them is
  kept by a dependency of its own;
- from g's tip with those heads merged in, in dependency order; a
  conflict there refuses the early start, and the change waits for its
  dependencies to land;
- once: a producer that asks, or fails for want of its dependencies'
  work, does not start early again: the next waits for them to land;
- a producer continuing from saved work merges g's tip into it first.

Done while kept, the producer's work waits: the change is not ready
until its last dependency has landed, then merges as any change. Its
head carries its dependencies' work as they pushed it, so a dependency
that is dropped takes its dependent with it: G cancels and remakes the
dependent rather than amending the dependency away, and the forge
refuses an amendment removing a dependency whose pushed head the
change's producer started from, naming it (N2). Designers start
early on S as producers do on code.

## 8. Questions

A change relays its agents' questions (N1).

1. **The producer asks** its requester, the change, in the question
   form, with its options and its recommendation, after pushing what it
   has. It is kept by its own question (jig's `domain/tasks.md`, 5.7):
   its run parks, no try spent.
2. **The change asks a foreman:** a delegate under the foreman's
   charter, read-only, its brief the change's words, the question and
   the change's pushed head, its checkout g's design. Its verdict is one
   of: *answer*, where the words, the design or the draft settle it;
   *finish*, where the work done holds and the rest belongs to the
   producer's Left list; or *escalate*. On the first two, the change
   answers its producer's question with the foreman's words. A
   designer's change has no foreman: its questions are about the draft,
   and go straight to G.
3. **Escalated, the change asks G** the same question, and is kept by it
   while it waits. G wakes and does one of:
   - answers it, in the answer form;
   - amends the change (its words widened to a file no other change
     owns, a Left item accepted) and answers with what changed;
   - makes a new change, adds it as a dependency, and answers that the
     work waits for it: the change holds the answer until the dependency
     has landed, and the producer's next run merges g's tip first (N2).
     A new change that revises the design is a designer's (section 5);
   - asks P, through a person task with the options as its choices
     (jig's `domain/people.md`, section 8), which keeps G from nothing;
     P answers in the inbox; the person task's result wakes G, and G
     answers the change.
4. **The change relays the answer down;** the producer's next run goes on
   from where it parked with the answer to read.

- **Bounded.** A change takes a number of questions (four in the
  reference); past it, the next is not given a foreman: the change holds
  itself, escalated to G. Released, the change relays that question to G
  with no foreman.
- **The setting.** A change's `questions` parameter, from its maker or
  the project's setting: `none`, the default, where its agents have no
  Message family and a question is answered that a change takes none;
  `foreman`, the route above; `up`, straight to its requester with no
  foreman, a designer's change's. With either of the last two, its
  producer, repairs and resolutions keep the Message family, a foreman
  is made per question, and the change's own page lists its open
  questions.
- **Answers stay in the store:** the asked task's question positions and
  the change's history (jig's `domain/tasks.md`, 8.2). A ruling that
  concerns many changes becomes design: a designer's change into g, and
  amendments of the live changes it concerns.
- **When G answers:** on each question, at once (N8).

## 9. Failures, holds and escalations

jig's rules, unchanged (jig's `domain/tasks.md`, 5.5 and section 9):

- **A producer's failures** are tried again within their class's tries:
  a peer gone, an agent crashed, a host lost. Its own failure (the task
  cannot be done as asked) holds the change at producing, with its words
  (`forge.md`, 8.4).
- **A change's bounds** (repairs, resolutions, each wait's stall, its
  questions) hold it, saying why; so does its authority spent (its tasks
  made: `forge.md`, 8.4; N21).
- **A budget spent** holds the task that spent it: it is the bound on a
  producer that loops.
- **Every hold escalates** to the nearest holder above that may decide
  it, skipping procedures and kept tasks: G, for the plan's changes and
  their agents. G releases it, amends then releases it (more budget, a
  wider scope, a dependency), cancels it, or passes it up to P with its
  reasons, for P to decide the same way.
- **A failed change's dependents are held** until G cancels them or
  amends their dependencies away (only those that did not start early on
  it: section 7).

## 10. Checked along the way

g is followed by a tier of its own (N5): the project's gate without its
slowest suites, defined as a tier is (`after-landing.md`, section 2).
Main's tiers follow main only.

- **One run at a time,** at g's newest tip, at most once per span; a run
  covers every merge since the last. It judges no landing: changes into
  g end at landed.
- **Green** says nothing to anyone.
- **Red,** rerun alone once (`after-landing.md`, 5.2); what still fails
  reaches G as news, as a red after landing does (`after-landing.md`,
  section 7): the tier, the failing tests, the run and the end of each
  failed job's log.
- **G triages it,** from the tree and the design: which change owns the
  failing files, and whether it has landed. A failure in a live change's
  files is that change's: G amends its words to say so. The rest makes a
  repair change into g, its words the failures, the log's end, and the
  live changes with what they own, to leave alone. The queue into g is
  not paused: other changes keep merging.
- **Reds of this tier never hold the goal,** and judge no landing; L's
  CI into main judges the whole (`reference-landing.md`).

## 11. Catching up with main

While the goal works, main moves with other landings. G hears those that
touch the goal's open work as news (`forge.md`, 7.3), its wake policy
pacing them (at most once in 30 minutes: N23), and takes them by a
catch-up change into g (N6): its producer the merge job, merging main's
tip into a branch made at g's tip and settling what the project's
tooling knows; what is left goes to a resolution in that merge in
progress; the result lands into g through its queue like any change.
One catch-up is live at a time; one that fails past its bounds holds and
escalates to G as any change does.

## 12. Ready to land

- **The plan is done** when every change has landed or been cancelled,
  every Left list is made into a change or accepted as left, and g's
  tier's last run is green or its red is owned by a live repair. G sees
  it when its last change's result wakes it.
- **The close-out, X** (N16): a change into g, its producer under the
  designer's charter, its words what the goal leaves for the next one:
  the status files, where the code still lags or departs; the designs'
  Later sections; and the draft's fate (folded into the design, kept, or
  removed). It lands into g as any change.
- **The landing, L:** X's result wakes G, which makes L, a change into
  main whose branch is g (N3). L goes its own way into main
  (`reference-landing.md`): CI at its head, queue, update, merge, tiers.
- **After L is made,** G makes a change into g only to answer L's needs:
  a move of g by G's other delegates is L's new head, checked again
  (N3).

## 13. What wakes G, and what it does

G's wake policy, the coordinator's charter's default (N23): questions,
escalations, failures, every delegate's result, words from its
delegates, its person's words, and news its connectors class as waking,
paced. A task made from a template takes its charter's policy; its
creator may narrow or widen it. Each wake is a fresh run, its brief the
goal, the plan's state and the messages that woke it (jig's
`domain/engine.md`, 7.4); a message that arrives while G runs is relayed
to that run instead.

| Wake | G reads | G decides |
|---|---|---|
| activation | the draft, g, the code | the plan's shape; S, the designers, its own changes (section 5) |
| a designer's question | the question, g's design | an answer; a change revising the design |
| a designer's result | the family's changes, its design in g | the family's batch |
| a question a change relays | the question, the foreman's reasons, the design | answer, amend, new change, ask P (section 8) |
| an escalation | the held task and why | release, amend, cancel, pass up (section 9) |
| a change's result | its producer's report and Left list | a new change for what is left, an amendment of a live change, or nothing; X once the plan is done (section 12) |
| a person task's result | P's choice | the answer it was waiting to give |
| a red of g's tier, as news | the tier, the failing tests, the log, the tree | a repair change into g, or an amendment (section 10) |
| a landing into main that touches the goal's open work (`forge.md`, 7.3) | the landing | a catch-up change into g (section 11) |
| X's result | its landing | L (section 12) |
| an escalation from L | L's hold and why | release, amend, a change into g first, cancel, pass up (`reference-landing.md`) |
| a red after landing, as news | the tier, the failing tests, the log | triage and a repair into main (`reference-landing.md`) |
| L's result: cleared | the landing | the report: the goal is done |

## 14. Configuration

What the reference sets, as settings (jig's `domain/engine.md`, section
17; temper's `../protocol/configuration.md`, section 13):

| Setting | The reference |
|---|---|
| the coordinator's charter | instructions for the cut, answers, holds, catch-ups, triage, the close-out and the landing; Claude Code; tools: `change` (N22), `delegate` (person tasks, a triage), `amend`, `cancel`, `release`, `decide`, `decide_escalation`, `propose`, `subscribe_forge`, `subscribe`, `note`, `recall`, `message`, `wait`, the workspace's inspect, sub-agents; starts fresh; waiting 0; its default wake policy (section 13); documents: the project's `AGENTS.md` and code rules |
| the designer's charter | instructions: write the design where the project keeps it (here `docs/design`, the status file beside each document, its Later section), cite by path and section, list the family's changes in the report; Claude Code; the workspace, delivery, sub-agents, `message` for questions, `note`, `recall`; documents: the project's design rules |
| the goal template for drafts | the coordinator's charter; section 4's LLM choice; tracked; a report; *its own goal branch* |
| a small goal's template | as jig's goal template, no branch of its own |
| the coding agent's charter | charter 2, its instructions telling it to push early, finish with a Left list, mark a story that fails for code it does not own as ignored with the fault as its reason, and never fix what it does not own, the change that fixes the fault removing the mark (N15); its run sized for the work (charter 2's 16 turns and 120 s are a small fix's) |
| the foreman's charter | read-only (inspect and the forge's reads); a verdict of answer, finish or escalate |
| the forge's charters (kind 3) | 1, the change's: the coding agent's; 2, the reviews': unused; 3, the foreman's |
| each change's LLM choices | a coding producer, repairs and resolutions: gpt-6.1-sol at the spec's effort; a designer: section 4's choice; a foreman: gpt-6.1-sol, medium |
| the project's policy and rules | tool families with notes; delegation to the coordinator's, the designer's, the coding agent's and the foreman's charters, the change procedure and the merge job; tasks and depth for the plan (N21) |
| the landing rule for goal branches | pattern `… branch temper`, open; no CI, not up to date |
| the landing rule for main | CI passed, up to date, enforced |
| the repository's merge style | a merge commit |
| the project's checks | `.temper/pre-pr`: format, lints, the package's focused tests, the counted baselines |
| g's tier | the gate without its slowest suites, following goal branches, at most once per 20 minutes |
| main's tiers | the slow suites, following main |
| questions | relayed through a foreman, four per change; a designer's straight up |
| the merge job | `merge_job`, the project's job kind that merges and recounts its counted baselines |
| the profile's limits | sized for a hundred changes under one goal, their reports and its calls |

## 16. Later

- Several goals at once, their branches overlapping: a probe's report of
  the overlap to both coordinators, and checkpoint landings
  (`reference-landing.md`, section 11).
- The order of work within one goal when capacity is short: repairs and
  resolutions before new producers (N9).
- An LLM choice per kind of wake, so a change's result read costs less
  than the cut (N11).
- A rehearsal: a goal that is never landed, its branch kept for people
  to read after the goal ends (N3).
- What the reference leaves off, each a charter and a delegate or a gate
  when wanted: reviews of a change before it merges, a critique of the
  plan's shape, skeleton changes for shared shapes, test writers, a
  structure review of the plan.
- Changes into a goal branch without a pull request each.
- Where builds run, and how many at once.
- Codex as a program backend, beside Claude Code (N20).

## 17. Open questions

- **A family's list of changes in g?** A designer's report carries it,
  bounded by the result's bytes (N21). A plan section in g, which the
  close-out removes, would carry any size and show in the branch's
  history, at the cost of a file that is not the project's. The report
  fits the run's largest family (eleven changes); revisit past that.
- **Should G catch up and repair along the way by judgement or by rule?**
  G decides both on news, which costs a wake at xhigh each (N11) and
  lets it choose a moment and an owner. A rule (a catch-up every span, a
  repair for every red) would cost no wake, and need a procedure under G
  to make the changes.
- **Should g's tier blame merges,** bisecting to the change whose merge
  brought a failure and naming it in the news, rather than leave G to
  triage? Triage matched the evidence; blame would need the tier's
  verdict machinery inside a goal.
- **Should a kept change start before its dependency's first push,** from
  the design alone? The rule waits for a push, so the producer builds on
  code rather than on a description of it.
