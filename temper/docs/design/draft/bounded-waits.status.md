# bounded-waits: status

2026-11-26. Where the design lags or departs from [bounded-waits.md](bounded-waits.md), once its milestone's seven goals have landed (`bounded-waits.execution.md`, 1).

- **Design lags:**
  - 4.1: a procedure waiting on its repository's fresh reads after a restart has no bound while its forge stays down (`../domain/forge/catching-up.md`, 1 and 4).
  - 4.3: only a worker's suspension moves a wait's bound, through the task's deadline (jig's `domain/engine/waits.md`, 5); a forge outage stops a change's or a landing's stall and starts it afresh at its end (`../domain/forge/change/stalls.md`, 1; `../domain/landing-waits.md`, 3), and moves no other wait's bound by its gap.
  - 5.2: `news` counts from each park (jig's `domain/engine/waits.md`, 3), so a task that keeps waking on its timer and parking again starts a fresh wait each time, and the `news` bound that jig's `domain/tasks/stuck.md`, 7 relies on to end it need never come.
  - 6.1 and 9: the project's waits list a producer's question to its change, not the relay's blocker holding its answer: a change with a live producer reports no wait of its own (`../domain/forge/waits.md`, 4; jig's `domain/engine/waits.md`, 4), so only `change show` names the blocker (`../domain/forge/change/notices.md`, 2).
  - 7: no design lists every fixed table with what a full one does; jig's views' limits (jig's `domain/engine/README.md`, 7) and the engine checks' room (`../testing/engine-checks.md`, 2.3) say it for theirs.
  - 12: the forge's own world checks no wait record at its quiet points, and no one check asks every record past its bound for its outcome: jig's kinds' alarms and the forge's inspection each check their own (jig's `domain/testing-fakes.md`, 1.2; `../testing/engine-checks.md`, 3.1).
- **Design departs:**
  - 5.1: a held answer is no edge the stuck walk reads, since it is an application's procedure's own state, not one of jig's edges; a circle through one ends at its asker's `question` bound or its watcher's `news` bound (jig's `domain/tasks/stuck.md`, 7, and its open question). The stuck goal's design, 2026-11-25.
- **Open:** none. The stock figures are the user's of 2026-11-24 (`bounded-waits.md`, 4.1).
