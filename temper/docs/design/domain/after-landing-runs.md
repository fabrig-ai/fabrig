# Running tiers after landing

This document holds tier definitions, scheduling and run reports, split from [`after-landing.md`](after-landing.md).

## 1. Tiers

- **A repository's tiers** are settings beside its gates (`after-landing.md`, section 11):
  each a name (letters, digits, `-` and `_`, at most 32 bytes), a
  workflow's file, its ignored paths, its report: the
  name of the artifact its run uploads and its format, a JUnit file or
  one test id per line; and the branch it follows.
- **The branch a tier follows** (`../draft/reference-code-production.notes.md`, N5) is named by
  the tier, `branch`, a branch of its repository; absent, it is the
  landing branch the repository's adoption names (`forge/adoption.md`), its
  main. A tier follows that branch alone: a move of any other branch of
  its repository (a goal's own branch, a release branch) starts none of
  its runs, and none of its verdicts judges a landing there. Following
  two branches takes two tiers, which may name one workflow.
- **Goal branches:** a tier whose `branch` is `goal`, the symbol
  `change`'s `landing` takes (`agent/tools-change.md`, 1), names no branch of
  that name: it follows the goals' own branches of its repository
  (`forge/branches.md`, 1), each from its goal's first landing into it to the
  goal's close, and judges no landing (`after-landing-runs.md`, section 2, and `after-landing.md`, section 5.4). It suits
  the gate without its slowest suites, checking a goal's work along the
  way while merges into it go on (`../draft/reference-code-production.notes.md`, N5); the
  repository's other tiers follow main.
- **A tier's span,** `span`, the least time between the starts of two
  of its runs on one branch; none when absent. A move within the span
  waits for its end, and the run then takes the newest tip. g's tier in
  the reference: 20 minutes.
- **A tier's key** is a gate's (`landing-proof.md`, 1.1), over the tier's own
  definition and ignored paths, and its receipts are a gate's: kept per
  project by key, made only from runs temper dispatched (`landing-proof.md`,
  1.2). A failed receipt keeps the failing tests' ids.
- **Dispatched as a gate is** (`landing-proof.md`, 2.1), never on the
  landing branch, which may move under the run: on a branch of
  temper's own made at the commit to run, `temper/tier/<tier>.<n>`, n
  the run's number in the tier's sequence, and a bisect's probe
  `temper/tier/<tier>.<n>.<k>` (skein's `Write::CreateBranch`), each
  within the authority's eight segments of a resource and parsed back
  by its dots, since a tier's name has none; deleted once the run has
  ended, with
  the inputs `temper_commit` and `temper_key`, and, for a rerun or a
  bisect's probe, `temper_tests`, the ids to run alone, one per line. A
  tier's workflow runs only those when it is given them. The run is
  found and checked at that commit, as a gate's is.
- **Only a whole run makes a receipt.** A rerun of failed tests and a
  bisect's probe run a part of the tier, so their outcomes are kept on
  the tier's record for the verdict they serve, never as receipts.
- **A landing on a branch no tier follows,** in a repository with no
  tier or on a branch none of its tiers names, is cleared as it
  lands.
- **A tier's start:** when a tier is first configured, it runs once at
  its branch's tip, read afresh (section 2), the baseline. Its failures, if any, are failing at
  the baseline: blamed on no landing, inherited by the first run after
  it, and kept on the tier's record. The first run's range starts after
  the baseline's tip.

A tier's span is excluded from its key. Default landing and explicitly named
branch definitions keep their byte encodings; following goal branches has a
distinct marker, so its key cannot equal either form merely because their
branch text matches (2026-11-04).

## 2. Following the mains

- **On each move of the tip of a branch a tier follows,** read as
  `forge/topics.md`, 1.2 reads landings, each tier following it with no run in
  flight starts one at the new tip. One with a run in flight waits, and
  when that run ends, the next starts at the newest tip, if it moved
  since. The forge follows a branch for its tiers as long as one names
  it, whether or not a goal or a change follows it too.
- **A baseline's tip is read afresh** (section 1's start): a tier with
  no judged tip, no run in flight and no baseline asks the forge for its
  branch's head, a fresh read of the tier's own (skein's `Read::Branch`),
  and its baseline is the answer's commit. The head the engine last knew
  never chooses it: a restart restores that head, and the branch may
  have moved since, by the very commit that added the tier's workflow.
  Until the answer, the tier starts nothing, and a move it sees then is
  no landing of its: the baseline's range is empty, and the first run's
  starts after the baseline's tip, as section 1 has it. A baseline
  started again, after its run was dropped (below), reads afresh too,
  and the moves seen meanwhile are within its tip. A run after a move
  reads nothing more: the move is the branch's tip as the forge last
  answered it.
- **A run in flight is polled** as a landing polls its runs
  (`landing-proof.md`, 2.1): at the client's `poll_at` for its repository,
  every `poll` or, while that repository hears, at its last hint plus
  `fallback`, and at once on a job's delivery (`forge/keeping-up.md`, 2).
- **A tip whose key has a receipt** is judged by the receipt, with no
  run: a documentation landing clears at once where every tier ignores
  it.
- **A run's range** is the commits after the last judged tip of that
  tier, exclusive, up to the run's tip, inclusive. The landings in it
  are the landings whose commits it holds: temper's, each with its
  changes, and those temper did not make, each a range of commits with
  no change of temper's.
- **On a goal's branch,** for a tier following goal branches: each
  branch is followed on its own, with its last run's tip, failures and
  start, while the tier keeps one run at a time across them, numbered
  in its one sequence. A branch is due when its tip has moved since its
  last run and its span has passed since that run started; when the
  tier has no run in flight, the branch due longest runs next, at its
  newest tip. The first run is at the first move after the goal's hold
  made the branch, so a branch no change has landed into is never run.
  A run's range holds no landing, so it judges none: the changes
  landing into a goal's branch are cleared as they land (`after-landing.md`, section 6). It
  takes no receipt and makes none: every run is dispatched, so its news
  names a run. The branch's following ends as its goal's hold is
  released; a run still in flight then ends with no news.
- **Faults** (`landing-proof.md`, 2.4's): rerun, then dispatched again, up to
  the project's `tier_retries`; one past its bound holds the tier's run
  task, whose escalation reaches the project's escalation role (jig's
  `domain/tasks/proposals.md`), WHY `held after N runner faults, last:
  KIND`.
- **A wait for a runner** is bounded as a gate run's is (`landing-proof.md`,
  2.1): the run's execution clock (`forge/change/stalls.md`, 2.2),
  counted from its dispatch or re-run and kept on its flight across a
  restart, counts its time waiting or blocked for a runner; past
  `forge.change_policy.runner_wait_ms` it holds the task so, WHY `held
  after M minutes waiting for a runner`, M that bound's whole minutes.
  A read of the run is due by the instant the clock would reach it. Its
  running has no bound (`after-landing.md`, section 13).
- **A held run says why** in its connector's words (jig's
  `client/domain/objects.md`, section 5): the held task's words name
  the tier and the run (`Tier NAME run N`), and its hold's words add
  WHY, then a clause for each job its run's last reading lists not
  completed, read then by skein's `Read::JobState` and worded as a
  gate's (`landing.md`, 4.2), `; JOB: waiting for a runner labelled L1,
  L2`, within the hold's bytes as a stall's (`forge/change/stalls.md`,
  4). The clauses live in the hold alone: a restart holds with WHY.
  Before it is held, the run posts the tier's verdict status on its tip
  as pending, WHY, linked to the last run (`after-landing.md`, section 9). The tier waits
  for a person's release, which dispatches the run again with its
  faults counted from 0, its landings still verifying; its verdict
  replaces that status.
- **A workflow missing at the run's commit:** a dispatch the forge
  refuses as missing (a 404, skein's `Error::Missing`) is a fault of its
  own kind, the run's workflow absent at the commit it was dispatched
  at. Counted and retried as any fault, it is worded, wherever a
  fault's kind is (the task's part words, the hold's `last: KIND`, the
  pending status), `workflow WORKFLOW not found at COMMIT`: WORKFLOW the
  tier's workflow file as its settings name it, COMMIT the full commit
  dispatched. A rerun refused as missing is `lost`, its forge run gone.
  Its kind is stored as the tier's other faults are, appended to them
  in place (`../protocol/store-versions.md`, section 1): every row stored before
  it still reads, and a restart keeps its words (the requester,
  2026-11-11).
- **A run whose task ends without its verdict,** cancelled by a person
  (the usual end of a held run) or ended with its project, is dropped
  as its task closes: in the decision that hears the close (jig's
  `domain/tasks/lifecycle.md`, 1), the hook a landing's cancellation
  takes too, never by a step of the task, which a closing task no longer
  takes. The tier's record is left with no run and no task, or with its
  next run's: no verdict, no news, no status; the run's landings go back
  to the front of the tier's range, still verifying, for the next run;
  its number is spent, the tier's sequence passing it, so no later run
  reuses its branch's name or its steps' purposes (`after-landing-storage.md`, section 1); and the
  tier starts its next run as a move would, at once when due, a baseline
  again, read afresh, when the dropped run was the baseline. The
  dropped run's branch stays on the forge (`after-landing.md`, section 14). A run ended
  done has left the record already: the step giving its verdict takes
  it.
- **A person's hold** (`../client/ctl/tools-acts.md`, section 1, `tier hold`
  and `tier release`): a project's owner, or the deployment's, holds a
  repository's tier with a reason, and releases it. While held, the
  tier starts no run: no baseline, no run at a move of its branch and,
  for a tier following goal branches, none on any goal's branch. Each
  move still adds its landings to the tier's range, verifying, and,
  under `wait: cleared`, the project's `verification_bound` counts as
  ever: past it they end landed and uncleared, and a later verdict
  still counts (`after-landing.md`, section 6).
  A run in flight when the hold comes is let finish and judged, its
  verdict posted (`after-landing.md`, section 9); one held on its faults or its wait for a
  runner stays held, and a release of its task dispatches it again
  (*A held run*, above); cancelled,
  it is dropped (above), and no run starts in its place. The release
  starts the next run as a move would: at once when due, at the
  branch's newest tip, its range every landing since the last judged
  tip, those made while held among them; for goal branches, the branch
  due longest first. The hold is kept on the tier's record (`after-landing-storage.md`, section 1
  ): a restart restores the tier held, and the hold outlasts the
  tier's removal from the settings, as the record does (`after-landing.md`, section 11).
  It says who and why, in the tier's words, `held by party N: REASON`,
  N the holding party's number: a verifying change's line for the tier
  (`../client/forge/README.md`, 2.1) reads `NAME held by party N:
  REASON` while no run of the tier covers its landing. A hold posts no
  status (`after-landing.md`, section 9): no run of the tier's is there to link, and no
  task to post it. A hold of a held tier is refused, naming its holder,
  and a release of a tier not held is refused (the person, 2026-11-14,
  `../draft/tier-controls.md`).
- **A restored record whose run's task is gone** restores as a tier
  with no run in flight, never a stopped start: as the start links the
  forge's rows to jig's tasks (`forge/effects.md`, 2), a tier's record
  naming a task jig no longer holds drops its run as a close would
  (above), and the restart saves it so. A record never needs its run's
  task to restore, whatever left it naming one: a crash, or an older
  build that never heard the close.
- **A run whose tier is no longer configured,** its tier removed from
  its repository's settings or renamed while the run lives (a rename
  is a removal and a new tier: the new name starts its baseline,
  section 1), ends as its tier being gone. The first step of its task
  that finds no tier of that name in the settings, at a restart or
  after a settings revision, ends the task done, its words `Tier NAME
  run N: ended, its tier is no longer configured`, and leaves the
  tier's record as a dropped run's (above): no run and no task, its
  landings back at the front of the range, its number spent. It ends
  so in any part: dispatched, polled, held on its faults or its wait
  for a runner, probing, or posting a verdict, which is then never
  posted. It gives no verdict, no news and no status, and issues no
  effect; an effect it issued before is settled, its receipt or refusal
  read as any, before the task ends. Its branch and a probe's stay on the
  forge, as a dropped run's do (section 5). Until it ends, nothing of
  the run reads its tier's settings: a restart builds it no follower
  and walks no tree for it, and a read's or an effect's answer for it
  is taken without them. Its record stays and restores as any
  (`after-landing.md`, section 11), readable with its hold and
  sequence; the tier configured again under that name numbers its next
  run after it.

A failed receipt at the tip is treated as a failed-test rerun that failed
again. Its failures go directly to inheritance and blame, with no further
confirmation run (2026-11-04).

Among due goal branches, one never run sorts before one with a previous
start; otherwise the oldest start runs first. Equal starts, including two
branches never run, are ordered by goal number, lowest first (2026-11-04).

## 3. What a run reports

- **Passed or failed,** from the run's conclusion at its tip.
- **Its failing tests by id,** read from the artifact the tier names
  (skein's `Read::Artifact`): a zip of one file, a JUnit file, whose
  failed and errored test cases are its ids (`classname.name`), or a
  text file of one id per line. Ids are bytes, each at most
  `test_id_bytes`, at most `failing_tests` per run; past it, the run's
  failures are "many", and its verdict blames by landing alone, never by
  test.
- **A many report's names,** what a report read as many still names, so
  that whoever repairs it reads its tests (2026-11-19):
  - **its tests:** the failed ids it read whole within `test_id_bytes`,
    unique, the lowest `failing_tests` of them in byte order;
  - **its more:** a count of the report's other failed cases or lines,
    each counted once as the report lists them: an id past
    `test_id_bytes` among them, and one a lower id took the place of;
  - **cut or not:** a read cut at its bound (`after-landing-storage.md`, section 1, *Reads*) reads
    what it holds whole before the cut, a `lines` file's complete lines
    and a JUnit file's cases closed before it, and ends there with no
    fault; its more is then a floor. Where the part before the cut is
    one its format cannot read, the names are none: the run is many as
    before.

  The names are shown, never judged: a many run is judged as many
  (`after-landing.md`, 5.2) whatever it names, and its reruns, probes and receipts read
  none. They come from the judged run's whole part alone, at its tip,
  or at a goal's branch's tip for a tier following goal branches (`after-landing.md`, 5.4);
  a tip judged by its receipt (section 2) has none and says `many`, as
  does a many with no names. Wherever they are shown, the ids come
  first, then the remainder: `and K more`, K the more and the ids the
  room left out; for a cut report, `and K more, its report cut at the
  read's bound`, or `and more, its report cut at the read's bound`
  where K is 0. A report read as many by a gate (`landing-search-by-test.md`,
  2) keeps no names.
- **A failed run that names no test,** or whose artifact is absent or
  unreadable, is a fault: the run is retried, never blamed.
- **A landing gate's report** is read otherwise, its errored cases left
  out, and a failed run naming no test, or with a report not read, no
  fault; only an absent one is (`landing-search-by-test.md`, 2).

## 4. Open questions

- **Tiers on the forge's runners or on workers:** both are allowed by
  the draft; the forge's runners here, workers once a worker runs jobs.

- **A tier's running,** unbounded: a run that runs and never ends is
  held by nothing, a bound being a new figure, the user's; and a
  release after a runner wait leaves the forge's waiting run queued,
  no write cancelling one, its result never read. Default: so (the
  request of 2026-11-21).

- **Records of removed tiers** stay (`after-landing.md`, section 11),
  so each tier removed or renamed keeps a record within the forge's
  repositories times `tiers`. When one may leave, its sequence and hold
  kept otherwise, is open. Default: it stays (2026-11-25).

## 5. Later

- **A dropped run's branch:** a run dropped as its task closes (`after-landing-runs.md`, section 2
  ) leaves its `temper/tier/<tier>.<n>` branch on the forge, since a
  closing task makes no effect, and no later run takes its name, its
  number being spent. Deleting it, by the tier's next run or by a sweep
  of temper's tier branches, waits; meanwhile a person deletes it.
