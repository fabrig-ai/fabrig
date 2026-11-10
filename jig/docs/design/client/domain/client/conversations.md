# Conversations

`jig_client::thread`, a helper (`helpers.md`): a conversation's thread,
for any presentation of a chat or of any agent task's activity.

- **A model the application holds,** `Thread`: step code with limits of
  its own, fed what the client gives: the pages of the task's activity
  read (`README.md`, section 6), the task's document and its run's text from their
  handles (`watches.md`, 1 and 3), and the task's pending `Message`s with their
  answers (`requests.md`), each alone or at its place in a compound
  request, whose answer numbers its words by that place (../contract/requests.md,
  4). It merges, in order, committed turns, the
  party's words, work summaries, proposals and provisional text, across
  windows and gaps. It sends nothing and reads nothing itself.
- **A committed line keeps its wall time** from the task's activity,
  including a turn that does not decode. A person's pending words have
  no committed time until their activity arrives.
- **Committed, read in pages from the end.** A task's activity is a list
  the engine keeps in the order it committed: the words parties wrote to
  it, each run's beginning and end, its turns, the results and news it
  heard, the decisions made about it (../contract/README.md, section 4). The
  application reads it a page at a time, newest first, as it needs
  older lines.
- **Turns, decoded below the client's surface.** A turn is bytes of
  smith's vocabulary, which the engine never reads
  (`../../../domain/engine/executors.md`, 2). `jig-client-protocol-link` decodes it
  with `jig-protocol-transcript` into the client's own turn: its
  messages; their blocks of text, calls and results, each call named by
  its tool with its input as written and its outcome summarised; what
  it used and spent; and `llm`, the model its completion was made with and
  the effort it asked for, as the turn names them (smith's
  `protocol/transcript.md`, section 2), none where it names no model. A
  call to the engine carries, as its objects and
  parts, what the core wrote beside the turn under its name
  (`../../../domain/engine/executors.md`, 2), never what its answer's words say.
  A call's outcome is its result's first line with words, within a
  field's `text`; its result block keeps the summary whole. The committed
  turn shares one `turn_text` budget, priced in `README.md`, section 11's order, so its
  calls' names go last; `llm` stands outside that budget, as its usage
  does, and is never cut. A text cut at the bound ends in "…", where it was
  cut, and the decoded turn is marked. A turn read alone (`Read::Turn`,
  an archived turn) is decoded whole instead: its texts within twice the
  bytes it was carried in, since each text is within them once and an
  outcome repeats at most its result's first line, and its messages and
  blocks within smith's transcript ceilings, so nothing of a turn the
  archive's byte bound admits is cut, a call's input included, and a
  reader paging it reads every byte; one past that bound is too large,
  as now (`README.md`, section 11).
  A turn read alone includes an opening's retained system text as a leading
  system message, including the supplied instructions and fitted brief.
  Activity pages keep conversation blocks alone, so inspection metadata
  consumes no conversation window or words budget. Brief-tail rendering
  excludes archived system context: a fresh run composes its own prefix.
  The archived message bound reserves one more than smith's message
  ceiling for that system message. Version-6 turns remain readable with
  their system context unavailable, never rebuilt from task settings.
  Neither the application nor the helpers link smith.
- **What the conversation last ran on.** `Thread.latest_llm` is the `llm`
  of the newest decoded turn that names a model, taken when the window
  reaches the end where live rows are added; reading older pages leaves it
  as it is. An undecodable turn or one naming no model is passed over;
  with no named model in that window the held value stays. It says what
  that completion used, whatever its run was given, and is none until a
  turn naming a model is read. A run that committed no turn supplies no
  new value; this is not proof of what the latest run used (`README.md`, section 15).
  `Thread::caught_up` says the thread has read as far as the task's
  document announces, so that a page can tell "none yet" from "none".
- **A job is a call's outcome,** worded like any other call's. A call
  that answered `running` reads "started as job N", never "is running":
  a job ends with its activation (smith's `protocol/transcript.md`,
  section 4). A `job` call names its job by number and what it
  answered. `JobsFull` and `UnknownJob` read as invalid calls. A call
  not run reads with its why: stopped, outside the choice, or after a
  failed write. The turn's count of jobs is not shown.
- **A turn that does not decode** reaches the application as such, with
  why: another version of smith's transcript (naming the one found),
  malformed, or past the client's bounds (`../../../protocol/client/wire.md`,
  section 1). It is a line of the thread, never dropped, and no other
  turn is read in its place.
- **Live, then committed.** While a run is live, its text is the
  thread's provisional line, kept under the turn's name until the
  committed turn of the same attempt and number takes its place: the
  thread keeps each turn's line, since the client holds only the turn
  being written (`watches.md`, 3). A line keeps its text's start, up to the thread's
  `live` bound. Within the same retry, suffix snapshots extend that prefix
  only where their bytes meet or overlap it; a gap leaves the prefix in
  place, marked cut, never joined across missing bytes. Its retained start
  and the newest stream offset are separate. A snapshot from the full
  origin, a newer retry or a new attempt replaces it; otherwise it only
  grows, and past the bound stays as it is until its committed turn takes
  its place. A run's end takes no line away by
  itself, since its committed turn may be a read away and a person may be
  reading those words: once the run has ended and the committed activity has
  been read, a line no committed turn replaced goes. That is a turn lost with
  its host, which never commits, and the next run starts after the last
  committed turn.
- **A person's words, in three states:** being sent, while their
  `Message` is pending; sent, once its answer names their message in
  the task's inbox; read, once a committed turn says it read through
  that message. A thread keeps at most its `bytes` of them, each within its
  `text`: `sending` refuses words past either, and whoever drafted them keeps
  them and says so.
- **A window bounded by size.** A thread's `bytes` bound its lines: a
  line weighs its bytes of words, texts, names, inputs, outcomes and
  summaries, and `LINE_ROOM` (64) for itself and for each message, block,
  object and part it holds, so the bytes bound what lines make of a page as
  they do what lines hold. Its `lines` is a second bound for a presentation
  that prices its tree by rows; one that doesn't sets it to what its bytes
  hold, `bytes / LINE_ROOM`, where it never bites. Its live lines keep their
  own `bytes`, as they did.
- **The newest are never lost.** A page that takes a window past a bound
  lets whole lines go, those furthest from what was just read: the newest
  when reading earlier, the earliest when reading later; never the page just
  read. Nothing says how many left. A line let go comes back, whole, when
  read again beside the line next to it (`Thread::older`, `Thread::newer`;
  ../contract/README.md, section 4). So each end of a window is one of three: it
  reaches its list's end, a page's cursor says more lies there, or the window
  let lines go there (`Edge::Dropped`). A window that let its newest go reads
  no new tail, and shows no provisional line and no pending words until it
  has them back: they belong at an end it doesn't hold. `Thread::latest`
  reads the list's end again and replaces the window with it, for a person
  sent to the end from far away.
- **A read that failed** leaves the window as it was and says which end
  it was for (`Thread::read_failed`), until the next read of that end.
- **How a thread shows** (a turn's calls folded to a line each, a call to
  the engine shown as what it made) is the presentation's; the reference
  UI's is ../view.md, section 9.
