# After landing

Provisional, 2026-10-27. What follows a landing: the slower checks a
repository runs on its newest main (its tiers), the verdict each run
gives the landings it covers, and a change that lives until its landing
is cleared, so that a goal ends only when its work is verified. It holds
for every landing on a repository's landing branch, by queue
(`forge.md`, sections 8 to 10), by turn (`landing.md`), or by anyone
outside temper. Findings, where a verdict files a flake or a red whose
goal has ended, are `findings.md`'s; a project without findings keeps
them in the verdict's own record. What is still open is section 13;
what waits, section 14.

## 1. In one page

- **A tier** is a check too slow or too broad for a gate (a browser
  suite, an exhaustive profile), defined like a gate: a dispatch-only
  workflow, its ignored paths, its key (section 2). It also names where
  its run reports its failing tests, and the branch it follows: its
  repository's landing branch, main, unless it names another or the
  goals' own branches.
- **Tiers follow their branch, not the landings** (section 3). When the
  tip of the branch a tier follows moves, by temper or by anyone, the
  tier runs on it: one run per tier at a time, the next taking the
  newest tip, so one run covers every landing since the last. A tip
  whose key has a receipt is judged by it, with no run. A landing on a
  branch no tier follows, a goal's own branch while its repository's
  tiers follow main, is cleared as it lands.
- **A tier following goal branches** (g's tier, section 2) follows each
  live goal's own branch on its own, at most once per its span, and
  judges no landing: a change into a goal's branch is cleared as it
  lands. A red there is rerun once alone, and what still fails is news
  to the goal holding the branch (section 7), never a verdict.
- **A run reports** pass or fail, and its failing tests by id, read from
  its artifact (section 4). A run that names no tests when it fails, or
  never ends, is a fault, never a red.
- **The verdict** (section 5): a green run clears every landing in its
  range. A red one runs its failed tests once more, alone; tests that
  pass then are a flake. Failures the previous run had are inherited,
  and blamed on no one new. New failures go to the one landing in the
  range, or to the first landing whose commit fails them, found by
  bisecting over the range's landings. A landing's verdict is its own:
  red only by failures blamed on it, cleared by the first run that
  covers it and blames it nothing.
- **A change lives to cleared** (section 6): landed, then verifying,
  then cleared, and its result comes only then, so its goal waits for
  verification. Past the project's bound with no verdict, it ends
  landed and uncleared, and a later verdict still counts.
- **A red goes to the landing's goals** (section 7) as news, with the
  failing tests and the log's end, and the goal repairs it with a change
  that names it. Other landings go on.
- **Landing news carries the trailers** of the commits it brought
  (section 8), so a designer's wake policy can wake on `Taught:` alone.
- **The forge shows** each verdict as a commit status,
  `temper/verdict/<tier>`, and a comment on each member's pull request
  (section 9).

## 2. Tiers

- **A repository's tiers** are settings beside its gates (section 11):
  each a name (letters, digits, `-` and `_`, at most 32 bytes), a
  workflow's file, its ignored paths, its report: the
  name of the artifact its run uploads and its format, a JUnit file or
  one test id per line; and the branch it follows.
- **The branch a tier follows** (`../draft/notes.md`, N5) is named by
  the tier, `branch`, a branch of its repository; absent, it is the
  landing branch the repository's adoption names (`forge.md`, section 4), its
  main. A tier follows that branch alone: a move of any other branch of
  its repository (a goal's own branch, a release branch) starts none of
  its runs, and none of its verdicts judges a landing there. Following
  two branches takes two tiers, which may name one workflow.
- **Goal branches:** a tier whose `branch` is `goal`, the symbol
  `change`'s `landing` takes (`agent.md`, 4.3.1), names no branch of
  that name: it follows the goals' own branches of its repository
  (`forge.md`, 11.1), each from its goal's first landing into it to the
  goal's close, and judges no landing (sections 3 and 5.4). It suits
  the gate without its slowest suites, checking a goal's work along the
  way while merges into it go on (`../draft/notes.md`, N5); the
  repository's other tiers follow main.
- **A tier's span,** `span`, the least time between the starts of two
  of its runs on one branch; none when absent. A move within the span
  waits for its end, and the run then takes the newest tip. g's tier in
  the reference: 20 minutes.
- **A tier's key** is a gate's (`landing.md`, 6.1), over the tier's own
  definition and ignored paths, and its receipts are a gate's: kept per
  project by key, made only from runs temper dispatched (`landing.md`,
  6.2). A failed receipt keeps the failing tests' ids.
- **Dispatched as a gate is** (`landing.md`, 7.1), never on the
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
  its branch's tip, the baseline. Its failures, if any, are failing at
  the baseline: blamed on no landing, inherited by the first run after
  it, and kept on the tier's record. The first run's range starts after
  the baseline's tip.

## 3. Following the mains

- **On each move of the tip of a branch a tier follows,** read as
  `forge.md`, 7.2 reads landings, each tier following it with no run in
  flight starts one at the new tip. One with a run in flight waits, and
  when that run ends, the next starts at the newest tip, if it moved
  since. The forge follows a branch for its tiers as long as one names
  it, whether or not a goal or a change follows it too.
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
  landing into a goal's branch are cleared as they land (section 6). It
  takes no receipt and makes none: every run is dispatched, so its news
  names a run. The branch's following ends as its goal's hold is
  released; a run still in flight then ends with no news.
- **Faults** (`landing.md`, 7.4's): rerun, then dispatched again, up to
  the project's `tier_retries`; one past its bound holds the tier's run
  task, whose escalation reaches the project's escalation role (jig's
  `domain/tasks.md`, section 9). A held procedure says why in its
  connector's words (jig's `client/domain/objects.md`, section 5): the
  held task's words name the tier and the run (`Tier NAME run N`), and
  its hold's words add `held after N runner faults, last: KIND`. Before
  it is held, the run posts the tier's verdict status on its tip as
  pending, `held after N runner faults, last: KIND`, linked to the last
  run (section 9). The tier waits for a person's release, which
  dispatches the run again with its faults counted from 0, its
  landings still verifying; its verdict replaces that status.

## 4. What a run reports

- **Passed or failed,** from the run's conclusion at its tip.
- **Its failing tests by id,** read from the artifact the tier names
  (skein's `Read::Artifact`): a zip of one file, a JUnit file, whose
  failed and errored test cases are its ids (`classname.name`), or a
  text file of one id per line. Ids are bytes, each at most
  `test_id_bytes`, at most `failing_tests` per run; past it, the run's
  failures are "many", and its verdict blames by landing alone, never by
  test.
- **A failed run that names no test,** or whose artifact is absent or
  unreadable, is a fault: the run is retried, never blamed.

## 5. The verdict

### 5.1 Green

Every landing in the run's range is cleared for that tier, but those
red by earlier failures it does not run (5.3).

### 5.2 Red

1. **Rerun once:** the failed tests alone at the same tip. Those that
   pass are a flake: not blamed, the flake kept on the tier's record and
   counted by test. If none fails again, the range is cleared, as if
   green.
2. **Inherited failures,** those that failed again and failed in the
   tier's previous judged run too, are blamed on no landing of this
   range: they stay with the landing that first failed them.
3. **New failures, one landing in the range:** they are that landing's.
4. **New failures, several landings:** bisected over the landings'
   commits in the range, oldest first, each probe the failing tests
   alone at a landing's commit (each was proved there, so the gates
   hold); the first landing whose commit fails a test owns it. A
   landing whose commit changes nothing the tier reads (its key equals
   the one before it) is skipped. The project's setting may say "tell
   all" instead: every landing in the range is a suspect, and each
   suspect's goal is told.
5. **A fault is not a red** (section 3).

### 5.3 A landing's verdict is its own

- **Red only by failures blamed on it,** per tier.
- **Cleared,** per tier, by the first judged run whose range holds it
  and blames it nothing; across tiers, once every tier following its
  branch has cleared it.
- **Once red, red** until a later run passes the failures blamed on it:
  then it is cleared, repaired by the landings in that run's range that
  name it (7).
- So a green tip never hides a red landing behind it, and a red run of
  inherited failures never reddens the landings after.

### 5.4 No verdict on a goal's branch

A run of a tier following goal branches gives no verdict. Green says
nothing to anyone. Red reruns its failed tests once alone, as 5.2, 1
does, and keeps the flakes on the tier's record; what fails again is
news to the goal holding the branch (section 7), each test marked *new*
or, where the previous run on that branch failed it too, *still
failing*. Nothing is blamed, bisected or inherited onto a landing, and
a fault is a fault (section 3).

## 6. A change lives to cleared

```
… ─► landed ─► verifying ─► cleared
                   │           ▲
                   └─► red ────┘
```

- **Landed means verifying.** The task lives, with no run, until its
  landing is cleared in every tier following the branch it landed on
  (at once, where no tier follows it, or only tiers following goal
  branches do: sections 2 and 3); its branch's
  hold stays until then, and the branch is deleted as the task closes
  (`forge.md`, 11), by queue and by turn alike. A goal's landing, a
  change on its requester's held branch (`forge.md`, 8.1), holds none:
  the goal's own branch stays the goal's, deleted as the goal closes
  once this change has landed it.
- **The red change read** names its merge commit, each tier and judged run,
  and every full test ID blamed on that landing. Owned failures and suspected
  failures are labelled separately; an overflow verdict stays explicit as
  `many tests`. Inherited failures do not redden a cleared landing.
- **Its result is given at cleared,** naming the landing and, for a red
  one, the landing that repaired it. Its goal waits for it as for any
  delegate's result; a coordinator that goes on sooner does not wait,
  and still hears the verdict as news. Each tier's clearing line says
  `Cleared by TIER run N.`, using the landing's retained verdict even
  after its tier task retires or another run judges a later tip. A
  repaired tier names its repairing run, `Cleared by TIER run N, which
  passed the failures blamed on it.` The result is composed before the
  landing's last verification row is released.
- **The bound:** past the project's `verification_bound` (two hours by
  default) with no verdict, because the runner was faulty or down, the
  change ends landed and uncleared, saying so, and its goal is told. A
  verdict that comes later still counts, through the landing log
  (section 10), which keeps the landing's changes after their
  verification rows go: cleared, it is news to the goals whose
  subscriptions still name the change; red, it stays on the tier's
  record and is filed as a finding (`findings.md`, section 4), since
  the goal may have ended.
- **Cancelling a verifying change** ends it landed and uncleared at
  once; its verdict counts as after the bound.

## 7. Where a red goes

- **To each goal of the landing's members** as news: the tier, the
  failing tests blamed on it, the run, and the end of each failed job's
  log (skein's `Read::Job`). The goal repairs it as it makes any change;
  that change names the red landing (`repairs`), and its own landing
  repairs it when a later run passes the failures.
- **A member's goal** is its change's requester's tracked ancestor, the
  goal its landings' news reaches (`forge.md`, 7.2): the news goes
  through that goal's subscription to the landings on the branch the
  landing went into, once per goal, as the verdict that reddens the
  landing is kept, and wakes it, whatever its overlap (`forge.md`,
  7.3). A goal with no such subscription left is told nothing.
- **The news's words:** `Red after landing: tier NAME run N blames
  landing COMMIT on BRANCH: K failing.`, the landing's commit whole,
  then each failing test blamed on it, one per line ("many" past
  `failing_tests`), then `Run: URL`, then each failed job's log's end,
  headed `End of job NAME's log:`. The failed jobs are those of the
  run's last part that failed; their logs' ends share the forge's
  `brief_bytes`, and a job whose log cannot be read is named with
  `[no log]`. The root cuts the whole to the deployment's
  `tasks.message_bytes`, the logs' ends first.
- **`repairs`** (`agent.md`, 4.3.1): the change names the red landing by
  its commit, checked by the root as it decides the delegate (a landing
  of temper's in the change's repository, on its landing branch, red in
  some tier), and kept with the change. When a later run of a tier
  passes the failures blamed on the red landing (5.3), the landing is
  cleared in that tier, repaired by the newest landing in that run's
  range whose change names it, or by the run alone when none does. The
  red landing's changes' results name the landing that repaired it
  (section 6): `Cleared by NAME run N, which passed the failures blamed
  on it; repaired by COMMIT.`, twelve digits of the repairing landing's
  commit.
- **A red of a tier following goal branches** (5.4) goes to the goal
  holding the branch, through its subscription to the landings on that
  branch, as news: `Red on BRANCH: tier NAME run N at COMMIT: K new, J
  still failing.`, then each failing test, one per line, prefixed `new:`
  or `still failing:`, then `Run: URL` and the failed jobs' logs' ends,
  as above. It wakes the goal when a test is new, and is kept for its
  next run otherwise. Nothing names it: the goal repairs it with any
  change into its branch, or amends the live change that owns the
  failing files (`../draft/reference-code-production.md`, section 10).
- **Others go on.** Landings keep landing, and their verdicts do not
  count the inherited failures. A project's setting may pause new turns
  (or queue merges) into that repository while one of its landings is
  red, instead.
- **A goal that has ended,** or a landing temper did not make: the red
  is kept as a red of no goal's, which findings file (`findings.md`,
  section 4), and the project's triager is told. The tier keeps them
  within its bound; past it, the oldest is dropped and counted on the
  tier's record, never holding the tier, so that no repository's
  verification stops for want of a triage.

## 8. What a landing tells

- **Its news carries the trailers** of each commit it brought to main:
  `Taught:`, `QA:`, `Fixes:` and `Repairs:`, read with the landing's
  commits (skein's `Read::Commits`), each line within `trailer_bytes`,
  at most `trailers` per landing.
- **A goal's wake policy may name trailers** (jig's `domain/tasks.md`,
  8.3, a news marker; temper numbers the forge's: 0 `Taught:`, 1 `QA:`,
  2 `Fixes:`, 3 `Repairs:`, 4 `Fixes-finding:`, the forge's table of
  what it offers naming them for ctl's and the host tools' words): the core keeps them with the goal's
  subscription to the landings of its subtree's repositories (jig's
  `domain/tasks.md`, 8.4; `forge.md`, 7.2), and the forge classes a
  landing carrying one as waking for that goal, and as kept otherwise,
  whatever its overlap. A designer's goal wakes on `Taught:` so, and
  reads the rest at its next run.
- **A read of landings since a time,** per project, with their
  verdicts and trailers, for a coordinator catching up: `landing list
  --project PROJECT --newer-than TIME`, from the landing log (section 10),
  with the time the engine saw each: temper's own merges, and each move
  of a landing branch the engine already watches, a tier's or one a
  goal's or a change's subscription follows (`forge.md`, 7.2), by queue
  or by anyone else. The log adds no watch of its own, since each costs
  a poll and a fresh read at every restart: a push to a main nothing
  follows is not logged. A coordinator catching up follows its
  repositories' landings anyway.

## 9. What the forge shows

- **A verdict's status** on the run's tip, `temper/verdict/<tier>`:
  success for green, failure for red with the blamed tests counted in
  its description, its link the run; pending while the tier is held on
  its faults (section 3), saying so.
- **A comment on each member's pull request,** once per tier verdict:
  "cleared by run N", or "red: tests X, Y" and, once one lands, "repaired
  by #M". Keyed, as every comment of temper's (`forge.md`, section 6).

## 10. Effects, reads and the store

- **A run is a task:** each tier run is a procedure task of the
  forge's, `tier`, made when the run starts, requested by the
  deployment on the project's behalf, funded from the project's spend,
  with the authority the project's policy gives tiers (kinds 6, 7, 9,
  16 and 17 on its repository), as a landing is (`landing.md`, 4.2). It
  owns the run's branch and its probes', its dispatch, reruns, status
  and comments, and ends done with the verdict as its result; its
  branches are released as it closes, by the release a change's branch
  takes (`forge.md`, 11), never by an effect of kind 15, so that no
  grant of it is needed. A fault past `tier_retries` holds it. The
  tier's record names its task; at most one per tier and repository
  lives, so they are bounded by the repositories and `tiers`.
- **The run task's words** start with `Tier NAME run N`, then name the
  repository, the full tested commit and the owned `temper/tier/NAME.N`
  branch at creation. Each part is identified separately: the judged
  run, a failed-test rerun and each bisection probe, with its commit and
  branch, and the forge run number and URL once observed. The procedure
  projects these facts through its own task's specification words in a
  due step (jig's `domain/tasks.md`, 5.3). New part words are saved
  before its effects are issued; the procedure continues at a deadline
  due immediately. The task keeps them after
  completion, branch release and restart. Both ctl `task show TASK` and
  `task show TASK --until ended` follow missing specification chunks within
  their output bounds, so `asked:` retains these identities beyond the
  overview prefix; when exhausted it
  gives a `task text` continuation at the first undisplayed byte.
  Observations not yet projected
  are kept on the tier row until a step acknowledges their task words,
  including across follower replacement and restart.
- **The completed run task's report** names its tier and judged run, and pairs
  each blamed merge commit with that landing's own failed IDs, distinguishing
  owned and suspected failures. Its summary counts inherited failures without
  attributing them to cleared landings. Singular counts say `1 test` and
  `1 landing`; overflow verdicts say `many tests`.
  The existing report contract's `words` bounds this result. Each attribution
  occupies a line, with a whole quoted ID (backslash, quote, CR and LF escaped)
  and, for a retained landing, `change show TASK`. Lines that do not fit are
  omitted whole; later shorter lines may still fit. An explicit remainder ends
  the report: `and N more failing tests; change show TASK`, naming the first
  omitted retained landing (`1 more failing test` for a single ID).
  If any omitted attribution is `many`, the
  remainder says `many` too. Full IDs remain in the change read's retained
  `Verification`; no separate record or larger task result is needed.
- **Effects:** the tiers' dispatches and reruns are `landing.md`, 11's
  kinds 16 and 17, their branches' creation kind 9, released at the
  task's close; statuses and comments are kinds 6 and 7; each its run's
  task's.
- **A purpose of its own,** tag 8 in `forge.md`'s table: a tier's step,
  the repository (16 bytes), the tier (2), the judged run's number in
  the tier's sequence (8), the operation (1: the branch, the dispatch,
  a rerun, a probe's branch, a probe's dispatch, the branch's deletion,
  the status, a comment), and the probe or the member it is for (2), 30
  bytes with the tag. The tier's record keeps its sequence and each
  operation's state, so a restart settles what was in flight.
- **Reads:** skein's `Read::Artifact { run, name, max_bytes }`, a run's
  artifact by name, unzipped (one file, stored or deflated), cut to
  `max_bytes`; and `Read::Commits { before, after, page }`, the commits
  of a range with their messages, oldest first.
- **In the store:** `Tier`, keyed by repository and tier, its run in
  flight, its last judged tip and failures, its flakes, the reds of no
  goal's until findings take them (section 7), with a count of those
  dropped past their bound, and a held run's last fault (section 3);
  for a tier following goal branches, each goal branch it follows,
  with its goal, its last run's tip, failures and start, and the branch
  its run in flight is for, within the forge's `holds`;
  `Verification`, keyed by repository and landing commit, for temper's
  own landings alone, made with the landing's `Landed` row and erased
  when its last change closes, so bounded by `changes`: the changes it
  landed, each tier's verdict (cleared, owns failures, or suspected of
  them), the failures blamed on it and the commit that repaired it; the
  `Change` family's landed state means verifying, and `Red` and
  `Cleared` join it; a change keeps the commit of the red landing it
  repairs (`repairs`). Through the baseline's rules (`landing.md`, section
  12).
- **The landing log,** a family of its own, `LandingLog`, per adopted
  repository: its last `landings_kept` landings, each with its commit,
  the engine's wall time when it saw the move, its changes (none for a
  push from outside temper), each with its pull request's number, so
  that a late verdict's comment reaches it, its trailers within `trailers` and
  `trailer_bytes`, and each tier's verdict (verifying, cleared, or red
  with its count); a ring, written at each move of a watched landing
  branch (section 8), at each merge of temper's and at each verdict, so that a landing's record outlives its changes'
  close and a range's judging.
- **Limits,** the forge's: `landings_kept` per repository (16 in
  `small`); `trailers` per landing (8) and `trailer_bytes` per line
  (200); `tiers` per repository (2 in `small`),
  `failing_tests` per run (16), `test_id_bytes` (256), and a tier's
  name at most 32 bytes. A main no tier or subscription follows has no
  log (section 8).

## 11. Settings

Beside kind 5's gates, each repository's tiers, each with the branch
it follows, or goal branches, and its span (section 2); with kind 4, the
project's `tier_retries` (2), `verification_bound` (two hours), several
suspects' rule (`bisect` or `tell all`), and whether a red pauses its
repository (no). Until the kinds are settings, the tiers,
`tier_retries` and `verification_bound` are in the document's
`landing` member, as the gates are, and the suspects' rule and a red's
pause wait for kind 4 (`landing.md`, 13.1).

## 12. The world

Stories in the forge's world, the fake forge running dispatched tiers
with failing tests by id and uploading their artifacts:

- **a green run** clears the two landings in its range, and each
  change gives its result;
- **a red run bisected:** three landings, one changing nothing the tier
  reads; two tests fail at the second's commit, which is red, and the
  others cleared;
- **inherited failures:** the next run fails the same two tests and
  clears the landing after the red one;
- **a flake:** one test fails, passes alone, and the range is cleared;
- **a fault:** a run lost when its runner restarts is retried, and the
  retry clears;
- **the bound:** a runner down past it ends a change landed and
  uncleared; the late red is kept for findings;
- **a repair** naming a red landing clears it, "repaired by";
- **a documentation landing** every tier ignores clears at once;
- **a landing temper did not make** is in a range like any other;
- **a branch no tier follows:** a change landing into a goal's own
  branch, in a repository whose tiers follow main, is cleared as it
  lands, and the move starts no run;
- **g's tier,** a tier following goal branches, its span 20 minutes:
  two changes landing into a goal's branch within the span are each
  cleared as they land, and one run starts at the newest tip once the
  span has passed; a red reruns, and what fails again reaches the goal
  holding the branch as news, its test marked new, waking it; the next
  run failing the same test sends it marked still failing, kept; a
  green says nothing; a goal's branch no change landed into is never
  run;
- **a red to its goal:** a red of main's tier blamed on a goal's
  landing reaches that goal as news, waking it, with the failing tests,
  the run and each failed job's log's end;
- **under queue and by turn,** the same stories.
- **The reference case** (`../draft/reference-case.md`): these mechanics
  open its steps 25 to 27, a red of main's tier triaged and repaired by
  a change that names it.

## 13. Open questions

- **The bound and the goal:** whether a goal may close at landed and
  uncleared on its own judgement before the bound.
- **Tiers on the forge's runners or on workers:** both are allowed by
  the draft; the forge's runners here, workers once a worker runs jobs.
- **Blame on a goal's branch:** whether a tier following goal branches
  should bisect a red to the merge that brought it and name that change
  in its news, rather than leave the goal to triage (5.4). Triage
  matched the evidence; blame would need the verdict machinery on a
  goal's branch (`../draft/reference-code-production.md`, section 17).

## 14. Later

- **A red after landing blamed within a goal's landing,** bisected over
  its changes' merges to the change that brought it. A goal's landing
  is one landing to its tiers, so its red names the goal's landing
  alone and the goal triages it (section 7;
  `../draft/reference-landing.md`, section 11).
