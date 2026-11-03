# reference-code-production: status

2026-11-03. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Design departs:**
  - **10, 13: a red of g's tier waking G.** `../domain/after-landing.md`,
    5.4 and 7 wake the goal on a red with a test the branch's previous
    run did not fail; a red whose tests all failed that run too is news
    kept for the goal's next run, not waking it, so that a red a live
    repair already owns does not wake G at xhigh at each span (N11).
    Decided by reference-m9's cut, 2026-11-03.
  - **5, 14: a foreman's LLM choice (N19).** `../domain/agent.md`,
    4.3.1: `change`'s `llm` names the producer's and the reviews' roles
    alone, and a change's foremen run at their charter's default:
    `../protocol/configuration.md`, 13.9 sets the reference's charter 6
    at gpt-6.1-sol, with no effort, where the draft has each change name
    its foremen's choice, gpt-6.1-sol at medium. Decided by
    reference-m5's cut, 2026-11-03: the reference sets the foreman's
    model on its charter (`../domain/agent.md`, section 10).
