# Connectors: facts and events

## 1. Facts and events

### 1.1 The working set

- **What the system owns of the live work:** for `ops`'s observability,
  the services' health and error rates; for its infrastructure, the
  services' versions and replicas, the environments' states, the pools'
  slots; for a forge, pull requests' heads and CI on them. A working set,
  not a copy: it grows with the live resources, never with the system's
  history.
- **Kept up** by reading what changed, sooner when a hint arrives, with
  polling as the backstop, at a cost that follows the rate of change.
- **Read afresh** before a decision depends on a fact, after a restart
  as at any time: a decision on a resource still waiting for its first
  fresh read waits for that read, or for the connector's fresh reads
  (`connectors.md`, section 14), a resource at a time, and a restart adds no barrier of
  its own. A procedure's step
  reads what its decision needs, and a judge reads what its verdict
  needs (`connectors.md`, section 7).

### 1.2 Events

- **Hints, never state.** A webhook, an alert, or a change keeping up
  found, tells the connector that something may have changed; it reads
  what it needs and updates its working set.
- **Echoes dropped.** What the application itself did comes back as
  events too; the connector drops those it can tell are its own (an
  effect it made, by its key; a run's write, by its writer slot,
  `connectors.md`, section 11) and reads the rest.
- **Then three things follow:** procedures whose facts changed step;
  holds whose resources drifted are checked (`connectors.md`, section 11); news is
  published on the topics it touches.

### 1.3 Topics

- **A topic is a subject a connector names:** a service's alerts, its
  health, an environment's state, the landings on a branch.
- **Subscribers are tasks,** by number, and their number per topic is
  bounded. A task subscribes through its tool, through its creator, or
  through the core (a goal's topics, 1.4).
- **Offered to agents.** A connector says which kinds of its topics an
  agent may subscribe to with `subscribe` (engine/executors-tools.md, 1), each with
  what a call names it by, as it offers kinds of effect (`connectors-effects.md`, 1.5); asked,
  it names the resources a topic concerns, which a read grant of the
  task's must cover (authority-checks.md, 1.5; hosts/charters.md).
- **Each event is classified per subscriber:** *wakes*, *kept* or
  *dropped*, by the connector's judgement and without an LLM: an alert
  on a service a watch covers wakes it; one it already has open news of
  merges; one below the watch's threshold is kept. The class travels
  with the news into the task's inbox, whose wake policy may lower it,
  never raise it (tasks/messages.md, 3).

### 1.4 Goals' topics

A connector may offer topics for goals: what a goal's subtree should
hear of the resources it works on, such as the landings that overlap a
goal's changes, or the alerts on the services an incident's
remediations touched. The core subscribes each goal to them as its
subtree names resources (model.md, 6.2). A goal is a task marked
tracked, by whatever request made or amended it: the core knows it by
that mark, never by a template or a label, and a connector hears of it
only from the core (tasks/README.md, section 3).

### 1.5 What clients see of it

A connector's values reach a party's client in the application's own
documents, beside jig's (`../client/domain/contract/documents.md`):

- **its subjects,** which parties watch as they watch jig's: the
  connector marks each changed, through the root, as its state changes,
  and makes its document when the views make a delivery
  (engine/views.md);
- **its reads,** a document of its own a party asks for once, which it
  makes when the core has admitted the read, as jig's reads are answered
  (engine/decisions.md, 1.3);
- **the scope each names:** a watch of one of its subjects, and a read
  of one of its documents, name a project, the deployment, or the
  party's own (what is the signed-in party's across projects), since the
  core cannot read a connector's subject. The core admits them by that
  scope (engine/views.md); the connector, asked for the document,
  judges that the subject is in that scope and what the party may see
  of it, and answers that it is unknown, or not allowed, otherwise;
- **windows:** a watch of one of its subjects, and a read, carry a
  window, as jig's do; the connector makes the document within it
  (`../client/domain/contract/watches.md`, 1.5);
- **document bytes:** the ops connectors' `document_bound` counts the
  largest document at their limits, including its owned nested values.
  It sizes a document's envelope, not the connector's heap;
- **its parts,** which jig's documents name by reference (a resource, a
  kind, a task's panel, a proposed effect, a hold's account), read and
  followed by its side of the application's client. An outbox entry is
  a part of its own, named by the connector's number and the entry's,
  apart from a proposed effect's description: an effect made or in
  flight, or the one that held a task. A part's value lives as long as
  the connector's records keep it (a procedure's result, while it keeps
  that procedure's record); past that, the part is unknown, and its
  connector answers so;
- **its parts of jig's task rows**
  (`../client/domain/contract/parts.md`): the kinds it makes, which the
  application declares in the core's configuration, each with its bytes
  and its inputs, among its task's line, its task's notes, its delegates'
  lines, its delegates' notes and the connector's own records of the task
  (engine/views.md, 2). Asked for a page of rows' parts, it makes each from
  its state, the core's and the store's, loading as a read loads, and
  answers each made or failed, within the decision or after its loads. It
  marks a task's parts through the root when its own records of that task
  change. jig reads none of a part's bytes;
- **its keyed requests:** a request about one of its own objects comes
  to it through the root, keyed by the people child; it decides it
  within the decision, asks the core for what it needs on the party's
  behalf, and keeps its outcome under the people child's token, durably,
  for the retention, to give again when the request is sent again
  (people/acts.md, 1).
