# reference-code-production: status

2026-11-03. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Design lags:**
  - **3, 5, 14: `change` and its fields (N22, N13).** `../domain/agent.md`,
    4.3.1: `change` names no `questions`, LLM choice
    or producer's charter. `../domain/forge.md`, 8.1 and 8.9: a
    change's producer runs under the forge's change charter, and
    `../protocol/configuration.md`, 13.2 numbers the forge's charters for
    the change and its reviews alone.
  - **8, 14: questions (N1).** `../domain/agent.md`, 8.2 and
    `../domain/forge.md`, 8.1: a change takes no questions, and its agents
    have no Message family; there is no `questions` setting, foreman or
    question bound. `../protocol/configuration.md`, 13.2: no foreman's
    charter among the forge's.
  - **5, 14: LLM choices per role (N19).** `../domain/forge.md`, 8.1: a
    change's parameters name its producer's spec and charter, and no LLM
    choice for any of its agents. jig's `domain/tasks.md`, section 4
    checks a batch's LLM choice against the models configured and its
    authority, not its charter's models, which jig's
    `protocol/configuration.md`, 3.1 and `domain/engine.md`, 17.4 check
    for templates and moves.
  - **5: the plan's text (N12).** jig's `domain/core.md`, 7.1 keeps a plan's
    strategy in its transcript, its notes and a goal's projection, not in
    the repository's design.
  - **2, 5, 14: surveys from temper's runs (N10).** smith serves
    sub-agents on Claude Code (smith's `protocol/programs.md`, 5.10), but
    no temper run reaches them: `../protocol/setup.md`, 8.1 gives the
    box's agent `conversations` 1, so every `sub_agent` is refused as too
    many, and `../client/tools.md`, 5.5 has `chat start` and `goal start`
    take a main model and effort alone, naming no sub-agents.
  - **4, 14: the profile's limits (N21).** `../protocol/configuration.md`,
    13.4 and section 7, and `../domain/agent.md`, 4.3.1: the stock limits
    are a chat's (18 tasks 4 deep, 6 tasks under `change`, a result of 512
    bytes, which jig's settings schema caps); jig's `domain/engine.md`,
    17.9 allows 2 documents per charter in Small. None is sized for a goal
    of a hundred changes.
  - **5, 7, 8: a producer's next run once its dependencies move (N2).**
    `../domain/forge.md`, 8.11 merges the landing branch's tip into a
    producer's pushed branch or saved work only for a change that started
    early, and spends an early start on a conflict or a failure alone. A
    change takes no questions yet (8.1), so nothing says that an early
    producer that asks waits, once answered, for its dependencies to
    land (section 7), nor that a producer's next run after a revision of
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
