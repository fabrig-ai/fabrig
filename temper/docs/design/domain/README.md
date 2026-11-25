# temper's domain

Provisional, 2026-10-04, revised 2026-10-20. The design of temper's domain
layer: its root, which composes jig's core with the forge connector; the
forge connector; the worker; and what temper fills in of smith's agent;
and how temper is tested. temper as a whole is `../README.md`; the domain
temper composes is jig's, whose design is jig's `domain/README.md`. The
mechanics are those of skein's `programming-model.md`. What is open across
the documents is listed in section 6.

## 1. Reading order

jig's domain comes first: its `domain/README.md`, then its documents in
the order it gives, jig's `domain/model.md` and `jig/docs/design/domain/engine/README.md` above
all. Then temper's:

1. **root.md:** temper's root: what it composes, its vocabulary, the one
   shape as temper holds it, routing, translation, putting outputs
   together, its store vocabulary, restart and stopping, its limits, and
   what it never does.
   - `root-translation.md`: the root's boundary translation and output assembly.
   - `root-restart.md`: restarting and stopping the root.
2. **forge/README.md:** the forge connector: adopting repositories, keeping up,
   effects, topics and landings as news, the change procedure and its
   requirements, landing queues, updates, branches, issues, and the ways
   temper takes part in what others own.
   - `forge/effects.md`: effects and recovery across restarts.
   - `forge/topics.md`: topics and landings as news.
   - `forge/updates.md`: lazy, mechanical updates.
   - `forge/adoption.md`: adopting repositories, rules, workflows and landing branches.
   - `forge/adoption-connection.md`: a forge's connection, access state and removal.
   - `forge/adoption-person.md`: a box's person on its forges.
   - `forge/change/README.md`: the change procedure and its documents.
   - `forge/change/drift.md`: a change's drift and its own writes.
   - `forge/change/panel.md`: a change's panel.
   - `forge/change/records.md`: a change's records and room.
   - `forge/change/repairs.md`: repairs and their outcomes.
   - `forge/change/repairs-resolutions.md`: merge jobs and conflict resolutions.
   - `forge/change/repairs-briefs.md`: gathering the forge sections of a brief.
   - `forge/change/repairs-releases.md`: holds and releases.
3. **worker/README.md:** the worker: its root on jig's hub and smith's host
   domain, the items an assignment gives a run's workspace, and the
   checkout.
   - `worker/hosted-runs.md`: assignments, lifecycle and failures.
   - `worker/agent-processes.md`: the worker's agent processes and their bounds.
   - `worker/checkouts.md`: workspaces, their stores and holds, a workspace kept for its run, and the checkout's open questions.
   - `worker/checkouts-prepare.md`: checkout preparation, starting points and failure diagnostics.
   - `worker/checkouts-push.md`: delivery, saving and their open questions.
   - `worker/jobs.md`: job kinds, commands and failure diagnostics.
4. **agent/README.md:** what temper fills in of smith's agent: the charter, the
   host tools, result contracts, conventions and the agents temper runs.
5. **../testing/README.md:** how temper is tested, on skein's testing strategy
   and jig's worlds: its tiers, its neighbours' fakes, its scenarios and
   referee, what its worlds check, and its layout.
6. **landing.md:** a project that lands by turn: landings across
   repositories and goals, prepare, receipts by key, proof by dispatch,
   and the project's landing turn.
   - `landing-prepare.md`: preparing candidates, rounds and merge trains.
   - `landing-proof.md`: keys, receipts and proof by dispatch.
   - `landing-turn.md`: the landing turn and its trains.
   - `landing-search.md`: a red round of several units narrowed by
     prefixes: probes at unit boundaries, the blamed unit repaired, the
     green prefix landed and the units above it in the next round.
   - `landing-search-by-test.md`: a search whose probes run only the
     cases the top's gates report failed, one run per workflow and
     commit, and nothing landed on such a probe: the next round proves
     the green prefix whole.
   - `landing-regate.md`: a gate run once more at the same content: a
     red of known flakes rerun before its receipt, and a failed receipt
     superseded once a repair found only flakes and a person released
     its change.
   - `landing-forming.md`: a landing's forming made whole: its task
     made before anything of it is saved, a refused make forming
     nothing while its train waits a queue window, and a start ending a
     landing whose task was never made.
   - `landing-members.md`: a landing's members kept once, by its row,
     a change's landing derived from the live rows, and the asks a
     landing hands a member queued on the change until it takes them.
   - `landing-preparing.md`: a round at preparing reading at each step
     whether its prepare lives, has answered or is absent, and
     recovering across a restart, a worker's loss and a release.
   - `landing-waits.md`: what each step waits for, no wait that nothing
     will answer, the next step due as a round ends, and a stall shown
     by `temper ctl landing list` and `landing show`.
   - `landing-history.md`: each change's and landing's steps kept as
     events after their rows go, swept by age and count, and read by
     `temper ctl landing history`.
   - `landing-times.md`: where each change's time went, from ready to
     landed or ended, split by code from those events into states that
     don't overlap, with each landing's rounds, read by `temper ctl
     landing times`.
7. **after-landing.md:** tiers on the newest main, verdicts by test,
   and a change that lives until its landing is cleared, by queue or by
   turn.
   - `after-landing-runs.md`: tier definitions, scheduling and run reports.
   - `after-landing-storage.md`: after-landing effects, reads and durable records.
8. **findings.md:** findings as labelled issues read back, matched,
   triaged and closed when their fix clears; QA after landing.
   - `red-findings.md`: under `wait: landed`, each red after landing a
     finding blamed on the landing and goal behind it, held until a run
     passes its tests, and shown on that goal and owed by its milestone.
   - `red-repairs.md`: the triager's sorts of a red finding, its blame
     moved, and repair goals, capped and judged at their end.
9. **areas.md:** a project's areas in words and their owners, the
   triager, and how a finding reaches its owner.
10. **milestones.md:** a milestone, a set of goals on jig's gathering:
    its template and identity, its goals asking its maker, its totals
    and the forge's row of it, and the status agent's milestones.

## 2. Structure

```
temper's engine
temper-engine-domain                       the root (root.md)
├── jig-domain-core                        jig's core and its children (jig's domain/engine.md)
├── temper-engine-domain-forge             the forge connector's top (forge.md)
│   ├── temper-engine-domain-forge-vocabulary  what crosses the top's edge (forge/README.md, section 3)
│   ├── skein-forge-client                 the working set, keeping up, fresh reads, the request budget, effects' keys, write lanes (skein's forge-client.md)
│   ├── temper-engine-domain-forge-change  the change procedure and the landing queues
│   ├── temper-engine-domain-forge-landing the landing by turn (landing.md)
│   └── temper-engine-domain-forge-issues  the goals' issues
├── temper-engine-domain-calendar          each project's funding period, renewed (routing.md, 7)
└── temper-engine-domain-trials            a model's trials (trial.md)

temper's worker
temper-worker-domain                       the worker's root (worker/README.md)
├── jig-domain-host                        jig's hub (jig's domain/hosts.md)
├── temper-worker-domain-checkout          the checkout (worker/checkouts.md)
└── smith-host-domain                      smith's agent processes (agent.md)
```

- **No agent in the engine.** temper's root has no hub child and no
  `smith-inline-agent`: every run is on a worker (root.md, section 2).
- **The forge names no type of jig's core.** Its crates depend on
  skein-lib, and its top on jig's people child alone, for its keyed
  adoptions' outcomes; the root translates between it and the core
  (root-translation.md, section 1).
- **The translation to smith is in the protocol layer:**
  `temper-engine-smith` writes the charter, decodes host calls and
  renders their answers, and reads a run's result
  (`../protocol/llm.md`, sections 4 and 5).

## 3. Conventions

- **A bare file name** names a document of this directory: `forge/README.md`
  is `forge/README.md`. temper's other documents are named by their
  relative path: `../protocol/forge/README.md`, `../client/README.md`,
  `../README.md`.
- **jig's documents** are named with jig's name and their path from
  jig's `docs/design/`, with jig's section numbers: jig's
  `domain/connectors.md`, 12; jig's `protocol/channel/owners.md`, 1.4. Never a
  bare jig file name: jig has two `jig/docs/design/client/domain/client/README.md` and two `../testing/README.md`.
- **smith's documents** likewise, by their path in smith's
  `docs/design/`: smith's `domain/run.md`. skein's foundation documents
  by their own names (`programming-model.md`, `testing-strategy.md`),
  and skein's design documents with skein's name: skein's `runtime/lib.md`.
- **Code cites a section** at module or type level, by the document's
  path from `docs/design/`: `forge/change/README.md, 2`; jig's as
  `jig's domain/connectors.md, 12`.
- **Facts about Forgejo and git** are stated as what temper relies on;
  the conformance's `facts.txt` (skein's `testing/fakes/throwaway.md`, section 2) and
  `crates/temper-git-transcripts-tool/facts.txt` hold the evidence, named by fact.
- **Each document** says in one page what it is, then its structure, its
  parts, what it owes and is owed below the domain, its world, and what
  is open. It states the contract; where the code lags or departs is
  its status file's, `<name>.status.md`, beside it.

## 4. Names

temper's own; jig's names are jig's `domain/README.md`, section 4, and
smith's are smith's.

| Name | What it is |
|---|---|
| the forge | the one system temper drives, Forgejo first, as a connector of jig's (skein/docs/design/forge/README.md) |
| repository | a forge's repository adopted into a project with a role, owned, fork or context, and named by the forge's id for it (forge/adoption.md) |
| landing branch | a branch a project names for temper to land changes into (forge/adoption.md) |
| prefix | the branch prefix temper writes under in a repository, `temper/` by default (forge/adoption.md and forge/branches.md) |
| change | a task the forge's change procedure runs: one change to one repository, landed into one branch (forge/change/README.md, 1) |
| producing, repair, resolution | a change's agent work: making its head; fixing failed CI, a gate's remarks, a person's request for changes on the forge or a semantic conflict; resolving a conflict from a merge in progress (forge/change/README.md, 2 and forge/change/repairs.md) |
| update | a change taking its base, made by the forge itself, lazily: when the change is first in its queue, about to get a run, or at its check with its head already behind (forge/updates.md, section 1) |
| gate | a verdict a change asks for at its exact head: an agent's review, a person's approval, or a repository's check; blocking or advisory (forge/change/gates.md, 1) |
| landing rule | what configuration requires of landing into a pattern of branches: CI, up to date, gates, approvals (forge/change/gates.md, 2; root-translation.md, section 1) |
| landing queue | the changes ready to land into one branch, by priority then by readiness; only the first is updated and merged (forge/queues.md) |
| queue repair | the one change made into a landing branch whose own CI fails, first in its queue (forge/queues.md) |
| follow, follow change | a project's setting following a branch of a repository it adopts as context, and the change its follow task makes for each landing there in the paths it names (forge/follows.md) |
| dependency | a project's setting naming another project of the deployment it builds on, shown beside its follows and acted on by nothing (forge/dependencies.md) |
| stall | a change's wait past its bound, which holds it (forge/change/stalls.md) |
| a change's, a task's, a saved-work branch | `temper/<root>/c<task>`, `temper/<root>/r<task>`, `temper/<root>/s<task>`: branches named after the task tree (forge/branches.md) |
| goal's issue | one issue per goal, a projection temper writes on the forge and never reads back (forge/issues.md) |
| hint | a webhook delivery's word that something changed: the forge reads sooner, and believes only what it reads (forge/keeping-up.md, 2) |
| checks | a repository's own checks, at `.temper/pre-pr`, which a run runs before it delivers (agent/charter.md, 4); for a repository without CI, those it is adopted with, each a gate (forge/change/gates.md, 1) |
| checkout | a run's workspace on a worker: the repositories' git trees side by side, cached by workstream (worker/checkouts.md) |
| merge in progress | a checkout that starts with the base merged into the change's branch, its conflicts left for a resolution (worker/checkouts.md) |
| push | temper's delivery: the tree the run checked, committed and pushed as a fast-forward, never forced (agent/README.md, section 5) |
| git identity | a forge's token and commit author for git, one per forge, one of temper's application credentials (`../protocol/credentials-git-identities.md`, section 1) |
| failed preparation | a repository the worker could not prepare, for good: the forge lacks it, its branch or its commit, or refused the identity (worker/hosted-runs.md, 1.3; root-translation.md, 2.4) |
| the forge's charters | temper's kind 3: the charters the agent tasks of a change are made under, its producer's by default and its reviews' (`../protocol/configuration/settings-kinds.md`, 1; root-translation.md, section 1) |
| standalone | one engine and one worker in one process on one person's machine (`../protocol/service-process.md`, section 1) |

## 5. Decisions this design takes

What temper decides within jig's design, with where each is answered:

- **One connector, the forge,** which the root knows by its number;
  test environments as a second connector are open (`../README.md`,
  section 8; root.md, section 2).
- **No agent in the engine.** Every run is on a worker; in standalone
  the worker shares the engine's process, and a restart settles its
  claims at once (root.md, section 2, and root-restart.md, section 1).
- **The root speaks for the forge to the core:** the forge's crates
  name no type of jig's core, and the root makes from the forge's terms
  what the core takes: an effect's description, a verdict, a procedure's
  step and its delegates, a hold's reason, a result (root-translation.md, section
  1).
- **Landing rules are requirements the forge judges:** CI, gates and
  approvals guarded, up to date observed at the decision (forge/change/gates.md, 2;
  root-translation.md, section 1).
- **Mechanics in code, judgement in an LLM.** The change procedure takes
  a change from produced to landed; an agent is woken only to produce, to
  repair, to review or to resolve (forge/change/README.md).
- **Landing queues,** one per branch, by goal priority then by when each
  change became ready, with aging; only the first is updated and merged
  (forge/queues.md).
- **Updates are lazy and mechanical:** the forge's own merge of the base,
  when a change is first or about to get a run; an LLM only resolves a
  conflict (forge/updates.md, section 1).
- **A merge lands exactly the head decided,** in the repository's merge
  style, and nothing is ever forced (forge/change/README.md, 2 and forge/change/gates.md, 2 and forge/branches.md).
- **Every landing is news;** it wakes a goal only when it overlaps the
  files of the goal's changes (forge/topics.md, section 1).
- **A goal's issue is a projection,** written, never read back
  (forge/issues.md).
- **Branches are named after the task tree,** under the repository's
  prefix, saved work on a branch of its own (forge/branches.md).
- **What people write on temper's pull requests is not read,** but their
  approvals; people act in temper's client (forge/change/gates.md, section 2, and forge/README.md, section 7
  ; `../client/README.md`).
- **Only LLMs are counted.** A forge request is load, within the forge's
  request budget, never spend (routing.md, 5; forge/keeping-up.md, 4).
- **A run's workspace:** its change's repository, or the project's home
  and what its spec names and its saved work, with the project's context
  repositories read only; cached by the change's task for a change's
  runs (root-translation.md, 2.1; worker/hosted-runs.md, 1.1).
- **Git identities are application credentials,** one per forge, made
  from the forge's records in jig's secrets' store and granted with an
  assignment; the forge's API token or App key, and a Forgejo forge's
  webhook secret, never leave the engine's protocol layer (root-translation.md, 2.1;
  `../protocol/credentials-git-identities.md`, section 1, and `../protocol/credentials.md`, section 5).
- **The checkout is the worker's alone:** cached by workstream, a merge
  in progress as a place to start, a push that commits exactly the tree
  the run checked, and no credentials on disk (worker/checkouts.md).
- **smith is the agent,** unchanged: temper writes its charters and
  serves its host tools, and its binary runs smith's agent as its
  `agent` command (agent/README.md, sections 1 and 3).
- **A repository's knowledge stays in the repository** (its
  `AGENTS.md`); notes are jig's, a repository's a resource scope on the
  forge (root-translation.md, section 1; agent/charter.md, 4).
- **Periods from the smallest source:** one per configured project at
  the first start (`../protocol/configuration/startup.md`, section 1), each
  renewed by the root's calendar (routing.md, 7).

## 6. What is open

Each document lists its own; the ones that reach across:

- what temper's root keeps between steps, and the forge's work it does
  that jig's connectors do themselves (root.md, section 12);
- an assignment naming the head its push must find (`worker/checkouts-push.md`, section 9);
- a person's approval posted on the forge as the person (`../README.md`,
  section 8; forge/README.md, section 11);

- which host tools a charter offers, and a cancel's grace (agent/README.md,
  section 8);
- jig's, which temper's root inherits: commits in flight, briefs'
  budgets, transcripts' limits, views' bounds and scaling out (jig's
  `domain/engine/README.md`, section 10).
