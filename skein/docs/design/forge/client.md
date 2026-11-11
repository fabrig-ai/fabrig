# The forge client

Provisional, 2026-10-20.

`skein-forge-client` keeps a bounded view of a forge for the domain that
owns it, and makes each write the owner commits at most once. It is
`no_std`, lib-only, and depends on `skein-lib` and on `skein-forge`'s
vocabulary (README.md, section 3). It makes no HTTP call and knows no
policy: the owner runs its calls (connection.md) and decides the
rest.

## 1. In one page

- **A capability, not a connector.** It keeps, reads and writes. What to
  watch, which effects exist, when to write and what people see are the
  owner's (section 9).
- **The owner commits, then releases** (section 2.1): a decision's saved
  rows, progress and outcomes first, its calls after.
- **A bounded working set** (section 3), refused whole past capacity,
  **kept up by listing,** sooner on a hint, which is a reason to read and
  never a fact, and less often while deliveries arrive (section 4).
  **Fresh reads go first** (section 5).
- **One request budget** (section 6): four classes share `rate` requests
  per `window`, never more than the forge says it has left, and a `304`
  costs nothing. A rate refusal holds them all until its reset.
- **Each effect is made at most once** (section 7). Its key, lane and
  recovery class decide how a lost answer is found. The attempt is saved
  before the write starts, so the search survives a restart (section 8).
- **One writer per forge,** configured or given at runtime once, and
  kept as a row; another author is refused (section 7.1).
- **Admission is bounded:** `valid` checks the limits (section 2.2).

## 2. Its owner, and how it is driven

The owner is a domain: a service's forge connector, which keeps the
service's durable records. The first is temper's. The client is its child
domain (../../foundation/programming-model.md, section 4.5), with its own vocabulary,
limits and tests; the owner translates between the two
vocabularies with total functions and routes the client's output.

- **Entry points,** each given `Env<Limits>`: `step(d, env, event, out)`
  decides one `Event`; `fire` expires one deadline; `resume` sends at
  most one eligible call. Questions (`next_deadline`, `is_due`,
  `is_ready`, `has_room`, `cached`) read state; `pop_fact` gives
  diagnostics, which decide nothing.
- **Construction:** `Domain::new(limits)`, `seeded(limits, seed)` and
  `configured(limits, seed, config)` with the owner's namespace and
  writers (section 7.1). A forge's writer may come later, at runtime, by
  `writer` (7.1). **The namespace is given before any restored
  entry or live resource is handed over** (section 8). An owner that
  learns it after construction sets it with `bind_deployment(id, limits)`,
  a 16-byte id, before that restore. It returns false, changing nothing,
  once the client has taken any `Restore`, `Keep`, `Make` or `Remake`,
  with no configuration, or when the result breaks section 7.1.
- **Calls are the owner's to run.** `Request::Call` carries a token, a
  repository and an `Op` (README.md, section 3); the owner returns exactly
  one `Event::Answered { call, cost, free, rate, result }` per token, a
  cost of 0 included, with the component's `free` and `rate`
  (connection.md, section 4.1).
- **The client keeps no store.** It emits `Save` of a `Stored` row (`Live`,
  `Repository` or `Writer`) and `Erase` of its `Key`. The owner commits an outbox
  `Entry` and hands it over with `Make`; the client returns `Progress`
  and, when the entry settles, an `Outcome`.
- **Preconditions,** asserted and never answered, since breaking one is
  the owner's bug: a `Make` whose entry's number is not above every
  held one (7.2); a `Restore` of a row already restored, or after
  `Restored` (section 8); an `Answered` for a token with no call out.

### 2.1 The commit rule

A decision is one call of `step`, `fire` or `resume`. The owner commits
the client's `Save`, `Progress`, `Erase` and `Outcome` of a decision
before it releases the `Request::Call`s the same decision made.

- **Why:** a write's attempt is in the `Progress` of the decision that
  sends it. After a crash that attempt says the write may have landed,
  and restart looks for it (section 8). A decision not committed sent
  nothing: the client rebuilds from the rows that did commit.
- **Outputs:** a decision pushes into the owner's uncapped queue without
  reserving room; the commit rule keeps its writes before its calls.

### 2.2 Limits

The owner chooses the limits and checks `valid` before
construction: `pending` and `calls` (at most `pending`), `op_bytes` and
`answer_bytes`; `rate`, `reserve` (at most `rate`) and `window`;
`read_attempts`, `backoff` and `backoff_max` (at least `backoff`);
`resources`, `repositories`, `rows` and `inbox`; `poll`, `poll_max` (at
least `poll`), `hinted`, `slow`, `settle` and `fallback`; `entries`,
`write_attempts`, `lifetime` and `clock_margin`; `facts`.

`valid(l)` refuses a zero limit except `op_bytes`, `answer_bytes`,
`reserve`, `backoff_max`, `poll_max`, `fallback` and `facts`, and refuses
a pair above out of order or an unrepresentable derived capacity.
`fallback` 0 stretches no pace (4.1). Operations, answers, pending
calls, rows and kept facts retain their edge caps.

### 2.3 Admission and answer checks

- **The measure** is the vocabulary's (README.md, section 3.7), which
  `skein-forge`'s `measure::op` and `measure::answer` give. The owner
  bounds what it stores of the client's by the same widths, with
  `skein-forge-client`'s `measure::stored(&Stored)` and
  `measure::effect(&Effect)`: a comment, a review or an echo a live row
  keeps counts 16, a label 16 and its length, a reviewer 8, and every
  string its length.
- **An operation is admitted** when its bytes are at most
  `op_bytes`, its pages start at 1, a file's path is not empty and a
  `max_bytes` is from 1 to `answer_bytes`; else it is `TooLarge`, with no call: `OpBytes` with the operation's bytes,
  or `AnswerBytes` with the `max_bytes` asked, both exact (README.md, 3.4).
- **An answer is checked** against its call before anyone takes it, by
  README.md, section 3.7's table. Past `answer_bytes`, or with a list past `rows` (every list but `Branches`' names), it is
  `TooLarge` for a read, naming the bound it passed with the answer's exact
  measure, `Rows` (its longest list's rows) before `AnswerBytes` (its bytes)
  where it passes both; for a write it is `InvalidAnswer`: a lost answer, since the write may have been done (7.4, step 5). Any other
  mismatch is `InvalidAnswer`, and so is an answered `Busy`: `Busy` is
  the client's own refusal, never a dialect's.

## 3. The working set

The client holds the resources the owner names, and nothing else.

- **A resource** is a `Repository`, `Branch`, `Pull` or `Issue` of a
  repository. A `Watch` adds whether it **participates**, delivering
  people's comments and reviews; only a pull request or issue may. A
  `Repository` watch keeps its repository's row and reads nothing of its
  own; a repository is listed only for its issue and pull watches
  (section 4).
- **Rows,** saved as they change (section 2.1): a repository's `mark` (the
  forge's time of its last complete listing) and `clock` (the newest
  forge time seen); per resource, its `Watch`, scan cursors, the stamps
  (id, revision) of comments and reviews delivered (at most `inbox`
  each), cached values, pushed head and echo ledger (section 7.6).
- **`Event::Keep { owner, watches }`** names the complete set the owner
  needs. `Kept` answers once, and anything but `Ok` changes nothing. Its
  refusal is the first of these that applies:
  1. `TooLarge`: over `resources` watches;
  2. `Busy`: before restore, or after a failed one (section 8);
  3. each watch in turn: `TooLarge` for a branch name empty or over
     `op_bytes`, then `Refused` for participation on a repository or
     branch, then `Refused` for a duplicate of an earlier watch;
  4. `Busy`: the set plus the leavers still busy passing `resources` or
     `repositories`.
- **Leaving.** A resource the owner stops naming is erased and gets no
  further call; a call already out keeps its one terminal, its answer
  dropped, and counts against capacity until then. A closed item is not
  dropped by the client: the owner stops naming it.
- **Scan positions** advance in the decision that delivers their news, so
  a restart between pages repeats no row. A scan that would pass `inbox`
  stamps or pages fails `TooLarge` and delivers nothing from that page.
  `cached(resource)` returns the latest values read; a decision that
  depends on one reads it afresh (section 5).
  `live_keys()` allocates the retained live rows' keys in the keep's order for a caller naming a row on a failure path.

## 4. Keeping up

- **A listing per repository** reads what changed since its mark: by the
  forge's time, inclusive, least recently updated first, paged from 1.
  Its cost follows the rate of change, not the items held. It repeats
  after `poll`, or `hinted` once a hint named the repository.
  - **Only for items.** A listing nudges only issues and pull requests
    (below), so it runs only while its repository has an `Issue` or
    `Pull` watch. A repository watched only for branches, or as a
    `Repository`, keeps its row and lists nothing, and a hint starts no
    listing there. When its last issue or pull watch leaves, its listing
    stops and its mark is cleared, saved; when one comes again, the
    listing starts with no mark, since every watch it then serves is new
    and read at once. A restored repository with no restored issue or
    pull watch lists nothing, and a fresh pass (section 8) owes no
    listing for it.
  - **No mark:** the first listing takes the forge's time as the mark and
    reads no history.
  - **Paging.** A full page ending at a later time moves `since` there
    and restarts at page 1. A full page of one time goes to the next.
    A page cut at the answer's bytes (`forgejo.md`, 1.3) is full: ending
    at a later time, it moves `since` to its last row's time and restarts
    at page 1, the stamps dropping the rows of that time it replays; of
    one time, the listing goes on from the row after its last at pages
    of one row, the page the rows before it number plus one, until a
    page ends at a later time, so no row a cut left is skipped. Its
    pages are as many rows as the bytes
    hold, so the listing's cost follows the changes' count and size, not
    the largest issue a repository could hold.
  - **A cut page of reviews** (`forgejo.md`, 1.1 and 1.3), in an item's
    scan or a find (7.5), is read on at the same page with `skip` the
    rows kept from it so far (`README.md`, 3.2), and the next page from
    `skip` 0 once a page ends uncut. A skip lives in memory: a stored scan
    position names whole pages alone, so a restart reads that page again
    and the stamps drop what it delivered.
  - **The mark** is the newest time seen. If an item changed at or after
    the listing began, it is instead the time of the first full page of
    one time, when there was one. The next listing replays what may have
    moved, and the stamps drop repeats.
  - **Never past the forge's time.** A page's advance and a completed listing's mark each stay at or below the `now` of the answer that moves them, and a restored mark at or below its row's restored `clock`, the latest `now` the forge answered: an item stamped later than the forge's own clock moves neither, and the listing that follows reads from the forge's time. Only the first listing's probe (`u64::MAX`: `forgejo.md`, 1.1) asks past it, and its mark is its answer's `now`.
  - **A listing that fails** keeps its place and goes again after a
    wait that grows (4.2). It reports nothing.
- **Per resource.** An item a listing reports changed, and a resource a
  hint affects, is read after `hinted`, as `Keep`. A pull request then
  rests `poll`, doubling after each quiet read to `poll_max` (capped at
  `slow`), and starts again at `poll` on a change or hint. That finds what
  moves no updated time: CI, a base. Issues and branches rest at `slow`.
  A branch, issue or pull that answers `Missing` rests at `slow`, or
  `hinted` when a hint named it: absence is an answer.
  - **A branch never there.** One that has not read present since the
    client started rests after `Missing` at the later of `slow` and
    `fallback`, whether or not its repository hears (4.1): only a push
    makes it, and a push's hint naming it, or its owner's `Pushed` for
    it, reads it at `hinted`. Once it reads present it rests as any
    branch. `fallback` 0 leaves it at `slow`.

  A branch created by someone else can therefore be seen up to `slow`
  late, `fallback` while it has never been there, or `hinted` with a
  hint. The owner knows the result of its own pushes and reads afresh
  before decisions (section 5).
  The read that ends a `slow` rest, or retries a failure, is `Slow`. A
  participating item's `Slow` read rescans its comments from 0 and rereads
  its reviews, finding an edit that moved no time; a hint naming an item
  asks for the same. Every rest and repair is stretched while its
  repository hears (4.1).
- **Comments since.** An item's read names `since` (README.md, 3.2) so the
  forge sends only the comments that can be new. It is the cached
  item's `updated` when the last read of the row ended its comments
  (`more` false) and this read goes on from it: every comment above
  `after` was made after that read's listing, so after that time. It is
  0 when the row has no cached item, while a scan goes on across reads
  (`more` was true), and for a rescan from 0. A row not participating
  reads no comment: its `after` is the largest id and its `since` the
  largest time (forgejo.md, 1.1), so the forge sends none. Nothing new is
  stored: the cached item and the scan's flags are the row's already.
- **Hints.** `Hint { repository, change, key }` names an item, a branch,
  a branch moved (with head and actor) or a commit. It affects the held
  resources it can have moved: the item, the branch, the pull requests
  with that branch as head or base, the repository's listing. A
  commit's hint moves no pull request's head or base, only its CI, which
  an owner that needs it at once reads afresh (section 5), so it affects
  no pull request: the pull's next timed read finds what it told.
  Nothing it says is stored or delivered. A branch name or key over
  `op_bytes` is refused with a fact and read for nothing.
  - **Every hint is handed over,** the owner's own echoes among them,
    which keep its repository hearing (4.1). The client reads on every
    hint it holds a resource for and drops the news its owner's writes
    cause by its echo ledger and keys (7.6); it uses a hint's pusher (its
    actor) for drift alone.
  - **No pace depends on a hint:** one lost, refused or never sent costs
    latency, never liveness.
- **Drift.** The owner says which branches a writer holds (`Writer`) and
  which commit its run pushed (`Pushed`). The client reports
  `Drift { resource, expected, observed }` when a branch's tip differs
  from the pushed head with no writer holding the branch, moves backwards
  under a writer (a `Fresh` ancestry range, `Commits { before: new, after:
  old, page: 1 }`, lists a commit: the new tip
  lacks the old; a refused ancestry read is remembered and skipped until its inputs change, as in 4.2), is moved by a hint's actor who is not a configured writer, or
  is gone (`observed` none). A forward move by a writer is its own and
  reports no news. An absent branch clears its cached tip in the stored
  live record and reports deletion drift only when that tip was present;
  a restart therefore repeats no deletion drift. A branch that reappears
  is observed under the same drift rules. Releasing a branch (`taken`
  false) reads it afresh.
- **News.** `Changed { resource, result }` carries the item with its new
  comments (a page at a time, `more`), the pull request, the new reviews,
  or the tip. A comment or review is delivered once for each id and
  revision, and again when its revision moves; one the owner wrote is not
  (section 7.6). `Changed { result: Err(Missing), .. }` is reported once
  per absence: on the first absent read, or after the resource was
  present and is now gone. Further absent reads report nothing. Absence
  is kept on the live row in memory, with no stored-format change: once
  per absence, and once more after a restart at its first absent read.
  The owner treats that notice as before the restart. Other errors
  report `Changed` and are kept or read again as 4.2 says.
- **Settling.** Forgejo answers a pull request not mergeable while it
  checks, and orders a job's end and its commit status nowhere (forgejo.md,
  section 2). Both settle within `settle`, which the client checks only
  to be above 0. It runs no window for it: the rereads within `settle`
  are the owner's fresh reads (`client.md`, section 5).

### 4.1 Hearing, and the fallback

Deliveries say what moved; timed reads catch what they miss. While a
repository's deliveries arrive, its timed reads fall back to `fallback`.

- **A repository hears** while its last hint came less than `fallback`
  ago. Every hint the client takes for a repository it keeps counts,
  whoever caused it: the owner's own echo proves that deliveries arrive
  as well as another's write does. A hint refused for its width counts
  for nothing, and a repository with no row hears nothing. Hearing is
  kept in memory alone, with no stored-format change: after a restart a
  repository hears nothing until its next hint.
- **Stretched while it hears.** Each timed read of a hearing repository
  falls due at the later of its own time and the last hint plus
  `fallback`: its listing's repeat, a resource's rest (`poll` doubling
  to `poll_max`, `slow`, a `Missing` rest), a participating item's
  repair, and a queued merge's look (7.7). A hint still reads at
  `hinted`. Neither a fresh read (section 5) nor a failure's wait (4.2)
  is stretched.
- **A missed delivery is still caught.** The read due at the last hint
  plus `fallback` reads all a delivery could have announced, so a lost
  one costs at most `fallback` of latency. Once `fallback` passes with
  no hint, the repository no longer hears and every pace is section 4's
  until its next hint. `fallback` 0 stretches nothing.
- **The owner's own polls** take the same rule from
  `Domain::poll_at(&self, limits: &Limits, now: Time, repository:
  Repository, interval: Duration) -> Time`: `now + interval`, or, while
  the repository hears, the later of that and its last hint plus
  `fallback`. It reads state and changes none.
- **The repository is the unit.** A forge sends a repository's
  deliveries of every kind through one hook (Forgejo's organisation
  hook, GitHub's App), so a hook that fails stops them alike, and that
  repository stops hearing within `fallback`. Hearing is never lent from
  one repository to another, whose hook may be another.

### 4.2 Refusals and failures

A keeping read that fails is refused for its inputs, or may pass when
asked again.

- **Refused for its inputs.** `TooLarge` answers a live resource's
  same inputs the same way: a comparison of two commits (the drift
  check's, section 4), a scan past `inbox` stamps or pages, and an
  answer past `answer_bytes` or `rows` (2.3). The live row keeps each refusal in
  memory: its phase, and for a comparison its two commits; a pull may hold
  one for each of its phases. It reports
  `Changed { result: Err(TooLarge) }` once and rests as after a read.
  While its inputs stand, a timed read skips the refused phase and goes
  on to the next one: a pull's object phase to its pull, its pull to its
  reviews when they are owed. A branch refused its comparison keeps its
  cached tip and claims neither drift nor news; a resource left with
  nothing to read rests.
- **Its inputs change** when a hint affects the resource or the listing
  reports it changed, and, for a comparison, when the branch's next read
  answers another tip, which is another pair. The next read of that
  phase then tries again. A restart forgets the refusal and tries once
  more. A read whose inputs changed while it was out keeps no refusal:
  its `TooLarge` is reported, and the next read tries again.
- **Any other failure** but `Missing` (section 4) may pass when asked
  again. It reports `Changed` each time and is read again after
  `backoff`, doubling after each failure in a row to at most `slow`, the
  backstop's pace, or `backoff` where `slow` is shorter; an answer the client
  takes sets the wait back to `backoff`, and one it refuses (`InvalidAnswer`)
  is a failure in a row. A listing
  that fails waits the same way and reports nothing, `TooLarge`
  included, since a listing's pages move as items change. These waits
  are not stretched (4.1): a failing read is not a quiet one.
- **Fresh reads keep section 5's rule:** one terminal, never kept as a
  refusal. What an owner asks again is the owner's.

## 5. Fresh reads

Before a decision depends on a fact, the owner asks for it afresh with
`Event::Read { owner, repository, read }`: any `Read` of the vocabulary
(README.md, section 3).

- **One terminal:** exactly one `Request::Read`, an answer or an error.
- **Shared.** A fresh read asked while the same read (its repository
  and `Read` alike) is queued, or is out and was sent after all of these
  the client has taken: the last write's answer for that repository,
  the last hint affecting the read's resource, and the last `Pushed`
  for its branch, joins it: one call, its cost counted once, each
  asker its own terminal, as if each had asked alone. A read sent
  before any of them is not joined, since its answer may be older than
  what the asker knows. A joined read is admitted as any read is (2.3,
  `Busy` included) and is answered with the call it joined; a failure
  is each asker's, and its retries the call's.
- **Refused at the entrance,** with no API call, `TooLarge` by section
  2.3 first, then `Busy` when the call slab is full or when the
  foreground share is spent and a keeping read is due (section 6).
- **Retries.** `Unavailable` and `Timeout` are tried again, `read_attempts`
  tries in all, after `backoff` doubling to `backoff_max` plus a draw
  below that delay, capped at `backoff_max`. A rate refusal queues the
  read again and spends no try. `NotConnected` and `NotInstalled` reached
  no forge (connection.md, section 3.5) and are final, as any other
  error is: the owner learns that the forge has no access, or that the
  repository's account has no installation accepted.
- **Settling** (section 4) is the owner's: it rereads a pull request not
  yet mergeable, or a commit status, with fresh reads within `settle`.
  The client keeps no window and treats each read as any other.

## 6. The request budget

Every call has a class. They go in this order, and in arrival order
within one: `Fresh` (a fact for a decision; a write's clock, checks and
finds), `Write` (an effect ready to be made), `Keep` (a read of changed
live resources) and `Slow` (the periodic backstop).

- **Charging.** A window starts with its first call and lasts `window`;
  at `rate` spent, nothing is sent until it ends. A call is charged 1 when
  sent and settled by the cost its answer reports less its `free`, the
  requests the forge answered `304`, which its rate does not count: 0
  refunds the 1 (in the same window), `n` above 1 debits `n - 1` more.
- **The forge's window.** An answer's `rate` says what the forge has
  left. When its `remaining` is below what the window has left, the
  window keeps that instead; at 0, all four classes are held until its
  `reset_at`, turned into the client's own time once, at the answer:
  `env.now` plus `reset_at` less `env.wall`, at most `window` on
  (../../foundation/programming-model.md, section 9). On GitHub each installation reports
  its own window, and the budget takes the least (section 11). A refund
  never lifts the window above what the forge last reported it has left.
  The last unit of a window the forge capped holds every class until the
  forge's own reset when the answer gave one, not the window's own end.
- **The reserve.** While a `Keep` or `Slow` call waits, `Fresh` and
  `Write` take at most `rate - reserve` of a window; with none waiting,
  all of it. Once that share is spent and a keeping read is due, a new
  fresh read is refused `Busy`, and `resume` sends the keeping read.
- **A rate refusal** (`RateLimited { after }`) holds all four classes
  until its reset; overlapping resets keep the later. The call is queued
  again under a new token, spending no retry, unless its resource has
  left.
- **In flight:** at most `calls`, one `Request::Call` per `resume`.

## 7. Effects: keys, write lanes and recovery

An effect is a `Write` of the vocabulary with its `Condition` (none for
most), committed by the owner as an `Entry`. The client makes it once
and says what happened.

### 7.1 Keys

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
  which a restart restores (section 8), and returns true. The same
  author again changes nothing and returns true. Another author for that
  forge, author 0, or a forge past `repositories` writers returns false
  and changes nothing: an identity that changed is setup's to resolve,
  never the client's (setup/README.md, section 3). A restored `Writer` row
  sets its forge's writer as if given; one that disagrees with the
  configured or an earlier one makes `restoration_failed()` true.

### 7.2 Entries and lanes

- **Admission.** `Make { entry }` is refused with `Outcome::Failed`, by
  the first of these that applies: `Busy` before restore ends;
  `Forbidden` where the forge has no writer yet, configured, given or
  restored; `TooLarge` over
  `op_bytes`, with an attempt but no start, an empty key, or a condition
  its write does not take (a merge takes a base, an update a head and a
  base, a review a head, the rest none); `Busy` with `entries` held. A
  new entry's number is above every held one (section 2,
  "Preconditions").
- **A lane** is a repository and an item number for a write that names an
  item (comment, review, edit, reviewers, close, reopen, merge, update),
  else the repository alone.
- **Commit order, one at a time.** An entry runs when no earlier entry of
  its lane remains. An uncertain entry holds its lane until found or
  settled, so a later write cannot overtake one that may still land.
- **Withdrawal.** `Withdraw { entry }` works only for an entry with no
  attempt: of one with an attempt, or one the client does not hold, it
  changes nothing and says nothing. A call already out ends it
  `Withdrawn` at its answer.

### 7.3 Recovery classes

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
(section 7.1). An issue, whose summary carries no provenance, settles
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
refused at any other (7.4). A head moved under an owner is the owner's to
judge.

### 7.4 One write's way

1. **Clock.** A new entry reads the forge's time (an empty listing, a
   `Fresh` read) as its start time.
2. **Check.** Only a write carrying a `Condition` (a merge, an update or
   a review) reads its pull request afresh; any other goes to step 3. A
   closed pull request fails `Closed`; another head or base fails `Stale`,
   an update's after a lost attempt too (section 7.5).
3. **Make.** In the decision that sends the call the client emits
   `Progress` with the entry's `start` (a `Position`, fixed once) and its
   `attempt`: the send, its monotonic `deadline` and its absolute
   `expires`, both `lifetime + clock_margin` after it. The owner commits
   it first (section 2.1). Each send is an attempt: a write sent again
   after a rate refusal (section 6), which the forge did not act on
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
   not so proved is not found, and so made again (section 7.5). A merge answered `Queued` is
   the queue's (7.7).
5. **A lost answer.** `Timeout` or `InvalidAnswer` on a write may have
   reached the forge: the entry counts a failure, emits `Progress` and
   `Outcome::Uncertain`, and is looked for (section 7.5). `Exists` is
   looked for too, since every write is sent under an attempt.
6. **A typed refusal** (conflict, protected branch, stale head, `Closed`
   and the like) is final, never retried blind. `Unavailable` did not
   reach the forge: it is tried again after `backoff` doubling to
   `backoff_max` plus jitter, up to `write_attempts` failures.
   `NotConnected` and `NotInstalled` reached no forge either, and fail
   the entry at once, as a typed refusal does: the forge's access, or
   the account's installation, is the owner's to restore before it
   commits the write again. A find that meets them fails as 7.5 says.
7. **Missing.** Deleting a branch already gone is made (`Set`). A find
   that meets `Missing` is a find not found; a set that meets it fails.

### 7.5 Finding again

`lifetime` bounds how long a sent write may still arrive, except for an
update: its idempotent transition rests on no lifetime (7.3). For an
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
  `Stale`, as any check does, whichever it was: `Stale` says the state it
  was decided on has gone, not that the update was not made. The owner
  reads afresh.
- **A set** (edit, reviewers, close, reopen, status) is not looked for. It
  waits out the first attempt's lifetime and is written again, so a late
  copy cannot land after the second.
- **Raced.** `Outcome::Raced { made, why }`: a known effect landed and its
  intended state cannot be confirmed, as when another hand closed the
  pull request, or a merge's pull request shows another base, head or
  merge commit.

### 7.6 Echoes

A comment or review the owner wrote comes back through the next scan, and
is not news. `Made::Commented` and `Made::Reviewed` add the new id to the
item's ledger (at most `rows`) until the scan passes it. A row is an echo
too when a configured writer wrote it and its key is an outbox entry's or
has version 1 and the owner's namespace. A keyed write waits for room in
the ledger, and its landing brings the item's next read forward to
`hinted`.

### 7.7 A merge queue

A merge answered `Queued` (README.md, section 3.3) is the queue's: it
was neither made nor refused, and it lands, or not, later.

- **Kept.** The client sets the entry's `queued`, emits `Progress` and
  `Outcome::Queued`, which, like `Uncertain`, settles nothing. The
  entry is never sent again, cannot be withdrawn, keeps no deadline (the
  queue's own timeout ejects it) and counts against `entries`.
- **Its lane.** A queued entry holds its lane until the queue decides
  (7.2's rule: a later write must not overtake a merge that may still
  land). So a comment or a review on a queued pull request waits until
  the queue decides; the owner knows this from `Outcome::Queued`.
- **Looked at.** A fresh `Read::Pull`, budgeted as a find (7.5), on each
  hint naming the pull request and at each `poll` while the entry is
  queued, stretched while its repository hears (4.1), on the timers the
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

### 7.8 A stop

An owner that stops settles its effects first (jig's
`domain/connectors.md`, 4.2; temper's `domain/root.md`, 8.2). `Settle`
tells the client to finish what it sent and to send nothing new; it is
the client's last state, and a start builds a new client.

- **Nothing new is written.** No write is sent from then on: not an
  entry's first, nor one again (after `Unavailable` or a rate refusal, a
  set past its lifetime, a remade entry). An entry not yet sent stays as
  it is, its row the owner's, for the next start; so does a write's
  call queued and not yet sent.
- **What was sent finishes.** An entry whose write this client sent and
  which has no outcome yet goes on to its outcome as ever: its answer,
  then the read that proves a review, a merge or an update (7.4, step
  4). A lost answer ends in `Uncertain`, its find left to the next
  start; a merge answered `Queued` ends there (7.7). The client starts
  no other read of its own meanwhile: no listing, no poll, no find.
- **`Settled { left }`** once every entry it sent has its outcome, at
  once if none lacked one; or at its bound, twice the sum of `lifetime` and
  `clock_margin` after `Settle` (the write's lifetime, then its proving
  read's), `left` naming each entry still without its outcome. The next
  start finds those by their attempts (section 8).
- **The owner** commits the outcomes as ever (2.1), and asks nothing new
  of the client while it settles.

## 8. Restart

After a restart the owner gives the client its rows, then has it read
before it writes.

1. **Restore.** The namespace is given first (section 2).
   `Restore { record }` for each stored row, its writers' among them,
   then `Restored { clock }`.
   A row that breaks a limit or is inconsistent (a bad page or stamps, a
   cache or echoes over bounds, a live row without its repository's, or
   the reverse) makes `restoration_failed()` true, for good: `Keep` is
   `Busy` from then on. The owner reconciles its store with its limits
   and builds a new client, which it restores again.
2. **The recovery clock** (`clock`): `Monotonic` while the saved origin
   survives, else `Wall`, when a find's remaining lifetime is the saved
   `expires` less the wall clock.
3. **Prepare.** The owner tells `Writer` for the branches its writers
   hold, sends `PauseOutbox`, and hands every stored entry not held for a
   person to `Make`. An entry with an attempt starts at its find.
4. **Read afresh.** `ReadAfresh` reads the listing of each repository
   with an issue or pull watch from its stored mark, inclusive (section
   4), and each live resource, all as `Fresh`, so what
   changed while it was down is read once. `ReadAfreshDone` ends it, once
   no watched resource or repository is left to read: after an answer, or
   after a `Keep` that drops the last one still owed (an owner that stops
   watching a resource it can never read again, such as a forge gone for
   good, does not hold the pass open).
5. **Settle.** `SettleOutbox` lets the entries run. Each with an attempt
   is found, or waits for its saved deadline (section 7.5); a queued one
   is restored queued and its pull read at once, within the same bounds
   (7.7). `OutboxDone` comes when each has been looked for once.

## 9. What it never knows

- **What an effect is for,** or which effects exist: the owner chooses
  the writes and their identities.
- **Policy:** gates, merges, landings, grants and subscribers.
- **The owner's outbox.** The client holds execution copies of the
  entries it is handed, and tasks only as numbers it echoes back.
- **What people see,** credentials, connections and webhook secrets.

## 10. Verification

- **Step tests,** in the crate, hold each rule above: the budget's
  order, reserve, refunds and rate refusal; the listing's ties and
  overlap; drift against the owner's pushes; the start and attempt saved
  before the first write; original deadlines across a restart and a
  changed clock; copied, revised and unknown rows never settling; golden
  bytes for keys; the refusals' order, a pair each for `Keep`, `Make`
  and `Read`; a write rate-refused, then sent under a new attempt; a
  `304` charged nothing; a forge's window below the budget's, and its
  reset holding every class; a writer given at runtime, given again,
  given as another author, and restored; a write meeting `NotConnected`;
  a merge queued, then merged, merged at another head, closed and
  ejected, a later comment on its pull waiting, and a queued entry
  restored; a stop that settles at once, waits for a sent merge's proving
  read, sends no write due or queued, and names an entry unfinished at
  its bound, one tick either side of it; a listing run only while an
  issue or pull watch lives, stopped with its mark cleared and started
  again with none; rests, repairs and a queued merge's looks stretched
  while a repository hears and at section 4's paces once `fallback`
  passes with no hint, a hint still read at `hinted`, and `poll_at`
  answering both; a refused comparison and an inbox scan past its cap
  reported once and read again only when their inputs change; a
  failure's wait doubling to `slow` and set back by an answer.
- **The client world** (../../foundation/testing-strategy.md, section 2.2) runs the client
  alone under a durable scripted parent that commits a decision whole
  before it releases any call, over skein's fake forge domain
  (../testing/fakes/forge.md, section 2) through small total translations. A seeded
  loop injects faults, late copies of lost writes, restarts and rate
  resets. Its stories: creations survive faults, late landings and
  restart; an uncertain write is found without a second; a created
  branch deleted before its find is held after its deadline; a
  conditional merge lands once; a queued merge settles once, landed or
  ejected; a late set reaches its final state; an update, by a merge and
  by a rebase, found if its copy lands within its lifetime, made again if
  not, its head holding its base whichever copy lands and with a copy past
  its lifetime, and failed `Timeout` when none lands in `write_attempts`;
  an
  effect committed before its forge's writer is given, refused, and one
  restored after the writer was kept, made; a
  branch name taken by another writer is reported, not overwritten;
  every fresh read finishes once; no comment or review is delivered
  twice; every table full at once, under a named scenario ceiling
  with headroom.
- **Its referee** reads no client state. It checks that every write
  released was planned by the parent and had its start and attempt saved
  first, that a keyed creation happens once on the fake, that news is
  delivered once for each id and revision, and that a seed replays, the
  diagnostics changing no decision.
- **The protocol world** joins the client, the component and the fake
  forge's protocol layer (connection.md, section 10). The owner's
  connector worlds and cost ceilings are the owner's.

## 11. Open questions

- **A window per installation.** GitHub reports each installation's
  rate apart (connection.md, section 4.4), and the budget is one:
  it takes the least `remaining` any answer reports, so one spent
  installation slows the calls of the others. Default: the least, since
  one owner seldom spends one installation's 5000 an hour; a budget per
  account would take the repository's account in each call.
- **A keyed class in use.** No write is `Keyed`, Forgejo's or GitHub's.
  A dialect with a deduplicating key would move a creation there.
  Default: every write keeps the class section 7.3 gives it, and `Keyed`
  stays unused.
- **Where `recovery` lives.** The classes sit in the client, one per
  write whatever the dialect. A dialect may carry its own (README.md,
  section 4), the client then taking the class as a value. Default: the
  classes stay in the client, Forgejo and GitHub sharing them.
- **An update's lifetime.** A forge does an update's git work inside the
  call and its real time is not known, so `lifetime` may be too short for
  one. Too short costs a repeated update and CI on a replaced head; a late
  copy may also cause its owner's drift hold, and a rebase may drop a later
  push (7.3). Default: `lifetime` is kept until update times and those costs
  are measured.
- **A quiet repository's hook.** Hearing is proved by deliveries alone
  (4.1), so a repository with no delivery for `fallback` polls at
  section 4's paces though its hook may be well. A heartbeat, such as a
  forge's test delivery, would prove the hook at a write per `fallback`.
  Default: deliveries alone; a quiet watch polls as before.
- **Hearing lent within a hook.** One Forgejo organisation hook serves
  every repository of its organisation, so one repository's deliveries
  prove its siblings' too. Default: each repository hears for itself
  (4.1), which never stretches a watch whose hook is not known to work.
