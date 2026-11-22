# Gathering a change's forge brief

This document holds the gathering of a change's forge brief, split from [`repairs.md`](repairs.md).

## 1. Gathering briefs

- **A forge section is sized by its cutoff,** whatever its forge reads are
  doing, so a large pull, a slow forge, a comparison far behind its base,
  many reviews or long job logs cut the section rather than failing its
  brief `SECTION: not gathered by its deadline` (jig's
  `domain/engine/briefs.md`, 1, *Within the deadline*, and 1.1). Every
  section the forge gathers with a deadline, a pull section, a reviews
  section and a CI section, has its cutoff three quarters of the way from
  the section's gathering to the brief's deadline (the setting
  `root.brief.gather_ms` after the gathering starts): 22.5 seconds in at
  the default 30, before the brief's settle point at seven eighths,
  leaving the brief its last quarter to size, cut and take its sections.
  A section gathered with no deadline has no cutoff. At its cutoff, a read
  of the section's still unanswered, at any stage, is left, its late
  answer dropped as a dropped section's is, and each read left is a `brief
  read refused` line (below). Past its cutoff the section starts no read
  but its identity's (below). What it does then depends on what it has:
  - *Past its identity,* it completes at once with what it gathered and a
    notice for what it left, within its byte budget. Its identity is the
    reads that name what it is for: a pull section's item and pull, the
    pull's head checked against the pin (below); a reviews section's
    reviews; a CI section's checks. Each notice says the section was cut
    at the brief's deadline, apart from the byte room's notices and a
    bound's (above), which keep their words:
    - its files, a pull section's:
      `[files cut at the brief's deadline: N of F files read; read_issues what=pull_files from page P lists the rest, and read_forge what=file reads a path at the head or the base]`,
      N the files its files reads answered, carried or named, F the
      forge's count of the pull's changed files from its pull read, as
      above, and P the page it was reading; where the forge gave no count,
      `N files read` alone. A pull read answered past the cutoff gives
      `0 of F files read` from page 1.
    - its comparison, a pull section's, whether its read was left or never
      started because the files were cut:
      `[comparison cut at the brief's deadline: not read; read it in your checkout]`
      for a resolution's and a repair's, and
      `[comparison cut at the brief's deadline: not read; read_issues what=pull_files lists the pull's files]`
      for any other pull section's. A section that would read no
      comparison (above) says nothing of one.
    - its remarks, a reviews section's:
      `[remarks cut at the brief's deadline: R of V reviews' remarks read; read_issues what=remarks reads a review's remarks]`,
      V the reviews the section kept at its head and R those whose remarks
      it read. A section so cut never says `[the request for changes gave
      no remarks]`: it did not read them all.
    - its logs, a CI section's: the failed check being read and each
      failed check after it keep their heads, each with
      `[job log cut at the brief's deadline; read_ci what=checks names its job and read_ci what=job reads its log]`
      in place of its log, as a log that could not be read keeps its
      check's marker (below).
  - *Short of its identity,* it has nothing usable. A half of a repair's
    pair whose other half still stands becomes words in its place, as a
    read of it that fails does (below), REASON `not read by the brief's
    deadline`: `[the pull request could not be read: not read by the
    brief's deadline]`, and the same for the reviews and the failed
    checks. Where both halves are short of their identities, the pull
    section becomes words and the reason waits on, whichever meets its
    cutoff first, since the reason is what the repair is for: a reason
    half stands on its pull section only once that is past its identity.
    Any other, a lone section or a half whose partner has already
    become words, waits on to the brief's deadline: its identity answered
    before then completes it at once, with the notices for all it left,
    and one still unanswered fails the brief `SECTION: not gathered by its
    deadline`, or `the forge client is busy` where the client refused it
    (below).
  This holds for every forge section gathered with a deadline: a
  resolution's, a repair's and a review's pull section, a reviews section
  and a CI section. A charter's document has no cutoff: it is optional,
  and the brief's settle point drops it unanswered.
- **A section the forge answers later has a read of its own,** in flight
  or waiting for the client's room, from the moment it is gathered to its
  answer; one that would have none is answered at once. Its cutoff and
  deadline time its reads, so a section waiting on no read would meet
  neither and fail its brief `SECTION: not gathered by its deadline` at
  every attempt, however long the deadline. A landing's repair's CI
  section (`../../landing.md`, 7.3; `../../landing-search.md`, 7) reads no
  forge: it is the failure the landing pinned, the round's failure while
  the landing keeps its row and then the gate's failed receipt, which
  outlives the row. Where the landing keeps neither, its row retired and
  its receipt past `receipts_per_gate`, the section is a read that fails,
  REASON `the landing no longer keeps its failure` (below), at once: words
  where its pull section stands (`[the failed checks could not be read:
  the landing no longer keeps its failure]`), its brief failed where not.
- **A pull section is pinned at the change's head,** its last head as
  the change records it: each push or save of a delegate's, reported or
  read afresh after a lost attempt, records it (`../../agent/README.md`, 5).
  A pull whose head is another than the pinned one is one of two:
  - *The task's own:* the change's branch's writer slot is held by the
    task whose brief it is, by any of its attempts, a run's or a lost
    attempt's awaiting its reread (jig's `domain/tasks/holds.md`, 3). An
    earlier attempt of the task pushed or saved that head, and the run's
    workspace starts from it. The section reads at the pull's head, its
    files and its comparison there, and says so after its identity:
    `[the pull's head is HEAD, past the change's recorded PINNED: an earlier attempt of this task pushed it, and this section reads at HEAD]`,
    each head in its first 12 hexadecimal digits. Nothing fails and no
    try is spent; the change takes the head as any push of its
    delegate's when the task ends.
  - *Another writer's:* the slot is free or another's. The move is the
    change's to judge (`README.md`, 2, and `drift.md`, 1), not the brief's: the
    section's read fails, and the connector wakes the change. A review's
    or a repair's read fails counting as capacity (the table below), so
    the task pauses with no try spent, as for a writer slot taken (jig's
    `domain/tasks/lifecycle.md`, 6). Its next step reads its branch
    afresh: a head it takes becomes the pin of the task's next try, and a
    drift holds the change for its cause, on its panel. While it is held,
    each try of the task pauses the same way, until a release takes the
    head (`drift.md`, 1) or a cancel ends the task. It is never a
    failure tried until the task is held.
    A resolution's read fails counting as transient instead, never as
    capacity: its change ends it at its next step (below), so a try
    meeting the same pin and the same head again is no room freeing by
    itself, and its tries bound it rather than a brief slot taken on
    every one of them.

  A lost attempt's reread that finds the slot claimed since by a later
  attempt of the same task records the head it read as the delegate's,
  undrifted, as one that frees the slot does, so the next brief pins it;
  it frees nothing, the slot being the later attempt's.

  **A resolution's moved head.** A resolution is asked for the conflict
  its change met at its last head, against its base, so a branch that
  has moved past that head leaves it a conflict nobody has read: the
  move may already hold the base, as a goal's catch-up landing into its
  branch does, or conflict otherwise. A change at
  resolving whose resolution has no run in progress (made and not yet
  started, between its tries, or held), its branch's writer slot free,
  and whose branch it reads at a head other than its last head, ends
  that resolution at that step, whoever moved the branch: it cancels it
  with the reason `the change's head moved to HEAD`, HEAD the head it
  read in full hexadecimal, as a green head's cancel names its head, and
  decides afresh as once its producer ends (`A resolution`, above),
  waiting for that task's terminal: it reads its branch, its base and
  its pull, never the cancelled delegate's status, and keeps the head it
  reads as its last head. With no conflict left, it goes on at that
  head, checked again; with a conflict left, it asks for a fresh
  resolution directly at that head and its base as now read, with no
  merge job before it, its section pinned there and gathered once,
  counted against its resolutions as any is, so a branch that keeps
  moving is bounded by them. A move the change takes for a drift
  (`drift.md`, 1) ends the resolution the same way, and then holds the
  change for the drift's cause, its release taking the head and deciding
  afresh as above. This cancel is the change's own: no hold says `its
  resolution, task N, was cancelled` for it (`A release`, below). A
  resolution whose run is in progress holds the writer slot, so the
  change decides nothing on its head until that run's push or end
  (`drift.md`, 1).

  The same rule ends a repair of a gate's refusal, a person's request
  for changes on the forge, a landing gate's failure, a CI failure,
  or CI failing after a clean update: its brief pins the failure at the
  last head, so a move leaves its work unjudged at the new head. At
  repairing, with that repair live but no run in progress (made and not
  yet started, between its tries, or held), its branch's writer slot
  free, and a branch read at another head, the change cancels the repair
  at that step with `the change's head moved to HEAD`, whoever moved it,
  a goal's landing into its branch included. It waits for the repair's
  terminal and decides afresh as once its producer ends, reading its
  branch, base and pull and keeping the head it reads; the cancelled
  repair's status causes no hold. Its counts remain spent. The new head
  is checked again and, if ready, forms a fresh landing whose gates
  prove its new candidate. A repair whose run is in progress keeps the
  writer slot as a resolution's does. A run claiming that slot while a
  change's read is in progress makes that read see it as running too,
  so an older read cannot end the repair on its own delivered head.
  A change already held for another writer's drift keeps that hold and
  its repair's brief as it stands (the open question below).

  **A delivered repair is taken before judging the new head.** A
  repair ending with a change result has delivered, even if another writer
  has since advanced its branch and its delivered head is an ancestor
  of the branch's head. Its terminal is recorded as a delivery, never
  turned into a delegate failure for that move. As for every pushing
  delegate's terminal, a gather already in progress starts again with
  a branch read sent after the terminal. The change reads its branch,
  base and pull afresh and takes the head it now reads, clearing its
  clean heads and judging its gates again there. A drift the read
  proves holds for its drift cause with its words (`drift.md`, 1),
  rather than saying that a repair which delivered failed. A live
  repair whose run is in progress is left to deliver; an idle repair
  whose branch moved before it delivered follows the cancel rule
  above, including while waiting for an answer to its question.

- **A brief section's read that fails.** Every section says why it could not
  be read, and how that counts (jig's `domain/engine/briefs.md`, 1.1):

  | What the read met | Counts as | REASON |
  |---|---|---|
  | `Busy`, still refused at the brief's deadline | capacity | `the forge client is busy` |
  | `NotConnected`: a wait for access, as for a stall (`stalls.md`, 1) | capacity | `the forge is not connected` |
  | no room for another section or read in the connector | capacity | `the connector has no room for another read` |
  | `Unavailable` | transient | `the forge could not be reached` |
  | `Timeout` | transient | `the forge did not answer in time` |
  | `RateLimited` | transient | `the forge is limiting calls` |
  | a review's or a repair's pull whose head is another writer's than the pinned one (above), or a change that has moved past it; a pull's files read the forge answers `Stale`, its head having moved between the pull's read and the files' | capacity | `the change's head moved` |
  | a resolution's pull whose head is another writer's than the pinned one (above) | transient | `the change's head moved` |
  | `InvalidAnswer` | permanent | `the forge's answer did not match its read` |
  | an answer of another shape | permanent | `the forge answered another read` |
  | `Forbidden`, `NotInstalled` | permanent | `the forge forbids this read`, `the forge App is not installed` |
  | `Missing`, `MissingJob` | permanent | `the forge has no such object` |
  | `TooLarge` | permanent | `the answer is past BOUND`, in a bound's words (below) |
  | any other error | permanent | `the forge refused this read` |
  | no change for the pull, its identity past the section's bytes, or a title that fills them; no repository for a home-access reason | permanent | `the change cannot be named in its section` |
  | failed checks whose heads do not fit their section | permanent | `the failed checks do not fit their section` |
  | a landing's repair whose landing keeps neither its row's failure nor the gate's failed receipt | permanent | `the landing no longer keeps its failure` |
  | a section the plan gave wrong bounds, or the connector no longer holds | permanent | `the connector cannot hold this section` |

- **A bound's words.** A `TooLarge` is named by its `Excess` (skein's
  `forge/README.md`, 3.4), in the same words wherever temper writes it: a
  brief's notices and its REASON above, a host tool's refusal
  (`../../agent/tools.md`) and the log's `brief read refused` line. BOUND is
  `forge.client.rows (LIMIT): SIZE rows`, `forge.client.answer_bytes
  (LIMIT): SIZE bytes` or `forge.client.op_bytes (LIMIT): SIZE bytes`,
  SIZE read `at least SIZE` where the size is not exact; `a bound of the
  forge's (LIMIT): SIZE` for `Forge`, or `a bound of the forge's` where it
  holds no figure; and `an unnamed bound` for `Unnamed`. LIMIT is the
  configured figure the client refused at, so the words agree with the
  configuration (`../../../protocol/configuration/limits.md`), as in
  `the answer is past forge.client.rows (8): 22 rows`. A pull section's
  comparison-omitted notice reads `[comparison omitted: past BOUND]`.

  A repair's two forge sections, its pull section and its reason (CI's or
  reviews'), are a pair, and each can stand without the other: the agent has
  the other's words and its forge tools to read again. A read of one that
  counts as transient or permanent, or one with nothing usable at its
  cutoff (above), becomes words in that section's place,
  after what it had gathered, and the brief goes on:
  `[the pull request could not be read: REASON]`,
  `[the failed checks could not be read: REASON]` or
  `[the reviews could not be read: REASON]`. The pull section's identity needs
  no read, so it always names the pull request.
  A repair with neither its reason nor its pull cannot start: when both fail,
  the second fails the brief with its own REASON, which counts by the table.
  A capacity refusal never becomes words: the brief fails and the task pauses,
  so the run starts with its sections once there is room.
  A section with no partner (a review's or a resolution's pull section) is
  what its run is for, so its read failing fails the brief. A charter's
  document stays optional (jig's `domain/engine/briefs.md`, 1), its failure leaving
  the missing marker. A failed job's log stays the check's own marker within
  a CI section that was read.
  Each read that fails, words or not, is one line in the engine's log, naming
  the task, the section, the stage, its REASON from the table, the pinned
  head and the head read where a head moved, and the forge's error where
  the forge answered one, in that order, as a tool read's `read refused`
  line does:
  `temper-box serve: brief read refused task=T section=pull stage=item reason=the forge's answer did not match its read error=InvalidAnswer`,
  and for a head another writer moved,
  `temper-box serve: brief read refused task=T section=pull stage=pull reason=the change's head moved pinned=PINNED head=HEAD`,
  each head in full hexadecimal, as the `read refused` line's `head=` is.
  A files read answered `Stale` read no head, so its line names the error
  alone:
  `temper-box serve: brief read refused task=T section=pull stage=files reason=the change's head moved error=Stale`.
  A read left at the section's cutoff names its section and stage, of
  any section and at any stage, and no error or heads:
  `temper-box serve: brief read refused task=T section=pull stage=compare reason=not read by the section's cutoff`.
  A read the forge answered and temper refused names no error: never
  `error=None`.
  A brief that fails adds one more, naming how it counts and its reason:
  `temper-box serve: brief failed task=T counts=permanent reason=pull: ...`.

## 2. Open questions

None.
