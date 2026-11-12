# Checkouts

The checkout child domain is the worker's workspace capability (jig's
`domain/hosts/README.md`, section 6): it owns the workspaces on the worker's
disk. Its vocabulary is typed git and file operations (make a workspace,
make a store, clone, fetch, point a reference, check out, merge, commit,
push), which the protocol layer
runs as contained processes and whose output it parses.

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
- **One object store per repository.** A checkout keeps no history of its
  own. Each repository the worker's prepares name, by its remote, has one
  bare store on the worker's disk, outside every workspace, whose objects
  every checkout of it borrows through an alternates file
  (`../../protocol/git.md`, 4.2 and 4.11). Every fetch from the forge goes
  into the store, and then points the checkout's reference at what it
  fetched (`../../protocol/git.md`, 4.3); a clone asks the forge for nothing.
  So a checkout made from an empty directory (a workstream's first on the
  worker, an evicted one, a saved start, a damaged one) transfers only
  what no earlier fetch on the worker brought, and never a branch the
  assignment does not name, but for the store's making. A prepare makes
  each store its repositories need that the cache has not made yet, before
  its workspace, as a bare clone of the forge's default branch alone, the
  one way git takes the remote's object format, SHA-1 or SHA-256 (a fetch
  refuses a remote of another format, and a checkout borrowing a store
  takes the store's). One prepare makes a store: another that needs it
  meanwhile waits, with no operation of its own, and goes on with it once
  it is made, and a making that fails or is cancelled passes to the one
  prepare that has waited longest, not to every one. Stores outlive a
  restart: a start keeps them, and the making of a store already there
  clones nothing (`../../protocol/git.md`, 4.11), so the first prepares
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
  made again. A checkout's own `.git`, which its run may write, is still
  never another workstream's, and no fetch runs in it.
- **Repositories side by side.** A workspace holds one directory per
  repository, so code that refers to a sibling by path works, and the
  agent's write authority is the list of writable ones.
- **One operation at a time.** A run holds its checkout from the moment
  its prepare is admitted, so a cancel aborts a prepare rather than
  waiting it out; one waiting for another's store has nothing in flight,
  and ends aborted at once. A checkout runs one operation at a time (a prepare, a
  push or a save), each a sequence of git operations with deadlines as
  data, one for the forge and one for the disk; the checkout arms no
  timers.
- **Prepare** fetches what the assignment names and checks out exactly its
  starting point. It fetches each branch of a repository at most once: a
  pin or a baseline naming a branch this prepare has fetched, or a commit
  it has fetched, takes that fetch's commit, and a pin is pointed at it
  locally (`../../protocol/git.md`, 4.12). A branch or base that does not exist is a preparation
  failure, permanent until something changes on the forge: the worker
  creates no branch but by pushing a run's change, and branches temper
  needs before then are made by the engine, as effects (../forge/branches.md
  ).
- **The landing pin.** A repository whose item names a landing branch
  (`README.md`, 4.1) has it fetched once per hold, during prepare, after its start is
  fetched, on the same remote, into
  `refs/temper/landing` (`../../protocol/git.md`, 4.3), or, where it is the
  branch the start fetched, pointed at that fetch's commit: every start of a
  change's producer, a saved or resumed one's included, and a goal's later
  run. That tip is frozen for the hold: no later operation of the hold
  moves the reference, so the project's checks take the same base at every
  delivery and at finish (`tools/pre-pr/design.md`, section 2). A missing
  landing branch fails preparation. The next hold fetches it afresh, so
  the landing tip may move between attempts; a pin a cached checkout kept
  from an earlier hold is overwritten, never read. The pin changes neither
  the checkout start nor ownership, drift admission or the saved-work
  baseline.
- **The replaced head's pin.** A writable repository whose item names no
  landing branch has the head its push branch has at prepare fetched once
  per hold, on the same remote, and the same `refs/temper/landing` pointed
  at it: the head its delivery will replace, the one fetch serving its
  baseline, its pin and, where it is the start's branch, its start. For a repair, that is the change's head it
  repairs, whatever its start (its branch, or saved work); for a resolution
  or a merge job, the branch's head the base is merged into. Where the push
  branch does not exist yet (a chat's first run), its start's commit is
  pinned. So a repair's checks select what its own edits changed against the
  head it repairs, never against the branch the change lands on: a repair of
  a goal's landing into main checks its own edits, not the goal's
  (`tools/pre-pr/design.md`, section 2), and no pin a cached checkout kept
  from an earlier hold is read. The pin is frozen for the hold as a landing
  branch's is. A read-only repository delivers nothing and runs no check, and
  its reference is left as it is. The item still names no landing branch,
  so nothing else of it changes: it is never taken for a goal's design
  delivery (`../agent/README.md`, section 5), and its start, push and expected
  head are as before.
- **An existing producer branch** compares with the pinned tip: it is the
  frozen comparison for existing work below, only for an inherited
  producer that starts on its push branch.
- **A base with heads merged in** fetches the base branch and each head,
  checks out the base's tip, and merges each head into it in their
  order, committing each merge at once, with two parents (what was
  checked out so far and the head) and the repository's identity as its
  author, and checks that commit out before going on (`../../protocol/git.md`,
  4.7: a commit moves neither HEAD nor the merge's state), so that the next
  merge can run and the agent is handed a checkout on the last commit with no
  merge in progress; a head the checkout already contains is no merge. The last
  commit is the checkout's start, for its saves and its push alike, and
  the branch it pushes is made by its first push, as a base's is. A
  head that does not merge cleanly fails the preparation as a conflict,
  naming the head, permanent for this assignment: nothing of the
  merges is kept, and nothing is pushed. The change it was for then
  waits for its dependencies to land (`../forge/change/questions.md`, 1).
  This is the one conflict a prepare refuses, and only an early producer's
  start meets it; its detail says why, as
  `merge: Conflicted: head <head> does not merge into <branch>`, `<head>`
  the head's full id in lowercase hexadecimal, 40 digits for a SHA-1 id and
  64 for a SHA-256 one, as a merge's title spells it
  (`../../protocol/channel.md`, 4), and `<branch>` the start's branch.
- **A merge in progress** fetches the branch and the base commit, checks
  out the branch, and merges the base into it as git does, leaving the
  files in conflict with their markers and the merge's state in the
  repository. The agent edits the files and runs the checks; it has no
  git writes (smith's `domain/tools.md`, section 5). Its push commits the
  tree as the merge, with two parents, the branch's head and the base
  commit, even when the tree is unchanged from either; a file the merge
  left in conflict that still holds a conflict marker is refused at
  push, and the run is told which. More files in conflict than the
  checkout's `conflicts` still make a merge in progress: each stays
  marked, and the start names the first `conflicts` of them, in git's
  order (`../../protocol/git.md`, 4.6). A conflict this start meets is the
  run's to settle, never a failure of the prepare: a resolution's start,
  whether it merges the base into the change's branch or, with nothing of
  the change pushed, into the base branch, is always prepared so, its
  files named in its start. A producer's start, the one naming its
  landing branch (`../forge/change/questions.md`, 1), first asks whether its branch already
  contains the base: if so there is nothing to merge, and it goes on as a
  start from that branch or saved work does, with no merge state and no
  second parent. A resolution's or a merge job's names no landing branch and
  always merges. A producer's base that merges cleanly is committed at once,
  as a base with heads merged in is: two parents (the commit it merged into
  and the base), the repository's identity as its author, the title
  `Merge <base>`, and that commit checked out (`../../protocol/git.md`, 4.7). So
  its run starts on a clean tree with no merge state, its earlier work and the
  base's files both in its head, and nothing staged to misread or remove. The
  commit it merged into stays the checkout's start, so a save with nothing new
  saves the merge commit, and its first push carries the merge, even with
  nothing new. A producer's conflict stays in progress as above, and its start
  names the files in conflict (smith's `domain/run.md`, 8.3).
- **A goal's later run** (`../forge/branches.md`, 3) starts at g's tip; where its
  run branch holds a head g lacks, it starts as a producer does, on a merge
  in progress naming its landing branch: the branch is g, the commit merged
  into it is that head. A clean merge is committed at once, and a conflict
  stays in progress. It pushes to g, a design delivery (below).
- **Push commits what the run checked.** When a run asks to deliver, at
  `finish` with a change or mid-run, it has run the repository's checks
  with nothing else writing to the checkout (smith's `domain/run.md`,
  8.1); the worker commits exactly that tree and pushes the commit, with
  the run's title and body. Delivery reads the checkout's current HEAD and
  merge state: an in-progress merge records HEAD and MERGE_HEAD as its two
  parents, even when its resolution is already staged. A committed merge
  keeps its ancestry. When the delivered tree is HEAD's tree and no merge
  is in progress and HEAD is not this hold's unchecked saved commit, the
  worker delivers that commit unchanged, retaining the producer's message and identity. Edits beyond a local commit build on that
  commit. A second parent, in progress or at HEAD, must be reachable from a
  branch or tag on the delivery branch's remote; otherwise delivery refuses,
  saying that its second parent is not reachable from that remote, before
  staging or committing. A new delivery commit becomes the checkout's HEAD
  and clears its completed merge state (../../protocol/git.md, 4.7).
  A push is a fast-forward and never forced: if
  the branch moved incompatibly with the run's commit, the push fails and
  the run is told. When the assignment names an expected head, the
  remote must also still be at that exact head, so no movement of the
  branch since the run started goes unseen. Saved starts use the fetched
  delivery head when none was supplied; for other starts without one,
  a rewind of the branch to an ancestor goes unseen (`README.md`, section 10).
- **A design delivery.** A repository naming the same push and landing
  branch is a goal's design delivery (`../forge/branches.md`, 3). It runs
  the repository's `.temper/pre-pr` as any delivery does (smith's
  `domain/run.md`, 8.1); a failed check refuses it. The checked tree is
  committed, then the goal tip fetched. If the commit lacks that tip, it is
  merged: a clean merge is committed with two
  parents (delivery commit and tip), the repository's identity as author
  and title `Merge <branch>`; a conflict stays in progress, its files
  marked and named in the refusal, for the run to settle and deliver again,
  committing the merge. Every path differing between the tip and commit
  must end in `.md`; the first that does not, in git's order, refuses it
  naming the path, since code goes through a change. The scan streams every
  name, without the conflict count limiting changed files. The push is a
  fast-forward, never forced, leased on the fetched tip. Movement meanwhile
  is fetched and merged afresh, up to three tries; then refusal says the
  branch kept moving. It is never stale: refusals leave the run going.
  A `save` pushes nothing and is refused. Its delivery receipt says when it
  merged the tip first. Every git fault keeps its bounded stderr tail in
  the refusal, including an unclassified failure.
- **A save pushes unchecked.** A run's `save` (`../agent/tools.md` and `../agent/README.md`, 5
  ) is a push to the same branch, with no check run before it;
  new commits are titled as unchecked; its landing is this hold's last push as a
  delivery's is. The hold records that its last push was a save while its
  head is that commit: the next commit of the hold, a checked delivery's or
  the save of saved work, is made even when the tree is unchanged
  (`../../protocol/git.md`, 4.7, `always`), with that commit's own message,
  so a save's commit is never what the hold's next delivery or saved work
  names. An unchecked save uses the same checkout snapshot as a checked delivery:
  staged merges keep both parents, a matching local commit is kept, and a
  new commit advances HEAD. Only the checked delivery runs the checks. A
  checked delivery's landing clears the record.
- **Each repository lands on its own.** Pushing several is not atomic:
  the run is told done only if every repository with a change landed it,
  moved if a branch cannot take the commit as a fast-forward, and nothing
  to push if none had a change. A repository whose head this hold's last
  push already landed is that landing again, not unchanged: a `finish`
  after a mid-run delivery, with nothing new, delivers what the run
  delivered. On a new hold of an inherited producer that starts on its
  push branch, an unchanged head not contained in the fetched landing tip
  is existing unlanded work. The worker pushes that head again, requiring
  the remote to equal the assignment's expected head, or the fetched start
  when none was supplied. Only successful or verified push returns its
  landing. An equal or already-contained head is unchanged. Ordinary
  runs, read-only items, saved starts, repairs and resolutions do not use
  this comparison; saves still compare against the actual checkout start.
  A push the forge refused (permissions, a
  protected branch, a hook) is a landing of its own, which a retry will
  not change.
- **Pushes are never cancelled.** A push in flight runs to its end, which
  its deadline bounds, so one that lands is reported landed; one that
  ended ambiguously (out of time, broken, the forge lost on the way) is
  verified against the forge, by fetching its branch.
- **Saved work** is ordinary commits on the saved-work branch, pushed the
  same way. A merge start keeps its saved-work destination; saving commits
  that merge with both parents, refusing unresolved conflict markers,
  and uses its committed head after a delivery.
  A run that resumes from saved work starts where the last one
  stopped: prepare rebuilds the cached workspace from an empty directory,
  clones the repositories again and checks out the saved commit's tree,
  leaving no cached tracked edits or untracked files. The saved commit
  is the checkout's start and head; its pushed baseline is the fetched
  delivery branch's head, or absent if that branch does not yet exist
  and no expected head was required. An explicit expected head stays in
  force; otherwise the fetched delivery head guards the push. A saved
  tree is deliverable without another edit, even if its head is already
  on the delivery branch: this hold establishes its landing through a
  push against that baseline, which later finishes reuse. Saving still
  compares with the checkout's start. A fresh start resets and cleans
  the checkout.
  Work done since the last save is lost if the worker dies. A merge start
  whose branch is the repository's saved-work branch started from saved work:
  a later save of it that lands nothing (nothing new, markers left, a failed
  push) leaves that saved work standing, and the next run starts from it. A
  run's own edits are in the saved commit, so in the next run's head, and
  `git status` shows none of them. A run that parks with a conflict marker
  left keeps none of that run's edits (`README.md`, section 10); what was saved before it
  stands.
- **No credentials on disk.** Every git operation that reaches the forge,
  or commits, names the repository's identity, which the protocol layer
  maps to credentials or an author for that invocation only, never into a
  checkout's configuration, where an agent could read them.

Each commit made while preparing a base with heads merged in has the title
`Merge <head>`, where `<head>` is the merged head's full hexadecimal object
ID (2026-11-04).

## 1. Open questions

- **A goal's saved start overtaken by its change's repair** (`checkouts.md`):
  starting from saved work preserves its edits, but a repair or resolution
  may have moved r since that save. Reconciliation is the forge's open
  question (`../forge/README.md`, section 11). Default: resume the saved tree; a
  moved delivery branch ends the run stale.
- **A park with markers left.** A save refuses a tree whose merge still holds
  a conflict marker, so a run that parks so keeps none of that run's edits,
  while earlier saved work stands. Its charter tells it to settle first
  (`temper help change`). Keeping them would need the list of files in conflict
  to outlive the hold: without
  it a saved tree's markers could be delivered, since the refusal at push
  reads that list. That list would be stored, so this is the user's.
  Default: settle before parking; keeping an unresolved tree waits for a
  deployment replay showing that need.
- **A push's message:** the worker commits the run's title and body; how
  a change's pull request takes them, and whether a squash keeps them.
- **An assignment naming an expected head.** The worker enforces one when
  given (`checkouts.md`), but the engine names none, so a branch another
  party rewound to an ancestor since the run started still takes a push
  that restores what that party removed. With branches held by one task
  (jig's `domain/connectors.md`, 3.3), such a rewind is drift the forge
  looks for after the push (../forge/change/README.md, 3). Naming the head prepare saw
  needs a decision for a branch that did not exist then.
- **Code graph:** indexing a checkout for codebase-memory-mcp as part of
  preparing it, and keeping that index with the cache.
- **A workspace's repositories in parallel:** a checkout runs one
  operation at a time, so its repositories are cloned, fetched and pushed
  one after another.
- **A store's upkeep.** Each fetch leaves what it received in the store as
  git wrote it, a pack or loose objects, and nothing repacks or prunes
  them, so a store grows across the worker's starts, since stores outlive a
  restart; and a store whose remote no prepare names since the start stays
  on disk, since the cache sets aside only the stores it knows. Default:
  none, and an operator who wants the space removes `stores` with the
  worker stopped; a repack that keeps every object a checkout may borrow,
  reachable or not, if a deployment shows a store's size or lookups.
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
- **The delivery's second-parent fetch** (`../../protocol/git.md`, 4.7) still
  asks the forge for every branch and tag, into the checkout: with the store
  borrowed, git sends only those branches' commits neither holds. Default:
  kept, since its rule is reachability from any branch or tag.
- **A workspace kept past its workstream.** The cache is keyed by workstream,
  so a new one starts from an empty directory and every file is new to a
  build, though its history is the store's. Keyed by repository, it could
  keep a working tree across workstreams and a
  restart once contained trees keep a run out of the git directory
  (`../../protocol/git.md`, section 10), perhaps with no switch. With that goes
  which workspace a workstream takes: the departure in `checkouts.status.md`
  considers only the least recently used, and after a restart only the one a
  first hold takes, since nothing says which workstream held which. So a
  clone another workstream wanted can be made again, wasteful and never wrong.
