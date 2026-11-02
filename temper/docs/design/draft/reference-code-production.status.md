# reference-code-production: status

2026-11-02. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Design lags:**
  - **1, 3, 4, 12, 14: the goal's own branch (N3).** temper's
    `../domain/forge.md`, 11.1 makes a goal's own branch "a change of its
    own", and 11 has that change create it at its base's tip as an effect
    and delete it as the change closes; section 2 has every branch under
    temper's prefix held by a change; 11 offers a template no symbolic
    *its own goal branch*. jig's `protocol/configuration.md`, 3.1: a
    template names no resources. jig's `domain/tasks.md`, 6.1 and
    `domain/connectors.md`, 3.3: a resource a landing queue orders is not
    held. jig names an effect of a hold's release, not of its taking
    (`domain/connectors.md`, 3.3 and 4.6; `domain/tasks.md`, 6.5). jig's
    `domain/tasks.md`, 6.3 reads as if every run under a hold takes its
    writer slot, where `domain/engine.md`, 7.1 takes the slots of what the
    run writes.
  - **3, 5, 14: `change` and its fields (N22, N13).** `../domain/agent.md`,
    4.3.1: `change` takes the words and a repository, makes one change per
    call and lands it into the repository's default branch; it names no
    landing branch, requester's branch, dependency, batch, `questions`,
    LLM choice, producer's charter or merge job, nor `repairs`, and no
    charter's authority governs them. `../domain/forge.md`, 8.1 and 8.9: a
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
  - **6, 14: the light rules (N4).** `../domain/forge.md`, 8.2 (steps 3 and
    6), 8.7 and 10: every change is checked by CI at its head and updated
    when first if it lacks its base's tip, whatever its landing branch's
    rule. A merge refused for a conflict sends the change back to first
    and its update (`../domain/forge.md`, section 6 and 8.2), which leaves a change under a rule
    that asks no update without a path to the merge job.
  - **6: a change's result (N15).** `../domain/forge.md`, 8.2, step 7: a
    change's result words are its landing's facts, without its producer's
    report.
  - **6, 11, 14: the merge job (N7).** `../domain/landing.md`, 5.3: the
    project's tooling settles conflicts for the turn's prepare alone.
    `../domain/forge.md`, 8.4 and 10: what an update leaves goes to a
    resolution agent. `../protocol/configuration.md`, section 13: no
    `merge_job` setting.
  - **7: early starts (N2).** `../domain/worker.md`, 4.1 and section 5: no
    checkout start merges dependencies' pushed heads onto the landing
    branch's tip. `../domain/forge.md`, section 8 says nothing of a
    producer's early start, its once rule, its refusal on a conflict, or
    an amendment refused for dropping a dependency a producer started
    from.
  - **10, 14: g's tier (N5).** `../domain/after-landing.md`, sections 1 to
    3: a tier follows the one branch it names and judges the landings on
    it; none follows the goals' own branches as they come and go, judges
    no landing, runs at most once in a span, or sends its reds to the
    goal as news without a verdict.
  - **11: catching up (N6).** `../domain/forge.md`, 10: a change never
    updates only because its base moved; 11.1: a goal's own branch takes
    the base's landings by an update. Neither has a catch-up change into
    a goal's branch.
  - **11, 13, 14: the coordinator's wake policy (N23).** jig's
    `domain/tasks.md`, 8.3 gives a coordinator a default that wakes on the
    last delegate's result rather than on each, and on waking news at
    once; its closed vocabulary batches by count or by age, a delay, and
    has no "at most once in 30 minutes". jig's
    `protocol/configuration.md`, 3.1 and section 3: a charter in
    configuration carries no default wake policy.
  - **4, 14: the profile's limits (N21).** `../protocol/configuration.md`,
    13.4 and section 7, and `../domain/agent.md`, 4.3.1: the stock limits
    are a chat's (18 tasks 4 deep, 6 tasks under `change`, a result of 512
    bytes, which jig's settings schema caps); jig's `domain/engine.md`,
    17.9 allows 2 documents per charter in Small. None is sized for a goal
    of a hundred changes.
