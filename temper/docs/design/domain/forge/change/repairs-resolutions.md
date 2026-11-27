# Resolving a change's conflicts

This document holds merge jobs and conflict resolutions, split from [`repairs.md`](repairs.md).

## 1. Resolutions

- **The merge job** comes before a resolution where the change's
  repository has a merge command at the head the merge is made on, its
  `.temper/hooks/merge` or, where that head holds no `.temper/hooks/`,
  its project's `merge_job` (`../../../protocol/hooks.md`, 4.2 and 5;
  `../../../protocol/configuration/engine.md`), and the change's
  authority delegates to the merge kind: a job of
  kind 3 (`../../worker/jobs.md`), delegated by the change, which holds the
  branch it writes. Once a conflict is found that such a job may take,
  the kind declared and in its authority, the change reads
  `.temper/hooks/` at that head, the answer kept while the head stands;
  a read with no answer leaves the change waiting, and it reads again
  at its next step. Its workspace is a resolution's, the base merged
  into the branch's head;
  the merge command settles the conflicts it knows
  (`../../../protocol/tooling.md`, 4.1); if none is left, the worker commits
  the merge and pushes it. On the job's result the change reads its
  branch afresh: a head that moved is the merge's, and the change goes
  on from it as from a resolution's push; a head that did not move
  means the command left conflicts, and the change makes a resolution
  of the same base, its words carrying the job's result, made text first
  (each byte outside UTF-8 a `?`, skein's `runtime/lib-bytes.md`, 1, since the
  job's result is the merge command's bytes and a task's words are
  text), cut to the resolution's words' room and marked. A job that fails for good holds
  the change as a resolution that fails does. Each of its failures, a
  try that will be spent again included, is a line on the worker that met
  it and on the engine, naming the job, its kind and why (`../../worker/jobs.md`
  ); the job's own hold, out of tries, names its failures' class,
  and its way out points at the engine's log, where each is named. Without a merge command at that head,
  without the kind declared (`../../worker/jobs.md`) or in its authority, or
  with the job's spec past the kind's bound, a conflict goes to a
  resolution at once. A merge job counts as one of the change's
  resolutions, and the resolution after it as none, so the bound counts
  each conflict once.
- **A resolution** is a task for an agent whose workspace starts from a
  merge in progress: the base merged into the branch, the conflicting
  files marked. The agent edits them and runs the checks, with no git
  writes of its own; the worker commits the tree as the merge, with both
  parents, and pushes it (`../../worker/checkouts.md`).
  **It waits for its producer.** A resolution, and a merge job before it,
  starts only once every task the change gave its branch to push, its
  producer and each repair, has ended: its terminal heard, a successful
  producer's result the head of its last push. The change keeps each such
  task's number from its delegation to its terminal, whatever it delegates
  meanwhile (a gate, a merge job), so a task no longer its current
  delegate still counts. A conflict the change meets while one still lives
  (a producer whose pull or title the change already held, a run whose
  reported push or save freed the branch's writer, a lost attempt's reread)
  is a wait, not a resolution: the change keeps the base it would resolve
  against, counts no resolution and no stall, holds nothing, and the
  task's terminal wakes it. Then it reads its branch, its base and its
  pull afresh (`../keeping-up.md`, 3) and keeps the head it reads as its
  last head, its own since the branch was held by its delegates alone.
  Still in conflict, it is resolved there, at the change's latest pushed
  head: the resolution's pull section is pinned at that head, and a merge
  job merges the base into it. Clean, it goes on with no resolution, a
  head that moved meanwhile checked again as any push of its own is.
  **A resolution's reads never fail it for good on a forge catching up:**
  a head that moves under them, as a landing's does when the forge
  updates its branch or a change lands into it, ends the resolution
  before it runs, and the change reads afresh and resolves again at that
  head where the conflict still stands (a resolution's moved head,
  below); a comparison past a bound is omitted (below); so a resolution
  fails for good only on what a fresh read would meet again.
  Its instructions keep
  both sides' intended content by the same preservation rule as repairs,
  which yields as theirs does to the change's current spec and rulings;
  the shared coding charter applies it to repairs and resolutions too,
  with the same proviso.
  **A deletion or a rename is intended content too**
  (`../../../draft/archive/merge-deletions.md`, 2): a file deleted on the base
  stays deleted unless the requested change needs it; a file deleted on
  the head, or renamed on either side, stays so, the other side's edits
  carried where they still apply; and the agent reads the commit that
  deleted a file before it restores anything. Keeping an intended
  deletion removes no content, so the rule against removing content to
  make a check pass holds beside it. The resolution's result says, for
  each file deleted or renamed on either side, whether it kept the
  deletion or restored the file, and why. The judgement is the agent's:
  neither the merge job nor the change settles a file one side deleted
  or renamed, since whether the change still needs it is a judgement
  (`../../../protocol/tooling.md`, 4.1), but for the paths the journeys'
  writer alone writes, which the reference's merge job takes from the
  base (`../../../client/journey-writer.md`, 5). The full instructions are
  exactly:

  > Resolve the merge conflict. Preserve the requested change, as its current spec and rulings state it, and unrelated content already landed on the base, including both independent appended lines and existing blank lines. A deletion or a rename on either side is an intended change too: a file deleted on the base stays deleted unless the requested change needs it, and a file deleted on the head or renamed on either side stays so, the other side's edits carried where they still apply. Read the commit that deleted a file before you restore anything. Do not remove content just to make a check pass, unless its current spec or a ruling says otherwise; keeping an intended deletion removes none. In your result, say for each file deleted or renamed on either side whether you kept the deletion or restored the file, and why. The worker commits and pushes the resolved merge.

  The specification reserves its pinned parameters' bytes before its words.
  Where the full instructions do not fit, a compact instruction keeps both
  sides, blank lines and each side's deletions and renames, with the full
  rule in the coding charter:

  > Resolve the merge conflict. Keep both sides, blank lines, deletions and renames unless its current spec or a ruling says otherwise. Restore a file the base deleted only if the change needs it, after reading the commit that deleted it; say in your result which you kept or restored, and why.

  Where neither fits, the change holds instead of issuing an oversized
  delegate. Either form carries the deletion rule whole, with a merge
  job's result after it or none.
  Required preservation instructions precede optional merge-job results. A
  result is appended only with room for its context and any truncation marker;
  it never displaces the preservation instructions, so a result cut or
  left out leaves the deletion rule whole. The merge job's records name
  each left file's kind and, for a deletion, the side and the commit
  that deleted it (`../../../protocol/tooling.md`, 4.1).
  The change's current spec and rulings follow them both (`questions.md`,
  1, *What every delegate is told*), their room taken before the merge
  job's result, which takes what they leave.
  Its pull section carries each of the pull's files whole, both sides, where
  they fit the room the section has left. A file that doesn't is named
  instead, never carried in part, for the resolver to read in its checkout,
  where the merge in progress holds it marked: `File: PATH`, then
  `[not carried, past the brief's room: B bytes before, A bytes after; read
  it in your checkout]`, a side the file lacks reading `absent before` or
  `absent after`. A named file takes its lines' room alone, so the files
  after it are carried as they fit. The charter's fit
  (`../../agent/room.md`, 2) can still cut the section's end, with its own line.
  **A file that is not text is named by its blob ids,** in every pull
  section, a repair's and a review's as well as a resolution's, and
  never carried, whole or in part: a file one of whose sides is not
  UTF-8, or holds a NUL byte, as git takes a binary file. It reads
  `File: PATH`, then `[not carried, not text: blob ID before, blob ID
  after; read it from the repository]`, a side the file lacks reading
  `absent before` or `absent after`. ID is git's object id of that side's
  bytes as the forge gave them, hashed over `blob `, the length in
  decimal, a NUL byte, then the bytes, in lowercase hex: SHA-1's 40
  digits where the section's pinned head is a SHA-1 id (its last 12 of
  32 bytes zero, as the engine reads a commit), SHA-256's 64 otherwise
  (skein's `runtime/lib-bytes.md`, 1.2 and 1.3). The agent reads it with `git
  cat-file -p ID`, or, resolving, as the merge in progress holds it.
  This check comes before the room's, so a file not text is named so
  whatever its size, and its line takes its room alone as a file named
  for room does. A brief is text, and smith refuses an assignment whose
  section is not (`Section.body not UTF-8`), so one such file carried
  would hold its task for good; the assignment's own check for text,
  behind this one, is `../../agent/room.md`, 2.
  Its pinned comparison, of the head and the base it conflicts with, is by
  names (skein's `forge/vocabulary.md`, 1.2 and 1.3), so a conflict however far its
  branches have gone apart is compared: `What landed in the base: N
  commits`, N its `ahead`; then each path it names, `Base file: PATH`,
  followed by `Conflicting file: PATH` where the pull's own files name it
  too; then, where it names fewer than its `changed`, `[K more base files
  not listed: past forge.client.rows (R); read them in your checkout]`, K
  what it left out and R the configured rows. A repair's comparison, from
  its last clean head, reads the same. A review's says `Against base:`,
  then its paths, then `[K more files not listed: past forge.client.rows
  (R); read_issues what=pull_files lists them]`. Each line is appended where it fits
  the section's room, so the section keeps what fits and names what it
  left out. A comparison refused `TooLarge` keeps the section's pull
  identity and gathered content and appends a comparison-omitted notice
  within its byte budget. This completes the section; it does not retry
  the comparison or raise the client's limits. Other comparison read
  failures are the section's read failures (below). The same rule applies
  to the pull sections for reviews and repairs.
  Likewise, if a page of the pull's files exceeds the forge client's answer
  byte or row limit (a page carries each of its files whole on both sides,
  skein's `forge/forgejo-operations.md`, 1, so one large document can pass it however small
  its change), the section keeps its pull identity, its gathered content and
  the pages before it, and appends a files-omitted notice within its byte
  budget, naming that page, its bound and the tools that read its files:
  `[files omitted from page P: past BOUND; the pull changes F files; read_issues what=pull_files from page P+1 lists the rest, and read_forge what=file reads a path at the head or the base]`,
  F the forge's count of the pull's changed files from the section's pull
  read (skein's `forge/vocabulary.md`, 1.6, `Pull`'s `size`), its clause
  `the pull changes F files; ` left out where the forge gave no count. The section then goes on as after its last page, to its
  comparison where it has one; it reads no later page of files and raises no
  limit. This holds for every pull section, a review's included, so a
  review of a small change to a large document starts.

## 2. Open questions

None.
