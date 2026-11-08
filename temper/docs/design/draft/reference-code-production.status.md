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
  - **5, 14: a foreman's LLM choice (N19).** `../domain/agent.md`,
    4.3.1: `change`'s `llm` names the producer's and the reviews' roles
    alone, and a change's foremen run at their charter's default:
    `../protocol/configuration.md`, 13.9 sets the reference's charter 6
    at gpt-6.1-sol, with no effort, where the draft has each change name
    its foremen's choice, gpt-6.1-sol at medium. Decided
    2026-11-03: the reference sets the foreman's
    model on its charter (`../domain/agent.md`, section 10).
  - **14: the project's checks.** `../protocol/configuration.md`, section
    11 and 13.9 have `.temper/pre-pr` run the format check and the counted
    baselines alone, on what a delivery adds over the checkout's `HEAD`,
    where 14's table has format, lints, the package's focused tests and
    the counted baselines: the coding agent's charter runs the changed
    packages' lints and focused tests, whether the script runs them too
    waits on the case's numbers (the user, 2026-11-02), and the whole
    workspace's checks ran past the agent's 60-second `check_timeout`
    (`archive/pre-pr-scoped.md`).
