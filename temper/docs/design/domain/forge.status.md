# forge: status

2026-11-02. Where temper's code lags or departs from [forge.md](forge.md).

- **Not yet:**
  - 4.1: a removal without the box, its group's act answered in its
    decision: an owned or fake forge's removal asks the box link for
    `remove N` whether or not a pipe is there, and without one waits for
    an answer that never comes, so it is never whole and the
    playground's reset stops at its forge
    (`temper-engine-domain-forge/src/removal.rs`, `begin`, sets
    `box_removed` by the forge's mode alone;
    `temper-service/src/route/box_link.rs`, `remove`).
  - 4.1, 5.1, 16: a kept task's names: a task's release erases its
    `Names` row (`temper-engine-domain-forge/src/domain.rs`,
    `continue_release` calls `unnamed`), so a removed forge's
    repository's row goes once no live task, hold or subscription names
    it (`erase_removed_repositories`) and a chat that worked on one reads
    its name until the chat ends, not until it is erased; nothing gives
    the connector `Unnamed` when a task is erased, the root taking jig's
    `Ask::Unname` for a handoff it never gets
    (`temper-engine-domain/src/route/forge_route.rs`, `effect_ask`); a
    project's release lets go of no ended forge
    (`temper-engine-domain-forge/src/adoption.rs`, `release`); and a
    start accepts the `Names` row of a task jig does not hold live only
    when all its names lie on one ended forge
    (`temper-engine-domain-forge/src/reconcile.rs`).
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
  - 8.2: the pull request's body naming its goal's issue; a landed change
    living until its landing is cleared (`after-landing.md`, section 6).
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
    and the forge's gate templates (`root.md`'s status).
  - 8.8: the rule and criterion a change waits on, and its charter, on its
    panel.
  - 8.10: a change drained.
  - 11.1: a goal's own branch.
  - 12: the issue's milestones but its lifecycle and revisions: the plan
    accepted, a change landed or held, a report kept.
  - 12.1: questions on the forge.
  - 13: participating objects beyond their news; a landing waiting for
    someone else's merge where temper may not merge (also 4.4 and 9).
  - 14: upstream contributions.
