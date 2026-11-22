# The workers' figures

Provisional, 2026-11-16. What a worker takes from its engine rather than
from its document, and how temper composes, sends and applies it: the
hub's, the checkout's and the git component's limits, the places it's
admitted at, and its agents' profile and endpoints. It also covers the
agent configuration temper writes from them, the `agents` setting, and
what a worker's document no longer holds. It carries
`../../draft/config.md`, 6.3, and, from 2026-11-20, the figures a running
worker takes live (`../../draft/worker-restarts.md`, 8). The record and
the two openings that carry it are jig's (jig's
`protocol/channel/figures.md`).

## 1. In one page

- **The engine composes the figures** (2) from its own limits, the
  endpoints of the LLM providers it runs (`providers.md`, 3) and the
  `agents` setting (5). It does so
  after its store's restore, at each change of that setting, and
  whenever it takes limits at once (`limit-settings.md`, 4), and gives
  them to its workers host. The host sends them at each opening and to
  every linked worker.
- **A worker alone takes them at each start** (3.1). It opens for its
  figures alone, then builds from them: its hub, its checkout and git
  component, and its agents' pinned configuration. Only then does it
  open its link.
- **A running worker takes a live figure at its next use** (3.2): a time
  bound or a ceiling that every holder reads at each use, such as
  `checkout.remote_timeout_ms`. What sizes a table, what its link agreed
  at its opening, its places and the `agents` setting, which re-pins its
  agent, wait for its next start. No run is cut: an operation in flight
  keeps the bound it began with, and the next one takes the new figure.
- **The box builds its worker from its own engine's figures** (4),
  before its store opens, with the `agents` setting as its last run kept
  it.
- **temper writes the agent configuration** (6) from the figures and
  what the machine found: its programs and its environment. No person
  writes `agent.json`.
- **A worker's document loses its copies** (7). Its `limits`, its
  `agent`, a worker alone's `application.limits` and an `agent.json`
  are each refused by name.
- **Breaking.** `jig-worker-wire` 16 is declared breaking, so an engine
  and its workers redeploy together (8).

## 2. What the engine sends

jig's `Figures` record (jig's `protocol/channel/figures.md`, section 2),
filled so:

| Member | From |
|---|---|
| `places` | jig's, for each worker as it's sent |
| `limits` | every limit of jig's that the engine holds, and every one of temper's (`limits.md`), each by its dotted name as an override names it, at its figure in force (`limit-settings.md`, 4): the value the engine started with, or the one it has taken at once since |
| `agents.profile` | the `agents` setting's profile figures (5), by the names smith's profile takes: `declared.NAME`, `policy.NAME` |
| `agents.endpoints` | each endpoint of the providers the engine runs (`providers.md`, 3), in their providers' order and each provider's: its name, number, dialect, provider, program, address, transport, headers and deadlines as its provider's setting holds them, and its limits the `agents` setting's for its name, else the setting's `fallback` |

- **One function composes them,**
  `temper_engine_service_vocabulary::figures::compose`. It takes the
  engine's typed configuration (`settings::Engine`, jig's engine part
  with temper's limits) and the `agents` setting's value, and returns
  the generated `Figures` with `places` 0. The engine calls it, and so
  does the box for its own worker (4), so both build the same figures.
- **The composer keeps the limits in force.** The engine's workers keep
  a `Composer`, made from the start's limits. When the engine takes
  limits at once, it gives the composer the figures now in force,
  `Composer::take_limits(&mut self, jig: &config::Limits, own:
  &limits::Limits)`, which replaces its limits by name and keeps its
  endpoints. `compose` is unchanged, so a later change of the `agents`
  setting composes with the limits in force too.
- **At most 64 endpoints.** `Agents.endpoints` holds 64, and jig's
  `provider` kind holds no more across its providers (jig's
  `domain/engine/llm-providers.md`, 4); temper's start checks still
  refuse running providers with more, naming where they came from:
  `providers.setting: N endpoints, at most 64, the workers' figures carry
  no more`. The check runs before `compose`, so `compose` stays total.
- **Limits by name both ways.** Each limits module has a list of its
  limits by name and value: jig's in `jig-protocol-config`, temper's in
  `temper-service-core`. Each is the reverse of that module's override
  reader, so a focused test holds that applying a list to any stock
  gives back the limits it was made from.
- **When.** The engine composes once its restore has read the `agents`
  setting, again at each change of it, and again whenever it takes
  limits at once (`limit-settings.md`, 4), then with the `agents` value
  it last composed. It gives each result whole to its workers host,
  which sends it to every linked worker. Until the first, the host
  refuses a figures opening as *starting*, so a worker that starts with
  its engine waits for the restore. A limit taken only at the next start
  changes no figure, so it sends nothing.
- **Logged.** At each composition, the engine's log says
  `figures for workers: L limits, E endpoints, agents setting revision
  R`; at one for limits taken at once, `figures for workers: L limits, E
  endpoints, limit settings revision S`.

## 3. A worker's figures

### 3.1 A worker alone's start

`temper-worker serve`, in order:

1. **Its file,** `temper.toml`, read and checked (`file.md`, section
   6), and the start's document it gives read as `README.md`, section 2
   reads it.
2. **Its figures.** It runs its link's figures form on its own, until
   the link hears the figures (jig's `protocol/channel/figures.md`,
   section 4). A refusal is logged as any opening's is (`../wires.md`,
   section 4), *starting* among them, and the link redials after its
   pause. Once they're heard, the log says `figures taken from ENGINE:
   P places, L limits, E endpoints`.
3. **Applied.** jig's limits are the stock's with each figure applied by
   name. So are temper's application limits, the checkout's and the git
   component's among them. A name neither knows is skipped and logged
   once. Its places are the figures' places, its setting's alone: its
   hub's slots, its checkout's workspaces and its git component's
   operations are those places (`../../draft/config.md`, section 7). The
   start's document's `slots` gives none (`file.md`, section 4), and no
   check of it bounds them; the checks that involve a worker's places
   run on the places it's built with.
4. **The agent configuration** is written from the figures and the
   machine (6), and pinned in the worker's root as today's file is
   (`../../domain/worker/agent-processes.md`, section 1).
5. **Checked and built** as today's start does from its document and
   agent configuration (`README.md`, sections 2 to 4). A problem refuses
   the start, naming the figure and saying the engine sent it, such as
   `the engine's figures: hub.turn_bytes 1048576 is below the agent's
   turn, 18431842`.
6. **Its link** opens with `opening: link`. At this opening, or later,
   it may hear figures other than those it was built from. It takes
   them as 3.2 has it.

- **Its own link's limits** come from the figures too. Only the figures
  form, which runs before them, uses the link's stock.
- **A worker started before its engine** waits in step 2, as a link
  waits for its engine today. It holds no run until it's built.

### 3.2 While it runs

- **What it hears.** Its link reports each `figures` it hears (jig's
  `protocol/channel/figures.md`, 4): at each opening, and whenever the
  engine composes them again (2), for limits taken at once or a change
  of the `agents` setting. A record equal to the last one heard is
  quiet.
- **Live or at its next start.** Each figure a worker reads is one or
  the other, as its limit's row says (`limit-settings.md`, 5.2 and 5.3):
  - **Live:** a time bound or a ceiling that every holder reads at each
    use, and that sizes no table, no carrier and nothing its link agreed
    at its opening. The worker's are the checkout's timeouts, its reds
    read and its reruns, its preparations and commit message bound, the
    git component's output, read and sweep bounds, the link's handshake,
    stall, backoff, ceiling, proved and busy timers, and the hub's
    event, accounts and turn queue ceilings: `limit-settings.md`, 5.2,
    lists them by name.
  - **At its next start:** every other figure it reads. Its places (its
    hub's slots, workspaces and git operations) and every figure that
    sizes a table or one of its link's carriers, `checkout.repositories`
    and the hub's turn among them; `io.output`, which its link's opening
    states; `link.ping_ms` and `link.silence_ms`, which make the stop
    bound its hello states, which the engine checks against its grace
    and holds its runs by (`../../draft/redeploys.md`, 5.3); and the
    `agents` profile and endpoints, which re-pin its agent
    (`../../draft/redeploys.md`, 4.4).
- **How it takes them,** on figures that differ from those in force on
  it:
  1. It reads them over the stock as its start does (3.1, step 3),
     keeping its places.
  2. It finds, by name, each figure that moved against those in force
     on it.
  3. It sets each live figure that moved over a copy of the limits in
     force on it, and runs its start's checks over the copy (3.1, step
     5): the hub's, the checkout's, the git component's and the link's,
     with its pinned agent's.
  4. Passing, it takes them all together. The checkout and the hub read
     theirs from the worker's environment at their next step; the git
     component and the link take theirs through an input of their own
     (jig's `protocol/channel/figures.md`, 4, for the link). Refused, it
     takes none of them, and runs on as it was.
  5. Every other figure that moved waits for its next start.
- **At the next operation that reads it.** Each holder reads its figure
  as an operation begins and keeps it for that operation: a git
  operation's deadline is set as it's sent, from
  `checkout.remote_timeout_ms` or `checkout.local_timeout_ms`; a reds
  read's as it starts; a git process's output bounds as it's spawned;
  the link's handshake at its next dial and its backoff at its next
  wait. An operation in flight keeps what it began with, and the next
  one takes the new figure.
- **No run is cut.** Taking a figure never cancels, drains, saves or
  ends a run or a job, and sends no hello: the hello's slots, stop bound
  and bounds are all figures that wait for the next start.
- **Its figures in force** are those it was built from, with each figure
  it took over them, and it compares later figures with them. A worker
  that restarts takes the engine's figures in force (2), which hold
  every figure it took.
- **Logged,** once for each change, a line for each part of it, each
  naming at most 8 figures by name and figure, then `and N more`:
  - `the engine's figures changed; taken live: checkout.remote_timeout_ms 600000, checkout.local_timeout_ms 20000`
  - `the engine's figures changed; at this worker's next start: hub.turn_bytes 2097152`
  - `the engine's figures changed; not taken live: PROBLEM; they apply at this worker's next start`,
    PROBLEM the first problem of the checks in its start's words (3.1,
    step 5), such as `the engine's figures: git_component.stdout_bytes
    1024 is below the checkout's largest output, 70000`.
- **The box's worker** takes them the same way, through its in-process
  link (4).

## 4. The box

- **Its worker's figures are its engine's,** composed by the same
  `compose` (2) from the box's own configuration. They're composed
  before the box's store opens, so the `agents` setting's value comes
  from `agents.setting` in the data directory, the setting's value as
  the box last kept it. With no such file, the setting's default is
  used, which a new store holds. A file that doesn't decode is logged,
  and the default is used.
- **The box keeps the setting:** its engine writes `agents.setting`
  whenever the setting changes, and after each restore where the file
  isn't the store's value. The file is the value's bytes, written whole
  and renamed into place. So a change applies at the box's next start,
  as at every worker's.
- **Its places are its setting `local`'s** (`../cluster.md`, section
  1), 4 by default. The box builds its worker before its store opens,
  so it keeps that setting in its data directory as it keeps
  `agents.setting`: `local.setting`, the setting's value, written by
  its engine whenever the setting changes and after each restore where
  the file isn't the store's value, whole and renamed into place. Its
  start builds its worker at the file's places, its hub's slots,
  workspaces and git operations sized so; with no file, the default's
  4; a file that doesn't decode is logged, and the default is used. Its
  engine admits it as it admits every worker, at the smaller of its
  setting's places and its hello's (`../../draft/config.md`, 6.7). So a
  change of `local`'s places applies at the box's next start. The
  start's document's `slots` gives none (`file.md`, section 4).
- **It isn't built at `fleet.slots`:** a worker's own link caps its
  output at a u32, its places times a run's turns, calls and asks, and
  at the stock's 8 turns of 18,431,842 bytes a place passes 147 MB, so
  32 places pass the cap and a box built at them couldn't start
  (`derived.md`, 5.3).
- **Its in-process link** hears its figures as any worker's does, and
  its worker takes them as any running worker does (3.2).
- **Its agent configuration** is written and pinned as a worker alone's
  is (6), from its own `application.workers` entry.

## 5. The `agents` setting

- **Kind 21,** one of temper's (`settings.md`, section 2): the
  deployment's scope, number 1, decided by the deployment's owners. It
  applies at each worker's next start, since it re-pins the worker's
  agent (`../../draft/redeploys.md`, 4.4). Kind 20 remains the deployment's welcome; kinds 16 to 19 are left for the draft's other milestones.
- **Its value,** a record of the client's wire:

  ```
  record AgentFigure { name: text 64 value: u64 }
  enum Oversized { fail, drop }
  record AgentLimits { window: u32 output: u32 reasoning_item: u32 head_ms: u64 idle_ms: u64 oversized: Oversized }
  record AgentEndpointLimits { endpoint: text 64 limits: AgentLimits }
  record AgentFigures { profile: list 128 AgentFigure endpoints: list 64 AgentEndpointLimits fallback: AgentLimits }
  ```

  `profile` is smith's profile figures by name over smith's `standard`.
  `endpoints` is the limits for each endpoint, by its name among the
  providers' endpoints. `fallback` is the limits of an endpoint the value
  doesn't name.
- **Its default is today's stock agent configuration:** the figures
  `temper-box init` writes in `agent.json` (`../setup/init.md`, 3). That
  is the profile's eleven, the four endpoints' limits (`openai`,
  `anthropic`, `anthropic-api` and `demo`), and `openai`'s limits as
  the fallback.
- **Its judge** refuses a value naming the item:
  - a profile name that isn't one of smith's declared or policy
    figures, or a value past its type;
  - a name given twice;
  - an endpoint named twice;
  - a window, output, reasoning item, head or idle of 0.

  It doesn't check the turn the profile derives against the engine's
  carriers. A worker's start refuses figures it can't run, naming them
  (3.1, step 5).
- **People change it** as any of temper's kinds: `temper ctl settings
  apply` with `{"kind": "agents", "profile": {"declared.memory": N, ...},
  "endpoints": {"openai": {"window": N, "output": N, "reasoning_item": N,
  "head_ms": N, "idle_ms": N, "oversized": "fail"}, ...}, "fallback":
  {...}}`. `settings show agents` reads it, or the stock's, marked so,
  and Ask temper proposes it as it does any kind.

## 6. The agent configuration temper writes

smith's agent configuration (smith's `protocol/agent.md`, section 4),
written as JSON by the worker's start:

```
{
  "profile": {"name": "standard", "declared": {NAME: VALUE, ...}, "policy": {NAME: VALUE, ...}},
  "endpoints": [ENDPOINT, ...],
  "programs": {"claude_code": {"path": CLAUDE, "directory": PROGRAMS}},
  "environment": [{"name": NAME, "value": VALUE}, ...]
}
```

- **`profile`** is the figures' `agents.profile`, each name split at its
  first dot into its group and its member.
- **`endpoints`** is each of the figures' endpoints in smith's form: its
  `address` as `HOST:PORT`, its `connect` and `handshake` in
  milliseconds, its `transport`, `server_name` and `trust`, its
  `headers`, its `program` when it names one, and its `limits`, with
  `head` and `idle` in milliseconds and `oversized_reasoning`. Smith's
  protocol provider is `openai` for dialect 3 and `anthropic` for dialect 2;
  an engine-local provider name such as `demo` remains in the figures.
- **`programs.claude_code.path`** is the file's `[programs]` `claude`
  where it names one (`file.md`, 3.1), else the `claude` found on the
  worker process's own `PATH` at start, or `"fetched"` with none
  (`../setup/owned.md`, 6). Its `directory` is `programs` in the data
  directory. Likewise the worker's git is `[programs]` `git` where the
  file names it, else the `git` found on `PATH` (`moved.md`, section
  2), checked as the found one is.
- **`environment`** is the agents' environment: the variables smith
  gives each program it runs for an agent, once it has cleared its own,
  Claude Code, a shell command and the checkout's check alike. It's the
  worker's own, never the engine's or a setting's
  (`../../draft/config.md`, 6.3), from the document's
  `application.environment` (`README.md`, section 3) and the worker
  process's own environment, by one rule:
  - **Absent,** it's the worker process's own `HOME`, then its own
    `PATH`, each where the process has it. A deployment that writes
    nothing still has agents that find what the worker's `PATH` finds,
    `cargo` among it.
  - **Given,** it's exactly the document's entries, in their order,
    then the worker's own `HOME` where the list names no `HOME`, then
    its own `PATH` where the list names no `PATH`. An empty list is as
    absent. No other variable of the worker's process reaches the
    agents: a deployment gives the rest in the list.
  - **Example:** a worker whose own `HOME` is `/home/w` and `PATH`
    `/usr/bin:/bin`, with `[{"name": "CARGO_TARGET_DIR", "value":
    "/srv/target"}, {"name": "PATH", "value": "/opt/cargo/bin:/usr/bin"}]`,
    gives `CARGO_TARGET_DIR=/srv/target`, `PATH=/opt/cargo/bin:/usr/bin`
    and `HOME=/home/w`, in that order; with no list, `HOME=/home/w` and
    `PATH=/usr/bin:/bin`.
  - **Every start takes it:** a worker alone's and the box's, through
    the one rule (`temper-worker-service`'s
    `startup::worker::application::agents_environment`), into the
    worker part's `environment` that this configuration is written from.
    The process's environment reaches that rule as its shell captured
    it, as the shell's `git` does; the rule reads nothing else of it.

- **Pinned as today's file is:** `agent/agent.json` under the worker's
  root, private, with the agent program copied beside it, both afresh at
  each start (`../../domain/worker/agent-processes.md`, section 1). smith's
  decoder reads the bytes back before the start goes on, so smith's
  checks hold them.
- **The agent program is temper's own,** derived and never written. The
  box's is its own executable, with `internal agent PATH`. A worker
  alone's is `temper-agent` in its own executable's directory, with
  `PATH` (`../binaries.md`, 3). If that isn't an executable file, the
  start is refused, naming the path.

## 7. What a worker's document loses

| Member | Refused with |
|---|---|
| `workers[0].limits` | `workers[0].limits: moved: a worker takes its limits from its engine; remove it` |
| `workers[0].agent` | `workers[0].agent: moved: temper writes the agent's configuration from its engine's figures; remove it` |
| a worker alone's `application.limits` | `application.limits: moved: a worker takes its limits from its engine; remove it` |
| `agent.json` in the configuration directory | `agent.json: moved: temper writes the agent's configuration under the worker's root; remove it` |

- **The box keeps `application.limits`,** since they're its engine's.
  It loses its worker part's `limits` and `agent`, and its `agent.json`,
  as a worker alone does.
- **Machine facts follow `moved.md`.** A worker's workspace is
  `workspaces` under its data directory, its git is found on `PATH`,
  and its graces and source-login interval are temper's constants.
  No `application.workers` entry is written.

- **`temper-box init`** writes neither the worker's `limits` and
  `agent` nor `agent.json` (`../setup/init.md`, 2 and 3).

## 8. Upgrades

- **`jig-worker-wire` 16 is breaking.** An engine and a worker of
  different floors refuse each other by version, so dev.fabrig.ai's
  engine and its workers redeploy together. Before the redeploy, each
  worker's document loses its limits and `agent`, and its `agent.json`
  goes (`../../draft/config.execution.md`, N3).
- **No store patch.** Kind 21 starts empty, over its default.
- **A worker on another host reaches the endpoints its engine runs,**
  as the copies it kept did before: an endpoint's address is the one its
  agents dial from their worker's host (jig's
  `domain/engine/llm-providers.md`, 2).
- **Figures taken live move no worker wire.** The engine already sends
  `figures` to every linked worker whenever its owner gives new ones
  (jig's `protocol/channel/figures.md`, 3). A worker of an earlier build
  logs them and runs on as it was built; a worker of this build under an
  engine of an earlier build hears changed figures only at a change of
  kind 21, and takes the live ones it holds. So an engine and its
  workers redeploy in either order, and no store or setting changes
  form.
- **`temper-client-wire` moves its version** for the Limits page's
  word for a live limit (`limit-settings.md`, 7; `../wires.md`, 9).
- **The agents' environment moves no version.** `application.environment`
  is a member the document gains, absent by default, and the pinned
  configuration keeps smith's form. A deployment whose agents need no
  more than the worker's own `HOME` and `PATH` takes no step. One whose
  agents need more, as dev.fabrig.ai's do, moves the `environment`
  entries of the `agent.json` it removed (section 7) into each worker's
  document, as `application.environment`, before that worker restarts.

## 9. Testing

Focused and fuzzy alone. The process tier's stories are written and
compiled, and run after landing.

- **`compose`:** a configuration and an `agents` value give figures
  holding every limit by name, the setting's profile, and each endpoint
  with its limits, or the fallback for one the value doesn't name. The
  limits lists apply back to the limits they were made from.
- **The engine:** once restored, it gives its host figures, and a
  worker's figures opening hears them. Before that, the opening is
  refused *starting*. A change of kind 21 reaches every linked worker.
- **The agent configuration:** figures of the stock and a machine with
  no `claude` give bytes that smith's decoder reads as today's
  `bare-agent.json` reads, with `"fetched"` as its path.
- **The agents' environment,** focused, in `temper-worker-service`: a
  worker alone built from its engine's figures with no
  `application.environment` gives its worker part the process's own
  `HOME` and `PATH` and nothing else of it; with two entries, those two,
  then `HOME` and `PATH` where the list omits them; an entry past its
  bounds, or a repeated name, is refused, naming the entry. The box's
  start gives the same, by the same rule.
- **An agent's command finds `PATH`'s programs:** in the real loop, a
  worker alone whose own `PATH` alone holds a program runs a scripted
  agent's shell command naming that program by its name, and the
  command's output reaches the agent.
- **A worker alone:** in the real loop, an engine alone and two workers
  alone, each with a document without `limits` or `agent`, take their
  figures and run a task each to its end (`../cluster.md`, section 6).
  A worker that hears changed figures takes its live ones and runs on.
- **Taken live,** focused, in `temper-worker-service`'s own tests: a
  worker built from figures, with two runs in flight, one of them in a
  git operation that reaches the forge, hears figures with
  `checkout.remote_timeout_ms` raised from 60,000 to 600,000. The
  operation in flight keeps its deadline; the next git operation of the
  other run is sent with the new one; no run is cancelled, drained,
  saved or ended, and no hello is sent again
  (`../../draft/worker-restarts.md`, 9, test 7). A lowering is taken the
  same way.
- **The rows and the worker agree,** focused, in the same tests: for
  each name of either names table that a worker reads, figures with that
  figure alone moved are taken live exactly when its row says live, and
  logged as waiting for the next start otherwise. A live figure the
  checks refuse is taken with none of the others moved with it, and
  logged in its start's words.
- **Composed at a take,** focused, in the engine's service: a durable
  revision moving a live figure reaches every linked worker as figures
  holding it, logged with its revision; one moving only figures taken at
  the next start sends none; a mixed one sends its live figures alone;
  with `limits-held`, none.
- **Refused by name:** each member of 7, and an `agent.json`.
- **The box:** a start with `agents.setting` builds its worker from it,
  and a change of kind 21 writes the file.
- **Kind 21:** applied, shown and refused as section 5 gives it.

## 10. Open questions

- **The judge and the carriers.** Kind 21's judge could refuse a profile
  whose turn the engine's carriers can't hold, rather than leaving it to
  a worker's start. That waits for limits as settings
  (`../../draft/config.md`, 6.1), when the carriers move with the
  setting.
