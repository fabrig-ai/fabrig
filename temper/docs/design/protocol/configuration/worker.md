# A worker's configuration

This document holds the worker and agent configuration, split from [`README.md`](README.md).

## 1. temper's members for a worker

**The welcome** is no member: it is temper's setting of kind 20
(`settings-kinds.md`, 1), shown by default under the box, and
`application.welcome` is refused (`moved.md`). It shows the welcome
(`../../client/pages/README.md`) and lets the deployment offer the
playground (../setup/playground.md), read by the client from the
deployment's read (`../../client/forge/records.md`) and by
`StartPlayground`'s offer.

**A worker's workspaces and git** were `application.workers`, a list
with one entry per entry of the top-level `workers`; they are temper's
constants now, and the list is refused (`moved.md`, section 2): the
root is `workspaces` in the data directory, and `git` the first on the
worker's `PATH` at its start. What each feeds, and how it is checked,
is as the members' were:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `root` | a directory, a data path | made private (0o700) if absent; an existing one must be the process's user's, private, and not reached through a link | the worker's workspace directory: every checkout's workspace is made under it (../git.md, section 2; `../../domain/worker/checkouts.md`) |
| `git` | string, or `null` | an absolute path to an executable file, checked at start, whose `git version` is at least the one ../git.md, section 3 names; `null` for none, as `temper-box init` writes it on a machine with no git (../setup/init.md, 2) | the program the git component runs for each invocation in a repository whose record names none (../channel.md, section 4), in the environment ../git.md, section 5 gives it; with `null`, each such invocation fails, naming its forge's host, and the row of each connected forge but a fake one warns (../git.md, section 3; `../../client/forge/deployment.md`). A forge whose record names a git program runs that instead: the playground's fake forge the machine's fake `git` (`engine.md`; ../setup/init.md, 2) |

**The agents' environment** is `application.environment`, in the
standalone and worker-alone shapes; an engine alone, which runs no
agent, refuses it as an unknown member. It's the worker's own: the
variables its agents' programs get, by the rule of `figures.md`,
section 6, which gives the worker process's own `HOME` and `PATH`
where the list is absent or omits them. Its entries have the form and
bounds a worker entry's `environment` had before the entries went:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `environment` | a list of `{"name", "value"}`, or absent | each entry an object of exactly `name` and `value`, both required, each a string of at most 1,048,576 bytes; names unique, a repeated one refused at its entry's `name`; each refusal names the entry, `application.environment[i]` and its member | the agent configuration's `environment`, which temper writes at each start (`figures.md`, section 6) |

- **smith's own checks hold too:** smith's decoder reads the written
  configuration back before the start goes on (`figures.md`, section 6),
  so a name that is empty or holds `=` or a NUL, or a value holding a
  NUL, refuses the start as smith's configuration refuses it.
- **Claude Code's program is no member:** it's the `claude` found on the
  worker's `PATH` at start, or `"fetched"` with none (`figures.md`,
  section 6).

- **The worker's root takes the rest from its engine's figures**
  (`figures.md`, section 3): the hub's limits, the checkout's and the
  git component's; smith's host limits and the bounds its hello states
  from the agent's configuration written from them (section 2), as jig's
  reference binary derives them, but for the agents the host holds, which
  are its slots, the smaller of its `slots` and the figures' places
  (`../../domain/worker/agent-processes.md`, section 1); its stop bound from jig's
  graces.
- **Workers are alike** (jig's `protocol/configuration.md`, section 4):
  no credential but the worker's secret, and no forge configuration.
  Remotes, git identities and a forge's git program come with each
  assignment, in its repositories' records (../channel.md, section 4).
- **A worker on another host** takes its own machine's git as its `git`:
  a fake forge is on loopback alone (../setup/playground.md), and serves no
  worker on another host. The cluster's workers are on its machine, and
  it serves them (`../cluster.md`, 2).

**The engine-away bound** is jig's worker grace `grace_ms`, temper's
constant (`moved.md`, section 2): how
long a worker keeps hosting its runs with no engine that has admitted
its hello (jig's `domain/hosts/hub.md`, 6; `../../domain/worker/README.md`, section
2). It is 300,000, five minutes
(`../../draft/archive/redeploys.md`, 5.3), so that an engine's restart (its
settled stop, the swap, its store's registry commit and jobs, its switch, its listeners
again) passes within it and each run is adopted where it stands; the
engine's grace is derived above it (below).

**The restart wait** is jig's engine override `fleet.restart_wait_ms`
(jig's `domain/engine/fleet.md`, *A restart said holds its runs*): how
long the engine's settled stop asks its workers to keep their runs after
their links drop, 1,200,000, twenty minutes, in both of jig's profiles
(`../../draft/archive/redeploys.md`, 5.3 and 12), with room over every engine
restart measured so far. `temper-box init` writes none. One setting
serves every worker, each of which keeps its runs for the longer of it
and its own grace (`../../domain/worker/README.md`, section 2); the
engine holds their claims as long at its next start. It bounds a lost
link, not a stop, so the worker's stop bound and its service manager's
stop timeout are unchanged.

- **The drain's bound** is jig's worker member `graces.drain_ms`: how
  long a worker told to stop lets its runs end their turns before it
  cancels them (`../../domain/worker/README.md`, section 2). It may be
  absent, and is then 180,000, the draft's three minutes
  (`../../draft/archive/redeploys.md`, 6.1); `temper-box init` writes none.
- **The drain's grace** is jig's worker member `graces.drain_grace_ms`:
  how long a drained run's calls in flight at its answer have to end
  before its agent cuts them (jig's `domain/hosts/hub.md`, 7). It may be
  absent, and is then 10,000, the draft's ten seconds
  (`../../draft/archive/worker-restarts.md`, section 3); `temper-box init` writes
  none. It runs within the drain's bound, so the stop bound is
  unchanged.
- **It is ordered under the engine's grace.** The stop bound a worker
  states is the longer of its ping interval, its silence deadline and
  this grace together, and its drain's bound; then the longer of
  `agent_stop_ms` and `delivery_ms`, then `save_ms`: 326 seconds with
  jig's link profile (5 and 15 seconds), `temper-box init`'s graces
  (300,000, 5,000, 1,000 and 1,000) and the drain's default, which the
  320 seconds of notice and grace outlast. It is a deadline the
  worker keeps (jig's `domain/hosts/hub.md`, 6), and it must stay below
  the engine's `fleet.grace`, or the hello is refused (jig's
  `domain/engine/fleet.md`): temper derives that grace from the stop
  bound, 360 seconds with jig's link profile (`moved.md`, section 2).
- **The service manager's stop timeout** sits above the stop bound: a
  unit that kills the worker sooner is a crash, losing the workspace's
  changes since its last save or delivery, and the attempt is lost
  (`../../draft/archive/redeploys.md`, 6.1). The worker's unit in `temper help
  server` gives `TimeoutStopSec=` the stop bound and a margin, 360
  seconds with these graces.
- **A hello refused for a charter's reserve names the setting.** The
  fleet judges a worker's checks' timeout against each modifying
  charter's `reserve_time` in the store's settings (jig's
  `domain/engine/fleet.md`). The refusal's words, on the wire and in
  the engine's log, name both: `charter N: needs checks within R ms; the
  agent's profile.policy.check_timeout is O ms; raise charter N's
  reserve_time (temper help settings) or lower the timeout`, within the
  refusal text's 256 bytes.
- **A start judges its own worker against the store,** not the
  document. Standalone, the engine's start checks its worker's agent
  against the configured charters only for the deployment's own bounds
  (a message, an inbox, the turns kept unacknowledged, `limits.md`); a
  charter's turns, time, waiting, checks or models are the store's
  settings' once a store exists, so a shortfall there is printed as a
  warning naming the configuration's path, and the hello's check, which
  names the setting, keeps runs off a worker too small for them.

## 2. The agent's configuration

- **smith's agent configuration** (smith's `protocol/agent.md`, section
  4), a file of smith's that temper writes at each worker's start, from
  the figures its engine sent and what the machine found (`figures.md`,
  section 6). No person writes it, and a worker's document names no
  `agent`. The agent program is derived, in one of two forms:
  - **the box's:** its own executable, with `["internal", "agent",
    <the configuration's path>]`, so each agent process is `temper-box
    internal agent AGENT.json` (../binaries.md);
  - **a worker alone's:** `temper-agent` in its own executable's
    directory (../binaries.md, 3), with `[<the configuration's path>]`,
    so each agent process is `temper-agent AGENT.json`.
- **Written once, pinned:** by the worker's start, `temper-box serve` or
  `temper-worker serve`, which reads it back for smith's host limits,
  the channel's limits and the agent's bounds, as jig's reference binary
  reads it; the start pins those same bytes, with the program, in its
  root (`../../domain/worker/agent-processes.md`, section 1), and each agent
  process reads the pinned copy as it starts. Figures that change, or a
  replaced program, change what the worker's next start pins, never a
  running worker's agents.
- **It holds no credential:** an agent's grants come over smith's
  channel (../credentials-llm-accounts.md, section 1).
- **A model's reasoning allowance,** its `reasoning_item` (smith's
  `protocol/agent.md`, section 4), bounds each reasoning item the model
  sends; past it, with smith's default `oversized_reasoning` of `fail`,
  the call fails and the run holds at once, naming the bound and its
  value (`../../domain/worker/hosted-runs.md`, 1.3). smith derives its largest turn
  from it, so the allowance and the hub's bounds go together: the start
  refuses a document whose bounds do not hold that turn (`limits.md`).
  temper's documents give a plan's model an allowance its reasoning
  does not reach in ordinary use (../setup/init.md, 1), a default a person
  may lower, with the bounds it derives.
- **`programs`** (smith's `protocol/agent.md`, section 4): each program
  the agent may run as a backend, by its adapter, with the program's
  absolute path and its programs directory, which holds a configuration
  directory for each program source, `<directory>/NAME`
  (../credentials-llm-accounts.md, section 1). Its paths are smith's, absolute, but for
  Claude Code's `path` `"fetched"`, temper's pinned build, which temper
  turns into its launcher's absolute path before smith reads the
  document (../setup/owned.md, 6); the
  programs directory `temper-box init` writes is `programs` in the data
  directory (`directories.md`). The owned example's agent configures
  Claude Code's (../setup/init.md, 1). A programs directory need not exist
  until `temper-box program-login` makes it; one that exists must be a
  directory of the user's own with no group or other bits, not reached
  through a link at its name, or the start is refused, naming it
  (skein's `runtime/io/files.md`, 3). A linked parent is accepted. temper reads
  nothing inside it and creates no missing programs directory at serve.
- **The same names as the engine's** hold by construction: the agent's
  endpoints are `engine.llm`'s, under the same names (`figures.md`,
  section 2), so no start compares them.
- **The worker must fit the configured charters.** After the figures checks,
  startup builds jig's core configuration once, expands its instruction files,
  and projects its fleet needs with jig's `fleet_needs::configured`. The
  worker's bounds are its agent configuration's `bounds`, which its
  hello carries unchanged, and jig's `shortfall` selects the first unmet
  requirement. Startup
  names that requirement's configuration path, required and offered amounts,
  with its dimension's units; the same core configuration then checks the
  root. Admission policy stays in jig, including checks fitting within a
  modifying charter's reserve.

### Local disk capacity

A standalone worker may set `TEMPER_WORKER_DISK_BUDGET` to two decimal
byte counts separated by a comma, a reserve and a positive allowance per
occupied place. Before its first placement and every two seconds, it samples
the data directory's filesystem. The number of places is the available bytes
above the reserve divided by the allowance, rounded down and bounded by the
configured slots. A failed sample offers zero places. This is admission
headroom shared by runs and jobs. A process already running can still use
more than its allowance. The worker journals capacity changes. Without the
setting, configured slots govern admission.

## 3. Open questions

- **A terminal approved at a public name** (`worker.md`, 1, clients): Google on the
  web and the between-hosts approval wait for `../../draft/tui.md`, section
  7's flow. Init's `approval` enables the loopback flow beside the
  printed link; it grants no approval on a listener beyond loopback.
