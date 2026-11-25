# stuck: status

2026-11-25. Where the tree's code lags or departs from [stuck.md](stuck.md).

- **Not yet:** 2 to 6: a parked task with an open question is woken once
  for each newer question while it holds a timer or a watch of a live
  task (`question_woken`), and polled otherwise; no walk judges its
  wakes, its hold's words name no chain, and nothing judges it again
  when what it waits on changes or at a start.
- **Not yet:** 5: a task past its run tries parked with nothing left to
  await ends failed, not held.
