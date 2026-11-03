# reference-code-production: status

2026-11-03. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Design lags:**
  - **3, 5, 14: `change` and its fields (N22, N13).** `../domain/agent.md`,
    4.3.1: `change` names no `questions`, LLM choice
    or producer's charter, nor `repairs`. `../domain/forge.md`, 8.1 and 8.9: a
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
  - **10, 14: g's tier (N5).** `../domain/after-landing.md`, sections 1 to
    3: a tier follows the one branch it names and judges the landings on
    it; none follows the goals' own branches as they come and go, judges
    no landing, runs at most once in a span, or sends its reds to the
    goal as news without a verdict.
  - **4, 14: the profile's limits (N21).** `../protocol/configuration.md`,
    13.4 and section 7, and `../domain/agent.md`, 4.3.1: the stock limits
    are a chat's (18 tasks 4 deep, 6 tasks under `change`, a result of 512
    bytes, which jig's settings schema caps); jig's `domain/engine.md`,
    17.9 allows 2 documents per charter in Small. None is sized for a goal
    of a hundred changes.
