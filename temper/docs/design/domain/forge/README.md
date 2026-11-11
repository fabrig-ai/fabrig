# The forge connector

Provisional, 2026-10-04, revised 2026-11-02. How temper drives a forge,
Forgejo or GitHub: the forge as a connector on jig's contract (jig's
`domain/connectors.md`), with its resources, effects, topics, procedures,
requirements and reads; a forge connected or not, and removed; adopting
repositories, and what the picker offers; keeping up; a change's way to
landing and the requirements its merge meets; landing queues, a forge's own merge
queue among them; updates and conflicts; issues; and the ways temper
takes part in what others own. Its protocol layer, the component that
makes each call to a forge and hears its deliveries, is
`../../protocol/forge/README.md`. What is still open is listed in section 11.

## 1. In one page

- **What the forge holds:** git; pull requests as the unit of a change;
  CI; branch protection, as a second net; and a record people can read.
  From what temper owns there, only CI, people's approvals and drift
  come back: no comment on it is an input, and no label marks what
  temper tracks.
- **Live work comes from the store.** The connector holds a working set
  of what the forge owns of the resources live tasks name (pull
  requests' heads and states, CI on those heads, landing branches'
  tips), kept up by listing what changed, sooner on a webhook's hint.
- **Quiet on the forge.** temper reads what a reader needs, when it can
  have moved (`keeping-up.md`, 2 and 3): a branch nothing has pushed
  waits on its push, with `fallback` as its only poll; a hint wakes only
  the changes it can concern; a gather reads each commit's CI once and
  shares a fresh read its fellows ask with it; the repository's own
  facts are read once and kept; pages hold as many real rows as their
  bytes allow; and a read of something unchanged costs a `304`.
- **The change procedure** takes a change from produced to landed:
  produce, open, CI, gates, its turn in its landing branch's queue, an
  update from its base when it lacks the base's tip, and a merge at
  exactly its head; CI and the update only where its landing branch's
  rules ask them (`change/gates.md`, 2). Agents are woken for what needs judgement: producing,
  repairs, reviews, conflicts.
- **Landing rules are requirements.** A project's rules say what a merge
  into a branch needs: CI passed, the head up to date with the branch,
  gates, people's approvals. The forge judges each from its own facts at
  the merge's exact head, which the forge's merge is conditional on (`change/gates.md`, 2).
- **By queue or by turn.** A project lands by the queue below, its
  default, or by turn: landings across repositories, prepared, proved by
  receipts and fast-forwarded in the project's turn (`../landing.md`),
  which replaces steps 3 and 5 to 7 of `change/README.md`, 2, and `queues.md` and `README.md`, section 6, for
  its changes.
- **Landing queues.** Changes ready to land in a branch wait in line, by
  their goal's priority then by when each became ready; only the first
  is brought up to date and merged. Where the forge keeps a merge queue
  of its own on the branch, temper's queue decides only what enters it
  and when; the forge lands it, or ejects it, which is a repair (`queues.md`, 1).
- **Connected or not.** A forge is connected while temper runs, or not:
  the root knows which from the routing's events, rebuilt at each start.
  A forge not connected answers every call `NotConnected` at once, and
  adopts nothing. Its writer, given when it connects, is the forge
  client's own record, restored at start (`adoption.md`, 1).
- **A forge can be removed,** whole, by the deployment's owners: ended
  at once, its live work cancelled, its secrets and records erased, its
  box group stopped and its directory removed, each act kept until the
  removal is whole; nothing on the forge is undone but what temper ran
  (`adoption.md`, 1).
- **Adoption picks from what temper can reach:** a GitHub App's
  installations, what a connected Forgejo's API token can push to, or
  any repository of a managed or owned Forgejo, where adoption first
  gives temper's user its access (`adoption.md`, 2, 3). What it reads of the landing
  branch's rules, and the merge style, it says in words (`adoption.md`, 4).
- **Updates are lazy and mechanical.** A change takes its base when it
  is first in line, about to get a run, or at its check with its head
  already behind (`change/README.md`, 2, step 3), through the forge's own update; an LLM
  only resolves a conflict, in a workspace where the merge is in progress.
- **Every landing is news** for every goal working in that repository;
  it wakes a goal's coordinator only when it overlaps the files of that
  goal's changes.
- **Issues stay, one per goal,** as a projection people can read: the
  goal, its plan as a task list, and its milestones. Pull requests refer
  to it.
- **Plugged in as it is.** temper adopts repositories with whatever
  access it is given; webhooks, labels, protection and CI are all
  optional. Nothing is forced: pushes are fast-forwards, and merges are
  made at exactly the head decided on.

## 2. What it gives

- **Resources,** named `forge` `<host>` `<owner>` `<repo>`, then:
  - `branch` and the branch's own segments: `… branch temper 7 c42`;
  - `pull` and a number; `issue` and a number;
  - nothing beyond, for the repository itself.

  The host, the owner and the name are those the repository was adopted
  with (`adoption.md`).
- **Holds** (jig's `domain/connectors.md`, 3.3): a branch under temper's
  prefix takes an exclusive hold, held by the task it is named after: a
  change, a task's runs, a landing, or a goal for its own branch (`branches.md`, 1).
  A hold already taken makes its task wait (jig's `domain/tasks/holds.md`,
  2), but a goal's own branch's, which refuses, naming the goal: no
  two goals share one. Landing branches, pull requests, issues and the
  repository itself are shared and take none: merging into a landing
  branch is ordered by its queue (`queues.md`). A goal's own branch is a
  landing branch too, held and ordered by its queue at once (`branches.md`, 1).
- **Symbols** (jig's `domain/connectors.md`, 3.1): 1, *its own goal
  branch*, offered on a tracked task only (`branches.md`, 1); a batch naming it for
  an untracked task is refused, naming the symbol.
- **Effects:** open, edit, update, merge, close and reopen a pull
  request; post a review, a comment or a commit status; create, edit,
  close and reopen an issue; set reviewers; create and delete a branch
  (section 4). A run's push is the worker's, over git, under its task's
  hold (`../worker.md`, section 5).
- **Kinds,** for authority (jig's `domain/authority.md`, section 4),
  numbered: 1, reads; 2, a run's push; 3 to 15, and 17, the effects, one
  per row of section 4's table (16 and 17 are also a landing's,
  `../landing.md`, 11). No kind implies another: each write a task means
  to make needs a grant of its own kind. A change is given kind 17 on its
  branch wherever it is given kind 2 there, since a push of the same head
  would run the same CI (`change/README.md`, 2, step 3). Landing rules govern kind 4,
  the merge (`change/gates.md`, 2). None forces a push (`change/gates.md`, 2).
- **Topics:** landings on a branch; CI on a head; a pull request's
  state; the comments and reviews on a participating object (section 5).
- **Procedures:** the change (`change/README.md`).
- **Requirements:** CI passed, up to date, a gate, an approval, which
  landing rules name as judges of the merge (`change/gates.md`, 2).
- **Reads an agent may call:** each of the forge client's reads but
  locating a repository: a page of the issues and pull requests changed
  since a time; an issue or a pull request and its comments; a pull
  request, or the one for a head and a base, its reviews, a review's
  remarks, and its files; a branch's head, and the branches; the
  statuses and checks on a commit, and a failed job's log; a comparison
  of two commits; a file at a head; a repository's protection, settings
  and collaborators, and a user's permission. Each answer is cut to its
  limit. The host tools that serve them, and those for effects and
  topics, are `../agent/tools.md`.
- **Effects offered to agents** (jig's `domain/connectors.md`, 4.5):
  creating an issue, posting a comment on an issue or a pull request,
  posting a commit status. What the change procedure owns (opening,
  updating, merging, reviewing as a gate) is not offered: a run that
  wants a change landed delegates it.
- **Brief sections,** of three kinds, each at the change's head: the pull
  request, its files and what differs from its base, or, for a
  resolution and a semantic repair, what landed in the base, with the
  files in conflict for a resolution; CI's failures, each with its
  failed step and the end of that step's output; reviews' remarks. And
  a fourth for any run, a document its charter names, at its branch's
  head (`../../protocol/configuration/settings.md`, 6).
- **Workspace items:** for each repository a run works in (../root.md,
  6.1), a checkout: where it starts (a landing branch or the
  repository's default, a branch, saved work, or a merge in progress),
  whether it may be written and the branch it pushes, and the git
  identity it is fetched and pushed with (`../worker.md`, 4.1;
  `../../protocol/credentials.md`, section 4).
- **Documents and parts** (jig's `domain/connectors.md`, 5.5): what
  temper's client shows of the forge, made from the connector's records
  and working set: a change's way to landing, its result and its hold,
  the queues, the repositories and their landing rules, goals' issues,
  findings and the forge itself (`../../client/forge/served.md`).
- **What temper owns there** (jig's `domain/model.md`, 8.1): what it
  creates, recorded in the store by key. On the forge, ownership shows
  as authorship, by temper's own user (on GitHub, its App's bot user),
  and as temper's branch prefix, which branch protection can reserve to
  it.

## 3. Structure

```
temper-engine-domain-forge              the connector's top: resources and holds, adoption, topics, judges, briefs, routing; the effects and their identities
├── skein-forge-client                  the working set, keeping up, fresh reads, the request budget, effects' keys, write lanes and recovery (skein's forge-client.md)
├── skein-forge                         the vocabulary the top and the client share: calls, answers, errors, hints (skein's forge/README.md, section 3)
├── temper-engine-domain-forge-change   the change procedure and the landing queues: decisions over facts
├── temper-engine-domain-forge-landing  the landing by turn: decisions over a landing's state (landing.md)
├── temper-engine-domain-forge-issues   the goals' issues: what each projects, when to write it
└── temper-engine-domain-forge-tiers    the tiers' verdicts: decisions over a tier's record, its run's report and receipts (after-landing.md)
```

- **The top** keeps the forge's resources as the project knows them
  (roles, what temper may do there), the holds on branches, the topics
  and their subscribers, the changes' procedure states, the issues'
  projection states, and the landing rules' gates and judges' parameters.
  It routes between its children and translates to and from the root.
  It names the client's live resources, owns the effects and their
  identities (section 4), and hands the client their bytes and its
  namespace, the deployment's id, before any restored entry or live
  resource, by which time the client has restored each forge's writer
  from its own record (section 9). It gives the client a forge's
  writer when the forge connects, and keeps each forge's connection as
  the root tells it (`adoption.md`, 1).
- **`skein-forge-client` is a capability:** it knows the forge's API as
  `skein-forge`'s vocabulary states it, the working set, the request
  budget, and how a write is made once. Its owner, how it is driven and
  what it never knows are skein's `forge/client.md`, sections 2 and 9.
- **`change`, `issues` and `tiers` are policy:** given a change's state
  and the facts, a goal's state, or a tier's record and what its run
  reported, they say what is due. They keep nothing between calls; their
  states are the top's, committed in the store.

## 4. Effects

Every effect is committed into the outbox and made by `client`, in
commit order per lane (a pull request, an issue, a repository) and one
at a time on each (jig's `domain/connectors.md`, 4.2). The top owns the
effects and their identities; the client's keys, its write lanes and its
recovery are skein's `forge/client.md`, section 7. temper's effects:

| Kind | Effect | What it is | Recovery class (jig's `domain/connectors.md`, 4.3) | Found again after an uncertain failure |
|---|---|---|---|---|
| 3 | open a pull request | a creation: from a branch into its landing branch, with a title, a body and a key | conditional: only while no pull request for its branches is open; Forgejo keeps it, closed or not | by its branches; the change accepts an open pull request, or one with its recorded number or pushed head |
| 10 | edit a pull request's title, body or base | a set | idempotent | written again |
| 11 | update a pull request from its base | a transition: the base's tip merged into its branch, or its commits rebased onto it (`../landing.md`, 5.2), by the forge | idempotent: every copy leaves its head containing the base, or changes nothing; a late copy may move the head (section 6) | its head contains the base tip it asked for |
| 4 | merge a pull request | a transition, conditional on its exact head; or, where the forge keeps a merge queue on the base, the pull request queued there (`queues.md`, 1) | conditional | merged, and at which commit; or in the forge's merge queue |
| 12 | close, reopen a pull request | a set | idempotent | written again |
| 5 | post a review | a creation, with a key | unrecoverable | by its key, among the reviews after the entry's start |
| 7 | post a comment | a creation, with a key | unrecoverable | by its key, among the comments after the entry's start |
| 6 | post a commit status | a set: a context on a commit | idempotent | written again |
| 8 | create an issue | a creation, with a key | unrecoverable | by its key, among the issues changed since the entry's start |
| 13 | edit, close, reopen an issue | a set | idempotent | written again |
| 14 | set a pull request's reviewers | a set | idempotent | written again |
| 9 | create a branch at a commit | a creation, named | unrecoverable: another hand can delete it, and nothing then shows it was made; but a goal's own branch, which temper alone makes, is conditional: made again while none of its name exists (`branches.md`, 1) | the branch exists at that commit |
| 15 | delete a branch | a set: temper's own branches only | idempotent | it is gone |
| 17 | rerun a job | a transition: a failed job of the change's CI at its head run again, with the jobs that need it, as its next attempt (skein's `Write::Rerun`; `change/README.md`, 2, step 3) | unrecoverable: the forge checks no attempt | the job's attempt moved past the one named; at that attempt still, completed, it was not made, and the change decides again on a fresh read |

- **The outbox is bounded** by the connector's `entries` limit: no entry
  is committed past it, and room frees only as an entry leaves it, by its
  write's outcome or its withdrawal. What asks for an effect while it is
  full waits or is refused by its own rule; a goal's projection waits
  (`issues.md`).
- **Each row is a kind of its own** (jig's `domain/connectors.md`,
  3.1): a policy that grants writing issues grants each kind it means.
- **A write in flight is a row of its own,** `Flight { task, entry }`
  (`../../protocol/store.md`, section 2): saved in the decision that
  commits an entry's attempt; kept while the write is unresolved, an
  uncertain outcome, a retry, a merge in the forge's queue (`queues.md`, 1) or the
  restart's settlement (section 9); erased in the decision that
  settles the entry for good or holds it for a person, and saved again
  before a held entry is made again. So the store shows which writes a
  stop left unsettled without decoding an entry. temper's patch set
  names its range (jig's `protocol/store/opening.md`, 3). temper keeps one
  schema family for all its records, so any patch of temper's refuses
  while one is there, whichever record it changes: a crash or a stop
  past its bound is rare, and the remedy is one more settled stop. The
  forge's restore links a `Flight` row to its entry, and one whose entry
  it does not hold is a writer's bug, a link that cannot be made, which
  refuses the start (`../root.md`, 8.1; jig's `domain/engine/restart.md`
  ).
- **Classes follow what the forges offer** (skein's `forge/client.md`,
  section 7.3; the facts, skein's `forge/forgejo.md`, section 2, and
  `skein/docs/design/forge/github.md`, section 13). Neither Forgejo nor GitHub keeps a
  key it would refuse a second creation by, so no kind is keyed. A pull
  request is opened only while none is open for its branches, and
  each forge keeps every pull request, closed or not, so the
  client can find the state it leads to, closed by another hand included.
  Branch names can repeat across deployments and a repository's past.
  A change accepts an open pull request on its branches. A closed or
  merged one is its own only when its number is the one the change
  recorded, or its head is a head the change pushed (`last_head`). Other
  closed or merged pull requests are Missing to the change; their numbers
  are never recorded. The change can then produce and open a new pull
  request on the reused names. Once it records a number, its fresh reads
  use that number rather than the branches, so an older pull request
  cannot replace its own. Its own closed pull request is drift and is
  reopened on release (`change/README.md`, 3). A branch is created
  only while none of its name exists, but another hand can delete it, and
  nothing then shows it was made: its creation is unrecoverable.
- **An unrecoverable creation is looked for first:** an uncertain review,
  comment, issue or branch found by its key, or by the state it
  leads to, is made; one not found by its deadline holds its task for a
  person, and is never sent again by temper on its own. The client makes
  the find (skein's `forge/client.md`, section 7). A goal's own branch is
  the exception: found at its commit it is made, found at another its
  making fails for good, and not found it is made again (`branches.md`, 1).
- **Keys are carried** in what temper creates: a marker at the head of a
  body, a comment or a review, hidden where the forge renders it (skein's
  `forge/README.md`, section 5; temper's word and namespace,
  `../../protocol/forge/README.md`, section 3). A pull request's branches locate
  a candidate; the change owns it when it is open on those branches or
  the change knows its number or pushed head, as above. An entry's start,
  and the find that reads only what came after it, are the client's (skein's
  `forge/client.md`, section 7).
- **An effect's identity** is its purpose, and the top encodes it. The
  client takes the bytes whole and never inspects them (skein's
  `forge/client.md`, section 7). A key is the client's frame (version 1, the namespace's
  length as two bytes big-endian, the namespace) and then the purpose's
  bytes: a tag, and its fields in order, each number big-endian. The
  namespace is the deployment's id, 16 bytes. A purpose is derived from
  what the effect is for, never from when it was asked or its entry's
  number, so asking again for the same thing gives the same bytes (jig's
  `domain/connectors.md`, 4.4):

  | Tag | Purpose | Fields, with their bytes | Bytes with the tag |
  |---|---|---|---|
  | 1 | a call: one agent's call in one task attempt | task (8), attempt (8), completion (4), position (4) | 25 |
  | 2 | a step: one purpose within a task's procedure | task (8), purpose (2) | 11 |
  | 3 | a projection: one part of a goal's issue | goal (8), part (1), number (8) | 18 |
  | 4 | a release: cleanup of a task's named resource | task (8), the resource's length (2), the resource | 11 and the resource |
  | 5 | a history milestone: one task's milestone in a goal's issue | family (1), goal (8), task (8), position (8) | 26 |

  A part of a projection is the issue's open (1), its body at a local
  revision (2), the plan accepted (3), a plan revision by its history
  number (4), a change landed (5), a change held (6), a report kept (7),
  the goal finished (8) and the issue's close (9); the number is the
  revision, the history number or the change's, and 0 for the others. A
  history milestone's family is a task's lifecycle position (1) or a
  task's semantic revision (2, whose position is the revision). So a
  call's key is 44 bytes, and a key is refused when it passes the
  client's `op_bytes` or the namespace is empty.
- **Conditions travel with the effect:** a merge names the head its
  decision saw, which the forge checks, and a stale head is refused, not
  merged. It names the base too: the pull request is read afresh first,
  and one retargeted since the decision is not merged.
- **A merge may be queued:** on a base the forge's merge queue guards,
  the merge answers `Answer::Queued`, final for the call and not for the
  entry: neither a merge nor a refusal. The entry is the client's until
  a read finds the pull request merged or out of the queue (skein's
  `forge/README.md`, 3.3; `skein/docs/design/forge/client.md`, 7.7): it is never sent again, and
  it holds its lane, so a comment or a review on that pull request waits
  until the queue decides. What follows is `queues.md`, 1's. Forgejo never answers
  it.
- **Failures:** a merge refused for a conflict, an update refused for a
  conflict, a pull request with nothing to merge, a protected branch and
  a forbidden write are each typed, and go to the procedure that asked,
  never retried blind. A forge not connected is not a failure of the
  procedure's: the entry fails at once, `NotConnected`, and is committed
  again once the forge connects (`adoption.md`, 1). A forge away is none either: an
  entry the client fails `Unavailable`, never sent, is committed again
  after a wait that grows (`keeping-up.md`, 5).
- **A stop settles first** (../root.md, 8.2): told to settle, the connector
  tells its client to (skein's `forge/client.md`, 7.8), so no write
  starts and none is tried again; a committed entry's notification is
  retired unmade, its row durable; no change steps. The writes already
  sent finish, with the reads that prove them, and their outcomes are
  saved as ever. It says it has settled when its client does: at once if
  nothing sent lacked its outcome, or at the client's bound, with each
  write still without one named by its entry and its task, which the
  next start settles as its class says (section 9).

## 5. Topics, and landings as news

### 5.1 Topics

| Topic | Published when | Read by |
|---|---|---|
| landings on a repository's branch | a landing branch's tip moves | goals working in that repository; landing queues; tiers (`../after-landing.md`, section 3); a subscription naming trailers (`../after-landing.md`, section 8) |
| CI on a head | a head's CI, as `keeping-up.md`, 3 reads it, changes | changes; tasks that subscribe |
| a pull request's state | opened, closed, merged, its head moved, its mergeability known | changes; tasks that subscribe |
| comments and reviews on a participating object | another party writes there | the task dealing with it (section 7) |
| a project's findings | a finding is filed, or its labels change | the project's triager (`../findings.md`, section 6) |

The change child domain reads its facts directly and needs no
subscription; topics are for tasks whose executors are agents, and for
goals.

### 5.2 Every landing is news

- **For two kinds of goal:** the goal whose change landed, whose step is
  done; and every other active goal with changes open or planned in the
  same repository, whose base has moved. Concurrent plans meeting this
  way is the ordinary case, not an exception for outside activity.
- **A goal hears of landings** through the forge's topic for goals
  (jig's `domain/connectors.md`, 5.4): the landings on the branches its
  subtree's changes target, which the core subscribes it to as the first
  change into each is made, until the goal ends. A goal ended by its
  closer (`branches.md`, 2) takes no subscription to the repository's default
  branch's landings, including for a repair there. Landing such a goal's
  held branch creates no main-landing subscription, either at the change's
  start or when its plan is projected. Reds still reach the goal through
  its branch's tiers (section 6).
- **Those are a goal's only topics from the core.** CI on its changes'
  heads is each change's own, read by its procedure (5.1); the goal hears
  a change's end or hold as any requester hears its delegate's (jig's
  `domain/tasks/messages.md`, 1), and a run that wants a head's CI subscribes to
  it. So the topics the core keeps for a goal number its landing branches,
  however many changes it makes, and its run keeps room in its task's
  subscriptions for its own: a person task it waits on, a repair before a
  held landing's release, its timers (`../agent/tools.md`). A goal's
  subscription to CI on a head that the core holds at restart is dropped
  there, its jig row with it.
- **A landing temper did not make** is news like any other: what landed
  is read, a pull request or a range of commits.

### 5.3 Which landings wake

Not every piece of news wakes. The connector classifies each landing for
each subscribed goal (jig's `domain/connectors.md`, 5.3), without an
LLM:

- **its own change landed:** kept; what comes after it is due
  mechanically, through the goal's dependencies;
- **another landing overlaps its work:** wakes. Overlap is the files the
  landing touched against those of the goal's own open pull requests at
  their heads, and, for changes not produced yet, the paths their specs
  name, as a hint;
- **no overlap:** kept, for its next run to read;
- **overlap unknown** (a landing too large to list, a read that failed):
  wakes, since a wasted turn is cheaper than a missed conflict.
- **A hold parked until a landing** (jig's `domain/tasks/lifecycle.md`, 10):
  subscribed to the landings on a branch, with path prefixes or none; a
  landing wakes it, and so lifts it, when one of its files lies under a
  prefix, or with none named, or when the overlap is unknown.

The files are read from the forge (a pull request's files by their paths
alone, `PullFiles` without `sides`, or the comparison of the branch's
old and new tips, by names alone: skein's `forge/README.md`, 3.2 and
3.3), cached per head and cut at a limit past which
the overlap is unknown. A comparison names at most `forge.client.rows`
paths and counts the rest (`changed`): one that leaves a path out, an
unusable answer or an uncertain completeness makes the overlap unknown
and wakes every affected subscriber, and its paths are a landing's files
for workflow discovery too (`adoption.md`, 7). A pull
request's pages count only while fresh reads before and after them name
the same head; a head that moved voids the listing.

## 6. Updates: lazy and mechanical

Under a project's queue; by turn, a landing prepares its candidates
instead (`../landing.md`, section 5).

- **Lazy:** a change takes its base when it is first in its queue, about
  to get a run for another reason (`change/README.md`, 2), or at its check with its head
  already behind and CI not concluded there, once for a head a push gave
  it (`change/README.md`, 2, step 3). It never does only because its base moved.
- **By its landing branch's rule:** only where the rules covering its
  landing branch ask up to date (`change/gates.md`, 2, `Requires`). Under rules asking
  none it is never updated, first, at its check or before a run, and a
  conflict with its base holds it (`change/README.md`, 2, step 6).
- **Mechanical:** the update merges the base into the change's branch,
  with no LLM. The forge makes it itself (its pull request's update, by
  a merge), so it is an effect, as a merge is. A merge rather than a
  rebase: the branch only moves forward, so it is never forced, and the
  heads before the update stay the change's own.
- **Made when the forge says, read afresh.** Forgejo has made the
  update when it answers; GitHub accepts it and makes it after (skein's
  `forge/github.md`, 5.2). Either way the update is done once the head,
  read afresh, contains the base's tip it asked for, which is also how
  its recovery finds it (section 4), and the new head is heard as any
  push. That head is the change's own, recorded as its branch's head in
  the step that settles the update, never drift (`change/README.md`,
  3, *Its own writes are its own*).
- **Lost, and made again.** An update whose answer is lost is looked for
  by that read and, not found after its lifetime, checked and made again,
  up to `write_attempts` (skein's `forge/client.md`, 7.5). It holds no one.
- **Let go.** An update settled `Failed` with `Timeout` or `Unavailable`,
  or with `Stale` from its check, says nothing of the change: the forge
  did not answer, or what it was decided on has gone, perhaps to a copy of
  its own. The top keeps its account and reports no result, and a change
  updating or preparing (`../landing.md`, 5.2) with no result and none
  outstanding decides again on a fresh read of its head and base, the
  base being the tip the update asked for, though the branch has moved
  on since:

  | Its head | Holds the base | It |
  |---|---|---|
  | where it was | no | updates again, counted in `updates` |
  | where it was | yes | goes on at that head |
  | moved | yes | takes it as its update, as if made |
  | moved | no | holds, `BranchMoved`: no update of its own shows there |
  | any | not yet known | waits, up to `stall` |

  So it never spins, each new update spending one of `updates` and up to
  `write_attempts` lifetimes, and never waits on nothing. Past `updates` it
  holds, `Updates`, a person told why in the last write's account
  (`update pull request: forge timed out; ...`).
- **Other failures** of an update, apart from a conflict below, are a
  person's: a protection or a forbidden write holds it, `Failed`, in its
  account. A release of it, or of `Updates`, drops the result and decides
  again as above, its count afresh.
- **What a late copy costs.** Once the change has moved on, a copy that
  moves the head is drift (`change/README.md`, 3): held, and its release takes the head and
  checks it, so a person and a CI run; gates ask again. A merge names its
  exact head (section 4), so a copy lands nothing unchecked. A rebase's
  copy can also drop a push made since it read the head (skein's
  `forge/client.md`, 7.3): the same hold, and a repair so dropped finds
  its failure again when the head is checked. A push by another hand so
  dropped is that hand's to make again, the hold saying the branch moved.
- **A conflict** makes the forge refuse the update, and the change is
  resolved (`change/repairs.md`): by the merge job, where its project names one, then
  a resolution for what the job leaves.
- **A catch-up** takes a base's commit into a branch that no update
  reaches, a goal's own (`branches.md`, 1): a change into that branch, made by the
  goal's coordinator naming the commit (`../agent/tools.md`, 1), its producer
  the merge job, which merges the commit into a branch made at the
  landing branch's tip (`change/README.md`, 2, step 1); its project must name a
  `merge_job`. It lands as any change into the branch does, and nothing
  makes one but its requester's call.
- **A semantic conflict** is CI failing on a change's head after a clean
  update. The name records that order, not proof that the base caused
  the failure. Its repair's brief carries the failed check and its output
  before the base merged in and what landed in it (`change/repairs.md`). While CI fails
  on the base's tip, the failure is
  not the change's: its queue is paused (`queues.md`), and the change
  waits for the branch's repair instead.
- **Gates carry over** a clean update temper made, where their
  `freshness` is clean (`change/gates.md`, 1); after a conflict an LLM resolved, they are
  asked for again.

## 7. Participating objects

Not needed first, and kept possible with one mechanism (jig's
`domain/model.md`, 8.2), for three cases: outside contributions to a
project temper works on; a repository where temper may not merge; and
temper's own contributions upstream (section 8).

- **What people write on what temper owns is no input.** A comment on
  temper's pull request or issue reaches no task; the pull request's
  body says where to reach temper instead. The one fact temper reads
  there is a review's verdict by a person a landing rule's approval
  counts (`change/gates.md`, 2); where that person asks for changes, the reviews at that
  head and their remarks reach the repair the request starts, as quoted
  words (`change/gates.md`, 1).
- **Outside contributions** to a project temper works on: their issues
  and pull requests, which a task may triage and review. A task given
  one subscribes to its comments and reviews; what others write reaches
  its inbox as news, in order, from a position the connector keeps.
  Outside contributors are not the project's parties unless adoption
  seeded them (`adoption.md`) or they sign in: what they write is a
  participant's words.
- **A repository where temper may not merge:** a change there is ready
  when its CI and gates hold, and then waits for someone else's merge,
  which is a landing like any other. People's reviews on its pull request
  are news for the change; a request for changes is a repair, given its
  remarks.
- **What temper writes** there (a reply, a review) is a participant's:
  drafted by an agent's task, and, where the project's policy says so,
  proposed to a person before it is posted.

## 8. Upstream contributions

Not needed first, and kept possible:

- **A fork,** on the upstream's forge where it can be, so that upstream's
  CI runs on it unchanged and the pull request is native there. Its main
  follows upstream's, fast-forward only, and temper's own changes land
  into a work branch in the fork: a goal's own branch (`branches.md`, 1).
- **The submission is a task:** brought up to date with upstream, its
  commits shaped to upstream's conventions, described and signed off,
  and accepted by a person before anything is posted, often under that
  person's own identity.
- **Once posted,** its pull request is a participating object
  (section 7): a task reads the maintainers' reviews, updates the
  branch, and drafts replies a person approves.

## 9. Restart

At a restart (jig's `domain/engine/restart.md`; jig's
`domain/connectors.md`, section 14) the connector is given its records
from the store: the projects' repositories and what temper may do there,
the live tasks' resources and holds, the kept tasks' names, the changes'
and issues' states, and its outbox's entries with their attempts. The
deployment's id is given to the client first, as its namespace, before
any restored entry or live resource; the client restores each forge's
writer from its own record before any entry too (`adoption.md`, 1). The client's
part is skein's
`forge/client.md`, section 8: what it reads afresh before anything else
steps, how it settles an outbox entry as its class says, and where its
listings start. That fresh read, each adopted repository's, covers what
changed while the engine was down on GitHub too, since no delivery
log's guid is kept across starts (`../../protocol/forge/README.md`, 4.1).

Each forge starts not connected, whatever it was before the restart, and
is connected as the routing tells the root (`../../protocol/forge/engine.md`, 1, and `../../protocol/forge/README.md`, 7.1
); an ended forge stays ended, and each act its removal has not
heard answered, its secrets erased or its group removed, is asked again
(`adoption.md`, 1); the adopted repositories of each managed or owned Forgejo are
named to the routing for its reconcile (`adoption.md`, 2). Each `Names` row of a task
jig holds live is that task's names, and every other row a kept task's
(`adoption.md`, 1), whatever forges it names; a start with more live rows than
`forge.tasks`, or more kept ones than `forge.repositories`, is refused
as any record past the limits (`../root.md`, 8.1). Until a forge connects,
its reads and entries answer `NotConnected`, which narrows nothing
(`adoption.md`, 1), so the steps below run on each forge as it connects. Before its
records, the connector loads its kinds' values from its family
`Setting`, is given the stock defaults and says which differ (`change/gates.md`, 3), so a
restored change is judged by the rules as they are. temper's steps:

1. it reads afresh each repository's landing branches' tips and each
   live pull request (head, state, mergeability, CI, its place in a
   forge's merge queue), before any change steps;
2. it settles its outbox, each entry as its class says (section 4): a
   pull request found by its branches, a merge by its pull request's
   state (merged, in the forge's merge queue, or neither), a creation by
   its key after the entry's start, a goal's own branch by its name, a
   set written again; one not found waits for the retry deadline its
   last attempt committed, and is then sent again, its conditions
   checked as at any attempt, unless it is unrecoverable, which holds
   its task for a person; a goal's own branch is made again (`branches.md`, 1);
3. it steps every change once, and every goal's issue. A change reads
   its recorded pull request by number. Without a number, it reads by
   branches and accepts an open pull request, or a closed or merged one
   at its pushed head. This recovers an uncertain open whose reply was
   lost, even if it merged before the restart. Other closed or merged
   history is Missing and never supplies the change's pull number.

Its listings start from the newest time the store kept for each
repository with an issue or pull watch, inclusive, so what changed while
the engine was down is read once.

## 10. The world

The connector's world, `tests/engine/forge`, runs its subtree with the
fake forge below it (`../../testing/fakes.md`, 2) and the root scripted above it;
the tiers that run it through bytes and on a real Forgejo are
`../../testing/README.md`, section 2. Its stories: a change produced, opened,
checked, gated, queued and landed; CI failing, repaired; a gate asking
for changes, repaired, its verdict given again; a gate added while
queued; two changes into one branch, the second updated cleanly and its
review carried over; an approval carried over a clean update and asked
again after a repair; a change that leaves the queue for a conflict and
comes back at its place; a low-priority change aging past the window
ahead of later ones; a conflict resolved from a merge in progress; a
conflict settled by the merge job alone, and one it leaves to a
resolution, under rules asking an update and under rules asking none; a
catch-up into a goal's branch, merged by the merge job, and one already
contained, ended with no change; a
semantic conflict repaired; a CI repair ended by CI passing at its head,
due, retrying or held, and one released after it with no fresh repair; a
run of one left to finish; CI passed on an earlier head, changing nothing;
a review's repair untouched; the branch's own CI broken by a landing
temper did not make, the queue paused and repaired; its pull request
closed by someone else, held, released and reopened; a landing that
overlaps a goal's change, and one that does not; a goal's issue written,
revised and closed; a merge whose answer is lost, found merged after a
restart; an update whose answer is lost, found, or made again and landed,
under a queue and as a turn's prepare; one lost for good, decided again
until its count holds it, told why, and released; one whose base went on
while it was lost; a copy of it landing after the change moved on, held;
a goal's landing whose own update lands, going on to CI on its new head
and merging, and one whose goal's close-out merged into its branch just
before, doing the same; a push from outside during an update, held with
both heads named; a stalled update held after the stall, its words
naming the step; two changes into one branch, the second, better placed,
staying queued while the first updates, is checked at its new head and
merges, then taking its turn unprompted; a change's CI failed at its
set-up step, run again and passing, its gates kept; one run again that
fails again, then repaired, its brief saying so; a workflow step's
failure repaired with no re-run; a re-run whose answer is lost, found made;
a mergeability unsettled within the window, then settled; a
forge preloaded with ten times the history, idle at the same cost. Of
the rules as settings (`change/gates.md`, 3): a landing rule changed between two merges,
the second judged by the new rule; a gate a rule adds asked for on a
queued change, which leaves the queue, and a gate a rule drops no longer
waited on; a project's rule adding an approval; a rule refused in its
change's decision; a change kept on the charter it was made with, and
one held when that charter is no longer offered; a restart that hands
the values back before the records.

Its stories of access and of GitHub's ways, the root and routing
scripted: a forge connected while temper runs, an adoption refused as
not connected before it and made after it, the writer given then and
restored at the next start; an owned forge set up from the client,
each state's row and `SetUp`'s answer checked together, with and
without the box: asked when offered and none sets up, "busy" while
any sets up, "not offered" connecting, connected, stopped, not owned
or asked, or not kept, "not under the box" without the box and
after `BoxClosed`, each state the routing tells kept as told and none
restored; a forge stopped while a change waits on it,
nothing narrowed, its entry committed again once it connects; a managed
Forgejo's adoption whose access is refused, refused and named, and one
given, then read; the picker's listing on each kind of forge, a managed
one's admin's view read while another run writes, one not listed, an
installation's repositories changed and an adoption marked,
then unmarked when it is listed again; an adoption on a branch whose
rules want a merge queue and signatures, said in words with the one
setting; a merge queued, landed at the group's commit, and another
ejected and repaired; an update GitHub makes after it answers, found by
the head read afresh; no commit status written on GitHub, and nothing
held for it. The forge below is skein's fake, its GitHub layer
included: no example calls github.com or uses a real credential.

Its stories of removal: a forge removed with a change on it, its
watches retired and nothing read of it after. With the root's, in the
engine's world (`tests/engine/domain`): a removal, and a project's
deletion (jig's `domain/people/acts.md`, 2), each cut by a crash after
each kept act and resumed at the next start to the same end, a chat
that worked on the forge reading its repository's name, open or ended,
until the chat is erased; a removed forge's number never given again;
and nothing of the forge or the project read afterwards but what `adoption.md`, 1
and jig's `domain/people/acts.md`, 2 keep.

Its referee, beyond jig's for every connector (jig's
`domain/connectors.md`, section 15): nothing lands without every
requirement its branch's enforced rules name, guarded ones met at the
exact head merged and up to date met on the reads of the decision that
merged it; merges land exactly the heads decided; temper never writes to
what it does not own, and its runs never force a push (the forge's prepare
rebase is `../landing.md`, 5.2); a held change makes no effect;
a landing wakes a goal exactly when it overlaps; no change waits in a
queue past the aging window behind changes that became ready after it;
temper's own pushes while it holds the writer slot are never taken for
drift, but a late update copy moving the head after its change moved on
is drift (section 6); the calls made while idle are bounded by the
limits; nothing is narrowed, and no change held,
for a forge not connected; a change whose merge was answered queued is
not merged again until the forge has landed or ejected it, and lands at
the group's commit; nothing is written to, read of or listed for a
forge once its removal's decision has committed, and a removal ends
whole.

## 11. Open questions

- **A workflow that runs on no pull request** (`adoption.md`, 7; `change/branch-path.md`, *CI's card*):
  CI's card names every workflow file on the landing branch but those
  the repository's test gates and tiers dispatch, so a release's or a
  schedule's workflow is named though it reports nothing at a change's
  head; and it names each by its file, not by the `name` its file gives
  it. Telling them apart reads each file and its triggers, a read per
  workflow at each change of the directory. Default: by file, every one
  but the dispatched.

- **A closer's stops on a goal's path** (`change/branch-path.md`; `branches.md`, 2): its close step's
  change and its landing draw as a change into the goal's branch and
  the goal's landing do, with no stop of an agent's; whether the path
  names them "close" and "land", and a closer's card, is open. Default:
  as any change's.

- **A close-out's own charter** (`change/branch-path.md`, *The coordinator's cards*): the
  "close-out" stop opens the goal template's charter, the coordinator,
  which makes the close-out, though the reference's is a designer's
  change (`../../protocol/configuration/reference.md`) and its card's model and
  "Asked to" are then the coordinator's. Naming the charter that writes
  it would be a member of the template's work, kind 11, read where it is
  set. Default: the coordinator's card, until a draft asks for a
  close-out's role of its own (`change/branch-path.md`, *The project's roles*).

- **Project-scoped rules on a real forge** (`change/gates.md`, 3): observe the
  every-project default, its override and reset, and branch-relative
  coverage through the existing process and live landing stories after
  landing. Default: those stories; focused and fuzzy checks on fakes
  establish no real-forge outcome.

- **The connector's own retries** (`keeping-up.md`, 3): a brief read or lost-read
  recovery waiting for client room is retried when the connector resumes
  with room, without a timed backoff. A run's host call instead uses
  smith's growing, capped backoff (`smith/docs/design/domain/run.md`,
  5.2). Whether those internal reads need a timed backoff is separate
  connector work. Default: retry when room is available; the host-call
  rule changes no connector timer or admission bound.

- **Forge-read recovery with real agents** (`keeping-up.md`, 3): the existing ChatGPT
  and Claude Code owned-forge landing stories are the after-landing
  check under real traffic. A fake-forge owner-collision regression
  establishes the routing mechanism alone, not either real-agent
  outcome or resumption after `serve` restarts. Default: keep those live
  stories; the ownership contract requires no change to them. Forge
  bounds and room figures remain separate work, as below.
- **Links beyond a box's one person** (`adoption.md`, 6): several members need a
  family of links and evidence of the connecting owner; a pasted App or
  a second account needs user authorization or a verified claim on a
  review (`../../draft/forge-identity.md`, section 12). Default: adoption's
  seeds there. Containing agents and CI is required before an owned
  Forgejo's approval can resist an agent trying to bypass it
  (`../../protocol/setup/README.md`, section 9).
- **The approval story on GitHub** (`adoption.md`, 6, `change/questions.md`, 2): the complete box story
  belongs beside the owned-Forgejo domain story, then a real person's
  review in the live tier (`../../testing/README.md`, section 7). Default: lower
  setup, link and review checks establish those separate pieces alone.
- **Saved branches outside the task's saved set** (11): a terminal the
  engine never heard, a repository no longer in the set, or full holds at
  close can leave a branch on the forge. Finding these needs a branch
  inventory beyond the set a next attempt resumes from. Default: manual
  deletion or deployment retention; a failed descendant's known saved
  work stays with its root until that root closes.
- **A clean update's other words** (`change/repairs.md` and `README.md`, section 6): the result still says
  `the base broke it`, and the client panel calls its repair a semantic
  conflict, although the path proves only that CI failed after an update.
  Aligning those words needs the client's owner too. Default: the task's
  instructions and brief name the failed check; the result and panel keep
  their words until that follow-up.
- **"The brief on Home" in Settings** (`attention.md`, 3): the status agent always runs
  for now; the switch is another plan's. Default: it runs.
- **A status run's kept row** (`attention.md`, 3): a task made for the deployment is
  never forgotten, as jig forgets a result only for a person's request. At
  most one a sitting and an hour, each small. Default: kept, until jig
  forgets a deployment's reads.
- **A stock project's room, shared** (`attention.md`, 3; `../../protocol/configuration/stock.md`):
  the chat's 24K outside the pools is what no pool reserved, and no brief's own: the
  forge's repairs draw on it and a member's first spend in a period reserves it, and
  a shared project's plan doesn't serve the run. Home then keeps its computed
  sentence, and nothing tells the owners why. Default: so; they can fund the period.
- **A late copy's hold** (10): a head moved by a late copy of the
  change's own update, once the change has moved on, cannot be told from
  another's move by temper's identity, and holds. Keeping the writer slot
  past a settled update would spare it, for a time nothing bounds.
  Default: it holds, and is measured after landing.
- **What lands on a goal's branch and then on main** (`attention.md`, 2) counts twice,
  each a change landed. Default: so, said here and in the design of Home.
- **What a refused copy leaves** (`adoption.md`, 5; skein's `forge/setup/README.md`,
  section 6): Forgejo keeps an empty repository at the name of a source
  it could not clone, and only the same address asked again takes it
  away: asked from another, the row holds no `copying` and stops at
  `Taken`. Default: so, and the person deletes it on the forge.
- **A queue's throughput:** landings into one branch wait on CI one after
  another. If that binds, landings are batched: tested together, and
  bisected when they fail.
- **Conflicts found before either side lands:** temper sees every open
  change, and could try them against each other and warn both goals.
- **Checks without an LLM:** a repository's checks run as agents' tasks
  at the exact head (`change/gates.md`, 1); a checks-only run on a worker, running the
  project's commands with no model, would cost no turns (`../worker.md`,
  section 12).
- **A person's approval as a rule:** temper takes it two ways, a person
  gate answered in the client (`change/gates.md`, 1) and a review the person posts on the
  forge, which a landing rule counts (`change/gates.md`, 2). Whether an approval given in
  the client should also be posted on the forge as the person, so that
  the forge's protection requires it independently, needs a forge
  credential of the person's (jig's `domain/people/README.md`, section 11).
- **A gate's live run past the stall** (`change/repairs.md`): a review's task stays a wait
  the stall bounds, though a review working 30 minutes is as normal as a
  producer. Its verdict is kept if it arrives on the hold, and a release goes
  on from it. Default: as it stands, its wait being named by `change/repairs.md`; the request
  `parallel-faults` of 2026-11-06 met no such hold.
- **A green head no wake reaches** (`change/repairs.md`, `keeping-up.md`, 2): a held change reads on a wake,
  and a poll wakes it only when what it read of the pull request moved. Under
  a rule's required contexts, a check they do not name can keep the pull
  request's CI as it was while theirs pass, so the hold waits for a hint or a
  person's release, which makes no fresh repair. Default: so; it costs time,
  not a repair.
- **A retry before the read** (`change/repairs.md`): a run ending into a retry wakes no change,
  so a backoff shorter than `poll_ms` can start another run at a green head,
  left to finish as any run is. Whether jig tells a procedure of its delegate's
  retry, so that none starts. Default: as it stands.
- **A hold at gating at a green head** (`change/repairs.md`): past the bound or short of
  authority there, a hold can be a gate's or CI's, and its state does not say
  which. Saying so changes what a hold stores. Default: it waits for a person's
  release, which decides afresh.
- **A repair of the branch's own CI at a tip since passed** (`queues.md`): the
  rule of `change/repairs.md` asks of a change's own head. That repair is a change of its own,
  made for a failed tip: CI passing on the tip ends the queue's pause, and that
  change goes on. Whether the rule reaches it. Default: as it stands.
- **A repair held for its brief** (`change/repairs.md`): the repair's task is held, naming
  its reason, while the change stays `Repairing` and its panel shows no hold:
  a person reads why on the task, and releases it there. Default: so; whether
  the panel says it too stays open.
- **A forge out of reach or not connected** (`change/repairs.md`): `Unavailable` counts as
  transient, so a long outage holds each repair preparing through it, naming
  why; `NotConnected` pauses with no bound and no hold, its notice naming why.
  Default: so; the allowance is the user's.
- **A failing check's repair brief:** a check gate that says no is
  repaired as a review that says no is, and its repair's brief carries
  the pull request's reviews section (`change/repairs.md`). A check's verdict is not a
  review on the forge: whether the brief should carry the check's own
  output instead is open.
- **A failure that is the runner's, not the change's:** a job whose
  set-up step failed, or whose runner was lost, is a repair like any
  other under the queue, where landing by turn retries it, never
  repaired (`../landing.md`, 7.4). Running the job again first, through the
  API, is `../../draft/ci-on-forgejo-17.md`, with the stalls' reasons and
  the agents' log tools.
- **Two Forgejo behaviours no fact records yet** (`../../protocol/forge/README.md`,
  section 12): that CI after a clean update runs on exactly the head the
  update made (section 6); and where a merge lands when its pull
  request is retargeted after the fresh read before it, since Forgejo's
  merge is conditional on its head alone (section 4).
- **Intake:** issues others open as a source of goals.
- **A resource refused as forbidden:** whether the connector should hear
  when it is readable again, rather than read it on a backoff.
- **A merge style per repository,** named by the project rather than the
  forge's (`adoption.md`, 4). Default: on Forgejo the repository's default (skein's
  `forge/README.md`, section 10); on GitHub the first the repository allows of
  merge, squash and rebase; either shown at adoption.
- **A change whose forge is not connected,** or whose repository is not
  reachable (an installation suspended, a repository it does not select):
  it waits, its stall not counting the wait, and nothing is narrowed
  (`adoption.md`, 1, 3). The alternative is holding it for a person past a bound.
  Default: it waits, and the client shows why.
- **`Forbidden` and `Missing` taken as access lacking** on a repository
  an installation suspended or stopped selecting (`adoption.md`, 3), rather than as
  facts that narrow what temper may do there. Default: access lacking,
  while the installation's events say so; any other refusal narrows, as
  ever.
- **Several changes in a forge's merge queue** (`queues.md`, 1): a change there
  leaves temper's turn, and the next enters behind it. The alternative
  is the first keeping its turn until it lands or is ejected, one at a
  time, as temper's own queue does. Default: several, the forge's queue
  grouping them.
- **The update before entering a forge's merge queue** (`queues.md`, 1): the first
  takes its base as any first does (section 6), though the queue merges
  the base itself. The alternative enters it as its head is, saving a CI
  run. Default: as any first.
- **What failed on a queue's group** (`change/repairs.md`, `queues.md`, 1): an ejection's repair
  brief carries the forge's reason alone, the pull request's `dequeued`,
  since skein's `Pull` names no commit of the group. The alternative
  reads the group's CI and gives the repair its failing output.
  Default: the reason alone.
- **Taking a change out of a forge's merge queue** when it is cancelled:
  the vocabulary has no call for it. Default: none; the cancel waits for
  the landing or the ejection (`queues.md`, 1). The alternative is closing the
  pull request, which the forge's queue drops.
- **Signed commits required** on a landing branch (`adoption.md`, 4): temper's
  workers do not sign. Default: landing there is a wait for someone
  else's merge, said at adoption; signing is later.
- **Connecting a forge as another user** (`adoption.md`, 1): refused, since the keys
  and echoes already on the forge are the old user's. Moving a
  deployment to another App or user is later; a forge goes whole, by
  its removal (`adoption.md`, 1).
- **Which checks are CI on GitHub, without required checks:** temper's
  draft (`../../draft/github.md`, section 9) takes the repository's Actions
  workflows' check runs there, so that another App's check run (a
  Codecov's) is not CI. That needs each row's App, which skein's
  `Status` carries only for a failed Actions run; until skein's dialect
  carries it on every row, every row is CI (`keeping-up.md`, 3), another App's check
  runs among them, and its suites with no run list nothing.
- **What ends the Away switch** (`attention.md`, 4): `until`, by the client's next
  08:00, and a hand. Quiet hours and notifications are the client's
  later parts (`../../client/README.md`, section 5); neither is the
  switch's.
- **The classes per person** (`attention.md`, 6; `../../draft/what-reaches-you.md`,
  section 11): a team may want one person asked about a path set and
  another not. Default: the classes are the project's and Away each
  person's, approvals naming roles as they do.
- **What code decides, listed** (`attention.md`, 5): a known flake's rerun
  (`../findings.md`, section 5) asks no one and writes no row. Whether
  Decided for you lists it. Default: not listed.
- **A refused `message` keeps no record** (`change/questions.md`, 1): the call's refusal of an
  agent's own words is rendered and kept with no call's record, as the
  `goal` tool's refusal is, where a decided call's answer is kept
  (`../routing.md`, 2). So a call asked again decides again, from the change as
  it then stands. Default: so.
- **A passed-up question whose asker ends** (`change/questions.md`, 1): its requester hears of the
  end when it answers, not before, since saying so once at the end would need a
  stored mark beside where the question stands. One that never answers hears
  nothing of it, and the question stays until the change ends. Default: so; it
  did reach someone.
- **A forge's run on a head replaced** (`change/README.md`, 2, step 3): the update first ends the
  change's wait for that head's CI, not the run, since temper has no effect that
  stops one, and a pull request must exist before the forge can update it, so a
  run that starts at its opening starts first. Default: so; what it spends goes by
  the repository's own CI settings.
- **An early update against the bound** (`change/README.md`, 2, step 3): it counts against
  `forge.change_policy.updates` (3 in `small`, 80 in `standard`), so under a base
  that keeps moving `Hold::Updates` comes one update sooner. Not counting it would
  leave the bound untrue of update attempts. Default: it counts; the figure is the
  user's.
- **A report's head past a merge** (`change/questions.md`, 1): a next producer's start that merges a
  tip it lacks makes a new head before its run if clean, or at its first push or
  save once its conflicts are settled. A report written before that merge names
  an older head; a contained tip makes no new one. Default: so; the head delivered
  names what is on the branch (`../agent/README.md`, section 5).
- **A person's request on a goal landing's pull request** (`change/gates.md`, 1; `../landing.md`): a
  landing is not a change, and its merge's refusal keeps `../landing.md`'s own rule,
  so such a request starts no repair. Whether a landing should repair it as a
  change does, from the remarks of a person reviewing a whole goal, is undecided.
  Default: so, until a request asks.
- **CI asked again at an unchanged head** (`change/repairs.md`; `../landing.md`): a change
  whose CI the runner failed has it run again (`change/README.md`, 2, step 3), but one
  that failed for another infrastructure fault inside a workflow's step (a network
  outage, a flaky test) cannot have its CI run again until a new commit or a cancel,
  nor can a landing's failed receipt be superseded; a person's ask, and a
  repair's verdict that the cause is outside the change, would rerun a change's
  failed jobs or supersede a landing's failed receipt. It needs a client
  request, stored retry state and a repair's contract, so the restart work of
  2026-11-10 left it for a plan of its own. Default: a new commit or a cancel.
- **A keyed creation lost across a restart** (section 4, 9): a review, a
  comment or an issue whose answer a restart lost, and that its find does not
  see by its deadline, holds its task for a person, though a find that read
  everything since its start makes a second one unlikely. Default: so; only a
  goal's own branch is made again.
