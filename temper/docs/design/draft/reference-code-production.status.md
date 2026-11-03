# reference-code-production: status

2026-11-03. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Design lags:**
  - **3, 5, 14: `change` and its fields (N22, N13).** `../domain/agent.md`,
    4.3.1: `change` names no LLM choice,
    producer's charter or merge job, nor `repairs`. `../domain/forge.md`, 8.1 and 8.9: a
    change's producer runs under the forge's change charter, and
    `../protocol/configuration.md`, 13.2 numbers the forge's charters for
    the change, its reviews and its foremen, none of a change's own.
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
  - **6, 11, 14: the merge job (N7).** `../domain/landing.md`, 5.3: the
    project's tooling settles conflicts for the turn's prepare alone.
    `../domain/forge.md`, 8.4 and 10: what an update leaves goes to a
    resolution agent. `../domain/forge.md`, 8.2, step 6: a change into a
    branch whose rules ask no update, a goal's own, is held when its
    merge conflicts, and `../domain/forge.md`, section 20 leaves open
    what settles it. `../protocol/configuration.md`, section 13: no
    `merge_job` setting.
  - **10, 14: g's tier (N5).** `../domain/after-landing.md`, sections 1 to
    3: a tier follows the one branch it names and judges the landings on
    it; none follows the goals' own branches as they come and go, judges
    no landing, runs at most once in a span, or sends its reds to the
    goal as news without a verdict.
  - **11: catching up (N6).** `../domain/forge.md`, 10: a change never
    updates only because its base moved; 11.1: a goal's own branch takes
    the base's landings by an update. Neither has a catch-up change into
    a goal's branch.
  - **4, 14: the profile's limits (N21).** `../protocol/configuration.md`,
    13.4 and section 7, and `../domain/agent.md`, 4.3.1: the stock limits
    are a chat's (18 tasks 4 deep, 6 tasks under `change`, a result of 512
    bytes); jig's `domain/engine.md`,
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
