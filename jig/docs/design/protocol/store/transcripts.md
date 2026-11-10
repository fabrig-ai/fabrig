# Transcripts

## 1. A file per task

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
  the core's schema says so (skein's `codec.md`, section 3):
  `transcript: bytes 33554432 holds smith-transcript Turn`, its ceiling
  smith's largest turn (smith's `protocol/limits.md`, 4). No frame is
  rewritten: one behind the binary's versions is patched each time it is
  read (6). The word is core-v6's: it is part of the family's layout
  and moves no byte (skein's `codec.md`, section 9).
- **The map keeps each transcript's places,** in the component's own
  family: its committed length and the offset of its current window's
  first turn, one row per task with a transcript; and each attempt's
  first turn's offset, one row per attempt. A turn is found from its
  attempt's first turn, by walking forwards.
- **Windows.** A frame whose turn opens a window moves the task's
  current window to it, in the same commit. A run that starts fresh opens
  a new conversation, whose first turn opens a window (smith's
  `protocol/transcript.md`, section 3). A patch marked fresh closes
  every live task's window, in the patched copy (`opening.md`, 2, step 4): the run
  proof's window says so (core-v6's `record Window { attempt: u64 turn:
  u32 closed: bool }`).

## 2. Written before its commit

The core commits a turn with its spend, its run proof and the messages it
read, in one decision, and acknowledges the host once that commit is
durable (`../../domain/engine/executors.md`, 2). For a commit with turns, the
component takes the following steps.

Before step 1, it encodes and checks every frame against `P` as described
in `README.md`, section 5, retaining each accepted encoding once for the pending batch.
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
  (`../../domain/engine/executors.md`, 2), read forwards a page at a time while the
  task prepares. The core asks only when its conversation has a window,
  and names the turn that opens it, as it keeps it (`../../domain/engine/README.md`,
  5.4); a window that starts at another turn fails the read as unread.
- **A window's turn past its page's room** is handed on cut: its fields
  and its transcript's length, not its bytes, as a party's page has a
  turn past its bound, alone in its page, with the continuation past its
  frame (`Event::Turns`, `cut`). A page that already holds a turn ends
  before it, so the next page starts at it. A window's page therefore
  always moves on, and a window's read is never unread for a turn's
  size: unread stays for a window that starts at another turn and for a
  frame unread or refused (below). The core keeps the cut turn's place
  and length, never its bytes (`../../domain/engine/executors.md`, 2).
- **A turn,** for a party's read: from its attempt's first turn, walking
  frames forwards to its number. A turn read alone is bounded by the
  read's byte bound (`loads.md`, 3): past it, the read answers the turn's size in
  place of its bytes (`../../client/domain/contract/README.md`, section 4).
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
- **A frame that fails its checksum** fails the read as unread, and so
  does one its machines refuse (6). The core
  treats the transcript as one the agent cannot use: the run fails as
  transient and the next starts fresh (`../../domain/engine/executors.md`, 2); a
  closed window's tail it cannot read, it leaves out.
- **Open files are bounded:** the component keeps a bounded set open,
  live tasks' first, and opens others to read them.

## 4. After a crash

At open, before the root starts, the component reads the core's run
proofs, one for each live task ever claimed (`README.md`, 4.4), and cuts each such
task's file to its committed length, syncing it: zero for a task with no
places yet, whose first turn's commit was lost; a missing file is left
missing. Only a crash leaves bytes past a committed length, since a
failed commit stops the engine, and only a claimed task has frames, its
claim durable before its run was placed (`../../domain/engine/README.md`, 5.2).
After the cut every file holds exactly its committed turns. An ended
task's file was cut while its task lived, and is not touched again.

## 5. Dropped

Retention drops a task's transcript whole (`README.md`, 6.1): the commit erases the
task's row of places, and the component removes its file and syncs the
directory before it submits the commit. The task's attempts' places,
as many as its attempts, are orphans from that commit on, read by
nothing, and go in the commits that follow, a bounded number each
(`README.md`, section 5, "A commit's operations are bounded"). A crash
before the commit leaves places whose file is gone: a read of it answers
as for a dropped transcript, and retention's next pass drops it again.
Removing a missing file is not a failure.

## 6. Patched as read

Patching never writes a transcript (`opening.md`, 2): the transcripts are most of
what a deployment holds, and a frame is only ever read. So a frame
behind the binary's versions is patched each time it is read, and
nothing is written.

- **The machines** are the frames' families' (`opening.md`, 3,
  `KeptFamily::frames`): one `Patcher` each, the body's family in jig's
  set and the turn's in smith's, made at every open that serves (`opening.md`, 1)
  from the sets it was handed, after the frames' checks of `opening.md`, 3, each
  kept text kept and each of its owner's patches planned, and held by
  the transcripts' part for the whole run. They are not patching's
  (`opening.md`, 3), which go with the copy's open. A family with no kept text has
  none: every real set is empty until the baseline, so no machine is
  made.
- **A frame's versions** are read with its head, past the frame's
  length: the body's first two bytes, and the turn's first two, past the
  `transcript` field's length, after the fields the head reads already.
  With smith's machine, a turn of fewer than two bytes is unread.
- **A body behind** (its version kept by jig's machine, before the
  binary's) runs jig's plans from its version to the binary's, from the
  frames' root (found in the frame's version by the kept texts' renames,
  as a held root is, `patching/walk.rs`), `next_pass` after the first.
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
- **Owners' order.** The copy runs owners' patches in their pins' order
  (`opening.md`, 3); a frame runs jig's first and smith's at the held turn. The two
  agree because neither reads the other's bytes: jig's written functions
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
  `Halt` of skein's `codec.md`, 11.4, a site's `NoSite` among them) is
  read as one that fails its checksum (3), in every read kind.
- **A written site goes up,** as patching's do (`opening.md`, 3): at
  `Step::Written` the component answers `Site` to the read's own token,
  `of` the site's owner, its `key` the frame's name (the task's key in
  the component's places, `README.md`, 4.4, then the frame's offset in its file,
  eight bytes big-endian), the site, the node and the machine, moved;
  the read waits for `Request::Site` with the result. A shell answers a
  site at once, by its owner's generated `written`, with a `Default`
  `Context`, its key set: reads come in any order and a frame is patched
  at each, so a frame's patch reads that frame alone (skein's
  `codec.md`, 11.5). The component's routing of `Request::Site` follows
  its phase: a walk's while patching, and the read waiting at a site
  while serving, its token the read's; any other is refused, as a site
  no walk waits for is. A shell that hands sets answers `Site` while
  serving as while patching, and one that hands none never sees it.
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
  and its machine is priced at that capacity from the baseline on, about
  249 MB with what its tables hold while they grow (skein's `codec.md`,
  11.7); a machine holds only what the largest record it has met needed
  (`README.md`, section 11).
- **Windows across a mark.** A patched turn is the same conversation in
  current bytes, as smith's patch makes it; one of smith's patches may
  say instead that a window across it starts fresh (`mark
  fresh-windows`). Its plan closes the live tasks' windows in the copy
  (`opening.md`, 2, step 4), so no run resumes from turns patched across it; a tail,
  a party's read and a page read them patched as any others. No attempt
  is live across such a patch: its upgrade is a full one, the workers
  stopped first (temper's `draft/upgrades.md`, 7.2), which temper's
  section 9 is to say of a pending patch marked fresh (`README.md`, section 11).
