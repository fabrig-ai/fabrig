# The forge client

Provisional, 2026-10-20.

`skein-forge-client` keeps a bounded view of a forge for the domain that
owns it, and makes each write the owner commits at most once. It is
`no_std`, lib-only, and depends on `skein-lib` and on `skein-forge`'s
vocabulary (vocabulary.md, section 1). It makes no HTTP call and knows no
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
- **Each effect is made at most once** (`client-effects.md`, section 1). Its key, lane and
  recovery class decide how a lost answer is found. The attempt is saved
  before the write starts, so the search survives a restart (`client.md`, section 8).
- **One writer per forge,** configured or given at runtime once, and
  kept as a row; another author is refused (`client-effects.md`, section 1.1).
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
  writers (`client-effects.md`, section 1.1). A forge's writer may come later, at runtime, by
  `writer` (`client-effects.md`, 1.1). **The namespace is given before any restored
  entry or live resource is handed over** (`client.md`, section 8). An owner that
  learns it after construction sets it with `bind_deployment(id, limits)`,
  a 16-byte id, before that restore. It returns false, changing nothing,
  once the client has taken any `Restore`, `Keep`, `Make` or `Remake`,
  with no configuration, or when the result breaks `client-effects.md`, section 1.1.
- **Calls are the owner's to run.** `Request::Call` carries a token, a
  repository and an `Op` (vocabulary.md, section 1); the owner returns exactly
  one `Event::Answered { call, cost, free, rate, result }` per token, a
  cost of 0 included, with the component's `free` and `rate`
  (connection-calls.md, section 1.1).
- **The client keeps no store.** It emits `Save` of a `Stored` row (`Live`,
  `Repository` or `Writer`) and `Erase` of its `Key`. The owner commits an outbox
  `Entry` and hands it over with `Make`; the client returns `Progress`
  and, when the entry settles, an `Outcome`.
- **Preconditions,** asserted and never answered, since breaking one is
  the owner's bug: a `Make` whose entry's number is not above every
  held one (`client-effects.md`, 1.2); a `Restore` of a row already restored, or after
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

- **The measure** is the vocabulary's (vocabulary.md, section 1.7), which
  `skein-forge`'s `measure::op` and `measure::answer` give. The owner
  bounds what it stores of the client's by the same widths, with
  `skein-forge-client`'s `measure::stored(&Stored)` and
  `measure::effect(&Effect)`: a comment, a review or an echo a live row
  keeps counts 16, a label 16 and its length, a reviewer 8, and every
  string its length.
- **An operation is admitted** when its bytes are at most
  `op_bytes`, its pages start at 1, a file's path is not empty and a
  `max_bytes` is from 1 to `answer_bytes`; else it is `TooLarge`, with no call: `OpBytes` with the operation's bytes,
  or `AnswerBytes` with the `max_bytes` asked, both exact (vocabulary.md, 1.4).
- **An answer is checked** against its call before anyone takes it, by
  vocabulary.md, section 1.7's table. Past `answer_bytes`, or with a list past `rows` (every list but `Branches`' names), it is
  `TooLarge` for a read, naming the bound it passed with the answer's exact
  measure, `Rows` (its longest list's rows) before `AnswerBytes` (its bytes)
  where it passes both; for a write it is `InvalidAnswer`: a lost answer, since the write may have been done (`client-effects.md`, 1.4, step 5). Any other
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
  each), cached values, pushed head and echo ledger (`client-effects.md`, section 1.6).
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
    scan or a find (`client-effects.md`, 1.5), is read on at the same page with `skip` the
    rows kept from it so far (`vocabulary.md`, 1.2), and the next page from
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
- **Comments since.** An item's read names `since` (vocabulary.md, 1.2) so the
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
    cause by its echo ledger and keys (`client-effects.md`, 1.6); it uses a hint's pusher (its
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
- **A fence.** The owner may fence a resource it watches (`Fence {
  resource }`): the answer of a keep read of that resource sent before
  the fence reports nothing when it comes, neither `Changed` nor
  `Drift`, and its kept state is not moved by it; the resource is read
  afresh, at `hinted`. An owner fences what a decision of its must not
  see from before it, such as a release of a held task's work. A fence
  lives in memory alone: a restart sends no read from before it.
- **News.** `Changed { resource, result }` carries the item with its new
  comments (a page at a time, `more`), the pull request, the new reviews,
  or the tip. A comment or review is delivered once for each id and
  revision, and again when its revision moves; one the owner wrote is not
  (`client-effects.md`, section 1.6). `Changed { result: Err(Missing), .. }` is reported once
  per absence: on the first absent read, or after the resource was
  present and is now gone. Further absent reads report nothing. Absence
  is kept on the live row in memory, with no stored-format change: once
  per absence, and once more after a restart at its first absent read.
  The owner treats that notice as before the restart. Other errors
  report `Changed` and are kept or read again as 4.2 says.
- **A pull request's size is not news.** A pull request read that
  differs from the kept one only in its `size` (vocabulary.md, section 1.6)
  reports nothing, and the kept pull takes the read's size. The size
  moves only with the head or the base the pull request names, which
  are news themselves, and an owner may keep no size, so a restart
  would otherwise report every pull request again. Each `Changed` for a
  pull request carries the size its read gave.
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
  repair, and a queued merge's look (`client-effects.md`, 1.7). A hint still reads at
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
(vocabulary.md, section 1).

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
  The client keeps no refused read: asking it again, and in what order
  beside the owner's other refused reads, is the owner's. A read the
  owner asks before `resume`, once a place is reclaimed, and the client
  admits takes that place ahead of the keeping and slow reads `resume`
  would start, so an
  owner that asks its waiting reads first gives them the freed call
  (temper's `domain/forge/keeping-up.md`, 3).
- **Its holders.** The client answers, at any time and changing
  nothing, the calls that hold the slab's places, oldest admitted
  first, each with its owner, its repository and what it reads, or that
  it is a write, never the write's content, so that
  an owner can say who holds a full slab (temper's `protocol/service-process.md`,
  1.3). It keeps the order each call was admitted in, a number, not a
  time.
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

Moved to [`client-effects.md`](client-effects.md), section 1.

## 8. Restart

After a restart the owner gives the client its rows, then, once its
engine has opened, has it read afresh and settle together: the owner
is catching up (temper's `domain/forge/catching-up.md`). Neither half
waits for the other. The reads wait on no write's outcome: an entry
holds a call slot only while its call is out, which ends on its own, so
a `Busy` while they run passes as calls end, and a read's retries
follow its own timers. A write waits on no read but its own. The owner
keeps its own reads of step 4 to reads too: a write it finds it needs
while it reads (a resource found missing that it must make) is a write
like any, in the owner's own order, and holds no read of the step open.

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
   hold, and hands every stored entry not held for a person to `Make`,
   in number order and before any entry made since the restart, as it
   begins catching up; until then it hands none, so nothing is sent.
   An entry with an attempt starts at its find. There is no pause: an
   entry made since the restart is admitted and sent as at any time,
   behind every earlier entry on its lane (`client-effects.md`, 1.2), so a write on a
   restored entry's lane waits for it, and a write on another lane goes
   ahead.
4. **Read afresh.** `ReadAfresh` reads the listing of each repository
   with an issue or pull watch from its stored mark, inclusive (section
   4), and each live resource, all as `Fresh`, so what
   changed while it was down is read once. `ReadAfreshDone` ends it, once
   no watched resource or repository is left to read: after an answer, or
   after a `Keep` that drops the last one still owed (an owner that stops
   watching a resource it can never read again, such as a forge gone for
   good, does not hold the pass open). It counts the pass's reads alone,
   never an entry's find, a queued merge's look or an echo.
5. **Settle.** `SettleOutbox`, given with `ReadAfresh`, starts the
   settle beside the pass. Each restored entry with an attempt is found,
   or waits for its saved deadline (`client-effects.md`, section 1.5); a queued one is
   restored queued and its pull read at once, within the same bounds
   (`client-effects.md`, 1.7). A restored entry's first find that finds nothing, the entry
   going on to wait for its deadline or to be made again, emits
   `Outcome::Uncertain` once more: its owner's word that this entry has
   had its first look; a find that finds it ends in its outcome, and an
   entry written by this run never emits it twice. `OutboxDone` comes
   when each restored entry has been looked for once, whether or not the
   pass has ended.
6. **What it still owes,** per forge, for its owner's catching-up state:
   `fresh_owed(forge)`, the pass's watched resources and listings not
   yet read, and `finds_owed(forge)`, the restored entries with an
   attempt not yet looked for once; both zero outside a restart. Per
   repository, `fresh_owed_in(repository)`: the pass's reads still owed
   on that repository's listing and live resources, zero outside a
   restart, so its owner can let a repository's decisions go once its
   own reads are in.

**What it still owes,** for an owner that reports a restart that runs
long (temper's `protocol/service-process.md`, 1.3), read at any time and changing
nothing: during step 4, each watched resource and each repository's
listing whose fresh read is still owed, in its table's order; during
step 5, each entry not yet looked for, by number. A `Settle` ends both
steps' waits (`client-effects.md`, 1.8): what they still owed is the next start's.

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
  rate apart (connection-calls.md, section 1.4), and the budget is one:
  it takes the least `remaining` any answer reports, so one spent
  installation slows the calls of the others. Default: the least, since
  one owner seldom spends one installation's 5000 an hour; a budget per
  account would take the repository's account in each call.
- **A keyed class in use.** No write is `Keyed`, Forgejo's or GitHub's.
  A dialect with a deduplicating key would move a creation there.
  Default: every write keeps the class `client-effects.md`, section 1.3 gives it, and `Keyed`
  stays unused.
- **Where `recovery` lives.** The classes sit in the client, one per
  write whatever the dialect. A dialect may carry its own (dialects.md,
  section 1), the client then taking the class as a value. Default: the
  classes stay in the client, Forgejo and GitHub sharing them.
- **An update's lifetime.** A forge does an update's git work inside the
  call and its real time is not known, so `lifetime` may be too short for
  one. Too short costs a repeated update and CI on a replaced head; a late
  copy may also cause its owner's drift hold, and a rebase may drop a later
  push (`client-effects.md`, 1.3). Default: `lifetime` is kept until update times and those costs
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
