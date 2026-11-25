# Pushing from a checkout

This document holds checkout delivery and saving, split from [`checkouts.md`](checkouts.md).

## 1. Push commits what the run checked

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

## 2. A design delivery

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

## 3. A save pushes unchecked

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

## 4. A landed dependency's catch-up

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
    gains `landed` and `unlanded`, which carry nothing, with no version
    move (`../../protocol/wires.md`, 3.1). A worker whose layout lacks
    them never asks, so it refuses as before; an engine whose layout
    lacks `landed` cannot read an ask that holds it (jig's
    `protocol/channel/layouts.md`, 5.3), so the engine goes first, as for
    the report (`../forge/reds.md`, 5.6).

## 5. Each repository lands on its own

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

## 6. A task's own delivery outlives its hold

- **A task's own delivery outlives its hold**
  (`../../draft/archive/delivered-across-runs.md`, 2). A run that delivered and
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

## 7. Pushes are never cancelled

- **Pushes are never cancelled.** A push in flight runs to its end, which
  its deadline bounds, so one that lands is reported landed; one that
  ended ambiguously (out of time, broken, the forge lost on the way) is
  verified against the forge, by fetching its branch.

## 8. Saved work

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

## 9. Open questions

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
