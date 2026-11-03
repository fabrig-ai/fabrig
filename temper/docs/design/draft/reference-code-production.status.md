# reference-code-production: status

2026-11-03. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Design lags:**
  - **3, 5, 14: `change` and its fields (N22, N13).** `../domain/agent.md`,
    4.3.1: `change` names no general producer job.
  - **4, 14: the profile's limits (N21).** `../protocol/configuration.md`,
    13.4 and section 7, and `../domain/agent.md`, 4.3.1: the stock limits
    are a chat's (18 tasks 4 deep, 6 tasks under `change`, a result of 512
    bytes, which jig's settings schema caps); jig's `domain/engine.md`,
    17.9 allows 2 documents per charter in Small. None is sized for a goal
    of a hundred changes.
  - **5, 7, 8: a producer's next run once its dependencies move (N2).**
    `../domain/forge.md`, 8.11 merges the landing branch's tip into a
    producer's pushed branch or saved work only for a change that started
    early, and spends an early start on a conflict or a failure alone.
    Nothing says that an early producer that asks waits, once answered,
    for its dependencies to land (section 7), nor that a producer's next
    run after a revision of
    the design or an answer that added a dependency merges g's tip first
    (sections 5 and 8).
  - **12: the plan done with g's tier settled (N5).**
    `../protocol/configuration.md`, 13.9: charter 5 lands the goal once
    every change of the plan has landed and nothing is left, whatever
    g's tier's last run said; the draft also asks that run green, or its
    red owned by a live repair.
- **Design departs:**
  - **10, 13: a red of g's tier waking G.** `../domain/after-landing.md`,
    5.4 and 7 wake the goal on a red with a test the branch's previous
    run did not fail; a red whose tests all failed that run too is news
    kept for the goal's next run, not waking it, so that a red a live
    repair already owns does not wake G at xhigh at each span (N11).
    Decided by reference-m9's cut, 2026-11-03.
