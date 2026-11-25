# lifecycle: status

2026-11-20. Where the tree's code lags or departs from [lifecycle.md](lifecycle.md).



- **Not yet:** 2: a standing task: no `NothingAwaited` notice, a finish parked, its `standing` ending (`standing.md`, section 2).
- **Not yet:** 2, *A question left open wakes it again*: a task with a timer or a watch of a live task is woken once for each newer question (`question_woken`) and then left idle, and one with neither is polled, whatever its wakes wait on. [in flight: change 17563]
- **Not yet:** 2, *Nothing left to await*, and 7: past its run tries the task ends failed, not held. [in flight: change 17564]
