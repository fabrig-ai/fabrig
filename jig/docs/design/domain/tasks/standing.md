# A task that stands

Provisional, 2026-11-11. An agent task made to stand: it lives until it
is cancelled, its budget is carved again each period, and its runs work
in sessions, each resuming while its model's prompt cache is warm, and
each starting fresh from the few lines it keeps, its Now. It is the
middle kind of agent task, between a run that finishes once and a
conversation a person is in: an application's agent that owns some work
for a project's life, works in short sessions and answers to a person.
It revises `README.md`, sections 3, 5 and 6, `lifecycle.md`, sections 2,
6 and 9, `messages.md`, section 6, `../engine/executors.md`, section 4,
`../engine/briefs.md`, sections 1 and 2, `../authority.md`, section 7,
`../people/acts.md`, section 1, and `../../protocol/configuration.md`,
section 3, which say where it differs. It takes up `../../draft/notes-memory.md`, section 4, for a
standing task's Now alone.

## 1. In one page

- **Made standing, never finished** (section 2): `ended_by` `standing`,
  an agent's, requested by a person, not a conversation, untracked. It
  is never told that nothing is awaited, a run's finish parks it, and only
  a cancel ends it.
- **Its person is its requester** (section 3): its questions,
  escalations and proposals reach that person as any party-requested
  task's do, and a move gives it another person, its open questions with
  it.
- **A budget per period** (section 4): carved again from its project's
  spend at each period's start, as a standing procedure's is, and again
  at a run's end where a live run made it wait.
- **Sessions** (section 5): within one, each run resumes its transcript;
  code ends it at the first of a quiet span longer than its model's
  cache keeps, its context's bound, its work closed, a waking message
  its parts don't offer, or its rules changed. The next wake starts
  fresh. An engine restart ends none.
- **A talk** (5.6): a person opens one with their words, and the session
  it begins stays warm while they are there; leaving ends it.
- **Its Now** (section 6): the note described `now` in its own scope,
  written by its runs, whole in every brief, read by its project's
  people; each write leaves its first line in the activity, so that
  each session's last says what it did.

## 2. A task that stands

- **Made standing.** A task's `ended_by` (`lifecycle.md`, 9) may be
  `standing`, fixed at its making as every ending is. Its shape: an agent
  executor; its requester a person, not the deployment and not a task; not
  a conversation; untracked. Any other shape is refused as its ending's
  malformation, as a requester's or a closer's is, at a batch, a
  template's start and a restart alike. A run's tools take no
  `ended_by` (`../../protocol/configuration.md`, 3.2), so only a party's
  batch, or a template a party starts, makes one.
- **It runs Open** (`../hosts/charters.md`, 1): granted the wait and never
  delivery, as a task its requester ends.
- **Never told that nothing is awaited.** `lifecycle.md`, 2's rule passes
  it by, as it passes a task its requester ends: idle with nothing
  pending, it waits for its next wake, numbers no `NothingAwaited`
  notice and spends no try.
- **A finish parks it.** A run that answers finished, done or failed,
  with follow-ups or none, is committed as parked: its turns first, then
  its saved state, with no try spent, and its result and follow-ups
  discarded, as a task its requester ends (`lifecycle.md`, 9). The fences
  of saved work, of charge and of messages are checked first, as for any
  finish. Its activity takes a line, `Stood`, naming the attempt and
  holding the result's words, cut to 512 bytes at a character's edge, so
  that what the run said shows. A failure that is not a finish keeps its
  retry policy. A finish committed as parked ends the first activation, as
  a park idle does (`messages.md`, 6).
- **Its brief carries no contract,** as one its requester ends does
  (`../engine/briefs.md`, 1): a standing task has no result to give.
- **Only a cancel ends it** (`changes.md`), as a recurring task ends
  (`README.md`, section 5).
- **No stall's bound while idle with delegates.** `README.md`, section
  6 bounds how long an agent task may sit idle with delegates still
  live; a standing task has no such bound, since a goal it started may
  run for days.
- **Its tries count per session.** Its tries per class of failure
  (`lifecycle.md`, 6) start again as each of its sessions begins
  (section 5.1), as a release starts them again, so that a failure in one
  week's work does not count against the next. Past them within a
  session, it is held as any task is.

## 3. Its person

- **Its requester is a person,** its person. What it asks its requester,
  the escalations of its holds, and its proposals beyond its authority
  reach that person's inbox as any party-requested task's do
  (`proposals.md`; `messages.md`, 5). It is the requester of what it
  starts, so its delegates' questions reach it, and what it cannot settle
  it asks its person.
- **A move gives it another person** (`changes.md`): a party who may
  amend it makes themselves its requester, its open questions following
  it to their inbox.
- **Its application may move it** to another person of its project, as
  the deployment, when its person leaves the project: the application's
  root names the person, and the move's checks and effects are
  `changes.md`'s, its open questions carried, the task funded anew from
  that person's pool for the open period (`../authority.md`, section 7).
  Where its application's inbox holds its people's answers, it asks its
  person by a person task addressed to them (`messages.md`, 5), so the
  application's root makes the move and addresses those person tasks to
  the new person in one compound decision (`changes.md`, Several
  operations, one decision); no new act.
  A move refused leaves it as it is, for the application to try again.

## 4. Its budget per period

- **Carved again each period,** as a standing procedure's is
  (`../authority.md`, section 7, *Standing work renews by period*;
  `README.md`, section 5): at a period's start, what it spent and holds
  reserved is kept in the old period's ledger, its delegates keep their
  reservations in the period that funded them, the fresh period reserves
  its whole budget, and the tasks it has made count from one again. Its
  budget is the amount it was made with, or as an amendment set it.
- **A run live at the period's start** holds a run's reservation, so the
  task renews when that run ends: a renewal is judged at each period's
  start and again at the end of each of its runs, while it is funded from
  an older period than its project's open one.
- **A renewal the fresh period cannot fund,** its project's spend having
  too little left, leaves it as it is, judged again at its next run's
  end and the next period's start.
- **Past its budget,** with too little left for its next run's reserve
  (`lifecycle.md`, 6), it is held for its budget, and the hold's
  escalation reaches its person, as money always waits for a person. A
  renewal lifts that hold in the decision that renews it, its cause
  passed, and withdraws its escalation. Its person may release or widen
  it before then, funding it as any widening is (`../authority.md`,
  section 7).

## 5. Sessions

### 5.1 A session

- **A stretch of its runs,** from the wake that begins it to the end code
  judges (5.2). Within it each run resumes the session's transcript
  (`../engine/executors.md`, 2 and 4); across sessions each starts fresh
  from a brief (5.4). A standing task whose charter starts fresh has a
  session of one wake.
- **Its record,** kept with the task (`README.md`, section 3) and saved
  with each decision that changes it, `None` while no session is open:
  - `first`: the attempt that began it, which names it;
  - `reasons`: the reasons its first run ran for (`messages.md`, 6),
    which chose its parts;
  - `last`: the wall time of its last committed turn, or of its first
    run's claim before any turn.
- **Begun at a claim.** A run claimed for a wake while no session is
  open, or whose open session the wake ends (5.2), begins a new one: its
  claim commits the old session's end, the new record, and the task's
  tries started again (section 2). Each committed turn of its runs moves
  `last`.
- **Judged at a wake.** A session's ends at a wake (5.2) are judged when
  a wake makes the task due and its run is prepared: a message, an entry
  waiting for its decision, or a timer. A retry, the continuation of a run
  parked kept (`lifecycle.md`, 8), and an attempt interrupted or lost and
  due again go on within their session, unjudged.
- **The open activation tells them apart.** The task's record keeps
  `activation_open`, set at the claim that begins an activation and
  cleared only when that activation ends idle (`lifecycle.md`, 2): a
  claim while it is clear is a wake's, and judged; one while it is set
  continues the activation, unjudged. The task's reasons cannot tell
  them apart, for an activation woken by an always-offered message alone
  has none. It is restored with the task, so an attempt interrupted at a
  restart still goes on.

### 5.2 Its ends

A session's end is a pure function of its record, the wall time of the
wake's preparation, the session's context, its charter, the kinds
that woke it and its talk (5.6). At a wake, the first of:

- **Left:** its talk has lapsed (5.6).
- **Quiet:** more than its quiet span (5.3) has passed since `last`,
  and no talk is open.
- **Bound:** its context is past its bound (5.3).
- **Unoffered:** a kind that woke it (`messages.md`, 6) is not among
  those its session offers: the always-offered, and the kinds named in the
  `on` of each part its session's reasons read.
- **Fresh:** its run cannot resume: its charter starts fresh, or its
  transcript's window cannot be resumed, closed, cut or past
  `resume_bytes` (`../engine/executors.md`, 2).

Judged where they happen, not at a wake:

- **Closed:** its run parks with its work closed: the task left idle with
  no live delegate, no question of its own still open, and no proposal of
  its own pending, and no talk open (5.6). Its timers and subscriptions
  do not count: they are standing interests, not work it started.
- **Rules:** its instructions change: a move to a newer revision of the
  settings (`../engine/settings.md`, 4), or an amendment giving or
  changing its own instructions (`changes.md`). A live run goes on, to its
  end, under the instructions it started with.
- **Talk:** a person opens a talk (5.6), so that the talk begins a
  session of its own. A live run goes on, to its end, as at Rules.
- **Left:** the talk's person leaves it (5.6). A live run goes on, to its
  end, as at Rules.

Nothing else ends a session: neither a restart (5.5), nor a failure, nor
a hold. Each end writes a line, `SessionEnded`, in its activity, naming
the session by its `first` and saying which end it was.

### 5.3 The quiet span and the bound

- **The session's model** is its task's main model, from its LLM choice,
  or else its charter's first (`README.md`, section 3).
- **The quiet span** is nine tenths of the model's prompt-cache life, its
  `cache_life_ms` (`../../protocol/configuration.md`, 3): a little under
  it, so that a wake within the span finds the cache warm. A model that
  declares none, or 0, keeps no prompt cache: its span is zero, and each
  wake of its task starts a session.
- **The bound** is half the model's window. The context it bounds is the
  bytes of the session's transcript window, as the core loads it for the
  run (`../engine/executors.md`, 1), the count `resume_bytes` bounds,
  against the window at four bytes a token: the engine holds a
  transcript's bytes, not its tokens.
- **Both shares are defaults** (section 9).

### 5.4 Parts and resuming

- **A charter whose runs resume may hold parts** (`messages.md`, 6). A
  session's first run reads its charter's instructions and the parts its
  reasons meet; each later run of the session resumes, reads the
  instructions its session's reasons chose, which keeps the cache, and is
  offered what those parts offer.
- **A message the session doesn't offer** keeps its place in the inbox
  (`messages.md`, 2 and 6): one whose kind wakes the task ends the session
  at that wake (Unoffered), and the run that follows, fresh, reads the
  parts for it; one that doesn't wake it waits for the next session.
- **Another task under such a charter,** one that does not stand, reads
  its charter's instructions alone, every run, and is offered every
  message, as under a charter with no parts: only a standing task has
  sessions in which to read new parts.
- **A session's first run's brief** carries its Now whole (section 6)
  and no transcript tail (`../engine/briefs.md`, 1): the Now takes the
  tail's place. A retry within the session carries its tail as any
  retry's does.

### 5.5 Across a restart

- **The record is the task's,** restored with it at restart
  (`../engine/restart.md`), and its times are wall times, so a restart
  ends no session. A run on a worker outside the engine's process is
  adopted and goes on (`../engine/restart.md`, its third step); a parked task's
  next wake is judged against its record as if no restart had come, and
  resumes while its cache is warm. A run in the engine's own process
  ends interrupted (`lifecycle.md`, 2), and its next attempt goes on
  within its session.

### 5.6 A talk

A person talking with a standing task, in the terminal or on the web:
their words wake it, its runs answer in their ordinary text, as an Open
run's reply is (`../hosts/charters.md`, 1), and the session stays warm
while they are there.

- **Opened by their words.** A party's `Talk` (`../people/acts.md`, 1)
  carries words, delivered as a `Message`'s are, a person's words, always
  offered and never batched (`messages.md`, 3 and 6). With no talk open,
  it opens one, and ends the task's open session, if any (5.2, Talk), so
  that the talk's first words find a session of its own: a fresh run,
  its brief holding its rules with the parts for words, its Now whole and
  what woke it (5.4). With that party's talk open, it carries their words
  into it. A standing task holds one talk at a time.
- **Its record,** kept with the task beside its session and saved with
  each decision that changes it, `None` while no talk is open:
  - `party`: the person talking;
  - `opened`: the wall time of the `Talk` that opened it;
  - `last`: the wall time of their last `Talk`.
- **While it is open,** its session is not ended by Quiet nor by Closed
  (5.2): the person may read and think for longer than a cache keeps, and
  each answer leaves the task's work closed. Bound, Unoffered, Fresh and
  Rules end it as ever; the next run starts fresh within the talk, and
  the session it begins is the talk's.
- **It lapses** once more than `TALK_LAPSE`, 30 minutes, has passed since
  the later of its `last` and its session's `last` (5.1): its person has
  gone without leaving. A lapsed talk is judged as one left: at a wake,
  the claim ends the talk and its session (5.2, Left), and a `Talk` by
  any party first ends a lapsed talk so, then opens its own. Nothing else
  watches a lapse: an idle task spends nothing on it.
- **Left by `Leave`,** from its person or a maintainer: the talk ends,
  and its session with it (5.2, Left), in that decision. A run live then
  goes on to its end, and writes its Now before it parks, as every run
  does (section 6), so the session's last Now is the talk's. A `Leave`
  while no talk is open, or by another member while one is, is refused.
- **Refused:** a `Talk` to a task that does not stand, as no talk; one
  while another party's talk is open and not lapsed, naming that party;
  and words a `Message` would refuse (`messages.md`, 2), as it refuses
  them, with nothing opened.
- **Its lines:** `TalkOpened { party }` as it opens, and
  `TalkEnded { party, lapsed }` as it ends, left or lapsed, each in the
  task's activity, so that a client reads a talk as the activity between
  them (`../engine/executors.md`, 2), its words and the runs' turns,
  never resumed once ended.
- **Restored with the task** (5.5): its times are wall times, so a
  restart neither ends nor lapses it.

## 6. Its Now

- **Its own note scope:** the goal scope named by its own number, as a
  root conversation's (`../engine/briefs.md`, 2): its runs write and read
  it, and its project's people read it as any note of the project.
- **Its Now** is the lowest-numbered note in that scope whose description
  is `now`: a page at most of where things stand, what it waits for and
  why, what it does next and when, what it asked and what it decided. Its
  runs write it with the note tool, a first version whole, then
  revisions by its name and the revision they read
  (`../engine/briefs.md`, 2).
- **Whole in every brief:** each run's brief of a standing task carries
  its Now, a required section of the core's after the task's, titled
  with the note's name, its revision and when it was written, or saying
  that none is written yet. It is placed whole within its own budget,
  `brief_core_budgets.now` (`../../protocol/configuration.md`, 3); a
  Now past it is cut there, the section saying its bytes and the bound
  so that the run shortens it, and it never fails the brief.
- **Its period beside it:** the section says, under its title, what the
  task has spent in the period that funds it and its budget there, as
  `../authority.md`, section 7 keeps them: "This period: spent S of B."
  in the deployment's unit, so that a run reports what it spent and
  knows what is left before it starts work.
- **Each write leaves a line.** When a standing task's run writes its
  Now, made whole or revised, the core adds `NowWritten { attempt,
  line }` to the task's activity in the decision that saves the note:
  the run's attempt, and the body's first line, cut to 512 bytes at a
  character's edge, as `Stood`'s words are. So a session's last
  `NowWritten` says what it did, the first line being where its runs
  say it, and a client says which session wrote the Now without reading
  the note.
- **Written before every park.** The section's words ask the run to
  write its Now before it parks, so that whichever run turns out to be
  its session's last has left it current. A run that writes none leaves
  the last; its session's end is a line in its activity all the same
  (5.2).
- **Read by people** with the notes read of its scope, by the
  description `now` (`../../client/domain/contract/README.md`, section
  4), as any
  note is read.

## 7. Its forms

Each is a contract the changes that build this document share:

- **The tasks child** (`jig-domain-tasks`): `EndedBy::Standing`; a
  `Session { first: u64, reasons: Reasons, last: Wall }` and
  `TaskRecord.session: Option<Session>`; `TaskRecord.activation_open:
  bool` (5.1); `SessionEnd`, one of `Quiet`,
  `Bound`, `Unoffered`, `Fresh`, `Closed` and `Rules`; and the activity
  lines `Line::Stood { attempt: u64, words: Box<[u8]> }` and
  `Line::SessionEnded { attempt: u64, end: SessionEnd }`.
- **The store** (`../../protocol/store/records.md`): `EndedBy`'s
  `standing` in the tasks, people and settings families; the record
  `Session` and its option on `TaskRecord`; the activity lines `stood`,
  its words at most 512 bytes, and `session_ended`, and the enum
  `SessionEnd`, `quiet`, `bound`, `unoffered`, `fresh`, `closed`,
  `rules`; the settings family's model's `cache_life_ms: u32`, 0 where
  none is declared.
- **The client wire:** `EndedBy`'s `standing` in `TaskFlags`, and the
  activity lines `stood` and `session_ended`.
- **Configuration** (`../../protocol/configuration.md`): a template's
  `ended_by` `"standing"` (3.2); a model's optional `cache_life_ms`, its
  prompt cache's life in milliseconds, 0 or left out for none (3); and a
  profile's `brief_core_budgets.now`, 4,096 bytes in both profiles.
- **The core's model** (`jig-domain-core`): `Model.cache_life_ms: u32`,
  read by the session's quiet span (5.3).

A talk and the Now's line (5.6 and section 6) add:

- **The tasks child:** `Talk { party: u64, opened: Wall, last: Wall }`
  and `TaskRecord.talk: Option<Talk>`; `SessionEnd`'s `Talk` and `Left`;
  the constant `TALK_LAPSE`, 30 minutes; the session's judgement taking
  the talk, open, lapsed or none, beside its other facts; and the
  activity lines `Line::TalkOpened { party: u64 }`,
  `Line::TalkEnded { party: u64, lapsed: bool }` and
  `Line::NowWritten { attempt: u64, line: Box<[u8]> }`, its line at most
  512 bytes.
- **The store:** the record `Talk` and its option on `TaskRecord`;
  `SessionEnd`'s `talk` and `left`; the activity lines `talk_opened`,
  `talk_ended` and `now_written`. Every row stored before them decodes
  as it did, a task with no talk.
- **The people child and the core:** the party acts `Talk { project,
  task, words }`, answered with the words' message number as a
  `Message` is, and `Leave { project, task }`, answered with nothing
  but its success (`../people/acts.md`, 1); their refusals, a task that
  does not stand, another party's talk open, naming its party, and no
  talk of theirs to leave.
- **The client wire** (`../../client/domain/contract/requests.md`, 2):
  `Ask::Talk { project: u32, task: u64, words: Text }` and
  `Ask::Leave { project: u32, task: u64 }`, offered on a standing task's
  document to whoever may send it a `Message` (`ActKind`'s `Talk` and
  `Leave`); the task's document's `talk: Option<{ party, opened, last }>`
  and `session: Option<{ first, last }>`, from its record; and the
  activity lines `talk_opened`, `talk_ended` and `now_written`.

## 8. The world

Focused, in the tasks child's world and the core's (`../testing.md`):

- a standing task made by a person's batch and by a template; one with a
  procedure, a person or a job for its executor, a task or the deployment
  for its requester, flagged a conversation, or tracked, refused as its
  ending's malformation;
- idle with nothing pending, it numbers no `NothingAwaited` notice and
  spends no try, across parks and a restart; idle with a live delegate
  long past the stall's bound, it is not held;
- a run finishing done, with a failure and with follow-ups, each
  committed as parked, the result discarded, a `Stood` line with its
  words cut at 512 bytes, the task live; a cancel ending it;
- at a period's start its budget carved again, its delegate keeping the
  old period's reservation; a run live at the start, the renewal made at
  its end; held for its budget and lifted by the next renewal, its
  escalation withdrawn;
- the session's end as a pure function, each end and nothing else
  ending it; a run resuming inside the quiet span and starting fresh past
  it, its brief holding the Now and no tail; no resume at all for a model
  with no prompt cache; a waking message no part of the session offers
  ending it, the fresh run reading that message's part; closed work and a
  move each ending it; tries started again with a new session and kept
  across a retry; a restart between two wakes leaving the session open
  and the next run resuming;
- a non-standing task under a resuming charter with parts reading its
  instructions alone;
- its Now whole in each brief, cut past its budget saying so, none yet
  said; a person of its project reading it;
- a talk: a person's `Talk` ending a warm session and the next run
  starting fresh with the Now in its brief; the talk's later words
  resuming past the quiet span, and a park with its work closed ending
  nothing while it is open; Bound or a move ending its session and the
  next run starting fresh within the talk; `Leave` ending the talk and
  its session at once, a live run going on to its park; a talk lapsed
  past `TALK_LAPSE` ending at the next wake's claim as left, and a
  second party's `Talk` refused while the first is open and admitted
  once it has lapsed; a `Talk` to a task that does not stand and a
  `Leave` with no talk refused; a talk restored across a restart;
- each write of its Now adding a `NowWritten` line with its first line,
  cut at 512 bytes; another note, or another task's `now`, adding none;
  and its brief's Now section saying its period's spend and budget.

Fuzzy, beside those below: random talks, words, leaves and wake times,
where no session resumes past its quiet span unless a talk was open and
unlapsed at that wake, and every talk's `TalkOpened` has one
`TalkEnded` once it ends.

Fuzzy: random wake times and restarts, where no run resumes past its
quiet span and every fresh run of an open task follows a `SessionEnded`
line; random periods, runs and cancels, where a standing task ends only
by a cancel and its spend is counted once across renewals (the
accounting referee).

## 9. Open questions

- **The quiet span's and the bound's shares,** nine tenths of the cache's
  life and half the window, set from the numbers of the first standing
  tasks that run.
- **An account that takes a longer cache,** where a provider offers an
  hour at a price and an account takes it, so that the span is the
  account's and the model's together.
- **The context in tokens,** reported by the run with its turns, in place
  of the transcript's bytes.
- **A standing task requested by a task,** so that one standing task may
  hand work to another.
- **A talk's lapse,** `TALK_LAPSE`, 30 minutes, set from the first
  talks' numbers; and whether a talk may hold two people at once, where
  one talk at a time is refused to the second today (5.6).
