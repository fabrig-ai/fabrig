# A change's drift

This document holds a change's drift, split from [`README.md`](README.md).

## 1. Drift

While its branch's writer slot is taken (jig's `domain/tasks/holds.md`, 3),
by a run of one of its delegates, by a run that handed it the branch and
has not yet answered, or by an update it asked for, the change decides
nothing on its branch's head: forward moves by temper's identity are
taken as that writer's (jig's `domain/connectors.md`, section 11), and
the change reads the head afresh once the slot is freed. Otherwise:

- **Its pull request closed** by someone else, or read as missing by
  its number: held, saying who and when. A release reopens it, by its
  number, never opening another (2, step 2); a cancel closes the
  change. **Only its own** (2026-11-15): a pull request the change does
  not own, an ended change's on the same branch among them, is no drift
  of its, nor is that change's branch: closed, reopened or retargeted,
  they wake and hold nothing of its (`../keeping-up.md`, 3).
- **Its branch moved** while nothing of temper's writes it, a late copy
  of its own update among them (`../updates.md`, section 1), or by another
  identity, or not forward: held, the head kept as it is, the hold's
  words naming the head it expected and the one it found, each at the
  forge's short length (`its branch moved: expected 0123456789ab, found
  89abcdef0123`). The head it expected is the one its step stood at
  (its last head, at a step that names none), and the one it found is
  the branch's head as the read that held it found it: never a head
  kept from an earlier read, its CI's, or its pull request's
  (2026-11-13). So the two always differ: a branch found at the head
  the change expects is no drift, whatever its pull request names
  (2, step 2), and a branch its own step reads as missing is deleted
  (below), not moved. Under its
  project's turn, no prepare writes the change's branch: its landing's
  job pushes to the landing's own (`../../landing.md`, 5.2). A head
  moved while the change is in a landing takes it out of the landing
  (`../../landing.md`, 4.3). A release
  takes the new head as the change's and checks it, gates asked again,
  its pull request opened first where none is open (2, step 2).
- **Its branch deleted:** held. A release creates the branch again at
  the head temper last knew, which the forge keeps under the pull
  request, and reopens the pull request.
- **Its pull request retargeted** to another base: held. A release
  retargets it back, or takes the new base as its landing branch if its
  requester amends the change so.
- **Its pull request merged by someone else:** landed, like any other
  landing. A merged pull request on reused branch names is its own only
  when its recorded number matches (`../effects.md`, section 1); any other
  merged pull request is Missing and cannot land the change. A head the
  landing branch already holds through another's merge ends the change
  with no change (2).

**On its requester's held branch** (1), the branch's writers are its
requester's delegates, one at a time, the change's own among them.
While the slot is taken by another of them (a change landing into the
branch, its merge job, its resolution), the change decides nothing on
its head, as above. Once the slot is freed it reads the head afresh,
and a forward move by temper's identity is its new head, never drift:
taken as after a repair, checked again (2, step 3), its gates asked
again; its repairs, resolutions and updates count on, and its merge is
never sent at a head it has not checked. Held, it decides nothing until
released, and the release takes the head as it then is, checked again.
A move with no writer of its requester's subtree in the slot, by
another identity, or not forward, is drift, as above.

**Its requester's own runs** (a goal's design writers, `../branches.md`, 3) are writers
the change doesn't wait for: they take no writer slot of the branch. A
forward move by temper's identity while one lives is that run's design
push, and the change's new head as it reads it, checked again as above,
while the run goes on, never "branch moved"; a move by another identity,
or not forward, is drift. The change's own delegates, its repairs, its
resolution and its merge job, take the slot as any claim does (jig's
`domain/tasks/holds.md`, 3), beside the design writer.

**Its own writes are its own.** A head a write of temper's made for the
change is the change's, recorded as its branch's head in the step that
settles the write, so no later read takes it for drift:

- **Its update** (`../updates.md`, section 1): the head the forge's answer
  names, or, its answer let go, a head read afresh that holds the base
  the update asked for (its `base`, though the base has moved on since).
  It goes on to CI on that head (2, step 3; under its turn, step 6).
- **On its requester's held branch, a merge into it** by a change of its
  requester's subtree (a goal's close-out or catch-up landing into the
  goal's branch, `../branches.md`): the merge holds the branch's writer
  slot from its entry until it settles, so the change decides nothing on
  the head meanwhile, as above; once settled, the merge's commit is the
  branch's head and the change's new head, checked again (2, step 3).
  The same holds for the merge job's push on the branch (`repairs.md`).
  A design writer's push recorded beside the merge that already descends
  from its commit stays the branch's head and the change's, checked
  again: the merge's record never moves the head back to it
  (`../branches.md`, 3, *In the order the forge took them*).

So a goal's landing whose own update, or its goal's close-out merged just
before, moves the goal's branch goes on to CI on that head and merges;
only a head no write of temper's in flight or settled explains is drift.

Nothing is written over: a held change makes no effect until it is
released. A drift is the change's to judge: what the forge says of a
known change's pull request or branch wakes its procedure, which holds
it for the cause above, rather than holding its task from outside before
the procedure has stepped, so the hold names its cause on the change's
panel.

## 2. Open questions

None.
