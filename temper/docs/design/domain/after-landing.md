# After landing

Provisional, 2026-10-27. What follows a landing: the slower checks a
repository runs on its newest main (its tiers), the verdict each run
gives the landings it covers, and a change that lives until its landing
is cleared, so that a goal ends only when its work is verified, or,
where its project says so, ends at its landing while the tiers judge
that landing later (section 6). It holds
for every landing on a repository's landing branch, by queue
(`forge/change/README.md` and `forge/queues.md` and `forge/updates.md`, section 1), by turn (`landing.md`), or by anyone
outside temper. Findings, where a verdict files a flake or a red whose
goal has ended, are `findings.md`'s, and under `wait: landed` every red
of a landing tier is a red finding (`red-findings.md`); a project
without findings keeps them in the verdict's own record. What is still open is `after-landing.md`, section 13;
what waits, `after-landing.md`, section 14.

## 1. In one page

- **A tier** is a check too slow or too broad for a gate (a browser
  suite, an exhaustive profile), defined like a gate: a dispatch-only
  workflow, its ignored paths, its key (`after-landing-runs.md`, section 1). It also names where
  its run reports its failing tests, and the branch it follows: its
  repository's landing branch, main, unless it names another or the
  goals' own branches.
- **Tiers follow their branch, not the landings** (`after-landing-runs.md`, section 2). When the
  tip of the branch a tier follows moves, by temper or by anyone, the
  tier runs on it: one run per tier at a time, the next taking the
  newest tip, so one run covers every landing since the last. A tip
  whose key has a receipt is judged by it, with no run. A landing on a
  branch no tier follows, a goal's own branch while its repository's
  tiers follow main, is cleared as it lands.
- **A tier following goal branches** (g's tier, `after-landing-runs.md`, section 1) follows each
  live goal's own branch on its own, at most once per its span, and
  judges no landing: a change into a goal's branch is cleared as it
  lands. A red there is rerun once alone, and what still fails is news
  to the goal holding the branch (`after-landing.md`, section 7), never a verdict.
- **A person holds a tier** (`after-landing-runs.md`, section 2) with a reason: it starts no run
  until released, its landings still verifying, and the release runs
  the newest tip.
- **A run reports** pass or fail, and its failing tests by id, read from
  its artifact (`after-landing-runs.md`, section 3). A run that names no tests when it fails, or
  never ends, is a fault, never a red. Past `failing_tests` its failures
  are many, judged by landing alone, and still named: the lowest ids it
  read, and how many more.
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
- **Or ends at landed** (section 6), where its project's kind 4 says
  `wait: landed`: its result comes at its landing, so its goal, and
  every goal waiting on that goal, goes on then. The tiers judge the
  landing as any other, its members read from the landing log, which
  keeps it until every tier following its branch has judged it.
- **A red goes to the landing's goals** (section 7) as news, with the
  failing tests and the log's end, and the goal repairs it with a change
  that names it. Under `wait: landed` it is a red finding instead,
  blamed on its landing and kept until a run passes its tests
  (`red-findings.md`). Other landings go on.
- **Landing news carries the trailers** of the commits it brought
  (section 8), so a designer's wake policy can wake on `Taught:` alone.
- **The forge shows** each verdict as a commit status,
  `temper/verdict/<tier>`, and a comment on each member's pull request
  (section 9).

## 2. Tiers

Moved to [`after-landing-runs.md`](after-landing-runs.md), section 1.

## 3. Following the mains

Moved to [`after-landing-runs.md`](after-landing-runs.md), section 2.

## 4. What a run reports

Moved to [`after-landing-runs.md`](after-landing-runs.md), section 3.

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
   commits in the range, each probe the failing tests alone at a
   landing's commit (each was proved there, so the gates hold); the
   first landing whose commit fails a test owns it. The project's
   setting may say "tell all" instead: every landing in the range is a
   suspect, and each suspect's goal is told.
   - **The candidates** are the range's landings whose key differs
     from the one before it, the first's from the last judged run's: a
     landing whose commit changes nothing the tier reads (its key
     equals the one before it) is skipped, never probed and never
     blamed. The last candidate's key is the tip's, so it fails every
     new failure.
   - **A group** is some new failures with the landings between two of
     the range's, `low` and `high`, inclusive, between which their
     owner lies. The first holds every new failure, from the range's
     first landing to its last.
   - **A probe halves a group:** while a group holds more than one
     candidate, its failures alone are run at its middle candidate's
     commit, of its c candidates the one at position ⌊(c − 1) / 2⌋,
     oldest first from 0. Those that fail there make a group from
     `low` to that landing; those that pass, one from the landing after
     it to `high`. A group with one candidate is that landing's, with
     no probe: it owns those failures. A group with none (its failures
     passed at every candidate probed, failing only at the tip) is its
     `high` landing's.
   - **Lowest first, one at a time:** of the groups left, the one whose
     `low` is lowest is probed next, the others waiting on the run's
     record (`after-landing-storage.md`, section 1). Groups never overlap and a probed landing is
     in no later group's middle, so a run probes each landing at most
     once and its probe's branch, `temper/tier/<tier>.<n>.<k>`, k the
     landing's index in the range from 0, is never made twice (`after-landing-runs.md`, section 1
     ). One failure over n candidates takes at most ⌈log2 n⌉ probes: a
     range of eight, one breaking, takes three.
   - **After many** (`after-landing-runs.md`, section 3), a probe runs the whole tier and the
     group splits by the run alone: a probe that fails anything keeps
     `low` to the middle, one that passes takes the landing after it to
     `high`, and the group's one candidate owns many. A probe of chosen
     tests whose report is many fails them all there. A probe's fault
     is a fault (`after-landing-runs.md`, section 2), retried as any part is.
5. **A fault is not a red** (`after-landing-runs.md`, section 2).

Failures reported as many have no failed-test rerun; their probes run the
whole tier. Enumerated failure IDs are sorted and unique before judging. A
landing blamed for many is repaired by a later judged run whose failures can
be enumerated by test, even if that run fails other tests; a later many
verdict does not repair it (2026-11-04).

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
a fault is a fault (`after-landing-runs.md`, section 2).

On a goal branch, many after many is still failing and is kept for the
goal's next run. Many after an enumerated result is new and wakes it. When
the new result enumerates tests but the previous result was many, none of
those IDs is known to be still failing; each is new (2026-11-04).

## 6. A change lives to cleared, or ends at landed

```
… ─► landed ─► verifying ─► cleared
                   │           ▲
                   └─► red ────┘
```

- **Landed means verifying.** The task lives, with no run, until its
  landing is cleared in every tier following the branch it landed on
  (at once, where no tier follows it, or only tiers following goal
  branches do: `after-landing-runs.md`, sections 1 and 2); its branch's
  hold stays until then, and the branch is deleted as the task closes
  (`forge/branches.md`), by queue and by turn alike. A goal's landing, a
  change on its requester's held branch (`forge/change/README.md`, 1), holds none:
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
  (`after-landing-storage.md`, section 1), which keeps the landing's changes after their
  verification rows go: cleared, it is news to the goals whose
  subscriptions still name the change; red, it stays on the tier's
  record and is filed as a finding (`findings.md`, section 4), since
  the goal may have ended.
- **Cancelling a verifying change** ends it landed and uncleared at
  once; its verdict counts as after the bound.
- **`wait`,** kind 4's (`landing-settings.md`, 1.1): whether a
  project's change into a branch a tier follows lives to cleared, as
  above (`cleared`, the default), or ends at landed (`landed`). It is
  taken at the next start, as kind 4's other members are. A change into
  a branch no tier follows, or only a tier following goal branches
  does, is cleared as it lands under either.
- **Under `landed`, a change ends at landed.** Its landing makes no
  `Verification` row (`after-landing-storage.md`, section 1), so no tier's verdict is the change's
  and no bound applies to it: it is never red, and it never waits. Its
  result is given at its landing, in the step that sees its merge,
  naming the landing's commit as every landed change's result does, and
  adding the line `Ended at landing; after landing judges it later.`
  Its state stays `Landed`, as a change no tier follows ends. Its
  branch's hold is released and the branch deleted as it closes, by
  queue and by turn alike (`forge/branches.md`). Its requester hears
  the result as any delegate's, so a goal's closer's *Land* ends at the
  goal's landing and the goal with it (`forge/branches.md`, section 2),
  and a goal that depends on that goal, in a milestone or not, starts
  then (`milestones.md`, section 4; `forge/dependencies.md`). Holding
  them for verification would protect nothing: the change is on its
  branch already, under every goal started after it.
- **Its landing is judged as any landing is:** each tier following its
  branch takes it into a run's range by its commits (`after-landing-runs.md`, section 2), keys
  and bisects it (5.2) and blames it by commit, and posts the verdict's
  status and its comments on the members' pull requests (`after-landing.md`, section 9),
  the members read from the landing log, which keeps the landing until
  every tier following its branch has judged it, within
  `landings_pending` (`after-landing-storage.md`, section 1). A red one is a red finding
  (`red-findings.md`).
- **Switching** a project from `cleared` to `landed` takes effect at
  the start that takes kind 4's new value: each of its changes still
  verifying or red then ends landed and uncleared at its first step, as
  a cancel would end it, its result's line `Not cleared: its project
  now ends a change at its landing; after landing judges it later.` Its
  `Verification` row goes as its landing's last change closes, as
  ever; its landing stays in the landing log, verifying in each tier
  that has not judged it, so a later verdict is posted on its members'
  pull requests from the log, and a red one is a red finding. A red
  landing's row first files its finding, blamed on that landing
  (`red-findings.md`, section 8), and its failures stay on its tier's
  record, inherited by the tier's next runs (5.2) and held by that
  finding. Switching back to `cleared` changes no ended change: the
  changes landing after that start live to cleared.

## 7. Where a red goes

- **To each goal of the landing's members** as news: the tier, the
  failing tests blamed on it, the run, and the end of each failed job's
  log (skein's `Read::Job`). The goal repairs it as it makes any change;
  that change names the red landing (`repairs`), and its own landing
  repairs it when a later run passes the failures.
- **Under `wait: landed`** (section 6), no red of a tier following a
  named branch or the landing branch reaches a goal as news, whether
  the landing's goals have ended or still live: each is a red finding,
  filed against its tier's uncleared red findings, blamed on the
  landing that caused it and cleared by a run that passes its tests
  (`red-findings.md`), and its project's triager hears it as it hears
  any finding (`findings.md`, section 6). A tier following goal
  branches is unchanged (below).
- **A member's goal** is its change's requester's tracked ancestor, the
  goal its landings' news reaches (`forge/topics.md`, 1.2): the news goes
  through that goal's subscription to its own branch's landings when
  that branch is what landed, without a subscription to main. Other
  members use the goal's subscription to the branch the landing went
  into. It goes once per goal, as the verdict that reddens the landing
  is kept, and wakes it, whatever its overlap (`forge/topics.md`, 1.3). A goal
  with no such subscription left is told nothing.
- **The news's words:** `Red after landing: tier NAME run N blames
  landing COMMIT on BRANCH: K failing.`, the landing's commit whole,
  then each failing test blamed on it, one per line ("many" past
  `failing_tests`, then its names, below), then `Run: URL`, then each failed job's log's end,
  headed `End of job NAME's log:`. The failed jobs are those of the
  run's last part that failed; their logs' ends share the forge's
  `brief_bytes`, and a job whose log cannot be read is named with
  `[no log]`. The root cuts the whole to the deployment's
  `tasks.message_bytes`, the logs' ends first.
- **`repairs`** (`agent/tools-change.md`, 1): the change names the red landing by
  its commit, checked by the root as it decides the delegate (a landing
  of temper's in the change's repository, on its landing branch, red in
  some tier), and kept with the change. When a later run of a tier
  passes the failures blamed on the red landing (5.3), the landing is
  cleared in that tier, repaired by the newest landing in that run's
  range whose change names it, or by the run alone when none does. The
  red landing's changes' results name the landing that repaired it
  (`after-landing.md`, section 6): `Cleared by NAME run N, which passed the failures blamed
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
- **A goal that has ended,** or a landing temper did not make, under
  `wait: cleared`: the red
  is a red of no goal's, which the root files as a finding of its
  project in the decision that keeps the verdict, once per landing it
  reddens (`findings.md`, section 4), with no area, so that the project's
  triager holds it and hands it on (`areas.md`, section 5); a landing
  that brought a follow change has it filed with its follow's owner's
  area instead, where the owner owns one (`forge/follows.md`, section
  6), its members read from its `Verification` row, else from the
  landing log (`after-landing-storage.md`, section 1). Past the
  project's findings' bound it is not filed, never holding the tier, so
  that no repository's verification stops for want of a triage.

Red news fits `tasks.message_bytes` in order: header and test IDs as whole
lines, then the run URL, then failed-job log ends sharing the remaining room
equally. Only a log's first line may be partial, since cutting a log
preserves its end. Each failed-job read is bounded by `brief_bytes` divided
by the failed-job count, using at least one as divisor. A failed or empty
read yields `[no log]` (2026-11-04).

The commit in `Red on BRANCH: tier NAME run N at COMMIT` is the tested tip's
first twelve hexadecimal digits; an after-landing red continues to name its
blamed landing's full commit (2026-11-04).

**A many red's names** (`after-landing-runs.md`, section 3): where a red's news says `many` and its
run's report kept names, its `many` line reads `many; its report names:`,
then each kept id follows on a line of its own, then the remainder on
one line, each line with the prefix the `many` line has (`new: ` or
`still failing: ` on a goal's branch). The ids are whole lines, taken
while the remainder's line and the `Run:` line still fit after them;
those left out join the remainder's K. A red of no goal's, filed as a
finding, carries these words in its body, so a goal it is routed to, a
repair's among them, reads them there (2026-11-19). For example, a run
whose report names 37 failing tests:

```text
Red after landing: tier main run 1460 blames landing COMMIT on main: many failing.
many; its report names:
suite::a
(fifteen more ids, one per line)
and 21 more
Run: URL
```

## 8. What a landing tells

- **Its news carries the trailers** of each commit it brought to main:
  `Taught:`, `QA:`, `Fixes:` and `Repairs:`, read with the landing's
  commits (skein's `Read::Commits`), each line within `trailer_bytes`,
  at most `trailers` per landing.
- **A goal's wake policy may name trailers** (jig's `domain/tasks/messages.md`,
  3, a news marker; temper numbers the forge's: 0 `Taught:`, 1 `QA:`,
  2 `Fixes:`, 3 `Repairs:`, 4 `Fixes-finding:`, the forge's table of
  what it offers naming them for ctl's and the host tools' words): the core keeps them with the goal's
  subscription to the landings of its subtree's repositories (jig's
  `domain/tasks/messages.md`, 4; `forge/topics.md`, 1.2), and the forge classes a
  landing carrying one as waking for that goal, and as kept otherwise,
  whatever its overlap. A designer's goal wakes on `Taught:` so, and
  reads the rest at its next run.
- **A read of landings since a time,** per project, with their
  verdicts and trailers, for a coordinator catching up: `landing list
  --project PROJECT --newer-than TIME`, from the landing log (`after-landing-storage.md`, section 1),
  with the time the engine saw each: temper's own merges, and each move
  of a landing branch the engine already watches, a tier's or one a
  goal's or a change's subscription follows (`forge/topics.md`, 1.2), by queue
  or by anyone else. The log adds no watch of its own, since each costs
  a poll and a fresh read at every restart: a push to a main nothing
  follows is not logged. A coordinator catching up follows its
  repositories' landings anyway.

## 9. What the forge shows

- **A verdict's status** on the run's tip, `temper/verdict/<tier>`:
  success for green, failure for red with the blamed tests counted in
  its description, its link the run; pending while its run is held
  (`after-landing-runs.md`, section 2), saying why.
- **A comment on each member's pull request,** once per tier verdict:
  "cleared by run N", or "red: tests X, Y" and, once one lands, "repaired
  by #M". Keyed, as every comment of temper's (`forge/effects.md`, section 1).
- **Posted by the run's task, after its verdict is kept:** once the
  verdict is judged and kept (sections 5 to 7: the record, the
  landings' verification, the news), the run's task posts the status,
  then the comments, one write at a time, in order, and ends done. It
  lives while it posts, so the tier's next run starts once it has
  posted. A tip judged by its receipt posts as a dispatched run does.
- **The status's words,** its description cut to the forge's 140 bytes
  at a character's boundary:
  - green, the range cleared (5.1; 5.2, 1 with no failure left):
    `success`, `run N passed`, or `run N passed by receipt`;
  - red, failures left at the tip once rerun: `failure`, `run N
    failed: B blamed, I inherited`, B the distinct failing tests blamed
    on the range's landings, owned or suspected, I the run's other
    failures, inherited; B `many` where a landing's blame is many, and
    I `many` where the run's failures are many (`after-landing-runs.md`, section 3) and no
    landing's blame is;
  - pending, before a hold of its run: `after-landing-runs.md`, section 2's WHY, posted again
    at each hold.

  Its link is the judged run's whole part's forge run, never a
  rerun's or a probe's; a receipt's run for a tip judged by its
  receipt; for the pending status, the run part's last forge run, or
  none where none was found.
- **The comments,** on each pull request of each member of a landing in
  the run's range, oldest landing first, then on each member of a red
  landing this run repairs (5.3; section 7, `repairs`), in the order
  the repairs are found. A member is a change of temper's the landing
  brought, with its pull request, read from the landing log (`after-landing-storage.md`, section 1
  ), else from the landing's `Verification` row and the change's row;
  a landing temper did not make has none, nor has one the log no longer
  holds, past `landings_pending`, that no `Verification` row names: its
  verdict posts its status and no comment. A change with no pull
  request is skipped. A comment's first line, TIER the tier's name:
  - `Cleared by TIER run N.` for a landing the verdict blames nothing;
  - `Red: TIER run N blames this landing for X, Y.` for one that owns
    failures, `for many tests` where they are many, and `for many tests;
    its report names X, Y and K more.` where its run's report kept names
    (`after-landing-runs.md`, section 3);
  - `Suspected: TIER run N suspects this landing of X, Y.` under "tell
    all" (5.2, 4), `of many tests` and `of many tests; its report names
    X, Y and K more.` as above;
  - `Repaired by #M: TIER run N passed the failures blamed on this
    landing.`, M the pull request of the change in the run's range
    that names the red landing, the newest's (section 7), or
    `Repaired: TIER run N passed the failures blamed on this landing.`
    where none names it.

  Then, after a blank line, `Run: URL`, the status's link, where there
  is one. Test ids are written whole, in order, while the comment and
  its key fit the forge client's `op_bytes` (skein's
  `forge/client.md`), then `and K more`; for a many's names, the
  remainder is `after-landing-runs.md`, section 3's, K counting its more too.
- **Keys:** each write a keyed effect of the run's task (`after-landing-storage.md`, section 1):
  purpose tag 8, operation 6 for the status, its part 0 for the
  verdict's and, for a pending one, the count of the run's holds so
  far, 1 for the first (a release resets the faults, not
  this count, which the run's posting keeps until its verdict, so
  each hold posts its own); operation 7 for a comment, its
  part the comment's index in the order above, from 0, so that a
  restart posts each at most once (`forge/effects.md`, section 1).
- **What is posted is kept** on the tier's record from the verdict to
  its last post (`after-landing-storage.md`, section 1): what the status says, and each comment's
  pull request and what it says, so a restart posts the rest, with the
  same words under the same keys, whatever its changes have become
  meanwhile.
- **A refused write is left:** a status or a comment the forge refuses
  (a pull request locked or gone, the write refused) is skipped and the
  next posted; it never holds the tier or its task. An uncertain write
  is settled by the forge client, as any is.
- **Nothing is posted** for a run with no verdict: a run of a tier
  following goal branches (5.4), a run dropped (`after-landing-runs.md`, section 2), or a
  person's hold (`after-landing-runs.md`, section 2).
- **`read_ci` reads a logged landing's red** (2026-11-19): the host
  tool's `read_ci what=checks` at a commit its repository's landing log
  holds (`after-landing-storage.md`, section 1) answers the forge's checks, then one row for each
  tier whose entry there is red: its context `temper/verdict/<tier>`,
  failed, its description `TIER run N found this landing red: X, Y.`,
  the ids its entry keeps, owned or suspected alike, or `TIER run N
  found this landing red: many tests; its report names X, Y and K
  more.` for a many with names (`many tests.` with none); its ids
  written whole while the description fits the forge client's
  `op_bytes`, then `and K more` (`after-landing-runs.md`, section 3's remainder for a many); its
  link empty and no job, since a tier's job logs are its runner's. Any
  caller reads them, not a repair alone. They are kept
  before the forge's own rows within the client's `rows`, after a
  landing repair's gate rows (`forge/change/repairs.md`, *A landing's
  repair*), so that the forge's rows give way first, as they do to the
  gate's. Inherited failures are not the tip's to show: they are the red
  of the landing that first failed them, read at its commit while the
  log keeps it.
- **Not CI:** temper reads none of its `temper/verdict/<tier>`
  statuses back as a commit's CI unless the landing branch's rule
  requires one (`forge/keeping-up.md`, section 3), so a red verdict on
  main's tip holds no landing (section 7, *Others go on*).

## 10. Effects, reads and the store

Moved to [`after-landing-storage.md`](after-landing-storage.md), section 1.

## 11. Settings

Beside kind 5's gates, each repository's tiers, each with the branch
it follows, or goal branches, and its span (`after-landing-runs.md`, section 1); with kind 4, the
project's `wait` (`cleared`, `after-landing.md`, section 6), `tier_retries` (2), `verification_bound` (two hours), several
suspects' rule (`bisect` or `tell all`), and whether a red pauses its
repository (no). Until the kinds are settings, the tiers,
`tier_retries` and `verification_bound` are in the document's
`landing` member, as the gates are, and the suspects' rule and a red's
pause wait for kind 4 (`landing-settings.md`, 1.1).

A tier's settings start its runs, never its record. A tier newly
configured starts its baseline at its branch's tip read afresh
(`after-landing-runs.md`, section 2), so a workflow that landed before the tier's first start,
the configuration that names it already given, is found at the
baseline's commit. A tier removed from the settings starts no run;
its record stays (`after-landing-storage.md`, section 1) and restores whatever its last run's
task became (`after-landing-runs.md`, section 2), so a person never removes a tier to make the
engine start. A run of it still live ends as its tier being gone, and
a renamed tier is a removal and a new tier (`after-landing-runs.md`, section 2).

## 12. The world

Stories in the forge's world, the fake forge running dispatched tiers
with failing tests by id and uploading their artifacts:

- **a green run** clears the two landings in its range, and each
  change gives its result;
- **a red run bisected:** three landings, one changing nothing the tier
  reads; two tests fail at the second's commit, which is red, and the
  others cleared; a range of eight landings, one breaking, found in
  three probes; two failures owned by two landings, each red with its
  own; a landing whose key equals the one before it never probed;
- **a person's hold:** held with a run in flight, the run is judged and
  no run follows the moves after it; held past the bound, its changes
  end landed and uncleared; the release starts one run at the newest
  tip, its range every landing made while held; a restart while held
  restores it held, and starts nothing;
- **inherited failures:** the next run fails the same two tests and
  clears the landing after the red one;
- **a flake:** one test fails, passes alone, and the range is cleared;
- **a fault:** a run lost when its runner restarts is retried, and the
  retry clears;
- **a cancelled run:** a tier's run cancelled by a person is dropped as
  its task closes; the engine starts again, and the tier's next run
  starts, numbered after the dropped one;
- **a stale head:** a tier first configured while the head the engine
  knows of its branch is behind the forge's runs its baseline at the
  forge's tip;
- **a runner never comes:** a run whose one job no runner takes is
  held at the runner wait, its labels named, its status pending;
  released, it dispatches again and is judged; one waiting less runs on;
- **a missing workflow:** a dispatch refused 404 is a fault worded
  `workflow WORKFLOW not found at COMMIT`, and past `tier_retries` the
  hold says so;
- **the bound:** a runner down past it ends a change landed and
  uncleared; the late red is kept for findings;
- **a repair** naming a red landing clears it, "repaired by";
- **the forge shows it:** a green run's `success` status on its tip and
  a `Cleared by` comment on each member's pull request; a red run's
  `failure` status counting the blamed tests and a `Red:` comment on
  the red landing's members, `Cleared by` on the others'; then the
  repair's run, its status, and `Repaired by #M` on the red landing's
  members; a restart between two comments posts each once; a run held
  on its faults posts its pending status first;
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
- **a many red's names:** a run whose report names 37 failing tests,
  one landing in its range: its news and its finding name the lowest 16
  in byte order and `and 21 more`, as do its comment and its task's
  report; `read_ci what=checks` at the landing's commit answers a
  `temper/verdict/<tier>` row naming them; a report cut at the read's
  bound names what it read whole and says it was cut; a tip judged by
  its receipt still says `many`; a gate's report is read as before;
- **ending at landing,** under `wait: landed`: a change into main ends
  at landed with no `Verification` row, its result naming its landing's
  commit and saying after landing judges it later, its branch deleted
  as it closes; a goal's closer lands the goal's branch and the goal
  ends at that landing; a goal waiting on it in a milestone starts then;
  the tier's next run, red, blames that landing by commit, posts its
  `failure` status and a `Red:` comment on each member's pull request,
  the members read from the landing log, tells no goal, and files the
  red once as a finding of the project; a landing dropped from the log
  past `landings_pending` is judged with its status and no comment;
- **the log keeps what is pending:** landings past `landings_kept`
  stay while a tier has not judged them, each member with its goal and
  milestone, and each tier's verdict is kept on its entry; a log of the
  layout before restores its landings verifying where a tier still
  holds them;
- **switching:** a project switched to `landed` with a verifying change
  and a red one ends both landed and uncleared at the start, and a
  later run's verdict on the verifying one's landing comments on its
  members from the log;
- **under queue and by turn,** the same stories; under `wait: cleared`,
  every story above but those of ending at landing and switching, as
  before.
- **The reference case** (`../draft/archive/reference-case.md`): these mechanics
  open its steps 25 to 27, a red of main's tier triaged and repaired by
  a change that names it.

## 13. Open questions

- **Blame on a goal's branch:** whether a tier following goal branches
  should bisect a red to the merge that brought it and name that change
  in its news, rather than leave the goal to triage (5.4). Triage costs
  a wake and no machinery; blame would need the verdict machinery on a
  goal's branch (`../draft/reference-code-production.md`, section 17).

## 14. Later

- **A red after landing blamed within a goal's landing,** bisected over
  its changes' merges to the change that brought it. A goal's landing
  is one landing to its tiers, so its red names the goal's landing
  alone and the goal triages it (section 7;
  `../draft/archive/reference-landing.md`, section 11).
- **A many's names elsewhere:** a change's panel, the status writer's
  words and the forge's `temper/verdict/<tier>` status still say `many`
  with no names, a receipt keeps none (so a tip judged by its receipt
  says `many`), and `read_ci` at a tip whose run failed only inherited
  tests shows none of them (section 9). Each waits for a want; the
  repairer reads the names in its red's news or finding and at the red
  landing's commit (sections 7 and 9).
