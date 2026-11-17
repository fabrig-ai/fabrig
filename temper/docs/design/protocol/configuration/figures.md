# The workers' figures

Provisional, 2026-11-16. What a worker takes from its engine rather than
from its document, and how temper composes, sends and applies it: the
hub's, the checkout's and the git component's limits, the places it's
admitted at, and its agents' profile and endpoints. It also covers the
agent configuration temper writes from them, the `agents` setting, and
what a worker's document no longer holds. It carries
`../../draft/config.md`, 6.3. The record and the two openings that carry
it are jig's (jig's `protocol/channel/figures.md`).

## 1. In one page

- **The engine composes the figures** (2) from its own limits, its
  `engine.llm`'s endpoints and the `agents` setting (5). It does so
  after its store's restore and at each change of that setting, and
  gives them to its workers host. The host sends them at each opening.
- **A worker alone takes them at each start** (3). It opens for its
  figures alone, then builds from them: its hub, its checkout and git
  component, and its agents' pinned configuration. Only then does it
  open its link. A change reaches it at its next start.
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
| `limits` | every limit of jig's that the engine holds, and every one of temper's (`limits.md`), each by its dotted name as an override names it, at the value the engine started with |
| `agents.profile` | the `agents` setting's profile figures (5), by the names smith's profile takes: `declared.NAME`, `policy.NAME` |
| `agents.endpoints` | each of `engine.llm`'s endpoints, in its order (`README.md`, section 2): its name, number, dialect, provider, program, address, transport, headers and deadlines as `engine.llm` holds them, and its limits the `agents` setting's for its name, else the setting's `fallback` |

- **One function composes them,**
  `temper_engine_service_vocabulary::figures::compose`. It takes the
  engine's typed configuration (`settings::Engine`, jig's engine part
  with temper's limits) and the `agents` setting's value, and returns
  the generated `Figures` with `places` 0. The engine calls it, and so
  does the box for its own worker (4), so both build the same figures.
- **At most 64 endpoints.** `Agents.endpoints` holds 64, so temper's
  start checks refuse an `engine.llm` with more, naming it:
  `engine.llm: N endpoints, at most 64, the workers' figures carry no
  more`. jig's own reader keeps its bound; the check is temper's, run
  before `compose`, so `compose` stays total.
- **Limits by name both ways.** Each limits module has a list of its
  limits by name and value: jig's in `jig-protocol-config`, temper's in
  `temper-service-core`. Each is the reverse of that module's override
  reader, so a focused test holds that applying a list to any stock
  gives back the limits it was made from.
- **When.** The engine composes once its restore has read the `agents`
  setting, and again at each change of it. It gives each result whole to
  its workers host. Until the first, the host refuses a figures opening
  as *starting*, so a worker that starts with its engine waits for the
  restore.
- **Logged.** At each composition, the engine's log says
  `figures for workers: L limits, E endpoints, agents setting revision
  R`.

## 3. A worker alone's start

`temper-worker serve`, in order:

1. **Its document,** `temper.json`, read and checked as today
   (`README.md`, section 2), without the members that moved (7).
2. **Its figures.** It runs its link's figures form on its own, until
   the link hears the figures (jig's `protocol/channel/figures.md`,
   section 4). A refusal is logged as any opening's is (`../wires.md`,
   section 4), *starting* among them, and the link redials after its
   pause. Once they're heard, the log says `figures taken from ENGINE:
   P places, L limits, E endpoints`.
3. **Applied.** jig's limits are the stock's with each figure applied by
   name. So are temper's application limits, the checkout's and the git
   component's among them. A name neither knows is skipped and logged
   once. The hub's slots are the smaller of its worker part's `slots`
   and the figures' places.
4. **The agent configuration** is written from the figures and the
   machine (6), and pinned in the worker's root as today's file is
   (`../../domain/worker/README.md`, section 5).
5. **Checked and built** as today's start does from its document and
   agent configuration (`README.md`, sections 2 to 4). A problem refuses
   the start, naming the figure and saying the engine sent it, such as
   `the engine's figures: hub.turn_bytes 1048576 is below the agent's
   turn, 18431842`.
6. **Its link** opens with `opening: link`. At this opening, or later,
   it may hear figures other than those it was built from. It then logs
   once, for each change, `the engine's figures changed; they apply at
   this worker's next start`, and runs on as it was built.

- **Its own link's limits** come from the figures too. Only the figures
  form, which runs before them, uses the link's stock.
- **A worker started before its engine** waits in step 2, as a link
  waits for its engine today. It holds no run until it's built.

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
- **Its in-process link** hears its figures as any worker's does. One
  that differs is logged the same way (3, step 6).
- **Its agent configuration** is written and pinned as a worker alone's
  is (6), from its own `application.workers` entry.

## 5. The `agents` setting

- **Kind 21,** one of temper's (`settings.md`, section 2): the
  deployment's scope, number 1, decided by the deployment's owners. It
  applies at each worker's next start. Kind 20 remains the deployment's welcome; kinds 16 to 19 are left for the draft's other milestones.
- **Its value,** a record of the client's wire:

  ```
  record AgentFigure { name: text 64 value: u64 }
  enum Oversized { fail, drop }
  record AgentLimits { window: u32 output: u32 reasoning_item: u32 head_ms: u64 idle_ms: u64 oversized: Oversized }
  record AgentEndpointLimits { endpoint: text 64 limits: AgentLimits }
  record AgentFigures { profile: list 128 AgentFigure endpoints: list 64 AgentEndpointLimits fallback: AgentLimits }
  ```

  `profile` is smith's profile figures by name over smith's `standard`.
  `endpoints` is the limits for each endpoint, by its name in
  `engine.llm`. `fallback` is the limits of an endpoint the value
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
  (3, step 5).
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
- **`programs.claude_code.path`** is the `claude` found on
  `PATH` at start, or `"fetched"` with none (`../setup/owned.md`, 6).
  Its `directory` is `programs` in the data directory.
- **`environment`** is the machine facts supplied to the worker's
  typed start, empty when none are supplied.

- **Pinned as today's file is:** `agent/agent.json` under the worker's
  root, private, with the agent program copied beside it, both afresh at
  each start (`../../domain/worker/README.md`, section 5). smith's
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
- **A worker on another host reaches the endpoints `engine.llm` names,**
  as the copies it kept did before.

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
- **A worker alone:** in the real loop, an engine alone and two workers
  alone, each with a document without `limits` or `agent`, take their
  figures and run a task each to its end (`../cluster.md`, section 6).
  A worker that hears changed figures logs them and runs on.
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
- **An endpoint seen from another host.** A worker uses `engine.llm`'s
  addresses. An endpoint the engine reaches on its loopback doesn't
  serve a worker on another host. That waits for providers in the store
  (`../../draft/config.md`, 6.2).
