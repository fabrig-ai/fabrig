# forge: status

2026-11-03. Where temper's code lags or departs from [forge.md](forge.md).

- **Not yet:**
  - 11, 11.1: a goal's own branch deleted as its goal closes once a change
    has landed it: the release keeps it whatever way the goal ended
    (`temper-engine-domain-forge/src/goal_branch.rs`).
  - 4.1: a removal without the box, its group's act answered in its
    decision: an owned or fake forge's removal asks the box link for
    `remove N` whether or not a pipe is there, and without one waits for
    an answer that never comes, so it is never whole and the
    playground's reset stops at its forge
    (`temper-engine-domain-forge/src/removal.rs`, `begin`, sets
    `box_removed` by the forge's mode alone;
    `temper-service/src/route/box_link.rs`, `remove`).
  - 4.4, 8.3: on GitHub, statuses and dispatches by temper's App for a
    project that lands by turn, and verdicts posted where an installation
    grants `statuses: write`; `status` is false there.
  - 4.4, 9.1, 18: the forge's own merge queue beyond a queued merge kept
    pending: `Pull.queued` and `dequeued` are not read, a queued change
    keeps temper's turn, an ejection makes no repair, the panel shows no
    place in it, and adoption's words leave out the queue, signatures,
    linear history, code owners and the App's bypass
    (`temper-service/src/family.rs`).
  - 7.1, 12: the findings topic and findings (`findings.md`).
  - 7.3: a hold parked until a landing.
  - 8.1: a change's own bounds, gates and charters: `change` takes
    parameters 1 to 6, 9 and 10, its bounds are the deployment's
    `change_policy` and its charters the deployment's
    (`temper-engine-domain/src/route/forge_route.rs`, `start_change`); its
    page's line that a change takes no words.
  - 8.1, 8.2 step 1, 10: a catch-up, the merge job its producer
    (parameter 16): the change procedure has no producer but an agent.
  - 8.2 step 6, 8.4, 10: the merge job before a resolution: a conflict
    goes to a resolution at once under rules asking an update, and holds
    the change under rules asking none
    (`temper-engine-domain-forge-change/src/domain.rs`, `first`).
  - 8.2: the pull request's body naming its goal's issue.
  - 8.3: a person's approval as a gate; a review's lens; gates run at once
    when the change asks; a verdict posted as a commit status; the review
    charter, kind 3's number 2, is fixed at charter 4.
  - 8.5: a retargeted pull request retargeted back on release; the change
    is held as a failed effect.
  - 8.7: the explicit choice to adopt a GitHub repository without CI;
    requirements 5 and 6.
  - 8.7 to 8.9, 16, 18: the landing rules and the forge's charter as
    settings of temper's kinds; the rules are read from the document's
    `landing` member at start and compiled into the core's requirements
    and the forge's gate templates (`root.md`'s status). The forge judges
    a rule of kind 1 or 2 and answers `requires` over rules held so
    (`temper-engine-domain-forge/src/settings.rs`), but nothing holds,
    stores or routes one: the procedure's `Domain::requires` reads the
    compiled rules (`gates.rs`).
  - 8.8: the rule and criterion a change waits on, and its charter, on its
    panel.
  - 8.10: a change drained.
  - 9: a branch a tier follows still pauses its queue on its own CI's
    red and makes a repair, where its tiers judge it instead
    (`temper-engine-domain-forge-change/src/domain.rs`, `base_ci`).
  - 12: the issue's milestones but its lifecycle and revisions: the plan
    accepted, a change landed or held, a report kept.
  - 12.1: questions on the forge.
  - 13: participating objects beyond their news; a landing waiting for
    someone else's merge where temper may not merge (also 4.4 and 9).
  - 14: upstream contributions.
