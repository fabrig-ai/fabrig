# Checkouts

The checkout child domain is the worker's workspace capability (jig's
`domain/hosts/README.md`, section 6): it owns the workspaces on the worker's
disk. Its vocabulary is typed git and file operations (make a workspace,
make a store, clone, fetch, point a reference, check out, merge, commit,
push, pack), which the protocol layer
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
- **Repositories side by side.** A workspace holds one directory per
  repository, so code that refers to a sibling by path works, and the
  agent's write authority is the list of writable ones.
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
- **Prepare** fetches what the assignment names and checks out exactly its
  starting point. It fetches each branch of a repository at most once: a
  pin or a baseline naming a branch this prepare has fetched, or a commit
  it has fetched, takes that fetch's commit, and a pin is pointed at it
  locally (`../../protocol/git-operations.md`, 1.12). A branch or base that does not exist is a preparation
  failure, permanent until something changes on the forge: the worker
  creates no branch but by pushing a run's change, and branches temper
  needs before then are made by the engine, as effects (../forge/branches.md
  ).
- **The landing pin.** A repository whose item names a landing branch
  (`hosted-runs.md`, 1.1) has it fetched once per hold, during prepare, after its start is
  fetched, on the same remote, into
  `refs/temper/landing` (`../../protocol/git-operations.md`, 1.3), or, where it is the
  branch the start fetched, pointed at that fetch's commit: every start of a
  change's producer, a saved or resumed one's included, and a goal's later
  run. That tip is frozen for the hold: no later operation of the hold
  moves the reference but a catch-up, which fetches it afresh (*A landed
  dependency's catch-up*, below), so the project's checks take the same base at every
  delivery and at finish until then (`tools/pre-pr/design.md`, section 2). A missing
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
  pinned. So a repair the change asks for itself checks what its own edits
  changed against the head it repairs, never against the branch the change
  lands on: a CI repair of a goal's landing into main checks its own edits,
  not the goal's (`tools/pre-pr/design.md`, section 2), and no pin a cached
  checkout kept from an earlier hold is read. A repair asked by a landing
  names a pin instead (*A named pin*, below). The pin is frozen for the hold
  as a landing branch's is. A read-only repository delivers nothing and runs
  no check, and its reference is left as it is. The item still names no
  landing branch, so nothing else of it changes: it is never taken for a
  goal's design delivery (`../agent/README.md`, section 5), and its start,
  push and expected head are as before.
- **A named pin.** A writable item may name a pin, a commit by its full
  object id, and then names no landing branch. The engine names one on the
  item of a repair asked by a landing alone: the base that landing's round
  used, its failed receipt's (`../forge/change/repairs.md`, *A landing's
  repair*). Prepare fetches the push branch's head as for the replaced
  head's pin, the one fetch serving its baseline and, where it is the
  start's branch, its start, and points `refs/temper/landing` at the pin in
  place of that head: a `Point` (`../../protocol/git-operations.md`, 1.12) where this
  prepare already fetched that commit, otherwise one fetch of it into
  `Into::Landing` (`../../protocol/git-operations.md`, 1.3), its want the commit. A
  pin the remote lacks fails preparation as a missing commit; it never
  falls back to the replaced head. So a landing's repair is judged as that
  landing judged its change: its diff, its figures and its tests against
  the base the landing used, whatever the change's head and however old
  its source (`tools/pre-pr/design.md`, section 2). The pin is frozen for
  the hold as every pin is, and changes nothing else of the item: its
  start, push, expected head, own delivery, ownership and drift admission
  are a repair's as before. An item naming both a pin and a landing branch
  is refused at the worker's entrance as an invalid start that names it
  (`hosted-runs.md`, 1.1).
- **Main's counts** (`../../draft/test-budgets.md`, 3). Each repository
  whose prepare writes `refs/temper/landing` (any pin above) asks one
  `Counts` at once after it, on the same remote and identity: the forge's
  `refs/temper/counts`, the commit holding `counts.json` that the gate's
  runner publishes, fetched into the store with its blob and pointed at in
  `refs/temper/counts`, which the view copies into the run's `.git`
  (`../../protocol/git-operations.md`, 1.3 and 1.13), so the run's checks
  read it offline. Then the pin's continuation follows.
  - **Where the forge has none,** nothing is fetched and nothing fails:
    no reference is written, a cached checkout's from an earlier hold is
    removed, and the prepare goes on as without it.
  - **Frozen for the hold:** written afresh by each prepare alone, never
    by a catch-up, so a cached checkout's is replaced, never read. A
    repository whose prepare writes no pin asks none, its reference left
    as it is.
  - **A fault** fails the preparation as a fetch's does, at the stage
    `fetch counts` (section 1.1): its remote has just served the pin, and
    no prepare ends with a reference it could not bring up to date. Its
    fetch counts toward the store's next `Pack` (*Kept packed*).
- **A goal's reds for each check** (`../forge/reds.md`, 5.2 and 5.3), in
  each writable repository with a pin, under the run's git directory, at
  `.git/temper` (`../../protocol/git-operations.md`, 1.13): prepare
  removes `temper/reds` and writes `temper/reds-wait`; a check's start
  removes `temper/reds`, and, once the engine answers, the checkout finds
  the check's base and each red's ancestry with git, in the worker's
  directory, whose HEAD and pin the run's view holds too
  (`git merge-base`, `git merge-base --is-ancestor`), and writes
  `temper/reds` through `temper/reds.tmp` and a rename; a check's end
  removes it. These are the checkout's typed operations, run as its
  others are, one at a time; a failed one writes no file and fails
  nothing else of the run.
- **An existing producer branch** compares with the pinned tip: it is the
  frozen comparison for existing work below, only for an inherited
  producer that starts on its push branch.
- **A base with heads merged in** fetches the base branch and each head,
  checks out the base's tip, and merges each head into it in their
  order, committing each merge at once, with two parents (what was
  checked out so far and the head) and the repository's identity as its
  author, and checks that commit out before going on (`../../protocol/git-operations.md`,
  1.7: a commit moves neither HEAD nor the merge's state), so that the next
  merge can run and the agent is handed a checkout on the last commit with no
  merge in progress; a head the checkout already contains is no merge. The last
  commit is the checkout's start, for its saves and its push alike, and
  the branch it pushes is made by its first push, as a base's is. A
  head that does not merge cleanly fails the preparation as a conflict,
  naming the head, permanent for this assignment: nothing of the
  merges is kept, and nothing is pushed. The change it was for then
  waits for its dependencies to land (`../forge/change/questions.md`, 1).
  This is the one conflict a prepare refuses, and only an early producer's
  start meets it; its detail (`checkouts.md`, section 1) is stage and fault
  `merge: Conflicted`, and its words say why,
  `head <head> does not merge into <branch>`, `<head>`
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
  order (`../../protocol/git-operations.md`, 1.6). A conflict this start meets is the
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
  `Merge <base>`, and that commit checked out (`../../protocol/git-operations.md`, 1.7). So
  its run starts on a clean tree with no merge state, its earlier work and the
  base's files both in its head, and nothing staged to misread or remove. The
  commit it merged into stays the checkout's start, so a save with nothing new
  saves the merge commit, and its first push carries the merge, even with
  nothing new. A producer's conflict stays in progress as above, and its start
  names the files in conflict (smith's `domain/run-delivery.md`, 1.3).
- **A goal's later run** (`../forge/branches.md`, 3) starts at g's tip; where its
  run branch holds a head g lacks, it starts as a producer does, on a merge
  in progress naming its landing branch: the branch is g, the commit merged
  into it is that head. A clean merge is committed at once, and a conflict
  stays in progress. It pushes to g, a design delivery (below).
- **Push commits what the run checked.** When a run asks to deliver, at
  `finish` with a change or mid-run, it has run the repository's checks
  with nothing else writing to the checkout (smith's `domain/run-delivery.md`,
  1.1); the worker commits exactly that tree and pushes the commit, with
  the run's title and body. Delivery reads the checkout's current HEAD and
  merge state, the worker's (*Two git directories*): an in-progress merge records HEAD and MERGE_HEAD as its two
  parents, even when its resolution is already staged. A committed merge
  keeps its ancestry. When the delivered tree is HEAD's tree and no merge
  is in progress and HEAD is not this hold's unchecked saved commit, the
  worker delivers that commit unchanged, retaining its message and identity. Edits beyond a local commit build on that
  commit. A second parent, in progress or at HEAD, must be reachable from a
  branch or tag on the delivery branch's remote; otherwise delivery refuses,
  saying that its second parent is not reachable from that remote, before
  staging or committing; a head of a base with heads merged in whose
  dependency has landed is caught up with instead (*A landed dependency's
  catch-up*, below). A new delivery commit becomes the checkout's HEAD
  and clears its completed merge state (../../protocol/git-operations.md, 1.7).
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
  `domain/run-delivery.md`, 1.1); a failed check refuses it. The checked tree is
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
  (`../../protocol/git-operations.md`, 1.7, `always`), with that commit's own message,
  so a save's commit is never what the hold's next delivery or saved work
  names. An unchecked save uses the same checkout snapshot as a checked delivery:
  staged merges keep both parents, a matching local commit is kept, and a
  new commit advances HEAD. Its second parent is checked, and a landed
  dependency's head caught up with, as a delivery's is. Only the checked delivery runs the checks. A
  checked delivery's landing clears the record.
- **A landed dependency's catch-up.** A producer that started early
  works on a base with heads merged in: its HEAD is the last of those
  merges, with the last head as its second parent. Once that head's
  dependency lands, the landing squashes it and its branch is deleted
  (`../forge/change/questions.md`, 1, *A running producer catches up*),
  so no reference on the remote reaches the head, and the delivery's
  check of its second parent ends `ParentUnreachable { head, merging,
  parent }` (`../../protocol/git-operations.md`, 1.7). The hold then catches up,
  in a delivery and a save alike; this is the one reconciliation a hold
  makes with its landing branch:
  - **When.** The repository's start is a base with heads merged in and
    `parent` is one of its heads. Any other unreachable parent is
    refused as before, with the same words, and nothing is asked.
  - **It asks.** The hold asks its client whether that head's
    dependency has landed, `Request::DependencyLanded { client, head }`,
    and waits, with no operation in flight, for exactly one
    `Event::DependencyLanded { hold, landed }`. The worker asks the
    engine (*Its ask*, below) and answers `landed: true` for the
    engine's `landed` alone; `unlanded`, a refused ask, an answer that
    doesn't decode and none within `checkout.reds_read_ms`, the bound of
    the worker's other read of the engine (`../forge/reds.md`, 5.3), are
    `false`. An abort, a release or a cut while it waits ends the push
    as one with nothing in flight does: that repository and those after
    it aborted, nothing committed or pushed.
  - **Not landed:** refused as before, the fault `Refused` with
    `delivery second parent is not reachable from the branch's remote`,
    nothing staged, committed or pushed.
  - **Landed: the work kept.** The working tree is committed locally on
    the `head` the delivery read, with the `merging` it found in progress
    as its second parent, the push's own title and body, and a
    delivery's staging (`../../protocol/git-operations.md`, 1.7); a tree equal to
    `head`'s commits nothing. That commit, or `head`, becomes the
    checkout's HEAD with every working file as it was, ignored files and
    hidden logs included. Nothing is pushed yet.
  - **The pin moved.** The landing branch, the start's branch, is
    fetched afresh into `refs/temper/landing` (`../../protocol/git-operations.md`,
    1.3): the one move of the pin a hold makes, explicit, so the checks
    that follow take the tip just caught up with as their base, and the
    goal's reds are read against it (`../forge/reds.md`, 5.2).
  - **The merge, work first.** That tip is merged into the work as a
    producer's merge start merges its base (*A merge in progress*,
    above): a clean merge is committed at once with two parents, the
    work first and the tip second, the repository's identity as its
    author and the title `Merge <branch>`, and checked out, clearing its
    state; a conflict is left in progress, the tip its MERGE_HEAD and its
    files named, for the run to settle as at any catch-up. A tip the work
    already contains is no merge. The tip is on its branch, so the next
    delivery's second parent is reachable, and its push carries the
    work, the dependency's head beneath it, to the change's branch as a
    fast-forward.
  - **What the run is told.** A delivery pushes nothing, since its
    checks ran on the tree before the merge: it is answered `CaughtUp {
    tip }`, which the run reads as refused, with no path, `a dependency
    you started from landed; its landing branch is merged into your
    checkout: deliver again`, so its next delivery checks the merged
    tree. A save, which runs no check, goes on as a save does from the
    merge, its push recorded as any save's is (above), and is answered
    `CaughtUpSaved { commit, tip }` once it lands: a save's receipt for
    `commit`, adding `, merging the landing tip <tip> first`, `<tip>` as
    a receipt spells a commit. A conflict, in either, is answered
    `CaughtUpConflict { files }`, which the run reads as refused at its
    first file, `a dependency you started from landed; settle the files
    in conflict with its landing branch and deliver again`; a save of
    that tree is refused for its markers until they are settled, as any
    merge's is.
  - **Its ask.** An application's ask on jig's channel, as the reds'
    is (`../forge/reds.md`, 5.4), the run's task and attempt beside it,
    one at a time per hold. `temper-worker-wire`'s `Ask` gains `landed`,
    `Head { head }`, `head` a full object id in 32 bytes as the items
    carry a merged head (`../../protocol/channel.md`, 4); its `Answer`
    gains `landed` and `unlanded`, which carry nothing. The family moves
    by one, from 9 to 10, its floor kept, with its kept text and patch
    (`../../protocol/wires.md`, 3.1). A worker of the previous version
    never asks, so it refuses as before; a worker of this version
    meeting an engine of the previous one can't send `landed` up, so it
    closes at the statement, `Ask` among the parts a worker sends whole,
    and the engine goes first, as for the report (`../forge/reds.md`,
    5.6).
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
- **A task's own delivery outlives its hold**
  (`../../draft/delivered-across-runs.md`, 2). A run that delivered and
  then parked ends its hold, and its next run's hold starts with nothing
  of its own pushed, at the head that run delivered. So the assignment
  names, for each writable repository, the task's own delivery,
  `delivered` (`hosted-runs.md`, 1.1; `../../protocol/channel.md`, 4): the
  head of this task's last checked push to that push branch, as the
  engine recorded it (`../forge/change/questions.md`, 1, *A checked
  head*), or none.
  - **Its evidence.** The hold takes `delivered` as this task's work only
    where the push branch's head fetched at prepare is that head and the
    checkout starts on it: a start from the push branch, a producer's
    merge start whose branch already contains its base among them, since
    it goes on as a start from that branch does (*A merge in progress*,
    above). Anywhere else it is no evidence, and nothing of it is read. A
    clean tree alone is never evidence.
  - **A checked push with nothing new pushes it again.** For a `deliver`
    or a `finish` with a change, a repository with that evidence and
    nothing new since the start is this task's delivered work, not
    unchanged: the worker pushes its head again, requiring the remote to
    equal the assignment's expected head, or the fetched start when none
    was supplied, as for existing producer work (above). Only a
    successful or verified push returns its landing, `Landed` at that
    head, which is then this hold's last push, so a later `deliver` or
    `finish` with nothing new is that landing again, as within one hold.
    A branch that moved since prepare fails the push as moved.
  - **Everything else as before.** With no `delivered`, or one the
    fetched head is not (the branch moved past the task's delivery, or
    holds another party's head), a clean tree is unchanged and a pushed
    head the branch cannot take is moved. A `save` with nothing new
    stays unchanged, and saved starts keep their own guarded first push
    (*Saved work*, below). For an inherited producer the evidence comes
    first: a head it does not cover goes to the landing tip's comparison
    above, unchanged.
  - **Nothing of it is kept on the worker.** Each hold reads its
    assignment's `delivered` afresh, so a restart of the worker, or a
    run placed on another, changes nothing.
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
  Work done since the last save is lost if the worker dies, unless the
  run's next attempt takes its workspace up on the same worker
  (*A workspace kept for its run*). A merge start
  whose branch is the repository's saved-work branch started from saved work:
  a later save of it that lands nothing (nothing new, markers left, a failed
  push) leaves that saved work standing, and the next run starts from it. A
  run's own edits are in the saved commit, so in the next run's head, and
  `git status` shows none of them. A run that parks with a conflict marker
  left keeps none of that run's edits (`README.md`, section 10); what was saved before it
  stands.
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
    keep replaces it with the `kept` mark above. A hold with no
    saved-work destination writes none.
  - **Found at the next start.** The start's layout keeps each workspace
    a `held` mark names as it keeps a `kept` one, a crash's leftover,
    and removes from each kept workspace's repositories the lock files
    a killed git leaves (`index.lock`, `HEAD.lock`, and any name ending
    `.lock` under `refs/`), in its `.git` and its worker's git
    directory, since no git of the previous process lives. The cache
    holds it, idle, under its workstream's key, so the hello names that
    workstream and the engine sends the run's next attempt back at once
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
- **No credentials on disk.** Every git operation that reaches the forge,
  or commits, names the repository's identity, which the protocol layer
  maps to credentials or an author for that invocation only, never into a
  checkout's configuration, where an agent could read them.

Each commit made while preparing a base with heads merged in has the title
`Merge <head>`, where `<head>` is the merged head's full hexadecimal object
ID (2026-11-04).

## 1. A failed prepare says where and why

A prepare that fails answers with a detail (`Prepared::Failed`'s `detail`):
operator evidence, bytes never parsed, that says which workspace and
repository, which stage and git command, and how long each part took, so
that a timeout reads as the operation's own time or as time spent before
it. It travels in jig's failure detail (`Failed.detail` and
`JobFailed.detail`, at most 4,096 bytes, jig's worker wire), so no wire or
stored format changes; the worker's line and serve's print it escaped
(1.3, 1.4).

### 1.1 Its form

```
<stage>: <fault> (workspace <w>, repository <i> <name>, command <command>, operation <o> ms of <d> ms, prepare <p> ms, queued <q> ms)[: <words>]
```

- **`<stage>`:** the step the prepare was in, in the words it has always had
  (`make`, `store`, `clone`, `fetch`, `check out`, `fetch landing`,
  `fetch counts`, `merge`, `check out merged`, a job's `fetch member` and the rest;
  `hold.rs`, `preparation_detail`). A step that checks reds files fails
  nothing and has no detail.
- **`<fault>`:** git's fault as `git::Fault::name` writes it
  (`Missing(Branch)`, `Unreachable`, `Broken`, `TimedOut`, ...), or
  `Conflicted` for a head that does not merge.
- **`workspace <w>`:** the workspace's token in decimal, as io names its
  directory, `workspaces/<w>` (`../../protocol/git.md`, section 2).
- **`repository <i> <name>`:** the repository the step names: its place in
  the spec in decimal, and its directory's name. A step that names none
  (`make`) says `repository none`.
- **`command <command>`:** the invocation the failure ended in, as the git
  component names it (`../../protocol/git.md`, section 6): git's subcommand,
  so `checkout` and `clean` are told apart within the one `check out`
  stage, or a file operation's word. A terminal that names none (one that
  ran no invocation, as an operation the component received past its
  deadline, and a conflict) says `command none`.
- **`operation <o> ms of <d> ms`:** the operation's time, from when the hold
  asked it of io to its terminal, against its deadline's length:
  `checkout.remote_timeout_ms` for one that reaches the forge, the local
  timeout otherwise (`git::Op::is_remote`).
- **`prepare <p> ms, queued <q> ms`:** the prepare's time, from its arrival
  to its failure, and the part of it spent queued for a permit, from its
  arrival to its admission, 0 for one admitted at once. Earlier operations
  and a wait for another hold's store are in `<p>` alone, so `<p> - <q> -
  <o>` is what the admitted prepare spent before this operation.
- **`<words>`:** the rest of why, last: git's diagnostic, the tail of its
  stderr (`PushDiagnostic`), or, for a head that does not merge,
  `head <head> does not merge into <branch>`. With nothing to say, the
  detail ends at the parenthesis.

Times are in whole milliseconds, rounded down, from the domain's clock at
each step (`env.now`): the checkout arms no timer for them. For example,
the stage that failed six times on 2026-11-15 would now read:

```
check out: TimedOut (workspace 3, repository 0 fabrig, command checkout, operation 60000 ms of 60000 ms, prepare 63120 ms, queued 2950 ms)
```

### 1.2 Its bound

A detail is at most 2,048 bytes. Its stage, fault and fields come first and
whole, but for a name past 256 bytes, cut to its first 256; its words take
the room left, and past it keep their first bytes. With the default
`name_bytes` (256) and a diagnostic's 512 bytes nothing is cut. Every cut
on the way, this one, the worker's (1.3) and serve's line's (1.4), falls at
the end, on the words: the fields keep their room. jig's hub keeps a
detail's last `hub.detail_bytes` (4,096 by default), past which a detail
of this bound never goes.

### 1.3 The worker names itself and logs it

- **The host.** As it answers a run or a job whose preparation failed
  (jig's `Unprepared`), the worker service puts `host <name>: ` before the
  detail, `<name>` its configured name (`workers[].name`, jig's
  `protocol/configuration.md`, section 4), and cuts the whole
  to `hub.detail_bytes`, its wire's bound, keeping its head. An answer the
  hub sends again after a relink carries the same. A prepare refused at
  the entrance or aborted (`prepare refused: Busy`, `prepare aborted`,
  `hosted-runs.md`, 1.3) takes the host too, with no fields.
- **One line each.** The worker logs a line for each failed preparation, a
  run's or a job's, through its notes (`worker_alone`'s `Note`), which its
  shell writes, an in-process worker's among serve's:

  ```
  <binary> worker: run <run> attempt <attempt> failed <failure>: <detail>
  ```

  `<failure>` is jig's failure as serve's run-failure line writes it
  (`Unprepared(Transient)`), and `<detail>` the detail with its host,
  escaped as serve's lines escape theirs, one line. It is logged once,
  when the answer first goes up: a kept answer sent again after a relink
  logs none. A job's own line (`jobs.md`) stays beside it.

### 1.4 Serve prints it whole

Serve's run-failure line (`hosted-runs.md`, 1.3) prints the detail whole, every
byte escaped. The wire bounds it at 4,096 bytes; a longer one, which no
wire carries, keeps its first 4,096, where the fields are.

## 2. Open questions

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
  looks for after the push (../forge/change/drift.md, 1). Naming the head prepare saw
  needs a decision for a branch that did not exist then.
- **A save as the task's own delivery** (*A task's own delivery outlives
  its hold*): only a checked push is a task's own delivery, since the
  engine's record is the change's last checked head and a save's tree
  ran no check. A run that saved on its push branch after its last
  delivery parks with a branch head no evidence covers, and its next
  run's clean finish is nothing, as before. Default: so (2026-11-17);
  counting a save would deliver an unchecked tree as checked.
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
- **An earlier head lost.** A delivery checks HEAD's second parent
  alone (`../../protocol/git-operations.md`, 1.7), so of a start with several heads
  merged in only the last is checked and caught up with (*A landed
  dependency's catch-up*); an earlier head whose dependency landed or
  was cancelled goes up beneath the push, as it did. Default: so; a
  check of every head the start merged, if a deployment shows a push
  carrying a cancelled dependency's work.
- **The delivery's second-parent fetch** (`../../protocol/git-operations.md`, 1.7) still
  asks the forge for every branch and tag, into the checkout: with the store
  borrowed, git sends only those branches' commits neither holds. Default:
  kept, since its rule is reachability from any branch or tag.
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
