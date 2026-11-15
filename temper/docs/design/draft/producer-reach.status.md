# producer-reach: status

2026-11-15. Where the design lags or departs from [producer-reach.md](producer-reach.md).

The design carries the draft. Section 1 is the stock coordinator charter's line (`../../charters/stock/coordinator.md`, *A question a change passes up*), a charter rather than a design document; the path it names was designed already (`../domain/forge/change/questions.md`, 1, *Held for a dependency it did not start from*). Sections 2 and 3 are `../domain/forge/change/questions.md`, 1, *Its requester's words and amendments*, and `../domain/forge/change/README.md`, 1. A change amends its live producer as any procedure amends its delegate (jig's `domain/connectors.md`, 6.3), so no words stand in for the amendment.

- **Design lags:** none.
- **Design departs:**
  - 3: a change's record keeps at most 16 of its requester's words waiting to pass; words taken past them are answered `Not passed: …` and go no further, where the draft passes every word. A stored list needs a bound, and words wait there only while a step lacks message room or a refused step's hold waits for release. The goal's designer, 2026-11-15.
  - 3: words from a requester that is a party, not a task, stay taken and dropped as before, where the draft passes a requester's words. A procedure's step messages tasks alone (jig's `domain/tasks/messages.md`, 5), so the change could not tell a party they were passed, and every path that makes a change makes it a task's. The goal's designer, 2026-11-15.
- **Open:** none.
