# Deployments

Kind: concept

A deployment is one temper: its engine, the workers that run its agents, the forges it works on, and the three directories that hold its configuration, its data and its cache.

## What it is

- **The engine** keeps state and decides acts. A **worker** checks out
  repositories and runs agents. `temper init` writes one worker in the
  engine's process, with one slot.
- **`temper serve`** runs the configured engine, workers or both.
  **`temper box`** runs serve and each managed forge's setup processes
  on this machine (temper help box). A **server** separates engine and workers (temper help server).

**The three directories:** `CONFIG` holds `temper.json` and
`agent.json`; `DATA` holds the store under `deployment`, workspaces,
forge state, the worker secret and box logs; `CACHE` holds fetched
Forgejo and runner binaries at pinned versions.

Each directory comes from its flag (temper help commands), else
`TEMPER_CONFIG`, `TEMPER_DATA` or `TEMPER_CACHE`, else
`$XDG_CONFIG_HOME/temper`, `$XDG_DATA_HOME/temper` or
`$XDG_CACHE_HOME/temper`. Without those variables the defaults are
`$HOME/.config/temper`, `$HOME/.local/share/temper` and
`$HOME/.cache/temper`. Relative paths start at the working directory.
`DATA` and `CACHE` belong to the user running temper; missing ones
are made with mode 0700.

**`CONFIG/temper.json` has three members:**

- `engine`: limits, model providers, admitted workers, listeners,
  owners, sign-in and first-start projects, each naming `"policy": 1`;
- `workers`: engine addresses, names, secret files, slots and agent programs;
- `application`: managed and fake forge locations, landing rules,
  client and webhook listeners, sign-in, scripted model and application limits.

Forges are made through setup or connect and kept in the store
(temper help forge). Charters, templates, policies and rules are
settings in the store (temper help settings). Writing them into this
configuration is refused; an inline project policy is refused as
`engine.projects[0].policy: name a policy, policy: 1; change it with
temper ctl settings apply`.

## How it changes

- **Start:** serve or box reads configuration, opens `DATA` and restores
  kept state. Each forge is read afresh.
- **Stop:** the first SIGINT or SIGTERM drains work and forge writes,
  closes the store and exits 0. A second signal aborts with exit 1.
- **Restart:** drain the stop, edit configuration, then start with the
  same `CONFIG`, `DATA` and `CACHE`.

## What temper guarantees

- A drained stop keeps state. A run resumes where its worker survives;
  a claim not reported within its grace is lost and retries or holds
  as its charter says (temper help task).
- ctl waits reconnect within the link deadline and resend kept requests
  under their key (temper help ctl).
- Refused configuration changes nothing. A refused start names its
  member, file or short limit and the required size. A refused restore
  names the phase, cause and record family and key when available:
  `temper serve: restore KIND: REASON` or
  `temper serve: restart refused during PHASE: REASON`. Keep `DATA`
  and report that line.

## Acting on it

- **The binary:** `temper init` writes a deployment; `temper box` runs
  one on this machine; `temper serve` runs one, or a worker alone;
  `temper program-login` signs a program in on this host. Each
  command's arguments are its row: temper help commands.
- **ctl:** `me` reads your party in the deployment; `settings show`
  reads how its work runs.

**Offering another model** (for example, `claude-haiku-5-5`):

1. In `CONFIG/temper.json`, append a declaration to `engine.llm.models`:

   ```json
   {
     "name": "claude-haiku-5-5", "endpoint": "anthropic",
     "window": 1048576, "output": 8192,
     "efforts": ["low", "medium", "high"], "effort": "high",
     "prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000}
   }
   ```

   Add `"claude-haiku-5-5"` to `engine.llm.providers[].models` for the
   provider serving it (`anthropic` in this example). Its `models` array
   holds names, not declarations. The declaration's `endpoint` names
   an existing entry in `engine.llm.endpoints`.
2. In `CONFIG/agent.json`, append a matching declaration to
   `endpoints[].models` under the endpoint named `anthropic`:

   ```json
   {
     "name": "claude-haiku-5-5",
     "window": 1048576, "output": 8192,
     "efforts": ["low", "medium", "high"],
     "prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000},
     "reasoning_item": 65536, "head": 60000, "idle": 30000
   }
   ```

   Use the same model and endpoint names and supported `efforts` in both
   files; the engine's `effort` names the default; left out, it is the first.
3. These are illustrative values, not provider specifications. Set
   `window` (context tokens), `output` (maximum output tokens), and
   `prices` (input, cached input and output costs per `unit` tokens)
   to the model's supported values and your deployment's accounting
   units in both files. Set the agent model's `reasoning_item`, the
   byte allowance per reasoning item, and its `head` and `idle`
   deadlines in milliseconds. Larger declarations may require larger
   agent and channel bounds; a refused start names the shortfall.
4. Preserve existing models, providers and endpoint settings, including
   addresses, transports and sign-in settings. Finish both edits
   before the first start. For a running deployment, stop it with
   Ctrl-C or SIGTERM and wait for it to exit before editing, then
   restart with the same `CONFIG`, `DATA` and `CACHE`. `model list`
   reads the models offered to a project; a charter or policy may
   restrict which it can use (temper help charter; temper help settings).

**Writing larger pieces:** `CONFIG/agent.json`'s
`profile.declared.tool_payload` bounds bytes per piece: a write's
content, or an edit's combined old snippets and combined new snippets,
each separately. A 4,487-byte design edit fits the shipped 10,662-byte
profile. This is a deployment setting; changing a charter cannot raise it.

Keep these coupled values in `CONFIG/temper.json` aligned:

- `application.limits.overrides.checkout.message_bytes` holds the payload.
- `workers[0].limits.overrides.hub.event_bytes` equals payload minus 2,
  the message separator: payload `8194` requires event bytes `8192`.
- Both engine and worker overrides `hub.turn_bytes` hold the derived
  agent turn; `hub.turn_queue_bytes` holds that times the agent's turns.
  Engine `core.resume_bytes` holds the turn plus 4; both
  `hub.transcript_bytes` hold that resume window including its framing.
- Application `root.journal.transcript_bytes` and `root.loads` byte
  overrides hold the turn, resume and encoded record carriers.
  Raise every short value startup reports together (temper help limits;
  temper help limits-engine).

For example, `workers[0].limits.overrides.hub.event_bytes: 4096 bytes,
short of agent.json profile.declared.tool_payload less the 2-byte
separator, 8192 bytes` means set that override to `8192`. An event bound
above that figure is refused with its configured value and upper bound;
set it to the same required figure.

A payload of `16386` can instead fail at the model:
`input-answer: llm.input[endpoint=1]=99876; llm.answer[endpoint=1]=65536`.
The encoded input must fit the model's answer capacity. Reduce the payload,
or declare a supported larger model `output` in both files; this example
needs at least 12,485 output tokens (eight answer bytes per token).
Do not lower a receiving check. Recheck all derived carriers after that
change. Drain the stop, edit both files, and restart with the same
`CONFIG`, `DATA` and `CACHE`; running agents retain their startup profile.

## Limits

For the configuration members that need a restart, read `temper help configuration`.

- temper reads its configuration at a start alone: a change to the
  document takes effect at the next start.
- temper has no command that upgrades a deployment in place or prints
  its versions.
- temper has no ctl group `deployment`: `me` and `settings show` are
  what ctl reads of one.
- A box on fake backends keeps a deployment of its own under
  `DATA/box/fake-backends`, and refuses a `CONFIG` changed after
  `temper init --box`.
