# Deployments

Kind: concept

A deployment is one temper: its engine, the workers that run its agents, the forges it works on, and the three directories that hold its configuration, its data and its cache.

## What it is

- **The engine** keeps state and decides acts from ctl, clients and runs.
- **A worker** checks repositories out, runs agents and carries their
  calls to the engine. Init writes one worker with one slot.
- **`temper serve`** runs the configured engine and workers together,
  the engine alone, or a worker on another host.
- **A box** runs `temper serve` and forge setup on one machine
  (temper help box). A server runs behind a proxy, with workers on
  other hosts (temper help server).

**The directories:** `CONFIG` holds `temper.json` and `agent.json`,
written by init. `DATA` keeps the store in `DATA/deployment`, worker
workspaces, owned Forgejo data, worker secrets and `DATA/box/logs`.
`CACHE` keeps the pinned Forgejo and runner binaries.

Each directory comes from its flag, else `TEMPER_CONFIG`, `TEMPER_DATA`
or `TEMPER_CACHE`, else its XDG directory under `temper`: `$XDG_CONFIG_HOME`,
`$XDG_DATA_HOME` or `$XDG_CACHE_HOME`, defaulting to `$HOME/.config`,
`$HOME/.local/share` or `$HOME/.cache` (temper help commands).
Relative paths start at the working directory. `DATA` and `CACHE`
belong to the user running temper, with no group or other access;
temper creates missing ones with mode 0700.

**`CONFIG/temper.json`** has three members:

- `engine`: limits, LLM providers and models, workers and listener,
  owners, sign-in and initial projects, each naming its policy
  (`"policy": 1`); init seeds no projects;
- `workers`: engine address, name, secret file, slots and agent program;
- `application`: owned and fake forges, GitHub, landing rules,
  repository gates, forge charter, listeners, Google sign-in,
  scripted model and application limits.

Forges are created by setup or connection and kept in the store
(temper help forge). Settings (charters, templates, policies and rules)
also live in the store (temper help settings). Configuration that
names settings or writes out a project's policy is refused:
`engine.projects[0].policy: name a policy, policy: 1; change it with
temper ctl settings apply`.

## How it changes

- **Started:** `temper serve` or `temper box` reads configuration and
  opens the store. Existing tasks, accounts, parties, settings and forge
  records are restored; each forge is read afresh.
- **Stopped:** the first SIGINT or SIGTERM stops admitting calls,
  settles work and forge writes, closes the store and exits 0.
  A second signal aborts with exit 1.
- **Restarted:** stop, edit configuration, then start with the same
  three directories.

## What temper guarantees

- **A drained stop keeps everything:** the same `DATA` resumes it.
- **A run outlives a restart where its worker does:** the engine
  reclaims it. An unreported claim is lost after its grace; the task
  retries or holds as its charter says (temper help task).
- **ctl rides a restart:** waits reconnect within the link's deadline;
  kept requests are resent under their keys (temper help ctl).
- **A refused start says why:** it names a member or file at fault,
  such as a secret readable by others, or a restore failure:
  `temper serve: restore KIND: REASON` or
  `temper serve: restart refused during PHASE: REASON`. The reason
  names the store's cause, record family and key when available.
  Limits below stock needs are refused before reading the store,
  naming the override and required size. Refused configuration changes
  nothing; refused stored state is a finding: keep `DATA` and report
  the line.

## Acting on it

- **The binary:** `temper init` writes a deployment; `temper box` runs
  one on this machine; `temper serve` runs one, or a worker alone;
  `temper program-login` signs a program in on this host. Each
  command's arguments are its row: temper help commands.
- **ctl:** `me` reads your party in the deployment; `settings show`
  reads how its work runs.

**Offering another model** (for example, `gpt-6.1-sol`):

1. In `CONFIG/temper.json`, append a declaration to `engine.llm.models`:

   ```json
   {
     "name": "gpt-6.1-sol", "endpoint": "openai",
     "window": 1048576, "output": 8192,
     "efforts": ["high", "medium", "low"],
     "prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000}
   }
   ```

   Add `"gpt-6.1-sol"` to `engine.llm.providers[].models` for its
   provider (`openai` here): names, not declarations. The `endpoint`
   names an existing `engine.llm.endpoints` entry.
2. In `CONFIG/agent.json`, append a matching declaration to
   `endpoints[].models` under the endpoint named `openai`:

   ```json
   {
     "name": "gpt-6.1-sol",
     "window": 1048576, "output": 8192,
     "efforts": ["high", "medium", "low"],
     "prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000},
     "reasoning_item": 65536, "head": 60000, "idle": 30000
   }
   ```

   Use the same model and endpoint names in both files, with supported
   `efforts`; the engine's first effort is the default.
3. These values are illustrative, not provider specifications. Set
   `window` (context tokens), `output` (output tokens) and `prices`
   (costs per `unit` tokens) in both files. Set the agent's
   `reasoning_item` (bytes per reasoning item), `head` and `idle`
   (milliseconds). Larger declarations may require larger receiving
   bounds; a refused start names the shortfall.
4. Preserve existing models, providers, endpoints, transports and
   sign-in settings. Finish both edits before the first start. For a
   running deployment, stop with Ctrl-C or SIGTERM and wait for exit;
   restart with the same `CONFIG`, `DATA` and `CACHE`. `model list`
   reads a project's models; charters and policies may restrict them
   (temper help charter; temper help settings).

**A model answer's allowance:** in `CONFIG/agent.json`,
`profile.declared.calls_per_response` derives `2*calls+2` completion
items: 48 calls gives 98 items. This counts the whole answer,
including reasoning and text, not just tool calls. Raising this value
or a model's `reasoning_item` increases the largest turn and its
receiving bounds.

**Recovering from an answer-limit hold:**

1. Read `task show TASK` for its reason and hold revision. Stop the box
   with Ctrl-C or SIGTERM and wait for it to exit.
2. Edit `CONFIG/agent.json` and the coupled receiving limits in
   `CONFIG/temper.json` together. The engine and worker need
   `hub.turn_bytes`, `hub.turn_queue_bytes` and `hub.transcript_bytes`;
   the engine also needs `core.resume_bytes`; the application needs
   `root.journal.transcript_bytes`, `root.loads.transcript_bytes`,
   `root.loads.bytes`, `root.loads.reply_bytes` and `root.loads.read_bytes`.
   A refused start names the shortfall. Byte overrides cannot bypass the
   selected profile's store page ceiling: `standard`, which init selects,
   has 59,570,691 bytes; `small` has 2,168,787 bytes. Reduce the calls or
   model allowances, or select a compatible existing profile with room
   for the turn and its frame. Store and wire admission still apply;
   do not raise a schema ceiling.
3. Start with the same `CONFIG`, `DATA` and `CACHE`. Resolve any startup
   refusal before releasing the task. Read its current hold revision,
   then `task release TASK --hold REV` (temper help task).

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
