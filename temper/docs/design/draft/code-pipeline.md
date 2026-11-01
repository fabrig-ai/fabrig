# The code pipeline

Draft, 2026-11-01. Nothing here is adopted until the user approves it.
The first half of `landing-model.md` revision 4 (10-30), split off at
the user's word and brought loosely in line with how fabrig's lab runs
today (fab, the agents repository at 1dc102c): how work goes from the
person's word to a change ready to land. What follows ready is
`landing-pipeline.md`; `reference-case.md` runs both. It is one model
over jig's primitives (tasks, executors, messages, authority,
connectors: jig's `domain/core.md`, 3), with temper's configuration and
its forge connector on them; fabrig's way is one configuration
(section 5). Gap ids (L, J, F) are revision 4's, and new ones carry on
from them (section 6). Machines, slots and sign-ins are left out for
now (section 7).

Unqualified, `landing.md`, `forge.md` and `agent.md` are temper's
`../domain/` documents; jig's are named as jig's.

## 1. In one page

- **jig's primitives carry the work; temper configures them.** A goal is
  a tree of jig tasks; its agents are charters; its mechanics are
  temper's forge procedures and jig's jobs. jig knows nothing of code or
  branches; temper keeps its own state about them in the store, never
  in a repository, which holds only the project's code and documents.
- **An LLM for judgement, code for mechanisms:** planning, producing,
  answering, reviewing and resolving are agent tasks; scheduling,
  merging, checking, holding and routing are code.
- **No hand work:** the person talks to a chat through `temper ctl` and
  answers their inbox; everything else is tasks.
- **The work is a tree of goals** (section 2): the person and their
  chat; a goal per draft, whose agent plans and delegates the plan's
  changes in batches, the certain ones first; its delegates. Questions
  go to the requester; holds have owners and modes; news goes to roles.
- **A goal may land through a branch of its own** (2.6), checked along
  the way, kept up with main and checked whole before it is ready.
- **Checked before ready, by the same keys** (section 3): a change's
  deterministic gates run at its head merged onto main, before it is
  ready; their receipts serve its landing.

## 2. The work around a landing

### 2.1 The tree

- **The person** directs the work, sets budgets and priorities, and
  answers what is theirs to answer. **Their chat** is a coordinator: an
  agent conversation, made from a template the person chose (a
  director, or a department such as QA), whose charter lets it make
  goals and steer them (jig's `domain/people.md`, 7), reached through
  `temper ctl`. No hold over the project holds it (2.4).
- **A draft's goal plans:** its agent, under a planning charter, cuts the
  draft into parts and slices (stories that run end to end) and
  delegates each part's changes in batches with their specs and
  dependencies (jig's `domain/tasks.md`, 3 and 4), at once for the parts
  that are certain while it plans the rest. Several goals run in one
  project at once, each on its own draft. A part may instead be a goal
  of its own, owning one area of the code; depth is policy (authority's
  depth), not a fixed set of layers.
- **A small request** ("fix this") needs no plan: the chat makes one
  change, or a goal of one change.
- **Delegates** are agents (producers, reviewers, verdict tasks,
  investigators, testers), procedures (a change, a landing), jobs (a
  gate's commands) or people (a question, an approval). Each runs under
  a charter that names its model and effort; a delegate's charter is one
  its parent's authority allows.
- **Briefs, notes and results:** a goal starts from its brief, keeps
  notes while it works, and ends with a result that says what it did and
  what is left, which can be the next goal's brief.
- **Checks on a period:** a goal wakes on its delegates' results and on
  news, and a timer or a recurring task wakes it on a period as well
  (jig's `domain/tasks.md`, 3 and 8.1).
- **Standing rules are instructions:** contracts that cross parts written
  first, an interface step landed before its readers, one writer per
  package within a batch, what a producer may edit outside its paths.
  Such rules live in the charters, are judged by the agents that read
  them, and reach live tasks with a charter's new revision (jig's
  `MoveToNewest`, `domain/engine.md`, 17.4). Only what section 5 lists
  is enforced by code.
- **Work goes on while a goal is away:** a park of its own or a usage
  limit on its own runs stops none of its running delegates, and
  delegates it made ahead, ordered by dependencies, start as the ones
  before them end, without waking it.
- **Each goal's times are recorded:** its start, its first delegate, its
  changes' ready and landed times, its questions and how long they
  waited; a project's report of its flow is computed from them (gap
  L19).

### 2.2 A change

- **The change procedure** (`forge.md`, 8) takes a change from produced
  to landed in its landing branch: it delegates the producer, an agent
  task in a workspace, pushes what it commits, runs its checks before
  ready (section 3) and its review, and lands it.
- **The producer's run** finishes, parks on a question (the change is
  kept, jig's `domain/tasks.md`, 5.7), or fails. A failure's class
  (transient, the agent's own, lost) is code's, from the run and its
  tools, never from the agent's words; the tries per class are jig's
  (5.5). Past them the change is held and escalated to its requester.
- **What it left:** a producer that finishes may list what lies outside
  its change and stays red, for its owner or the goal's final check.

### 2.3 Asking and answering

- **Any task may ask the person** if its authority lets it make a person
  task addressed to them. A question carries its options and a
  recommendation (an instruction, not a mechanism).
- **A question to the requester keeps the asker** until it is answered,
  and moves with it. A change's question goes to its goal, whose agent
  answers it: from the spec's options when they settle it (perhaps by a
  cheap verdict task first), else by judgement, preferring to leave what
  is not the change's to its owner; what is the person's, it asks the
  person.
- **When a goal wakes for questions** is its wake policy: at once when
  one holds up other work, else in a batch by count or age (gap J14).
- **Questions to the chat stay with the chat** (gap J7): a delegate that
  asks its requester, the chat, is asking the chat, which answers itself
  or asks the person in its own words (gap J8). Where person answers are
  on, the person's answer goes straight to the asking task and the chat
  hears a notice.
- **An answer after its asker ended** goes to the asker's successor
  (2.5); with none, to its requester, as news naming the question.
- **On the forge, optionally:** a question projected as an issue, its
  comments answered or recorded by a reply task (`forge.md`, 12.1).

### 2.4 Holds

- **A hold has an owner,** the party or task that placed it: only the
  owner lifts it, or a party whose role stands above the owner's; a task
  above a party's hold may only propose its lift, which waits in the
  owner's inbox (gap J1).
- **Two modes:** **stop** cancels runs at once and holds the tasks;
  **drain** lets running runs finish their turn and running changes
  finish, landing included, starts nothing new, and holds each task as
  it goes idle (gap J2).
- **Three scopes:** a task's subtree; a project, which tasks made later
  inherit and the person's chat is exempt from (gap J3); and a kind of
  procedure's admission in a project, such as the landing pipeline's
  (gap J9).
- **Parked until an event** (gap J4): a task-owned hold that lifts
  itself when a named event comes: a landing on a branch, a task's end,
  a time.
- **Held past a bound:** a change past its tries, or a goal's branch
  past its bounds (2.6), is held and escalated to the task above it
  (jig's `domain/tasks.md`, 5.5 and 9), whose agent decides.

### 2.5 Handing over

- **A long-lived goal starts afresh from its notes** (gap J11): when its
  context nears its charter's bound, its run ends with a handover note,
  and its next run starts from its brief, its notes and the handover
  rather than its transcript.
- **A successor** (gap J6): a goal that ends can name a live goal as its
  successor, which receives its open questions' answers, its
  subscriptions and kept news.
- **A live goal moved under another task** (gap J5): its results,
  questions and escalations go to the new parent from then on.

### 2.6 A goal's branch

- **A goal may land through a branch of its own** (`forge.md`, 11.1; gap
  L21): its changes land into it through the change procedure; it is a
  change of its own, landed into main once its changes have. Or each
  change lands into main: a goal's setting (section 5).
- **Dependents start from it:** a change that depends on another starts
  once that one has landed into the goal's branch, or earlier, beside
  it, from its pushed branch, merged again before it lands (gap J13: a
  task started beside its dependency, jig's `domain/connectors.md`,
  6.1).
- **Conflicts:** one the project's tooling knows (a counted baseline,
  recounted) its tooling settles (`landing.md`, 5.3); any other is a
  resolution (`forge.md`, 8.4): the workspace starts from a merge in
  progress, an agent settles the files, the procedure commits.
- **Checks along the way and at the end:** a job runs the project's
  gates on the goal's branch on a cadence while changes land, and in
  full once all have; a red is a repair, a change of the goal's branch.
- **Kept up with main** by an update (`forge.md`, 10) on a timer, a
  conflict being a resolution.
- **Bounds:** past a number of refused updates, red final checks or
  rejections by the landing queue, the goal's branch is held and
  escalated to the goal (2.4).
- **Ready:** green in full, it is ready; `landing-pipeline.md` takes it.
- **Goals beside each other:** a goal hears when another goal's branch
  touches the same paths (merging both with main in memory); its
  planning brief lists the goals beside it (gap L23).

### 2.7 Who hears what

- **News goes to roles in the tree,** resolved when the news is made (gap
  J10): the change's goal, the goal above it, the person's chat. Which
  outcome goes to which role is the project's routing table (section
  5).
- **What no goal owns** (a red no open goal owns, a tier that cannot
  run, a finding not the code's) is news to the chat its template names,
  whose agent decides: start a goal, wait, or ask the person.
- **A delivery never changes an outcome:** news is an effect through the
  outbox, sent at least once.

## 3. Before ready

- **Ready** when its procedure marks it so: its producer's result is in,
  and its own deterministic gates hold at its **own candidate**, its
  head merged onto main's tip, uncommitted, in its workspace (gap L11).
  Judgement gates (a review, a person's approval) are blocking gates
  like any (`landing.md`, 7.2).
- **Its own deterministic gates** are the project's gates of the
  deterministic class (`landing-pipeline.md`, 3) over the packages the
  change reaches, and the repository gates whose paths it touches. They
  run as a job in the workspace; their results are receipts by key.
- **Measured gates wait for the prover:** tests run before ready (the
  producer's focused tests, a change of meaning's readers' tests) are
  advisory there: reported, never a receipt.
- **What the check meets** is the change's: a conflict onto main is a
  resolution, a red a repair, a fault of the workspace not the change's.

## 4. fabrig's code pipeline in the model's terms

What fab does today, its term in the model, and where fab departs
(**gap**: by hand, by an agent where the model has code, or temper's
data kept in the repository; temper does not copy it).

| fab | The model | Where |
|---|---|---|
| the user; the director, a session | the person; their chat | 2.1 |
| `fab call cut`, by hand (gap); the cut, one Claude call, releasing its plan early | a draft's goal planning and delegating in batches | 2.1 |
| plan.toml and task specs in the repository (gap) | the goal's plan and specs in the store | 2.1 |
| Codex implementer runs in builder slots | producers under the change procedure | 2.2 |
| a run's report line `Done`, `Question`, `Infra`, `Failed` (the transient class from the agent's words: gap) | finished, kept, failed by class | 2.2 |
| the foreman settling from a spec's options; the answer call | a verdict task; the goal answering | 2.3 |
| the user's page, forge issues with reply calls | the inbox; questions on the forge | 2.3 |
| a plan's stop, a note; a hand merge (gap) | a hold escalated to the goal | 2.4, 2.6 |
| the integration trunk, merges into it, the old-forms recount | a goal's branch | 2.6 |
| early starts; the implementer merging its needs again (gap) | started beside a dependency | 2.6 |
| checks along the way, the end check (`fab check`) | jobs on the goal's branch | 2.6 |
| catch-ups; catch-up repairs that merge and push themselves (gap) | updates, resolutions | 2.6 |
| the overlap probe | goals beside each other | 2.6 |
| signals on the page, read by a session (gap) | news to a chat | 2.7 |
| precheck, run by the implementer (gap); its receipt | the check before ready | 3 |
| the call review run, amend, settle | a review gate, a repair, a hold escalated | 2.2, 2.4 |
| skeleton calls (off in fabrig's plans) | a scaffold change, its readers started beside it | 2.6 |

## 5. Settings

| Setting | Scope | Default | fabrig's |
|---|---|---|---|
| a chat's template | party | a director | a director |
| a goal lands through its own branch | goal | no | yes for a draft |
| started beside a dependency | goal | off | on |
| review of a plan's batches | goal | off | off |
| scaffolds first | goal | off | off |
| a change's tries; a repair's | project | jig's | 4; 2 |
| answering: a verdict task first; when the goal wakes | project | no; at once | yes; at once for a goal's branch, else at 4 or after 30 minutes |
| checks of a goal's branch | project | at the end | every 10 landings or 45 minutes, without the fuzzy tests; in full at the end |
| updates of a goal's branch | project | before its checks | every 30 minutes |
| a goal's branch's bounds | project | 3 each | 3 refused updates, 3 red final checks, 3 queue rejections |
| what a producer may do unasked | charter | nothing | the arms and fields a type it changed forces; a scoped lint exception |
| models | charter | — | producers Codex at high; verdict tasks Codex at medium; goals and chats Claude |
| an agent's context | charter | measured | about 600k, measured (the user, 10-30) |
| questions on the forge | project | off | on |
| routing: outcome to role | project | 2.7 | the same |

A goal's settings are temper's kinds of setting on jig's settings store
(gap J15).

## 6. What temper and jig have, and what to build

Revision 4's gaps that this pipeline keeps, and new ones (L21 on, J13
on). jig's are knobs; none is a new primitive.

| Id | Gap | Grade |
|---|---|---|
| L11 | Checks before ready: a change's own deterministic gates in its workspace at its own candidate | design |
| L19 | Views and acts in `temper ctl`; the records a report is computed from | design |
| L21 | A goal's own branch: changes landed into it, checks on a cadence and in full, timed updates, bounds escalated | design (`forge.md`, 11.1) |
| L23 | Goals beside each other: overlaps as news, the goals beside listed in planning | design |
| J1 | Owned holds and proposals of their lift | built |
| J2–J4, J9 | Drain; a project's holds; parked until an event; a hold over admission | design |
| J5, J6, J11 | A live task moved; a successor; a fresh run from notes | design |
| J7, J8 | A question to the chat stays with it; the chat asks its person | conflict; design |
| J10 | An address by role in the tree | design |
| J13 | A task started beside its dependency (jig's `domain/connectors.md`, 6.1) | design |
| J14 | Waking on questions by count and age | design |
| J15 | Application kinds of setting (jig's `domain/engine.md`, 17.5) and a scope under a project | design |
| J16 | A chat from any template in `temper ctl`; `goal` choosing its template; `change` taking a typed spec and dependencies in a batch | design |

## 7. Later

- Machines, slots and sign-ins (the user, 11-01): workspaces as claimed
  resources with shares and lends (J12), run targets and calm builders,
  disk admission, custody of accounts.
- Leases on paths (L12): fab has none; overlaps are shown instead.
- Surveys as a stage of their own; test-writers and acceptance runs; a
  self-critique before ready; probes; a slice complete when its story
  test passes; a structure review writing consolidation changes.
- The web UI, a track of its own (the user, 11-01).

## 8. Open questions

- **H4, routing by role:** an address in jig for the nearest goal of a
  role above a task (2.7: recommended), or subscriptions each goal
  makes.
- **H5, hold scopes:** subtree, project and a kind of procedure's
  admission (2.4: recommended), or admission held by the serializer's
  own switch.
- **H8, handing over:** the same task's fresh run from its notes (2.5:
  recommended), or a successor task with a warm-up overlap.
