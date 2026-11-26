# A landing's forming

Provisional, 2026-11-19. How the decision that forms a landing
(`landing-turn.md`, section 2) is made whole: its task made first, and the
landing's row, which makes it its members' landing, and its branches'
holds saved only once jig holds that task; a refusal that may pass,
jig's or a full `landings`, forming nothing, stored on its train with
what it names and when it is tried again, and named in the waits of
the train's changes and in the engine's log; and
a start over a store that already holds a landing whose task was never
made ending that landing, so its members wait again. Which units form a
landing, and when forming is judged, is `landing-turn.md`, section 2; the
landing task is `landing.md`, 4.2; the restart is `landing.md`, section
15.

## 1. In one page

- **The task first** (section 2): the forming decision asks jig to make
  the landing's task before it saves anything of the landing, and jig
  answers the make within that decision.
- **Saved only once made:** the landing's row, which makes it its
  members' landing (`landing-members.md`, 2.1),
  its branches' holds and its members' `joined` news are made in that
  same decision only where jig then holds the task. No stored row and no
  hold names a landing task that was never made.
- **An ended landing yields its place** among the `landings` once its
  writes have left the outbox and it follows no run (section 2.3).
- **A refusal that may pass forms nothing** (section 3): jig's refusal
  of the make, or the `landings` at their bound with no row that
  yields. Its members stay ready and in no landing, keeping their
  places; the train stays free; the refusal is stored on the train,
  named in its changes' waits and in the engine's log; a train jig
  refused forms again no sooner than a queue window later, a restart
  included, and one refused by `landings` as soon as a row yields.
- **A store that already holds one starts** (section 4): the start's
  link pass ends each live landing whose task jig doesn't hold, its
  members waiting again, and drops each hold whose task jig doesn't hold
  and under which no task jig holds writes, logging each. Anything a
  task jig holds names, it still refuses.

## 2. The forming decision

### 2.1 Its order

The decision that forms a free train's landing (`landing-turn.md`, section 2)
does, in this order:

1. **Its names reported** to jig: each member's branch in the landing
   (`landing-prepare.md`, 1.1), as before.
2. **Its task's make:** jig's `Event::ConnectorRepair` with the
   project's procedure seed, the landing's spec, contract and holdings
   (`landing.md`, 4.2), as before.
3. **Its admission:** where jig holds a task by the seed's number, the
   landing's row (`landing.md`, section 12), which makes it each
   member's landing by the index in the same decision
   (`landing-members.md`, 2.1) and tells each member's goal it joined (`landing.md`,
   4.2), then a
   hold of each member's branch by the landing's task. Where jig holds
   none, the refusal (section 3), and none of these.

- **Answered in the decision:** jig's make drains within the decision
  that asks it, made or refused, its names reported before it, as the
  status procedure's admission already reads (`temper-engine-domain`'s
  `status_route::admission`). The admission reads jig's tasks after the
  make; it never assumes either answer.
- **Activation after:** the made task's activation and first step come
  after the admission in the decision's order, so its first step finds
  its row.
- **One commit:** the decision commits whole, so a store holds the
  landing's row and its holds with its task, or none of the three.

### 2.2 What jig answers

- **A refused make** is told to every connector as `Ask::RepairRefused`
  (jig's `domain/authority.md`, section 6), which carries:
  - `repair: Option<u64>`, the task jig's refusal names where it names
    one, as before;
  - `number: u64`, the refused make's own number, its seed's;
  - `why: tasks::Refusal`, jig's reason.
- **temper keeps it for the admission:** temper's handler of
  `RepairRefused` (`route/asks.rs`) keeps `why` against `number`,
  unstored, where a forming awaits its admission by that number, so the
  admission names it. Its status and queue repairs read `repair`, as
  before.
- **A made task** is read as jig's tasks holding it: jig sends no
  acknowledgement of a make.

### 2.3 Its place among the landings

From P124 (`../../development/failures-1113-1120.md`): on the reference
deployment a cancelled landing's row kept its place for good, since it
kept its gate's dispatch in the outbox.

- **Bounded by `landings`** (`landing.md`, section 12): the admission
  counts every landing row the forge keeps, live or ended, and the check
  before the root makes the landing's task counts them the same way.
- **An ended row keeps its place until a forming needs it:** a landing
  that has ended, landed or cancelled, keeps its row, which its views
  read, until a forming finds the rows at the bound; that forming erases
  one ended row that yields, and takes its place.
- **It yields** once no entry of its task is in the outbox and it
  follows no run: its writes have left (`forge/room.md`, 2) and its runs
  have ended, or it was cancelled (`landing.md`, 4.2). The marks of its
  ended runs, kept on the row for its views, don't hold it.
- **A row that doesn't yield keeps its place:** with no other, the
  forming is refused, `landings` (section 3), before jig is asked to
  make anything, and its changes wait ready. It is never erased sooner: an entry
  settling later would find no landing to settle it, and a run's receipt
  no landing to make it.

## 3. A refusal that may pass

From `../draft/landing-liveness.md`, 4.3. A train refused for a cause
that may pass waits on a refusal of its own, before any landing exists.
A refusal that won't pass, a unit across repositories or past
`landing_members`, stays a hold on the unit's first change, and the
train passes over the unit (`landing.md`, 4.1).

- **Its causes:** jig refuses the landing task's make (section 2.2); or
  the forge's `landings` are at their bound with no row that yields
  (section 2.3), found as the train's selection is admitted, before the
  make, so that jig is asked nothing.
- **Nothing of the landing is saved:** no row, no link, no hold, no
  news and no history event. A make jig refused keeps its seed's number
  consumed: jig never reuses one.
- **Its members wait,** ready, in no landing, each keeping the time it
  first became ready, so the train's next landing takes them in their
  order (`landing-turn.md`, section 2). No member is held, repaired or told:
  the refusal is the deployment's room, configuration or funding, never
  a member's fault.
- **Stored on its train:** the train's refusal is kept on its
  repository's row (`landing.md`, 12), one per train, by its branch:
  - **since,** the wall time of the train's first refusal since it last
    formed, kept while it is refused again, whatever the cause;
  - **its cause:** `landings`; or `jig`, with the refused make's number,
    jig's reason as words of at most 256 bytes, and **retry**, the wall
    time the deployment's queue window (`forge.queue_window_ms`,
    `../protocol/configuration/limits.md`) after this refusal.

  A later refusal of the same train replaces its cause and keeps its
  *since*. The running engine keeps no refusal of its own.
- **Tried again by what can change it:**
  - **`jig`:** forming is judged for the train again at the first
    judgement at or after its retry; other trains form as ever. The
    domain's next deadline includes the earliest retry still waiting, so
    a judgement comes then though no member wakes it (a ready change
    keeps no timer of its own).
  - **`landings`:** at every judgement of forming, since judging it reads
    the landing rows alone and asks jig nothing. The judgement after a
    landing ends, or after an ended landing's last entry leaves the
    outbox or its last followed run ends, finds the row that yields and
    forms in its place (section 2.3).
  - **Across a restart** the refusal stands: a train jig refused is not
    asked again before its retry (`landing.md`, 15), and its *since* is
    unchanged. So a refusal neither stops its train for good, nor forms
    it again in its own decision, nor at once after a start.
- **Cleared** when the train's landing is admitted (section 2.1), or
  when no change into the train is left waiting: the train's judgement
  that finds none erases it.
- **In its changes' waits** (`forge/waits.md`, 4): each waiting change
  of a refused train waits on its refusal. Refused by `landings`, a
  `room` wait naming the table and the landings holding it, oldest
  first: the live ones and the ended ones that still follow a run for a
  receipt or have an entry in the outbox; it is behind the oldest of
  them whose task lives. Refused by jig, a `step` wait naming the
  refused make, with no deadline, since the deployment's room is no
  member's to bound.
- **In the engine's log,** a warning for a train's first refusal since
  it last formed, and for each whose cause differs from the one stored,
  jig's reason included:

  ```
  BINARY serve: warning: landing L into BRANCH of repository F/R not formed: jig refused its task: WHY; its changes C1, C2 wait
  BINARY serve: warning: landing into BRANCH of repository F/R not formed: the forge's landings are full, N of N, held by landings L1, L2; its changes C1, C2 wait
  ```

  `L` the refused make's number, `BRANCH` the train's branch, `F/R` its
  repository's forge and repository numbers, `WHY` jig's reason as
  stored, `N` the deployment's `landings`, `L1, L2` the landings holding
  them in the order the wait names them, and the changes in the
  order they would have stacked. A refusal repeated for the same cause
  logs nothing more; the changes view still shows them waiting
  (`landing.md`, section 14).
- **jig's reason** is its `tasks::Refusal` as Rust's `Debug` prints it,
  as the log has given it since section 2 was built. Every one is the
  same here: the live-task or project-task room (`Live`, `Project`),
  the holdings (`Holds`, `HoldTaken`), the funder (`Funding`), a busy
  or unavailable resource. The log and the stored refusal name which.

## 4. A store that already holds one

A store written before section 2 was built may hold a landing whose task
jig never made: its row live (`waiting`, round 0), its members linked to
it, and its branches held by its number. Its train is never free, and
the start's link pass refused that store, a hold linking a missing task.

- **In the link pass,** after the restored names' and tiers' links and
  before `missing_link` judges the restored rows (`temper-engine-domain`'s
  `Work::ForgeRestored`):
  - **A live landing jig holds no task for,** its phase neither landed
    nor cancelled, its outbox holding no entry, and jig's tasks holding
    no task by its number, whether jig never made its task or its task
    has gone, is ended as `landing-liveness.md`, 6.2 ends it: as a
    cancelled landing ends (`landing.md`, 4.2), its row kept ended and
    yielding its place, a merge of unknown outcome read back first. Each
    member's change in it is ready, in no landing, keeping its place,
    unless it is recorded landed. A landing that never stepped writes
    nothing to the landing history.
  - **A hold whose task jig doesn't hold,** on a forge not ended, so
    that the link pass would refuse it for its holder, is dropped, its
    row erased, where no task jig holds writes under it: neither its
    writer's run's task nor any of its design writers. A hold such a
    task writes under is left, and the link pass refuses it as before.
  - **Each is logged,** a warning in the engine's log, `L`, `BRANCH`,
    `F/R` and the changes as section 3 names them, `T` the hold's task:

    ```
    BINARY serve: warning: landing L into BRANCH of repository F/R has no task: ended; its changes C1, C2 wait again
    BINARY serve: warning: hold on BRANCH of repository F/R by task T, which has no task: dropped
    ```

    A landing row with no members, which forming never writes, names no
    repository: it is ended like the rest, without a line.
- **What it still refuses:** every other link the pass judges, as
  before: anything a task jig holds names, a hold a live writer names, a
  live landing with an entry in its outbox, and every other family's
  links.
- **Its writes are the start's,** in the link pass's commit, before the
  sweep and the switch, as jig's start repairs its projections there
  (jig's `domain/engine/restart.md`), so a second start finds nothing to
  repair.
- **Its members after:** the restart judges forming for every train
  (`landing.md`, section 15), and the members form the train's next
  landing in their order, through section 2.

## 5. Testing

- **Focused,** `temper-engine-domain-world`'s `suite`:
  - a forming whose make is refused, the live-task room full: no
    landing row, no hold, each member ready in no landing, the warning
    naming `Live`, no second forming in that decision; with room freed
    and a queue window passed, the train forms and lands. Red before
    section 2 is built, green after;
  - the same refusal across a restart within the window: the train's
    refusal is read back, no make is asked before its retry, and after
    it the train forms;
  - a train refused by a full `landings`, its rows held by a live
    landing and by an ended one with a write still settling: no make
    asked, the train's refusal stored, its changes' waits naming the
    table and both landings, one warning; once the ended row yields,
    the train forms in its place with no timer, and its refusal is
    gone;
  - a store holding section 4's landing starts: the row ended, its
    members' links cleared, its holds dropped, a warning each; its
    members then form a landing and land;
  - the same store with a hold a live task writes under still refuses,
    naming that hold's link.
- **Focused,** `temper-engine-shell`'s lines: the warnings' words; and
  `jig-domain-core`: `RepairRefused`'s `number` and `why`; and
  `temper-protocol-store`: a repository's row with a train's refusal of
  each cause round-trips, and one the previous build wrote reads with
  none.
- **Focused,** `temper-engine-forge-world`'s `suite`, for section 2.3
  and `forge/room.md`, 2 (P124's scenario, active, with no sweep
  exclusion):
  - `cancelled_landing_frees_its_write_entries`: with the outbox full
    and the `landings` at their bound, a landing cancelled while its
    gate's dispatch is in flight; its dispatch settles after the cancel
    and leaves the outbox, the change waiting for room is admitted, the
    cancelled landing's close completes, and the train's next landing
    forms in its row's place;
  - the same for a landing that lands: its dispatch's entry leaves at
    its outcome while it still follows the run, a change waiting for
    room is admitted while the gate runs, and once landed its row yields
    to the next landing.
- **Fuzzy:** the landing and walking suites stay green, unchanged.

## 6. Later

- A refused forming as a landing-history event per member, readable by
  `landing history` (a kind of `landing-history.md`, 2.2), and as news
  to its members' goals.
- A dropped hold's branch, where it was ever pushed, stays on the forge:
  no task releases it.
- A forming refused for good (a landing's holdings past jig's bound)
  logs once and waits each queue window, its changes' waits with no
  deadline; nothing holds its train for a person.
- A train's refusal in `landing list` and the waits read, with jig's
  words: the views are bounded waits' (`../draft/bounded-waits.md`, 9),
  and they read it from the train's stored refusal and its changes'
  waits.
