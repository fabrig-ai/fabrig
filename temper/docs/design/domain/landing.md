# Landings

Provisional, 2026-11-13. How a project that lands by turn lands its
changes: a train per repository and branch, one landing at a time on
each, the next formed from the changes waiting in the queue's order, a
batch of them up to the project's `batch`; each landing's members
stacked by a merge job, a prefix per member on a branch the landing
holds; gates proved by receipts kept by key, at the stack's top; and
the landing's own pull request fast-forwarded into the branch, which
lands its members by containment. It is the merge queue
(`../draft/merge-queue.md`, its execution's M1 to M3); a red round of
several units is narrowed by prefixes, `landing-search.md`. It replaces, for such a project, the change's CI, its queue,
its updates and its merge in the repository's style
(`forge/change/README.md`, 2 steps 3 and 5 to 7, `forge/queues.md` and
`forge/README.md`, section 6). What follows a landing, its tiers and its
verdict, is `after-landing.md`. The forge's calls it makes are skein's
`forge/README.md`, 3.2. What is still open is `landing.md`, section 17.

## 1. In one page

- **A project lands by queue or by turn,** its own setting (section
  13). `queue` is `forge/README.md`'s change procedure, `forge/change/README.md` and `forge/queues.md` and `forge/README.md`, section 6, and
  stays the default until a project's turn has been walked on real
  backends. `turn` is this document. A change follows the way its
  project names when it becomes ready.
- **A train** is a repository and the branch its changes land into
  (section 9): a repository's main, or a goal's branch. It has at most
  one landing at a time; the other ready changes wait in the queue's
  order, and the branch moves only by that landing, so a proof never
  goes stale on another landing's turn.
- **A landing** is a batch of units, each a change or changes that
  land together (section 4): the units waiting on a free train, in
  order, up to the project's `batch` changes, formed and sealed when
  its train is free. It is a procedure task of the forge's, made in the
  decision that forms it, requested by the deployment on the project's
  behalf. Its members' change tasks wait on it, and their goals hear
  each step as news.
- **Prepare** (section 5) is a job on a worker: from the train's tip,
  each member's head combined onto the one below by the train's
  `combine` (13.1), a merge with no fast-forward (stock into a
  repository's main) or a squash, one commit with that merge's tree
  (stock into any other branch, a goal's). Each such commit is the
  member's prefix, pushed to a branch of the member's that the landing
  holds; the last is the candidate. No member's own branch is rewritten.
  A conflict sends the member to a resolution, or, with an earlier
  member alone, back to wait; it leaves, and the job stacks the next.
- **Receipts by key** (section 6). A key is what a gate's result
  depends on: the candidate's tree without the paths the gate ignores,
  with the gate's definition, and the base's tree for a gate that reads
  the base. A receipt is a gate's result under a key, kept per project
  and reused by every landing and worker. A gate whose key has a
  receipt never runs again.
- **Proof by dispatch** (section 7). Gate workflows run only when
  temper dispatches them, at the candidate, the top, on its branch. A
  real failure in a round of one unit is a repair under its changes,
  which leave; in a round of several, a search by prefixes blames a
  unit, which is repaired, and lands the green prefix below it (7.3). A runner's fault is retried. Judgement gates hold at
  each member's prefix (section 3).
- **The turn** (section 8): the train's tip and each member's head read
  afresh, then the landing's own pull request merged fast-forward only
  at the round's proved prefix: its candidate, or its search's green
  prefix. The branch then contains each member's head in it,
  merged, or its squash commit: each has landed, its landed
  commit its prefix, the one that brought it. No
  gate runs inside the turn, and the branch is never gated again: its
  commit is the gated one.
- **A member leaves** before the turn on a conflict, a moved head, a
  cancel, a hold, a refused judgement gate or a red, with its whole
  unit (4.3); it keeps its place in line. The members below it keep
  their prefixes; those above it are stacked again in the next round.
- **The forge shows it** (section 10): the landing's pull request, each
  gate's receipt as a commit status, `temper/gate/<name>`, on the
  candidate, which the fast-forward carries onto the branch, and each
  member's pull request merged: a merged member's by the forge's
  detection of a manual merge, a squashed member's recorded merged at
  its squash commit by the landing.

## 2. Structure

```
temper-engine-domain-forge              the top: landings' and receipts' states, the trains, routing; the effects
├── temper-engine-domain-forge-change   the change procedure: under turn, up to ready, its judgement at its prefix, its repairs and resolutions
└── temper-engine-domain-forge-landing  the landing: decisions over its state and the facts; keeps nothing
```

- **The top keeps** each landing's state (family `Landing`), the
  receipts (family `Receipt`) and the keys in flight, as it keeps the
  changes' states (`forge/README.md`, section 3). Trains are derived,
  never stored: a train is held by its live landing (section 9).
- **`landing` is policy,** as `change` is: given a landing's state, its
  members' states, the receipts and the facts the top read, it says what
  is due and the landing's next state, one decision a step:

  | Decision | What the top does |
  |---|---|
  | prepare | delegates the round's prepare job (5.2) from the round's base or the last kept prefix, for the members from the first with no prefix |
  | open | opens the round's pull request, the top's branch, or the green prefix's after a search, into the train's (section 10) |
  | key | reads the trees a gate's key needs, at the candidate or a probed prefix and, for a gate that reads the base, at the base (6.1) |
  | dispatch, retry, post | a gate's run, its rerun, its status on the candidate or a probed prefix (7.1, 7.4, section 10) |
  | judge | asks a member's change for its judgement gates at its prefix (section 3) |
  | merge | merges the landing's pull request, fast-forward only, at the round's proved prefix (section 8) |
  | record | records a squashed member's pull request merged at its prefix (section 8) |
  | landed | tells a member's change it landed, at its prefix (section 3) |
  | leave | records why a member left, with its unit, and asks its change for what follows: a resolution, a repair, or nothing (4.3) |
  | close | closes the round's pull request and deletes the branches from the round's first changed member up: the round ends unlanded (5.1) |
  | finish | ends the landing, its result naming what landed and what left (4.2) |
  | hold | holds the landing, naming the step and what it waits on (4.2) |

  The facts it is given: the train's tip and each member's head, read
  afresh; whether each member is ready, judging, cancelled or held, the
  members it depends on, and its judgement at its prefix; the prepare
  job's state and outcome, and each member's branch's tip; each gate's
  key, receipt and run in flight at the top; the landing's own effects'
  outcomes.
- **Keys are computed in the domain** from what the forge answers of a
  tree (skein's `Read::Tree`), or taken from a prepare's report: the
  engine never reads a repository (`worker/README.md`, section 2).
- **The worker** runs a landing's prepare (section 5) as a job, never
  an agent's run.

## 3. A change under the turn

```
producing ─► opening ─► gating ─► ready ─► in its landing ─► judging ─► landed
                          ▲         ▲             │              │
                          │         └─ leaves ────┤              │
                          │                   resolving     repairing
                          └───────────────────────┴──────────────┘
```

- **Producing and opening** are `forge/README.md`'s 8.2, steps 1 and 2. No CI
  runs on a pushed head: the project's gate workflows are dispatch-only
  (section 7), and the change has no checking step.
- **Ready:** its head pushed and its pull request open; every blocking
  judgement gate the landing rules and the change ask for (a review, a
  person's approval: `forge/change/gates.md`, 1) holds at its head; and its task
  is not kept by a dependency. A review runs as soon as the head is
  pushed. Ready and in no landing, it waits for its train (section 9),
  keeping the time it first became ready while it stays live.
- **In its landing,** the change writes nothing for it: the landing's
  job prepares its prefix on its branch in the landing, the landing
  proves the top and merges it. The change steps on its landing's asks.
- **Judging at its prefix:** once its prefix is pushed, the landing
  asks the change for its blocking judgement gates at its prefix,
  while the test gates run at the top beside them. A verdict whose
  freshness is `clean` (`forge/change/gates.md`, 1), given at the
  change's head or a head its clean updates led through, counts at the
  prefix: temper's merge, or the squash that stands for it, settled no
  conflict, so the member's diff is unchanged. An `exact` verdict, and a check's, is asked again at the
  prefix, on the round's pull request: the review's brief names that
  pull request, the member's branch in the landing, the train's branch
  as its base and the prefix as its head (`forge/change/gates.md`, 1). The landing waits
  for it there. A gate that says no makes the change repair, as
  at its head (`forge/change/repairs.md`), and the member leaves. A
  prefix rebuilt in a later round asks again.
- **Repairing** (`forge/change/repairs.md`): a gate's real failure at the
  candidate, given the failed run's output (7.3), or at the red prefix
  its landing's search blamed (`landing-search.md`, 7). The member leaves its
  landing; its new head makes the change ready again, and it waits for a
  later landing, first in line by its kept time.
- **Resolving** (`forge/change/repairs.md`): a conflict with the branch the prepare job met, in
  a workspace where the round's base is merged into the change's branch
  with the files in conflict marked, made afresh from the forge. Its
  pushed merge contains the base, and it waits for a later landing. A
  conflict with an earlier member alone asks nothing: the change stays
  ready and waits, first in line (5.2).
- **Leaving otherwise:** a new head its goal delivered takes the change
  back to gating at that head, a head moved by another identity holds
  it as drift (`forge/change/README.md`, 3), and a cancel ends it; in
  each the member leaves (4.3).
- **Landed** when its landing's turn has made the train's branch
  contain its prefix: its head, merged, or its squash commit. Its landed
  commit is its prefix, the commit that brought it, which the landing
  tells it; it reads no pull request merged for it. Its pull request
  reads merged, by the forge's detection of a manual merge, or recorded
  so by the landing for a squashed member (section 8). A squashed
  member's branch is done once it lands: later work on its goal starts
  from the goal's branch. It then verifies, and ends at cleared
  (`after-landing.md`, section 6).

## 4. A landing

### 4.1 Its unit

- **A unit** is what must land together: a change alone, or a closure
  of changes joined by `with` or by a dependency that joins, in its
  dependency order:
  - **`with`:** a change names the changes it must land with, a
    parameter of its own (`forge/change/README.md`, 1), when it is made or by an
    amendment of a task that may name the other one (jig's
    `domain/tasks/changes.md`'s rule for dependencies). `with` is
    symmetric and transitive, and a change is ready to land only when
    every change of its closure is.
  - **A dependency across repositories** joins its two changes, unless
    the project's setting `dependencies_join` says such dependencies
    only order them. A dependency that only orders, and any dependency
    in one repository, orders results, as jig's dependencies do (jig's
    `domain/tasks/lifecycle.md`, 8): the dependent is kept, so not
    ready, until its dependency has landed, and then waits for its own
    landing.
  - **Pin-only members:** where the project keeps downstream pins at
    their upstream's main, a unit with a member upstream would gain a
    member in each downstream repository the project's pins name, its
    only change the bumped pin.
- **Units are indivisible:** they join, leave and land whole. A unit's
  changes are consecutive members of its landing, in its dependency
  order, and share the unit's number there (section 12). A dependent
  kept by its dependency is not waiting (section 3), so it never forms
  beside it.
- **A unit in one repository** joins a landing whole. One across
  repositories, or with pin-only members, is refused as its train would
  form with it: its first change is held, its hold naming the unit's
  changes and what brings such a landing, the merge queue's later
  batches across repositories (`../draft/merge-queue.md`, 13). A unit
  of more changes than `landing_members` is refused the same way, its
  hold naming the bound: it can never fit. The train forms without it.
  A release of the hold refuses it again while the unit stands.
- **Sealed:** no change joins a formed landing. A dependency or `with`
  given to a member after forming leaves it where it is; its unit is
  judged at the next landing's forming. A member whose dependency,
  another member, leaves, leaves with it (4.3).

### 4.2 The landing task

- **A procedure task of the forge's,** `landing`, made in the decision
  that forms it (section 9), with its members. It holds each member's
  branch in the landing from its making (5.1).
- **Requested by the deployment on the project's behalf,** with the
  authority the project's policy gives landings, as a branch's repair
  under `queue` is (`forge/queues.md`); funded from the project's
  spend, made with a budget of zero, its priced effects funded at the
  effect from the period's unreserved spend, so that a period reserved
  whole never stops it (jig's `domain/authority.md`, section 6). It answers to no one:
  its escalations go to the project's escalation role, and its members'
  goals hear of it as news.
- **Each member's change task waits on it:** it steps only on its
  landing's asks while its landing lives.
- **Its goals hear each step,** as news to each member change's goal,
  its own member's steps and the landing's holds, in these words, `L`
  the landing's task, `C` the change's, commits in hexadecimal at the
  forge's length:
  - `landing L: change C joined, into REPOSITORY BRANCH, round N`, then
    `, with changes C1, C2` naming the landing's other members in its
    order, where it has any;
  - `landing L: change C proving at PREFIX on BASE`, then `, under TOP`
    where its prefix is below the round's top;
  - `landing L: change C probing at PREFIX, red at RED, on BASE`, to
    each member with a prefix in the round, once per probe
    (`landing-search.md`, section 8);
  - `landing L: change C landed at PREFIX`;
  - `landing L: change C left: WHY`, WHY as 4.3 names it;
  - `landing L: held: WHY`, WHY as the hold names it.
- **It ends** done once no member is left to land, its result naming
  each landed member's commit and each member that left, and why. Its
  train frees at once, and each change that left, once ready again, is
  first in line by its kept time. Cancelled, by a person's act on the
  landing task, its members are left ready and in no landing, each
  unless it is recorded landed (section 8), and the train forms again.
- **No silent wait: held** when a step makes no progress past its bound,
  naming the step and what it waits on, its members' goals hearing it:
  - the prepare job failed past its tries (5.2), `prepare job: WHY`;
  - a gate's runner faults past `gate_retries` (7.4), `gate NAME of
    REPOSITORY failed on its runner past its retries`;
  - a gate's run in flight past the project's `gate_timeout` (7.1),
    `gate NAME of REPOSITORY: run RUN running past its bound`;
  - its pull request or a merge refused for a cause not stale (section 8),
    `turn: WHY`.

  A step a member's change makes (its judgement, its repair) is
  bounded by the change's own stall (`forge/change/repairs.md`): the
  change holds, and the member leaves. A release lifts the landing's
  hold, and it prepares again in a new round, keeping its receipts.
- **Drained** (jig's `domain/connectors.md`, 6.7): a landing in a
  draining hold's scope that has begun its proof goes on, through the
  turn, to land; one still preparing is idle, and held. A stale base or
  a fault that would send it back to prepare makes it idle there.

### 4.3 Leaving

- **A member leaves its landing,** with its whole unit, before the
  turn, when:
  - its merge onto the stack conflicts (5.2), with the round's base:
    `conflict with the branch: FILES`, and its change resolves; or with
    an earlier member alone: `conflict with an earlier member: FILES`,
    and its change waits;
  - its head moved, its goal having delivered or another hand having
    pushed: `head moved to HEAD`;
  - it is cancelled, `cancelled`, or held, `held`;
  - a judgement gate at its prefix says no (section 3): `gate NAME
    refused`, and its change repairs;
  - a test gate fails at the candidate in a round of one unit (7.3):
    `gate NAME failed in RUN`, and its change repairs; or its round's
    search blames its unit: `gate NAME failed in RUN at RED, above
    GREEN`, and its change repairs (`landing-search.md`, section 4);
  - another change of its unit left: `change C of its unit left`; or
    its dependency, another member, left: `its dependency, change C,
    left`.
- **It keeps its place:** its first ready time is unchanged, so once
  ready again it is first in line by that time. Leaving is never a
  repair by itself.
- **The rest stay:** a member that leaves while the job stacks is
  skipped by that job (5.2). One that leaves later ends the round (5.1),
  but for a unit its round's search blames (`landing-search.md`, 4):
  the members below it keep their prefixes and receipts, those above
  it are stacked again onto the last kept prefix, and the landing
  proves its new top. With no member left, the landing ends (4.2).

## 5. Prepare

### 5.1 Rounds

- **A round** is one stack: its base, the train's tip read afresh as it
  starts; each member's head, read afresh; the train's upstream's tip,
  read afresh, where the train has an upstream (5.2); each member's
  prefix, the commit the prepare job pushed for it, and how it combined
  the member, merged or squashed; and its pull request, from the top's
  branch. The top is the last member with a prefix; its prefix is the
  round's candidate. The landing's rounds are counted from 1.
- **A branch per member:** member n's, n its number in the landing from
  1 (section 12), is `temper/<root>/l<task>`, the landing's own branch,
  for the first member, and `temper/<root>/l<task>-<n>` for each other
  (`forge/branches.md`), the landing's task; each held by the landing
  from its making, since its members are sealed. The round's job pushes
  each member's prefix to its branch, expecting the branch new; a member
  that conflicted, or left, has none in that round.
- **A round ends** at its turn, when a member leaves after the job
  stacked it (4.3), but for a unit its search blames, or when its base
  goes stale (section 8). Ended
  unlanded, the landing closes the round's pull request and deletes the
  branches of its first changed member and of every member above it,
  its own effects (section 11), before anything else, so that the next
  round's job finds them new: from the member that left, or every
  member's for a stale base. The members below keep their branches,
  prefixes and receipts.
- **The next round** takes the members still in the landing: its job
  stacks only those above the last kept prefix, onto it or onto the
  base (5.2); with none, the last kept prefix is its top, and no job is
  made. Its receipts hold for every key that did not change.
- **A member whose head the base contains** has landed already, by
  another hand: nothing is merged for it, its change is told it landed
  at the base, and the stack goes on without it.
- **Released** as the landing closes: its branches deleted, as any held
  branch's release does (`forge/branches.md`); one never pushed is
  dropped with no write.

### 5.2 temper's merge train

The default, for a project with no prepare command:

- **A job** (`worker/jobs.md`, kind 1, `prepare`), delegated by the
  landing, so that its pushes are made under the landing's holds on its
  branches (jig's `domain/tasks/holds.md`, 1): the train's repository
  checked out at the commit the round stacks from, its base or the last
  kept prefix, by its commit; each member above it, in the landing's
  order, combined onto the prefix below by the train's `combine` (13.1);
  once every member is combined, each such commit, the member's prefix,
  pushed to the member's branch, expecting it new, from the lowest up.
  temper's own prepare runs no command of the project's. The two ways,
  with the messages `../protocol/tooling.md`, 4 gives:
  - **merge:** the head merged, no fast-forward, the prefix below as
    the first parent and the head as the second, committed with the
    member's merge message: its title, its description's first
    paragraph, and its change, pull request and head as trailers,
    with its goal's for a train into its repository's main;
  - **squash:** the same merge, its tree committed as one commit whose
    only parent is the prefix below, with the member's squash message:
    its title, its description's first paragraph, then its change,
    pull request, head and goal as trailers (`../draft/squash-in.md`,
    section 3). A squash whose tree is the one below's is still a
    commit.
- **A conflict:** a member whose merge onto the prefix below does not
  apply cleanly is skipped, and the job stacks the next member on the
  same prefix. The job tells which it meets by the member's head merged
  onto the round's base in the same job: a conflict there is with the
  branch, its files that merge's, and the member's change resolves
  (section 3); a clean merge there leaves a conflict with an earlier
  member alone, its files the stack's, and the change waits, first in
  line, since once that member has landed its conflict, if any, is with
  the branch. A member stacked straight onto the base conflicts with
  the branch.
- **Authored and committed** by the repository's git identity
  (`../protocol/credentials.md`, 4), both ways: the identity the
  deployment's agents' commits carry, and temper's own.
- **A goal's branch reads one commit per change,** its title and
  trailers naming the change's task, pull request, head and goal, and a
  merge for each catch-up or resolution of main
  (`../draft/squash-in.md`, sections 2 and 6); a goal's landing into
  main is a merge titled with the goal's title, its trailers the goal's
  landing change's and the goal's, which `../testing/qa/landings.md`, 4
  reads.
- **Why merge or squash** is `../draft/merge-queue.md`, 5.3: a goal's
  branch goes on after it lands, and merges onto main; a change into a
  goal's branch ends when it lands, and squashes. A squash meets the
  conflicts its merge would, and has its tree, so its keys (6.1).
- **A member that brings its upstream's commits is merged** whatever
  `combine` says: one whose head holds a commit the round's base lacks
  and the train's upstream holds, as a catch-up of main made as a
  change into a goal's branch does, so that those commits stay the
  upstream's. A train's upstream is the branch its own branch was made
  from and lands into, a goal's own branch's base (`forge/branches.md`,
  1); a train into a repository's main, or into a branch no goal holds,
  has none. The job tells, from the upstream's tip the round read
  (5.1), given in its request (`worker/jobs.md`).
- **Its outcome:** the engine reads each member's branch afresh once
  the job has ended. Its tip is the member's prefix when its parents
  are the prefix below and the member's head, a merge, or the prefix
  below alone with the member's head in its `Head` trailer, a line of
  its message's last paragraph, a squash (skein's `Read::Commits`); the
  round keeps which. The prefix
  below is that of the nearest member below with one in the round, or
  the commit the round stacks from. A member the job's result names in
  a conflict leaves (4.3), saying which.
- **A job's own failure** (a preparation that fails, a push refused, a
  deadline passed) is the project's, not a member's: jig tries it
  again within the job's tries, then holds it, and the landing holds,
  saying so (4.2).
- **Lost or partial:** a job whose answer is lost is tried again by
  jig. Before each delegation, and once a job has ended, the landing
  reads its members' branches: each prefix found there in order, each
  on the one below, for the round's base and that member's head, is
  taken, and a job is made only for the members above the last taken.
  A job whose push finds a branch already there fails as undelivered,
  and the read that follows takes what it finds; a failed job after
  which the read takes no new prefix holds the landing.
- **Not drift:** the landing's branches are its own, written only by
  its jobs under its holds; no member's own branch is written.

### 5.3 The project's tooling

- **A project's prepare command,** named in its landing settings, runs
  in the same job, at the home's pinned revision, with no model and no
  forge credentials: it may regenerate derived files and settle
  conflicts its rules know, and makes the prefixes the job pushes to
  the members' branches as temper's own does. Its conflict sends the
  member to a resolution as 5.2's does.
- **Its contract** (the arguments, the files, their formats, the
  environment, the exit codes) is `../protocol/tooling.md`.

## 6. Keys and receipts

### 6.1 A key

- **What a gate's result depends on,** as 32 bytes: SHA-256 of a
  canonical encoding, the gate (its repository and name), the gate's
  definition (its setting's value, section 13: its workflow, its
  ignored paths, whether it blocks, whether it reads the base), the
  candidate's tree filtered by the gate's ignored paths, and, for a gate
  that reads the base, the base's tree filtered the same way.
- **The filtered tree:** the entries of the commit's root tree,
  ordered by their full paths from the root, byte by byte (not git's
  tree order), where an entry is listed whole unless an ignored path
  lies at or below it: an ignored entry is left out, and a directory
  above one is replaced by its own entries, recursively. An ignored
  path is a path prefix of whole segments (`docs`,
  `crates/x/README.md`), never a pattern.
- **The encoding,** each number big-endian: the version, one byte, 1
  for a gate that does not read the base and 2 for one that does; the
  repository's forge and id, eight bytes each; the gate's name and its
  definition's bytes, each after its length in two bytes; for version 2,
  the base's filtered tree's entry count in four bytes and its entries;
  then each entry of the candidate's filtered tree: its path's length in
  two bytes and its path from the root, its mode as four bytes, and its
  object id as 32 bytes, as skein's vocabulary carries every id, a SHA-1
  one padded. A definition that does not read the base encodes as it
  did before the base was a member, so its keys are unchanged.
- **Read from the forge:** the top reads the trees it needs (the root,
  and each directory above an ignored path) with skein's `Read::Tree`,
  and the landing computes the key.
- **A project's `key` command** gives its own instead: in its prepare
  job's result when the project names a prepare command too, and
  otherwise by a key job (`worker/jobs.md`) per repository and gate at
  the candidate, delegated by the landing; the engine reads only the
  job's result.
- **A judgement gate's key** is not a tree: it is the gate and the head
  its verdict was given at, carried over clean merges (section 3).

### 6.2 A receipt

- **A gate's result under a key:** passed or failed, the run (the
  forge's run, its number and its link), the candidate it ran at, and
  when it ended.
- **Made only from a run temper dispatched** for that key and saw end
  at exactly that candidate. A status someone else wrote, or a run
  temper did not dispatch, is never a receipt.
- **Kept per project,** in the family `Receipt` (`../protocol/store.md`,
  section 2), and reused across landings, rounds and workers. A gate
  keeps its newest receipts up to `receipts_per_gate`, oldest first
  out, but never the receipt of the key at its repository's main tip,
  which stays while it is main's.
- **A baseline at main's tip.** When a repository first lands by turn,
  or a gate is added or changed, and the key at main's tip has no
  receipt, the gate is dispatched once at main's tip, on a branch of
  temper's own made at that commit, `temper/gate/<gate>.<n>`, as a
  tier's are (`after-landing.md`, section 2), so
  that a landing whose keys equal main's (a documentation landing) has a
  receipt to land on.
- **A failed receipt** makes a key that fails at once, with its run's
  output, until the tree changes. A failure whose tests all match open
  flake findings is rerun once first (`findings.md`, section 5).

## 7. Proof

### 7.1 Dispatch

- **Each gate workflow runs only when dispatched** (skein's
  `Write::Dispatch`), never on a push or a pull request event:
  adoption says when a gate's workflow names another trigger (13.2).
- **A train's gates:** a repository's test gates (13.1) prove the
  trains into its main. A train into any other branch, a goal's, has
  no test gate: each delivery's own check stands for it, and its
  landing is proved by its judgement gates alone.
- **For each of its train's test gates,** at the candidate, the
  round's top, and at its search's probes, each blocking gate at a
  probed prefix on that member's branch (`landing-search.md`, 3): a key with
  a passed receipt holds; one with a failed receipt fails (7.3); one in
  flight for another landing is waited for; otherwise the landing
  dispatches the gate's workflow on the top's branch, whose head is
  the candidate, with the inputs `temper_commit` (the candidate) and
  `temper_key` (the key in hexadecimal), and, for a gate that reads the
  base, `temper_base` (the round's base). A gate that does not read the
  base is given no `temper_base`.
- **The run is found and followed:** among the runs of that workflow
  at the candidate (skein's `Read::Runs`), one made by a dispatch,
  triggered by temper's own user on that forge, after the dispatch's
  start; then read as it goes (`Read::Run`), on the job deliveries'
  hints where they come and on the landing's own polling of its runs in
  flight: a dispatch-only job publishes no commit status on its head
  (skein's `forge/forgejo.md`, section 2), so no status read wakes it. Its polls
  are due at the client's `poll_at` for the run's repository, every
  `poll` or, while that repository hears, at its last hint plus
  `fallback` (`forge/keeping-up.md`, 2). A
  run another user triggered is never taken. Two that match are two of
  temper's own dispatches of one key at one commit, either of which
  proves it: the first to complete makes the receipt. A run whose
  commit is not the candidate is a fault (7.4): the branch moved under
  it. A run an ended round dispatched is followed to its end for its
  key's receipt, and acts in a later round only through that receipt,
  where its key is that round's (`landing-search.md`, section 6).
- **Bounded:** a run in flight longer than the project's `gate_timeout`
  since its dispatch holds the landing (4.2).
- **Its status:** `temper/gate/<name>` on the candidate, pending once
  dispatched, then the receipt's verdict (section 10).

### 7.2 Proved

- **When the round's blocking gates hold:** its test gates by receipt
  at exactly its proved prefix, its candidate or the green prefix its
  search held (`landing-search.md`, 5), and each of that prefix's
  members' judgement gates by their verdicts at its own prefix (section 3). An advisory gate is dispatched,
  read and reported, and holds nothing.
- **A landing whose keys all have passed receipts** (documentation,
  when every gate ignores it) is proved with no run.
- **A train with no test gate** is proved by its judgement gates alone,
  and its repository's adoption says so for its main.
- **One proof or probe at a time per train,** since a train has one landing;
  trains prove side by side, and a key in flight is dispatched once for
  all of them.

### 7.3 A real failure

- **A run whose workflow's own step failed** makes a failed receipt,
  read with each failed job's failed step and the end of its log
  (skein's `Read::Job`) as the landing makes it.
- **A round of one unit:** its changes are repaired
  (`forge/change/repairs.md`), each brief carrying the gate, its run's
  link, the candidate and its base, and those steps and logs. Its
  members leave (4.3), and the landing ends with nothing landed. A
  repaired head waits for a later landing, whose keys that did not
  change hold by their receipts.
- **A round of several units is narrowed by prefixes**
  (`landing-search.md`): the same gates at the unit boundaries below
  the candidate find a green prefix and the red one a unit above it;
  that unit's changes are repaired and leave, the green prefix lands in
  the round's turn, and the units above it go on in the next round.

### 7.4 A fault of the runner

- **The runner's, not the change's:** a job whose set-up step failed, or
  whose completion step alone failed, or that failed with no step
  failed; a run cancelled by no one of temper's; a run never found, or
  found at another commit; a dispatch refused for a missing workflow or
  runner. Never read from a log's words (`../draft/ci-on-forgejo-17.md`,
  2.1).
- **Retried, never repaired:** the failed jobs rerun (skein's
  `Write::Rerun`, naming each job's attempt, so that a rerun whose
  outcome was lost is found by the job's next attempt), or the gate
  dispatched again, up to the project's `gate_retries` per key: a run
  cancelled, or never found, is dispatched afresh, never rerun. No
  receipt is made.
- **Past its bound** the landing is held, saying which gate and why
  (4.2).

## 8. The turn

- **Held by the landing whose train it is,** for the seconds its merge
  takes: a train has one landing, so no other waits for its turn, and
  trains turn side by side.
- **First, freshness.** The train's tip and each member's head are
  read afresh. A head that moved makes its member leave (4.3). A tip
  that is not the round's base makes the round stale: a push from
  outside the landing moved the branch. The round ends unlanded (5.1),
  the stale count grows, and the landing prepares again in a new round,
  every member stacked afresh, keeping its receipts.
- **Then the branch fast-forwards to the round's proved prefix,** its
  candidate or, after a search, its green prefix, at which this
  section's candidate reads (`landing-search.md`, 5), by the merge of
  the round's pull request, fast-forward only, at the candidate (skein's
  `Write::Merge { how: FastForwardOnly }`), made by the landing task
  and judged by the landing rules for every member in the stack (`forge/change/gates.md`, 2, where a rule by path compares the train’s branch tip read afresh
  with the candidate, so it sees every member's paths, and `ci` is the
  test gates by receipt at the candidate; a gate's verdict counts at
  each member's prefix as section 3 says, and an approval counts each
  member's pull request's reviews at the heads it counts at, the prefix
  being a clean merge of the member's head, or a squash with that
  merge's tree; the merge is admitted only where each member's
  requirements hold).
  On GitHub, the landing advances the branch with a fast-forward-only
  `Write::UpdateRef`; temper’s App must always bypass a rule requiring
  a pull request (`forge/adoption.md`, 4).
- **Every member lands** once the branch's tip, read back, is the
  candidate: in the stack's order, the landing's state records each
  landed at its prefix, then its pull request reads merged, then its
  change is told it landed, at its prefix. The merge is the landing
  task's effect, but each landed commit is its member change's: the
  change lands, and its goal hears its own change's landing, as under
  `queue`. With members left in the landing, above a blamed unit, the
  next round starts (5.1); with none, the landing ends.
- **Its pull request reads merged:** a merged member's by the forge's
  detection (section 10), with nothing written. A squashed member's
  head is in no branch, so the landing records its pull request merged
  at the prefix, the merge (kind 4) recorded at a commit its base
  already holds (skein's `Write::Merge` with `how` `Recorded { commit
  }`; Forgejo's `manually-merged`), before its change is told, since a
  change's branch deleted as it closes would close its pull request
  unmerged. A record refused holds the landing, `turn: record of #N:
  WHY`, its member recorded landed; a release records it again, and the
  members above it go on. A landing cancelled once its members are
  recorded landed tells each change it landed, its pull request as it
  stands. On GitHub, which records no
  manual merge, the landing closes a squashed member's pull request
  instead (kind 12).
- **Nothing runs inside the turn:** no gate, no prepare, no run of an
  agent's. A merge refused or uncertain is settled as `forge/README.md`,
  section 4 settles any merge: a fast-forward-only merge raced by a
  push to the branch answers uncertain or refused, its find reads the
  pull request unmerged, and the freshness of the next try makes the
  round stale.
- **On GitHub,** which merges no pull request fast-forward, the turn
  would update the branch's ref to the candidate, fast-forward only
  (skein's `Write::UpdateRef`), under temper's App's `Always` bypass of
  a rule requiring a pull request (`forge/adoption.md`, 4); GitHub
  marks the pull requests merged once the branch holds their heads.
- **No turn without a fast-forward:** a repository where temper cannot
  fast-forward a branch to a candidate cannot land by turn: on GitHub,
  one whose landing branch's rules require a pull request that temper's
  App may not always bypass, or that requires the forge's merge queue,
  which lands by its own method at a commit temper did not prove. Its
  project's `turn` is refused, naming the repository and the one setting
  that would let temper land (the App's `Always` bypass; the queue's
  removal). There is no merge-commit fallback: the user's answer to the
  process model's open question, 2026-10-27,
  "for now if GitHub doesn't have the bypass, let's fail". The refusal
  says what is missing, the App's `Always` bypass on the landing
  branch's rule, and how a person grants it: in the repository's
  ruleset for that branch, the App added to the bypass list with
  "Always allow".

## 9. Trains

- **A train is a repository and the branch its changes land into:** a
  repository's main, or a goal's branch, each a train of its own.
- **Held** by its live landing from the decision that forms it to its
  end. Held is derived from the landings' phases, never stored.
- **Waiting changes** are ready (section 3) under turn and in no
  landing. Their order is `forge/queues.md`'s: the goal's priority, then
  the time each first became ready, which a change keeps while it stays
  live, with aging past the deployment's queue window, and the task's
  number last. Derived, not kept.
- **Forming is level-triggered:** whenever a train is free and has a
  waiting unit (4.1), every change of it waiting, a decision forms its
  next landing: the waiting units in order, each whole, up to the
  project's `batch` changes for a train into a repository's main, and
  one unit for a train into any other branch (13.1). The first unit is
  always taken, even past `batch`. The first unit that would pass it
  beside those before it ends the batch and waits, first in line. A refused unit (4.1) is held
  and passed over. Forming is judged when a change becomes ready, when
  a landing ends, when a change's priority moves, and at a restart, for
  each train they touch.
- **No race, no reservation:** a train's branch moves only by its
  landing's turn, or by a push from outside temper, which makes a round
  stale (section 8). Stale rounds are counted on the landing and
  reserve nothing.
- **Bounded:** `landings` counts live landings, one per train; a train
  with no room waits for one, as a change waits for a hold.

## 10. What the forge shows

- **The round's pull request,** the landing's own: from the top's
  branch into the train's branch, titled `Land #N: TITLE`, N and TITLE
  the member's pull request's number and title, for a stack of one, and
  `Land K changes: #N1, #N2, …` for a stack of K, the numbers in its
  order, as many as the title's bound holds whole; its body naming the
  round and its base, then a line per member in the stack's order: its
  change, pull request, head and prefix, then, after a search, the
  search's line (`landing-search.md`, 8). It is merged by the turn, or
  closed as its round ends unlanded or its search cuts the stack.
- **Each gate as a commit status on the candidate,** and on each prefix
  a search probed,
  `temper/gate/<name>`: pending with its key's first 16 hexadecimal
  digits once dispatched; success or failure with the run as its link
  once its receipt is made; success with the receipt's run as its link
  when a receipt held with no run. A fast-forward makes the branch's
  commit the candidate, so the branch shows the statuses that proved
  it, and protection may require them.
- **A merged member's pull request is marked merged** by the forge's
  detection of a manual merge (Forgejo's "autodetect manual merge",
  which adoption reads, 13.2): its head is in the branch, its merge
  commit its prefix. temper writes nothing to it at the turn.
- **A squashed member's is recorded merged** by the landing at its
  squash commit (section 8), Forgejo's "manually merged" with that
  commit, which needs the repository to allow a manual merge (13.2).

## 11. Effects and kinds

Three kinds join `forge/README.md`'s section 4, each a row of its own:

| Kind | Effect | What it is | Recovery class | Found again after an uncertain failure |
|---|---|---|---|---|
| 19 | update a branch's ref, fast-forward only (GitHub's turn) | a transition: the branch moved to the candidate | conditional: the forge refuses a move that is not a fast-forward | the branch at the candidate, or at a commit containing it |
| 16 | dispatch a workflow | a creation: a run of a workflow at a branch, with inputs | unrecoverable: the forge keeps no key for it | among the runs of its workflow at its commit made after the entry's start |
| 17 | rerun a job | a transition: the job's next attempt | unrecoverable: the forge checks no attempt | the job's attempt moved past the one named |

- **Two change their parameters:** the merge (kind 4) carries how it
  merges, the repository's style, fast-forward only, or recorded at a
  commit its base already holds (a squashed member's pull request,
  section 8), the last a new variant at the end, found again after an
  uncertain failure by its pull request read merged at that commit; the
  update (kind 11), merge or rebase.
- **The landing's own effects** are its dispatches, reruns, statuses
  and merge (kinds 16, 17, 6 and 4) on its train's repository, a
  squashed member's record (kind 4) or, on GitHub, its closing (kind
  12), and its round's pull request's opening and closing (kinds 3 and
  12) and its branches' deletions as a round ends unlanded (kind 15, on
  its own held branches). The prepare job's pushes are the worker's, under the landing's
  holds; the branches are released as the landing closes, by the release a
  change's branch takes (`forge/branches.md`).
- **The landing rules govern kind 19 as they govern kind 4,** the merge
  (`forge/change/gates.md`, 2): each requirement judged at the candidate, CI being
  the test gates by receipt.
- **The grant:** the project's policy gives the landing procedure kinds
  3, 4, 6, 12, 15, 16, 17 and, on GitHub, 19 on the project's
  repositories, as it
  gives a queue's repair its kinds (`forge/queues.md`); temper's
  root grants them to each landing task it makes. Until jig gives the
  tasks the deployment starts on a project's behalf only what the
  policy gives them (jig's `domain/authority.md`, section 6, built so
  far as the project's whole ceiling), a landing holds the ceiling, as
  a queue's repair does; its effects are still each admitted by the
  core and judged by the enforced landing rules.
- **A purpose of its own,** tag 6 in `forge/README.md`'s table: a landing's
  step, its task (8 bytes), the member (2), the gate (2), the
  operation (1), the round,
  counted from 1 in the landing (2), and the try within it (2), 18
  bytes with the tag. The member is the one the step writes for: the
  round's top for its gates, statuses, pull request and merge, a
  squashed member for its record, and each deleted branch's member for
  its deletion. The operations' codes are the order they were
  made: 1 a dispatch, 2 a rerun, 3 a pending status, 4 a verdict's
  status, 5 the merge, then 6 an opening, 7 a closing, 8 a deletion and
  9 a squashed member's record. A baseline's dispatch is its gate's, tag 7: the
  repository (16), the gate (2) and the first eight bytes of the key
  (8), 27 bytes with the tag.

## 12. In the store

- **`Landing`,** keyed by its task: its project, phase, train's branch,
  members in the stack's order (each its repository, change task, pull
  request, its unit's number, its branch in the landing, the head and
  the base of its round, the upstream's tip it read, its prefix and how
  it combined the member, each gate's key and state at its prefix,
  whether its judgement was asked, its landed commit, whether its pull
  request's record is made and its change told, or why it left), its
  round, counted from 1, the round's pull request, its search's bounds
  (`landing-search.md`, 9), its stale rounds and
  its faults per key, and, held, its cause. Why a member left is one of
  4.3's causes.
- **Single landings outside a train:** an upgraded v15 landing keeps its
  phase, members, preparation, candidates, gate proofs, asks and pending
  effects. Its target is the stored adopted repository's default branch,
  and its round is absent. It finishes through the single-landing
  procedure: preparation updates the member, gates follow its branch,
  and the turn merges its pull request at the proved candidate. It opens
  no round branch or pull request. The domain represents an absent round
  by an empty round branch; only the single-landing procedure handles it.
  A new landing always has a named round branch. Such a single landing
  holds its train until it ends; already pending effects retain their
  purposes and identities.
- **`Receipt`,** keyed by the project, the repository, the gate and the
  key: the verdict, the run, the candidate and the time.
- **A change's state** grows its landing, its clean merges' heads, its
  way (`queue` or `turn`), its judgement at its prefix, its landing's
  asks: judge, resolve, repair, landed.
- **Patched,** before the first release (`../protocol/store.md`,
  section 2): each new member at the end of its record, added by the
  store's patch (`../protocol/baseline.md`, 7), every existing row
  taking none, zero, false or empty, its member branch its round's
  branch; and a new variant at the end of an enumeration.
- **Bounded** by the forge's limits (`../protocol/configuration/limits.md`
  ): `landings`, live landings per deployment, one per train;
  `landing_members`, changes per landing, at least every project's
  `batch`, each member a branch the landing holds;
  `gates`, test gates per repository; `receipts_per_gate`;
  `ignored_paths`, per gate, each within a branch name's bounds; and
  `inputs`, within skein's `INPUTS`. A landing that would pass one is
  refused at its forming, naming the limit; its changes stay ready and
  wait for room, as a change waits for a hold. Entrance constraints
  and representable counts are checked by the forge's `valid`.

## 13. Settings

### 13.1 Kinds

Two kinds of temper's join `../protocol/configuration/settings.md`, 2, each
the forge's, judged in the change's decision:

| Kind | Scope | Numbers | Value |
|---|---|---|---|
| 4, a project's landing | each project | 1 | the members below |
| 5, a repository's gates | each project | the repository's adoption, unique in the deployment (its forge's number shifted 16 bits left, with the forge's repository number below) | the repository (its forge and id); `gates`, its test gates, each a name, the workflow's file, its ignored paths, whether it blocks, and whether it reads the base (`base`, no when absent), each proving the trains into its main (7.1); `tiers`, each a name, the workflow's file, its ignored paths, the artifact and its format, and the branch it follows (`after-landing.md`, section 2) |

Kind 4's members, each with its default and the section it serves:

| Member | Default | Serves |
|---|---|---|
| `by`: `queue` or `turn` | `queue` | section 1 |
| `batch`: the most changes a landing into a repository's main forms with, from 1 to the deployment's `landing_members` | 1 | 4.1, section 9 |
| `combine`: how a train combines its members (5.2), `merge` or `squash`, `main` for the trains into a repository's main and `other` for those into any other branch | `merge`; `squash` | 5.2, 8 |
| `prepare`, `key`: paths in the home, or none; `home_revision`; `prepare_timeout` | temper's own prepare and key | 5.3, 6.1 |
| `merge_job`: a path in each repository it merges, or none | none: a conflict goes to a resolution at once | `forge/change/repairs.md` and `forge/README.md`, 6 |
| `dependencies_join` | yes | 4.1 |
| `pins`: each a downstream and an upstream repository; `pins_follow_main` | none; no | 4.1, 5.3 |
| `gate_retries`; `gate_timeout` | 2; one hour | 7.4; 7.1 |
| `tier_retries`; `verification_bound`; `suspects`: `bisect` or `tell_all`; `red_pauses` | 2; two hours; `bisect`; no | `after-landing.md`, 3, 5.2, 6, 7 |
| `findings`: on or off; `triager` and `chooser`, a task or a role; `tester_charter`; `areas`, each a repository and an area | off; the person's chat; the chat; none | `findings.md`, 2, 6, 8 |
| `questions_on_forge` | off | `forge/issues.md`, 1 |

- **Until both kinds are settings** on jig's application settings (jig's
  `domain/engine/settings.md`, 5), a project's `by`, its `batch`, its `combine`, its `tier_retries` and
  `verification_bound` (`verification_bound_ms` there), its
  `gate_timeout` (`gate_timeout_ms` there), its
  `merge_job`, and its repositories' gates and tiers are
  configured in the document's `landing` member
  (`../protocol/configuration/engine.md`), as the landing rules are,
  and read at the start.
- **Read by a branch's path** (`forge/change/branch-path.md`): the project's `by`,
  and each repository's gates and tiers with the branch each tier
  follows, as the forge holds them, so that a path draws "prepare",
  "prove" and "turn" under `turn` and a tier's stops where it has one.
- **`turn` is refused** for a project with a repository where temper
  cannot fast-forward main to a candidate (section 8: on GitHub, no
  `Always` bypass of a rule requiring a pull request, or a required
  merge queue; or an installation that cannot write statuses or
  dispatch workflows), naming the repository and the setting that
  would let it land.
- **A gate's name** is letters, digits, `-` and `_`, unique in its
  repository, and its status context's last segment.
- **The stock's deployment ceiling** admits what landings and tiers are
  granted (kinds 3, 4, 6, 7, 9, 12, 15, 16 and 17, and the forge's
  procedures `landing` and `tier`), since a project's ceiling may not
  pass the deployment's; each grant grows every row that carries
  authority, so none is added that a release at close can spare.

### 13.2 What adoption says

- **Under `turn`,** for each repository: its gates, each workflow
  checked to exist at the landing branch's tip and to name
  `workflow_dispatch` alone among its triggers, or said otherwise in
  words; whether the forge marks a pull request merged once its base
  holds its head (Forgejo's "autodetect manual merge"), which lands a
  merged member's pull request (section 10); where one of the project's
  trains squashes, whether the forge lets a pull request be recorded
  merged by hand (Forgejo's "allow manual merge"), which a squashed
  member's record needs (section 8); and whether the branch's rules
  admit the landing's pull request with the gates' statuses on the
  candidate.

## 14. Views and acts

Under `../client/ctl/tools.md`, 1's names (a group `landing`):

| Tool | What it answers or does |
|---|---|
| `landing show LANDING` | a landing: its train, its round and its base, its pull request, its members in the stack's order, each with its unit, its branch, its head, its prefix and how it combined the member, its judgement, and, ended, its landed commit or why it left; each gate's key, receipt or run at the top and at each probed prefix; its round's search (`landing-search.md`, 8); its step, its stale rounds and faults |
| `landing list --project PROJECT [--ended]` | the project's landings, newest first, with their trains and phases; the live ones first |
| `repository receipts REPOSITORY [--gate GATE]` | a repository's receipts, newest first: gate, key, verdict, run, candidate, time |
| `landing history --project PROJECT [--change TASK \| --landing TASK] ...` | the steps of the project's changes and landings as events, kept after their rows go (`landing-history.md`, 5.1) |

The reads above describe turn landing tasks. A queue change creates no
landing task: `landing show TASK` on its change task answers
`no landing for task TASK` (exit 2), in text and JSON, and its queue merge
does not appear in `landing list`, including with `--ended`. Read the
change and its merge with `change show TASK`. A matching change document
on the shared task watch means no landing; malformed documents and
documents for another task or project remain errors.

- **The changes view** (`forge/queues.md`'s subject `changes`), under
  `turn`: per train, its landing in flight, with its members in the
  stack's order, its round and step, probing with its prefix while its
  search probes, then the waiting changes in their order.
- **A change's panel** (`forge/change/README.md`, 5) names its landing and its way.
- **Acts** are jig's on the landing task: release and cancel, by the
  project's maintainers.

## 15. Restart

- **Restored whole** with the forge's other families (`forge/README.md`,
  section 9). Trains are derived again from the live landings.
- **Before any landing steps,** the train's tip and each member's head
  are read afresh, and each run in flight is read (`Read::Run`); a
  dispatch whose outcome was uncertain is found by its runs, as section 11 says.
- **In each phase:** preparing, the job's delegate is jig's and lives
  on, and the members' branches are read before any new job (5.2);
  proving or probing, its runs are found again, and its search at its
  stored bounds (`landing-search.md`, 6); the round's pull request, its
  opening uncertain, is found by its branches; judging, each change's
  state carries it; in its turn, its merge settles as `forge/README.md`'s merges
  settle, and each member found landed is landed, recorded once; a
  squashed member's record, uncertain, is found by its pull request
  read merged at its prefix, and made where it is not; ended, its
  release goes on.
- **Forming** is judged again for every train (section 9).

## 16. The world

Stories in the forge's world (`forge/README.md`, section 10; `../testing/fakes.md`,
2), the fake forge running dispatched workflows, detecting manual
merges and recording one at a commit (skein's `testing/fakes/forge.md`,
2.3), and the engine's domain
stories, with the prepare job's outcome given as a worker gives it. The
forge's world holds:

- **a landing of one** is prepared by its job onto the landing's
  branch, proved by one dispatched run there, its pull request
  fast-forwarded; the branch carries its gate's status, and the
  member's pull request reads merged;
- **trains:** one landing per train, the rest waiting in order, aging
  included; a goal's branch's train beside main's, with no test gate,
  one change at a time whatever `batch` says; forming when a train
  frees and at a restart; a unit across repositories refused, naming
  its changes;
- **forming:** up to `batch` changes in order, the first unit taken past
  it, the first that would pass it waiting first in line; units whole;
  sealed against a change ready later or a dependency given later;
- **a batch** of three stacked by one job, a prefix per member on its
  branch, proved by one run at the top, landed by one fast-forward,
  every member's pull request merged and every change told its prefix;
- **a lost or partial prepare job** is found by its branches, the
  prefixes it pushed taken and a job made for the rest, and the
  landing goes on, no one asked;
- **a member leaving mid-proof** ends the round: the branches from it
  up deleted, those below kept, the members above stacked again on the
  last kept prefix; the top leaving needs no job; a unit leaving whole,
  and a dependent with its dependency;
- **conflicts told apart:** one with the branch resolved, one with an
  earlier member alone waiting first in line and landing next;
- **a red round of several units** narrowed by prefixes, as
  `landing-search.md`, section 10 lists;
- **the turn's merge** judged for every member in the stack;
- **combining:** a merge train and a squash train over the same member
  give the same tree and keys; a squash commit's single parent, message
  and trailers; a member bringing its train's upstream's commits merged
  under `squash`; a squashed member's pull request recorded merged at
  its commit, and a refused record holding the landing;
- **a documentation change** whose every gate ignores its paths lands
  on its receipts, with no run;
- **a tree is never gated twice:** a later landing reaching a key
  already received runs nothing;
- **a gate that reads the base** is given it, and its key moves with
  the base;
- **leaving:** a conflict resolved by a merge of the base into the
  member, which lands in a later landing; a moved head; a cancel; a
  hold; a refused judgement gate; each keeping its place in line;
- **judgement gates:** a `clean` verdict carried to the prefix, an
  `exact` one asked at the prefix on the landing's pull request;
- **a real failure** repairs the member, which leaves, and only its
  changed keys run in its next landing;
- **a runner's fault** is rerun, never repaired, and past its bound
  holds the landing; a run past `gate_timeout` holds it;
- **a stale base at the turn** ends the round, every member stacked
  again, and the next round lands;
- **a restart** in each phase finds the landing again and carries it
  on, a batch's included;
- **news** at each step to each member's goal, and the changes view
  per train;
- **under `queue`,** every story of `forge/README.md`'s world passes unchanged;
- **on the fake GitHub:** a repository without `Always` refused,
  naming the bypass and how a person grants it; a required merge queue
  refused.

## 17. Open questions

- **Reviews in a landing:** before forming (the default here), or on
  the candidate.
- **A judgement gate's key** as a patch id of the member's diff, which a
  merge over conflicting lines would keep and a clean merge (the
  mechanism here) may not; kept as clean merges until a project needs
  more.
- **A gate for a goal's branch's train:** none here (7.1); a project
  that wants one names it when one does.
