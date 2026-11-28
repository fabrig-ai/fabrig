# The change procedure

## 1. What a change is

A change task asks for one change to one repository, landed into one
branch. Its parameters:

- **the repository and the landing branch;**
- **the branch,** held by the task: one it is handed, already pushed (a
  chat's small fix, handed over by the chat that made it: jig's
  `domain/tasks/holds.md`, 4), or its own, named after its tree
  (`../branches.md`), which a producing task pushes, or which temper
  makes at the head a person pushed, for a change they open from it
  (`opened.md`); or, held by its
  requester and not by the task, its requester's held branch: the
  requester's own goal branch (`../branches.md`, 1), which the change lands whole
  (below);
- **what to produce,** when nothing is pushed yet: the spec of its
  producing task, and its producer's charter, which its repairs,
  resolutions and checks' gates run under too: the one `change` names,
  or the forge's change charter (`gates.md`, 3); or, for a catch-up, the commit
  the merge job merges into a branch made at its landing branch's tip
  (`../updates.md`, section 1), parameter 16, its producer that job and no agent; or,
  for a goal's close step, the close request (`../../../protocol/tooling.md`,
  4.2), bytes parameter 17, its producer the close job, kind 4, run at
  a branch made at its landing branch's tip, and no agent (`../branches-closer.md`, 1);
- **its agents' LLM choices,** by role (jig's `domain/tasks/README.md`, section
  3): the producer's, for its producing task, its repairs, its
  resolutions and its checks' gates; the reviews', for its agents'
  gates. Each is given as the LLM choice of every delegate of its role;
  a role without one leaves its delegates to their runs' checks;
- **its gates** (`gates.md`, 1);
- **`with`,** under its project's turn: the changes it must land with,
  in any repository and of any goal its requester may name
  (`../../landing.md`, 4.1);
- **the pull request's title and body,** or what to make them from, and
  the goal it is part of. The change's words are its spec, kept whole
  for its producer's brief and bounded by the task's own limits, never
  by the forge's. The producing task's successful result supplies the pull's
  explicit title, retained with that result and copied into the change before
  opening. A visible pushed branch waits for this metadata; it alone cannot
  open the pull. This wait counts the stall as the producing phase does (`stalls.md`, 1). A
  failed producer supplies none, and holds whatever it pushed (`repairs.md`).
  The title is fitted to 256 UTF-8 bytes, preserving punctuation
  and marking a cut with “…”; it is never interpreted as sentences. A handed-in
  branch supplies its title as bytes parameter 10. A missing or empty title on
  that path uses “Update repository”, never context from the brief. An empty
  title reaches no forge by any path: a pull request opened with none takes
  that same default. The body
  holds the supplied body or the words whole, fitted to the forge client's
  operation bound and marked when cut. The complete production brief stays in
  the task spec, separately from the title and body. A goal's issue title uses
  its spec's first sentence, with the trailing full stop dropped and a cut at
  a word marked with “…”; a landing's name prefixes the explicit change title
  with “Land ”, fitted to its task bound without sentence extraction.
  A change is never refused or held for the length of its words;
- **its bounds:** repairs, resolutions, updates, the re-runs of a job
  the runner failed at one head (`reruns`, 2, step 3), and the stall of
  each wait;
- **its questions:** how it takes its agents' questions (`questions.md`, 1): `none`,
  the default, asking no one; `up`, each passed straight to its
  requester; and how many it takes, a bound beside the others.

A run asks for one with `change`, giving the words, and the root fills
in the rest from the project (`../../agent/tools-change.md`, 1). A run whose authority
holds jig's control family names, too, what a plan decides: the landing
branch, its own goal branch or a branch of the repository; its
dependencies, task numbers or changes of the same call; several
changes in one call, made as one batch; its producer's charter; and its
agents' LLM choices; and its questions. The landing branch is parameter 2, the
dependencies are the task's own (jig's `domain/tasks/README.md`, section 4),
and a change made with neither is as a chat's. The producer's charter
and the LLM choices are one text parameter, 11, so that a change stays
within the deployment's parameters per spec (jig's `tasks.parameters`,
4 in small): `C;P;R`, `C` the producer's charter's number in decimal,
`P` and `R` the producer's and the reviews' LLM choices, each part
empty where the change names none. A choice is spelled as jig keeps
it, by configured index (jig's `domain/tasks/README.md`, section 3): its main,
then each sub-agent, separated by one space, each `M` or `M.E`, the
model's index and its effort's, in decimal, then, where it names a
speed, `@` and the speed by its name, `standard` included, since jig
keeps a speed by name (`6;3.1@ultrafast 0.0 2;3.0`). An entry with no
`@` names no speed, so a parameter 11 spelled before speeds reads as it
did, and a follow's producer, spelled the same (`../follows.md`), reads
the speed too. A
change without parameter 11, or with an empty part, runs its producer
under the forge's change charter, or leaves that role to its charter's
default; a parameter 11 that does not spell them refuses the change's
start.

Before building a change batch, the root reserves one task for each
member and shares the remaining descendant capacity equally, bounded
by each member's change policy: its task allowance. Room beyond a member's cap is redistributed;
remainder slots go in call order. Each member needs at least one
descendant for its producer. One change keeps its policy allowance
within the requester's remaining authority. These allowances are
prospective within the call, not reservations retained across calls.
Admission remains atomic, with the same dependency and funding checks.
An insufficient batch names the available and minimum required task
capacity (two per member), and whether fewer changes per call can fit.
Below two available tasks, smaller calls cannot help. The requester's
remaining authority counts tasks already made over its life, including
ended tasks, rather than allowances granted by earlier calls.

Its agents hold the forge's reads (kind 1, `../README.md`, section 2) on its
repository, beside the kinds it makes (`../../agent/tools-change.md`, 1): a reviewer
reads its own pull request and its files, and a repair a failed job's
log, through the reads an agent may call, past what its brief carries.

Every agent delegate of a change, its producer, repairs, reviews and
resolutions alike, and its approvals' and checks' tasks, has its words
end with the change's current spec and rulings: the change's number
and spec revision, its words as they stand, each amendment's reason
and each answer given on it, read from the change task and its record
(`questions.md`, 1, *What every delegate is told*). They outrank its
original request: the pull request's title and body, CI's remarks and
reviews' are supporting evidence, and to preserve the requested change
is to preserve it as they state it (`repairs.md`). An amendment reaches
the delegate live when it is taken, a producer, repair or resolution
(`questions.md`, 1, *Its requester's words and amendments*), and a
review's verdict counts only at the spec revision it judged
(`gates.md`, 1, *Valid at a spec revision*).

A change made with questions `none` asks no one. What to change is
agreed before it is asked for, by a goal that asks first where its
person wants to agree a plan (`../../agent/agents.md`, 2), and the procedure
delegates its agents without the Message family. One made with
`up` relays its own agents' questions instead (`questions.md`, 1); what
follows describes `none`; an enabled route's fallback feedback follows 8.11.
A change with `none` takes no questions, answers or words, and says
so rather than drop one: at its next step, a question in its inbox is
answered, in the answer form to the question's number, that a change
takes no questions and that its requester is task R; a task's words are
answered with words saying the same, to that task, but its requester's
while it has a live producer, which pass to that producer under either
route (`questions.md`, 1, *Its requester's words and amendments*); each
reply commits with the step that takes what it answers, so a retried
step answers once. A party's words reach its live producer or are refused,
naming the amendment that carries them (`questions.md`, 1, *Its requester's
words and amendments*); the change's page says that a change takes no words. An
amendment reaches its live producer, repair or resolution under either
route (`questions.md`, 1, *Its requester's words and amendments*); its
cancels are its own (4).

A goal across repositories is several change tasks, ordered by their
dependencies: skein's change lands before temper's that bumps its lock.

**A change on its requester's held branch** (`change`'s `"branch":
"mine"`, `../../agent/tools-change.md`, 1) is how a goal lands what it holds: the
goal's landing, a change into the base whose branch is the goal's own
(`../branches.md`, 1). What a goal's runs write needs no such change: they push it
straight to the goal's branch (`../branches.md`, 3). It is an ordinary change, but for
the branch:

- **No producing step,** as for a branch handed over: it starts at
  opening, its head the branch's tip as read then. Its pull request is
  the branch's into its landing branch, its title its words' first line
  (bytes parameter 10, fitted as a handed branch's title is; an empty
  first line takes that path's default), its body its words. Every pull
  request's body holds at most 65,536 bytes of them, whatever
  `forge.client.op_bytes` allows, since a forge refuses more: one past it
  is cut and says that the change's full words are on its task in temper.
- **Held by its requester, not by the change.** The change takes no
  hold of the branch and hands none: its requester holds it for its own
  life (`../branches.md`, 1). A goal's landing made by the goal's closer (`../branches-closer.md`, 1) is on
  the goal's held branch, its requester's requester's: the goal holds
  it, and the closer, which holds none, makes no other change on it. The change writes it only through its own delegates (its
  repairs and its resolution) and the updates it asks for, each taking
  the branch's writer slot under its requester's hold as it writes
  (jig's `domain/tasks/holds.md`, 3), as a change's delegates write its own
  branch. Its requester's other delegates write it too, a change landing
  into it among them: each such move is the change's new head (`drift.md`, 1).
- **One at a time:** a second change on the same held branch, while one
  lives, is refused at its batch, naming the first.
- **Closing deletes nothing:** landed, cancelled or failed, the change
  leaves the branch its requester's, deleted or kept as its requester
  closes (`../branches.md`). Cancelled, its pull request is closed as any
  (4), and its requester may make another.
- **Beside the run that made it:** the goal's own runs push design to
  the branch while the change lives (`../branches.md`, 3). That doesn't stop the change,
  which steps at the head it reads (`drift.md`, 1).

The question route is number parameter 14: 2 for `up`; absent means
`none`. The root writes it only for `up`. Any other value, 1, the retired
foreman's (`questions.md`, 1), among them, refuses the change's start as a
malformed parameter (2026-11-04).

The close request a goal's close step carries is bytes parameter 17, the
request's exact text form (`../../../protocol/tooling.md`, 4.2), within the close
kind's spec bound less the command's path; a value in another form, beside
parameter 16 or on a change with a branch of its own pushed or handed,
refuses the change's start as a malformed parameter. Its start is refused,
naming the kind, where its authority does not delegate to kind 4. Like a
catch-up's, its producer is the job: the change makes its own branch at its
landing branch's tip, the job's checkout, and a job that pushed nothing ends
the change with no change, as a catch-up already contained does (`../branches-closer.md`, 1).
Its job finds the close command in its checkout, `.temper/hooks/close`
or, where it holds no `.temper/hooks/`, the project's `close_job`
(`../../../protocol/hooks.md`, 4.2 and 4.3). Bytes parameter 20, the
hook's path an earlier engine's closer wrote where its read found one,
is read as no parameter and never written.

The red landing a change repairs is carried as bytes parameter 12, its merge
commit as 40 or 64 lowercase hexadecimal digits. The start keeps it with the
change; a value outside that text form refuses the start as a malformed
parameter. Without it the change names no red landing to repair
(2026-11-04).

The source landings a follow change follows are bytes parameter 18,
`N:COMMIT[,COMMIT]...`, made by a follow's task alone and checked by the
root as it decides the delegate (`../follows.md`, section 4). The start
keeps them with the change (`follows`); a value outside that form refuses
the start as a malformed parameter. A change with parameter 18 takes
question route `up` with no parameter 14, never starts early, under its
follow's `owner` start produces only once its owner's word starts it, and
carries its follow's check (`../follows.md`, sections 4 and 6).

The source of a change a person opens from a branch they pushed is bytes
parameter 19, `BRANCH` or `BRANCH:COMMIT` (`opened.md`, 3.1), made by the
root's expansion of their request alone. The start keeps it with the
change (`source`); a value outside that form, or beside parameter 3,
number parameter 11 of 1, or parameter 16, 17 or 18, refuses the start as
a malformed parameter. Such a change has no producing task: its first
state is taking, which makes its own branch at the source's head
(`opened.md`, 4).

Documents:

- `opened.md`: a change opened from a pushed head.
- `gates.md`: gates and landing rules.
- `drift.md`: a change's drift and its own writes.
- `panel.md`: a change's panel.
- `records.md`: a change's records and room.
- `repairs.md`: repairs and their outcomes.
  - `repairs-resolutions.md`: merge jobs and conflict resolutions.
  - `repairs-briefs.md`: gathering the forge sections of a brief.
  - `repairs-releases.md`: holds and releases.
  - `repairs-later.md`: repairs, resolutions and releases' open questions.
- `questions.md`: questions and the change's words.
  - `notices.md`: what a relay tells the senders of answers and words.
- `branch-path.md`: the goal's branch path.
- `stalls.md`: waits and their bounds.

## 2. Its way to landing

An explicit successful no-change result from the producer ends the change
with its explanation, without opening a pull request, merging, holding or
escalating. It claims nothing landed. The requester receives the successful
result. Failure words never imply this outcome: production and forge errors
keep their recovery holds.

A change whose pushed head its landing branch already contains, landed
there by another change's merge or by any push, has nothing left to land:
it ends with no change, as a contained catch-up does (step 1), its words
saying so, `The landing branch already contains its head H.`, H in
hexadecimal as the catch-up's commit is. Its pull request, where one is
open, is closed with a comment of the same words, and its branch is
deleted as its hold is released, as a cancel's are (4). The connector
reads it at a step that reads the change's branch and its base: a head
that is the base's tip is contained with no read; a head that lacks the
base's tip is read once for each pair of head and tip, the commits the
head has that the tip lacks (`../keeping-up.md`, 3), an empty and complete
range meaning contained, and any other answer meaning not, or not known.
It is judged only while nothing of the change's own is under way on its
branch or its pull: no task that pushes its branch lives, its writer slot
is free, it has no drift, and no update, prepare or merge of its own is
sent; and only before its turn (step 6), since from its turn on the
same reading lands it (below). A merge in flight settles as its landing does (4): a change whose
own pull request merged is landed, never this. A held change judges it
once released. So a change started from it (`questions.md`, 1) finds its
dependency done, not cancelled, and goes on from its own head; jig's rules
stay as they are: a cancelled dependency holds its dependent, and one
started from is never removed (jig's `domain/tasks/lifecycle-holds.md`, 1, and
`jig/docs/design/domain/connectors-procedures.md`, 1.1).

**From its turn on, a contained head has landed** (2026-11-13). Under
its project's queue, from its turn (step 6, *First*) through its merge
(step 7, *Landing*), the same reading says its own head has landed: by
a merge made outside the forge's merge, say by hand, or by any push of
it into its base. Before it sends its merge, and at every step while
its merge is under way, being retried by the forge client, or failed,
a head its landing branch contains is landed, and no merge is sent or
sent again. Its merge is the base's tip the reading found holding the
head (a commit holding it, not necessarily the first), its result the
landing's as step 7 says, and it verifies and closes as any landed
change does. A merge of its own still in the forge client's hands is
withdrawn, and the answer to one already sent changes nothing. It reads
so at each step it takes there, and a move of its landing branch wakes
it to read afresh, so the reading ends a merge retried in a loop as it
ends one not yet sent. A failed merge holds the change (`repairs.md`)
only where that step's reading finds its head not contained, or cannot
tell. A held change reads it once released, at the step the release
brings. The pull request merged on the forge (`drift.md`, 1) stays the landing
wherever the forge says so, its merge commit the merge. Under its
project's turn, the change sends no merge: its landing's merge is the
landing's (`../../landing.md`, section 3), and a candidate already in
the branch stays the landing's to judge (8).

**A write that finds the outbox full waits** (2026-11-16). Each of the
change's writes below (its opening, a reopening, a branch made again,
an update, a prepare, a re-run, a follow's check and its merge) that
finds no room in the forge's outbox waits for it in the outbox's line,
in the order the writes found it full (`../room.md`, 3): never a hold,
and no write a person must release. It waits at the step its decision
started from, saving its place alone, holding no entry number and
polling nothing. Asked once room frees, it reads its facts afresh and
decides again, sending what that decision says, which may differ from
what waited; a decision for no write leaves the line. Its wait counts
against no allowance and no stall, keeps its place across a restart,
and ends with a cancel (`../room.md`, 4). Its panel says it waits
(`panel.md`, section 1).

Under its project's queue, the default. Under its project's turn, a
change has no checking step, and once ready its landing prepares,
proves and lands it (`../../landing.md`, section 3).

```
producing ─► opening ─► checking ─► gating ─► queued ─► first ─► landing ─► landed
                          ▲   │        │                  │
                          │   └────────┴──► repairing ────┤
                          │                               ├─ lacks the base's tip ─► updating
                          └───────────────────────────────┤                          │
                                                          └──── a conflict ◄─────────┘
                                                                      │
                                                                  resolving
```

1. **Producing:** a task for an agent, with the charter the change
   names, whose workspace starts from its landing branch (or its saved
   work; or, started early, from its landing branch with its
   dependencies' last checked heads merged in: `questions.md`, 1) and pushes the change's
   branch (`../../agent/README.md`, section 5). Its
   result is the head it pushed. **A result with a head always moves
   the change on** (2026-11-13): to opening its pull request (step 2),
   whatever its producer asked on its way (`questions.md`, 1) and
   however far its landing branch moved meanwhile. A head that lacks
   the branch's tip opens as any head does, and takes its base where
   checking and its turn say (steps 3 and 6). What the change reads or
   waits on before it takes that step is bounded as every wait is: the
   stall counts, or a named exemption stands (`stalls.md`, 1).
   An attempt whose answer was lost may
   have pushed all the same: its writer slot is freed for another task only once the
   branch has been read afresh (jig's `domain/tasks/holds.md`, 3), and a
   head found there then is the change's own (the branch is held by this
   change alone), so the change cancels its producing task, whose work
   is done, and goes on to open it, in the same commit that frees the
   slot, before any retry could claim it. A change handed a pushed
   branch starts at the next step. Every producing delegate's writable
   repository carries the change's landing branch beside its write
   destination, whatever its start, and the worker pins its tip for the
   project's checks (`../../worker/hosted-runs.md`, 1.1 and `../../worker/checkouts.md`). A producing delegate
   retried while it still owns an existing change branch starts at that
   branch. Only a writable producer whose start is that destination gets
   the comparison for existing work;
   saved starts, repairs, resolution, gates and ordinary runs do not. The
   worker fetches the landing tip per hold and verifies an existing
   unlanded head through a guarded push (../../worker/checkouts.md). This does
   not substitute a historical receipt for current ownership or drift checks.
   A catch-up's producer is the merge job (`repairs.md`): its landing branch's
   tip with the catch-up's commit merged in, pushed as the change's
   branch; what the job leaves goes to a resolution in a merge in
   progress of that commit at that tip, which pushes the branch. A
   catch-up whose commit the landing branch's tip already contains ends
   at once with no change, its words saying so.
2. **Opening:** its pull request into its landing branch, keyed by its
   branches, its body naming its goal's issue ("Part of
   `ai/temper#12`"). **Its own pull request** (2026-11-15): a change
   owns only a pull request it opened, or one whose number it stored.
   With no number, it looks one up by its branches and takes it only
   open (`../keeping-up.md`, 3): a closed match, merged or not, belongs
   to an ended change, a cancelled predecessor's on the same branch at
   the same head among them (4), and the change opens its own. It
   stores the number of the one it takes, or of the one it opens as the
   opening's answer names it, in the step that settles it. From then on
   it reads and writes that pull request alone, a restart included, and
   never opens another while it has one: its own found closed is
   reopened, by a release (`drift.md`, 1), and its own read as missing holds it as
   its pull request closed does (`drift.md`, 1). An uncertain opening found again
   by its branches (`../effects.md`, section 1) is the change's only
   where what is found is open; a closed one is no answer to it, and the
   change opens its own. **No step past it without its pull:** CI runs on
   pull requests, so a change reaches checking or gating only with its
   pull request open. A change released from a hold at opening (its
   branch moved or missing, 3) comes back to this step at the head it
   then reads: it opens its pull request, reopens its own, or finds the
   one already open, before it reads CI. A release of any other hold
   that finds its pull request not open does the same. **Its pull request at another head is
   not drift** (2026-11-13): the forge updates a pull request's head
   after a push, so a pull request it finds open naming another head
   than the branch's, while the branch is at the head the change
   expects (the head it entered this step with), is the forge catching
   up. The change waits for the pull request to name its head, its
   stall counting (`stalls.md`, 1), and goes on from there.
   Only its branch found at another head than the change expects is
   drift (`drift.md`, 1).
3. **Checking:** CI runs on its head, where its landing branch's rules
   ask CI passed (`gates.md`, 2). A failure is a repair (`repairs.md`): a task given the
   failing output, unless the runner caused it (below). A repository without CI is checked by its checks
   (`gates.md`, 1). Where they ask no CI, the change goes on to its gates at once,
   and neither a check nor CI at its head holds it or asks a repair.
   **A head already behind takes its base first.** Where its landing
   branch's rules ask up to date (`gates.md`, 2), this step's fresh reads (`../keeping-up.md`, 3) show
   its head lacks the base's tip, and CI read at that head has not concluded
   (unknown or pending), the change is updated now (`../updates.md`, section 1), since its
   turn could only replace that head. Of a head a push gave it, one such
   update at most: the head it leads to waits for its CI though the base
   moves again, and is updated at its turn. It counts as any update and never
   takes the change's last, which stays for its turn, so it makes no hold of
   its own; where it would, the change waits for CI as it would have. A
   conclusion is kept: a green head goes on to its gates and its turn and is
   updated there, and a red one is a repair, which takes its base first (below).
   CI not read at that head is no conclusion, and the change waits. Under
   rules asking none, nothing here applies. A run the forge has started on
   the head replaced stays the forge's, since temper has no effect that stops
   one: the change stops waiting for it, and what it spends goes by the
   repository's own CI settings.
   **The runner's failures run again** (`../../../draft/ci-on-forgejo-17.md`, 2). When CI
   fails at the head, the change reads the run of each failed check's job
   (skein's `Read::Run`). A job's failure is the runner's when its first
   failed step is the set-up step or the completion step (every workflow
   step having passed), or it failed with no step failed, or the job was
   cancelled by no one of temper's; never by its log's words. A failure in
   a workflow's own step and a failed check that names no job are the
   change's, and stand beside any runner's fault (`../judgement.md`, 2).
   When every failed check is the runner's and each one's job was re-run
   fewer than `reruns` times at this head, the change re-runs those jobs,
   one write each (`../effects.md`, section 1, kind 17), instead of asking
   a repair. **Past `reruns` it holds,** asking none: its cause the
   runner's, its words naming each job, its check and its fact, and that
   the change wasn't judged at this head (`repairs-releases.md`, 1). A
   green CI read at the head ends that hold as it ends a CI hold, and a
   release counts the jobs' re-runs afresh from their attempts at the
   release, so the change runs them again up to `reruns` more times.
   A job's re-runs at a head are counted by the forge's attempts, not
   kept by the change: a job at attempt *a* was run again *a* − 1 times,
   by temper or by anyone, and a new head's jobs start at their first.
   A re-run keeps what the head earned: it moves no head, so the gates'
   verdicts there stay valid and a live gate runs on; no repair is
   counted; under its queue the change keeps its turn (`../queues.md`);
   CI read at the head is pending again until the new attempts end, and
   the wait on it counts its stall afresh from the re-run. Each re-run's
   account, as every write's, names its job, its check and its reason
   (`set-up failed`, `completion failed`, `no step failed`, `cancelled`).
   Where a failed check is the change's, the failure is a repair, as
   above, whose brief says, of each failed
   check, whether its failure was the runner's, why, and how many times
   its job was run again at this head (`repairs.md`). A re-run whose
   answer is lost is found by the job's attempt moved past the one it
   named; not found, it was not made, and the change decides again on a
   fresh read. Only a change's CI at its head is re-run: the branch's
   own CI on its tip keeps its repair (`../queues.md`).
4. **Gating:** each gate without a verdict valid at this head runs: a
   task at the exact head. A blocking review or check that asks for
   changes is a repair, given its remarks; a blocking approval a person
   declines holds the change (`gates.md`, 1).
5. **Queued:** ready (CI passed where its landing branch's rules ask
   it, and every blocking gate holds, at its head, and its task is not
   kept by a dependency) and waiting its turn
   in its landing branch's queue (`../queues.md`).
6. **First:** its turn, which it keeps through what follows unless it
   leaves the queue (`../queues.md`). If its head lacks the base's tip and
   its landing branch's rules ask up to date (`gates.md`, 2), it is
   **updated** (`../updates.md`, section 1): cleanly, and its new head is checked again,
   its gates carried over where they count clean updates (`gates.md`, 1); or in
   conflict, and it leaves the queue to be
   **resolved** (`repairs.md`: the merge job, then a resolution for what it
   leaves, each only once its producer and repairs have ended, at its
   latest pushed head), its new head checked again and its gates asked again.
   Where they ask no update, it is merged as its head is, once the forge
   finds it mergeable; one the forge finds in conflict, or whose merge it
   refuses for a conflict, leaves the queue to be resolved the same way,
   the base merged in being its landing branch's tip as last read; its
   new head goes on as any head does under those rules, to its gates and
   its turn, with no check and no update.
   A pull request whose mergeability has not settled (`../keeping-up.md`, 2) is neither:
   the change waits for it, until the window's end at most, and the
   stall does not count that wait.
7. **Landing:** merged at exactly its head, in the repository's merge
   style (`../adoption.md`, 8), automatically, unless its landing branch
   already holds its head, which has landed it with no merge sent
   (*From its turn on, a contained head has landed*, above): what more a landing needs is a landing
   rule's, which the core checks at the merge (`gates.md`, 2). Where the forge's
   own merge queue guards the branch, the merge puts the pull request in
   that queue at exactly its head, and the change lands or is ejected
   later (`../queues.md`, 1). The merge waits for dependencies (jig's
   `domain/connectors-effects.md`, 1.1): the core refuses it while the change's
   task is kept, naming what it waits for, a backstop to the queue's own
   rule (`../queues.md`). Merged, the change is **landed**: its result is the
   landing (the pull request, the merge commit, the queue's group's
   where the forge's queue landed it, whose id in hexadecimal is the
   result's words' first line, the pull request with its address the
   second, and the landing said in words the third, `Landed: pull request
   #7 merged into main as 0123456789ab.`: `../../root-translation.md`, section 1; then its
   repairs, when any were asked, with their count, why each was asked
   (a check that failed, a review's remarks), and each successful repair's
   result words, attributed to its task in completion order; then its
   producer's last report, the words of its producing task's last
   successful result as that task wrote them, under a line `Report:`
   (none for a change handed a pushed branch). After repairs that line is
   `Producer report (historical; before repairs):`: its export description
   remains, while its unresolved-work statements describe the producing
   head, before those repairs. The repair accounts precede those caveats
   so a long producer report cannot spend their result space. Failed,
   cancelled and unrelated children supply no successful repair account,
   and a child that changed nothing supplies one only where it is a CI or
   a landing repair (`repairs.md`, 1, *A repair that changes nothing is
   checked once more*). Only production sets the pull request's title (1).
   Then what the landing passed, as facts the change itself saw: each
   check at the landed head with its conclusion, and the attempt it
   concluded at where its job ran again (`ci/test passed at attempt 2`,
   step 3), and each review's verdict
   (without repairs, their count remains after the checks). All that
   follows the three lines is cut to the result's bound, the change's
   contract's words (the deployment's `tasks.result_bytes`), from the end
   at a UTF-8 boundary and marked with the omitted byte count; a check or
   a test the change never saw is never named, so whoever reports the
   landing reports only what ran). The producer report is kept with the
   change from its producer's result, as its pull request's title is
   (1), bounded by that result's own bound. Successful repair words are
   retained with the repair reasons in their existing bounded history,
   including through restart; its prefix and omitted byte count survive
   later additions. It then verifies until its landing
   is cleared (`../../after-landing.md`, section 6), and closes, its branch
   deleted as its hold is released (`../branches.md`).

Lazy, too, for runs: before a repair or a resolution is created, the
change takes its base if its head lacks the base's tip and its rules ask
up to date (`gates.md`, 2), so a repair starts from current code. Beyond that and step 3's
head already behind, it never updates only because its base moved.

A completed change (landed, or ended without landing) cancels its live delegates, its producer, a repair, a resolution or a gate, whose work no longer counts, each with the reason `its change landed` or `its change ended`, in the step that completes it; it offers its result to the core once their terminals are heard, so no run whose change has completed parks in `wait`, and what waits on the change goes on. A delegate's successful result heard before the cancel is kept as any result is (its repair's words in the landed result). A landing observed while a resumed repair's terminal is still on its way leaves the procedure waiting; the delegate's terminal wakes it to offer the landed result.

## 4. Cancelled

A cancelled change closes as every task does (jig's `domain/tasks/lifecycle.md`,
1): its live delegates close first, their runs ended; an entry already
sent, such as a merge, settles, and what landed counts; then its hold is
released (jig's `domain/tasks/holds.md`, 5): its pull request closed with a
comment saying why, and its branch deleted, its commits still readable
under the closed pull request. A change whose merge landed meanwhile
ends landed, not cancelled.

**A pull request already closed counts as closed** (2026-11-17). One the
forge shows merged though temper recorded no merge of it, merged by
another hand or by the forge itself, or closed by hand, takes the close
as made: the forge's `Close` answers `Done` for a closed item (skein's
`forge/forgejo.md`, 1.4, *A close of a closed item*). So a change that
ends cancelled, or with no change (2), while its pull request shows
merged ends as its ending says, its branch deleted, and is never held
for the close; a change held for that refusal before ends at its next
release. Its ending stays its own: a merge temper did not record is
not its landing.

**One temper saw closed takes no close** (2026-11-17). The release
reads temper's own record of the pull request, the state its last poll
kept, and reads nothing afresh: where that record shows it closed,
merged or not, the release sends neither its comment nor its close and
counts the close as made, as a removed forge does (`../adoption-connection.md`, 1).
A pull request last seen open, or never seen, is closed as above, and a
close the forge refuses while temper's record has meanwhile come to show
it closed counts as made too; a refused close of one still seen open
holds, as any failed effect. The rest of its release is any ended
change's, and nothing the merge leaves holds it: the head branch of a
merged pull request is the forge's to keep or delete, so a branch the
forge no longer has counts as deleted, and the release deletes or
rewrites nothing else to end.

**A held release takes the ending asked again** (2026-11-17). A release
keeps its ending, but that a cancel takes over any other. One held for
a failed effect, asked again by the same root with another ending, as
when its parent fails after its cancel, starts again under the ending
asked: a failed ending closes no pull request and gives the change's
own branch to its root, so nothing is deleted that the first ending
had not. A release not held is still refused another ending than its
own, but for a cancel.

**Ended, it is frozen** (2026-11-15). Its closed pull request stays its
own: a change made after it on the same branch, at the same head or
not, opens its own (2, step 2), and never takes it. From its end, its
pull request and its branch raise no drift on any live task, its
successor, its goal and the branch's holder among them, whatever the
forge or a person does to them, a reopen by hand included
(`../keeping-up.md`, 3), and its history gains no event but those
`../../landing-history.md`, 2.2, keeps for after an end.

## 6. Drained

A drain is jig's (jig's `domain/connectors-procedures.md`, 1.7). A change in a
draining hold's scope is idle, and held, at its next step unless it is
landing: under the queue, while its merge is sent and not yet settled;
under the turn, while its landing has begun its proof and not yet landed
(`../../landing.md`, 4.2). Its producing, repair or resolution run finishes
its turn and parks, as every run under a drain does.

## 8. Open questions

- **A turn's candidate already in its branch** (2, *From its turn on, a
  contained head has landed*): under its project's turn, the landing's
  fast-forward recognises a branch at exactly its candidate, not one
  past it holding the candidate, so a candidate put into the branch by
  hand is merged again or prepared afresh. Default: the landing's, as
  it stands; the request of 2026-11-13 named a change's own merge.

- **A pull request a contained head leaves open** (2, *From its turn
  on, a contained head has landed*): a change landed by that reading
  leaves its pull request as the forge keeps it, open where the forge
  marks no merge made outside it; its branch is deleted as any landed
  change's is. Closing it with a comment naming the merge, as a
  no-change does, is an effect the request left out. Default: left
  open.


- **A forge's run on a head replaced** (`README.md`, 2, step 3): the update first ends the
  change's wait for that head's CI, not the run, since temper has no effect that
  stops one, and a pull request must exist before the forge can update it, so a
  run that starts at its opening starts first. Default: so; what it spends goes by
  the repository's own CI settings.

- **An early update against the bound** (`README.md`, 2, step 3): it counts against
  `forge.change_policy.updates` (3 in `small`, 80 in `standard`), so under a base
  that keeps moving `Hold::Updates` comes one update sooner. Not counting it would
  leave the bound untrue of update attempts. Default: it counts; the figure is the
  user's.
