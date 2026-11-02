# reference-landing: status

2026-11-02. Where the design lags or departs from [reference-landing.md](reference-landing.md).

- **Design lags:**
  - **1, 2, 3: L on its requester's held branch (N3).**
    `../domain/forge.md`, 8.1: a change's branch is its own or one handed
    to it, not its requester's held branch; 8.5: a move of it by another
    of its requester's delegates is drift, not its new head.
    `../domain/forge.md`, 11 and `../domain/after-landing.md`, section 6:
    a change's branch is deleted as its task closes, or, the change
    failed, kept until its tree's root closes; neither leaves it its
    goal's, kept after a goal that never landed it.
  - **3, 5: the merge job before a resolution (N7).** `../domain/forge.md`,
    8.4 and 10: an update refused for a conflict goes to a resolution
    agent, with no job before it. `../protocol/tooling.md`, sections 1 and
    4: the project's tooling has `prepare` and `key` alone, run for a
    turn, and no merge job.
  - **6: a repair names its red (N22).** `../domain/agent.md`, 4.3.1:
    `change` takes no `repairs`, which `../domain/after-landing.md`,
    section 7 gives a repair.
