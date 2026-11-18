# Deployments

Kind: concept

A deployment is one temper: its engine, the workers that run its agents, the forges it works on, and the three directories that hold its configuration, its data and its cache.

## What it is

- **The engine** keeps state and decides acts. A **worker** checks out
  repositories and runs agents. `temper-box init` writes one worker in the
  engine's process, with 4 slots.
- **`temper-box serve`** runs any shape: engine, workers or both.
  **`temper box`** runs serve and each managed forge's setup processes
  on this machine (temper help box). A **server** runs `temper-engine serve` for its engine alone and
  `temper-worker serve` for each worker alone (temper help server).

**The three directories:** `CONFIG` holds `temper.json`; `DATA` holds `agents.setting`, the box’s
copy of its agents setting, the store under `deployment`, workspaces,
forge state, the worker secret and box logs; `CACHE` holds fetched
Forgejo and runner binaries at pinned versions.

Each directory comes from its flag (`temper-box help commands`), else
`TEMPER_CONFIG`, `TEMPER_DATA` or `TEMPER_CACHE`, else
`$XDG_CONFIG_HOME/temper`, `$XDG_DATA_HOME/temper` or
`$XDG_CACHE_HOME/temper`. Without those variables the defaults are
`$HOME/.config/temper`, `$HOME/.local/share/temper` and
`$HOME/.cache/temper`. Relative paths start at the working directory.
`DATA` and `CACHE` belong to the user running temper; missing ones
are made with mode 0700.

**`CONFIG/temper.json` has three members:**

- `engine`: limits, admitted workers, listeners,
  owners, sign-in and first-start projects, each naming `"policy": 1`;
- `workers`: engine addresses, names, secret files and slots; agent programs are derived;
- `application`: managed and fake forge locations, landing rules,
  client and webhook listeners, sign-in and application limits.

Forges are made through setup or connect and kept in the store
(temper help forge). Charters, templates, policies and rules are
settings in the store (temper help settings). Writing them into this
configuration is refused; an inline project policy is refused as
`engine.projects[0].policy: name a policy, policy: 1; change it with
temper ctl settings apply`.
LLM providers are settings in the store (temper help settings): see them with
`temper ctl settings show provider`, change them with `temper ctl settings apply`;
a change is taken at the engine's next start.

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
  `temper-box serve: restore KIND: REASON` or
  `temper-box serve: restart refused during PHASE: REASON`. Keep `DATA`
  and report that line.

## Acting on it

- **The binaries:** the box's `temper-box init` writes a deployment;
  `temper box` runs it; `temper-box serve` runs any shape;
  `temper-box program-login` signs a program in on this host. A server's
  `temper-engine serve` runs its engine, `temper-worker serve` each worker,
  and `temper-worker program-login` signs a program in on each worker's
  host. Each binary's `help commands` lists its own commands.
- **ctl:** `me` reads your party in the deployment; `settings show`
  reads how its work runs.
- **The client:** Deployment, at the rail's foot, holds Accounts, Forges,
  Models, Workers, Members and Limits. On a phone, choose Deployment in
  the project switcher.

**Offering another model** (for example, `claude-haiku-5-5`):

1. Read `temper ctl settings show provider N`, where N is the provider's
   number. Copy its whole value to FILE in this settings form:
   `{"items": [{"setting": {"kind": "provider", "number": N}, "set": VALUE}]}`.
   In VALUE, set `keep` patterns for names to read and `drop` patterns for
   names to exclude; preserve `sources` and `endpoints`. Without `keep`,
   that provider reads no list. Apply with
   `temper ctl settings apply FILE --against REVISION --key KEY`, using the
   newest revision (temper help settings). Drain the stop and restart with
   the same `CONFIG`, `DATA` and `CACHE`. Where the provider has a list,
   temper reads it as an account becomes usable, once a day and on
   **Read again** in **Deployment**, **Models**.
2. For what a list doesn't show, including a new Claude subscription model,
   use **Add a model by name** on **Deployment**, **Models**: choose a
   provider and enter the model's name. Prices are estimated until set;
   the first run shows whether the provider serves it.
3. To set its prices, window or efforts, tell **Ask temper**.
   A task keeps the model it has. Adding or changing a model needs no restart.
   `model list` reads the models offered to a project; a charter or policy
   may restrict which it can use (temper help charter; temper help settings).
4. For a local model, read its provider with `temper ctl settings show provider N`.
   Its `endpoint` names an entry in its `endpoints` list; set that entry's
   `address.socket` to the local server's address. Keep the other members,
   including `sources`. Write the whole provider setting and apply FILE as
   in step 1, then drain the stop and restart with the same `CONFIG`, `DATA`
   and `CACHE`.
5. For a larger output than an agent's endpoint's limits allow, raise those
   limits in the `agents` setting and restart, then change the model.
   Preserve endpoint addresses, transports and sign-in settings. The endpoint's
   `reasoning_item` is the byte allowance per reasoning item; `head_ms` and `idle_ms`
   are deadlines in milliseconds. A model can go quiet for minutes in a response;
   init gives OpenAI's an `idle_ms` of 300000 and Anthropic's 120000.
   Larger outputs may require larger agent and channel bounds; a refused start
   names the shortfall. Restart with the same `CONFIG`, `DATA` and `CACHE`.

**Writing larger pieces:** the `agents` setting’s
`declared.tool_payload` bounds bytes per piece: a write's
content, or an edit's combined old snippets and combined new snippets,
each separately. A 4,487-byte design edit fits the shipped 10,662-byte
profile. This is a deployment setting; changing a charter cannot raise it.

Keep these coupled values in `CONFIG/temper.json` aligned:

- `application.limits.overrides.checkout.message_bytes` holds the payload.
- `engine.limits.overrides.hub.event_bytes` equals payload minus 2,
  the message separator: payload `8194` requires event bytes `8192`.
- Engine overrides `hub.turn_bytes` hold the derived
  agent turn; `hub.turn_queue_bytes` holds that times the agent's turns.
  Engine `core.resume_bytes` holds the turn plus 4; both
  `hub.transcript_bytes` hold that resume window including its framing.
- Application `root.journal.transcript_bytes` and `root.loads` byte
  overrides hold the turn, resume and encoded record carriers.
  Raise every short value startup reports together (temper help limits;
  temper help limits-engine).

For example, `the engine’s figures: hub.event_bytes 4096: 4096 bytes,
short of agents setting declared.tool_payload less the 2-byte separator, 8192 bytes` means set that override to `8192`. An event bound
above that figure is refused with its configured value and upper bound;
set it to the same required figure.

A payload of `16386` can instead fail at the model:
`input-answer: llm.input[endpoint=1]=99876; llm.answer[endpoint=1]=65536`.
The encoded input must fit the model's answer capacity. Reduce the payload,
or raise the agent endpoint's output limit in the `agents` setting and restart,
then change the model's `output` through Ask temper; this example needs
at least 12,485 output tokens (eight answer bytes per token).
Do not lower a receiving check. Recheck all derived carriers after that
change. Drain the stop, edit configuration, and restart with the same
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
  `temper-box init --box`.
