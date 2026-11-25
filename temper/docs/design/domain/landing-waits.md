# A landing's waits

Provisional, 2026-11-24. What each step of a landing waits for, the
rule that no step waits on what nothing will answer, the bound on its
waits on the forge, and how a person sees a landing that stopped:
`temper ctl landing list`, `landing show` and the engine's log. It
details `landing.md`, 4.2 (*No silent wait*), 5.1 (*A round ends*) and
14. What each step does stays `landing.md`'s; a step's events stay
`landing-history.md`'s.

## 1. In one page

- **A step acts or waits on something it names.** Each wait the policy
  decides names what it waits for (section 2); the top keeps the
  latest on the landing, and `landing show` prints it.
- **No wait without its wake** (section 3). A step waits with no
  deadline only on an effect the landing started, whose answer wakes
  it, on a member's change taking its own step, or, held, on its
  release. Any other wait has a deadline, at which the landing reads
  afresh and decides again.
- **A wait on the forge is bounded** (section 3): reads the forge keeps
  answering, and writes it doesn't, while the landing takes no step that
  advances it hold it past the deployment's stall from its last advance,
  naming the read and how many it made; the engine logs a line as a wait
  for a fresh read starts, as it ends with the read taken, and as the
  landing holds.
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
| `prepare` | the round's prepare lives (`landing-prepare.md`, 1.2) | its answer | `the round's prepare job` |
| `prefixes` | the prepare ended, the prefixes it pushed not yet read, or the round's prepare ended with no answer it keeps, before the same prepare is asked once again (`landing-preparing.md`, 5) | a deadline: its members' branches read again | `the prepared prefixes on the members' branches` |
| `open` | the round's pull request opening (`landing.md`, 10) | its result | `the round's pull request to open` |
| `close` | the round's pull request closing (`landing-prepare.md`, 1.1) | its result | `the round's pull request to close` |
| `delete` | a member's branch in the landing deleted (`landing-prepare.md`, 1.1) | its result | `change C's branch in the landing to be deleted` |
| `post` | a gate's status posted (`landing.md`, 10) | its result | `gate NAME's status at change C's prefix` |
| `run` | the landing's own run of a gate (`landing-proof.md`, 2.1) | its end, read at a deadline | `gate NAME's run RUN at change C's prefix` |
| `receipt` | a run of the same key in another landing, or a run whose receipt is not yet read | that receipt, read at a deadline | `gate NAME's receipt for change C's prefix` |
| `judgement` | a member's judgement gates at its prefix (`landing.md`, 3) | the change's verdict | `change C's judgement at its prefix` |
| `merge` | the turn's merge and its readback (`landing-turn.md`, 1) | its result; the readback at a deadline | `the turn's merge` |
| `record` | a squashed member's pull request recorded merged (`landing-turn.md`, 1) | its result | `the record of change C's pull request as merged` |
| `ancestry` | a single landing's read of whether the branch holds its member's head | a deadline | `whether the branch holds change C's head` |
| `read` | the top's read of the landing's facts (`landing.md`, 2) failed or found one not there yet: a member's pull request, a branch, a pull request, commits, a tree, the cleanup's reads; the top's | a deadline: its facts read again; past the stall, held (section 3) | `a fresh read of its facts from the forge` |
| room | a write waiting in the outbox's line (`landing.md`, 11), the top's | room (`forge/room.md`, 3) | `room in the forge's outbox` |

`run` names RUN where the landing knows it, and drops `RUN ` where it
does not. `turn` drops `, held by landing L` where no landing holds
it. A step that acts keeps no wait: `nothing; it steps next`.

## 3. No silent wait

- **Every wait names its wake.** A wait with no deadline is one of the
  table's whose wake is an effect's result, a change's step or a
  release; every other wait carries a deadline, the forge's `poll`
  (`landing-proof.md`, 2.1), after which the landing reads its facts afresh.
  A fact that no write in flight will answer is read again at a
  deadline, never awaited.
- **A wait on the forge is bounded** (2026-11-24, revised for
  `../draft/bounded-waits.md`, 4.1). A read the forge answers the same
  way at each deadline (a page it keeps refusing, a head that never
  shows, a comparison that never changes) would keep a landing waiting
  for good: on dev.fabrig.ai on 2026-11-24 landing 15091's round read
  one 24-commit comparison more than 400 times, answered each time,
  waiting four hours while the forge answered every poll. A bound
  counted from a run of `read` waits alone missed a landing whose waits
  changed between reads, so the bound counts from the landing's last
  advance, which no read again moves:
  - **The last advance** is the landing's last step that committed
    anything to its row but its wait, its last event's time, its turn's
    start and its hold's words; or its release; or the engine's start,
    which reads every fact afresh (`landing.md`, 15); or the end of a
    cause that stops its clock (below). A step that only waits again,
    on the same wait or another, is no advance. The running engine
    keeps it, unstored, with the count of reads the top made for the
    landing since it, as it keeps a change's one-step hold
    (`landing-turn.md`, 2).
  - **A wait on the forge** is a step's wait on a read again at a
    deadline or on the forge's answer to a write the landing started:
    `prefixes`, `open`, `close`, `delete`, `post`, `receipt` for its own
    run, `merge`, `record`, `ancestry` and the top's `read`. Not one: a
    `run` while its gate's own clock counts it (`landing-proof.md`,
    2.1), which bounds it; a wait behind another task (`ready`,
    `judgement`, `turn`, a `receipt` for another landing's run), which
    that task's waits bound; `prepare`, whose job has its own bound;
    `release`; and room, whose bound is `forge/waits.md`, 7's.
  - **Its clock** counts from the last advance. It stands still while
    the landing's forge is not connected or a read of it is retried for
    an outage, as a change's stall does (`forge/change/stalls.md`, 1,
    *A wait for access is not one*), and counts afresh from that cause's
    end.
  - **Past the deployment's stall,** `forge.change_policy.stall_ms`,
    the bound every wait of a change on the forge has
    (`forge/change/stalls.md`, 1), a landing still waiting on the forge
    holds (`landing.md`, 4.2) instead of waiting again, its hold `read`,
    words `read: WHAT, for change C: WHY; R reads since HH:MMZ, none
    advanced the round`. It is woken at the bound, so a read that never
    answers (one the forge client keeps in its line for room) holds
    there too. The stall, not a count of reads, since a deadline's
    length is the forge's `poll`, which moves; the stall is the time a
    person is owed word of a wait, as a change's is. A release, as any
    landing hold's, prepares again in a new round, keeping its receipts.
  - **WHAT** names, for a wait on a read, the read the landing last
    made, in skein's terms, commits in hexadecimal at the forge's
    length: `commits after BEFORE up to AFTER, page P`
    (`Read::Commits`); `branch BRANCH`; `pull request #N`; `pull request
    #N's reviews, page P`; `the comparison of BEFORE and AFTER`
    (`Read::Compare`); `tree OBJECT, page P`; any other read, its name
    in skein's `forge/vocabulary.md`, 1.2, in lower case (`run`, `runs`,
    `checks`). For a wait on a write's answer, it is the wait's words in
    section 2's last column. `, for change C` follows where the wait is
    a member's (its branch, its pull request, its prefix's commits), C
    its change task, and is dropped for the landing's own (the train's
    tip, the round's pull request).
  - **WHY** is the last answer: a refusal in the forge's effect words
    (`temper-engine-domain-forge-vocabulary`'s `error_words`, such as
    `past ` and the bound), `not there yet` for an answer the step can't
    take yet, `answered` for an answer it took that advanced nothing, or
    `no answer` where none came.
  - **R** counts the reads since the last advance, HH:MMZ that advance's
    time; the clause is dropped where R is 0. For instance, `read: the
    comparison of ceebfe4a3e16 and 5ddd6bc722da, for change 15090:
    answered; 412 reads since 02:07Z, none advanced the round`. The
    words are cut to the history's reason bound as any hold's are
    (`landing-history.md`, 2.2, `held`), and the hold's kind stays
    `read` on the client's wire, so no version moves.
  - **A run of read waits** is the log's unit (section 4) and bounds
    nothing: it starts at a step that waits `read` while the landing is
    in none, and lasts while each of its steps waits `read` again; any
    other decision ends it, and so do a release, the landing's end and a
    restart.
- **No decision is lost.** A decision the root cannot carry out, its
  procedure step not due or its write refused before it is kept,
  commits nothing: the landing's state stays as it was, nothing is
  kept for the write, and the landing is woken to decide again in its
  next due step. A write waiting for room is the one wait whose wake is
  room alone.
- **After a round ends** (`landing-prepare.md`, 1.1), once its cleanup's close
  and deletions are answered, the next step is due at once and is one
  of: a prepare for the members above the last kept prefix, the opening
  of the round's pull request from its top, the top's proof and turn,
  or the landing's end. It reads each effect's result and the prepare
  of the current round only: one of an ended round reads as absent.
- **The policy's fuzzy** holds it as a property: over random landings
  with conflicts, earlier conflicts, head moves, cancels, red gates a
  search narrows to a blame and a cut, and stale bases, a harness
  playing the top keeps each effect the landing started outstanding
  and answers it some steps later, some of its round's closes and
  deletions refused, and each member's change takes its asks. Each wait
  with no deadline names an outstanding effect, a change's step or a
  release; a refused close or deletion holds the landing `cleanup` at
  its next decision, and is decided again only after a release
  (`landing-prepare.md`, 1.1); and every landing reaches landed,
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
  field null where its words name none. A landing held past its wait on
  the forge's bound (section 3) says `held: ` and the hold's words above
  it, `held: read: WHAT, for change C: WHY; ...`, as each other hold names
  its cause; the hold's kind is `read`, its words kept with it, a kind
  the client's wire carries as it carries the others, its layout changed
  for it (`../protocol/wires.md`, 9).
- **Kept by the top:** the time of its last event, written with each
  event; the time its turn began; its latest wait. Each is a field of
  the landing's row with a default (`landing.md`, 12): a row kept
  before has none. Its last advance, its reads since it and a run of
  read waits (section 3) are not kept.
- **The engine's log** (2026-11-24) has a line as a run of read waits
  starts, as it ends with the landing deciding anything but `read` or
  its hold, and as a wait on the forge holds, so that an operator sees a
  landing waiting on the forge without asking for it. L is the landing's
  task, N its round (`Landing.preparation`), READ the read a run of read
  waits last waited on, C and WHY as section 3 names them, AGE the run's
  time on the first two lines and the time since the last advance on
  the third, as this section prints ages, and WORDS the hold's words
  (section 3):

  ```
  BINARY serve: landing L round N waits for a fresh read: READ, for change C: WHY
  BINARY serve: landing L round N has its fresh read after AGE: READ, for change C
  BINARY serve: warning: landing L round N held after AGE: WORDS
  ```

  `, for change C` is dropped where section 3 drops it. A run logs each
  of its two lines once at most: its deadlines' reads log nothing more.
  A run a release, the landing's end or a restart ends logs no second
  line. A hold logs its line once.

## 5. Open questions

- **The public landing,** a person's page of it, shows no wait or age;
  whether it should is a later choice.
