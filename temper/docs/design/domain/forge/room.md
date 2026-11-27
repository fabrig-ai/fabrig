# The outbox's room

Provisional, 2026-11-16, from `../../draft/archive/outbox-room-wait.md`. How the
forge's outbox, bounded by `forge.entries` (`effects.md`, section 1),
gives its room to the writes that ask for it: a write that finds no room
waits for it in one line, first come first served, and a hold stays only
for a write that can never fit. Goal branches' makings waited so first
(`branches.md`, 1); a change's effects, a landing's writes and release
cleanup wait the same way, in the same line.

## 1. In one page

- **Room is the bound less what holds it:** the entries in the outbox
  and the reservations, each room promised to a write a check admitted
  whose entry is not yet kept (section 2).
- **An entry leaves with its write's outcome,** though its writer still
  follows what it made, and an ended landing's writes leave as they
  settle (section 2).
- **One line, by place:** a write that finds no room takes the line's
  last place and waits, holding no entry number and polling nothing.
  Room that frees goes to the first waiter by place, and a newcomer
  never overtakes a waiter (section 3).
- **Asked, it decides again:** the waiter asked takes a fresh entry
  number, reads its facts afresh, is admitted by the core's authority
  again, and journals its entry before the client sends it, as every
  write does (section 3).
- **A wait is no failure:** it counts against no allowance and no
  stall, keeps its place across a restart, and a cancel ends it; it is
  bounded by its holders' owners, and held only once they have all
  ended (section 4).
- **Agent calls do not wait:** a call that finds no room is answered
  `Unavailable` at once, and the room its check finds is its own until
  its entry is kept (section 5).
- **A hold only for what can never fit** (section 6).
- **A waiting change says so** on its panel (section 7).

## 2. Room

- **Free room** is `forge.entries` less the entries in the outbox and
  the reservations. A reservation is room promised to one write:
  - a waiter's, from its ask until its entry is kept, its turn ends
    without one, or it leaves the line (section 3);
  - an agent call's, from the check that admits it until its entry is
    kept or its flight ends without one (section 5);
  - a goal projection's, from the step that builds its write until its
    entry is kept or its effect ends without one (`issues.md`).
- **One check for every writer:** a change's effect, a landing's write,
  release cleanup, a goal branch's making, an agent call, a goal
  projection and a verification tier's write each take room only where
  free room is above zero and no waiter that could be asked waits ahead
  of it. A write that holds a reservation takes its own. A write that is
  not in the line is behind every waiter in it. An entry already in the
  outbox takes no new room: one made again after a hold, committed
  again after an outage or a restart, or saved with its progress.
- **Room frees** as an entry leaves the outbox, by its write's outcome,
  its withdrawal or its settling as not made, and as a reservation ends
  without an entry.
- **An entry lives as long as its write, no longer:** it leaves at its
  write's outcome even where its writer goes on following what the
  write made, as a landing follows its gate's run; the writer keeps what
  following needs on its own row (`../landing-proof.md`, 2.1). A landing that
  ends, landed or cancelled, withdraws each of its entries not yet sent,
  and each in flight leaves at its outcome, settled as that outcome
  says, though its landing has ended (`../landing.md`, 4.2). So an
  ended landing holds no room once its writes settle, and its row
  yields its place among the landings (`../landing-forming.md`, 2.3).

### 2.1 A full outbox names its holders

From `../../draft/reconciling-start.md`, 5: the 11-20 start waited on an
outbox whose eight places cancelled landings held, and no line said so.

- **Full** while free room is zero. The forge keeps the time it filled,
  and forgets it once a place frees.
- **Its holders, oldest first:** its entries by number, which follow
  commit order, then its reservations in the order they were made; each
  named by its entry and task or by whom its room is promised to, in the
  words of `../../protocol/service-process.md`, 1.3, with the room line's waiters
  counted (`room.md`, section 3).
- **Past the report bound,** and again at each doubling of how long it
  has been full, the forge reports them, the root's fact the service
  writes; after such a report, one more when a place frees
  (`../../protocol/service-process.md`, 1.3). The forges' document carries it
  while it lasts, past the bound (`../../client/forge/deployment.md`,
  *Its load*).
- **Read admission** is reported the same way, from the client's calls
  holding its places, oldest admitted first (skein's `forge/client.md`,
  5, *Its holders*).
- **A report changes nothing:** no holder is released, no waiter moved,
  and the figure stays the user's (section 6).

### 2.2 Each use checks its holder

From `../../draft/misbehaving-agents.md`, 3.2. The room is held by
records, each its owner's, and the room taken is the sum of their
counts, never above `forge.entries` at any step. Each step that takes,
moves or gives back room checks the holder it uses
(skein/docs/foundation/programming-model.md, 10.4); a write that finds
no room still waits, or is answered `Unavailable`, as a value (sections
3 and 5).

- **The holders:** each entry in the outbox, under its number with its
  write's task; each asked waiter's reservation, by its waiter; each
  agent call's, by its flight; each goal projection's, by its entry's
  number. A full outbox's report reads them (2.1) and holds none.
- **A new entry takes one place.** It is kept only where its write's
  check found free room in that step, or into its own reservation,
  which the same step gives back first. Keeping it asserts that its
  number is new to the outbox and that the outbox had its place, naming
  the entry and its task. An entry already in the outbox takes none
  (section 2): keeping it again asserts that it is the same task's
  write.
- **A reservation is made once and used by its holder.** An ask asserts
  that its waiter is not asked already, a call's reservation that its
  flight holds none, a projection's that its number holds none. An
  entry kept into one, and one that ends without an entry, assert that
  the reservation was held, naming its holder and its owner. A call or
  a projection whose reservation is not held never falls back to the
  open room. An asked waiter's check finds its own reservation and needs
  no other room.
- **An entry leaves once, by its own write.** Its outcome, its
  withdrawal or its settling as not made asserts that the entry is in
  the outbox under its write's task. An outcome for an entry the outbox
  no longer holds, reported again by the client or late from an outage,
  is a value, dropped as now.
- **Each holder against its owner.** At each use, the holder's owner is
  in a state that holds it: an asked waiter's owner keeps its place in
  the line (section 3); an agent call's or a goal projection's
  reservation has its effect staged in the root (section 5, `issues.md`);
  an entry's owner is live, or has ended with the entry sent (section
  2). So the step that ends an owner (a change, a landing, release
  cleanup, a goal branch's making, a call's or a projection's staged
  effect, a verification tier's write) gives back its place, its
  reservation and its unsent entries, and asserts that it still holds
  only sent entries, each marked as its ended owner's, which leave as
  they settle.
- **Its sums are exact:** free room and the room reserved are counted by
  checked arithmetic, never saturated, since every take above asserts
  its place.

## 3. The line

- **Its waiters:** a change's effect (`change/README.md`, 2), a
  landing's write (`../landing.md`, section 11), release cleanup's
  comment, close and branch deletion (`effects.md`, section 1), and a
  goal branch's making (`branches.md`, 1). Each joins the line only once
  what refuses it for good has passed (`room.md`, section 6).
- **A place:** a write that finds no room takes the place one past the
  greatest in the line, 1 in an empty line, and keeps it while it waits;
  one that meets no room again waits again in the same place. A change,
  a landing and release cleanup keep their place on their record, its
  `room`, saved as the wait begins; a goal branch's making keeps its
  place in memory, as it keeps the making itself (`branches.md`, 1).
- **It holds no entry number:** the number its turn was given is given
  up as it waits, since a number kept and not committed stops the
  client's handoff of every later entry (`branches.md`, 1). Nothing
  polls: its procedure waits with no timer, woken by its ask, or by what
  wakes it now (a fact that changed, a release, a restart), which reads
  afresh and waits again where the room is still not its.
- **Asked in order:** as room frees, the forge asks waiters while free
  room is above zero, one per unit of room, each the first by place that
  is not asked and whose task is not held. Each ask is a reservation
  (section 2). A change or a landing is asked by its procedure's wake, a
  release by `ContinueRelease`, a making by `ContinueGoalBranch`; each
  takes a fresh entry number from the root as its turn begins, so
  numbers follow the order of the asks.
- **Asked, it decides again:** the waiter asked reads its facts afresh,
  decides again, and its effect is admitted by the core's authority
  again; the entry it then keeps is journalled before it is sent, as
  every entry is (`effects.md`, section 1). Its decision may differ from
  the one that waited: a change that now needs an update rather than its
  merge sends the update. A turn that keeps an entry leaves the line, its
  reservation taken by the entry; a turn that decides no write leaves
  the line, its reservation freed; a turn that finds no room again,
  its reservation lost to a restart, waits again in its place.
- **Held, passed over:** a waiter whose task is held keeps its place and
  is not asked; one held after its ask gives its reservation back, and
  the next waiter is asked. Released, it steps as any released task
  does, and is first again by its place at the next room.
- **Nothing of its decision is kept but its place:** a change or a
  landing that waits saves its place alone; its policy state stays the
  one its decision started from, so its next step decides again from
  there, and nothing records a write asked for that was never kept.
  Release cleanup saves its place with no pending entry.

## 4. A wait is no failure

- **No allowance, no stall:** a wait spends none of a change's or a
  landing's allowances (repairs, resolutions, updates), and its time
  counts toward no stall: a change holding a place is never held
  `Stalled` for the time it waits behind live writers.
- **Its bound is its holders' owners'** (2026-11-24,
  `../../draft/bounded-waits.md`, 4.1): while an entry or a reservation
  holding the room is a live task's, the waiter waits behind that task,
  whose own waits bound it, so a stall holds where it is. Once every
  holder's owner has ended, as a cancelled landing's write still in
  flight outlives it (P124), nobody's wait ends the room's, so the waiter
  takes the room figure, `waits.room_ms`, from the latest of those ends
  and is held there, naming the room, the entry and its ended owner
  (`waits.md`, 7). Its record names the entries holding the room, as a
  full outbox's report names its holders (2.1).
- **Restart:** the places restored with their records are the line, a
  place on the record of a task that has ended dropped; makings rejoin
  it behind them as they meet no room. No ask survives a restart: from
  the beginning of the forge's catching up, at the engine's opening
  (`catching-up.md`, 3), the forge asks again from the first place as
  room allows, the restored entries counting against the room. A restored change
  steps as every restored change does, and waits again in its place
  where the room is not its.
- **Cancel:** a cancelled change or landing leaves the line as it ends,
  an ask's reservation given back. Release cleanup keeps its place
  through a cancel that replaces a closing's ending, since its write is
  still to be made. A forge removed takes its waiters out of the line
  (`adoption-connection.md`, 1).

## 5. Agent calls answer at once

Decided by the user, 2026-11-16.

- **No room, `Unavailable`:** an agent's forge call (`effect_forge`:
  an issue, a comment or a status) whose description finds no free
  room, or finds a waiter that could be asked, is answered `Unavailable`
  at once, with no hold and no person; its agent may try again. A
  holder's acceptance of a proposed effect that finds none is answered
  `Wait`, its proposal kept, as jig's `describe_busy` does now.
- **Why it does not wait:** a call's entry number is jig's, given as the
  core admits the effect, and the client takes new entries in rising
  order only. A call waiting in the line would need jig to give a
  waiting effect a new number, a change to jig's stored call records,
  for a write its agent can simply ask again. The only hold a call met
  on a full outbox came from the room its check found being taken before
  its entry was kept, and the reservation below closes that.
- **The check respects the line:** a call is behind every waiter, so its
  check finds room only where no waiter that could be asked waits.
- **Its room is reserved** from the check until its entry is kept, or
  its flight ends without one: dropped, kept as a proposal, or refused
  by the core. The entry kept takes its reserved room and never meets a
  full outbox. jig's call records are unchanged.

## 6. What still refuses

- **A write that can never be sent** refuses as before, judged before
  room, so a full outbox never hides it and it takes no place: a
  repository not adopted, adopted to be read, or whose adoption does not
  take the write's kind; an entry number already in the outbox; and a
  write over `forge.client.op_bytes`, measured as the client measures a
  new entry (skein's `forge/client-effects.md`, 1.2). Each ends as its writer
  ends such a write now.
- **A full outbox refuses no one:** the limit's refusal for
  `forge.entries`, and the hold its words named (raise the limit,
  restart, release), are no longer reached by a write. The figure stays
  the user's (`../../draft/archive/outbox-room-wait.md`, 2).

## 7. A waiting change's panel

- **Its wait is typed:** while a change holds a place, its panel carries
  the wait `OutboxRoom` (`change/panel.md`, 1), and the client says
  `Waiting for room in the forge outbox.` beneath its step
  (`../../client/forge/README.md`, 2.1). Admitted, or leaving the line,
  its next save clears it. A held change shows its hold alone. The
  public projection of a change carries no wait
  (`../../client/forge/README.md`, 9.4).

## 8. Open questions

- **Writes outside the line:** a goal projection waits its policy's
  interval (`issues.md`) and a verification tier's write waits on its
  task's own wakes, each behind every waiter by the check in section 2.
  Under long pressure they come last; neither holds. Whether either
  joins the line is open.
- **A landing's or a release's wait on a page:** only a change's panel
  says it waits.
- **Agent calls in the line,** with jig renumbering a waiting effect:
  set aside by the user for now (section 5).
