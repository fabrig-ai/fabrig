# temper ctl task

Kind: reference

Tasks: read one and its parts, and act on it.

## task list

```
task list --project PROJECT [--label chat|goal] [--tracked] [--mine] [--ended] [--order activity|priority] [--after CURSOR] [--parent TASK] [--since DIGEST] [--until CONDITION] [--timeout DURATION]
  --project PROJECT: a project's name or number; required. The project
  --label chat|goal: one of chat or goal; optional. Only chats, or only goals
  --tracked: a switch. Only tracked tasks
  --mine: a switch. Only the tasks your party asked for
  --ended: a switch. Ended tasks, not live ones: a project's ended goals; ended chats with --mine, a task's ended delegates with --parent
  --order activity|priority: one of activity or priority; optional. By last activity or by priority; by number when absent
  --after CURSOR: a cursor, as a read prints it; optional. The cursor a page printed, to read on
  --parent TASK: a number; optional. Only this task's delegates
  --since DIGEST: a digest, as a read prints it; optional. Wait until it differs from the read that printed DIGEST
  --until CONDITION: a condition; optional. Wait until it holds: changed
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
Read a project's tasks, a line each
Prints: A line per task, its number, kind and phase, then `read on with: task list ... --after CURSOR` or `no more`; the first page of live tasks ends with a digest
--json: the record tasks.
Key: none.
A page holds at most 32 tasks. Live and ended tasks are read apart.
Only the first page of live tasks has a digest to wait on.
```

## task show

```
task show TASK [--tree] [--activity COUNT] [--run] [--since DIGEST] [--until CONDITION] [--timeout DURATION]
  TASK (task): a number; required. The task's number
  --tree: a switch. Add its live tree of delegates
  --activity COUNT: a count from 1 to 32; optional. Add its latest COUNT lines of activity
  --run: a switch. Add its live run
  --since DIGEST: a digest, as a read prints it; optional. Wait until it differs from the read that printed DIGEST
  --until CONDITION: a condition; optional. Wait until it holds: ended, waiting-on-me, held, replied, phase:PHASE, or changed. PHASE is a live phase in any case: waiting, idle, due, preparing, claimed, running, backing-off (or BackingOff), active, closing or held
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
Read a task: its state, what it waits on, and the acts you may take
Prints: `task N`, its kind, phase, project and tracking, its fields, `acts:` and a digest; each part asked for after it, with its own digest
--json: the record task.
Key: none.
`acts:` lists the calls the engine offers your party, ready to copy.
An act marked `(no tool)` has no tool here yet; one followed by a
colon and a reason is refused as it stands; one marked `proposed
to` asks another party first. A change task adds `next:` and the
call that reads its change. An ended task prints its result.
Text reads and waits fill `asked:` from the specification within their output
bounds. If words remain, `task text TASK --field specification --from OFFSET`
reads on from the first undisplayed byte. JSON keeps the bounded overview
and its digest.
```

## task activity

```
task activity TASK [--before CURSOR] [--limit COUNT]
  TASK (task): a number; required. The task's number
  --before CURSOR: a cursor, as a read prints it; optional. The cursor a page printed, to read further back
  --limit COUNT: a count from 1 to 32; optional. At most COUNT lines
Read a task's activity from its end
Prints: A line per event, oldest first, then `read on with: task activity TASK --before CURSOR` or `no more`
--json: the record activity.
Key: none.
A turn's conversation prints whole; read its supplied system text with task turn TASK TURN.
Each part is cut at 64 KB with `read on with:`;
a turn past a page's bytes prints as its size and
`read with: task turn TASK TURN`.
With `--limit`, the page keeps its last COUNT lines, and `read on
with:` repeats the read without the limit.
```

## task turn

```
task turn TASK TURN [--part PART] [--from OFFSET]
  TASK (task): a number; required. The task's number
  TURN (turn): an attempt and a turn, A.T; required. The attempt and turn, as A.T, as task activity prints them
  --part PART: a number; optional. Only this part
  --from OFFSET: a number; optional. From this byte of the part
Read one turn of a task's run, part by part
Prints: With --part, a cut part first prints `part P page N of M holds B bytes;` and the bytes left with the call that reads on, or `no more`; then a turn's header, then its words including the brief supplied to the run, calls and results in order; a cut part ends with `N bytes left; read on with: task turn TASK TURN --part PART --from OFFSET` or `no more`
--json: the record turn.
Key: none.
Read task activity TASK to find the attempt and turn as A.T, then
task turn TASK A.T. Archived turn text includes the brief supplied to the run,
with the selected charter-part instructions, charter document sections and
their recorded commits. These are the retained supplied words. Older turns
that did not retain system text have that context unavailable; task text
and current settings cannot recover it.
A part prints at most 65536 bytes at a time. A turn past ctl's
bound prints its size and `past the limit; report it`, and one it
cannot decode `unreadable; report it`, both with exit 0: report
either as a finding.
```

## task run

```
task run TASK [--since DIGEST] [--until CONDITION] [--timeout DURATION]
  TASK (task): a number; required. The task's number
  --since DIGEST: a digest, as a read prints it; optional. Wait until it differs from the read that printed DIGEST
  --until CONDITION: a condition; optional. Wait until it holds: ended, or changed
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
Read a task's live run: its attempt, its call in flight, its words so far
Prints: `run: A` or `run: none`, the call in flight, the words so far, and `digest: run`
--json: the record run.
Key: none.
With `--until ended`, it returns when the run ends, or at once when the task has no live run; a watch that ends with no read of it delivered is refused: `gone`, status 2.
```

## task message

```
task message TASK TEXT --key KEY
  TASK (task): a number; required. The task's number
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. The words
Send words to a task
Prints: `done: task N, message`
--json: the record message.
Key: --key KEY, required.
```

## task answer

```
task answer TASK QUESTION TEXT [--key KEY]
  TASK (task): a number; required. The task's number
  QUESTION (question): a number; required. The question's number, as the inbox prints it
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. The answer
task answer TASK --rev REV CHOICE TEXT [--key KEY]
  TASK (task): a number; required. The person task's number
  --rev REV: a number; required. The person-task revision shown by task show
  CHOICE (choice): a number; required. The contract's choice code
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. The reason for this choice
task answer TASK --rev REV TEXT [--key KEY]
  TASK (task): a number; required. The person task's number
  --rev REV: a number; required. The person-task revision shown by task show
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. The answer in words
task answer TASK --rev REV --approve STATE [TEXT] [--key KEY]
  TASK (task): a number; required. The person task's number
  --rev REV: a number; required. The person-task revision shown by task show
  --approve STATE: text, at most 1048576 bytes; required. The exact approval state, as lowercase hexadecimal; empty is an empty state
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; optional. An optional explanation
task answer TASK --rev REV --cannot REASON [--key KEY]
  TASK (task): a number; required. The person task's number
  --rev REV: a number; required. The person-task revision shown by task show
  --cannot REASON: text, at most 1048576 bytes, or - to read stdin; required. Why the task cannot be done
Answer a numbered question or a revisioned person task
Prints: `done: task N, answer`
--json: the record answer.
Key: --key KEY, optional.
Read task show first. A role-addressed person task must be taken
with task take before answering; task hand-back returns it to the role.
Use the printed revision: a changed contract refuses a stale answer.
Choice codes belong to the contract; TEXT explains the choice.
CHOICE answers only a task with choices; a task taking words is answered with TEXT alone.
TEXT's bound is the most any call takes; a person task may ask for less:
task show prints what it asks, and a refusal past it names the bound.
```

## task release

```
task release TASK --hold REV [--again] [--key KEY]
  TASK (task): a number; required. The task's number
  --hold REV: a number; required. The hold's revision, as the task prints it
  --again: a switch. Repeat an uncertain effect: only once you have checked it did not happen
Lift a task's hold
Prints: `done: task N, release`
--json: the record release.
Key: --key KEY, optional.
A task a party's hold holds is theirs to release, or a role's above
theirs: task show says whether you may. Releasing the stopped task
itself lifts its hold, with every task the hold holds.
```

## task stop

```
task stop TASK [--key KEY]
  TASK (task): a number; required. The task's number
Hold a task and every task below it, stopping their runs
Prints: `done: task N, stop`
--json: the record stop.
Key: --key KEY, optional.
The stop is a placed hold of yours over the task's subtree. The
tasks stay held until task release of the stopped task lifts it,
hold lift lifts it, or task cancel ends them.
```

## task cancel

```
task cancel TASK [--grace DURATION] REASON [--key KEY]
  TASK (task): a number; required. The task's number
  --grace DURATION: a duration: seconds, or a number with s, m or h; optional. How long its run may finish first; the engine's default when absent
  REASON (reason): text, at most 1048576 bytes, or - to read stdin; required. Why
Cancel a task and its tree
Prints: `done: task N, cancel`
--json: the record cancel.
Key: --key KEY, optional.
A task that has ended refuses a cancel (exit 2), saying how it
ended. The same call again under its key prints its first answer,
whatever the task's state now.
```

## task tree

```
task tree TASK [--since DIGEST] [--until CONDITION] [--timeout DURATION]
  TASK (task): a number; required. The task's number
  --since DIGEST: a digest, as a read prints it; optional. Wait until it differs from the read that printed DIGEST
  --until CONDITION: a condition; optional. Wait until it holds: ended, or changed
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
Read a task's live tree: the task and each delegate its window holds, a line each
Prints: `tree:`, the task, then a line per delegate; `cut:` and the call that reads the rest when it folds some; and `digest: tree`
--json: the record tree.
Key: none.
With `--until ended`, it returns when a delivery shows the tree ended or its watch reports completion. Watch completion prints `tree ended (watch)` and labels the retained rows `last observed snapshot`; their phases and digest describe that snapshot. JSON keeps the tree unchanged and adds `completion: {source: "watch", ended: true}` and `snapshot: "last_observed"` outside it. A watch that ends with no delivery is refused: `gone`, status 2.
A large tree folds what does not fit: read the rest with task list
--parent TASK, a page at a time.
```

## task amend

```
task amend TASK [--rev REV] [--priority PRIORITY | --untracked] [--deadline TIME | --no-deadline] [--instructions FILE] [--spec FILE] [--dependency TASK]... [--no-dependencies] [--model MODEL [--effort EFFORT]] [--tasks N] REASON [--key KEY]
  TASK (task): a number; required. The task's number
  --rev REV: a number; optional. The spec revision the task prints; not needed for its priority alone
  --priority PRIORITY: a number; optional. Track it at this priority
  --untracked: a switch. Stop tracking it
  --deadline TIME: text, at most 1048576 bytes; optional. Its new deadline, as 2026-10-24T17:00Z
  --no-deadline: a switch. Remove its deadline
  --instructions FILE: a file, at most 1048576 bytes; optional. A file of its whole instructions, or -; an empty file removes them
  --spec FILE: a file, at most 1048576 bytes; optional. A file of its spec's words, or -
  --dependency TASK: a number; repeated, at most 32. A task it waits on; repeated, the whole list
  --no-dependencies: a switch. Remove every dependency
  --model MODEL: text, at most 1048576 bytes, or - to read stdin; optional. A model the task may use, as models lists it
  --effort EFFORT: text, at most 1048576 bytes, or - to read stdin; optional. An effort that model offers
  REASON (reason): text, at most 1048576 bytes, or - to read stdin; required. Why
  --tasks N: a number; optional. The tasks its subtree may make over its life; the rest of its authority kept
Amend a task: each control you name is replaced, the rest kept
Prints: `done: task N, amend`, or `proposed: PROPOSAL, to RECIPIENT` when another party must agree
--json: the record amend.
Key: --key KEY, optional.
A task whose spec moved since your read fails with `read again:
task show TASK`. Each file is read before anything is sent, one
final newline trimmed, at most 32768 bytes of UTF-8. An amendment
that names no change fails (exit 1); one that names a value the task
already has is sent, and the engine answers it.
```

## task decide

```
task decide TASK --hold REV release|leave|pass-up [REASON] [--key KEY]
  TASK (task): a number; required. The held task's number
  --hold REV: a number; required. The hold's revision, as the task prints it
  release|leave|pass-up (outcome): one of release, leave or pass-up; required. The decision
  REASON (reason): text, at most 1048576 bytes, or - to read stdin; optional. Why; sent with leave alone
Decide the escalation of a held task
Prints: `done: decide`, or `decided first:`, by whom, how and when, if its decision is retained
--json: the record decide.
Key: --key KEY, optional.
A decision made already, by your party or another, answers
`decided first:` with exit 0 and changes nothing.
```

## task text

```
task text TASK [--field specification|instructions|result] [--from OFFSET]
  TASK (task): a number; required. The task's number
  --field specification|instructions|result: one of specification, instructions or result; optional. Which source to read; specification when absent
  --from OFFSET: a number; optional. Byte offset printed by the previous chunk; zero when absent
Read a chunk of a task's specification, effective instructions or kept report
Prints: A cut page first prints `FIELD page N of M holds B bytes;` and the bytes left with the call that reads on, or `no more`; then the source words, byte offsets and total size; then a continuation call or `no more`
--json: the record task-text.
Key: none.
Each chunk keeps UTF-8 characters whole. Follow the printed call
until no more. Instructions use the task's override, or its charter
at the task's recorded settings revision. Result reads the kept terminal
report; unavailable reports are unknown.
```

## task take

```
task take TASK [--key KEY]
  TASK (task): a number; required. The person task's number
Take a person task addressed to your role
Prints: `done: task N, take`
--json: the record take.
Key: --key KEY, optional.
Read task show for the current holder and offered acts.
```

## task hand-back

```
task hand-back TASK [--key KEY]
  TASK (task): a number; required. The person task's number
Return your person task to its role
Prints: `done: task N, hand-back`
--json: the record hand-back.
Key: --key KEY, optional.
Only the current holder may hand it back.
```

## task notes

```
task notes TASK --project PROJECT [--after CURSOR]
  TASK (task): a number; required. The task's number
  --project PROJECT: a project's name or number; required. The task's project
  --after CURSOR: a cursor, as a read prints it; optional. The cursor a page printed, to read on
Read a task's notes, each body whole
Prints: Each note's number, revision, description and whole body; `body cut; report it` when supplied cut, `read on with: task notes TASK --project PROJECT --after CURSOR` for a further page, or `no notes`
--json: the record notes.
Key: none.
Reads the task's own note scope, as a chat's document pane does.
Bodies print whole, including those longer than a turn's part.
An unknown task or an empty scope answers `no notes`.
```
