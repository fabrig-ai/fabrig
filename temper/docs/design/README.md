# temper's design

Provisional, 2026-10-18, revised 2026-10-19. What temper is, what it is
built from, and where each part of its design lives. This document
comes first; the documents it lists in section 6 go deeper into each
part. temper is an application
of jig, the kit for agentic applications: jig's design is jig's own
(jig's `README.md`), and temper's documents say what temper adds,
decides or chooses, citing jig for the rest. The mechanics are those of
skein's `programming-model.md`. What is still open is listed in
section 8.

## 1. In one page

- **Software made by agents, directed by people.** temper is where
  people direct the development of software and agents do it: they
  write, check, review and land the changes, on a forge. People decide
  at the level of goals, designs and plans, and steer where agents
  cannot decide. Feedback on code is the agents' business.
- **An application of jig.** jig's core runs the work: tasks, their
  executors (agents, procedures, people), messages, authority, and
  connectors to external systems (jig's `domain/model.md`). temper brings
  what is its own: the forge as a connector, its agents' charters and
  tools, the checkout its workers prepare, its client, and the forge's
  setup.
- **The forge is the one external system** (`domain/forge/README.md`).
  Repositories are adopted as they are. A change has a procedure of its
  own to landing: produced, checked by CI, reviewed by gates, repaired,
  brought up to date, landed through a queue per branch. A goal shows on
  the forge as an issue, a projection temper writes and never reads back.
- **What temper does on the forge is temper's.** It writes only to what
  it owns, lands into shared branches, and takes part as one participant
  in what others own. It never forces a push, and merges exactly the head
  its decision saw. A change it did not make to what it owns is drift:
  the task relying on it is held, and nothing is written over.
- **Mechanics are code, judgement is an LLM's.** Procedures carry out
  what needs no judgement: a change's way to landing, a branch brought
  up to date. An agent is woken for what does: making a change,
  repairing a failure, reviewing, resolving a conflict, revising a plan.
- **The agents are smith's** (`domain/agent/README.md`). temper writes their
  charters and serves their tools; it builds no agent. Its worker hosts
  smith's agent processes, each run in a checkout of the repositories its
  task names (`domain/worker/README.md`).
- **The root is temper's, the rails are jig's.** temper's root composes
  jig's core with the forge connector, speaks jig's hub's link to its
  workers, and routes and translates between them; it decides nothing
  jig's core decides (`domain/root.md`). jig's rails hold temper to
  jig's promises: skein-lib's journal, jig's restart script, its
  conformance world and its referee, with temper's own wiring and its
  fake forge (jig's `README.md`, section 6).
- **One binary for the box, one per process for a deployment,
  standalone first** (`protocol/README.md`; `protocol/binaries.md`).
  `temper-box serve` runs the engine and one worker in one process on
  one person's machine; `temper-box open` is the person's terminal;
  `temper-box internal agent` is smith's agent, one process per run;
  `temper-box forge` and `temper box` set a forge up and run it
  (`protocol/setup/README.md`). A deployment runs four binaries, each
  holding its process's code alone: `temper-engine`, `temper-worker`,
  `temper-agent` and the client, `temper`; the box links them all
  (`protocol/binaries.md`, 1 and 4).
- **Tested at every tier** (`testing/README.md`): domains and fakes in worlds,
  then through bytes, then the standalone service, hosted agents and
  fake peers on one real ring, then the real binary with an owned
  Forgejo, its runner and real git, then a real model.

## 2. The first milestone

The smallest product that works end to end, for real: one person, on
their own machine, with a terminal, a real Forgejo repository and a real
model. They adopt the repository, ask a chat for a change, follow the
work at the chat's level, and see what landed. The change is checked by
CI and an agent's review, and lands with no person in between.

One story:

1. `temper-box serve` runs on a deployment's directories,
   configured for one Forgejo (with Actions, or with project checks), one
   git identity, one worker and its LLM providers, never an account
   (`protocol/configuration/README.md`; `protocol/credentials.md`, sections 3
   and 4). The forge is connected, managed, or owned on a box
   (`protocol/setup/README.md`).
2. `temper-box open DIRECTORY LINK` signs the person in by the printed link
   and shows their inbox (`client/line.md` and `client/README.md`, section 4). With
   no LLM account yet, a banner offers the ways to add one: a sign-in
   with an LLM provider, a Claude Code login made with `temper-box
   program-login`, or a key; the person adds one on the accounts page
   (`client/pages/README.md` and `client/pages/accounts.md`, 2).
3. The person adopts a repository into the configured project, through
   the adoption form (`client/forge/family.md`, 4).
4. The person starts a chat and asks for a change; the stock chat reads
   the code through the forge and delegates the change whole to a change
   task (`domain/agent/tools.md`, 1; `domain/forge/change/README.md`, 1).
5. Agents do the rest: the change is produced, its pull request opened,
   checked by CI and by an agent's review (a gate the usual landing rule
   asks for, `domain/forge/change/gates.md`, 1 and 2), repaired when either
   fails, and landed. No person acts on it.
6. The person follows it from the chat: its delegates and their phases
   in words, and, if the change is held, why, with what may be done
   (`client/forge/README.md`, 2.4 and 2.5).
7. The person sees what landed: the pull request and the merge commit,
   in the forge's words, with a link to each on the forge
   (`client/forge/README.md`, 2.5).
8. `temper-box serve` is stopped and started again; nothing the person saw
   is lost, and work in flight goes on.

Not part of it: a person's approval of changes (a choice a person makes
when giving direction, never temper's default), the browser, a run's
live text, the forge's panels in full, GitHub, several people.

## 3. The system

```
person    temper-box open: temper's terminal, its pages around jig's client (client/README.md)
engine    temper-box serve: temper's root around jig's core and the forge connector (domain/root.md)
store     jig's store in the engine's process: jig's records and temper's, committed together (protocol/store.md)
worker    standalone, in the engine's process: temper's worker root on jig's hub, the checkout (domain/worker/README.md)
agents    smith's, one `temper-box internal agent` process per run, on the worker (domain/agent.md)
forge     Forgejo and its runner: polled, hinted by its webhooks (domain/forge.md; protocol/forge.md)
```

- **One engine per deployment.** It is the store's only writer and the
  forge's only client (jig's `domain/model.md`, section 2).
- **The engine decides, the worker hosts, agents think.** The worker
  prepares checkouts, runs agents and pushes what they made; agents reach
  the engine only through the tools their host relays (jig's
  `domain/hosts/README.md`).
- **People reach temper through its client:** temper's pages on jig's
  client, `jig-client`, in a terminal and in a browser
  (`client/README.md`; `client/web/README.md`).
- **Standalone first.** Hosts apart, webhooks from another host, TLS on
  the engine's listeners and contained process trees come with a
  deployment beyond one person's machine (`protocol/README.md`,
  section 11).

## 4. On skein, smith and jig

temper builds on three repositories, each pinned by `Cargo.lock`, and
follows their design:

| Repository | What temper takes from it |
|---|---|
| skein | the programming model and the testing strategy; lib and its journal; io, its machines and protocols (HTTP, JSON, TLS, the key-value store, channels, accounts); the UI kit and its text shell; the simulator, the fake machine and the fake person |
| smith | the agent: its runs, sessions and tools' families, its agent shell, its host domain and host protocol |
| jig | the core and its children (tasks, authority, people, the fleet, accounts, briefs, notes, views); the hub; the root's shape and its rails; the store, the workers' channel, LLM accounts, configuration, the clients' component; `jig-client`; the conformance world and its referee |

What temper adds is listed in `domain/README.md` (the domain) and
`protocol/README.md`, section 1 (the protocol layer). Where temper needs
something jig lacks, it is added to jig, with temper as its first user,
never built around jig in temper.

## 5. What temper promises

jig's promises (jig's `domain/model.md`, section 11) hold for temper,
checked by jig's referee in temper's worlds with temper's wiring. On the
forge, temper adds:

- **Nothing is forced.** temper never forces a push, never writes to an
  object it does not own but as a participant, and never overwrites a
  change it did not make (`domain/forge/change/README.md`, 3, and `domain/forge/change/gates.md`, 2).
- **A merge lands what was checked:** exactly the head its decision saw,
  with every requirement its branch's rules enforce met: CI, gates and
  approvals at that head, up to date on the decision's reads
  (`domain/forge/change/gates.md`, 2).
- **A landing temper did not make is news,** like any other
  (`domain/forge/README.md`, 5.2).
- **A change temper did not make to what it owns holds the task that
  relies on it,** and the client says why (`domain/forge/change/README.md`, 3).

## 6. Documents

- **`domain/`:** temper's domain layer on jig's: its root, the forge
  connector, landings by turn (`landing.md`), what follows a landing
  (`after-landing.md`) and findings (`findings.md`), the worker, and
  what temper fills in of smith's agent; and `testing/README.md`, temper's tiers, its fakes, its scenarios and its layout,
  on skein's testing strategy and jig's worlds. Its `README.md` gives the
  reading order, the structure, the names and the decisions.
  - `domain/worker/README.md`: the worker's root, assignments, runs and world.
  - `domain/worker/checkouts.md`: workspaces, preparing, delivering, saving and open questions.
  - `domain/worker/jobs.md`: job kinds, commands and failure diagnostics.
  - `testing/qa/README.md`: QA after landing, beside the deployment: its
    tool, home and loop; its ledger (`testing/qa/ledger.md`), the
    watcher on main (`testing/qa/landings.md`), kept builds
    (`testing/qa/builds.md`), triage (`testing/qa/triage.md`), places
    and copies (`testing/qa/places.md`), passes (`testing/qa/passes.md`),
    findings (`testing/qa/findings.md`) and the setups' catalogue with
    its twins (`testing/qa/setups.md`).
- **`protocol/`:** temper's protocol layer on jig's: the processes and
  the binary, the configuration, the credentials, temper's parts on the
  workers' channel, its families in the store, the host tools and LLM
  accounts, the forge's protocol on skein's forge kit (skein's
  `forge/README.md`), git, and the forge's setup. Its `README.md` gives the
  reading order and the names.
- **`client/`:** temper's client on `jig-client`. Its `README.md` is the
  client: its client root and pages, the forge's client family, its view,
  the terminal, signing in, and the clients' listener; `client/forge/README.md` is the
  forge's part of the client, `client/web/README.md` the browser shell,
  `client/web/journeys.md` its pages recorded and printed,
  `client/web/look-requests.md` the look requests' round trip,
  `client/box-binary.md` how `temper box` finds or downloads
  `temper-box`,
  `client/drive.md` QA's session driving the browser, the TUI and ctl a
  line at a time from a given build, `client/recipe.md` the recipes it
  walks to the states QA's cases start from,
  `client/ctl/README.md`
  `temper ctl`, temper for agents (reads, jig's operations and waits,
  from a command line), and `client/web/mockups/` the pages' design: their look,
  parts, words and acts, binding for what is not built yet and
  reconciled to what temper draws, with a `STATUS.md` of their own.
  Its documents go by surface (the user, 2026-11-07): what is generic
  to the client in `client/` itself, the browser's in `client/web/`, the
  TUI's in `client/tui/`, and `temper ctl`'s in `client/ctl/`.
  - `client/tui/pages/README.md`: the TUI's state, shared words, foot and commands.
  - `client/tui/pages/sign-in.md`: signing in and renewing from the engine's link.
  - `client/tui/pages/launchpad.md`: the launchpad and its needs walk.
  - `client/tui/pages/chat.md`: a chat's conversation and question.
  - `client/tui/pages/design-session.md`: the draft pane, reply choices and starting a goal.
- **`cli/`:** temper's command lines: each binary's package, crate,
  table, help, exits and environment, and the test set its tests read
  (`cli/README.md`), and the reference documents generated from the
  tables beside it, each binary's `<binary>-args.md` and
  `<binary>-env.md`, the tree's common environment document and the
  test set's `test-env`.
- **`playground.md`:** the box's playground run whole in a visitor's
  browser: the world's composition, its stories as data, the page and
  its frame, how it is served, bounded and tested.
- **`release.md`:** the download: the one Linux binary of a release,
  its identity and floors, what the machine must have, how a release
  is made, Apple Silicon's file for the site, and its walk on a fresh
  machine.
- **`draft/`:** the drafts, the vision this design is written from
  (the repository's `AGENTS.md`, section 6).
- **Status files:** beside each document, `<name>.status.md`, where
  the code lags or departs from it; `STATUS.md` indexes them.

## 7. Conventions

- **A bare file name** names a document of the same directory: in
  `domain/`, `client/forge/README.md` is `domain/forge/README.md`; in `protocol/`, it is
  `protocol/forge/README.md`. temper's other documents are named by their path
  from there: `protocol/forge/README.md`, `../client/README.md`; from this
  directory, `domain/root.md`.
- **Code cites a section** by the document's path from this directory,
  at module or type level: `domain/forge/change/README.md`, 2; `protocol/forge/README.md`,
  2.2; `client/pages/README.md`.
- **jig's documents** are named with jig's name and their path in jig's
  `docs/design/`: jig's `domain/connectors.md`, jig's
  `protocol/channel/README.md`, jig's `client/domain/client/README.md`. jig has two
  `jig/docs/design/client/domain/client/README.md` and two `testing/README.md`, so a bare jig file name is never
  enough.
- **smith's documents** likewise: smith's `domain/run.md`. skein's
  foundation documents by their own names: `programming-model.md`,
  `testing-strategy.md`, `notes.md`; skein's design documents with
  skein's name: skein's `runtime/lib.md`.
- **Each document** says in one page what it is, then its structure, its
  parts, what it owes and is owed below the domain, its world, and what
  is open. It states the contract; where the code lags or departs is its
  status file's.

## 8. Open questions

- **A second connector:** test environments, which an agent provisions,
  explores and tears down. It would check that jig's connector contract
  is not the forge's in disguise. It waits for a need.
- **A person's approval as a rule:** whether temper alone enforces it,
  or posts the approval as the person, with a forge credential they
  signed in with, so that the forge's protection requires it too
  (`domain/forge/README.md`, section 11).
- **Contributions in and out:** outside contributions to a project
  temper works on, and temper's own contributions upstream
  (`domain/forge/README.md`, sections 7 and 8). Neither is needed first.

## 9. Crates' names

temper's crates follow jig's rule (jig's `README.md`, section 15), with
temper's side before the layer:

- `temper-engine-domain` (the root), its forge children
  `temper-engine-domain-forge`, `-forge-change`, `-forge-landing`,
  `-forge-issues` and `-forge-tiers`, which own skein's forge client,
  `skein-forge-client`, and its calendar, `temper-engine-domain-calendar`;
  `temper-worker-domain` (the worker's root),
  `temper-worker-domain-checkout` and its job capability,
  `temper-worker-domain-tooling` (`domain/worker/jobs.md`);
- the protocol layer's: `temper-worker-protocol-git`,
  `temper-protocol-store`, `temper-protocol-records` (the forges'
  records), `temper-protocol-worker-wire`, `temper-protocol-brief` (the
  brief's fitting), and `temper-engine-smith`, the translation to smith; the forge component is skein's,
  `skein-forge-connection`;
- the forge's, of no side: skein's, `skein-forge` (its documents and
  call plans, which the forge component and the setup share) and
  `skein-forge-setup` (reconcile); its setup's rest, outside the
  services, `temper-forge-setup`, `temper-forge-setup-shell` and the
  box's machine, `temper-box-machine`; a pinned binary's fetch and Claude Code's launcher, of
  no side, `temper-fetch`, `temper-fetch-shell` and `temper-launcher`;
- the user documentation's, of no side, `temper-docs`
  (`protocol/documentation.md`); the settings' form, `temper-settings-form`;
  `temper-box init`'s files, `temper-init`; the fakes the binary ships,
  `temper-fake` and `temper-fake-git` (`testing/fakes.md`, 3);
- the playground's, of no side: its composition, `temper-playground`,
  its page, `temper-playground-page`, and the fake git program it hosts,
  `temper-fake-git-hosted` (`playground.md`, 3);
- the client's, under `temper-client-` (`client/README.md`, section 2);
- the services, `temper-service-core`, `temper-engine-service`,
  `temper-worker-service` and their one process, `temper-service`
  (`protocol/services.md`); the box's binary, `temper-box`, and
  the engine's, `temper-engine`, packages under `bin/` whose crates are
  `temper-box-shell` and `temper-engine-shell`, with the shell they
  share, `temper-shell`, and `temper-version` (`cli/README.md`, 2); the
  deployment's client, `temper`, with its `temper-client-shell` crate, and its agent, `temper-agent`
  (`protocol/binaries.md`).

`protocol/components.md` says what each holds and depends on. Test
kits are named by their kind (`testing/temper-github-app`); the
forge's are skein's (`skein-fake-forge-domain` in skein's `crates/`,
`testing/skein-throwaway-forgejo`), and the worlds live in `tests/`
(`testing/README.md`, section 5).
