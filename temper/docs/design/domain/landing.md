# Landings

Provisional, 2026-11-19. How a project that lands by turn lands its
changes: a train per repository and branch, one landing at a time on
each, the next formed from the changes waiting in the queue's order, a
batch of them up to the project's `batch`; each landing's members
stacked by a merge job, a prefix per member on a branch the landing
holds; gates proved by receipts kept by key, at the stack's top; and
the landing's own pull request fast-forwarded into the branch, which
lands its members by containment. It is the merge queue
(`../draft/merge-queue.md`); a red round of
several units is narrowed by prefixes, `landing-search.md`. It replaces, for such a project, the change's CI, its queue,
its updates and its merge in the repository's style
(`forge/change/README.md`, 2 steps 3 and 5 to 7, `forge/queues.md` and
`forge/updates.md`, section 1). What follows a landing, its tiers and its
verdict, is `after-landing.md`. The forge's calls it makes are skein's
`forge/vocabulary.md`, 1.2. What is still open is `landing.md`, section 17.

## 1. In one page

- **A project lands by queue or by turn,** its setting (`landing-settings.md`, section 1).
  `queue` is the change procedure (`forge/change/README.md`,
  `forge/queues.md`, `forge/updates.md`, section 1), the default until
  that project's turn has been walked on real backends. `turn` is this
  document. A change takes its project's way when it becomes ready.
- **A train** is a repository and its landing branch (`landing-turn.md`, section 2): main
  or a goal's branch. At most one landing holds it; other ready changes
  wait in queue order. Only that landing moves the branch, so another
  landing's turn cannot stale its proof.
- **A landing** batches units, each a change or changes landing
  together (section 4). Formed and sealed when its train frees, it
  takes waiting units in order, up to the project's `batch` changes.
  That decision creates its forge procedure task, requested by the
  deployment for the project. Member change tasks wait on it; their
  goals hear each step as news.
- **Prepare** (`landing-prepare.md`, section 1) is a worker job stacking members' heads from
  the train's tip by its `combine` (`landing-settings.md`, 1.1): a no-fast-forward merge
  (stock into main) or a squash with that merge's tree (stock into
  other branches, a goal's). Each commit is its member's prefix, pushed
  to a member branch held by the landing; the last is the candidate.
  Members' own branches stay unchanged. A conflict sends its member to
  resolution, or back to wait if only with an earlier member; it leaves
  and the job stacks the next.
- **Receipts by key** (`landing-proof.md`, section 1). A key captures a gate's dependencies:
  its definition, the candidate's tree minus ignored paths, and the
  base's tree if it reads the base. A receipt keeps its result under
  that key per project, reused by every landing and worker. A gate
  never reruns a receipted key.
- **Proof by dispatch** (`landing-proof.md`, section 2). Only temper dispatches gate
  workflows, at the candidate atop its branch. A real failure in a
  one-unit round repairs its changes, which leave; with several units,
  prefix search blames and repairs a unit and lands the green prefix
  below (`landing-proof.md`, 2.3). Runner faults retry. Judgement gates hold at each
  member's prefix (`landing.md`, section 3).
- **The turn** (`landing-turn.md`, section 1) rereads the train's tip and members' heads,
  then merges the landing's pull request fast-forward only at the
  round's proved prefix: its candidate or search's green prefix.
  Contained by head or squash commit, each member has landed at its
  prefix, the commit that brought it. No gate runs inside the turn or
  again on the branch: its commit is already gated.
- **A member leaves** before the turn on conflict, moved head, cancel,
  hold, refused judgement gate or red, with its whole unit (4.3),
  keeping its place in line. Members below keep their prefixes; those
  above are restacked next round.
- **The forge shows it** (section 10): the landing's pull request;
  receipts as candidate commit statuses, `temper/gate/<name>`, carried
  onto the branch by fast-forward; and member pull requests merged,
  detected by the forge as manual merges or recorded by the landing
  at their squash commits.

## 2. Structure

```
temper-engine-domain-forge              the top: landings' and receipts' states, the trains, routing; the effects
├── temper-engine-domain-forge-change   the change procedure: under turn, up to ready, its judgement at its prefix, its repairs and resolutions
└── temper-engine-domain-forge-landing  the landing: decisions over its state and the facts; keeps nothing
```

- **The top keeps** each landing's state (family `Landing`), the
  receipts (family `Receipt`) and the keys in flight, as it keeps the
  changes' states (`forge/README.md`, section 3). Trains are derived,
  never stored: a train is held by its live landing (`landing-turn.md`, section 2).
- **`landing` is policy,** as `change` is: given a landing's state, its
  members' states, the receipts and the facts the top read, it says what
  is due and the landing's next state, one decision a step:

  | Decision | What the top does |
  |---|---|
  | prepare | delegates the round's prepare job (`landing-prepare.md`, 1.2) from the round's base or the last kept prefix, for the members from the first with no prefix |
  | open | opens the round's pull request, the top's branch, or the green prefix's after a search, into the train's (section 10) |
  | key | reads the trees a gate's key needs, at the candidate or a probed prefix and, for a gate that reads the base, at the base (`landing-proof.md`, 1.1) |
  | dispatch, retry, post | a gate's run, its rerun, its status on the candidate or a probed prefix (`landing-proof.md`, 2.1, 2.4; section 10) |
  | judge | asks a member's change for its judgement gates at its prefix (section 3) |
  | merge | merges the landing's pull request, fast-forward only, at the round's proved prefix (`landing-turn.md`, section 1) |
  | record | records a squashed member's pull request merged at its prefix (`landing-turn.md`, section 1) |
  | landed | tells a member's change it landed, at its prefix (`landing.md`, section 3) |
  | leave | records why a member left, with its unit, and asks its change for what follows: a resolution, a repair, or nothing (4.3) |
  | close | closes the round's pull request and deletes the branches from the round's first changed member up: the round ends unlanded (`landing-prepare.md`, 1.1) |
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
- **The worker** runs a landing's prepare (`landing-prepare.md`, section 1) as a job, never
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
  (`landing-proof.md`, section 2), and the change has no checking step.
- **Ready:** its head pushed and its pull request open; every blocking
  judgement gate the landing rules and the change ask for (a review, a
  person's approval: `forge/change/gates.md`, 1) holds at its head; and its task
  is not kept by a dependency. A review runs as soon as the head is
  pushed. Ready and in no landing, it waits for its train (`landing-turn.md`, section 2),
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
  candidate, given the failed run's output (`landing-proof.md`, 2.3), or at the red prefix
  its landing's search blamed (`landing-search.md`, 7). The member leaves its
  landing; its new head makes the change ready again, and it waits for a
  later landing, first in line by its kept time.
- **Resolving** (`forge/change/repairs.md`): a conflict with the branch the prepare job met, in
  a workspace where the round's base is merged into the change's branch
  with the files in conflict marked, made afresh from the forge. Its
  pushed merge contains the base, and it waits for a later landing. A
  conflict with an earlier member alone asks nothing: the change stays
  ready and waits, first in line (`landing-prepare.md`, 1.2).
- **Leaving otherwise:** a new head its goal delivered takes the change
  back to gating at that head, a head moved by another identity holds
  it as drift (`forge/change/drift.md`, 1), and a cancel ends it; in
  each the member leaves (4.3).
- **Landed** when its landing's turn has made the train's branch
  contain its prefix: its head, merged, or its squash commit. Its landed
  commit is its prefix, the commit that brought it, which the landing
  tells it; it reads no pull request merged for it. Its pull request
  reads merged, by the forge's detection of a manual merge, or recorded
  so by the landing for a squashed member (`landing-turn.md`, section 1). A squashed
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
    `domain/tasks/lifecycle-endings.md`, 1): the dependent is kept, so not
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
  that forms it (`landing-turn.md`, section 2), with its members. It holds each member's
  branch in the landing from its making (`landing-prepare.md`, 1.1). The landing is saved once
  jig has made it (`landing-forming.md`).
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
  unless it is recorded landed (`landing-turn.md`, section 1), and the train forms again.
  Either way every change that names it, member or not, stops naming
  it as it ends, an ask it still keeps for one going onto that change.
  Either end wakes its own train's waiting changes, no other's, and
  its writes leave the outbox (`forge/room.md`, 2): its entries not
  yet sent withdrawn, each in flight as it settles. Cancelled, it
  follows its runs no further (`landing-proof.md`, 2.1).
- **No silent wait: held** when a step makes no progress past its bound,
  naming the step and what it waits on, its members' goals hearing it:
  - the prepare job failed past its tries (`landing-prepare.md`, 1.2), `prepare job: WHY`;
  - a gate's runner faults past `gate_retries` (`landing-proof.md`, 2.4), `gate NAME of
    REPOSITORY failed on its runner past its retries`;
  - a gate's run in flight past the project's `gate_timeout` (`landing-proof.md`, 2.1),
    `gate NAME of REPOSITORY: run RUN running past its bound`, or past
    the runner wait, `...: run RUN waiting for a runner past its bound`, then,
    after `; `, a clause for each of the run's jobs not completed, as a
    change's stall names its pending checks
    (`forge/change/stalls.md`, 4), each named
    by the job's name in its run: `JOB: waiting for a runner labelled
    L1, L2`, `JOB: blocked by the jobs it needs`, `JOB: running at step
    N, NAME`, `JOB: pending, its job unread`;
  - an effect refused at admission or failed at the forge, but its
    merge, naming the operation, its resource and the refusal or forge
    answer, with its outbox entry and the task's release command;
  - its pull request refused for a cause not stale, `turn: WHY`; its
    merge not made, read again at the round's base with its pull request
    not closed, refused or past its bound (`landing-turn.md`, section 1), `turn: merge of #N: WHY;
    read #N STATE, BRANCH at TIP`.

  A step a member's change makes (its judgement, its repair) is
  bounded by the change's own stall (`forge/change/stalls.md`): the
  change holds, and the member leaves. A release lifts the landing's
  hold, and it prepares again in a new round, keeping its receipts.
  Short of a hold, a step waits only on what it names, and with no
  deadline only on its own effect, a change's step or a release
  (`landing-waits.md`, 3).
- **Drained** (jig's `domain/connectors-procedures.md`, 1.7): a landing in a
  draining hold's scope that has begun its proof goes on, through the
  turn, to land; one still preparing is idle, and held. A stale base or
  a fault that would send it back to prepare makes it idle there.

### 4.3 Leaving

- **A member leaves its landing,** with its whole unit, before the
  turn, when:
  - its merge onto the stack conflicts (`landing-prepare.md`, 1.2), with the round's base:
    `conflict with the branch: FILES`, and its change resolves; or with
    an earlier member alone: `conflict with an earlier member: FILES`,
    and its change waits;
  - its head moved, its goal having delivered or another hand having
    pushed: `head moved to HEAD`;
  - it is cancelled, `cancelled`, or held, `held`;
  - a judgement gate at its prefix says no (section 3): `gate NAME
    refused`, and its change repairs;
  - a test gate fails at the candidate in a round of one unit (`landing-proof.md`, 2.3):
    `gate NAME failed in RUN`, and its change repairs; or its round's
    search blames its unit: `gate NAME failed in RUN at RED, above
    GREEN`, and its change repairs (`landing-search.md`, section 4);
  - another change of its unit left: `change C of its unit left`; or
    its dependency, another member, left: `its dependency, change C,
    left`.
- **It keeps its place:** its first ready time is unchanged, so once
  ready again it is first in line by that time. Leaving is never a
  repair by itself.
- **Its link goes as it leaves:** at the leave, whatever its change
  does next, the change stops naming the landing (section 12), and the
  landing's ask for it, a resolution or a repair with its context, goes
  onto the change, kept there until it takes it. No landing keeps a
  change that left it out of its queue.
- **The rest stay:** a member that leaves while the job stacks is
  skipped by that job (`landing-prepare.md`, 1.2). One that leaves later ends the round (`landing-prepare.md`, 1.1),
  but for a unit its round's search blames (`landing-search.md`, 4):
  the members below it keep their prefixes and receipts, those above
  it are stacked again onto the last kept prefix, and the landing
  proves its new top. With no member left, the landing ends (4.2).

## 5. Prepare

Moved to [`landing-prepare.md`](landing-prepare.md), section 1.

## 6. Keys and receipts

Moved to [`landing-proof.md`](landing-proof.md), section 1.

## 7. Proof

Moved to [`landing-proof.md`](landing-proof.md), section 2.

## 8. The turn

Moved to [`landing-turn.md`](landing-turn.md), section 1.

## 9. Trains

Moved to [`landing-turn.md`](landing-turn.md), section 2.

## 10. What the forge shows

See [Landings at the forge](landing-forge.md), section 1.

## 11. Effects and kinds

Three kinds join `forge/effects.md`'s section 1, each a row of its own:

| Kind | Effect | What it is | Recovery class | Found again after an uncertain failure |
|---|---|---|---|---|
| 19 | update a branch's ref, fast-forward only (GitHub's turn) | a transition: the branch moved to the candidate | conditional: the forge refuses a move that is not a fast-forward | the branch at the candidate, or at a commit containing it |
| 16 | dispatch a workflow | a creation: a run of a workflow at a branch, with inputs | unrecoverable: the forge keeps no key for it | among the runs of its workflow at its commit made after the entry's start |
| 17 | rerun a job | a transition: the job's next attempt | unrecoverable: the forge checks no attempt | the job's attempt moved past the one named |

- **Two change their parameters:** the merge (kind 4) carries how it
  merges, the repository's style, fast-forward only, or recorded at a
  commit its base already holds (a squashed member's pull request,
  `landing-turn.md`, section 1), the last a new variant at the end, found again after an
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
- **A full outbox is a wait:** a landing's write that finds no room
  waits in the outbox's line, at the step its decision started from,
  and decides again when asked (`forge/room.md`, 3), never a hold.
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
  effects. Its target is the stored adopted repository's landing branch
  (`forge/adoption.md`, 8),
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
  asks: judge, resolve, repair, landed. It names only a landing it is
  in and has not left; an ask handed over as it left or as its landing
  ended is kept on the change (4.2, 4.3). The landing's row keeps the
  times of its last event and its turn's start, and its latest wait
  (`landing-waits.md`, 4).
- **Registered layouts,** converted at read (`../protocol/store-versions.md`,
  section 1): new fields declare their defaults, renamed parts their
  former names. A change no plan derives is a written job, not a map
  patch; no version moves (`../protocol/floor.md`, 6). The floor's
  rows already hold each member's branch; opening guesses none.
- **Bounded** by the forge's limits (`../protocol/configuration/limits.md`
  ): `landings`, live landings per deployment, one per train;
  `landing_members`, changes per landing, at least every project's
  `batch`, each member a branch the landing holds;
  `gates`, test gates per repository; `receipts_per_gate`;
  `ignored_paths`, per gate, each within a branch name's bounds; and
  `inputs`, within skein's `INPUTS`. A landing that would pass one is
  refused at its forming (`landing-forming.md`, 2.3), naming the limit; its changes stay ready and
  wait for room, as a change waits for a hold. Entrance constraints
  and representable counts are checked by the forge's `valid`.

## 13. Settings

A landing's settings are `landing-settings.md`'s (2026-11-16).

### 13.1 Kinds

`landing-settings.md`, 1.1.

### 13.2 What adoption says

`landing-settings.md`, 1.2.

## 14. Views and acts

Under `../client/ctl/tools.md`, 1's names (a group `landing`):

| Tool | What it answers or does |
|---|---|
| `landing show LANDING` | a landing: its train, its round and its base, its pull request, its members in the stack's order, each with its unit, its branch, its head, its prefix and how it combined the member, its judgement, and, ended, its landed commit or why it left; each gate's key, receipt or run at the top and at each probed prefix; its round's search (`landing-search.md`, 8); its step and what it waits for (`landing-waits.md`, 4), its stale rounds and faults |
| `landing list --project PROJECT [--ended]` | the project's landings, newest first, with their trains and phases, each live one's step and the age of its last event, and the turn holder with how long it has held its turn; the live ones first |
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
- **A change's panel** (`forge/change/panel.md`, 1) names its landing and its way.
- **Acts** are jig's on the landing task: release and cancel, by the
  project's maintainers.

## 15. Restart

- **Restored whole** with the forge's other families (`forge/effects.md`,
  section 2). Trains are derived again from the live landings. A live
  landing jig never made ends at the link pass (`landing-forming.md`, 4).
- **Before any landing steps,** the train's tip and each member's head
  are read afresh, and each run in flight is read (`Read::Run`); a
  dispatch whose outcome was uncertain is found by its runs, as section 11 says.
- **In each phase:** preparing, the job's delegate is jig's and lives
  on, the start's read decides nothing, and the round's prepare is read
  from jig's restored context (`landing-preparing.md`, 3), the members'
  branches before any new job (`landing-prepare.md`, 1.2);
  proving or probing, its runs are found again, and its search at its
  stored bounds (`landing-search.md`, 6); the round's pull request, its
  opening uncertain, is found by its branches; judging, each change's
  state carries it; in its turn, its merge settles as `forge/README.md`'s merges
  settle, and each member found landed is landed, recorded once; a
  squashed member's record, uncertain, is found by its pull request
  read merged at its prefix, and made where it is not; ended, its
  release goes on.
- **A link to an ended landing is dropped** as the store is read: a
  change naming a landing ended, or one the store does not hold, stops
  naming it, an ask that landing still keeps for it going onto the
  change; ready, it waits in its train's queue by its kept time.
- **Forming** is judged again for every train (`landing-turn.md`, section 2).

## 16. The world

The forge's world (`forge/README.md`, section 10; `../testing/fakes.md`,
2) uses the fake forge's dispatched workflows and detected or recorded
manual merges (skein's `testing/fakes/forge.md`, 2.3); engine domain
stories supply prepare outcomes as a worker does. The forge's world holds:

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
- **a runner's fault,** an unreported run's too, is rerun, never repaired,
  and past its bound holds the landing; a run cancelled by a person is dispatched afresh
  at its key and candidate; a run past `gate_timeout` holds it;
- **a stale base as prepare ends or at the turn** ends the round, every
  member stacked again once on the new base, and the next round lands;
  a released prepare's prefixes are cleaned before that new job, including
  after restarts during cleanup and before the new preparation's delivery;
- **a restart** in each phase finds the landing again and carries it
  on, a batch's included;
- **news** at each step to each member's goal, and the changes view
  per train;
- **under `queue`,** every story of `forge/README.md`'s world passes unchanged;
- **on the fake GitHub:** a repository without `Always` refused,
  naming the bypass and how a person grants it; a required merge queue
  refused.

The domain's stories add, through its root: a restart in each phase,
the prepare job living on its worker; a round at preparing recovering
across a restart, a worker's loss and a release
(`landing-preparing.md`, 7); `batch` raised at a restart; a
saved delivery across its worker's loss beside a proving landing; going
live by `turn` with running work's settings kept (13); a copied policy's
ceilings on the shipped capacities; a goal's delivery leaving its
landing mid-proof, or sharing its run; a turn's merge not made (8):
raced by a goal's design delivery and answered `500`, its round ending
with no hold and the next landing; lost and found merged, landed; lost
with its pull request open, made again and landed; withdrawn with it
open, asked again and landed; closed unmerged,
prepared again; past its tries, held with what it read, another goal's
train landing beside it; two goals' changes, each
approved by its review and waiting on one person task, released
together by the person's answer, forming one landing of two by `turn`
at a `batch` of two, proved by one green run at the top and landed by
one fast-forward, each member's pull request merged at its prefix and
each change landed there, as the turn-landing setup's scripts carry
them (`../testing/qa/setups.md`, 6).

## 17. Open questions

See [Landings at the forge](landing-forge.md), section 2.
