# bounded-waits: status

2026-11-24. Where the design lags or departs from [bounded-waits.md](bounded-waits.md).

- **Design lags:**
  - 3: what a task waits on is one computed value with no object for most kinds, no *since* and no bound (`jig/docs/design/domain/engine/views.md`, 3). temper's procedures name their waits for the test worlds alone (`../testing/engine-checks.md`, 2.2; `../domain/landing-waits.md`, 2). No design says how a wait's facts read in the previous build's records.
  - 4: no table of figures by kind. A landing's read waits read again at the forge's poll with no end (`../domain/landing-waits.md`, 3). Read admission, restart freshness, host places, accounts with no reset, questions, parked news and sign-off have no bound (`../domain/forge/change/questions.md`, 2). An entry at the last holder has no overdue state.
  - 5: circles are checked for dependency, delegate and resource edges alone, when they are made (`jig/docs/design/domain/tasks/holds.md`, 7). A goal with an unanswered question is woken once per newest question (`jig/docs/design/domain/tasks/messages.md`, 3).
  - 6: a held answer tells its sender nothing, and a party's words to a change are taken and dropped (`../domain/forge/change/questions.md`, 1). An ordinary task-made batch or watch may name only tasks it references, and subscriptions count a topic apart (`jig/docs/design/domain/tasks/messages.md`, 4 and 5). An escalation left undecided waits for a change that a riding-along result is not (`jig/docs/design/domain/tasks/proposals.md`).
  - 7: tables' designs don't all say what a full table does; a sign-in's exchanges stay taken by a parked act; a stored charter is not checked against its room at start.
  - 8: a placed hold's reason may be empty (`jig/docs/design/domain/tasks/lifecycle-holds.md`, 2). A person task gives one stall notice and stays (`jig/docs/design/domain/people/README.md`, 7).
  - 9: no read lists a project's waits (`../client/ctl/tools-reads.md`, 1). `milestone show` gives no goal's wait, and a landing's wait has no since or bound in its reads (`../domain/landing-waits.md`, 4).
  - 12: failure-worlds' wait check asks for a wake, not a record and a bound (`failure-worlds.md`, 3.1; `../testing/engine-checks.md`, 2.2).
- **Design departs:** none.
- **Open:** none. The stock figures are the user's of 2026-11-24 (`bounded-waits.md`, 4.1).
