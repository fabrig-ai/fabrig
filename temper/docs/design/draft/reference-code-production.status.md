# reference-code-production: status

2026-11-12. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Open:**
  - **1 to 5, 12 to 14:** revised by [`goal-as-designer.md`](goal-as-designer.md), which wins where they differ until its plans fold it into this draft.
- **Design departs:**
  - **10, 13: a red of g's tier waking G.** `../domain/after-landing.md`,
    5.4 and 7 wake the goal on a red with a test the branch's previous
    run did not fail; a red whose tests all failed that run too is news
    kept for the goal's next run, not waking it, so that a red a live
    repair already owns does not wake G at xhigh at each span (N11).
    Decided 2026-11-03.
  - **5, 14: the foreman retired.** `../domain/forge/change/questions.md`,
    1 and `../protocol/configuration/reference.md`: the reference has no
    foreman and no charter 6, and each change, a coding change's and a
    designer's alike, takes questions `up`, where the draft has a foreman
    settle each coding change's question first, at gpt-6.1-sol and
    medium; so the stage-one check's golden has no foreman role
    (`../testing/checks.md`, 5). `stock-charters.md`, 3 retires it, the
    user's draft of 2026-11-12, since the implementer now checks its spec
    itself before it asks; `notes.md`, N1 keeps the reasons it had.

- **Design departs:** 5: `../protocol/configuration/reference.md` requires
  confirmed landing of every required design change into g before
  dispatching dependent coding, including the first batch and designer
  result paths. A pushed head, delivery or designer report does not
  confirm landing; a spec cites landed design by path and section,
  omitting its ended task from `after` and keeping other live
  prerequisites. The finding 459 repair requires this release order
  (the user, 2026-11-08); early producer scheduling remains
  `../domain/forge/change/questions.md`, 1's.
