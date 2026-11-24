# Transcripts

## 1. A file per task

From milestone S7 a task's turns go to its turns log, one of its task
logs (`logs.md`, 5), read by every kind of 3 as a file's frames are.
The files this document keeps are those of tasks that had one before:
such a task goes on writing its file until it ends, and no file is
converted; retention drops them as it drops a turns log.

- **One file per task,** named by the task's number, in directories by
  the number's high bits, so each directory holds a bounded number of
  files. The name is derived: nothing lists the files.
- **A turn per frame,** appended in commit order: the turn's attempt and
  number, its spend, the last message it read, its time, whether it opens
  a window, then smith's bytes, with a checksum, and the frame's length
  at both ends, so the file can be walked from either end.
- **A frame keeps the layouts it was written at,** with no field of its
  own. Its body is the core's `TurnRecord`, a versioned record, so its
  first two bytes are its layout's id; its `transcript` is
  smith's turn, versioned on its own (smith's `protocol/transcript.md`,
  section 6), so its first two bytes are the turn's layout's id, and
  the core's schema says so (skein's `data/codec/README.md`, section 3):
  `transcript: bytes 33554432 holds smith-transcript Turn`, its ceiling
  smith's largest turn (smith's `protocol/limits.md`, 4). No frame is
  rewritten: one at another layout than the binary's is converted each
  time it is read (6). The word is core-v6's: it is part of the family's layout
  and moves no byte (skein's `data/codec/fingerprints.md`, section 1).
- **The map keeps each transcript's places,** in the component's own
  family: its committed length and the offset of its current window's
  first turn, one row per task with a transcript; and each attempt's
  first turn's offset, one row per attempt. A turn is found from its
  attempt's first turn, by walking forwards.
- **Windows.** A frame whose turn opens a window moves the task's
  current window to it, in the same commit. A run that starts fresh opens
  a new conversation, whose first turn opens a window (smith's
  `protocol/transcript.md`, section 3). The run proof's window records
  whether it is closed (core-v6's `record Window { attempt: u64 turn:
  u32 closed: bool }`). No upgrade closes windows by a map patch
  (`opening.md`, 2).

## 2. Written before its commit

The core commits a turn with its spend, its run proof and the messages it
read, in one decision, and acknowledges the host once that commit is
durable (`../../domain/engine/executors-transcripts.md`, 1). For a commit with turns, the
component takes the following steps.

Before step 1, it encodes and checks every frame against `P` as described
in `commits.md`, section 1, retaining each accepted encoding once for the pending batch.
No frame from a refused commit is placed on the file IO queue.

1. writes each turn's frame at its file's end, opening the file
   (`OpenWrite`), or creating it where it is missing. The end is the
   committed length, plus the frames queued ahead and not yet durable,
   since a task's next turn may be decided before its last one commits;
2. syncs each file it wrote, and, for a file it created, the directory
   that holds it, and that directory's own, for a directory it made;
3. submits the commit to skein-kv, with each transcript's new places among
   its operations;
4. answers committed once skein-kv has.

A crash after step 2 and before the commit is durable leaves a frame past
the committed length. The next open cuts the file to its committed length
(4), and the host, never acknowledged, sends the turn again.

## 3. Read back

- **Resuming:** a run that resumes is given its current window's turns,
  from the window's first turn to the end, within the resume limit
  (`../../domain/engine/executors-transcripts.md`, 1), read forwards a page at a time while the
  task prepares. The core asks only when its conversation has a window,
  and names the turn that opens it, as it keeps it (`../../domain/engine/decisions.md`,
  1.4); a window that starts at another turn fails the read as unread.
- **A window's turn past its page's room** is handed on cut: its fields
  and its transcript's length, not its bytes, as a party's page has a
  turn past its bound, alone in its page, with the continuation past its
  frame (`Event::Turns`, `cut`). A page that already holds a turn ends
  before it, so the next page starts at it. A window's page therefore
  always moves on, and a window's read is never unread for a turn's
  size: unread stays for a window that starts at another turn and for a
  frame unread or refused (below). The core keeps the cut turn's place
  and length, never its bytes (`../../domain/engine/executors-transcripts.md`, 1).
- **A turn,** for a party's read: from its attempt's first turn, walking
  frames forwards to its number. A turn read alone is bounded by the
  read's byte bound (`loads.md`, 3): past it, the read answers the turn's size in
  place of its bytes (`../../client/domain/contract/reads.md`, section 1).
- **A page's turns,** for a party's page of a task's activity (`loads.md`, 3):
  backwards, one frame for each turn line, from the frame of the page's
  newest turn: the transcript's last frame when the page reads from the
  activity's end, otherwise found as a turn is. A turn past the page's
  bound has its fields and length read, not its bytes.
- **A page of a task's turns,** forwards or backwards, from a cursor that
  is an offset the component gives and takes back.
- **Direct complete-turn requests** other than a window's that cannot
  produce a turn under the read's room terminate `Unread`, never a
  retryable empty page at an unchanged offset; a nonempty prefix may give
  a continuation. A party's
  single-turn `TooLarge` answer is unchanged.
- **A tail,** for a fresh run's brief: the window's last turns as text,
  each rendered by the host's protocol layer when loaded, and each turn's
  text at most the tail's own budget, as the core's memory prices it.
- **Never past what is committed.**
- **Current bytes only:** each kind hands on a frame at the binary's
  layouts, one at another converted as it is read (6).
- **Turns-log demands** ask for one record in either direction
  (`logs.md`, 5).
- **A dropped transcript,** a task with no row of places: a party's
  page of its activity carries each turn line without its turn, as
  unread, and goes on past it, since none of its frames can be read
  (`retention.md`, 7). A transcript read of it answers unread, and a
  party's read of one turn finds none.
- **A frame that fails its checksum** fails the read as unread, and so
  does one its layouts' plans refuse (6). The core
  treats the transcript as one the agent cannot use: the run fails as
  transient and the next starts fresh (`../../domain/engine/executors-transcripts.md`, 1); a
  closed window's tail it cannot read, it leaves out.
- **Open files are bounded:** the component keeps a bounded set open,
  live tasks' first, and opens others to read them.

## 4. After a crash

At open, before the root starts, the component reads the core's run
proofs, one for each live task ever claimed (`keys.md`, 1.4), and cuts each such
task's file to its committed length, syncing it: zero for a task with no
places yet, whose first turn's commit was lost; a missing file is left
missing. Only a crash leaves bytes past a committed length, since a
failed commit stops the engine, and only a claimed task has frames, its
claim durable before its run was placed (`../../domain/engine/decisions.md`, 1.2).
After the cut every file holds exactly its committed turns. An ended
task's file was cut while its task lived, and is not touched again.

## 5. Dropped

Retention drops a task's transcript whole, at its turns' horizon after
the task ended (`retention.md`, 6.1), or with the task when it is
erased (`growth.md`, 1.1): the commit erases the task's row of places, and the component removes its file and syncs the
directory before it submits the commit. The task's attempts' places,
as many as its attempts, are orphans from that commit on, read by
nothing, and go in the commits that follow, a bounded number each
(`commits.md`, section 1, "A commit's operations are bounded"). A crash
before the commit leaves places whose file is gone: a read of it answers
as for a dropped transcript, and retention's next pass drops it again.
Removing a missing file is not a failure.

## 6. Converted as read

An upgrade never writes a transcript (`opening.md`, 2): the transcripts
are most of what a deployment holds, and a frame is only ever read. So
a frame keeps the layouts it was written at, and is converted each time
it is read, by plans from the registry's texts, as a map value is
(`layouts.md`); nothing is written.

- **Two value types a frame** (`layouts.md`, 1): its body, the frames'
  root of jig's frames' family, the core's `TurnRecord`; and the turn
  its `transcript` holds, the frames' root of smith's,
  smith-transcript's `Turn`. Each is named by its family's
  `Frames::number` (`opening-owners.md`, 1), with no key range, its
  layouts rows of the registry (`layouts.md`, 3). The body's layout is
  computed without smith's text, so its held turn is named by family
  and type (skein's `data/codec/fingerprints.md`, 1.4) and its plan
  copies the turn as it is (skein's `data/codec/layouts.md`, 1): the
  turn moves by its own layout, never its holder's. A record of the
  map that holds a turn (the core's parts, `records.md`, 1) takes it
  into its own layout, as any held record (`layouts.md`, 1).
- **Heads:** the body's first two bytes are its layout's id, and the
  turn's first two, past the `transcript` field's length, its own. A
  frame is written with the binary's ids, each family's version the
  generated code writes mapped to its value type's binary id as the
  write edge maps a map value's (`layouts.md`, 4), before the frame's
  checksum; a turns log's record the same. A frame is read with each
  mapped back: an id that is the binary's, or one whose plan keeps, has
  its family's version written in its place; another planned id is
  converted by its plan, the turn within its field, the field's length
  written again; an id its value type has no row for, or a value its
  plan refuses, is a frame unread (3).
- **Before the registry:** a frame written before its value types had
  rows carries its families' versions, the body the core's 6, the turn
  smith-transcript's 7 or 8. The open that finds no row of a frames'
  value type registers each of its family's `texts` at its version as
  its id, before the binary's layouts (`layouts.md`, 5.2): jig's the
  core's `current`, at 6; smith's smith-transcript's kept texts, at 7
  and 8. Each such frame's heads then name rows as any frame's do. A
  frames' family's `texts` keep this seed until dev.fabrig.ai's store
  holds the rows (temper's `draft/archive/store-floor.md`).
- **Planned at open,** each row of a frames' value type to the binary's
  layout, as a log-held value type's (`layouts.md`, 5.3; `logs.md`, 8):
  every layout is held, since no census counts frames, so one with no
  plan refuses the open; none retires; none settles (`settling.md`); a
  later one is planned in Read mode (`layouts.md`, 5.6). The rehearsal
  decodes each frame's body and turn as generic values of their
  layouts' texts (`opening-rehearsal.md`, 1).
- **Every read kind** of 3 (resuming, a turn, a page's turns, a page
  of turns and a fresh run's tail, the last read as resuming's is) reads
  its frames through one step, the frame's decode, and converts there.
  The core, a party's read and a host's protocol layer are handed
  current bytes alone: `jig-protocol-transcript` reads smith's current
  version only, as smith's agent does.
- **Bounds on converted bytes.** A frame is read whole and converted
  before anything is judged: its size, against a page's bound, a turn's
  (a party's charge, the page's row and the turn's bytes) and the
  resume limit, is its converted body's and its 12 bytes of framing,
  for every frame, converted or not, so a frame's charge does not hang
  on its layout, and a turn refused past a bound
  (`TurnFailure::TooLarge`) names that size. A direct Window/Page read
  charges the whole frame, including metadata, payload and framing.
  The head alone (a turn past a page's bound, 3) serves a frame whose
  ids are the binary's or keep; another is read on, whole, from the
  bytes already read. A frame converted past the page's bytes is
  unread.
- **Nothing is written.** The read hands on the converted bytes and the
  file keeps the frame's own, so its lengths, its checksum and the map's
  places stay true, and the frame is converted again at its next read. A
  cut anywhere in a read leaves every file as it was.
- **One read at a time** (`transcripts/read.rs`): commits wait behind a
  read as behind any read (`transcripts/write.rs`).
- **Memory:** the read edge's buffer and workspace (`layouts.md`, 7),
  the workspace fitted at open to every frames' plan beside the map's,
  the buffer grown to the largest frame converted, at most the page's
  bytes `P` (`commits.md`, section 1). No patch machine is made, and no
  frame goes up to the shell: a frame's conversion calls no owner.
- **The formats:** a frame's layouts move as a registered value's do;
  its families' versions stay, the core's 6 and smith's 8 (temper's
  `draft/layouts-both-ways.md`, 3). A change no plan derives refuses
  the open, as a log-held value type's does (`logs.md`, 8), so temper's
  stored-layouts check holds every deployed frames' layout to a plan
  (temper's `protocol/floor.md`, 6). A frame's move is no level's:
  smith's level stays 0 (`opening.md`, 3; `records.md`, 3).
