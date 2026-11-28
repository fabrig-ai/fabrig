# A procedure's waits

Provisional, 2026-11-24, from `../../draft/bounded-waits.md`, 3.2, 4.1
and 4.2. How the forge connector's procedures, a change, a landing and a
goal's closer, report each wait their task is in to jig's core as the
record every task's wait is (`jig/docs/design/domain/engine/waits.md`,
6); the codes that name their steps and the things they wait on; each
wait's bound, its own clock or its kind's figure, and what happens at
it; and the bounds of the forge's work that runs in no procedure task of
its own, a goal branch's making and a release's cleanup.

## 1. In one page

- **The forge reports, the core lists** (2): after each step of a
  change, a landing or a closer, the forge tells the core the task's
  waits, at most four, and once for each such live task as it catches
  up after a start.
- **Codes name steps and things** (3): one table, which the client reads
  to name them.
- **A change's waits** (4) are its stall's: their bound is the instant
  the stall, its execution clock or its reads' count would hold it. A
  sign-off is the one wait that goes overdue rather than holding.
- **A landing's waits** (5) follow its step's wait
  (`../landing-waits.md`, 2): behind another task, its own gate's clock,
  or a wait on the forge bounded by the stall from its last advance.
- **A closer's waits** (6) take the step figure from its last step that
  advanced, then hold.
- **Room** (7): a waiter is behind the owners of what holds the room
  while one lives; behind owners that have all ended, it takes the room
  figure from the last end, then holds (P124).
- **A making and a release** (8) run for tasks whose waits the forge
  does not report; each keeps its bound in the forge and gives up, or
  fails, at it, a write in flight excepted.
- **At the bound** (9): one shape of words, and a hold keeps what any
  hold keeps; a write in flight is never given up.
- **Across a restart** (10): a change's and a sign-off's *since* are
  stored; the others start again at the start.

## 2. The report

- **Who reports:** the forge top, for each live task of three of its
  procedures: a change, a landing and a goal's closer. An after-landing
  tier's run, a status and a follow report none: each waits on a
  delegate or a person's answer, which jig computes, or on a bound of
  its own (`../after-landing-runs.md`, 2), and their idle time is jig's
  `news` (`jig/docs/design/domain/engine/waits.md`, 4).
- **What:** at most four records, each of kind `step`, `room`, `turn` or
  `sign-off`, with its object, its *since* and its bound: its own
  clock's instant with what happens then, its kind's figure with what
  happens then, or none with its reason (jig's `waits.md`, 6). A task
  that waits on a live delegate (its agent, its gate's task, its
  prepare, its child change), on its agent's open question, or that is
  held, reports none of these: jig's `delegate`, `question` and `party`
  waits are its waits then (jig's `waits.md`, 4).
- **When:** after each step the top takes for the task, and after each
  event that moves one of its waits without a step: it joins or leaves
  the outbox's line, an entry holding its room settles or that entry's
  owner ends, its forge is lost or found again. And once for each live
  change, landing and closer as the forge's catching up begins
  (`catching-up.md`, 3), an empty report included, so the core's wait for
  one report per live procedure after a start ends (jig's `waits.md`,
  6). A report equal to the task's last changes nothing, so a read again
  that changes nothing keeps its wait's *since*.
- **A refusal** (jig's `procedures::Ask::WaitsRefused`) is a fault of the top's: it steps
  no procedure, and the worlds' checks find it
  (`../../testing/engine-checks.md`, 3.1).
- **The figures** are the core's own, `waits.step_ms`, `waits.room_ms`
  and `waits.sign_off_ms` (jig's `waits.md`, 5): the root gives the forge
  the same values it gives the core, so a bound the forge reports as its
  kind's figure is the instant at which the forge acts on it.

## 3. Codes

A procedure's step and the things it waits on are numbers on the record
(jig's `waits.md`, 3), the forge's own, carried by jig's client wire as
they are. Their table is temper's client vocabulary, kept once where the
engine and the client both reach it; the client names each code in a
task's lines (`task show`), and its JSON keeps the numbers. A code once
given keeps its meaning; a new one is added, never renumbered.

| Step | Code | Named |
|---|---|---|
| a change's state | 101 taking, 102 producing, 103 opening, 104 recreating, 105 reopening, 106 checking, 107 gating, 108 ready, 109 preparing, 110 cancelling gates, 111 queued, 112 first, 113 updating, 114 waiting for its producer, 115 resolving, 116 repairing, 117 landing, 118 landed, 119 red, 120 cleared, 121 uncleared, 122 judging | as its stall's words name the step (`change/stalls.md`, 3) |
| a landing's wait | 201 prefixes, 202 open, 203 close, 204 delete, 205 post, 206 run, 207 receipt, 208 merge, 209 record, 210 ancestry, 211 read, 212 ready, 213 judgement, 214 room | as `../landing-waits.md`, 2, names the wait |
| a closer's step | 301 closing, 302 landing, 303 reporting, 304 making | `closing`, `landing`, `reporting`, `its goal branch's making` |

| Thing | Code | Number | Named |
|---|---|---|---|
| an outbox entry | 1 | its entry number, with its writer as owner | `entry E of task T` |
| a task | 2 | its number | `task T` |
| a pull request | 3 | its number | `pull request #N` |
| a run | 4 | its run number | `run N` |
| a gate | 5 | its place among its repository's gates, from 0 | `gate N` |
| a role | 6 | its number | `role N` |
| a landing rule | 7 | its place in its scope, from 0 | `rule N` |
| a forge | 8 | its number | `forge N` |
| the forge's landings (`../landing-forming.md`, 3) | 9 | the deployment's `landings` | `the forge's N landings` |
| a landing's make jig refused (`../landing-forming.md`, 3) | 10 | the make's number | `landing L's make, refused by jig` |

A code the client does not know is printed as its number.

## 4. A change's waits

The first row that holds, in this order:

| While the change | Kind | Object: step and things | Since | Bound |
|---|---|---|---|---|
| has a live agent, its agent's question open, its gate's task live, or is held | none | | | |
| has its forge not connected or its repository out of reach | none: its stall stands still (`change/stalls.md`, 1), and jig's `news` names what wakes it | | | |
| holds a place in the outbox's line | `room` (7) | | | |
| is at producing with a pending dependency | `step` | producing; each pending dependency, a task | the stall's origin | none: behind the first pending dependency |
| is short of a sign-off at its head (`change/branch-path.md`, step 5) | `sign-off` | its step; the rule, then each role that can still sign, at most three | the change's readiness (`Change.ready_since`) | the sign-off figure, *overdue* |
| waits to land while the operator holds every landing (`../landing-settings.md`, 1.3) | `step` | its step | the stall's origin | none: an operator's hold |
| waits quietly for its turn, with no read of its own and no stall counting: queued behind the queue's first (`queues.md`), or ready under turn in no landing while its train's live landing holds it (`../landing-turn.md`, 2) | `step` | its step; that first change, or that landing, a task | the stall's origin | none: behind that task, whose own waits bound it |
| is ready under turn in no landing, its train free, while changes into its train settle in a cohort (`../landing-turn.md`, 2) | `step` | its step; each settling change, a task, by number, at most four | the stall's origin | none: behind the first settling change, whose next decision ends it |
| is ready under turn in no landing, its train refused by the forge's `landings` (`../landing-forming.md`, 3) | `room` | its step; the forge's landings, then the landings holding them, oldest first, at most three, each a task | the later of its train's refusal's *since* and the stall's origin | none: behind the oldest holder whose task lives, else behind the oldest holder, whose writes in flight settle within the forge client's bounds |
| is ready under turn in no landing, its train refused by jig (`../landing-forming.md`, 3) | `step` | its step; the refused make | the later of its train's refusal's *since* and the stall's origin | none: no deadline |
| waits quietly with no task to name: red, waiting for a repair's verdict (`../after-landing.md`, 6) | `step` | its step | the stall's origin | none: no deadline |
| any other step | `step` | its step; what it waits on: its pending write's entry, its pull request, its CI's newest run where it knows one | the stall's origin, `Change.since`; where its reads' own count runs (`change/stalls.md`, 1), that count's start | the instant at which the change would be held if nothing moved: the stall, or at checking with a reading the earlier of the execution clock's bounds as the clock stands (`change/stalls.md`, 2.3), or its reads' count's end; *hold* |

- **At the bound** the change is held as its stall holds it, with its
  stall's words (`change/stalls.md`, 3 and 4): they name the step, the
  bound and what it waited on, and at checking what CI last showed.
- **A sign-off past its figure** holds nothing: the change stays out of
  the queue as its sign-off waits (`queues.md`), and the record, past its
  bound, is the overdue entry the views show (the draft, 9). A change
  short of a sign-off has no stall (`change/branch-path.md`, step 5).
- **A landed change** waiting on its verification reports that wait with
  the bound its design gives it (`../after-landing.md`, 6): its
  verification bound, *give up*, while no verdict has come; none, with no
  deadline, once red (row above), since only a repair's landing ends it.
- **A quiet wait is still reported** (2026-11-24): where the change's
  policy waits with no instant of its own, its record says what it waits
  behind or that nothing bounds it, so its age shows; it invents no hold.
  A read the step makes still counts its reads' own bound first (last
  row).

## 5. A landing's waits

From its step's wait (`../landing-waits.md`, 2), the top's own `read`
and room among them:

| Its wait | Kind | Object | Since | Bound |
|---|---|---|---|---|
| `release`, held | none | | | |
| `prepare` | none: its prepare is a live delegate | | | |
| `ready`, `judgement` | `step` | the wait; the member's change, a task | its last advance | none: behind that change |
| `turn` | `turn` | the landing holding the turn | its last advance | none: behind that landing |
| `receipt` for a run of another landing | `step` | receipt; that landing, a task | its last advance | none: behind that landing |
| `run`, its own gate's run while that gate's clock counts it | `step` | run; the gate, the run | its last advance | the gate's clock: its timeout or its runner wait as the clock stands (`../landing-proof.md`, 2.1); *hold* |
| any other wait on the forge (`../landing-waits.md`, 3) | `step` | the wait; the member's change, or the round's pull request | its last advance | the stall from its last advance; *hold* |
| room | `room` (7) | | | |

The last advance and the bound on a wait on the forge, with its hold's
words, are `../landing-waits.md`, 3's.

## 6. A closer's waits

- **Since** its last step that advanced: a step that kept anything new on
  its row, a release, or the engine's start. The top keeps it unstored.
- **Its waits:**
  - delegating its change, at its close or land step, and waiting for
    that change: none (a live delegate);
  - at its land step, reading its goal's and its base's tips and whether
    the base holds the goal's tip (`branches-closer.md`, 1): `step`, landing,
    the step figure from its *since*, *hold*;
  - at its land step while the operator holds every landing: `step`,
    landing, none: an operator's hold;
  - at its report step, reading pull requests, checks and job logs: `step`,
    reporting, the step figure, *hold*;
  - waiting on its goal branch's making: `step`, its goal branch's making,
    the step figure, *hold*.
- **At the bound** the closer is held as a closing step's hold is
  (`branches-closer.md`, 1), with 9's words: `no progress at landing for 30
  minutes: waiting for whether main at 89abcdef0123 holds the goal's tip
  0123456789ab; the last answer: not there yet`.

## 7. Room

A waiter in the outbox's line (`room.md`, 3), a change, a landing, a
release or a making:

- **Since** it joined the line. The top keeps it with the place, unstored:
  a place restored at a start joins at the start.
- **Object:** the waiter's step, and the entries holding the room, oldest
  first, at most four, each with its writer as owner.
- **Bound:** while an entry or a reservation holding the room is a live
  task's, the waiter is behind the oldest such task: that task's own
  waits bound it (the draft, 4.1). Once every holder's owner has ended,
  the waiter takes the room figure from the latest of those ends, or from
  its *since* where that is later, and holds at it.
- **An owner's end** is when the forge learns that the entry's writer
  closes or ends (its settling, its ending), kept with the entry,
  unstored; an entry restored whose writer is no live task has its end at
  the start.
- **At the bound:** a change is held as its stall holds it, a landing
  holds `read` (`../landing-waits.md`, 3), a release fails and a making
  gives up (8), each with 9's words, WHAT `room in the forge's outbox,
  held by entry E of ended task T`, E the oldest holder. A held change or
  landing keeps its place (`room.md`, 3).

## 8. A making and a release

A goal branch's making runs for its goal's task, and a release's cleanup
for its closing task. Neither task's waits are the forge's to report: a
goal is an agent's task, and a closing task waits on its own ending,
which no wait of the record names (jig's `waits.md`, 3). So each keeps
its bound in the top alone, from its last step that advanced, unstored:

- **A making** (`branches.md`, 1), whose reads (its branch, its landing
  tip) have been retried past the step figure with no write in flight,
  gives up: it fails as a refused making does, its goal's effect failing
  with 9's words, and the goal's run decides what follows. A making past
  its room's bound (7) gives up the same way. Its branch's creation, once
  in flight, is never given up.
- **A release's cleanup** (`effects.md`, 1), waiting past the step figure
  on its reads (fresh observations, a writer's reread, a goal branch's
  making) with no write in flight, fails as a failed reread fails it, with
  9's words. Past its room's bound (7) it fails the same way. Its
  comment, close or branch deletion, once in flight, is never given up.

## 9. At the bound

- **Its words** are a change's stall's shape (`change/stalls.md`, 3):
  `no progress at STEP for N minutes: waiting for WHAT`, N the bound's
  figure in whole minutes, followed, where the wait was on a read the
  forge answered or refused, by `; the last answer: WHY`, WHY in
  `../landing-waits.md`, 3's terms. STEP is the procedure's step as 3
  names it, `making` for a making and `release` for a release's cleanup.
  A change keeps its stall's words as they are; a landing's are its own
  (`../landing-waits.md`, 3).
- **What a hold keeps** is what any hold keeps today: the task's place in
  the outbox's line, passed over while it is held, and an ask's
  reservation given back (`room.md`, 3); its entries in flight, each with
  its read-back as its wake. A write in flight is never given up: its
  outcome is recorded on the held row whenever it comes. A held change
  gives up the landing turn as it does today (`change/stalls.md`, 5); the
  turn is the landing train's (the draft, 11).
- **Its hold escalates** as every hold does (`change/stalls.md`, 5), and
  a release starts a fresh wait, its next step an advance.

## 10. Across a restart

- **Stored:** a change's stall's origin, `Change.since`, and its
  readiness, a sign-off's *since*; whether a change settles in a cohort
  (`../landing-turn.md`, 2); a train's refusal with its *since*
  (`../landing-forming.md`, 3): a start finds the same episode.
- **Started again at the start:** a change's reads' count, a landing's
  last advance, a closer's, a making's and a release's, and a waiter's
  join and its holders' owners' ends, each of which lives in memory: it
  is counted again from the start, as the draft allows for a wait whose
  place lives in memory (the draft, 3.4).
- The core floors each *since* by its kind's floor (jig's `waits.md`, 7).

## 11. Open questions

- **A wait for access:** no reason of the record's names a forge's
  outage, so a change or a landing whose forge is lost reports none and
  shows jig's `news`. Default: so; a reason of its own would move jig's
  client wire.
- **A live gate's task or prepare:** jig shows the delegate, not the
  stall that bounds it. Default: so, jig's rule (jig's `waits.md`, 4).
- **Unstored clocks:** a restart starts a landing's, a closer's, a
  release's and a room wait's clock again. Default: so; a stored one
  would let a bound outlive restarts, at the price of a stored field each.
- **Unbounded quiet waits:** a red change waiting for its repair, and a
  ready change under turn whose train jig refused, report no deadline,
  though the draft bounds every wait it names
  (`../../draft/bounded-waits.md`, 4.1). Default: so; a red change's
  repair is its goal's work, after-landing's waits are not this
  milestone's fourth goal's, and jig's room, holdings and funding are
  the deployment's, which no member can move, its train asked again
  each queue window (`../landing-forming.md`, 3).
