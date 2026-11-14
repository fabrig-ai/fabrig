# A goal's reds

Provisional, 2026-11-14. Written from `../../draft/branch-reds.md`,
sections 3 to 7, 9 and 10, its milestones B1 and B2. What temper keeps
of the failures a goal's branch carries, how a goal's run records and
ends them, how a delivery's check reads them at its start, and what the
check reports at its end: the reds it passed, which end when its change
lands, and the tests that failed only under load, filed as flakes. The
reference project's check reads and writes them as
`tools/pre-pr/design.md`, 4 to 7 says. What is open is listed in
section 9.

## 1. In one page

- **A red** is a failure a goal's branch carries that no live change of
  the goal owns: its key, as the project's check names a failure, the
  goal, the commit it was recorded at, why, who recorded it, when, and
  its generation (2).
- **Recorded and ended by the goal's run,** by the host tool `red`, as
  its coordinator judges; nobody else records one (3).
- **Bounded per goal** by `forge.branch_reds`: a record past it is
  refused and the goal held (3).
- **Ended** by `red`, with its goal, and by what lands (4).
- **Read at each check's start:** the worker asks the engine for the reds
  of the goal its run's work lands into, keeps those whose commit the
  check's base contains, and writes them to `temper/reds` in the
  repository's git directory, in the check's own form (5).
- **The check waits for that file,** a bounded time, where the worker has
  said it writes one, so that a check never reads an earlier check's
  file for its own (5.3).
- **Reported at each check's end,** passed or refused: the check writes
  `temper/check-results`, the reds it passed or failed and the tests
  that failed under load but passed when their process ran again (5.5);
  the worker reads it and sends it to the engine, an application's ask
  the engine keeps (5.6).
- **A change's passes kept,** from its passed checks, each red's latest
  observation; when the change lands into the goal's branch, each red it
  passed ends, unless recorded again since (5.7).
- **Flakes filed** as `flake` findings of the run's project, counted on an
  open one with the same signature (5.8).
- **Kept** as three families of the forge connector's (6).

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
- **By what lands:** a red a change's passed check passed outright ends
  when that change lands into the goal's branch, unless it was recorded
  again after that pass (5.7).

## 5. A check's reds and results

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
- **`temper/rerun`,** beside it: the line `temper-rerun 1 N`, N the
  decimal `checkout.rerun_processes`, the most failing tests' processes
  the check runs again after its run (5.5). Written at prepare with the
  marker, written to `temper/rerun.tmp` and renamed; a check that finds
  none runs nothing again.

### 5.3 When

- **At prepare:** the worker removes `temper/reds` and
  `temper/check-results`, and writes `temper/reds-wait` and
  `temper/rerun`.
- **At each check's start,** when the run tells its host its checks are
  starting (smith's `domain/run.md`, 8.1, the `long` of a check): the
  worker removes `temper/reds` and `temper/check-results`, asks (5.4),
  and, with an answer within
  `checkout.reds_read_ms`, writes the file in each repository (5.2).
  An ask that fails or isn't answered in time writes nothing, and the
  check waits out its bound and judges with no reds. Every check starts
  afresh: a refused check, one run again for time and the check before
  a `finish` each read the reds as they are then, so a red recorded while
  the change waits on a question counts at its next check.
- **At each check's end,** when the run tells its host its checks ended,
  passed or refused: the worker removes `temper/reds`, so the next check
  waits for a file of its own, and reads and reports
  `temper/check-results` (5.6). An answer that comes after its check
  ended, or after another check started, is dropped.
- **Why the check waits:** smith starts the check before it tells its
  host, and waits for nothing from it (smith's `domain/run.md`, 8.1), so
  the worker's file can only follow the check's start. The check, finding
  the marker, waits for the file (`../../../../../tools/pre-pr/design.md`, 5),
  and the file it reads is the one written for it.

### 5.4 The ask's way

- **An application's ask on jig's channel** (jig's
  `domain/hosts/hub.md`, 10): the worker's domain asks its hub, for the
  hosted run, with temper's bytes; the hub sends `ask` up, the run's task
  and attempt beside them; the engine's root answers from 5.1, and
  `asked` brings the answer down by its number (jig's
  `protocol/channel/records.md`, 1 and 2). It decides nothing and keeps
  nothing on either side but the worker's outstanding number.
- **temper's bytes,** `temper-worker-wire`'s (`../../protocol/channel.md`):
  the ask `Ask`, an enumeration whose one variant, `reds`, carries
  nothing, the run's identity being jig's; the answer `Answer`, an
  enumeration whose one variant, `reds`, is `Reds { reds }`, a list of at
  most `forge.branch_reds` `Red { key, head }`, the key at most
  `forge.red_key_bytes` and the head 20 or 32 bytes, in key order. An
  ask the engine refuses, or answers with bytes that don't decode, is a
  read that failed (5.3).
- **Its versions:** `jig-worker-wire` and `temper-worker-wire` each move
  by one, with their kept texts and bridges (jig's
  `protocol/channel/bridges.md`). A worker of the previous versions
  never asks, so its checks have no reds file, as a tree without the
  list has none (`../../draft/branch-reds.md`, 10). A worker of these
  versions meeting an engine of the previous ones can't send `ask` down,
  so it closes its channel at the statement and the engine goes first
  (jig's `protocol/channel/bridges.md`, 6.3).
- **The worker's figures:** an ask of reds is a few bytes; an answer at
  most `forge.branch_reds` reds of `forge.red_key_bytes`, which temper
  gives jig as the application's largest answer (jig's
  `protocol/channel/README.md`, 10). A report's ask is 5.6's.

### 5.5 The results file

The check's contract with the worker, as `temper/reds` is the worker's
with the check. The reference's check writes it as
`../../../../../tools/pre-pr/design.md`, 6 says.

- **Where and when:** `temper/check-results`, at `git rev-parse
  --git-path temper/check-results`, written by the check at its end,
  passed or refused, to `temper/check-results.tmp` beside it and renamed,
  so the worker reads a whole file or none. A check writes it only where
  it found temper's marker `temper/reds-wait` (5.2): a checkout outside
  temper gets none. A cache hit writes none, nor does a check stopped
  before its end.
- **Its form,** each line ending with a line feed:
  - `temper-check 1`;
  - `check TREE BASE VERDICT`: TREE the tree it checked and BASE its
    base, each 40 or 64 lowercase hexadecimal digits, BASE `-` where it
    had none; VERDICT `passed` or `refused`, as its exit says;
  - `passed KEY`: an entry of `temper/reds` its run selected and that
    passed outright: not ignored, starved, cancelled or unobserved, and
    not passing only when its process ran again;
  - `failed KEY`: an entry of `temper/reds` its run selected that failed,
    inherited as it was;
  - `flake KEY`: a test that failed in its run and passed when its
    process ran again (`../../draft/branch-reds.md`, 7; the reference's,
    `../../../../../tools/pre-pr/design.md`, 4), then its log under load,
    each line of it as `| TEXT`, an empty one as `|`, at most 1,536 bytes
    of text.

  The `passed` and `failed` lines come in key order, a key once and
  never in both; the `flake` lines after them, in the order the check ran
  them again, a key once, at most `temper/rerun`'s N. A key is the
  check's form of an entry, `test <binary id> <name>` or
  `lint <package> <code> <file>`, at most 512 bytes.
- **Bounded:** at most 262,144 bytes. The worker reads a file past it,
  or one with any other line (a wrong header or check line, an unknown
  word, a key past 512 bytes or on two lines, a `|` line not after a
  `flake`), as malformed, and reports nothing of it.

### 5.6 The report

- **At each check's end** (5.3), in each repository 5.2 writes the reds
  in, the worker reads `temper/check-results`, then removes it. The read
  is that checkout's next work, ahead of a delivery's push the run asks
  for after its check, so that the report goes up before the run's
  answer: the fence (jig's `domain/hosts/hub.md`, 10) refuses an ask
  once the attempt has ended. A missing file reports nothing: a cache
  hit, a check of a tree without the reference's check, a check
  stopped. A malformed one reports nothing of that repository, and the
  worker logs why.
- **One ask per repository with a file,** in tag order, each sent once
  the previous one is answered or refused: an application's ask, as 5.4's,
  the run's task and attempt beside it. The worker waits for nothing on
  it: the answer, `kept`, says the engine took it, and a refused report is
  lost, never sent again. A report not yet sent when the next check
  starts is dropped.
- **Its bytes,** `temper-worker-wire`'s (`../../protocol/channel.md`):
  `Ask` gains `results`, `CheckResults { tag, check, tree, base,
  verdict, passed, failed, flakes }`: `tag` the repository's, as `Left`
  names one; `check` the worker's number of the check within the
  attempt, from 1, its identity with the run's task and attempt; `tree`
  and `base` 20 or 32 bytes, `base` an option; `verdict` `Verdict { passed,
  refused }`; `passed` and `failed` lists of keys; `flakes` a list of
  `Flake { key, log }`, the log its `|` lines' text joined by line feeds.
  `Answer` gains `kept`, which carries nothing. Bounds: on the engine,
  `passed` and `failed` at most `forge.branch_reds` keys of
  `forge.red_key_bytes`; on the worker, the schema's ceilings, 256 keys
  of 512 bytes; on both, at most 16 flakes, each key at most 512 bytes
  and log at most 2,048 bytes. A report the engine can't decode is
  refused, as 5.4's ask is.
- **On the engine,** the service answers `kept` at once and hands the
  report to the root as the worker's input, with the run's task and
  attempt; the root decides on it in its next decision (5.7, 5.8).
  Nothing is kept for the ask itself (jig's `domain/hosts/hub.md`, 10):
  a report lost with an engine's restart is lost. A worker sends each
  check's report once, so the engine decides on each identity once.
- **Its versions:** `temper-worker-wire` moves by one, with its kept
  text and bridge. A worker of the previous version reports nothing:
  its checks write files no one reads. A worker of this version meeting
  an engine of the previous one can't send `results` down, so it closes
  at the statement, `Ask` counted among the parts a worker sends whole
  (jig's `protocol/channel/bridges.md`, 6.3), and the engine goes first,
  as for B1.
- **The worker's figures:** the application's largest ask is a report's
  worst case under the side's bounds above, within jig's ceiling for an
  ask's bytes (`../../protocol/channel.md`, 8).

### 5.7 Passes kept, and reds ended by what lands

- **The report's change** is the run's task, or its nearest requester,
  holding a `Change` row; its goal, the one holding that change's
  destination, as 5.1 finds them. A report keeps no passes where the run
  has no such change or goal, where its task is not live or its attempt
  not current, or where `tag` names a repository other than the
  change's.
- **A passed check's alone:** checked is delivered (smith's
  `domain/run.md`, 8.1), so a refused check's tree is never the
  change's, and its `passed` and `failed` lines change nothing. Its
  flakes are filed (5.8).
- **Each key's latest observation:** for each `passed KEY` the goal holds
  as a red, the change's pass of it, `RedPass { change, key, generation
  }`, its generation `RedNumber`'s last given (0 with none), replacing
  any; for each `failed KEY`, the change's pass of it erased; a key the
  report doesn't name keeps what the change's earlier passed checks
  observed. Then each pass of the change whose key the goal no longer
  holds is erased, so a change holds at most `forge.branch_reds`. A
  change's check selects what it changed and its dependents, so a later
  check, a resolution's above all, may run less than an earlier one:
  what it did not run again, the earlier pass still says.
- **At its landing:** in the decision that first records the change's
  merge into its destination, wherever that is recorded (a merge the
  forge made for it, a pull request found merged, a landing read back):
  each red of the goal whose key the change has a pass of, with the
  red's generation at most the pass's, ends, its row erased as `red`'s
  end erases it. A red recorded again after the pass, at a later
  generation, stays. Then the change's passes are erased. Its record's
  fields go with it; the change's landing is in its history.
- **Never landed, nothing ended:** a change that ends without a merge,
  or whose row is dropped, has its passes erased.
- **A cache hit** reports nothing (5.6), so the change keeps what its
  last passed check observed, the same tree's, since the check's cache
  holds its last pass alone (`../../../../../tools/pre-pr/design.md`, 7).

### 5.8 Flakes filed

- **Each `flake KEY`** of a report, of a passed or a refused check, files
  or counts a `flake` finding in the run's task's project
  (`../findings.md`, 4), in the decision on the report. Its signature:
  the repository `tag` names, the source `check`, the key as its one id,
  no first line.
- **Counted:** where the project has an open finding, of kind `flake`
  and not `dropped`, with that signature, its `seen` count grows by one,
  and nothing else changes and no one hears.
- **Filed:** otherwise, a finding of kind `flake`, severity `low`, no
  area, so its project's triager's (`../areas.md`, 5: routing by path is
  later), its filer the run's task, `seen` 1, the signature above, its
  title `Flake in a delivery's check: KEY`, cut to 200 bytes at a
  character's start, and its body, cut to 4,096 bytes from its end,
  marked `[cut]`:

  ```
  A test failed in a delivery's check and passed when its process ran again, with nothing else of the check running.
  Repository: OWNER/NAME
  Test: KEY
  Seen in task T's check C (attempt A): tree TREE, base BASE.
  Its log under load:
  LOG
  ```

  TREE and BASE as hexadecimal, BASE `none` without one. A filing the
  project's bound refuses (`../findings.md`, 9) is dropped; the rest of
  the report stands.
- **Once per check:** each report is decided once (5.6), so a flake is
  filed or counted once per check identity.

## 6. Kept

- **Three families of the forge connector's** (`../../protocol/store.md`,
  2): `Red`, keyed by the goal, then the key's bytes, a red's row with
  every field of section 2; `RedNumber`, one row, `{ last }`, the
  last generation given, as `ForgeNumber` keeps the last forge number;
  and `RedPass`, keyed by the change's task, then the key's bytes, `{
  change, key, generation }` (5.7).
- **Written** in the decision of `red`'s record, the red's row and
  `RedNumber` together; **erased** by its end, with its goal (4) and by
  its change's landing (5.7). `RedNumber` is never erased, so a
  generation is never given twice. A pass is written and erased in the
  decision on a passed check's report, and erased at its change's
  landing, its end without a merge and its row's drop (5.7).
- **Bounded:** reds by `forge.branch_reds` times `forge.holds`: a goal
  holding reds holds its goal branch, so at most `forge.holds` goals
  hold any; passes by `forge.branch_reds` times `forge.changes`. The
  restore refuses a store past either, as other families' bounds are
  checked.
- **Restored** at the forge's restart step, as every family is
  (`../../protocol/store.md`, 5).

## 7. The limits

`forge.branch_reds`, 64 in both profiles, and `forge.red_key_bytes`, 256,
among the forge's limits; `checkout.reds_read_ms`, 5,000, and
`checkout.rerun_processes`, 4, from 0 to 16, among the checkout's
(`../../protocol/configuration/limits.md`). `rerun_processes` is the
worker's, which writes it for the check (5.2), though the draft names it
among the forge's; 16 is the report's ceiling of flakes (5.6).

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
  and the host tools' contract moved (`../../protocol/wires.md`, 7); the
  hold past the bound.
- **The ask:** the engine's answer to a producer's ask, to one of an
  attempt that isn't live (refused), and to bytes that don't decode;
  both wires' patches and bridges, by their crates' tests.
- **The worker:** the file written fresh at each check's start, a check
  run again included, and removed at its end and at prepare; reds whose
  head the base doesn't contain left out; a slow or failed ask writing
  none; a late answer dropped.
- **The forge domain's passes, focused:** a passed check's pass kept with
  the generation it was observed at, a `failed` line erasing one, a key
  the report doesn't name keeping its pass, a refused check's lines
  changing nothing, a pass of a key the goal doesn't hold not kept; at
  the change's landing, by each way a merge is recorded, a passed red
  ended and one recorded again after the pass kept; a change ending
  without a merge, and one dropped, ending nothing and erasing its
  passes; a restore of `RedPass`, and one past its bound refused.
- **Fuzzy,** in the same module: random records, replacements, ends,
  reports (passed and refused, with passes and failures), landings,
  ends without a merge and goal releases keep each goal within
  `forge.branch_reds` and each change's passes within it, each key once,
  and end a red only at a landing of a change whose latest passed check
  passed it after its record.
- **Flakes, focused:** a flake filed with its title, body, signature and
  `seen` 1; a second report's flake of the same key and repository
  counted on it; one of another repository, or after the first is
  dropped, filed anew; a refused check's flake filed; the project's bound
  refusing a filing without refusing the report.
- **The report's way:** the engine answering `kept` and deciding on a
  report, a report of an attempt not live refused, bytes that don't
  decode refused; `temper-worker-wire`'s patch and bridge by its crate's
  tests.
- **The worker's report:** `temper/rerun` written at prepare;
  `temper/check-results` removed at prepare and at each check's start,
  read at each check's end, a refused check's included, and sent with
  the check's number; no file, a malformed one and an oversized one
  reporting nothing; two repositories reported one after the other; the
  report going up before a `finish`'s run answer.
- **A story across both,** in the engine service's channel world, its
  scripted worker root asking through its real hub: a goal's red, a
  change's producer reporting a passed check that passed it, the pass
  kept, and a flake of the same report filed as a finding; then the
  change's landing ending the red.

## 9. Open questions

- **A person recording a red,** by ctl or the page, as `party N`: the
  record keeps the recorder's kind for it. Default: none; a goal's run
  records them.
- **A goal told of a red its landing ended,** as news, or kept in its
  activity. Default: not told, so that no coordinator wakes for it; the
  goal's `red list` no longer names it.
- **A refused report sent again,** with the engine deduplicating by its
  identity. Default: lost; a lost report keeps what the change kept,
  which ends less, never a red the change's last passed check failed,
  but where that check's own report was the one lost.
