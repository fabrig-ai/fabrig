# The outbox's room

Provisional, 2026-11-16, from `../../draft/outbox-room-wait.md`. How the
forge's outbox, bounded by `forge.entries` (`README.md`, section 4),
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
  stall, keeps its place across a restart, and a cancel ends it
  (section 4).
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
  following needs on its own row (`../landing.md`, 7.1). A landing that
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
  words of `../../protocol/README.md`, 6.3, with the room line's waiters
  counted (section 3).
- **Past the report bound,** and again at each doubling of how long it
  has been full, the forge reports them, the root's fact the service
  writes; after such a report, one more when a place frees
  (`../../protocol/README.md`, 6.3). The forges' document carries it
  while it lasts, past the bound (`../../client/forge/deployment.md`,
  *Its load*).
- **Read admission** is reported the same way, from the client's calls
  holding its places, oldest admitted first (skein's `forge/client.md`,
  5, *Its holders*).
- **A report changes nothing:** no holder is released, no waiter moved,
  and the figure stays the user's (section 6).

## 3. The line

- **Its waiters:** a change's effect (`change/README.md`, 2), a
  landing's write (`../landing.md`, section 11), release cleanup's
  comment, close and branch deletion (`README.md`, section 4), and a
  goal branch's making (`branches.md`, 1). Each joins the line only once
  what refuses it for good has passed (section 6).
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
  every entry is (`README.md`, section 4). Its decision may differ from
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
  `Stalled` for the time it waits.
- **Restart:** the places restored with their records are the line, a
  place on the record of a task that has ended dropped; makings rejoin
  it behind them as they meet no room. No ask survives a restart: once
  the restored outbox has settled (`README.md`, section 9), the forge
  asks again from the first place as room allows. A restored change
  steps as every restored change does, and waits again in its place
  where the room is not its.
- **Cancel:** a cancelled change or landing leaves the line as it ends,
  an ask's reservation given back. Release cleanup keeps its place
  through a cancel that replaces a closing's ending, since its write is
  still to be made. A forge removed takes its waiters out of the line
  (`adoption.md`, 1).

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
  new entry (skein's `forge/client.md`, 7.2). Each ends as its writer
  ends such a write now.
- **A full outbox refuses no one:** the limit's refusal for
  `forge.entries`, and the hold its words named (raise the limit,
  restart, release), are no longer reached by a write. The figure stays
  the user's (`../../draft/outbox-room-wait.md`, 2).

## 7. A waiting change's panel

- **Its wait is typed:** while a change holds a place, its panel carries
  the wait `OutboxRoom` (`change/README.md`, 5), and the client says
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
