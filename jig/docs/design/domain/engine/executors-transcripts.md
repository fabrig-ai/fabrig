# The engine's executors: transcripts

## 1. Transcripts

- **A transcript belongs to its task,** whichever runs added to it. Its
  conversation is the turns from the last run that started fresh, and
  its current window the conversation's turns from the last that opens a
  window: a window turn, which a compaction made, or the conversation's
  first (smith's `protocol/transcript.md`, section 3). That conversation
  is the transcript's, which every agent task has; a task flagged
  *conversation* is another matter, whose turns are entries for its
  requester (`views.md`, 4).
- **Beside it, the task's activity:** the words parties wrote to it as
  they wrote them, its runs' beginnings and ends, the results and news it
  heard and the decisions made about it, kept in the order they
  committed, so a client reads a conversation whole after its words have
  been taken from the inbox (`../../client/domain/contract/reads.md`, section 1).
  A failed run's end retains its class and the optional invalid-result
  kind computed for that attempt (../tasks/lifecycle.md, 7), beside the task record's
  current reason. Starting another attempt clears the current reason,
  never the ended run's activity.
  A message is a line in the activity of the task that hears it
  (../tasks/messages.md, 1): a party's answer to a question a task heard is that
  task's own line; a task's answer to the delegate that asked is the
  delegate's line, never repeated in the answering task's activity,
  whose turn holds the call that sent it; a read of the question joins
  its answer by a point read (`../../protocol/store/loads.md`, 3). Who may read
  an answer is the client's contract's (`../../client/domain/contract/reads.md`,
  section 1).
- **Committed turn by turn.** A run sends each turn up as it ends: what
  the LLM said and the calls it made, their results, what it spent, the
  last inbox message it read, and whether it opens a window. The core
  commits the turn with its spend, with the messages offered to its
  attempt up to that one taken but those it says the run refused
  (../hosts/hub.md, 4): a message its attempt wasn't offered, by its parts or
  a pace, stays, whatever the fence passes (../tasks/messages.md, 2 and 6); and, when it
  opens a window, with its task's current window starting there; then
  it acknowledges it. A host keeps each turn until it is acknowledged,
  and a run with too many unacknowledged turns waits (../hosts/hub.md, 4).
  Admission is whole: the tasks child validates the attempt, turn and
  offered read-through before changing its spend: the read-through names
  a message offered to the attempt, still waiting or, for a proposal or
  an escalation, decided since it was offered (../tasks/proposals.md), and
  none past the attempt's highest offer; a refused turn changes nothing,
  and a duplicate accepted turn changes nothing again. The core keeps
  the ids it offered an attempt with the run's proof until a turn's
  fence reaches them or the run refuses them, so such a fence survives
  the entry's decision and the core's restart, and they hold their room
  in the run's inbox until then (4).
- **The core never looks inside.** A turn is bytes of the agent's own
  conversation vocabulary, bounded, which a client's protocol layer
  renders. The core reads only its size, its spend, what it took and
  whether it opens a window, which the turn carries beside its bytes.
- **What a turn's calls to the engine made is the core's to say.** As
  it commits a turn, the core writes beside it, from the parts of the
  answers it committed for the turn's calls (3), what each call made,
  under the call's name (its completion and its position,
  `../../protocol/channel/README.md`, section 6), as
  `../../client/domain/objects.md`, section 10, shows them: the tasks a
  delegation made, the proposal a `propose` made, a decision's object,
  what an effect made, a note. A call it holds no part for names
  nothing. No layer reads what a call made from its answer's words,
  which its protocol layer renders for the LLM (../hosts/charters.md):
  smith's transcript keeps a host tool's answer as text alone (smith's
  `protocol/transcript.md`, section 2).
- **Resuming.** A run that resumes is given its current window only,
  within the resume limit (the deployment's bound on the current
  window's bytes, across the attempts it spans, as the window crosses to
  a host: `../../protocol/channel/owners.md`, 1.1), and, as a note after
  it, the calls committed after its last turn, with their answers. A
  start that crosses a worker's link carries those answers within the
  link's room: the newest whole and each older host answer cut to fixed
  words saying the call ran and how many bytes its answer had
  (`../../protocol/channel/liveness.md`, 1, *A resumed start fits its room first*),
  so however many calls a run made since its last turn, it is told of
  each and resumes. The core keeps every call as committed. One
  past the limit starts fresh from a brief that carries the transcript's
  tail. Earlier windows' turns stay readable by parties
  (`../../client/domain/contract/reads.md`, section 1), and are never sent to a
  run. A transcript the agent cannot use fails its run as transient,
  saying so, and closes its window (its run proof's `CurrentWindow`
  marked closed): the next run is prepared fresh from that window's
  tail, and never resumes it. **Only the window's own history closes
  it:** a transcript the agent refuses or reports unusable, or a window
  whose load is unread. Any other failure, transient ones included (a
  preparation, a brief's gathering, a workspace another run holds), and
  an interruption (`restart.md`, 5; ../hosts/hub.md, 6), leave the window open, and the
  next attempt resumes it within the resume limit, whatever tries the
  task spent before. If a closed window's tail is unread, preparation
  continues fresh without the tail and spends no transient try. An
  open window whose load is unread counts one transient preparation
  failure and is closed; its retry continues fresh without that tail. A
  failed load, rather than an unread window, remains a preparation
  failure and closes nothing.
- **Parts kept between turns** (`../../protocol/channel/records.md`, 5;
  smith's `domain/turn-parts.md`): the core keeps each part a run sends
  as it keeps a settled call, under its task, attempt and number, with
  its turn, thread, call, kind and bytes, then acknowledges it, a copy
  already kept acknowledged again and dropped. Parts have a room of their
  own, `core.call_records` of them, apart from the named calls', so a
  part never takes a call's room (2026-11-25: sharing it refused named
  calls on the reference deployment all day). A part that finds its room
  full never waits: every part its task keeps goes, and the part is
  acknowledged unkept, so the run goes on and a resume finds no answer
  for the cut turn and plans it afresh, as before parts. A committed turn retires
  every part of its task from an earlier attempt and those of its own
  attempt whose turn is at most its number; the task's end retires the
  rest. Each use checks the part's own place
  (skein/docs/foundation/programming-model.md, 10.4): keeping a part
  asserts that its key is new to the room and that the room had its
  place in the same step, naming the part and its task, and a
  retirement erases only the parts it found, so each goes once. A resumed run is given those kept, by attempt, then number,
  when its window resumes and the window's bytes and theirs fit the
  resume limit: main's parts first, then each sub-agent's all or none in
  call order while they fit, none where main's do not. A run that
  starts fresh is given none. The core family keeps them as a value
  type of its own, an added part with its version unmoved
  (`../../protocol/store/layouts.md`, 1), its body held as a turn's,
  `holds smith-transcript Turn`.
- **Resumed only where it ran.** A window is resumed only on the
  endpoint it ran on, program to direct or back alike. The endpoint
  names the backend too: a program source runs on its model's program
  endpoint, any other source on its provider's direct endpoint
  (`Settings::model_endpoint`, `../../protocol/configuration-engine.md`,
  section 1). The run proof keeps the endpoint its claim chose for its
  main model, `RunProof.endpoint` (core-v6, an added field whose default
  is none). A claim whose main model's endpoint is another than its
  previous proof's resumes nothing: a task moved from a program login to
  a key or back, or to another provider. It starts fresh from a brief
  that carries the transcript's tail, as one past the resume limit does,
  and its activity says so, `Moved` (*Why a run began fresh*).
  That is a transition that counts nothing: no failure, no transient try,
  no refusal, and no window closed. A standing task's session ends there,
  `Fresh` (`../tasks/standing.md`, 5.2), so its next run is its session's
  first and reads its Now whole and its notes' index, as on any fresh
  session.
  - **Judged where its accounts are chosen:** at the activation's check,
    from whose choice the brief is planned, and again at the claim's own
    check (`executors.md`, section 1). If either finds another endpoint the run starts
    fresh, so a window never crosses. That holds even when only the
    second finds it, though then the brief, planned already, has no tail.
  - **A proof that keeps no endpoint,** written before the field was
    kept, gives its window's backend by its binding for this claim's
    main provider. The window resumes only if that binding is there, its
    source is still known, and its kind runs on the backend this claim's
    source runs on: a program source on the program backend, any other
    on the direct one. Otherwise it starts fresh.
  - **Why the host decides, and not the agent.** A run that resumes has
    no brief (`briefs.md`, 1). An agent sees its start only after the
    claim, with no brief to start fresh from, so a window from another
    backend is one it can only carry over, sending its history to a
    backend that did not write it (smith's `domain/session-context.md`, 1.2). A
    program's turns keep none of its reasoning (smith's
    `protocol/programs-claude-code.md`, 1.5), so whether a direct provider takes
    what a program wrote, or a program what a provider wrote, would
    depend on each pair of backends. A brief's tail is text, which every
    backend takes. The core chooses the source, and with it the backend
    and the endpoint, in the same preparation that decides resuming and
    judges a standing session's end.
- **A turn too large for a page.** A window is loaded a page at a time,
  each within the store's page room. A turn past that room is loaded
  cut: its fields and its length, never its bytes
  (`../../protocol/store/transcripts.md`, 3), handed to the core in its place as
  `TranscriptLoad::Cut`. The core counts its length in the window's bytes,
  keeps the fixed text `[a turn too large to load]` in its place in the
  tail (cut to the tail's budget as any turn's text is), and resumes no
  window that holds one: the run starts fresh from a brief carrying the
  transcript's tail, as one past the resume limit does, a transition
  that counts nothing, no failure, no transient try, no refusal. So a run's
  window is the transcript's tail that fits, never a failure for the
  window's length or for one turn's size: a window's load fails only for
  what could not be read (a frame unread or refused, below) or a page its
  root could not take.
- **A closed window.** A turn is handed to a run at the binary's
  layout, converted as read when it was written at another
  (`../../protocol/store/transcripts.md`, 6), and an open window may
  resume from it. A stored run proof's `CurrentWindow` may instead be
  closed, keeping its attempt and turn. The core restores it closed, and
  the task's next attempt starts fresh from a brief carrying the
  transcript's tail, as one past the resume limit does: the closed
  window's turns are loaded for their text alone, none is resumed, and
  the new attempt is its conversation's first. It is a transition, never
  a failure: the attempt counts as any attempt does, its brief's tail
  within the tail's budget, and no transient try, failure, refusal or
  unread transcript is counted; its activity shows a run begun fresh,
  its window closed (*Why a run began fresh*). A closed window's tail the store cannot read
  (a frame unread, `../../protocol/store/transcripts.md`, 3) is left out: the attempt
  starts fresh without it, and nothing is counted. When a turn's rendered
  text needs cutting, a tail budget too small for the 19-byte cut marker
  gives no text for that turn. If no turn leaves text, the tail is empty:
  the closed window's next attempt still starts fresh, without a failure
  counted. A window page that cannot hold one turn hands that turn on
  cut (above). An open unread window counts one transient failure and
  is closed, its retry continuing fresh without that tail, as for an
  unread closed window.
  However the upgrade's
  stop classes the attempt it ends (temper's `draft/upgrades.md`, 7.2),
  a closed window is kept for its tail's load and never resumed, and an
  open one is resumed as any is (above). The run proof
  keeps the window as a `CurrentWindow` (its attempt, its turn, whether
  it is closed: core-v6's `Window`); the transcript a run prepares keeps
  the pair it loads from, and whether it is closed beside it.
- **Why a run began fresh.** A run's beginning in its task's activity,
  `RunBegan`, says why it ran and whether it resumed. A run that does
  not resume though its charter resumes and its task has a window, open
  or closed, from an earlier attempt also says why it began fresh, in
  its `fresh`, the first that holds of:
  - `Moved`: its window ran on another endpoint or backend (*Resumed
    only where it ran*);
  - `Closed`: its window is closed (*A closed window*);
  - `Cut`: its window holds a turn too large to load (*A turn too large
    for a page*);
  - `Limit`: its window is past the resume limit (*Resuming*);
  - `Session`: a standing task's session ended, its `SessionEnded` line
    just before saying how (`../tasks/standing.md`, 5.2).

  A task's first run, a resumed run, and a run whose charter starts
  every run fresh have none. The cause is judged in the claim's
  preparation, where the window's load and the endpoints are still
  there, and is committed with the claim, never inferred from the proof
  the claim replaces. It is stored on the tasks family's `BeganLine` as
  `fresh: option Fresh = none`, so a line written before it reads none,
  and crosses to clients as `ActivityRunBegan.fresh`, a `RunFresh`
  (`moved`, `closed`, `cut`, `limit`, `session`), which moves
  `jig-client-wire` to 37 (`../../protocol/client/versions.md`, 1). Its
  words are its readers': temper's task page says `Run N began fresh:`
  and why (temper's `client/pages/task.md`, 2).
- **A crossed claim is never adopted.** The stored proof's
  `RunProof.crossed` (core-v6) fences the claim independently of its
  window: a claim resuming an earlier attempt's window, one whose
  attempt has yet to commit a turn and has no window, and one whose
  window is already closed alike. Registration and the jobs' runner
  infer no fresh mark and close no windows; the core consumes the
  proof as stored. At the restart a crossed claim is never
  adopted (`restart.md`, step 3): in the engine's process it is settled at
  once, lost or not started as at any stop; on another worker its run is
  stopped when the worker lists it, and the claim ends lost once the run
  has stopped or the engine's grace has passed, as a claim lost with its
  worker does, so its writer slot is freed only once nothing of it may
  still write (../tasks/holds.md, 3). The mark is on the proof, so a crash after
  the switch, before the restart ends the claim, ends it at the next. Each claim's preparation writes a proof of its own, never
  crossed, so the next attempt is adopted at a later restart as any
  claim is, until another mark crosses it. So every window the next
  attempt opens is its own, and open.
- **Turns are numbered within their attempt,** from one. The task keeps
  the last turn committed for its live attempt, restored with the fleet's
  adoption of the claim, so a copy already committed is acknowledged
  again and dropped, after a restart too.
