# A landing's members and the asks it hands them

Provisional, 2026-11-25. Who is in a landing, kept once: the landing's
row holds its members, a change's landing is derived from the live
rows, and what a landing hands a member waits in a queue on the change
until the change takes it. It details `landing.md`, 4.2 (*It ends*),
4.3 (*Its link goes as it leaves*), 12 and 15, and carries
`../draft/landing-liveness.md`, 3 and 8. Which changes form a landing
stays `landing-turn.md`'s and `landing-forming.md`'s; what a member's
change does with an ask stays `landing.md`, section 3.

## 1. In one page

- **The row holds the members** (section 2): each member in the stack's
  order, in it, landed or left, as `landing.md`, section 12 stores it.
  A change stores no landing number.
- **A change's landing is derived** from the live rows by an index the
  forge keeps unstored: built from the rows at every start, and changed
  in the decision that changes a row. A landing that ends takes its
  members with it, so no link can go stale.
- **Two live landings claiming one change** (2.2): the older keeps it,
  and the newer holds at start with words naming the change.
- **A handed ask is a queue on the change** (section 3): a prepare, a
  judgement, a resolve, a repair or the word that it landed, each with
  its landing, its round, its kind and a number of its own; at most one
  of each kind per landing; `Landed` first and never dropped.
- **Taken by its number:** the change's step names the ask it took, in
  the decision that records what it did with it.
- **Only a superseded command goes untaken** (3.4): a prepare or a
  judgement that its change can no longer answer is dropped with words
  to the change's goal once the change's readiness is checked again.
- **The previous build's links and asks are converted** at the first
  start of this one (section 4), from the rows' slots and the changes,
  with a log line for a stored link that disagrees with the rows.

## 2. The row holds its members

### 2.1 The index

- **What it maps:** each change that a live landing (its phase neither
  landed nor cancelled) holds as a member that hasn't left, to that
  landing. A member that landed stays in its landing's index entry
  until its landing ends, as its link did.
- **One function of the rows:** a change's landing is the oldest live
  landing, by task number, whose row holds it so. Forming takes only
  changes in no live landing, so a second claim never forms; one the
  store already holds is 2.2's.
- **Built at start** from the restored rows, once every forge row and
  both parts of each change are read, before the link pass judges
  links (`landing-forming.md`, 4).
- **Kept in step:** every decision that saves, admits or erases a
  landing's row changes that landing's entries in the same decision, so
  no reader sees a row and an index that disagree.
- **Read by** the change's step (as a fact, its landing), forming, a
  landing's wake of its members, the views (`landing.md`, section 14;
  `forge/change/panel.md`, 1) and the root's routing, each by the change's
  task, never by scanning every change.

### 2.2 A change two landings claim

- **Checked at start,** before the index is built: a change that two
  or more live rows hold as a member that hasn't left is the oldest's.
  Each newer one holds before it steps, `landing L: held: change C is a
  member of landing O too, which keeps it` (`landing.md`, 4.2's news),
  `O` the oldest.
- **Its release** holds it again with the same words while `O` still
  holds `C`. Cancelling it ends it as any cancel does; its members are
  in no landing but `O`'s.
- No running decision makes a claim: it is a store an earlier build
  wrote.

## 3. The asks a landing hands

### 3.1 An ask

An ask a landing hands a member's change, kept on the change's row:

- **its number,** from the change's own count of asks handed to it,
  which the row keeps; never reused;
- **its landing,** the task that handed it, and **its round,** that
  landing's round as it handed it, counted from 1;
- **its ask,** one of a change's landing's asks (`landing.md`, 12):
  `judge`, `resolve`, `repair`, `landed`, and a single landing's
  `prepare`. Its kind is the ask's.

### 3.2 The queue

- **At most one of each kind per landing:** a landing handing an ask of
  a kind its change already holds from it removes the older one; the
  newer joins the queue's end with its own number.
- **Bounded:** at most 16 asks; one per kind per landing, and a change
  joins a landing only with an empty queue (`landing-turn.md`, 2), so
  only the conversion (section 4) can hold more than one landing's.
- **Its head,** the ask a step is given: a `landed` first, whatever its
  place; otherwise the lowest number.
- **Handed in the landing's decision:** the decision that hands an ask
  saves the change's row with it, and wakes the change, as a member's
  ask woke it before. A change that doesn't step is waiting on its own
  bound (`forge/change/stalls.md`), so an ask has no clock of its own.

### 3.3 Taking an ask

- **By its number:** the change's step is given the queue's head and
  names the number of the ask it took. The decision that records the
  step removes that ask and no other, so an ask handed while the step
  was decided stays queued.
- **When it is taken,** as the change acts on it:
  - `landed`, as the change records itself landed at its prefix, before
    any other ask or its ordinary step, held or not;
  - `judge`, as it begins judging at the target (its state keeps the
    target);
  - `prepare`, as it asks its update (its state keeps both heads);
  - `repair` and `resolve`, as their delegate is made.
- **A single landing's `asked` fact** (`landing.md`, 12) reads the
  member's change as asked while it holds an ask of that landing
  queued, prepares, or judges at that landing's target.

### 3.4 What is dropped

- **A prepare or a judgement its change can't answer** is dropped, and
  nothing else is: a `landed` never; a `repair` or a `resolve` is
  answered by the change's repair or resolution.
  - a `prepare` whose head is not the change's head;
  - a `judge` whose landing the change is in no longer by the index:
    its head moved and it left (`landing.md`, 4.3), or its landing
    ended.
- **When:** in the change's step that finds it ready at its head
  (`landing.md`, section 3), so a held change stays held with its
  queue.
- **With words** to the change's goal, as its landing's news reach it
  (`landing.md`, 4.2), commits as that news prints them:
  - `change C: prepare for HEAD dropped: the head is now NOW`;
  - `change C: judgement of landing L, round N, dropped: the change is
    in that landing no longer`.
- **After it,** the change goes on as with no ask: ready, it joins its
  train's next landing (`landing-turn.md`, 2).

### 3.5 As a landing leaves its members

- **A member that leaves** keeps what the leave hands it (`landing.md`,
  4.3: a resolve or a repair), queued on its change; its index entry
  goes in the same decision.
- **A landing that ends,** landed, cancelled or ended unmade
  (`landing-forming.md`, 4), removes its members' entries, and removes
  from their queues the `prepare` and `judge` asks of its own still
  queued: no landing is left to answer. Its `landed`, `resolve` and
  `repair` asks stay until taken.

## 4. Across a restart and a build

### 4.1 Every start

The index is built from the rows (2.1), after the claim check (2.2).
The queues are restored with the changes' rows. Nothing else is
repaired: there is no link to go stale.

### 4.2 The previous build's links and asks

The build before this one kept a change's landing number and one
handed-over ask on the change, and an ask slot per member on the
landing's row; and a single landing (`landing.md`, 12) owed a member it
had recorded landed a `landed` from its prefix, which no slot held.
Their stored fields stay readable until the reference deployment has
run this build (`../draft/landing-liveness.md`, 8). In the start's link
pass, after the index is built and before the unmade pass
(`landing-forming.md`, 4), a written conversion takes them:

- **From each row,** live or ended, for each member whose change the
  forge holds: its slot's ask, with the row's landing and round; for a
  single landing, a `landed` at the prefix of each member it recorded
  landed; and, for a live row's member in it by the index whose gate's
  proof failed at its candidate, its change's last head that candidate
  and its slot empty, a `repair` for that gate at that candidate, as
  the previous build derived it.
- **From each change,** its handed-over ask, its landing the judgement's
  own where it is one, else the newest row holding the change as a
  member, else none (landing 0, round 0, which no landing has).
- **Into the change's queue,** numbered in this order: every `landed`
  first, then the rest by landing, oldest first. One per kind and
  landing: a row's ask is kept over the change's when both are of one
  kind, and of two `landed`, the row's. Past the queue's bound, the
  oldest landing's asks other than `landed` are dropped, each logged.
- **The stored landing number** is read only to log one that names
  another landing than the index:

  ```
  BINARY serve: warning: change C named landing L; by the landings' rows it is in landing M
  BINARY serve: warning: change C named landing L; by the landings' rows it is in no landing
  ```

- **Its writes are the start's,** in the link pass's commit: each row
  and change it converted written with the old fields empty, so a
  second start finds nothing to convert.
- **Why not the store's written job:** a store job converts one value
  type's values alone (jig's `protocol/store/jobs.md`, 4), and a
  change's asks are read from the landings' rows as well as the change.
- **It goes** once the reference deployment's store has run it, its
  old fields then given tombstones (`../draft/landing-liveness.md`, 8).

## 5. In the store

- **A change's hot part** (`../protocol/store.md`, 2) gains its queue,
  `asks`, at most 16, each its number (u32), landing (u64), round (u16)
  and ask, and `handed`, its count of asks handed (u32), both with
  defaults: empty, and 0.
- **The previous build's fields** keep their places and layouts: the
  change's `landing` and `landing_ask`, and the landing row's `asks`
  slots, written empty by this build (4.2).
- **The landing's own hold** gains a cause, `claimed`, naming the change
  (2.2), a variant after every kept one.
- Each is a change a plan derives (`../protocol/store-versions.md`,
  1): no version moves and nothing is patched.

## 6. Testing

Focused and fuzzy suites only.

- **The index:** after any landing's step, leave, finish, cancel or
  unmade end, each change's derived landing is the oldest live row
  holding it, not left; after any ending, no change's landing is the
  ended one.
- **The claim:** a store with two live rows holding one change starts
  with the change in the older and the newer held in 2.2's words; a
  release holds it again.
- **The queue:** one per kind and landing, a newer replacing an older;
  `landed` first; an ask handed while a step was decided survives the
  step; each kind taken as 3.3 says, once, across a restart between
  handing and taking.
- **A dropped ask:** a prepare for a moved head and a judgement for a
  landing the change has left are dropped with 3.4's words, and the
  change joins the next landing; a held change keeps its queue.
- **The conversion** starts from stores the previous build wrote, at
  its layout: stale links, asks in both places, a single landing's
  recorded prefix and a failed member's derived repair; it opens with
  none stale, every ask in its queue once in 4.2's order, a log line
  for each disagreeing link, and a second start converting nothing.

## 7. Later

- An ask's own bound, should a change that steps never take one.
