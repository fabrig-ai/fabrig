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
- **A frame keeps the versions it was written in,** with no field of its
  own. Its body is the core's `TurnRecord`, a versioned record, so its
  first two bytes are the core family's version; its `transcript` is
  smith's turn, versioned on its own (smith's `protocol/transcript.md`,
  section 6), so its first two bytes are smith-transcript's version, and
  the core's schema says so (skein's `data/codec/README.md`, section 3):
  `transcript: bytes 33554432 holds smith-transcript Turn`, its ceiling
  smith's largest turn (smith's `protocol/limits.md`, 4). No frame is
  rewritten: one behind the binary's versions is patched each time it is
  read (6). The word is core-v6's: it is part of the family's layout
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
  versions, one behind patched as it is read (6).
- **Turns-log demands** ask for one record in either direction
  (`logs.md`, 5). A written patch site pauses the current frame before
  the next record is demanded (6).
- **A dropped transcript,** a task with no row of places: a party's
  page of its activity carries each turn line without its turn, as
  unread, and goes on past it, since none of its frames can be read
  (`retention.md`, 7). A transcript read of it answers unread, and a
  party's read of one turn finds none.
- **A frame that fails its checksum** fails the read as unread, and so
  does one its machines refuse (6). The core
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

## 6. Patched as read

An upgrade never writes a transcript (`opening.md`, 2): the transcripts are most of
what a deployment holds, and a frame is only ever read. So a frame
behind the binary's versions is patched each time it is read, and
nothing is written.

- **The machines** are the frames' families' (`opening-owners.md`, 1,
  `KeptFamily::frames`): one `Patcher` each, the body's family in jig's
  set and the turn's in smith's, made at every open that serves (`opening.md`, 1)
  from the sets it was handed, after the frames' checks of `opening-owners.md`, 1, each
  frames' text kept and each of its frames' patches planned, and held
  by the transcripts' part for the whole run. No map patch machine is
  made. A family with no frames' text has no machine.
- **A text kept once.** Each `Patcher` an open makes of a frames'
  family (the frames' check's, the one that sizes a machine and the
  machine itself) keeps each of the family's texts once: a copy of the
  shell's prepared schema whose whole text equals it (`opening-owners.md`, 1),
  or its own parse where none does. What the open reads of a kept text
  after (its version, its root, its nodes' bound) it reads from the
  schema the `Patcher` keeps, never from a parse of its own.
- **A frame's versions** are read with its head, past the frame's
  length: the body's first two bytes, and the turn's first two, past the
  `transcript` field's length, after the fields the head reads already.
  With smith's machine, a turn of fewer than two bytes is unread.
- **A body behind** (its version kept by jig's machine, before the
  binary's) runs jig's plans from its version to the binary's, from the
  frames' root (found in the frame's version by the kept texts' renames,
  by `transcripts/patches.rs`'s `held_root`), `next_pass` after the first.
  At its held turn the machine stops (`Step::Held`): at the first pass,
  smith's machine patches the turn from its version to smith's last
  kept, and its bytes go back with `put_content`; the passes after carry
  them (`carry`). A body at the binary's version is decoded by the
  binary's codec, and its turn, if behind, is patched by smith's
  machine alone, from smith's frames' root.
- **One rule for versions.** Where a family has a machine, a version
  among its kept texts and before its last is patched, its last goes on
  as it is, and any other, older than its first or past its last, is a
  halt (`Decode` or `Order`), so the frame is unread. Where a family has
  none, its bytes go on as they are: a body its codec cannot
  read is unread (3), and a turn's reader refuses a version not its
  own (smith's `protocol/transcript.md`, section 4).
- **Owners' order.** A frame runs jig's plans first and smith's at
  the held turn (`opening-owners.md`, 1). Neither reads the other's bytes:
  jig's written functions
  on a frame's body never read its `transcript`, and smith's read their
  turn alone (smith's `protocol/transcript.md`, section 12).
- **Nothing is written.** The read hands on the patched bytes and the
  file keeps the frame's own, so its lengths, its checksum and the map's
  places stay true, and the frame is patched again at its next read. A
  cut anywhere in a read leaves every file as it was.
- **Every read kind** of 3 (resuming, a turn, a page's turns, a page
  of turns and a fresh run's tail, the last read as resuming's is) reads
  its frames through one step, the frame's decode, and the machines run
  there. The core, a party's read and a host's protocol layer are handed
  current bytes alone: `jig-protocol-transcript` reads smith's current
  version only, as smith's agent does.
- **Bounds on patched bytes.** A frame behind is read whole and patched
  before anything is judged: its size, against a page's bound, a turn's
  (a party's charge, the page's row and the turn's bytes) and the resume
  limit, is its patched body's and its 12 bytes of framing. Where either
  machine is made, every frame is charged so, behind or not, so a frame's
  charge does not hang on its version, and a turn refused past a bound
  (`TurnFailure::TooLarge`) names that size. A direct Window/Page read
  charges the whole frame, including metadata, payload and framing,
  whether or not a machine is made. With no machine, a party's read
  charges the turn's bytes. The head alone (a turn past a
  page's bound, 3) serves a frame at the binary's versions only: a
  head whose body or turn is behind is read on, whole, from the bytes
  already read. A patched record past its machine's bytes or nodes
  halts (`Room`).
- **A halt is an unread frame.** A frame the machines refuse (any
  `Halt` of skein's `data/codec/patches.md`, 4, a site's `NoSite` among them) is
  read as one that fails its checksum (3), in every read kind.
- **A written site goes up** (`opening-owners.md`, 1): at
  `Step::Written` the component answers `Site` to the read's own token,
  `of` the site's owner, its `key` the frame's name (the task's key in
  the component's places, `keys.md`, 1.4, then the frame's offset in its file,
  eight bytes big-endian), the site, the node and the machine, moved;
  the read waits for `Request::Site` with the result. A shell answers a
  site at once, by its owner's generated `written`, with a `Default`
  `Context`, its key set: reads come in any order and a frame is patched
  at each, so a frame's patch reads that frame alone (skein's
  `data/codec/patches.md`, 5). The component routes `Request::Site`
  to the read waiting at that token; any other is refused. There is no
  map walk's site to route. A shell that hands no sets never sees `Site`.
- **One read at a time** (`transcripts/read.rs`): a read
  waiting at a site holds the component's read, commits wait behind it
  as behind any read (`transcripts/write.rs`), and a machine serves one
  frame from `start` to `finish`.
- **Memory:** each machine's `Patcher::workspace_bound` of its capacity,
  in serving's worst case (`README.md`, section 8), priced at startup; none for sets
  with no kept text. A capacity's versions, patches and text are its
  owner's kept versions, patches and largest text. Its bytes and nodes
  are its frames' bounds, never the map's: jig's, the frames' largest
  body and the most nodes any kept body takes, its held turn one node;
  smith's, the `transcript` field's bytes and the most nodes any kept
  turn takes. Both are counted from the frames' root in each kept text
  (`transcripts/patches.rs`, `node_bound`), so every frame the kept
  texts allow is read and none is refused for its nodes. A turn's bytes
  do not bound its nodes (a list of empty records takes none), so
  smith's real turn, 4 messages of 128 blocks, takes 1,054,764 nodes,
  and its machine is priced at that capacity, about
  249 MB with what its tables hold while they grow (skein's `data/codec/patches.md`,
  7); a machine holds only what the largest record it has met needed
  (`README.md`, section 11).
- **The formats are fixed** at the core's version 6 and smith's 8
  (temper's `draft/store-floor.md`, section 5). Their frame patcher and
  fixtures, `jig-test-patches-jig` and `jig-test-patches-smith`, stay.
  A change to a frame format designs how the deployment's store reaches
  it in that landing; no mark schedules a window-closing walk at open.
  Smith's 8 (the `interrupted` outcome, smith's `protocol/transcript.md`,
  section 12) reaches it by this patching: a turn at 7 is patched by
  smith's machine each time it is read, and nothing is written.
- **A frame's move is no level's.** Smith's level stays 0 (`opening.md`,
  3; `records.md`, 3): a turn's version moves by its kept text and
  patch, read here, never by a level, so a store written at smith's 7
  opens at equal levels. The core's `TurnRecord` keeps its version 6
  while the turn it holds moves: the body's own layout is judged with
  its held turn at one version on both sides, and the held turn's move
  by smith's kept texts and patches. temper's stored-layouts check
  (`temper/tests/codecs`) holds both.
