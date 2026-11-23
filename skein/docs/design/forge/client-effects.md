# The forge client: effects, keys, write lanes and recovery

## 1. Effects: keys, write lanes and recovery

An effect is a `Write` of the vocabulary with its `Condition` (none for
most), committed by the owner as an `Entry`. The client makes it once
and says what happened.

### 1.1 Keys

`effect_key(namespace, identity, max)` gives the bytes of version `1`,
the namespace's length as two bytes big-endian, the namespace, and the
identity. It is none for an empty or over-long (65535 bytes) namespace,
an empty identity, or a total over `max`. A creation carries its key in
a marker (README.md, section 5).

- **The namespace** is the owner's, stable across restarts and releases;
  two services on one forge have two. **The identity** is the owner's
  nonempty bytes: the same effect has the same, different effects
  different ones. The client never inspects it.
- **Belief.** A key read from the forge is the owner's when it has
  version 1, the configured namespace and, as author, a configured writer
  of that forge. A party copying its bytes gains no standing.
- **`Config { namespace, writers }`** holds `Writer { forge, author }`
  entries: none to `repositories` of them, at most one per forge, no
  author 0, the namespace of 1 to 65535 bytes, and both within
  `op_bytes`, each writer counted at `WRITER_BYTES`, 16 bytes on every
  platform. `configured` refuses an invalid one (`TooLarge`). Until a
  forge has a writer, the client refuses its effects `Forbidden` and
  believes no key of it.
- **A writer at runtime.** `writer(d, env, Writer { forge, author },
  out)` gives a forge's writer once its access is known
  (connection.md, section 3.5). It emits `Save` of a `Writer` row,
  which a restart restores (`client.md`, section 8), and returns true. The same
  author again changes nothing and returns true. Another author for that
  forge, author 0, or a forge past `repositories` writers returns false
  and changes nothing: an identity that changed is setup's to resolve,
  never the client's (setup/README.md, section 3). A restored `Writer` row
  sets its forge's writer as if given; one that disagrees with the
  configured or an earlier one makes `restoration_failed()` true.

### 1.2 Entries and lanes

- **Admission.** `Make { entry }` is refused with `Outcome::Failed`, by
  the first of these that applies: `Busy` before restore ends;
  `Forbidden` where the forge has no writer yet, configured, given or
  restored; `TooLarge` over
  `op_bytes`, with an attempt but no start, an empty key, or a condition
  its write does not take (a merge takes a base, an update a head and a
  base, a review a head, the rest none); `Busy` with `entries` held. A
  new entry's number is above every held one (`client.md`, section 2,
  "Preconditions").
- **A lane** is a repository and an item number for a write that names an
  item (comment, review, edit, reviewers, close, reopen, merge, update),
  else the repository alone.
- **Commit order, one at a time.** An entry runs when no earlier entry of
  its lane remains. An uncertain entry holds its lane until found or
  settled, so a later write cannot overtake one that may still land.
- **Withdrawal.** `Withdraw { entry }` of an entry with no attempt ends
  it `Withdrawn`: at once, or at its answer when a call is out. Of one
  with an attempt (2026-11-14), it stops the write being sent again and
  leaves what was sent to settle: a call out ends as its answer says, a
  lost answer is looked for (section 1.5), and the entry holds its lane
  as any uncertain entry does. Found, it ends `Made { found: true }`;
  wherever it would be sent again, or be `Held` for a person's `Remake`,
  it ends `Withdrawn` instead. A queued merge stays the queue's (1.7).
  Of an entry the client does not hold, it changes nothing and says
  nothing.

### 1.3 Recovery classes

A class says what the forge lets the client do after a lost answer. It is
the forge's guarantee, not the owner's choice, and a client's check never
changes it, because a late copy can pass the check. **Keyed:** the forge
refuses a second creation with the same identity (no Forgejo write is).
**Conditional:** it checks the state the write names. **Idempotent:**
repeating gives the same state, the one the write names: a set's value,
an update's head holding its base. **Unrecoverable:** none of these.

`recovery(&Write)` gives the class. A lost write is found again by:

| Write | Class | Found by |
|---|---|---|
| open a pull request | Conditional | the open pull of its branches |
| merge, in the repository's style or fast-forward only | Conditional | its pull merged at its exact head |
| update from base, by a merge or a rebase | Idempotent: every copy leaves its head containing the base, or changes nothing | its head containing the base |
| dispatch a workflow | Unrecoverable | a run of its workflow at its branch's commit made since start (`Runs`) |
| rerun a job | Unrecoverable: Forgejo checks no attempt, and refuses only a run not completed | the job's attempt past the one named (`Run`) |
| update a ref, fast-forward only | Conditional: the forge refuses a move that is not a fast-forward | the branch at the commit, or at a commit containing it (`Branch`, then the ancestry range `Commits { before: tip, after: commit, page: 1 }` complete and empty): made, though main moved past it since |
| create an issue, comment, review | Unrecoverable | its key since start |
| create a branch | Unrecoverable | the branch at that commit |
| delete a branch | Idempotent | the branch gone |
| edit, reviewers, close, reopen, status | Idempotent | not looked for |

A branch can be deleted by another hand and nothing then shows it was
made; Forgejo's update checks neither the old head nor an operation id.
Of several pulls for its branches, the open one is found, else the
oldest. A comment or a review settles a find only if it is original (not
revised, provenance known) and carries the entry's key, believed
(section 1.1). An issue, whose summary carries no provenance, settles
it by its key, believed, alone.

**What a late copy of an update may do.** An update has no value a copy
could write over a newer one: whichever copy lands, and whenever, its
head holds the base it asked for, or the copy is refused or changes
nothing. So its class rests on no lifetime (jig's
`domain/connectors.md`, 4.3), and a forge's git work may outlast one. What
a copy costs is what it moves:

- **By a merge,** the branch only moves forward. A copy that read a head
  since moved is refused at its push; one that reads a head holding the
  base changes nothing; one that reads it after the base went on merges
  the newer tip, a new head.
- **By a rebase,** the forge's push is forced and names no head. A copy
  rewrites the head: the same commits on the base, under another id. One
  that read the head before a later push and lands after it **drops that
  push from the branch**.

Neither lands anything unchecked: a merge names its exact head and is
refused at any other (1.4). A head moved under an owner is the owner's to
judge.

### 1.4 One write's way

1. **Clock.** A new entry reads the forge's time (an empty listing, a
   `Fresh` read) as its start time.
2. **Check.** Only a write carrying a `Condition` (a merge, an update or
   a review) reads its pull request afresh; any other goes to step 3. A
   closed pull request fails `Closed`; another head fails `Stale`, as a
   merge's other base does. An update's pull request at its head but on
   another base commit fails `BaseMoved { base }`, `base` the commit the
   read reported (none where it reported none), so its owner can tell
   which moved. An update's check after a lost attempt fails alike
   (section 1.5).
3. **Make.** In the decision that sends the call the client emits
   `Progress` with the entry's `start` (a `Position`, fixed once) and its
   `attempt`: the send, its monotonic `deadline` and its absolute
   `expires`, both `lifetime + clock_margin` after it. The owner commits
   it first (`client.md`, section 2.1). Each send is an attempt: a write sent again
   after a rate refusal (`client.md`, section 6), which the forge did not act on
   (forgejo.md, section 1.4), saves a new attempt in place of the last,
   its deadline from the new send.
4. **Answer.** A typed answer ends in `Outcome::Made { made, found }`.
   A review, merge and update are proved by a read after the answer: the
   review by its key, the merge by its pull request merged at the entry's
   head and base, the update by an ancestry range showing the head holds
   the base: `Commits { before: head, after: base, page: 1 }`, complete
   and empty; one that lists a commit or is refused `TooLarge` shows it
   does not, and any other failure, or an empty page that says more
   follow, is a failed read. Neither reads a comparison's files. A merge
   not so proved is `Raced { Merged, Stale }`; an update
   not so proved is not found, and so made again (section 1.5). A merge answered `Queued` is
   the queue's (1.7).
5. **A lost answer.** `Timeout` or `InvalidAnswer` on a write may have
   reached the forge: the entry counts a failure, emits `Progress` and
   `Outcome::Uncertain`, and is looked for (section 1.5). `Exists` is
   looked for too, since every write is sent under an attempt.
6. **A typed refusal** (conflict, protected branch, stale head, a base
   moved, `Closed` and the like) is final, never retried blind. `Unavailable` did not
   reach the forge: it is tried again after `backoff` doubling to
   `backoff_max` plus jitter, up to `write_attempts` failures.
   `NotConnected` and `NotInstalled` reached no forge either, and fail
   the entry at once, as a typed refusal does: the forge's access, or
   the account's installation, is the owner's to restore before it
   commits the write again. A find that meets them fails as 1.5 says.
7. **Missing.** Deleting a branch already gone is made (`Set`). A find
   that meets `Missing` is a find not found; a set that meets it fails.

### 1.5 Finding again

`lifetime` bounds how long a sent write may still arrive, except for an
update: its idempotent transition rests on no lifetime (1.3). For an
update it sets when a find not found permits another attempt, even if an
earlier copy is still doing git work. A find does not restart the saved
deadline.

- **Found:** `Outcome::Made { found: true }`.
- **Not found before the deadline:** the entry waits for it, then looks
  once more. A find that fails is tried again after `backoff`, with no
  ceiling while the entry is uncertain.
- **Not found after it:** an unrecoverable write is `Outcome::Held`. The
  owner holds its task for a person; the client never sends it again by
  itself. `Remake`, a person's choice, hands the entry back with that rule
  lifted. Any other write, or one remade, fails `Timeout` after
  `write_attempts` failures; else it is checked and made under a new
  attempt.
- **An update checked again** may find its head or its base moved: by a
  late copy of its own, by another hand, or a base gone on. It fails
  as any check does, `Stale` for its head and `BaseMoved` for its base
  (1.4): each says the state it was decided on has gone, not that the
  update was not made. Its owner tells this from a refusal before any
  send by the entry's `attempt`, and reads afresh.
- **A set** (edit, reviewers, close, reopen, status) is not looked for. It
  waits out the first attempt's lifetime and is written again, so a late
  copy cannot land after the second.
- **Raced.** `Outcome::Raced { made, why }`: a known effect landed and its
  intended state cannot be confirmed, as when another hand closed the
  pull request, or a merge's pull request shows another base, head or
  merge commit.

### 1.6 Echoes

A comment or review the owner wrote comes back through the next scan, and
is not news. `Made::Commented` and `Made::Reviewed` add the new id to the
item's ledger (at most `rows`) until the scan passes it. A row is an echo
too when a configured writer wrote it and its key is an outbox entry's or
has version 1 and the owner's namespace. A keyed write waits for room in
the ledger, and its landing brings the item's next read forward to
`hinted`.

### 1.7 A merge queue

A merge answered `Queued` (vocabulary.md, section 1.3) is the queue's: it
was neither made nor refused, and it lands, or not, later.

- **Kept.** The client sets the entry's `queued`, emits `Progress` and
  `Outcome::Queued`, which, like `Uncertain`, settles nothing. The
  entry is never sent again, cannot be withdrawn, keeps no deadline (the
  queue's own timeout ejects it) and counts against `entries`.
- **Its lane.** A queued entry holds its lane until the queue decides
  (1.2's rule: a later write must not overtake a merge that may still
  land). So a comment or a review on a queued pull request waits until
  the queue decides; the owner knows this from `Outcome::Queued`.
- **Looked at.** A fresh `Read::Pull`, budgeted as a find (1.5), on each
  hint naming the pull request and at each `poll` while the entry is
  queued, stretched while its repository hears (`client.md`, 4.1), on the timers the
  client already keeps.
- **Decided.** Merged at the entry's head: `Made { Merged(commit) }`.
  Merged at another head: `Raced { Merged, Stale }`. Closed unmerged:
  `Failed(Closed)`. Open with `queued`: still kept. Open and out of the
  queue: `Failed(Stale)` where its head moved, `Failed(Conflict)` where
  it is not mergeable, else `Failed(Refused)`; the pull's `dequeued`
  holds the queue's own words, for the owner to read. The owner repairs
  and commits a new merge entry; the client never queues a pull again by
  itself. A look answered `Missing` is `Failed(Missing)`, and the lane
  is free.

### 1.8 A stop

An owner that stops settles its effects first (jig's
`domain/connectors.md`, 4.2; temper's `domain/root-restart.md`, 1.2). `Settle`
tells the client to finish what it sent and to send nothing new; it is
the client's last state, and a start builds a new client.

- **Nothing new is written.** No write is sent from then on: not an
  entry's first, nor one again (after `Unavailable` or a rate refusal, a
  set past its lifetime, a remade entry). An entry not yet sent stays as
  it is, its row the owner's, for the next start; so does a write's
  call queued and not yet sent.
- **What was sent finishes.** An entry whose write this client sent and
  which has no outcome yet goes on to its outcome as ever: its answer,
  then the read that proves a review, a merge or an update (1.4, step
  4). A lost answer ends in `Uncertain`, its find left to the next
  start; a merge answered `Queued` ends there (1.7). The client starts
  no other read of its own meanwhile: no listing, no poll, no find.
- **`Settled { left }`** once every entry it sent has its outcome, at
  once if none lacked one; or at its bound, twice the sum of `lifetime` and
  `clock_margin` after `Settle` (the write's lifetime, then its proving
  read's), `left` naming each entry still without its outcome. The next
  start finds those by their attempts (`client.md`, section 8). The bound is one
  function, `stop_bound(&Limits) -> Duration`, saturating, which the
  settle and an owner that prints its own stop bound both read.
- **The owner** commits the outcomes as ever (`client.md`, 2.1), and asks nothing new
  of the client while it settles.
