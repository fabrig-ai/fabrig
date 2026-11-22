# The forge's effects and recovery

This document holds effects and their recovery across restarts, split from [`README.md`](README.md).

## 1. Effects

Every effect is committed into the outbox and made by `client`, in
commit order per lane (a pull request, an issue, a repository) and one
at a time on each (jig's `domain/connectors.md`, 4.2). The top owns the
effects and their identities; the client's keys, its write lanes and its
recovery are skein's `forge/client.md`, section 7. temper's effects:

| Kind | Effect | What it is | Recovery class (jig's `domain/connectors.md`, 4.3) | Found again after an uncertain failure |
|---|---|---|---|---|
| 3 | open a pull request | a creation: from a branch into its landing branch, with a title, a body and a key | conditional: only while no pull request for its branches is open; Forgejo keeps it, closed or not | by its branches; the change accepts an open pull request alone, never a closed or merged one (`change/README.md`, 2, step 2) |
| 10 | edit a pull request's title, body or base | a set | idempotent | written again |
| 11 | update a pull request from its base | a transition: the base's tip merged into its branch, or its commits rebased onto it (`../landing.md`, 5.2), by the forge | idempotent: every copy leaves its head containing the base, or changes nothing; a late copy may move the head (`updates.md`, section 1) | its head contains the base tip it asked for |
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
  write's outcome or its withdrawal. A full outbox is a wait, never a
  hold (`room.md`): a change's effect, a landing's write, release
  cleanup and a goal branch's making that find no room wait in one line,
  first come first served, and are admitted in order as entries leave,
  each reading its facts afresh and admitted by its authority again; the
  wait survives a restart in its place, and a cancel ends it. An agent's
  call that finds no room is answered `Unavailable` at once, the room its
  check finds reserved until its entry is kept (`room.md`, 5); a goal's
  projection waits its interval (`issues.md`). Only a write that can
  never be sent refuses: one its repository's adoption does not take,
  an entry number in use, or one over `forge.client.op_bytes`
  (`room.md`, 6).
- **Each row is a kind of its own** (jig's `domain/connectors.md`,
  3.1): a policy that grants writing issues grants each kind it means.
- **A write in flight is a row of its own,** `Flight { task, entry }`
  (`../../protocol/store.md`, section 2): saved in the decision that
  commits an entry's attempt; kept while the write is unresolved, an
  uncertain outcome, a retry, a merge in the forge's queue (`queues.md`, 1) or the
  restart's settlement (section 2); erased in the decision that
  settles the entry for good or holds it for a person, and saved again
  before a held entry is made again. So the store shows which writes a
  stop left unsettled without decoding an entry. temper's owner set
  names its range (jig's `protocol/store/opening.md`, 3). The registered
  family converts at read; an in-flight write is not an upgrade
  refusal. The forge's restore links a `Flight` row to its entry, and one whose entry
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
  recorded (2026-11-15: never by its head, since a change remade at a
  cancelled one's head would take that one's closed pull request).
  Other closed or merged pull requests are Missing to the change; their
  numbers are never recorded. The change can then produce and open a
  new pull request on the reused names. It records the number of the
  one it opens from the opening's answer, and the one it takes open from
  its read, in the step that settles each; from then on its reads use
  that number rather than the branches, so an older pull request cannot
  replace its own, and it opens no other (`change/README.md`, 2, step 2). Its own closed pull request is drift and is
  reopened on release (`change/drift.md`, 1). A branch is created
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
  the change knows its number, as above. An entry's start,
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
  again once the forge connects (`adoption-connection.md`, 1). A forge away is none either: an
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
  next start settles as its class says (section 2).

## 2. Restart

At a restart (jig's `domain/engine/restart.md`; jig's
`domain/connectors.md`, section 14) the connector is given its records
from the store: the projects' repositories and what temper may do there,
the live tasks' resources and holds, the kept tasks' names, the changes'
and issues' states, and its outbox's entries with their attempts. The
deployment's id is given to the client first, as its namespace, before
any restored entry or live resource; the client restores each forge's
writer from its own record before any entry too (`adoption-connection.md`, 1). The client's
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
(`adoption-connection.md`, 1); the adopted repositories of each managed or owned Forgejo are
named to the routing for its reconcile (`adoption.md`, 2). Each `Names` row of a task
jig holds live is that task's names, and every other row a kept task's
(`adoption-connection.md`, 1), whatever forges it names; a start with more live rows than
`forge.tasks`, or more kept ones than `forge.repositories`, is refused
as any record past the limits (`../root.md`, 8.1). Until a forge connects,
its reads and entries answer `NotConnected`, which narrows nothing
(`adoption-connection.md`, 1), so the steps below run on each forge as it connects. Before its
records, the connector loads its kinds' values from its family
`Setting`, is given the stock defaults and says which differ (`change/gates.md`, 3), so a
restored change is judged by the rules as they are. The engine opens
once these records are in and its runs adopted; the forge then catches
up while it serves (`catching-up.md`): steps 1 and 2 run together from
the opening, and each decision waits as `catching-up.md`, 4, says.
temper's steps:

1. it reads afresh each repository's landing branches' tips and each
   live pull request (head, state, mergeability, CI, its place in a
   forge's merge queue), and a change decides nothing on a fact of
   these before its own read of it;
2. it settles its outbox, each entry as its class says (section 1): a
   pull request found by its branches, a merge by its pull request's
   state (merged, in the forge's merge queue, or neither), a creation by
   its key after the entry's start, a goal's own branch by its name, a
   set written again; one not found waits for the retry deadline its
   last attempt committed, and is then sent again, its conditions
   checked as at any attempt, unless it is unrecoverable, which holds
   its task for a person; a goal's own branch is made again (`branches.md`, 1);
3. it steps every change once, and every goal's issue. A change reads
   its recorded pull request by number. Without a number, it reads by
   branches and accepts an open pull request alone. This recovers an
   uncertain open whose reply was lost. Closed or merged history is
   Missing and never supplies the change's pull number: one of its own
   merged before the restart with its number unrecorded is found by its
   head, which the landing branch then holds (`change/README.md`, 2).

Its listings start from the newest time the store kept for each
repository with an issue or pull watch, inclusive, so what changed while
the engine was down is read once.

**What steps 1 and 2 wait for** (`../../draft/reconciling-start.md`, 5).
The forge answers the root, at any time during its restart, what each
step still owes, by kind, each with its count and its oldest three
named, in the words and order of `../../protocol/README.md`, 6.3:

- **reading afresh** owes each term its end waits on: the client's
  fresh reads still owed, by resource and listing (skein's
  `forge/client.md`, 8, *What it still owes*), the landings' gathers,
  the CI reads, the goal files' reads and the lost writers' reads still
  pending, and the goal branches still being made (`branches.md`, 1);
- **settling the outbox** owes each restored entry not yet looked for,
  by its number and task.

The root reports them past the report bound as its step's waiters
(`../root.md`, 8.1). Nothing in them is a deadline.

**A stop during steps 1 and 2.** Told to settle while it reads afresh
or settles its outbox (`../root.md`, 8.2), the forge stops waiting for
either: it settles as at any stop (section 1), its client sending
nothing new (skein's `forge/client.md`, 7.8). A read or a find
answered after is taken as a fact and steps nothing, and the end of
either step is no longer awaited, its report, should it come, dropped.
What either owed is the next start's to read and find again.

## 3. Open questions

- **Two Forgejo behaviours no fact records yet** (`../../protocol/forge/README.md`,
  section 12): that CI after a clean update runs on exactly the head the
  update made (`updates.md`, section 1); and where a merge lands when its pull
  request is retargeted after the fresh read before it, since Forgejo's
  merge is conditional on its head alone (`effects.md`, section 1).
- **A keyed creation lost across a restart** (`effects.md`, section 1, 2): a review, a
  comment or an issue whose answer a restart lost, and that its find does not
  see by its deadline, holds its task for a person, though a find that read
  everything since its start makes a second one unlikely. Default: so; only a
  goal's own branch is made again.
