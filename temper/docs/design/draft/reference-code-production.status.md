# reference-code-production: status

2026-11-08. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Open:**
  - **1 to 5, 12 to 14:** revised by [`goal-as-designer.md`](goal-as-designer.md), which wins where they differ until its plans fold it into this draft.
- **Design departs:**
  - **10, 13: a red of g's tier waking G.** `../domain/after-landing.md`,
    5.4 and 7 wake the goal on a red with a test the branch's previous
    run did not fail; a red whose tests all failed that run too is news
    kept for the goal's next run, not waking it, so that a red a live
    repair already owns does not wake G at xhigh at each span (N11).
    Decided 2026-11-03.
  - **5, 14: a foreman's LLM choice (N19).** `../domain/agent/tools.md`,
    1: `change`'s `llm` names the producer's and the reviews' roles
    alone, and a change's foremen run at their charter's default:
    `../protocol/configuration/reference.md` sets the reference's charter 6
    at gpt-6.1-sol, with no effort, where the draft has each change name
    its foremen's choice, gpt-6.1-sol at medium. Decided
    2026-11-03: the reference sets the foreman's
    model on its charter (`../domain/agent/README.md`, section 8).

- **Design departs:** 5: `../protocol/configuration/reference.md` requires
  confirmed landing of every required design change into g before
  dispatching dependent coding, including the first batch and designer
  result paths. A pushed head, delivery or designer report does not
  confirm landing; a spec cites landed design by path and section,
  omitting its ended task from `after` and keeping other live
  prerequisites. The finding 459 repair requires this release order
  (the user, 2026-11-08); early producer scheduling remains
  `../domain/forge/change/questions.md`, 1's.
