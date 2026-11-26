# A landing that can move

Provisional, 2026-11-26. The check that a live landing can move, run
at every step and at start; what a landing that can't move does, ended
when nothing of it landed and held with words when something did; the
wakes that bring a landing to its next step when a member or its
prepare goes; and the one settling every ending of a landing shares.
It details `landing.md`, 4.2 (*It ends*, *No silent wait*) and 15, and
carries `../draft/landing-liveness.md`, 5, 6 and 11, whose reasons are
in `../draft/landing-liveness.notes.md`. What each step does stays
`landing.md`'s; what a step waits for, `landing-waits.md`'s; who is in a
landing, `landing-members.md`'s.

## 1. In one page

- **The check** (section 2): after the policy's own departures and its
  decision, a live landing can move when its task is live, each member
  still in it is live, unheld and its own, no other live landing holds
  its train, and it has a wake that can come. It runs in every step,
  and so after every wake and fired deadline, and at start.
- **A landing that can't move** (section 3) first settles a merge whose
  outcome is unknown, as its turn reads one back; then, with nothing
  landed, it ends, cancelled through its task; with something landed,
  it holds, `cannot move`, with words naming what failed.
- **A member's hold, cancel or end wakes its landing** (section 4) in
  the decision that saves it, through the index, and so does its
  prepare child's; its next step lets the member leave.
- **One settling for every ending** (section 5): landed, cancelled or
  failed, the landing's task's closing ends a row still live as a
  cancelled one ends, withdraws its unsent writes and settles those in
  flight.
- **At start** (section 6): a live row whose task jig doesn't hold is
  ended by the forge, a merge of unknown outcome read back first once
  its repository is read afresh; every other live landing is checked by
  its first step on fresh facts.

## 2. The check

### 2.1 What it checks

In every decision that steps a landing, the policy first does what it
does today: members held, cancelled or ended leave, with their units and
dependents (`landing.md`, 4.3), an absent prepare is asked again or held
(`landing-preparing.md`, 5), and the step decides. Then, on the state
that decision leaves, the check. A live landing can move when:

- **its task is live:** a step is its task's, so a landing that steps
  has one. No live row outlives its task: every ending makes its row
  ended as its task closes (section 5), and a row whose task is gone is
  the start's (6.2).
- **each member still in it**, neither left nor landed, is live and not
  held, by its facts: the step's hearing names a member whose task jig
  holds as held, and one whose task is closing, ended or not in jig's
  live tasks as ended (section 4). Each is in this landing alone by the
  index (`landing-members.md`, 2.1): a change two live rows claim holds
  the newer before it steps (`landing-members.md`, 2.2), so a step finds
  it so.
- **no other live landing holds its train:** the train's holder
  (`landing-turn.md`, 2), the oldest live landing on its repository and
  branch, is this one. A single landing (`landing.md`, 12) keeps the
  project's turn as today, and waits `turn` behind the landing holding
  it.
- **it has a wake that can come:** its decision acts (anything but
  `wait`, `landing.md`, 2); or it waits with a deadline its policy
  names; or it waits with none on what its facts say will answer:
  `prepare` on a prepare that is running; `ready` or `judgement` on a
  member still in it, live and unheld; `turn` on a live landing that
  holds the turn; `open`, `close`, `delete`, `post`, `record` and
  `merge` on its write pending at the forge, or `merge` on the readback
  its turn owes (`landing-turn.md`, 1); `release` while held. Each other
  wait of `landing-waits.md`, 2 carries its deadline and its bound
  (`landing-waits.md`, 3; `forge/waits.md`, 5). The poll the top arms
  for a wait with no deadline of its own is a read again, never a wake
  (`../draft/bounded-waits.md`, 4.1).

The check reads the landing's row, its facts and the index alone,
never every task. It is the policy's, so its fuzzy holds it (section 8);
the top gives it the one fact the policy had no need of, the train's
holder where it is another landing.

### 2.2 When it runs

- **In every step,** last, on the state the step's decision leaves.
- **When a wake is withdrawn or an armed deadline fires:** each wakes
  the landing (section 4), and the step that follows checks it.
- **At start** (section 6).
- **Not while held:** a held landing takes no step, and its hold names
  its way out, a person's release or its own bound
  (`../draft/bounded-waits.md`, 8). The step after its release checks
  it. A wake never releases a hold.

## 3. A landing that can't move

### 3.1 What it may have landed, first

A landing that fails the check while its round's merge has no known
outcome, in flight or answered uncertain (`landing-turn.md`, 1), settles
it before anything else: the step's decision stands as its turn decides
it, so the landing keeps its turn and its resources, reads back the
train's tip and the round's pull request, and records each landed
member once. The verdict (3.2) waits until the readback has said what
landed. Ending or holding before it would lose a landing the forge made,
or make it twice (`../draft/landing-liveness.notes.md`).

### 3.2 Ended or held

On the state the step leaves, the first clause of 2.1 that fails names
the cause (3.3), and:

- **Nothing landed** (no member recorded landed): the landing ends. Its
  decision `end` asks jig to cancel its task, by the deployment for its
  project, as a cancel the policy decides does today (`landing.md`,
  4.2), the words as its reason, so the task's ending names them. In
  the same decision each member's goal hears `landing L: cancelled:
  cannot move: WORDS`, once. The task's closing ends the landing as
  section 5 says: its members ready and in no landing, keeping their
  places, and its train forms again.
- **Something landed:** the landing holds, its own hold `cannot move`,
  its words `cannot move: WORDS`, and its members' goals hear `landing
  L: held: cannot move: WORDS` (`landing.md`, 4.2's news). The hold
  escalates as any hold does. Its release steps it, and the step's check
  holds it again, with the same words, while the cause stands.

A landing with nothing landed loses nothing by ending: its members go
back to the queue. One with a member landed needs a person to see it.

### 3.3 Its words

C a member's change task, O a landing's task:

- `member change C is held`; `member change C has ended`, for one whose
  task is closing, ended or gone;
- `landing O holds its train`;
- `its wait has no wake: WAIT`, WAIT the wait's words in
  `landing-waits.md`, 2's last column.

Words are cut to the history's reason bound as any hold's are
(`landing-history.md`, 2.2, `held`).

## 4. What wakes a landing

- **A member's hold, cancel or end:** the decision that saves a member's
  change task held, closing or ended wakes the member's landing, looked
  up by the index (`landing-members.md`, 2.1), as a change made ready
  wakes it. The landing's next step hears the member held, or, closing,
  ended or gone, ended, and the policy lets it leave with its unit and
  dependents (`landing.md`, 4.3: `held`, `cancelled`). So S7's landing,
  its prepare child gone and its member held by a person, moves without
  a person releasing it.
- **Its prepare child's hold or end:** the decision that saves the
  landing's live prepare child held, closing or ended wakes the landing
  too. Held, the step reads the prepare failed and holds `prepare job`
  as today; ended with no answer, absent, it is asked again or held
  (`landing-preparing.md`, 5).
- **Its writes,** withdrawn or answered, wake it (`forge/room.md`, 2),
  and **its deadline** steps it (`landing-waits.md`, 3), as today.
- **A wake to a held or closing landing** does nothing: it steps once
  released, and a closing one is settled by section 5.

## 5. Every ending settles what it holds

The three endings share the settling of what a landing holds, and keep
apart what differs:

| Step | Landed | Cancelled | Failed |
|---|---|---|---|
| The row ends: its members leave, each with the ask the ending owes it (`landing-members.md`, 3.5) | at its finish, after `landed` for each member that landed | at its task's closing: each member recorded landed is told, every other is ready and in no landing, keeping its place | at its task's closing, as cancelled (today it skips this) |
| Its train is freed | as its row ends, once the merge is recorded | at its task's closing | at its task's closing |
| Unsent outbox entries withdrawn; writes in flight awaited and settled (`forge/room.md`, 2) | yes | yes | yes, as cancelled (today it skips this) |
| A run followed for a receipt (`landing-proof.md`, 2.1) | followed, keeping no outbox entry | no longer followed | no longer followed |
| Branch holds | released as its task's release runs | released as its task's release runs | released as its task's release runs, or handed to another root where a task's release allows it today |

- **One rule at closing:** whatever jig's ending, the closing of a
  landing's task ends its row if it is still live, as a cancelled one
  ends: its phase `cancelled`, its deadline cleared, its runs no longer
  followed, its index entries and its own `prepare` and `judge` asks
  gone, its train woken, its history's open round ended `cancelled` and
  its `landing-ended` written `cancelled` (`landing-history.md`, 2.2).
  A landed landing's row ended at its finish, so its closing settles its
  writes alone.
- **A failed landing reads cancelled:** its row's phase and its
  history's outcome say how the row ended, unlanded; its task's own
  ending says it failed, with jig's reason. So no stored or client enum
  gains a variant for it.

## 6. At start

### 6.1 A row with its task

The link pass gives a change two rows claim to the older
(`landing-members.md`, 2.2), as before. Every other live landing whose
task jig holds, unheld, steps once its facts are read afresh
(`forge/catching-up.md`; `landing.md`, 15), as every restored landing
does, and that step's check is the start's check of it, its forge's
facts fresh. A held one is checked after its release (2.2).

### 6.2 A row whose task is gone

In the link pass, once the index is built and the previous build's
asks converted (`landing-members.md`, 4.2), a live row whose task jig
doesn't hold has no task to hold, so the forge ends it, as
`landing-forming.md`, 4 ended a landing whose task was never made, which
this case takes in:

- **With no merge of unknown outcome:** it ends there, as section 5
  ends a row at its task's closing, and the holds its task kept are
  dropped. A row with no round (round 0, never stepped) writes no
  history. The ended row yields its place among the `landings` at once
  (`landing-forming.md`, 2.3).
- **With a merge of unknown outcome** (its round at its turn, a merge
  counted and no outcome kept): its ending waits, keeping its train,
  until its repository has been read afresh (`forge/catching-up.md`).
  The forge then takes the train's tip from those facts: a tip that is
  or contains the round's proved top records each member up to it
  landed at its prefix, once, its change handed `landed`
  (`landing-members.md`, 3), as the turn records it; then it ends as
  above. A squashed member's pull request is not recorded merged: that
  write is its gone task's (section 9).
- **With an entry in its outbox,** the link pass refuses it, as before
  (`landing-forming.md`, 4, *What it still refuses*).
- **Unstored:** a restart before a deferred ending finds the row
  taskless again and starts over.
- **Logged,** a warning each, `L`, `BRANCH`, `F/R` and the changes as
  `landing-forming.md`, 3 names them; the clause for changes landed by
  the readback, and the one for changes that wait again, are dropped
  where they name none, and the last is added only where a squashed
  member landed:

  ```
  BINARY serve: warning: landing L into BRANCH of repository F/R has no task: ended; its changes C1, C2 wait again
  BINARY serve: warning: landing L into BRANCH of repository F/R has no task: ended after reading back its merge; changes C1, C2 landed; its changes C3 wait again; the pull requests of changes C2 are not recorded merged
  ```

## 7. In the store and on the wires

- **The landing's own hold** gains a cause, `stuck`, with what failed
  (3.3): the clause, the change or landing it names, and the wait: a
  variant after every kept one, a change a plan derives
  (`../protocol/store-versions.md`, 1), so no version moves and nothing
  is patched. On the client's wires it reads as a `read` hold does, by
  its words, as `cleanup` does (`landing.md`, 4.2), so no client wire
  moves.
- **Nothing else is stored:** a failed landing's row reads `cancelled`
  (section 5), and a deferred ending at start is unstored (6.2). A
  step's hearing and the policy's facts cross no wire.

## 8. Testing

Focused and fuzzy suites only.

- **The policy's fuzzy** (`landing-waits.md`, 3): as a property, each
  step's state passes the check, or its decision is the check's verdict
  for the cause it names; over random landings with conflicts,
  departures, members held, cancelled and ended at any step, another
  landing holding the train, and merges answered with no known outcome
  or lost, read back from the tip and the round's pull request. No
  landing with a merge of unknown outcome ends or holds for the check
  before its readback, and every landing reaches landed, cancelled or
  held within its bound of steps.
- **The engine's worlds:** the forge world and the root's world check
  2.1 for every live landing at each quiet point (`../testing/engine-checks.md`,
  2.2), in their quiet and busy runs, with members held and cancelled by
  a person, landings cancelled and failed, uncertain merges, cohorts of
  several changes made ready at once, and restarts mid-landing.
- **S7** (`../testing/engine-checks.md`, 5.1): its landing moves with no
  person's release.
- **Focused,** each verdict with its words and the readback before it;
  each wake of section 4 in the decision that saves its cause; each
  ending's settling of section 5, the failed one's included; and the
  start's cases of 6.2, from stores holding such rows.

## 9. Later

- A taskless row's squashed member's pull request recorded merged, by a
  task the forge would make for it.
- `member change C is held by P`, naming the party whose placed hold it
  is, where jig names one.
