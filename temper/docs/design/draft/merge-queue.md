# merge-queue: landing by turn as a merge queue

`turn` (`../domain/landing.md`) already proves a candidate, main's tip with a change on top, and fast-forwards main to exactly that commit. This draft makes it a merge queue: one landing per train at a time, a batch of the changes ready when it forms, stacked by merges, one proof at the top, a red narrowed by prefixes, and the proved green prefix landed. It is written for a project of one repository at twenty goals in flight, each landing proved by the project's full suites, and changes as little of `turn` as it can.

## 1. In one page

- **A train** is a repository and the branch its changes land into (section 3). Each has at most one landing at a time, from its prepare to its last turn. Other ready changes wait in a queue's order: the goal's priority, then the time each became ready, with aging (`../domain/forge/queues.md`). The branch moves only by that landing, so a proof never goes stale on another landing's turn.
- **A batch is the next landing** (section 4): the changes waiting on a free train, in that order, up to the project's `batch`, joined changes kept whole. A change ready later waits for the next one. A member leaves before its turn when it conflicts, its head moves, it is cancelled, refused or blamed; it keeps its place in the queue.
- **Stacked by merges** (section 5): a job on a worker merges each member onto the one before it, the first onto the branch's tip, each merge a commit on a branch the landing makes and holds. Member k's commit is the prefix: the tip and members 1 to k. No member's branch is rewritten.
- **One proof, at the top** (section 6): the project's gates dispatched at the last prefix, receipts by key as `turn` keeps them.
- **A red is narrowed by prefixes** (section 7), with the same gates, to a green prefix and the red one above it: the member that prefix added is repaired and leaves, the green prefix lands, and the members above it go on in the landing's next round.
- **Each round's turn** (section 8) fast-forwards the branch to its proved prefix by the landing's own pull request; every member in that prefix has landed.
- **Goals hear every step** (section 9); no landing waits without a hold that says why.

## 2. Where it starts

- **Built in `turn`:** a landing task per ready change; prepare by the forge's rebase onto main's tip (`../domain/landing.md`, 5.2); keys and receipts (6); gates dispatched at the candidate, runner faults retried (7.1, 7.4); the turn, a fast-forward of main (8); restart recovery (15); views.
- **Designed in `turn`, not built:** several members in one repository, prepared by a job (4.1, 5.3); a project's prepare and key commands.
- **What keeps `turn` from being a queue:**
  - **Proofs run side by side** (7.2) and the first proved lands first (9): at twenty ready changes, each proves at once, one lands, and the rest go stale and prove again, twenty, then nineteen, and so on.
  - **Independent changes never share a landing:** members join only by `with` or a dependency, and two live landings never merge (4.1).
  - **A red with several members in a repository** cannot say which member caused it (7.3).
  - **The forge's rebase replays a goal's branch** commit by commit: a branch that took its changes and main by merges meets again, at each landing, the conflicts its merges settled.
- **What the project needs** (2026-11-11, at ten to seventeen goals): a goal ready to land every eight to eleven minutes at twenty goals; a full proof of four to five minutes, one at a time on a shared host; one red in about seventeen landings, by a previous batching queue's record on this repository; a quarter of goals meeting a conflict with main.

## 3. Trains

- **A train is a repository and a destination branch.** Every member of a landing lands into the same one; everything below is per train. A repository's main is one train; each goal's branch, into which a goal's own changes land, is another.
- **The project's `batch`** applies to trains into a repository's main. A train into any other branch takes one change at a time, proved by the gates the project names for that branch (section 11).
- **A train is held** by its live landing from the decision that forms it to the end of its last round's turn, its probes included. Held is derived from the landings' phases, never stored, as the turn is (`../domain/landing.md`, 2).
- **Waiting changes** are ready (`../domain/landing.md`, 3) and in no landing. Their order is `../domain/forge/queues.md`'s: the goal's priority, then the time each first became ready, which a change keeps while it stays live, with aging past the deployment's queue window. Derived, not kept.
- **No race, no reservation:** `../domain/landing.md`'s 7.2 (side by side) and 9 (first proved, first landed, the reservation) go. A push to the branch from outside temper still makes a proof stale (8), and the landing prepares again, keeping its members.
- **One repository per landing:** a closure that joins changes in several repositories (`../domain/landing.md`, 4.1) is refused under this draft's rules, naming them, until section 13's batches across repositories.

## 4. The batch

### 4.1 Units

- **A unit** is what must land together: a change alone, or a closure of changes joined by `with` or by a dependency that joins (`../domain/landing.md`, 4.1), in its dependency order. A dependency that only orders results orders members, and a member whose dependency left leaves with it.
- **Units are indivisible:** they join, leave and land whole. A prefix that would cut a unit is never a landing boundary (sections 7 and 8).
- **A unit past `landing_members`** (section 10) is refused at forming, naming the bound: it can never fit.

### 4.2 Forming

- **Whenever a train is free and has waiting units,** a decision forms its next landing: the units in order, up to `batch` changes, the first unit always taken even past the cap. A unit that would pass the cap beside those before it waits, first in line. Forming is level-triggered: the same holds at a restart, and when a change becomes ready on an idle train.
- **Sealed:** no unit joins a formed landing. One that becomes ready meanwhile, or a dependency or `with` given to a member after forming, waits for the next landing (the member, with its unit, leaving if its unit is no longer whole).
- **`batch` 1** is one landing at a time: a stack of one unit.

### 4.3 Leaving

- **A member leaves its landing,** with its whole unit, before its round's turn, when:
  - its merge onto the stack conflicts (5.2);
  - its head moves: its goal delivered, or another hand pushed, which is drift as ever (`../domain/forge/change/README.md`, 3);
  - it is cancelled, or held, or a judgement gate it needs is refused (6);
  - it is blamed for a red (section 7).
- **It keeps its place:** its first ready time is unchanged, so once ready again it is first in line. Leaving is never a repair by itself.
- **The rest stay:** the round ends (7.3); the members below it keep their prefixes and receipts, those above it are merged again onto the last kept prefix (5.1), and the landing proves its new top.
- **A goal may deliver at any point,** as its charter says: its change leaves, and its new head waits for a later landing.

### 4.4 Its task and rounds

- **The landing task** is `../domain/landing.md`, 4.2's, made in the decision that forms the batch, with every member. Its members' change tasks wait on it as there.
- **A round** is one stack: its base (the branch's tip it was prepared on), its members in order with the heads they were merged at, and its prefix commits. A round ends at its turn, or when a member leaves or the base goes stale; the next round starts from the members still in the landing.
- **The train stays held** across rounds; the landing records each round's landed members and its result before the next round starts.
- **It ends** done once no member is left to land, its result naming each landed member's commit and each member that left, and why; held as `../domain/landing.md`, 4.2 says, its members' goals hearing it.

## 5. Stacking

### 5.1 The merge train

- **Every landing prepares by a job** (`../domain/landing.md`, 5.3; `../domain/worker.md`, 4.4), a stack of one included, temper's own when the project names no prepare command: from the round's base, for each member in order, a merge of the member's head, no fast-forward, the branch's line as the first parent, the member's title and change in the message.
- **Each merge is a prefix,** pushed by the worker to a branch of its own that the landing makes and holds, named per landing, round and member under the landing's branch name (`../domain/forge/branches.md`), each push expecting the branch new. The last is the round's candidate. The job writes under the landing's hold, as a pin-only member's does (`../domain/landing.md`, 4.1).
- **Its report** gives, per member, the prefix commit it pushed, or the conflict that made it skip the member (5.2); and each prefix's keys where the project's key command runs in it, otherwise the top reads the prefixes' trees and computes them (`../domain/landing.md`, 6.1).
- **Prepares again** only from the first prefix that changed, a new round's branches beside the old. The prefixes below it keep their commits, keys and receipts.
- **Lost or partial:** a job let go is found again by the branches it named and their expected trees, and what it didn't push is pushed afresh; a job's own failure is the project's, not a member's, as `../domain/landing.md`, 5.3 says.
- **Cleaned up:** a round's branches, and its pull request if it opened one, are closed and released when the round ends; the landing's, when it ends.

### 5.2 A conflict

- **A member whose merge conflicts** leaves with its unit (4.3), and the job goes on with the next unit on the same prefix.
- **The job says whether it conflicts with the branch alone:** its head merged onto the base, tried in the same job.
  - **With the branch:** a resolution under its change, as today (`../domain/forge/change/repairs.md`): the base merged into its branch, the files marked. Its pushed merge contains the base, and it waits for a later landing.
  - **With an earlier member only:** nothing to resolve yet. It waits, first in line; once that member has landed, its conflict is with the branch, and the next landing's job finds it so.
- **The project's `merge_job`** (`../domain/landing.md`, 13.1), where named, settles the conflicts its rules know inside the job before a member leaves; a member whose conflict it settled has no clean carry-over (6).

### 5.3 Why merges, not rebases

A goal's branch takes its changes and main's tip by merges, and its resolutions live in those merges. A rebase replays its commits one by one onto the new base, meeting those conflicts again, and rewrites a branch its goal may be delivering to. A merge of the branch's head meets only the conflicts its tree has with the base, and rewrites nothing; main's first-parent line gains one landing merge per member, and the goal's branch, child and catch-up merges included, stays reachable below it. Keys are trees (`../domain/landing.md`, 6.1), so the merge's commits cost no receipt. `../domain/landing.md`, 5.2's forge rebase is no longer used by a project that lands this way.

## 6. Proof

- **At the top only:** the train's gates dispatched at the round's candidate, on its prefix's branch, as `../domain/landing.md`, 7.1 dispatches at a member's candidate, with the round's base as an input (section 11). A key with a receipt is not run again.
- **A key** is `../domain/landing.md`, 6.1's, with the base's tree added for a gate that compares with the base (section 11), so a verdict never depends on what isn't in its key.
- **Judgement gates,** as `../domain/landing.md`, 3 and `../domain/forge/change/gates.md`, 1 and 2 say, each member's candidate being its prefix: a `clean` verdict given at the member's head carries over a merge that settled no conflict; an `exact` verdict, or any after a settled conflict, is asked again at the member's prefix, on the landing's pull request for its round, and the landing waits for it there rather than the member leaving; a refusal makes the member leave. A rebuilt prefix asks again.
- **The forge's own rules on the branch** (required approvals, required statuses) judge the landing's pull request: adoption checks that temper can meet them with each member's evidence and the gates' statuses on the prefix (`../domain/landing.md`, 13.2).
- **A runner's fault** is retried, never repaired, within `gate_retries` (`../domain/landing.md`, 7.4).
- **A real failure** is a red (section 7).
- **One gate run at a time** per train follows from section 3: a landing proves, then probes, one run after another.

## 7. A red

### 7.1 A round of one unit

Its members' changes are repaired, given the failed run's output (`../domain/landing.md`, 7.3), and leave. The landing ends with nothing landed from that round.

### 7.2 Narrowing by prefixes

- **The search:** with the top's run failed, the landing dispatches the same gates at the middle unit boundary; a failure searches below it, a pass above, until it holds a green prefix and the red prefix one unit above it. The base counts as green: it is the branch's tip, which landed proved. Each prefix is a tree with its own key, so a receipt found earlier holds. At most `ceil(log2(n))` runs beyond the top's, n the round's units.
- **What it promises:** an observed boundary, not the lowest red or the longest green prefix: a later member can fix what an earlier one broke. The blame is the unit at that boundary, in that context: the failure appears with it, on top of a prefix that passed.
- **The blamed unit's** changes are repaired, given the red prefix's failed run, and leave (4.3).
- **The green prefix lands** in this round's turn (section 8), when it is not the base. The units above the red one go on in the landing's next round, stacked on the new tip, first in line.
- **What a run's failure means:** the project's gate workflow runs a failed test again alone in the same run, then at the round's base, and fails only what fails at its commit and passes at the base (section 11). A failure with no test to run again (a build, the docs check) is the commit's. A failure that reaches temper is the commit's.
- **Until the search is built** (the execution's M3), a red round of several units ends; its units leave, keep their places, and the train's landings take them one unit at a time, by a fallback the landing records (10), until each has landed or been repaired; then batching resumes.

### 7.3 A search's identity

- **A search belongs to its round:** its base, its members and their heads, its prefix commits. A member leaving, a head moving or a stale base ends the round, and the search with it.
- **Late outcomes** make receipts for their exact keys, which a later round may reuse, but never advance or blame in another round.
- **The turn** re-reads the base and each member's head first (section 8): anything moved since the round was made makes the round stale, and the landing prepares again from the first changed prefix.

### 7.4 What the blamed unit's goal hears

The red, the prefix it was found at and the green one below it, the members that landed, the failed run's output and how to read it, as a repair's brief carries a gate's failure (`../domain/forge/change/repairs.md`).

## 8. The turn

- **First, freshness:** the branch's tip and each member's head in the prefix are read afresh; one that moved makes the round stale (7.3), and the turn goes.
- **Then the branch fast-forwards to the proved prefix,** by the merge of the round's own pull request from that prefix's branch into it, fast-forward only (`../domain/landing.md`, 8, on Forgejo), judged by the landing rules at that commit (6).
- **Each member in the prefix lands:** the branch contains its head. Its change's landed commit is its prefix, the merge that brought it, and the change is kept for its goal as any landed change is (`../domain/forge/change/README.md`, 7). Its pull request reads merged, by the forge's detection of a manual merge, which adoption checks (`../domain/landing.md`, 13.2).
- **Recorded before the next round:** the landed members and the round's result; a merge found done at a restart, with its record missing, is recorded, never merged again.
- **No empty turn:** a round whose green prefix is its base lands nothing and goes straight to the next round.
- **After the last round,** the landing's branches are released and the train frees (section 3).

## 9. What goals and people see

- **News at every step,** to each member's goal: joined (with its peers), proving, probing, landed (the commit), left (why: a conflict with the branch or with a peer, a moved head, a refusal, a hold, a red, with what follows).
- **No silent wait:** a step that makes no progress past its bound holds the landing, naming the step and what it waits on. A refusal names its bound.
- **The changes view** (`../domain/forge/queues.md`, shown whole): per train, the landing in flight, its round, members, step and probes, then the waiting changes in order.
- **The round's pull request** shows its stack, each prefix's gate statuses, `temper/gate/<name>` (`../domain/landing.md`, 10), and the search.

## 10. Settings, store and bounds

- **`batch`:** a member of kind 4, a project's landing (`../domain/landing.md`, 13.1): the most changes a landing into a repository's main forms with; stock 1.
- **The store:** a `Landing` keeps its rounds (base, members and heads, prefix commits, branches), each member's outcome and why it left, the search's bounds, and the train's fallback (7.2) while it lasts. In place, before the first release (`../protocol/store.md`, section 2).
- **Bounds** (`../protocol/configuration/limits.md`): `landing_members` bounds a landing's changes, at least `batch`; a unit past it is refused (4.1). `landings` counts live landings, one per train. Ended changes stay while `../domain/forge/change/README.md`, 7 keeps them, and a landing's row goes only once each member's outcome is on its change: bounds cover live and retained work, never all of history.
- **Steps' bounds:** the prepare job's timeout, a run's, a probe's; past one, a hold (section 9).

## 11. The project's own setup

What the project configures, outside temper's code:

- **`by` `turn`, `batch` 1, then 4** once batches are built.
- **One blocking test gate for main:** a workflow dispatched only, given the candidate and the round's base, running the build and the full focused and fuzzy suites, one process per test binary, and the docs check. It runs a failed test again alone, then at the base, and fails only what fails at the candidate and passes at the base. No ignored paths. Clippy and the format check run elsewhere: each delivery's own check, and clippy after landing (the user's word, 2026-11-11).
- **No gate for a goal's branch:** each delivery's own check stands for it, as today.
- **No pull request CI into main:** the gate is the landing's (`../domain/landing.md`, 7.1).
- **After landing:** main's focused and fuzzy tier goes, since every commit main reaches was proved; clippy and slower tiers run there (`../domain/after-landing.md`, 2).
- **The forge:** fast-forward merges allowed, detection of a manual merge on, and main's rules met by the landing's pull request.
- **The runner's host** runs one gate at a time, ahead of builds.

## 12. Acceptance

Focused and fuzzy suites alone, in the forge world and the engine's domain stories, each failing first:

- trains: one landing per train, the rest waiting in order, aging included; formation when a train is idle and at a restart; a goal's branch one at a time; a closure across repositories refused;
- forming: the cap, the first unit past it, units whole, sealed against a later change or dependency;
- the merge train's job: prefixes on new branches, the report, a lost job found again, preparing again from the first changed prefix, branches cleaned up;
- leaving: a conflict with the branch (a resolution), with a peer (waits, first in line), a moved head, a refusal, a cancel; the unit leaving whole; the rest proving again;
- judgement gates: a `clean` verdict carried, an `exact` one asked at the prefix;
- a red: a round of one repaired; the search's boundary, the green prefix landed, the units above it in the next round, no empty turn, at most `ceil(log2(n))` probes; a late outcome ignored by the next round; the fallback before the search;
- a runner's fault retried at a probe;
- the turn: freshness, the branch at the prefix, members landed, pull requests merged, a done merge recorded once at a restart; stale from outside;
- a restart in each phase found again and carried on;
- news, holds past a bound, and the changes view.

After landing, on a staging deployment before the project's own: goals land through it at the project's volume.

## 13. Later

- **Probes by test:** a probe that builds and runs only the tests the top failed, in the packages that hold them, in a fraction of the full gate's time.
- **The next batch proved on top of the one proving,** to overlap proofs when one at a time runs short.
- **A project prepare command** that regenerates derived files a merge conflicts on, such as generated totals, so those conflicts never reach a goal.
- **Batches across repositories,** with pins and upstream-first turns (`../domain/landing.md`, 5.3 and 8), and GitHub's turn.
