# Branches

- **Named after the tree.** Every branch temper makes is under the
  repository's prefix (`temper/` by default, one deployment's on a
  repository: `adoption.md`) and its tree's root task:
  - a change's own branch: `temper/<root>/c<task>`, pushed by its
    producer, or made by temper at the head a change a person opened
    takes from the branch they pushed (`change/opened.md`, 4);
  - a task's saved work: `<prefix><root>/s<task>` in each writable
    repository, using that repository's own prefix (`temper/` by
    default). The task is the running agent's, even when its workstream
    and delivery branch belong to its parent change. Saved work has its
    own branch, so it starts no CI and moves no pull request;
  - a landing's branches, where its prepare job pushes each round's
    prefixes, one per member: `temper/<root>/l<task>`, the landing's
    task, for its first member and `temper/<root>/l<task>-<n>` for its
    member n, each held by the landing from its making, deleted as a
    round that changed its prefix ends unlanded and as the landing
    closes (`../landing-prepare.md`, 1.1);
  - a goal's own branch: `temper/<root>/g<task>`, the goal's number, in
    its project's home, held by the goal (1);
  - an agent's task's own branch, for runs that push outside a change,
    as a chat making a small fix does:
    `temper/<root>/r<task>`, in each repository its runs may write, the
    project's home among them, which the task holds from the commit that
    makes it. A chat hands it to the change task that lands it. A goal's
    runs push to its goal branch instead (3): a goal holding a run
    branch has nothing push it, and its close deletes it as below.

  A task made before its project's home was adopted holds no run branch
  there: its runs read the home without pushing, and its brief says why.
  A new task gets the branch.

  A later attempt of the same task reclaims its own writer slot, whether
  running or lost; its own earlier writer never holds it. An earlier
  attempt's reread cannot free the later writer, report drift for it or
  hold the later run. A failed reread holds the task as a resource
  unavailable, naming the task, run, branch, repository and read error.
  Another holder's contention names the resource and holder in a resource
  wait hold. Neither is a procedure hold: an agent task has no procedure.

  They are siblings under `temper/<root>/`, so no name is the prefix of
  another as git refs forbid, and one grant covers a tree's branches;
  a task's own branches are named in its batch symbolically and resolved
  once it has its number (jig's `domain/authority.md`, section 4).
- **Held by the task it is named after,** a change's by the change that
  lands it, a goal's own by its goal, and written by its delegates one
  at a time (jig's `domain/tasks/holds.md`, 3); a goal's own is written by
  its own runs too, beside them (3).
- **Landing branches are never made by a push.** A landing branch must
  exist when it is adopted or named; a goal's own branch (1) is made
  at its base's tip as its goal's hold is taken, as the taking's effect
  (jig's `domain/connectors-effects.md`, 1.7), and a change into it reads it
  once it is made.
- **Deleted as their tasks close,** as the release of their holds (jig's
  `domain/tasks/holds.md`, 5): a landed change's branch, a cancelled one's,
  and a task's saved work. A failed change's branch is kept for a person
  to read, and deleted as its tree's root closes. A goal's own branch is
  deleted as its goal closes if a change landed it (its tip reachable
  from that change's landing branch), and otherwise kept for a person to
  read, until deleted by hand or by the deployment's retention. A change
  on its requester's held branch (`change/README.md`, 1) holds no branch and deletes none
  as it closes: the goal's landing leaves its goal's branch to the
  goal's own close. A goal's run branch goes as its goal closes. A
  release waits for a lost writer's reread, and fails only if that
  reread fails. A branch temper knows no head of once that reread is done (a change's branch before its first push, a run branch nothing pushed) is not deleted: its hold is dropped with no write, as a goal's unused run branch is.

A goal's release recognises its branch as landed from a change in the same
repository on that branch, in landed, red, cleared or uncleared state, whose
landed head equals the branch's last known tip. With no known tip or no
matching landed change it keeps the branch; a subsequent unlanded move keeps
it too. This uses the retained change and branch records, with no new
ancestry read (2026-11-04).

A task's saved work is named by the set its task's record keeps of the
repositories a next attempt would resume from (`../worker/checkouts.md`). As
its task's release begins, each branch of that set becomes a hold of the
closing task, with no writer, and goes as any hold does: deleted, a branch
already gone counting as deleted (skein's `forge/client-effects.md`, section 1);
or, if its task failed and is not its tree's root, kept for a person to
read and deleted as the root closes. Such a hold has no holding in the
hub, and no head is read for it. A branch outside that set stays until
deleted by hand or by the deployment's retention, as does a save whose
terminal the engine never heard, its worker lost, and a branch the forge's
full holds left no room for at the close.

## 1. A branch per goal

Whether a goal's changes land into a branch of its own, which then lands
whole, or each into the shared base, is the plan's choice like any
other: a change's landing branch is its parameter. Landing each into the
base is the usual answer: plans meet early and in small pieces; a goal
that needs another's work has it once that lands; and repositories that
pin each other's commits (temper locks skein's in `Cargo.lock`) pin a
commit of the base, which no later squash of a goal's branch replaces.
A goal's own branch suits work likely to be dropped; a goal
checked whole before it lands (by an end-to-end run, by a review of the
whole); intermediate states that would break the base; and upstream
contributions (`README.md`, section 8). Such a branch is short-lived, and the goal's
own, never a change's:

- **Named by the goal's template,** or by the batch that makes the goal
  by hand, as the symbol *its own goal branch* (`README.md`, section 2), offered on a
  tracked task only, since its goal is the one its changes serve (jig's
  `domain/model.md`, 6.2). Resolved once the goal has its number to
  `temper/<root>/g<task>` in the project's home (`branches.md`).
  `<root>` is its tree's root, as for every branch of the tree: a goal
  made beneath a milestone, or beneath any task, takes that task's root,
  so its branch sits under the milestone's root beside its changes'
  branches, where the change tool, the goal's closer and a held making
  each look it up. A goal made beneath a task before its holdings were
  asked under its tree's root holds its branch under its own number,
  `temper/<task>/g<task>`; no lookup finds it, so its changes land into the
  base and its branch is kept as it closes, for a person to read. A goal
  cannot take a hold once active (jig's `domain/tasks/holds.md`, 2), so the
  choice is made with the goal.
- **Held by its goal for its whole life,** an exclusive hold that
  refuses, and made at the tip of the home's landing branch, the one its
  adoption names (`adoption.md`, 12), as the hold is taken, as the taking's effect. A making that fails for good holds the
  goal, its words naming the branch and what the forge answered.
  A making waits for the outbox's room and never fails for it: with the
  outbox full (`forge.entries`), it holds nothing and polls nothing, as a
  read answered `Busy` waits for the client's room. Its reads, the
  branch's and the landing branch's tip, wait for the client's room in
  the read line, in arrival order beside the connector's other reads
  waiting there, each on a deadline and retried at it for as long as it
  waits, the log saying when it begins to wait and when it is admitted
  (`keeping-up.md`, 3, *Reads waiting for room take turns*).
  It gives up its entry number while it waits, so it reserves nothing:
  a kept number would stop every later entry's handoff, and an outbox
  full of those would never free. It waits in the outbox's line beside
  the changes, landings and releases that wait (`room.md`, 3), its place
  kept in memory with the making. Once an entry frees, the waiters go on
  in the order of their places, each making taking a fresh number
  and reading the landing branch's tip afresh, as a restart's making
  with no entry does; fresh numbers follow that order, so the client's
  entries stay in increasing order. A making that meets a full outbox
  again waits again, keeping its place.
  What no freed entry mends holds the goal at once, each cause in its
  own words after the branch's `could not be made: `: a repository not
  adopted, `the repository is not adopted`; one adopted to be read,
  `the repository is adopted to be read, not written`; one whose
  adoption takes no branch writes, `the repository's adoption takes no
  branch writes` (`adoption.md`, 6); and an entry number already in the
  outbox, `its outbox entry N is already in use`, N that number. Access
  is judged first, so a full outbox never hides a making that could
  never be sent.
- **Made whatever restarts.** temper alone makes and deletes a goal's
  own branch, so its making is conditional (`effects.md`, section 1): while the goal
  holds the branch and none of its name exists, it is made, and never
  reset once it exists. A restart's making is two steps, a read and a
  write, and only the read belongs to the forge's fresh reads as it
  catches up after the opening (`catching-up.md`, 2):
  - **Its read** is one of the fresh reads (`effects.md`, section 2)
    while it is out: is the branch there, at which commit, and is a
    creation entry of its own in the outbox. There, it is the goal's;
    missing, with its making's entry restored, that entry settles as its
    class says, its find answering not found sending it again at its
    commit. A read the client refuses for room, or one waiting out its
    forge's absence or its connection (`keeping-up.md`, 5), is no fresh
    read until it is out again: it waits in the read line and is asked
    in its turn, as any making's read waiting for room is
    (`keeping-up.md`, 3), and the start's fresh reads end without it.
    The branch's own readers wait for its making whatever the fresh
    reads say: a change into it, its design writers, its closer and a
    release that judges it decide nothing on its head while its making
    is pending (`catching-up.md`, 4). Counted as a fresh read, a read
    waiting for room would hold its repository's fresh reads, and every
    procedure on the goal's home with them, for a call that others keep
    taking; the start goes on and the making is asked after.
  - **Its write** waits in the outbox's line. Missing, with no entry
    (the restart cut its making before it committed one), the making
    takes a place in the line, behind the places restored with
    their records (`room.md`, 4), and asks nothing more while it
    reads: no entry number, no landing branch's tip. The line asks it
    in its place's order as room frees from the beginning of catching
    up (`catching-up.md`, 3), as any making waiting for room: it takes
    a fresh entry number from the root, reads the landing branch's tip
    afresh and makes it there, its entry sent behind its lane.

  So reading afresh ends on reads alone, whatever room the outbox has:
  a full outbox delays a making, never the start. A change into the
  branch waits for it as for any making, within the change's stall. A
  forge away while it is made is ridden (`keeping-up.md`, 5), never a failure. A goal
  held for its making that fails for good is made again when a person
  releases that hold: the release reads the branch, as a restart does,
  and, still missing, its making waits in the line the same way.
- **Written by change, and by the goal's design:** each of the goal's
  changes lands into it as its landing branch, and the goal's own runs
  push their design and status documents to it (3); code reaches it
  by change alone. An ordinary `change` call from the goal or its
  subtree defaults to the nearest applicable held goal branch; an explicit
  destination outside it refuses the whole call before delegation. This
  also holds for every member of a batch. The deliberate `"branch": "mine"`
  change lands that held branch whole into the base instead (`change/README.md`, 1).
  Each child merge is ordered by the branch's queue (`queues.md`
  ) and takes the branch's writer slot while in flight; the goal's own
  runs take no writer slot of it (jig's `domain/tasks/holds.md`, 3; 3),
  so merges into it go on while they run.
  A change whose landing branch is a goal's own, made by a task that is
  neither that goal nor one of its subtree, is refused at its batch,
  naming the goal.
- **What a goal writes goes straight to g,** where its charter's
  `modify` is set (`../agent/tools.md`): its runs are g's design writers
  (3), and `deliver` pushes their design to g, at any point in a run
  and as often as it likes, with no change to land it. A charter without
  `modify` leaves its goal's run to inspect its confirmed branch, as it
  starts there.
- **Landed whole** by a change into the base whose branch is the goal's
  own: made by the goal's closer (`branches-closer.md`, 1) where the goal is ended by one,
  and otherwise by the goal on its held branch (`change/README.md`, 1); and deleted as the
  goal closes once that change has landed it (`branches.md`); otherwise
  kept as the goal closes. A goal takes the base only by a catch-up
  (`updates.md`, section 1) for one of its changes that needs what the
  base gained since its branch was made, or by its closer's for a close
  found outdated (`branches-closer.md`, 1, *A close found outdated*), never to stay current
  (`../../draft/squash-in.md`, 4): its landing takes the base at the
  front of the base's queue, as any change does (`change/README.md`,
  2), and a conflict goes to the merge job and a resolution
  (`change/repairs.md`). A landing into the base that touches its open
  work reaches it as news (`topics.md`, 1.3), for it to judge whether a
  change needs it. A project's rule may require or forbid a catch-up.

## 2. A goal's closer

Moved to [`branches-closer.md`](branches-closer.md), section 1.

## 3. A goal's design writers

A goal's design goes straight to its goal branch. A run of the goal
whose charter sets `modify`, its goal branch made, confirmed and
undrifted, is a **design writer** of it: its workspace starts at g's tip
(or its saved work, or, where its run branch holds a head g lacks, g's
tip with that head merged in) and names g as both its push branch and
its landing branch, which the worker takes as a design delivery
(`../worker/checkouts.md`): Markdown only, g's moved tip merged first, a
fast-forward never forced. Its repository's other writes are as before.

- **Its first run among them.** A goal's run prepared while its goal
  branch is still being made (the taking's effect in flight, jig's
  `domain/connectors-effects.md`, 1.7) or not yet confirmed (its head not yet
  recorded as temper's own) waits, as a write into a branch still being
  made does, and starts once g is made and confirmed, at g's tip, a
  design writer like any later run. A modifying goal holding g never
  starts on its base or with its workspace's writes read-only, so its
  first run can deliver the first wave's design before it cuts its
  changes. A making that fails for good holds the goal (1); a drifted
  g holds it (`change/drift.md`, 1), and its run starts once a release settles it.
- **How it calls it:** smith's `deliver`, with a `title` and a `body`
  as a change's delivery has them, at any point in its run and as often
  as it likes; the project's checks run at it (`../agent/README.md`, section 5).
  It answers a delivery's receipt, `pushed <head> to branch <g>; your
  checkout stays detached, as expected`, or, where g had moved, `pushed
  <head> to branch <g>, merging its tip <tip> first; ...`; or a refusal
  the run settles and delivers again: a path that isn't Markdown, named,
  since code goes through `change`; files in conflict with g's moved
  tip, named, the merge left in its checkout; g kept moving. It is never
  stale, and a `save` there pushes nothing and is refused.
- **No writer slot.** A design writer takes no writer slot of g (jig's
  `domain/tasks/holds.md`, 3): g's hold keeps its design writers beside its
  slot, each a run's attempt, so merges into g, a landing's repairs and
  the goal's other writers go on while it runs. The hold keeps them with
  it in the store, so a restart keeps them.
- **Its pushes are admitted first.** Each push of a design writer, at
  each of its tries, is admitted into g's record of temper's writes
  before the worker makes it (`writes.md`, 2), so any read of g that
  shows it, the keep's or a change's, while the run lives or after it
  has answered, finds it explained, never drift (`writes.md`, 4); a head
  no entry explains is drift as before (`change/drift.md`, 1), and holds
  the goal where no change on g reads it. When the run answers, or its
  attempt is lost, its pushes still unsettled are settled as
  `writes.md`, 3 says. What the run reports pushing is not taken as the
  head, since merges land beside it.
- **In the order the forge took them** (2026-11-19). g's head, and with
  it the forge client's expected head (`Pushed`), follows temper's own
  writes in the order they reached the forge, never the order their
  answers reach the engine: the record orders them (`writes.md`, 4,
  *Ordered*), so a merge into g whose answer comes after a design push
  that already descends from its commit never moves g back to it, and a
  restart, which restores the record, orders them the same. Drift is
  unchanged: a head none of temper's writes made, even on top of g's
  head, and a move not forward from it, still hold the goal
  (`change/drift.md`, 1).
- **Changes on g take it.** A change on g (the goal's landing,
  `"branch": "mine"`, the closer's land step) treats a live design
  writer as its requester's own run (`change/drift.md`, 1): a head the record explains is its new
  head, checked again, never "branch moved".
- **Its brief says so**: one line, that `deliver` pushes the
  checkout's Markdown straight to the goal branch, merging its moved tip
  first, and that code goes through `change`.
- **The goal's own runs alone.** A designer the goal makes besides
  itself stays a change (`../../draft/goal-as-designer.md`, section 2): its
  producer delivers its change's branch, and its design reaches g by
  that change's pull request, as any change's does. A push to g from
  anywhere but a design writer holds as drift (`change/drift.md`, 1).

## 4. Open questions

- **Saved branches outside the task's saved set** (`README.md`, 11): a terminal the
  engine never heard, a repository no longer in the set, or full holds at
  close can leave a branch on the forge. Finding these needs a branch
  inventory beyond the set a next attempt resumes from. Default: manual
  deletion or deployment retention; a failed descendant's known saved
  work stays with its root until that root closes.
