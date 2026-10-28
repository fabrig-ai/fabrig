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
the order it gives, jig's `domain/core.md` and `domain/engine.md` above
all. Then temper's:

1. **root.md:** temper's root: what it composes, its vocabulary, the one
   shape as temper holds it, routing, translation, putting outputs
   together, its store vocabulary, restart and stopping, its limits, and
   what it never does.
2. **forge.md:** the forge connector: adopting repositories, keeping up,
   effects, topics and landings as news, the change procedure and its
   requirements, landing queues, updates, branches, issues, and the ways
   temper takes part in what others own.
3. **worker.md:** the worker: its root on jig's hub and smith's host
   domain, the items an assignment gives a run's workspace, and the
   checkout.
4. **agent.md:** what temper fills in of smith's agent: the charter, the
   host tools, result contracts, conventions and the agents temper runs.
5. **testing.md:** how temper is tested, on skein's testing strategy
   and jig's worlds: its tiers, its neighbours' fakes, its scenarios and
   referee, what its worlds check, and its layout.
6. **landing.md:** a project that lands by turn: landings across
   repositories and goals, prepare, receipts by key, proof by dispatch,
   and the project's landing turn.
7. **after-landing.md:** tiers on the newest main, verdicts by test,
   and a change that lives until its landing is cleared, by queue or by
   turn.
8. **findings.md:** findings as labelled issues read back, matched,
   triaged and closed when their fix clears; QA after landing.

## 2. Structure

```
temper's engine
temper-engine-domain                       the root (root.md)
├── jig-domain-core                        jig's core and its children (jig's domain/engine.md)
└── temper-engine-domain-forge             the forge connector's top (forge.md)
    ├── skein-forge-client                 the working set, keeping up, fresh reads, the request budget, effects' keys, write lanes (skein's forge-client.md)
    ├── temper-engine-domain-forge-change  the change procedure and the landing queues
    ├── temper-engine-domain-forge-landing the landing by turn (landing.md)
    └── temper-engine-domain-forge-issues  the goals' issues

temper's worker
temper-worker-domain                       the worker's root (worker.md)
├── jig-domain-host                        jig's hub (jig's domain/hosts.md)
├── temper-worker-domain-checkout          the checkout (worker.md, section 5)
└── smith-host-domain                      smith's agent processes (agent.md)
```

- **No agent in the engine.** temper's root has no hub child and no
  `smith-inline-agent`: every run is on a worker (root.md, section 2).
- **The forge names no type of jig's core.** Its crates depend on
  skein-lib, and its top on jig's people child alone, for its keyed
  adoptions' outcomes; the root translates between it and the core
  (root.md, section 6).
- **The translation to smith is in the protocol layer:**
  `temper-engine-smith` writes the charter, decodes host calls and
  renders their answers, and reads a run's result
  (`../protocol/llm.md`, sections 4 and 5).

## 3. Conventions

- **A bare file name** names a document of this directory: `forge.md`
  is `domain/forge.md`. temper's other documents are named by their
  relative path: `../protocol/forge.md`, `../client/README.md`,
  `../README.md`.
- **jig's documents** are named with jig's name and their path from
  jig's `docs/design/`, with jig's section numbers: jig's
  `domain/connectors.md`, 12; jig's `protocol/channel.md`, 6.4. Never a
  bare jig file name: jig has two `client.md` and two `testing.md`.
- **smith's documents** likewise, by their path in smith's
  `docs/design/`: smith's `domain/run.md`. skein's foundation documents
  by their own names (`programming-model.md`, `testing-strategy.md`),
  and skein's design documents with skein's name: skein's `lib.md`.
- **Code cites a section** at module or type level, by the document's
  path from `docs/design/`: `domain/forge.md, 8.2`; jig's as
  `jig's domain/connectors.md, 12`.
- **Facts about Forgejo and git** are stated as what temper relies on;
  the conformance's `facts.txt` (skein's `fake-forge.md`, section 7) and
  `tools/git-transcripts/facts.txt` hold the evidence, named by fact.
- **Each document** says in one page what it is, then its structure, its
  parts, what it owes and is owed below the domain, its world, and what
  is open. It states the contract; what is built is `../STATUS.md`'s.

## 4. Names

temper's own; jig's names are jig's `domain/README.md`, section 4, and
smith's are smith's.

| Name | What it is |
|---|---|
| the forge | the one system temper drives, Forgejo first, as a connector of jig's (forge.md) |
| repository | a forge's repository adopted into a project with a role, owned, fork or context, and named by the forge's id for it (forge.md, section 4) |
| landing branch | a branch a project names for temper to land changes into (forge.md, section 4) |
| prefix | the branch prefix temper writes under in a repository, `temper/` by default (forge.md, sections 4 and 11) |
| change | a task the forge's change procedure runs: one change to one repository, landed into one branch (forge.md, 8.1) |
| producing, repair, resolution | a change's agent work: making its head; fixing failed CI, a gate's remarks or a semantic conflict; resolving a conflict from a merge in progress (forge.md, 8.2 and 8.4) |
| update | a change taking its base, made by the forge itself, lazily: when the change is first in its queue or about to get a run (forge.md, section 10) |
| gate | a verdict a change asks for at its exact head: an agent's review, a person's approval, or a repository's check; blocking or advisory (forge.md, 8.3) |
| landing rule | what configuration requires of landing into a pattern of branches: CI, up to date, gates, approvals (forge.md, 8.7; root.md, section 6) |
| landing queue | the changes ready to land into one branch, by priority then by readiness; only the first is updated and merged (forge.md, section 9) |
| queue repair | the one change made into a landing branch whose own CI fails, first in its queue (forge.md, section 9) |
| stall | a change's wait past its bound, which holds it (forge.md, 8.4) |
| a change's, a task's, a saved-work branch | `temper/<root>/c<task>`, `temper/<root>/r<task>`, `temper/<root>/s<task>`: branches named after the task tree (forge.md, section 11) |
| goal's issue | one issue per goal, a projection temper writes on the forge and never reads back (forge.md, section 12) |
| hint | a webhook delivery's word that something changed: the forge reads sooner, and believes only what it reads (forge.md, 5.2) |
| checks | a repository's own checks, at `.temper/pre-pr`, which a run runs before it delivers (agent.md, 4.5); for a repository without CI, those it is adopted with, each a gate (forge.md, 8.3) |
| checkout | a run's workspace on a worker: the repositories' git trees side by side, cached by workstream (worker.md, section 5) |
| merge in progress | a checkout that starts with the base merged into the change's branch, its conflicts left for a resolution (worker.md, section 5) |
| push | temper's delivery: the tree the run checked, committed and pushed as a fast-forward, never forced (agent.md, section 6) |
| git identity | a forge's token and commit author for git, one per forge, one of temper's application credentials (`../protocol/credentials.md`, section 4) |
| failed preparation | a repository the worker could not prepare, for good: the forge lacks it, its branch or its commit, or refused the identity (worker.md, 4.3; root.md, 7.4) |
| the forge's charter | the charter configuration names for the agent tasks the forge's procedures make (root.md, section 6) |
| standalone | one engine and one worker in one process on one person's machine (`../protocol/README.md`, section 8) |

## 5. Decisions this design takes

What temper decides within jig's design, with where each is answered:

- **One connector, the forge,** which the root knows by its number;
  test environments as a second connector are open (`../README.md`,
  section 8; root.md, section 2).
- **No agent in the engine.** Every run is on a worker; in standalone
  the worker shares the engine's process, and a restart settles its
  claims at once (root.md, sections 2 and 9).
- **The root speaks for the forge to the core:** the forge's crates
  name no type of jig's core, and the root makes from the forge's terms
  what the core takes: an effect's description, a verdict, a procedure's
  step and its delegates, a hold's reason, a result (root.md, section
  6).
- **Landing rules are requirements the forge judges:** CI, gates and
  approvals guarded, up to date observed at the decision (forge.md, 8.7;
  root.md, section 6).
- **Mechanics in code, judgement in an LLM.** The change procedure takes
  a change from produced to landed; an agent is woken only to produce, to
  repair, to review or to resolve (forge.md, section 8).
- **Landing queues,** one per branch, by goal priority then by when each
  change became ready, with aging; only the first is updated and merged
  (forge.md, section 9).
- **Updates are lazy and mechanical:** the forge's own merge of the base,
  when a change is first or about to get a run; an LLM only resolves a
  conflict (forge.md, section 10).
- **A merge lands exactly the head decided,** in the repository's merge
  style, and nothing is ever forced (forge.md, 8.2, 8.7 and section 11).
- **Every landing is news;** it wakes a goal only when it overlaps the
  files of the goal's changes (forge.md, section 7).
- **A goal's issue is a projection,** written, never read back
  (forge.md, section 12).
- **Branches are named after the task tree,** under the repository's
  prefix, saved work on a branch of its own (forge.md, section 11).
- **What people write on temper's pull requests is not read,** but their
  approvals; people act in temper's client (forge.md, sections 8.7 and
  13; `../client/README.md`).
- **Only LLMs are counted.** A forge request is load, within the forge's
  request budget, never spend (root.md, 5.5; forge.md, 5.4).
- **A run's workspace:** its change's repository, or the project's home
  and what its spec names and its saved work, with the project's context
  repositories read only; cached by the change's task for a change's
  runs (root.md, 7.1; worker.md, 4.1).
- **Git identities are application credentials,** one per forge, made
  from the forge's records in jig's secrets' store and granted with an
  assignment; the forge's API token or App key, and a Forgejo forge's
  webhook secret, never leave the engine's protocol layer (root.md, 7.1;
  `../protocol/credentials.md`, sections 4 and 5).
- **The checkout is the worker's alone:** cached by workstream, a merge
  in progress as a place to start, a push that commits exactly the tree
  the run checked, and no credentials on disk (worker.md, section 5).
- **smith is the agent,** unchanged: temper writes its charters and
  serves its host tools, and its binary runs smith's agent as its
  `agent` command (agent.md, sections 1 and 3).
- **A repository's knowledge stays in the repository** (its
  `AGENTS.md`); notes are jig's, a repository's a resource scope on the
  forge (root.md, section 6; agent.md, 4.5).
- **Periods from the smallest source:** one per configured project at
  the first start (`../protocol/configuration.md`, section 8).

## 6. What is open

Each document lists its own; the ones that reach across:

- what temper's root keeps between steps, and the forge's work it does
  that jig's connectors do themselves (root.md, section 13);
- an assignment naming the head its push must find (worker.md, section
  12);
- a person's approval posted on the forge as the person (`../README.md`,
  section 8; forge.md, section 20);
- a calendar for periods, as a child of temper's root
  (`../protocol/configuration.md`, section 11);
- which host tools a charter offers, and a cancel's grace (agent.md,
  section 10);
- jig's, which temper's root inherits: commits in flight, briefs'
  budgets, transcripts' limits, views' bounds and scaling out (jig's
  `domain/engine.md`, section 16).
