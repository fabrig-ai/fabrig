# reference-code-production: status

2026-11-03. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Design lags:**
  - **3, 5, 14: `change` and its fields (N22, N13).** `../domain/agent.md`,
    4.3.1: `change` names no requester's branch, `questions` or merge
    job, nor `repairs`. `../protocol/configuration.md`, 13.2 numbers the
    forge's charters for the change and its reviews alone.
  - **8, 14: questions (N1).** `../domain/agent.md`, 8.2 and
    `../domain/forge.md`, 8.1: a change takes no questions, and its agents
    have no Message family; there is no `questions` setting, foreman or
    question bound. `../protocol/configuration.md`, 13.2: no foreman's
    charter among the forge's.
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
  - **4, 14: the profile's limits (N21).** `../protocol/configuration.md`,
    13.4 and section 7, and `../domain/agent.md`, 4.3.1: the stock limits
    are a chat's (18 tasks 4 deep, 6 tasks under `change`, a result of 512
    bytes, which jig's settings schema caps); jig's `domain/engine.md`,
    17.9 allows 2 documents per charter in Small. None is sized for a goal
    of a hundred changes.
