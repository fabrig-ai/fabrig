# merge-job-runs-2: a change's merge job runs on a remote worker

On a real deployment at fabrig `69cff99edb` and `a66fcfb9f7` (one engine, one remote worker on another host, a Forgejo forge, the project's `merge_job` set to `.temper/merge`), every merge job a change made (job kind 3, `temper/docs/design/domain/worker.md`, 4.4) ended held: `out of tries: transient failures`. Neither the engine's log nor the worker's logged a line, and the tasks' activity showed one run per attempt, ended, with nothing more. Five jobs have failed this way so far, for catch-ups of main into a goal's branch and for merges of another branch's head. Releasing them changed nothing. Goals catch up only through a producer's own merge meanwhile, and `deliver` publishes that as a single-parent commit. Evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The failing story first,** at the lowest tier that holds the engine, a remote worker and the merge job: a change on a project with a `merge_job` makes the job, the worker takes it, prepares the checkout (the head and the base merged in), runs `.temper/merge`, and delivers or not. Rows:
   - a clean merge;
   - a merge with a conflict the command settles;
   - one it leaves;
   - a base that is a commit only on a branch the worker's checkout doesn't fetch by default.

   Find where it fails. Candidates:
   - the worker doesn't declare kind 3 in its hello, so the engine's claim is refused and the attempt counted transient;
   - the checkout's preparation can't fetch the base;
   - the command's path, the repository's name and then the `merge_job` path, is wrong;
   - the job's spec or result passes a bound.
2. **The fix,** with each failure logging a line on the side that failed, naming the job, its kind and why. A job that fails for good holds as the design says, with its reason in the hold.
3. **The design says so:** `worker.md` 4.4 and `forge.md` 8.4.

## Not in this plan

Any figure.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test; no test reaches a real host. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
