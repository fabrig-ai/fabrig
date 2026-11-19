# A landing's waits

Provisional, 2026-11-18. What each step of a landing waits for, the
rule that no step waits on what nothing will answer, and how a person
sees a landing that stopped: `temper ctl landing list` and `landing
show`. It details `landing.md`, 4.2 (*No silent wait*), 5.1 (*A round
ends*) and 14. What each step does stays `landing.md`'s; a step's
events stay `landing-history.md`'s.

## 1. In one page

- **A step acts or waits on something it names.** Each wait the policy
  decides names what it waits for (section 2); the top keeps the
  latest on the landing, and `landing show` prints it.
- **No wait without its wake** (section 3). A step waits with no
  deadline only on an effect the landing started, whose answer wakes
  it, on a member's change taking its own step, or, held, on its
  release. Any other wait has a deadline, at which the landing reads
  afresh and decides again.
- **No decision is lost.** A decision the root cannot carry out as it
  is made leaves the landing as it was and is decided again in the
  landing's next due step; nothing keeps a write's result pending that
  no write carries.
- **After a round ends,** cut, stale or left, the landing's next step is
  due at once: the next round's prepare, its pull request's opening,
  its proof and merge, or its end.
- **A stall shows itself** (section 4): the list gives each live
  landing's step and the age of its last event, and the project's turn
  holder with how long it has held its turn.

## 2. What a step waits for

The policy's wait (`landing.md`, 2, the decision `wait`) names one of
these, the first that holds in the step's order; the top names the
last two rows itself. C is the member's change task, NAME the gate's name,
RUN the run's number where the top knows it, L a landing's task.

| Wait | When | Woken by | `landing show` says |
|---|---|---|---|
| `release` | held (`landing.md`, 4.2) | a person's release | `its release` |
| `ready` | a member's change not ready: judging at its head, repairing, held | the change's step | `change C to be ready` |
| `turn` | a single landing proved while another holds the turn (`landing.md`, 12) | that turn's end | `the turn, held by landing L` |
| `prepare` | the round's prepare lives (`landing.md`, 5.2) | its answer | `the round's prepare job` |
| `prefixes` | the prepare ended, the prefixes it pushed not yet read, or the round's prepare ended with no answer it keeps, before the same prepare is asked once again (`landing-preparing.md`, 5) | a deadline: its members' branches read again | `the prepared prefixes on the members' branches` |
| `open` | the round's pull request opening (`landing.md`, 10) | its result | `the round's pull request to open` |
| `close` | the round's pull request closing (`landing.md`, 5.1) | its result | `the round's pull request to close` |
| `delete` | a member's branch in the landing deleted (`landing.md`, 5.1) | its result | `change C's branch in the landing to be deleted` |
| `post` | a gate's status posted (`landing.md`, 10) | its result | `gate NAME's status at change C's prefix` |
| `run` | the landing's own run of a gate (`landing.md`, 7.1) | its end, read at a deadline | `gate NAME's run RUN at change C's prefix` |
| `receipt` | a run of the same key in another landing, or a run whose receipt is not yet read | that receipt, read at a deadline | `gate NAME's receipt for change C's prefix` |
| `judgement` | a member's judgement gates at its prefix (`landing.md`, 3) | the change's verdict | `change C's judgement at its prefix` |
| `merge` | the turn's merge and its readback (`landing.md`, 8) | its result; the readback at a deadline | `the turn's merge` |
| `record` | a squashed member's pull request recorded merged (`landing.md`, 8) | its result | `the record of change C's pull request as merged` |
| `ancestry` | a single landing's read of whether the branch holds its member's head | a deadline | `whether the branch holds change C's head` |
| `read` | the top's read of the landing's facts (`landing.md`, 2) failed or found one not there yet: a member's pull request, a branch, a pull request, commits, a tree, the cleanup's reads; the top's | a deadline: its facts read again | `a fresh read of its facts from the forge` |
| room | a write waiting in the outbox's line (`landing.md`, 11), the top's | room (`forge/room.md`, 3) | `room in the forge's outbox` |

`run` names RUN where the landing knows it, and drops `RUN ` where it
does not. `turn` drops `, held by landing L` where no landing holds
it. A step that acts keeps no wait: `nothing; it steps next`.

## 3. No silent wait

- **Every wait names its wake.** A wait with no deadline is one of the
  table's whose wake is an effect's result, a change's step or a
  release; every other wait carries a deadline, the forge's `poll`
  (`landing.md`, 7.1), after which the landing reads its facts afresh.
  A fact that no write in flight will answer is read again at a
  deadline, never awaited.
- **No decision is lost.** A decision the root cannot carry out, its
  procedure step not due or its write refused before it is kept,
  commits nothing: the landing's state stays as it was, nothing is
  kept for the write, and the landing is woken to decide again in its
  next due step. A write waiting for room is the one wait whose wake is
  room alone.
- **After a round ends** (`landing.md`, 5.1), once its cleanup's close
  and deletions are answered, the next step is due at once and is one
  of: a prepare for the members above the last kept prefix, the opening
  of the round's pull request from its top, the top's proof and turn,
  or the landing's end. It reads each effect's result and the prepare
  of the current round only: one of an ended round reads as absent.
- **The policy's fuzzy** holds it as a property: over random landings
  with conflicts, earlier conflicts, head moves, cancels, red gates a
  search narrows to a blame and a cut, and stale bases, a harness
  playing the top keeps each effect the landing started outstanding
  and answers it some steps later, and each member's change takes its
  asks. Each wait with no deadline names an outstanding effect, a
  change's step or a release, and every landing reaches landed,
  cancelled or held within a bound of steps.

## 4. What a person reads

Under `landing.md`, 14. Ages are the engine's wall time as it builds
the document, less the time read, printed in the two largest of days,
hours, minutes and seconds (`2d 3h`, `4h 31m`, `37m 12s`, `12s`); a
time ahead of the engine's reads `0s`.

- **`landing list`:** its heading names the turn holder and how long
  it has held its turn, since its phase became `turn`: `landings of
  project P · turn: landing N for AGE`, or `turn: free`. Each live
  landing's line gives, after its phase, its step in `landing show`'s
  words and the age of its last event, any event its history keeps
  (`landing-history.md`): `landing N · PHASE · STEP · last event AGE
  ago · change C ...`. A landing whose row has kept no event's time, as
  one kept before this design, says `last event unknown`. An ended
  landing's line is as before. Its JSON adds `now`, the engine's wall
  time in nanoseconds; `turn_since`, when the holder's turn began, or
  null; and, on each landing, `step`, null once ended, and
  `last_event`, its time or null.
- **`landing show`:** a line `waits for: WORDS`, after its round or its
  heading where it has none, in section 2's words, for a live landing.
  Its JSON adds `waits`: null once ended, otherwise `{"for": WAIT,
  "change": C, "gate": NAME, "run": RUN, "landing": L}`, WAIT the
  table's first column (`room` for the top's) or `nothing`, each other
  field null where its words name none.
- **Kept by the top:** the time of its last event, written with each
  event; the time its turn began; its latest wait. Each is a field of
  the landing's row with a default (`landing.md`, 12): a row kept
  before has none.

## 5. Open questions

- **The public landing,** a person's page of it, shows no wait or age;
  whether it should is a later choice.
