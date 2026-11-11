# The change procedure

## 1. What a change is

A change task asks for one change to one repository, landed into one
branch. Its parameters:

- **the repository and the landing branch;**
- **the branch,** held by the task: one it is handed, already pushed (a
  chat's small fix, handed over by the chat that made it: jig's
  `domain/tasks/holds.md`, 4), or its own, named after its tree
  (`../branches.md`), which a producing task pushes; or, held by its
  requester and not by the task, its requester's held branch: the
  requester's own goal branch (`../branches.md`, 1), which the change lands whole
  (below);
- **what to produce,** when nothing is pushed yet: the spec of its
  producing task, and its producer's charter, which its repairs,
  resolutions and checks' gates run under too: the one `change` names,
  or the forge's change charter (`gates.md`, 3); or, for a catch-up, the commit
  the merge job merges into a branch made at its landing branch's tip
  (`../README.md`, section 6), parameter 16, its producer that job and no agent; or,
  for a goal's close step, the close request (`../../../protocol/tooling.md`,
  4.2), bytes parameter 17, its producer the close job, kind 4, run at
  a branch made at its landing branch's tip, and no agent (`../branches.md`, 2);
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
  open the pull. This wait counts the stall as the producing phase does (`repairs.md`). A
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
  the default, asking no one; `foreman`, each settled first by a foreman
  of its own; `up`, each passed straight to its requester; and how many
  it takes, a bound beside the others.

A run asks for one with `change`, giving the words, and the root fills
in the rest from the project (`../../agent/tools.md`, 1). A run whose authority
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
model's index and its effort's, in decimal (`6;3.1 0.0 2;3.0`). A
change without parameter 11, or with an empty part, runs its producer
under the forge's change charter, or leaves that role to its charter's
default; a parameter 11 that does not spell them refuses the change's
start.

Before building a change batch, the root reserves one task for each
member and shares the remaining descendant capacity equally, bounded
by each member's change policy: its task allowance, plus its question
allowance for `foreman`. Room beyond a member's cap is redistributed;
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
repository, beside the kinds it makes (`../../agent/tools.md`, 1): a reviewer
reads its own pull request and its files, and a repair a failed job's
log, through the reads an agent may call, past what its brief carries.

A change made with questions `none` asks no one. What to change is
agreed before it is asked for, by a goal that asks first where its
person wants to agree a plan (`../../agent/agents.md`, 2), and the procedure
delegates its agents without the Message family. One made with
`foreman` or `up` relays its own agents' questions instead (`questions.md`, 1); what
follows describes `none`; an enabled route's fallback feedback follows 8.11.
A change with `none` takes no questions, answers or words, and says
so rather than drop one: at its next step, a question in its inbox is
answered, in the answer form to the question's number, that a change
takes no questions and that its requester is task R; a task's words are
answered with words saying the same, to that task; each reply commits
with the step that takes what it answers, so a retried step answers
once. A party's words stay unanswered, since a procedure's step
messages tasks alone (jig's `domain/tasks/messages.md`, 5): they are taken and
dropped, and the change's page says that a change takes no words. Its
amendments and cancels are its own (2, 4).

A goal across repositories is several change tasks, ordered by their
dependencies: skein's change lands before temper's that bumps its lock.

**A change on its requester's held branch** (`change`'s `"branch":
"mine"`, `../../agent/tools.md`, 1) is how a goal lands what it holds: the
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
  life (`../branches.md`, 1). A goal's landing made by the goal's closer (`../branches.md`, 2) is on
  the goal's held branch, its requester's requester's: the goal holds
  it, and the closer, which holds none, makes no other change on it. The change writes it only through its own delegates (its
  repairs and its resolution) and the updates it asks for, each taking
  the branch's writer slot under its requester's hold as it writes
  (jig's `domain/tasks/holds.md`, 3), as a change's delegates write its own
  branch. Its requester's other delegates write it too, a change landing
  into it among them: each such move is the change's new head (3).
- **One at a time:** a second change on the same held branch, while one
  lives, is refused at its batch, naming the first.
- **Closing deletes nothing:** landed, cancelled or failed, the change
  leaves the branch its requester's, deleted or kept as its requester
  closes (`../branches.md`). Cancelled, its pull request is closed as any
  (4), and its requester may make another.
- **Beside the run that made it:** the goal's own runs push design to
  the branch while the change lives (`../branches.md`, 3). That doesn't stop the change,
  which steps at the head it reads (3).

The question route is number parameter 14: 1 for `foreman`, 2 for `up`;
absent means `none`. The root writes it only for `foreman` or `up`. Any
other value refuses the change's start as a malformed parameter
(2026-11-04).

The close request a goal's close step carries is bytes parameter 17, the
request's exact text form (`../../../protocol/tooling.md`, 4.2), within the close
kind's spec bound less the command's path; a value in another form, beside
parameter 16 or on a change with a branch of its own pushed or handed,
refuses the change's start as a malformed parameter. Its start is refused,
naming the setting, where the change's project names no `close_job`, and
naming the kind where its authority does not delegate to kind 4. Like a
catch-up's, its producer is the job: the change makes its own branch at its
landing branch's tip, the job's checkout, and a job that pushed nothing ends
the change with no change, as a catch-up already contained does (`../branches.md`, 2).

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
sent. A merge in flight settles as its landing does (4): a change whose
own pull request merged is landed, never this. A held change judges it
once released. So a change started from it (`questions.md`, 1) finds its
dependency done, not cancelled, and goes on from its own head; jig's rules
stay as they are: a cancelled dependency holds its dependent, and one
started from is never removed (jig's `domain/tasks/lifecycle.md`, 6, and
`domain/connectors.md`, 6.1).

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
   dependencies' pushed heads merged in: `questions.md`, 1) and pushes the change's
   branch (`../../agent/README.md`, section 5). Its
   result is the head it pushed. An attempt whose answer was lost may
   have pushed all the same: its writer slot is freed for another task only once the
   branch has been read afresh (jig's `domain/tasks/holds.md`, 3), and a
   head found there then is the change's own (the branch is held by this
   change alone), so the change cancels its producing task, whose work
   is done, and goes on to open it, in the same commit that frees the
   slot, before any retry could claim it. A change handed a pushed
   branch starts at the next step. Every producing delegate's writable
   repository carries the change's landing branch beside its write
   destination, whatever its start, and the worker pins its tip for the
   project's checks (`../../worker.md`, 4.1 and section 5). A producing delegate
   retried while it still owns an existing change branch starts at that
   branch. Only a writable producer whose start is that destination gets
   the comparison for existing work;
   saved starts, repairs, resolution, gates and ordinary runs do not. The
   worker fetches the landing tip per hold and verifies an existing
   unlanded head through a guarded push (../../worker.md, section 5). This does
   not substitute a historical receipt for current ownership or drift checks.
   A catch-up's producer is the merge job (`repairs.md`): its landing branch's
   tip with the catch-up's commit merged in, pushed as the change's
   branch; what the job leaves goes to a resolution in a merge in
   progress of that commit at that tip, which pushes the branch. A
   catch-up whose commit the landing branch's tip already contains ends
   at once with no change, its words saying so.
2. **Opening:** its pull request into its landing branch, keyed by its
   branches, its body naming its goal's issue ("Part of
   `ai/temper#12`"). **No step past it without its pull:** CI runs on
   pull requests, so a change reaches checking or gating only with its
   pull request open. A change released from a hold at opening (its
   branch moved or missing, 3) comes back to this step at the head it
   then reads: it opens its pull request, or finds the one already open,
   before it reads CI. A release of any other hold that finds no pull
   request open does the same.
3. **Checking:** CI runs on its head, where its landing branch's rules
   ask CI passed (`gates.md`, 2). A failure is a repair (`repairs.md`): a task given the
   failing output, unless the runner caused it (below). A repository without CI is checked by its checks
   (`gates.md`, 1). Where they ask no CI, the change goes on to its gates at once,
   and neither a check nor CI at its head holds it or asks a repair.
   **A head already behind takes its base first.** Where its landing
   branch's rules ask up to date (`gates.md`, 2), this step's fresh reads (`../keeping-up.md`, 3) show
   its head lacks the base's tip, and CI read at that head has not concluded
   (unknown or pending), the change is updated now (`../README.md`, section 6), since its
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
   step having passed), or it failed with no step failed; never by its
   log's words. A failure in a workflow's own step, a cancelled job, and a
   failed check that names no job are the change's. When every failed
   check is the runner's and each one's job was re-run fewer than
   `reruns` times at this head, the change re-runs those jobs, one write
   each (`../README.md`, section 4, kind 17), instead of asking a repair.
   A job's re-runs at a head are counted by the forge's attempts, not
   kept by the change: a job at attempt *a* was run again *a* − 1 times,
   by temper or by anyone, and a new head's jobs start at their first.
   A re-run keeps what the head earned: it moves no head, so the gates'
   verdicts there stay valid and a live gate runs on; no repair is
   counted; under its queue the change keeps its turn (`../queues.md`);
   CI read at the head is pending again until the new attempts end, and
   the wait on it counts its stall afresh from the re-run. Each re-run's
   account, as every write's, names its job, its check and its reason
   (`set-up failed`, `completion failed`, `no step failed`). Otherwise
   the failure is a repair, as above, whose brief says, of each failed
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
   **updated** (`../README.md`, section 6): cleanly, and its new head is checked again,
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
   style (`../adoption.md`, 4), automatically: what more a landing needs is a landing
   rule's, which the core checks at the merge (`gates.md`, 2). Where the forge's
   own merge queue guards the branch, the merge puts the pull request in
   that queue at exactly its head, and the change lands or is ejected
   later (`../queues.md`, 1). The merge waits for dependencies (jig's
   `domain/connectors.md`, 4.1): the core refuses it while the change's
   task is kept, naming what it waits for, a backstop to the queue's own
   rule (`../queues.md`). Merged, the change is **landed**: its result is the
   landing (the pull request, the merge commit, the queue's group's
   where the forge's queue landed it, whose id in hexadecimal is the
   result's words' first line, the pull request with its address the
   second, and the landing said in words the third, `Landed: pull request
   #7 merged into main as 0123456789ab.`: `../../root.md`, section 5; then its
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
   cancelled, no-change and unrelated children supply no successful
   repair account. Only production sets the pull request's title (1).
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

## 3. Drift

While its branch's writer slot is taken (jig's `domain/tasks/holds.md`, 3),
by a run of one of its delegates, by a run that handed it the branch and
has not yet answered, or by an update it asked for, the change decides
nothing on its branch's head: forward moves by temper's identity are
taken as that writer's (jig's `domain/connectors.md`, section 11), and
the change reads the head afresh once the slot is freed. Otherwise:

- **Its pull request closed** by someone else: held, saying who and
  when. A release reopens it; a cancel closes the change.
- **Its branch moved** while nothing of temper's writes it, a late copy
  of its own update among them (`../README.md`, section 6), or by another
  identity, or not forward: held, the head kept as it is, the hold's
  words naming the head it expected and the one it found, each at the
  forge's short length (`its branch moved: expected 0123456789ab, found
  89abcdef0123`). Under its
  project's turn, the rewrite its own prepare made is the change's: the
  forge's rebase (`../../landing.md`, 5.2) or a prepare job's delivery
  (`../../worker.md`, 4.4). A release
  takes the new head as the change's and checks it, gates asked again,
  its pull request opened first where none is open (2, step 2).
- **Its branch deleted:** held. A release creates the branch again at
  the head temper last knew, which the forge keeps under the pull
  request, and reopens the pull request.
- **Its pull request retargeted** to another base: held. A release
  retargets it back, or takes the new base as its landing branch if its
  requester amends the change so.
- **Its pull request merged by someone else:** landed, like any other
  landing. A merged pull request on reused branch names is its own only
  when its recorded number or pushed head matches (`../README.md`, section 4); any other
  merged pull request is Missing and cannot land the change.

**On its requester's held branch** (1), the branch's writers are its
requester's delegates, one at a time, the change's own among them.
While the slot is taken by another of them (a change landing into the
branch, its merge job, its resolution), the change decides nothing on
its head, as above. Once the slot is freed it reads the head afresh,
and a forward move by temper's identity is its new head, never drift:
taken as after a repair, checked again (2, step 3), its gates asked
again; its repairs, resolutions and updates count on, and its merge is
never sent at a head it has not checked. Held, it decides nothing until
released, and the release takes the head as it then is, checked again.
A move with no writer of its requester's subtree in the slot, by
another identity, or not forward, is drift, as above.

**Its requester's own runs** (a goal's design writers, `../branches.md`, 3) are writers
the change doesn't wait for: they take no writer slot of the branch. A
forward move by temper's identity while one lives is that run's design
push, and the change's new head as it reads it, checked again as above,
while the run goes on, never "branch moved"; a move by another identity,
or not forward, is drift. The change's own delegates, its repairs, its
resolution and its merge job, take the slot as any claim does (jig's
`domain/tasks/holds.md`, 3), beside the design writer.

**Its own writes are its own.** A head a write of temper's made for the
change is the change's, recorded as its branch's head in the step that
settles the write, so no later read takes it for drift:

- **Its update** (`../README.md`, section 6): the head the forge's answer
  names, or, its answer let go, a head read afresh that holds the base
  the update asked for (its `base`, though the base has moved on since).
  It goes on to CI on that head (2, step 3; under its turn, step 6).
- **On its requester's held branch, a merge into it** by a change of its
  requester's subtree (a goal's close-out or catch-up landing into the
  goal's branch, `../branches.md`): the merge holds the branch's writer
  slot from its entry until it settles, so the change decides nothing on
  the head meanwhile, as above; once settled, the merge's commit is the
  branch's head and the change's new head, checked again (2, step 3).
  The same holds for the merge job's push on the branch (`repairs.md`).

So a goal's landing whose own update, or its goal's close-out merged just
before, moves the goal's branch goes on to CI on that head and merges;
only a head no write of temper's in flight or settled explains is drift.

Nothing is written over: a held change makes no effect until it is
released. A drift is the change's to judge: what the forge says of a
known change's pull request or branch wakes its procedure, which holds
it for the cause above, rather than holding its task from outside before
the procedure has stepped, so the hold names its cause on the change's
panel.

## 4. Cancelled

A cancelled change closes as every task does (jig's `domain/tasks/lifecycle.md`,
1): its live delegates close first, their runs ended; an entry already
sent, such as a merge, settles, and what landed counts; then its hold is
released (jig's `domain/tasks/holds.md`, 5): its pull request closed with a
comment saying why, and its branch deleted, its commits still readable
under the closed pull request. A change whose merge landed meanwhile
ends landed, not cancelled.

## 5. Its panel

What the connector makes of a change for the client's panel
(`../../../client/forge/served.md`, 2), as the core asks for the document
(`../../routing.md`, 3), from its record and its working set:

- **From its record:** its repository, by owner and name from the
  repository's record; its branch, landing branch and head; its pull
  request's number; its state, with the step it was held at; the task
  its one delegate runs; its gates and their verdicts, each judged at
  the current head with its freshness (`gates.md`, 1), a gate whose review runs at
  that head pending until it reports; its repairs, resolutions and
  updates used, with their limits from `change_policy`; and its
  questions, its route, those taken of its bound and its open ones
  (`questions.md`, 1).
- **CI at its head** is what its last step read (`../keeping-up.md`, 3), a held change's
  read for a green head (`repairs.md`) among them, kept in memory
  beside its record, one verdict per change, while the change is live: a
  restart forgets it, and the panel says unknown until the change's next
  step reads it again. A change that has landed or ended keeps its last
  verdict at its exact head with its record (`ChangeRow`'s `ci`, written
  by the save its finish already makes), restored with the row, since no
  step will read it again and its panel stays as long as its record
  (`../../../client/forge/served.md`, 2). The panel carries separately whether the
  repository has CI and its effective landing rules ask for it (`gates.md`, 2).
  Where CI is not required, the client says “CI not required”, even if
  the retained observation is pending or failed. The observed verdict
  remains evidence at its exact head; a no-CI landing never makes it a
  passing verdict. The requirement is read from the held rules when the
  panel is made, including after restoration.
- **Its place in its queue** is computed as the document is made,
  in `../queues.md`'s order among the changes queued or first in its
  landing branch at the core's time: its place from 1, and the tasks
  ahead of it, nearest first.
- **Marked** as 9.2 of the client's says: its own subject each time its
  record is saved and each time a step reads a CI verdict at its head
  other than the one kept; and, when it enters its landing branch's
  queue, leaves it or moves in it, the changes behind it, nearest first,
  within the views' subjects marked per change (`../../root.md`, section 9).
  One past that bound shows its new place at its own next mark.
- **In its project's scope:** a change into a repository the project
  holds. Any other task, or a scope that is not that project, is
  unknown. Before the change's record exists, the engine still answers
  Gone; the client's bounded reopen and its limit are in
  `../../../client/forge/served.md`, 2, **Its subject**.
- **Kept as long as its record:** the record outlives the task's end, so
  a landed or cancelled change still has its panel, until a new change
  needs its room (section 7).
- **Its rules and its charter:** what the landing rules wait on at its
  head, the rule and criterion its last verdict named (`gates.md`, 2), and the
  forge's charter it was made with (`gates.md`, 3).

## 6. Drained

A drain is jig's (jig's `domain/connectors.md`, 6.7). A change in a
draining hold's scope is idle, and held, at its next step unless it is
landing: under the queue, while its merge is sent and not yet settled;
under the turn, while its landing has begun its proof and not yet landed
(`../../landing.md`, 4.2). Its producing, repair or resolution run finishes
its turn and parks, as every run under a drain does.

## 7. Its room

The forge keeps a record of each change, `forge.changes` of them at
most, live and ended alike: in memory and as the store's `Change` row
(`Key::Change`, `../../../protocol/store.md`, 2). A record outlives its
task (section 5), and leaves only when a new change needs its room, so
`forge.changes` bounds the changes kept at once, and through them the
changes live at once, never every change made.

- **Settled:** a change is over for good once its task has ended (its
  ending recorded and its release finished, so no release row of its
  task is left) and its state is `Ended`, `Cleared` or `NoChange`. A
  live change never goes, nor a landed one whose task lives.
- **Dropped last:** an `Uncleared` change, or a `Landed` or `Red` one,
  whose task has ended (its ending recorded, its release finished) is
  never stepped again (`../../after-landing.md`, 6). It goes only when
  no settled change can, oldest first, pinned as any settled change is,
  so a tier runner down for long refuses no new change while such a
  change can go (the user, 2026-11-11).
- **Pinned while read:** a settled change still stays while
  - **its goal lives:** the tracked goal it served (its row's `goal`),
    or else the tracked ancestor its goal's closer finds for it, is a
    live task. The closer reads every original change of its goal for
    the landed set, its close request and its report (`../branches.md`,
    2), and the close command settles each `[in flight: change N]` tag
    by it (`../../../protocol/tooling.md`, 4.2): a goal's change goes
    only once its goal has ended;
  - **a landing names it:** a landing row of `../../landing.md`, ended
    or not, holds it among its members, whose steps and document read
    each member's record (`../../landing.md`, 4 and 14).
- **Dropped when room is needed, oldest first:** a new change finding
  the table full drops, in the same decision that registers it, the
  settled, unpinned change with the lowest task number; task numbers
  are jig's, deployment-wide and never reused, so the lowest was made
  first. Its record leaves memory and its `Change` row is erased. While
  the table has room nothing is dropped, so a quiet deployment keeps as
  many panels as the table holds.
- **The root chooses:** only the root knows which goals live, so it
  picks the change to drop, a settled one first and one dropped last
  only when no settled one can go, as `start_change` makes the registration,
  and asks the forge to drop it first (`forge::Event::DropChange { task
  }`, then `forge::Event::Change`). The forge drops it only if its
  table is full and the change is still settled, or dropped last with
  no settled one left to go, and no landing names it; otherwise it
  drops nothing.
- **What goes with it:** every account the forge keeps by its task: its
  panel's CI, its settling, its start conflicts, its wakes and its
  deadline, its pending steps and goal-file reads (a late answer to one
  then finds nothing and is dropped), its goal files, and its `landed`
  commits, each erased where it is stored; and the queue repairs it
  made leave its landing branch's count of them. Its landing log entries stay
  (`../../after-landing.md`, 10), so its project's landed history and
  status, which read the log, keep it.
- **Its landed commits are bounded alike:** the forge's `landed` table,
  each merge temper made and its task (the store's `Landed` row,
  `Key::Landed`), holds `max(forge.landings, forge.changes)` entries
  at most. A merge finding it full makes room in the same decision,
  erasing one entry where it is stored: one whose task has no change
  record first (a dropped change's, or a record a deployment removed by
  hand), else one whose change is settled, else the oldest; lowest task
  number first, the merging change's own last. Every reader of the table
  asks about a change it keeps (its release, its tier's ownership of a
  landing, a subscription's own landing), so an entry without a record
  is read by no one. The merge's own effects never wait on that room:
  its landing branch's head moves to the merge, is saved and pushed
  (`Pushed`), and its tiers observe it, whether or not its entry is kept
  (the operator, 2026-11-11: a full table held goals on their own merges
  into g).
- **A dropped change reads as an unknown task:** its panel is Gone
  (section 5); a subscription naming it reads it as over, as it read
  its ended record, and wakes no one; a public merge one of whose
  members is gone shows that member from the landing log's own entry,
  its task, pull request and title; a tier reads its landing as one
  not its own (`../../after-landing.md`, 2).
- **Refused, named:** a new change that finds the table full and
  nothing to drop is held before it registers, its words naming the
  bound, as every limit's refusal does
  (`../../../protocol/configuration/limits.md`):
  `forge.changes = 128; required usage 129; reduce the named work or
  raise this limit in the deployment configuration, restart, then
  release`. A release registers it once a change can go.
- **Why no bound of its own for kept panels:** a second limit, the
  ended changes kept, would need its own room in the store's sizing,
  and could refuse a live change beside room held for panels. Dropping
  on demand gives every row to live changes first and keeps the most
  history the table can hold.

## 8. Open questions

- **Ten busy goals:** pinning each live goal's changes keeps every
  record its closer reads, but a deployment whose live goals hold more
  than `forge.changes` changes between them refuses the next, naming
  it, until one ends or the limit is raised
  (`../../../protocol/configuration/limits.md`). A
  closer reading a compact account of its goal's changes, kept beside
  its own row, would lift the pin.
