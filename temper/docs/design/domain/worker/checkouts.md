# Checkouts

The checkout child domain is the worker's workspace capability (jig's
`domain/hosts/README.md`, section 6): it owns the workspaces on the worker's
disk. Its vocabulary is typed git and file operations (make a workspace,
make a store, clone, fetch, point a reference, check out, merge, commit,
push, pack), which the protocol layer
runs as contained processes and whose output it parses.

Preparing a checkout is in [`checkouts-prepare.md`](checkouts-prepare.md); pushing from one, saving included, is in [`checkouts-push.md`](checkouts-push.md).

## 1. A cache, keyed by workstream

- **A cache, keyed by workstream.** The runs of one piece of work (a
  change's producing, its repairs, its resolutions) share a key, so they
  find the repositories already cloned. The engine runs them one at a
  time; a worker given a second while the first holds the checkout fails
  its prepare as transient. A cached checkout is reused only if it holds
  exactly the repositories named, by directory and remote, and is reset
  to the assignment's starting point: nothing local is authoritative.
  One that a killed or broken git may have damaged is not trusted, and is
  rebuilt from an empty directory, as a new one is. The cache holds a
  fixed number of checkouts; once they are all made, a new workstream
  takes the least recently used one that no run holds.

## 2. One object store per repository

- **One object store per repository.** A checkout keeps no history of its
  own. Each repository the worker's prepares name, by its remote, has one
  bare store on the worker's disk, outside every workspace, whose objects
  every checkout of it borrows through an alternates file
  (`../../protocol/git-operations.md`, 1.2 and 1.11). Every fetch from the forge goes
  into the store, and then points the checkout's reference at what it
  fetched (`../../protocol/git-operations.md`, 1.3); a clone asks the forge for nothing.
  So a checkout made from an empty directory (a workstream's first on the
  worker, an evicted one, a saved start, a damaged one) transfers only
  what no earlier fetch on the worker brought, and never a branch the
  assignment does not name, but for the store's making and the file
  contents it checks out. A store is blob-less: its making and every fetch
  into it bring the history's commits and trees whole and no file
  contents (git's `--filter=blob:none`), but main's counts' one file
  (*Main's counts*), and a check out or a merge fetches
  the contents it needs from the forge into its checkout, which keeps them
  (`../../protocol/git-operations.md`, 1.5, 1.6 and 1.11). A prepare makes
  each store its repositories need that the cache has not made yet, before
  its workspace, as a bare clone of the forge's default branch alone, the
  one way git takes the remote's object format, SHA-1 or SHA-256 (a fetch
  refuses a remote of another format, and a checkout borrowing a store
  takes the store's). One prepare makes a store: another that needs it
  meanwhile waits, with no operation of its own, and goes on with it once
  it is made, and a making that fails or is cancelled passes to the one
  prepare that has waited longest, not to every one. Stores outlive a
  restart: a start keeps them, and the making of a store already there
  clones nothing (`../../protocol/git-operations.md`, 1.11), so the first prepares
  after a restart fetch only what each store lacks, while the workspaces
  still start empty. A store's token is its remote's, the first eight
  bytes of the SHA-256 digest of the remote's bytes read big-endian, so a
  restart finds each remote's store under the name it had, whatever order
  prepares name remotes in. The cache keeps at most
  `checkout.workspaces` times `checkout.repositories` stores
  (`../../protocol/configuration/limits.md`), the most its checkouts can
  borrow at once; a remote that finds them all taken takes the slot of
  one no workspace borrows, whose store is set aside. A store is never
  made again for a fault: git writes each object whole, under its name, so
  a git killed in a store leaves at most the references it was writing,
  which are its workspace's alone and set aside when that workspace is
  made again. A checkout's git directories are never another
  workstream's, and no fetch of history runs in either: only a check
  out's or a merge's fetch of file contents, into the worker's.

## 3. Two git directories

- **Two git directories** (`../../draft/confinement.md`, 6.3). Each
  repository of a checkout has the worker's own git directory, outside
  the workspace (`../../protocol/git.md`, section 2), and the run's, the
  `.git` its work tree holds.
  - **The worker's** is named, with the work tree, by every git operation
    of the worker's, which never finds either from where it runs and
    reads the work tree as files (`../../protocol/git.md`, section 3). Its
    configuration, hooks and references are the worker's alone, so
    nothing a run writes in its `.git` reaches the worker's git: not a
    filter or merge driver, a hook path or an alias, not a `.git` file
    naming another directory, not a moved reference.
  - **The run's** is the run's to write: its index, references and
    configuration. Its objects are borrowed, read-only, from the worker's
    directory's and, through them, the store's. The worker never reads
    it, but for the one file a check writes for it (`../forge/reds.md`,
    5.5), read as bytes; it writes into it, after each operation that
    moves its own, a view of its HEAD, index, merge, pins and main's
    counts, so a run's
    git, its checks among them, works as with one directory: clean at the
    start, a merge in progress shown, `refs/temper/landing` the pin
    (`../../protocol/git-operations.md`, 1.13). A run that breaks its `.git` has it
    made again by the next such operation.
  - **A delivery commits the work tree's files** on the worker's HEAD and
    merge (*Push commits what the run checked*, below), so a commit or a
    merge a run made itself reaches the branch as its files alone.
  - **A checkout made with one directory**, before the worker kept two,
    is set aside at the worker's start with every workspace
    (`../../protocol/git.md`, section 2); where a deployment keeps its
    workspaces across a restart (`checkouts.status.md`), the first hold
    gives it a worker's directory made afresh from its store, and its
    `.git` becomes the run's, never read, nothing in it trusted, so
    nothing in it cleaned.
  - **What it does not close:** a run's command can still write the
    worker's directory or a store itself, as the worker's user, until
    confinement's fence keeps it out (`../../draft/confinement.md`, 5.2).

## 4. Kept packed

- **Kept packed.** Each fetch into a store, and each check out's or
  merge's fetch of contents into the worker's git directory of a checkout,
  leaves a pack, and the worker's git runs no maintenance of its own
  (`../../protocol/git.md`,
  section 3). So the cache counts, for each store, the fetches asked into
  it, and for each workspace's repository, the check outs and merges asked
  in it and the commits asked with a delivery's or a save's snapshot,
  whose check of a second parent fetches into its worker's git directory
  (`../../protocol/git-operations.md`, 1.7), each since the last `Pack` that named
  it, from 0 when it is made and at the worker's start. A repository's
  count is its worker's git directory's: it stays with the clone while
  the workspace is reused or kept, and starts at 0 again when the
  workspace is made again. A store is due at 25, and so is a
  repository: half the 50 packs past which a `Pack` repacks
  (`../../protocol/git-operations.md`, 1.14), so a place holds about 75 packs at
  most between two counts. A prepare, a run's or a job's, that would end
  ready asks first, in the spec's order, one `Pack` for each repository
  whose store or itself is due, naming the store only where it is due:
  so the stores and the worker's git directories are both kept packed.
  Its repository is its own workspace's, which no job uses while it
  prepares. Its store other workspaces' jobs may be reading, which git
  allows: a repack puts its new pack in place before it deletes the old
  ones, and a reader that misses a pack looks again and finds the new one
  (the user, 2026-11-19); the git component keeps any other repack or
  fetch out of the store meanwhile (`../../protocol/git-operations.md`, 1.14).
  Asking a `Pack` sets the counts of the places it names back to 0,
  whatever its end, so that a prepare in another workspace meanwhile
  does not name the same store again, and what is fetched while it runs
  counts toward the next.
  Nothing else asks a `Pack`: a push, a save, a catch-up and a verifying
  fetch run while the hold's job uses its checkout, though their fetches,
  check outs, merges and snapshots count. A `Pack`'s fault, a time-out or
  a cancel among them, fails nothing and spoils nothing, since git
  replaces packs whole: the prepare ends ready as it would have without
  it. An abort or a release cancels it as any operation, and the prepare
  ends aborted, its workspace as the `Pack` found it.

## 5. Repositories side by side

- **Repositories side by side.** A workspace holds one directory per
  repository, so code that refers to a sibling by path works, and the
  agent's write authority is the list of writable ones.

## 6. One operation at a time

- **One operation at a time.** A run holds its checkout from the moment
  its prepare is admitted, so a cancel aborts a prepare rather than
  waiting it out; one queued for a permit (below) or waiting for another's
  store has nothing in flight, and ends aborted at once. A checkout runs one operation at a time (a prepare, a
  push or a save), each a sequence of git operations with deadlines as
  data, one for the forge and one for the disk; the checkout arms no
  timers. A check out and a merge take the forge's deadline
  (`checkout.remote_timeout_ms`) and name their repository's identity, since
  each may fetch the file contents it lacks: such a lazy fetch runs within
  its operation, under that deadline and with that identity's credential.

## 7. Preparations a few at a time

- **Preparations a few at a time.** At most `checkout.preparations`
  prepares are active at once on a worker, 1 in both profiles
  (`../../protocol/configuration/limits.md`), a run's and a job's alike,
  since a job prepares through the checkout as a run does (`jobs.md`).
  Each active prepare holds a permit; there are `checkout.preparations`
  of them, and a limit of 0 refuses the worker's start, naming it.
  - **Queued in order of arrival.** A prepare that passes the entrance
    holds its workspace and borrows its stores as before, so a refusal
    (every workspace held, the workstream's held by another run) comes at
    once, never after a wait. With every permit taken it is queued, with
    no operation in flight, and told as queued (`README.md`, section 6).
    Each permit that comes back goes to the prepare queued longest.
  - **Its deadlines count from its admission.** Admitted, a prepare takes
    its permit and asks its first operation, as one that found a permit
    free does at once; each operation's deadline counts from when it is
    asked, so no time a prepare spent queued counts against any of them.
  - **The permit comes back when the prepare has settled:** once it ends
    ready, failed or aborted. A prepare waiting for another's store keeps
    its permit while it waits. A cancelled prepare ends only once its
    operation in flight has ended, whatever its end, so no git it started
    still runs when the next prepare is admitted. A queued prepare asked
    to abort or to release ends aborted at once and leaves the queue,
    taking no permit.
  - **Only preparing waits.** A push or a save takes no permit, so a
    run's delivery never waits behind another run's preparation. Run
    slots are as they were (`agent-processes.md`, section 1): an admitted run keeps
    its slot while its preparation is queued.
  - **So a forge meets a few clones at once,** one per worker by default,
    after a worker's restart or a burst of due work alike
    (`../../draft/redeploys.md`, 6.4).

## 8. Prepare

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 1.

## 9. The landing pin

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 2.

## 10. The replaced head's pin

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 3.

## 11. A named pin

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 4.

## 12. Main's counts

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 5.

## 13. A goal's reds for each check

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 6.

## 14. An existing producer branch

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 7.

## 15. A base with heads merged in

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 8.

## 16. A merge in progress

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 9.

## 17. A goal's later run

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 10.

## 18. Push commits what the run checked

Moved to [`checkouts-push.md`](checkouts-push.md), section 1.

## 19. A design delivery

Moved to [`checkouts-push.md`](checkouts-push.md), section 2.

## 20. A save pushes unchecked

Moved to [`checkouts-push.md`](checkouts-push.md), section 3.

## 21. A landed dependency's catch-up

Moved to [`checkouts-push.md`](checkouts-push.md), section 4.

## 22. Each repository lands on its own

Moved to [`checkouts-push.md`](checkouts-push.md), section 5.

## 23. A task's own delivery outlives its hold

Moved to [`checkouts-push.md`](checkouts-push.md), section 6.

## 24. Pushes are never cancelled

Moved to [`checkouts-push.md`](checkouts-push.md), section 7.

## 25. Saved work

Moved to [`checkouts-push.md`](checkouts-push.md), section 8.

## 26. A workspace kept for its run

- **A workspace kept for its run.** A run a drain or a stop cut, whose
  answer is `Cancelled(Shutdown)` and whose save landed, leaves its
  workspace on the worker for that run's next attempt alone (temper's
  `draft/worker-restarts.md`, section 4): its trees, their ignored files
  and build outputs, each repository's `.git` with the checks' results
  directory in it, and the worker's git directories beside them.
  - **Marked when its save lands:** a mark beside the workspaces,
    `kept/<token>`, names its workstream, the commit the save made for
    each repository, and when it was kept. A save that lands nothing new
    names the commit the run started from. A run with no save, or whose
    save failed, leaves no mark, and its workspace is idle as any.
  - **Kept across the worker's restart:** the start's layout keeps each
    workspace and git directory a mark names, and moves every other to
    the trash, as before; a mark whose workspace is missing goes. The
    cache starts holding each kept workspace, idle, under its workstream's
    key, oldest first.
  - **Taken up by a saved start of its workstream** whose fetched saved
    commit is, for every repository, the one its mark names: the hold
    keeps the trees as they are, with no make, clone or checkout, its
    start and head that commit and its pushed baseline fetched as a saved
    start's is. Any other start of that workstream, or a saved commit
    that differs, makes the workspace again from empty, as a saved start
    does. The mark goes when the workspace is held again, either way: a
    run cut again leaves a new one.
  - **Gone when its run ends or the room is needed.** Once its run's
    next attempt ends on the worker, the workspace is released as any.
    A kept workspace is idle, so the cache's eviction takes it, oldest
    first, when a hold finds no room; another workstream never gets it
    as it is, since an eviction makes the workspace again from empty
    (redeploys' concern, temper's `draft/redeploys.md`, 6.3, so never
    arises).
  - **The checks' results directory** of each writable repository is
    `.git/smith-check` in its tree, which the worker names to smith in
    the start (smith's `domain/run-delivery.md`, 1.1): kept with the tree, and
    made again with it.
  - **Marked while its run holds it,** so that a crash takes the same
    path (temper's `draft/worker-restarts.md`, section 7). A hold with a
    saved-work destination writes its workspace's mark, `held`, once its
    prepare has succeeded and before its run starts: its workstream, and
    for each repository its head, the commit its next save takes as
    parent (its start, or its last save or delivery that landed), the
    merge it holds, if any, and whether its next save commits though
    unchanged (`unchecked`). It writes the mark again whenever one of
    these changes, and removes it when the hold is released. The drain's
    keep replaces it with the `kept` mark above. A worker that stops without
    a drain, the one-process worker (`README.md`, "In one process"), leaves
    its held marks as a crash does, and they are taken up at its next start.
    A hold with no saved-work destination writes none.
  - **Found at the next start.** The start's layout keeps each workspace
    a `held` mark names as it keeps a `kept` one, a crash's leftover,
    and removes from each kept workspace's repositories the lock files
    a killed git leaves (`index.lock`, `HEAD.lock`, and any name ending
    `.lock` under `refs/`), in its `.git` and its worker's git
    directory, since no git of the previous process lives. The start also
    removes from every shared store, whether or not a mark names a workspace,
    any name ending `.lock` under its `refs/`, since a fetch writes the run's
    references there and a killed one leaves its lock before any mark is
    written. The cache holds it, idle, under its workstream's key, so the
    hello names that workstream and the engine sends the run's next attempt back at once
    (jig's `domain/engine/executors.md`, 6.5).
  - **Saved at its take-up,** as the drain would have saved it. The
    next start of its workstream on this worker whose repositories are
    the mark's, by name and remote, first saves the trees, before its
    run's agent starts: each writable repository's tree committed on the
    mark's head, with the merge it names and refusing conflict markers,
    then pushed to this assignment's saved-work destination with this
    assignment's identity. Its save landed, or with nothing new to save,
    the hold keeps the trees as a kept workspace's take-up does, its
    start and head that save's commit (or the mark's head), and goes on
    as a saved start of it; the run's answer names that save as its
    saved work. A save that fails, a mark whose repositories differ, or
    any other start makes the workspace again from empty, and the crash
    costs what its last save left out, as before. The save waits for an
    assignment because the worker holds no identity until one grants it
    (`../../protocol/credentials-git-identities.md`, section 1).
  - **The mark's form,** `kept/<token>`, is `temper-kept 2`: version 1's
    fields, then its state, `kept` or `held`, and each repository's merge
    and `unchecked`. A version 1 mark reads as `kept`, with no merge and
    `unchecked` false; an older build meeting a version 2 mark removes
    it and moves its workspace to the trash.

## 27. No credentials on disk

- **No credentials on disk.** Every git operation that reaches the forge,
  or commits, names the repository's identity, which the protocol layer
  maps to credentials or an author for that invocation only, never into a
  checkout's configuration, where an agent could read them.

## 28. Merge commits' titles

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 11.

## 29. A failed prepare says where and why

Moved to [`checkouts-prepare.md`](checkouts-prepare.md), section 12.

## 30. Open questions

Pushing's open questions are in [`checkouts-push.md`](checkouts-push.md), section 9.

- **Code graph:** indexing a checkout for codebase-memory-mcp as part of
  preparing it, and keeping that index with the cache.
- **Which queued preparation goes first.** Queued prepares are admitted in
  order of arrival, a job's and a run's alike, a resumed run's no sooner
  than a new one's (`checkouts.md`). Default: arrival; an order by kind, a
  landing's jobs first, if a deployment shows them held behind runs'
  preparations.
- **A workspace's repositories in parallel:** a checkout runs one
  operation at a time, so its repositories are cloned, fetched and pushed
  one after another.
- **A store's upkeep.** Nothing prunes a store, and one whose remote no
  prepare names since the start stays on disk, since the cache sets aside
  only the stores it knows. Default: none, and an operator who wants the
  space removes `stores` with the worker stopped.
- **A store damaged past its references.** A corrupt pack would break
  every prepare of its repository until its store is removed by hand,
  with the worker stopped, since stores outlive a restart: nothing seen
  suggests one, and making a store again on any broken operation would
  cost a whole fetch each time. Default: a store is never made again for a
  fault (`checkouts.md`).
- **Two remotes of one token.** A store's token is 64 bits of its remote's
  digest (`checkouts.md`), so two remotes could share one; among the few
  remotes a worker serves that is not expected, and nothing checks it.
  Sharing would mix their objects in one store, which every fetch still
  names by its own remote. Default: unchecked.
- **File contents fetched once per checkout.** A check out's or a merge's
  lazy fetch writes into its checkout's worker's directory, not the store, so two
  checkouts of one repository each fetch the contents they check out, and a
  checkout made again from an empty directory fetches them again. Measured
  on one deployment's forge, a blob-less store's clone took 2.3 s and 50 MB
  where a full one took 17.4 s and 198 MB (2026-11-12). Default: so; a
  store that keeps the contents its checkouts fetched, if a deployment
  shows the repeated fetches. A store made full before stores were
  blob-less stays whole, its later fetches blob-less.
- **A workspace kept past its workstream.** The cache is keyed by workstream,
  so a new one starts from an empty directory and every file is new to a
  build, though its history is the store's. Keyed by repository, it could
  keep a working tree across workstreams and a
  restart once confinement keeps a run's commands out of the worker's git
  directory (`../../protocol/git-later.md`, section 1), perhaps with no switch. With that goes
  which workspace a workstream takes: the departure in `checkouts.status.md`
  considers only the least recently used, and after a restart only the one a
  first hold takes, since nothing says which workstream held which. So a
  clone another workstream wanted can be made again, wasteful and never wrong.
