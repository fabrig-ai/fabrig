# A goal's reds

Provisional, 2026-11-13. Written from `../../draft/branch-reds.md`,
sections 3 to 5, 6's second and third points, 9 and 10, its milestone B1.
What temper keeps of the failures a goal's branch carries, how a goal's run
records and ends them, and how a delivery's check reads them at its start.
The reference project's check reads them as `tools/pre-pr/design.md`, 5
says. What is open is listed in section 9.

## 1. In one page

- **A red** is a failure a goal's branch carries that no live change of
  the goal owns: its key, as the project's check names a failure, the
  goal, the commit it was recorded at, why, who recorded it, when, and
  its generation (2).
- **Recorded and ended by the goal's run,** by the host tool `red`, as
  its coordinator judges; nobody else records one (3).
- **Bounded per goal** by `forge.branch_reds`: a record past it is
  refused and the goal held (3).
- **Ended** by `red`, and with its goal (4).
- **Read at each check's start:** the worker asks the engine for the reds
  of the goal its run's work lands into, keeps those whose commit the
  check's base contains, and writes them to `temper/reds` in the
  repository's git directory, in the check's own form (5).
- **The check waits for that file,** a bounded time, where the worker has
  said it writes one, so that a check never reads an earlier check's
  file for its own (5.3).
- **Kept** as two families of the forge connector's (6).

## 2. A red

- **Its fields:**
  - `goal`: the goal's task number, the task holding a goal branch
    (`branches.md`) whose run recorded it;
  - `key`: the check's name for the failure, opaque to temper: 1 to
    `forge.red_key_bytes` bytes of UTF-8 on one line, with no control
    character and no leading or trailing whitespace. The reference's
    check names its failures `test <binary id> <name>` and
    `lint <package> <code> <file>` (`../../../../../tools/pre-pr/design.md`, 5);
  - `head`: the commit it was recorded at, the goal branch's commit the
    asking change's check started from, kept as its 20 or 32 bytes and
    shown as 40 or 64 lowercase hexadecimal digits;
  - `why`: 1 to 256 bytes of UTF-8 on one line;
  - `by`: who recorded it, `task N` or `party N`. A run's `red` records
    as its task; no party records one yet (section 9);
  - `at`: the wall time of its record, in nanoseconds;
  - `generation`: the deployment's count of records when it was made,
    one more at each record (6).
- **Each key once in a goal.** A record of a key the goal already holds
  replaces that red whole: its head, why, recorder and time, under a new
  generation. So a later record of a key is told from an earlier one by
  its generation (`../../draft/branch-reds.md`, 6, which ends by it).
- **Bounded:** a goal holds at most `forge.branch_reds` reds
  (`../../protocol/configuration/limits.md`).
- **It applies where its cause is:** a check counts a red only where its
  base contains the red's head (5.2). A change based before it doesn't
  carry its cause, so a failure there is the change's own. temper checks
  nothing else of the head: a commit no checkout's base contains is a red
  that never counts.
- **An ended red goes at once:** its row is erased in the decision that
  ends it. What it was stays in the goal's activity, in the answer of the
  call that ended it, or of the call that recorded it.

## 3. The `red` tool

- **Of the delegate family,** declared where `finding` is
  (`../agent/tools.md`, the delegate family's tools), so offered to the
  stock charters 1, 5 and 13 to 16; the run's authority decides each
  call by that family. Its writes are named and decided once, as
  `finding`'s; `list` is a read.
- **A goal's alone:** a call whose run's task holds no goal branch is
  refused, `red: task N holds no goal branch; only a goal's run keeps
  reds`, an error, and the run goes on. The goal is always the run's own
  task: no call names another.
- **Its acts,** its input an object whose `act` says which:
  - `{"act": "record", "key", "at", "why"}`: `at` the commit the red is
    recorded at, 40 or 64 lowercase hexadecimal digits: the base the
    asking change's check names in its summary (`base <id>`). Answers
    `Recorded red KEY at AT (generation G); a check whose base contains
    AT passes it.`, ending, where it replaced a red of the same key,
    ` It replaced the one recorded at OLD (generation G0).`;
  - `{"act": "end", "key", "why"}`: answers `Ended red KEY, recorded at
    AT by WHO on DATE: WHY0.`, WHY0 its record's why. A key the goal
    doesn't hold is refused, `red: goal G holds no red KEY; red list
    shows what it holds`;
  - `{"act": "list"}`: `Goal G holds N reds (at most B):`, then a line
    each, in key order, `KEY at AT, generation G, by WHO, DATE: WHY`;
    with none, `Goal G holds no reds.` Cut to the answer's bound as
    `finding`'s list is, whole lines only, the last line
    `and K more`.

  DATE is the record's time, UTC, `YYYY-MM-DD HH:MM`; WHO is `task N`
  or `party N`.
- **Decoding refuses at once,** deciding nothing, each as an error naming
  the member: a member another act takes (`red: member why does not
  belong to this act`), a missing one, a key or why empty, too long, on
  more than one line or with a control character, an `at` not 40 or 64
  lowercase hexadecimal digits.
- **Past the bound:** a record of a key the goal doesn't hold, when it
  holds `forge.branch_reds` already, is refused, `red: goal G holds its
  most reds (at most B); end one, or look at what its branch carries`,
  and the goal's task is held, its account those words, as the root
  holds a task for a reason of the forge's (its `hold_reason`): a
  branch carrying that many is a goal for a person to look at, not a
  list to grow. Its release is the person's; a replacement of a key it
  holds is never past the bound.

## 4. How one ends

- **By `red`'s `end`** (3), with why.
- **With its goal:** a goal's reds are erased in the decision that
  releases its goal branch at its task's end, done, cancelled or failed
  (`branches.md`), so a goal that has ended holds none.
- **By what lands:** a red a change's check passed outright ends when
  that change lands into the goal's branch (`../../draft/branch-reds.md`,
  6, its milestone B2); not in this design yet.

## 5. The read at a check's start

### 5.1 What the engine answers

- **The ask names the run:** its task and attempt (5.4).
- **The goal is the one its work lands into:** for a run producing,
  repairing or resolving a change, the goal holding the change's
  destination, its `Change` row's `base` branch, as a goal branch
  (`branches.md`); for a goal's own run, its own. A repair of a goal's
  landing into main, whose destination no goal holds, has none: main's
  reds are no goal's (`../../draft/branch-reds.md`, 12).
- **The answer:** that goal's reds as pairs of key and head, every one,
  in key order, at most `forge.branch_reds`. A task that is not live, an
  attempt that is not its current one, or a run with no such goal is
  answered with none. The answer decides nothing and keeps no record.

### 5.2 What the worker writes

- **In each writable repository of the run's checkout with a pin,**
  `refs/temper/landing` (`../worker/checkouts.md`, *The landing pin*),
  the worker finds the check's base as the check does before stepping
  past unchecked saves (`../../../../../tools/pre-pr/design.md`, 2):
  `git merge-base refs/temper/landing HEAD`, with `MERGE_HEAD` as a third
  where a merge is in progress, the first it prints. It keeps each red
  whose head is an ancestor of that base or is that base
  (`git merge-base --is-ancestor HEAD_OF_RED BASE`). A head the checkout
  doesn't have is not contained; with no base, it keeps none.
- **`temper/reds`,** at `git rev-parse --git-path temper/reds` in that
  repository: the line `temper-reds 1`, then one line `- KEY` for each red
  it kept, in key order, each key once; every line ends with a line
  feed. A file of the header alone is no reds. It is written to
  `temper/reds.tmp` beside it and renamed over it, so the check reads a
  whole file or none.
- **`temper/reds-wait`,** beside it: the line `temper-reds-wait 1 MS`,
  MS the decimal milliseconds the check waits for `temper/reds` at its
  start, `checkout.reds_read_ms` plus 1,000. The worker writes it in each
  such repository at prepare, so a check knows the file is coming.

### 5.3 When

- **At prepare:** the worker removes `temper/reds` and writes
  `temper/reds-wait`.
- **At each check's start,** when the run tells its host its checks are
  starting (smith's `domain/run.md`, 8.1, the `long` of a check): the
  worker removes `temper/reds`, asks (5.4), and, with an answer within
  `checkout.reds_read_ms`, writes the file in each repository (5.2).
  An ask that fails or isn't answered in time writes nothing, and the
  check waits out its bound and judges with no reds. Every check starts
  afresh: a refused check, one run again for time and the check before
  a `finish` each read the reds as they are then, so a red recorded while
  the change waits on a question counts at its next check.
- **At each check's end,** when the run tells its host its checks ended:
  the worker removes `temper/reds`, so the next check waits for a file
  of its own. An answer that comes after its check ended, or after
  another check started, is dropped.
- **Why the check waits:** smith starts the check before it tells its
  host, and waits for nothing from it (smith's `domain/run.md`, 8.1), so
  the worker's file can only follow the check's start. The check, finding
  the marker, waits for the file (`../../../../../tools/pre-pr/design.md`, 5),
  and the file it reads is the one written for it.

### 5.4 The ask's way

Designed with the worker's wire (section 9).

## 6. Kept

- **Two families of the forge connector's** (`../../protocol/store.md`,
  2): `Red`, keyed by the goal, then the key's bytes, a red's row with
  every field of section 2; and `RedNumber`, one row, `{ last }`, the
  last generation given, as `ForgeNumber` keeps the last forge number.
- **Written** in the decision of `red`'s record, the red's row and
  `RedNumber` together; **erased** by its end and with its goal (4).
  `RedNumber` is never erased, so a generation is never given twice.
- **Bounded** by `forge.branch_reds` times `forge.holds`: a goal holding
  reds holds its goal branch, so at most `forge.holds` goals hold any.
  The restore refuses a store past it, as other families' bounds are
  checked.
- **Restored** at the forge's restart step, as every family is
  (`../../protocol/store.md`, 5).

## 7. The limits

`forge.branch_reds`, 64 in both profiles, and `forge.red_key_bytes`, 256,
among the forge's limits; `checkout.reds_read_ms`, 5,000, among the
checkout's (`../../protocol/configuration/limits.md`).

## 8. Testing

- **The forge domain, focused:** a record, a replacement under a new
  generation, an end, a list; a record past the bound refused; a goal's
  release erasing its reds; the answer for a change's producer, for a
  repair into main (none), for a goal's own run, for a task not live
  (none); a restore of each family, and one past the bound refused.
- **Fuzzy,** in the forge domain's `fuzzy` module: random records,
  replacements, ends and goal releases over a few goals and keys keep each
  goal within `forge.branch_reds`, each key once, its generations
  increasing, and every answer naming only the live records of its goal.
- **The store:** the families' golden bytes and the patch
  (`../../protocol/baseline.md`, 7).
- **The tool:** each act decoded, each refusal, each answer rendered,
  and the host tools' contract moved (`../../protocol/wires.md`, 6); the
  hold past the bound.
- **The worker:** the file written fresh at each check's start, a check
  run again included, and removed at its end and at prepare; reds whose
  head the base doesn't contain left out; a slow or failed ask writing
  none (section 9).

## 9. Open questions

- **The ask's way to the engine** (5.4): no worker asks the engine
  anything outside a run's own calls, and no wire bridges are built
  (`../../draft/redeploys.md`, 4, which its status says the design
  lags), so the ask moves the worker's wires for every part at once. The
  user is asked (2026-11-13); until then nothing writes `temper/reds`,
  and the check reads the tree's list alone.
- **A person recording a red,** by ctl or the page, as `party N`: the
  record keeps the recorder's kind for it. Default: none; a goal's run
  records them.
