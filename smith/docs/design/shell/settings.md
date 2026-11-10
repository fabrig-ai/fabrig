# Settings and presets

## 1. Settings

### 1.1 Layers

Settings are TOML, strict and bounded. Each layer overrides the one
before, key by key:

1. the built-ins: the presets and defaults (section 2);
2. `$XDG_CONFIG_HOME/smith/config.toml`, the user's;
3. `<repo>/.smith/config.toml`, at the top of the workspace's
   repository, or in the workspace directory when it is not one;
4. `-c KEY=VALUE`, in order;
5. flags.

- **Tables merge, values replace.** The commands' environment is a
  table, so a layer sets one variable without restating the others.
- **The repository layer shapes runs, not authority.** A repository is
  content a person may not have written. Its layer may set instructions,
  conventions, the contract, the model and effort among configured
  endpoints, and environment values. It may not set endpoints, accounts,
  the account a run takes, trust, the credentials directory, the
  programs' paths, inherited environment names, writable caches, the
  trace, the deployment's values, or a budget above the user's. A key it
  may not set is refused, naming the file.
  - **The line.** What the layer may set is what the repository's own
    files already do to a run: its guide is read into the system text,
    its checks are its own executable (../domain/run.md, 3.3 and 8.1), and
    the agent's commands build and test its code with the person's
    authority. Its environment values are constants it writes, as its
    own scripts may export: they shape the commands and checks, and widen
    nothing a command can reach (`README.md`, section 4). What it may not set reaches
    past the repository: where requests go and who pays for them
    (endpoints, accounts, the account a run takes, trust), where smith
    finds credentials and the programs it runs (the credentials
    directory, the programs' paths), what is taken from smith's own
    environment (inherited names), what is written outside the workspace
    (writable caches, the trace), and what the person pays for or
    reserves (the deployment's values, a budget above the user's).
  - **No approval.** A person never approves a repository's settings,
    and smith keeps no record of trusted repositories: an approval would
    guard nothing the repository's own files cannot already do, and
    `smith exec` and a host's runs, which no person watches, would need
    a way around it. The refusal is the boundary, checked at every
    start.
- **State** lives under `$XDG_STATE_HOME/smith`: chats by workspace and
  temporary chats, and, unless `credentials` names another directory,
  the credentials directory's files (`credentials.md`).

### 1.2 Contents and defaults

A first run needs a model and an account:

```toml
model = "openai/gpt-6.1-sol"
effort = "high"

[accounts.chatgpt]
provider = "openai"
source = "sign_in"            # then `smith login chatgpt`
```

Several accounts, several per provider:

```toml
[accounts.claude-alice]
provider = "anthropic"
source = "program"            # Claude Code, signed in by `smith login claude-alice`

[accounts.anthropic-key]
provider = "anthropic"
source = "key"                # kept by `smith login anthropic-key < FILE`

[accounts.ci]
provider = "openai"
source = "env"
variable = "OPENAI_API_KEY"

[endpoints.anthropic]
account = "claude-alice"      # the endpoint's account, when its provider has several
```

| Key | What it is | Default |
|---|---|---|
| `model`, `effort` | the main model, `endpoint/name`, and its effort, one the model declares when it declares any (2.1) | none; the provider's effort |
| `account` | the account main's endpoint runs on, for this invocation; `--account` | the endpoint's own |
| `credentials` | the credentials directory (`credentials.md`) | `$XDG_STATE_HOME/smith` |
| `profile` | the named bundle of deployment values | `standard` |
| `instructions` | the role (../domain/run.md, section 3.1) | the task role (2.6): smith's own, one text for a chat and for a task |
| `chat` | the interactive chat | `default` |
| `waiting_seconds` | how long an interactive run waits before it parks | 2.4 |
| `sub_agents` | whether runs may delegate | off (../domain/run.md, section 5.3) |
| `[budget]` | spend, seconds, turns and the reserve | 2.4 |
| `[models."<endpoint>/<name>"]` | a model's declared values, its efforts and its prices | the preset's, 2.1 |
| `[endpoints.<name>]` | a preset, or its dialect, program, address, TLS name, trust and headers; and its account | `openai` and `anthropic` (2.2) |
| `[accounts.<name>]` | its provider, its source and the source's keys (`credentials.md`) | none: no preset names an account |
| `[programs]` | each program's path, by the program's name: `claude_code` | `claude`, found on `PATH` at startup |
| `[deployment]` | declared deployment values and policy values, a duration in whole seconds (the effective limits give it in milliseconds: ../protocol/limits.md, section 9) | the profile's, 2.3 and 2.5 |
| `[workspace]` | directories, conventions, the contract, delivery fields, the title field, pushes | the workspace directory; a report |
| `[environment]` | `inherit`, the names taken from smith's own environment, and `set` | the core list (`README.md`, section 4) |
| `[build]` | a build preset and writable caches | none |
| `[trace]` | a path and a capture policy (../protocol/events.md) | none |

- **Presets.** `openai` and `anthropic` each supply an endpoint of that
  name, the sources its provider takes, and their known models' declared
  values (section 2). Neither names an account: a first run needs one of
  the person's. A model a preset does not know declares its `window`,
  `output` and prices in settings.
- **Accounts.** An account is a provider and one source (`credentials.md`),
  declared under `[accounts.<name>]`, several per provider, each signed
  in on its own.
  - **A provider** is a preset, `openai` or `anthropic`. An endpoint the
    settings declare without a preset is a provider of its own, under
    the endpoint's name, and takes `key`, `env` or `none`.
  - **A name** is 1 to 32 characters of `a-z`, `0-9` and `-`, since it
    names files and a directory in the credentials directory.
  - **An endpoint's account** is its `account`, by default the only
    account of its provider. For an endpoint of the run's models,
    several accounts of its provider with none named is refused, naming
    the key; for another endpoint, it is a warning.
  - **`--account NAME`,** or `account`, sets main's endpoint's account
    for one invocation, and one of another provider is refused, naming
    both. An account is fixed for an invocation, as main's endpoint is
    (`commands.md`, 1), and a run never moves to another account, at a limit or
    otherwise (`credentials.md`).
  - **A source's program** is its endpoint's: an account whose source is
    `program` on an endpoint that names no program is refused, naming
    both.
- **The run's models and accounts.** A run's charter lists main's model
  and, with `sub_agents` on, every model a preset knows or settings
  declare whose endpoint has an account whose source is not a program;
  the asking LLM names one of them, or gets main's (../domain/run.md, 5.3).
  A sub-agent runs on a program only on main's account, as the models
  of main's endpoint do (../protocol/programs.md, 5.10); another program
  account's models stay out.
  Every model main's endpoint declares is also the run's, so that
  `/model` may choose any of them (`commands.md`, 1): they share main's account and
  connections, and the derivation is the largest of them. An endpoint
  with no account is one the person has not set up, so its models stay
  out: a first run with one account needs no other credential.
  - **Needed accounts** are the accounts of the run's models. Main's
    endpoint must have an account, or the command exits 2 with the first
    run's lines (1.3). Each needed account's credential must be usable
    (exit 3, `commands.md`, 3). An account no model of the run needs is reported, and
    its problems are warnings.
  - **The agent's configuration holds the run's models only,** so the
    derivation and every relationship (../protocol/limits.md) are those of
    the models the run may name. A model outside the run is still
    checked as values: priced above zero at every rate, and its output
    below its window. `smith check` names each model outside the run and
    why.
- **Prices.** A preset model carries notional prices, the provider's list
  prices, so that spend is real (../domain/budget.md). A
  subscription account's spend is shown as notional, never as a charge.
  Every rate is positive: a model that costs nothing per token, such as
  a local one, is given notional prices, and settings that price a model
  at zero at any rate are refused, naming the key.
  The local host's unit is the micro-dollar; settings and the terminal
  use dollars.
- **Tiered list prices.** Where a provider's list price for a model rises
  past a prompt size, the preset prices the model at the highest rates a
  prompt within its window can reach, its window as settings leave it:
  Haiku 5.5 at its long-prompt rates whenever its window passes 100,000
  tokens, and each OpenAI model at its long-context rates whenever its
  window passes 272,000 tokens, so at its base rates at the preset's
  window. A charter's prices stay flat (../domain/budget.md), so
  spend is never under-counted. A person who wants the lower rates
  declares a window at or below the tier's size; prices a person sets
  are used as set. The tiers are a prompt's: the benchmarks price each
  response at the rates its own prompt reaches, as the provider bills
  it (../benchmarks/README.md, 8.1), and smith's own spend stays at the flat
  rates its window can reach.
- **Raw byte limits are not settings.** Tests may override them; a person
  declares quantities (../protocol/limits.md).

### 1.3 Validation

- **Each layer is read strictly.** An unknown key, a wrong type, or a
  value past its bound is refused, naming the file, the line and the
  key's path. A file holds at most 1 MiB.
- **The resolved settings are checked whole,** by `smith check`'s
  checks:
  - every relationship between declared values (../protocol/limits.md);
  - the budget against the profile's ceilings, and against one
    completion's maximum cost, so an impossible spend is refused before
    a run starts (../domain/budget.md);
  - each model known to a preset or declared, and priced above zero at
    every rate;
  - each endpoint of the run's models resolved, and each needed
    account's source usable (1.2): a sign-in's record present and its
    scopes granted, a key kept, a variable set, and a program source's
    check usable (../protocol/programs.md, section 3);
  - the tools' paths, and each program a run may use.
- **A refusal names the relationship:** the keys on each side, their
  values and their layers, and the relationship's name, for example:

  ```text
  smith: budget.turns = 5000 (from -c) exceeds the standard profile's ceiling of 4096 turns
  ```

- **Startup is `smith check`.** Every command runs its checks first and
  refuses with exit 2, or 3 for credentials.
- **A first run's lines.** With no model, or no account for main's
  endpoint, `smith` and `smith exec` exit 2 and print what to add and
  the command to run after:

  ```text
  smith: no account: add to ~/.config/smith/config.toml
    model = "openai/gpt-6.1-sol"
    [accounts.chatgpt]
    provider = "openai"
    source = "sign_in"
  then run `smith login chatgpt`
  ```

- **Settings of an earlier version are refused,** each naming what takes
  its place, since strict settings never guess:
  - `source = "borrow"`: "declare `provider = "openai"` and `source =
    "sign_in"`, then run `smith login <account>`";
  - a `sign_in` account's `client_id`, `redirect_uri`, `scope` or
    `token_format`: "the provider's preset gives the registration";
  - an endpoint's `identity`, `dialect = "codex"`, and the `codex`
    endpoint: "use `openai/<model>`".
- **An older local chat is refused before Store mutation,** with exit 1,
  whether interactive or headless, including /resume. The named chat lives
  under the workspace's `chats` tree in smith's state directory (1.1).
  A missing layout record says:

> Chat `<chat>` was kept by an older build of smith, from before chats recorded their layouts, and this start has changed nothing in its directory. Before the first release a chat is not translated, so every start refuses this one until it is moved. To start afresh, stop each smith using it, move its directory under smith's state directory aside, whole, and start again: nothing is deleted, and the next start makes a new, empty chat, without this chat's history or its deliveries' answers.

A different store layout says:

> Chat `<chat>` was kept by an older build of smith, whose records' layout differs from this build's, and this start has changed nothing in its directory. Before the first release a chat is not translated, so every start refuses this one until it is moved. To start afresh, stop each smith using it, move its directory under smith's state directory aside, whole, and start again: nothing is deleted, and the next start makes a new, empty chat, without this chat's history or its deliveries' answers.

A different transcript layout says:

> Chat `<chat>` was kept by an older build of smith, whose turns' layout differs from this build's, and this start has changed nothing in its directory. Before the first release a chat is not translated, so every start refuses this one until it is moved. To start afresh, stop each smith using it, move its directory under smith's state directory aside, whole, and start again: nothing is deleted, and the next start makes a new, empty chat, without this chat's history or its deliveries' answers.

Earlier startup may already prepare credentials, make directories, generate
child configuration and open traces outside the Store. An operator-configured
credentials directory inside the chats tree is an explicit exception to the
unchanged-directory claim. Explicitly colocated `-o` output may be written
after the refused Load outcome, during teardown; those later output bytes
are outside that claim. Configured local and generated child trace destinations
resolving inside the workspace's chats tree, including aliases and absent
files under existing parents, are refused before any trace open.

### 1.4 Generated agent configuration

Hosts generate an agent's configuration, so it stays JSON
(../protocol/agent.md, section 4): `smith-agent CONFIG.json`. In the
product the shell writes the agent configuration from the presets and
the resolved settings: in one process as a value it hands the service,
with no file; when it spawns an agent (a run whose account's source is a
program, the worlds, and an `isolate` option later), as that JSON, with
the spawned agent's `exit_grace` derived beside it (2.5). It names each
endpoint's dialect and program, each program's path, and the programs
directory, `<credentials>/programs` (`credentials.md`); never an account or a
credential, which the run's charter and grants carry
(../protocol/charter.md, section 2; ../protocol/channel.md, section 6).

## 2. Presets' declared values

These are the product's defaults: the per-model values each preset
declares, the `standard` profile's deployment and policy values, and
the defaults of a run the local host starts. Everything else is derived from them (../protocol/limits.md) and checked at
startup. Each is validated by the probes of ../benchmarks/README.md, and moves when
a probe shows it should. "Provisional" marks a value whose evidence is
still thin. Settings override any of them (section 1). In code, the
per-model values and the run defaults are `smith-local-shell`'s
presets, and the profile's values are the service crates' `standard`
profile (`README.md`, 2.1).

### 2.1 Per model

| Preset | Model | `window` | `output` | `reasoning_item` | `head` | `idle` |
|---|---|---:|---:|---:|---:|---:|
| `openai` | `gpt-6-astra` | 272,000 | 32,000 | 64 KiB | 60 s | 300 s |
| `openai` | `gpt-6-luna` | 272,000 | 32,000 | 64 KiB | 60 s | 300 s |
| `openai` | `gpt-6.1-sol` | 272,000 | 32,000 | 64 KiB | 60 s | 300 s |
| `anthropic` | `claude-opus-5-5` | 200,000 | 32,000 | 64 KiB | 60 s | 120 s |
| `anthropic` | `claude-sonnet-5-5` | 200,000 | 32,000 | 64 KiB | 60 s | 120 s |
| `anthropic` | `claude-haiku-5-5` | 200,000 | 32,000 | 64 KiB | 60 s | 120 s |

- **`window`** is usable input tokens, the most one prompt may hold
  with a completion's output beside it; **`output`** is the most output
  tokens, the wire's cap where the dialect sends one, and the output the
  headroom above the threshold holds for the completion whose results a
  compaction carries (../protocol/limits.md, 2.1 and 3.9; ../domain/session.md,
  section 8.1).
- **`efforts`** are the reasoning efforts a model accepts, each a name
  within 32 bytes, as its provider documents them for that model. They
  are the model's, not the dialect's: a provider's models differ, and
  their names change with the provider. `smith check`, `--effort`,
  `/effort` and the run's entrance refuse an effort outside the list,
  naming it and the list. A model that declares none takes any effort
  within the bound, and a provider that refuses it says so as the
  request's failure. A preset declares a model's efforts only from its
  provider's documentation, with that source, and until it does the
  model declares none.
- **The Anthropic models take 1,000,000 input and 128,000 output
  tokens.** The preset declares less, so that a window is compacted
  before its cost grows and the headroom holds a likely output. Settings
  raise them, a window at most what ../protocol/limits.md, 2.1, allows
  beside the output declared.
- **All per-model values are provisional.** The history probe validates
  `window`, the large-write probe `output`, and the slow-completion probe
  `reasoning_item`, `head` and `idle`.
- **A model no preset knows** declares `window` and `output`;
  `reasoning_item`, `head` and `idle` default to its provider's values in
  this table.
- **The working tier's model.** `gpt-6.1-sol` is the working tier's
  OpenAI model, and a tier's model is one the presets declare
  (../benchmarks/README.md, section 7), so that the tier runs smith as it ships. Its
  notional prices are its provider's list prices, tiered as 1.2's
  "Tiered list prices" says, and the code holds the numbers. It declares
  no efforts until a preset takes them from its provider's documentation
  (the `efforts` bullet above).
- **The small tier.** `gpt-6-luna` and `claude-haiku-5-5`, each at `low`
  effort, are the models benchmarks' probes run on (../benchmarks/README.md,
  section 7). Each preset model takes its provider's effort names, `low`
  among them; without `effort`, the provider's default applies.
- **Every preset holds a window.** With the `standard` profile's 80%
  threshold, a window's room after two outputs, counting a byte as a
  token (../protocol/limits.md, 3.9), is 96,000 tokens for the Anthropic
  models and 153,600 for the OpenAI models, and the headroom above the
  threshold, less one output, is 8,000 and 22,400. Both pass
  `fresh-window` and `compaction-headroom`. Less the summary instruction
  and a marker for each of `calls_per_response` calls, that is each
  model's spare, the room between its threshold and its line
  (../protocol/limits.md, 3.9). Halved after smith's own
  part, the room bounds one offer, and so a message, a person's line or
  a headless prompt, near 20 KiB when an Anthropic model is declared and
  near 34 KiB with the OpenAI models alone; `smith check` prints the
  exact values.

### 2.2 Per provider

| | `openai` | `anthropic` |
|---|---|---|
| endpoint | `api.openai.com:443`, TLS name `api.openai.com`, `/v1/responses`, OpenAI's public Responses dialect, `openai` (skein's `llm/README.md`, 4.8) | `api.anthropic.com:443`, TLS name `api.anthropic.com`, `/v1/messages`, Anthropic's Messages dialect, `anthropic`, with a key (skein's `llm/README.md`, 4.7) |
| program | none | `claude_code`, Claude Code (../protocol/programs.md) |
| sources | `sign_in`, with smith's registration; `key`; `env` | `program`; `key`; `env` |
| how a credential is sent | `Authorization: Bearer`, a sign-in's token or a key alike (skein's `llm/README.md`, 2.6) | a key, in `x-api-key` |
| `connect`, `handshake` | 10 s each | 10 s each |
| `connection_keep` | 30 s, provisional | 30 s, provisional |

- **The dialects** are named `openai` and `anthropic`, in settings as an
  endpoint's `dialect` (1.2), and the local host writes each into the
  agent's configuration with its number (../protocol/agent.md, section 4).
- **smith's OpenAI registration,** the `openai` preset's and never a
  setting's: Sign in with ChatGPT for an open-source program, with no
  client secret and no partner key.
  - **Authorization** at `https://auth.openai.com/api/accounts/authorize`
    and the token endpoint
    `https://auth.openai.com/api/accounts/oauth/token`, with OpenAI's
    published keys for the ID token and its revocation endpoint.
  - **The entry client** `dynamic_agent_client`, used only to register:
    the first sign-in sends `agent_name_hint` `smith` and the
    installation's `ext_agent_host_id` (`credentials.md`), and keeps the
    client id the callback issues for every exchange, refresh and
    revocation after.
  - **The scopes** `openid profile email` for identity and
    `offline_access resource.invoke chatgpt.tokens.use.direct` for plan
    usage, with `chatgpt.tokens.use.direct` required for a run;
    `resource` `https://api.openai.com/v1`.
  - **The redirect** `http://127.0.0.1:<port>/auth/callback`, whose port
    alone may change between sign-ins (`credentials.md`). The callback listener
    admits a request head of at most 16 KiB and 64 header fields: room for
    a browser's roughly 2 KiB, 20-field navigation, its authorization
    query and client hints, with bounded headroom for larger codes and
    browser metadata.
  - **The condition:** OpenAI sanctions plan usage for open-source
    programs, and smith is MIT-licensed (`README.md`, 3.1). A preset carries a
    provider's registration only on such a sanction; one withdrawn is a
    change to this document.
- **What the `openai` preset shows,** as OpenAI asks of a program on a
  person's plan:
  - at an account's first sign-in with plan usage, once: "You're using
    your ChatGPT plan. Eligible requests from smith use your plan's
    included usage; manage it at chatgpt.com/settings/usage.";
  - the usage address, `chatgpt.com/settings/usage`, in a limited
    account's last line (`commands.md`, 3);
  - after a sign-out whose revocation is not confirmed, to disconnect
    smith in ChatGPT's settings (`commands.md`, 6);
  - "Using ChatGPT plan" on the status line (`commands.md`, 1), and a plan's spend
    as notional, never as a charge (1.2).
- **The `anthropic` preset carries no registration.** Its subscription
  is reached through Claude Code, signed in by the program's own flow
  (../protocol/programs.md, section 4), and its own loop takes a key.
- **The plan's models:** which of the `openai` preset's models a ChatGPT
  plan serves is what `/v1/models` lists for its account, open until a
  capture (`README.md`, section 7).

The connection keep is a live service's policy, never how it ends:
pools close at the answer (`README.md`, section 5). It and the deadlines are policy
values of ../protocol/limits.md, 2.3.

### 2.3 Per deployment: the `standard` profile

The declared quantities of ../protocol/limits.md, section 2.2:

| Value | Default | Status |
|---|---|---|
| `conversations` | 5: main and four children, the parallel-children probe's, within `memory` | provisional; the parallel-children probe |
| `tool_payload` | 40 KiB, so that a call at the payload, escaped at its largest (6 bytes a byte), fits one answer of the smallest preset `output` (32,000 tokens at 8 bytes a token): skein's `input ≤ answer` (../protocol/limits.md, 4) | provisional; the large-write probe |
| `file_bytes` | 1 MiB: the largest file the tools read whole and edit | provisional; the large-write probe |
| `calls_per_response` | 16 | provisional; the parallel-read probe |
| `jobs` | 4 jobs held by one session at once | provisional; the tool-output probe's `background` |
| `read_window` | 32 KiB; a call may ask up to `tool_payload` | decided |
| `guide` | 16 KiB, a guide kept whole up to it | decided |
| `shell_output` | 16 KiB | provisional; the tool-output probe |
| `shell_head`, `shell_tail` | 16 KiB and 48 KiB: a call may ask for up to 64 KiB, kept in that proportion | model-selectable decided; sizes provisional, the tool-output probe |
| `search_hits`, `search_bytes` | 100 and 16 KiB | provisional; the tool-output probe |
| `list_entries` | 512 | provisional |
| `llm_pool` | 128 MiB | provisional; peak memory in `session.ended` |
| `memory` | 2,176 MiB (34 × 64 MiB) | declared process memory, carried without a startup fit check (../protocol/limits.md, section 5) |

### 2.4 Run defaults

| Value | Default | Status |
|---|---|---|
| `budget.spend` | 25 dollars, notional | provisional |
| `budget.seconds` | 1 hour per activation | provisional |
| `budget.turns` | 512, a loop guard | provisional |
| `budget.reserve` | 2 turns and 5 minutes | provisional; the budget probe |
| waiting time | 15 minutes interactive; 0 for `smith exec` | provisional |
| `sub_agents` | off | decided: children run one at a time, and the parallel-children probes are uncalibrated (`../../development/benchmarks/sub-agents-by-default.md`) |

### 2.5 Policy values

The profile's values that bound no bytes, under ../protocol/limits.md's
names (`../protocol/limits.md`, section 2.3):

| Value | Default | Status |
|---|---|---|
| `max_turns`, `max_spend`, `max_time`, `max_waiting` | 4,096 turns, 1,000 dollars notional, 24 hours of wall time, 24 hours of waiting | provisional |
| `sections`, `host_tools`, `verdicts`, `items`, `fields` | the charter schema's ceilings | decided: the profile narrows none |
| `inbox` | 64 messages unread | provisional |
| `unacknowledged` | 64 turns | provisional |
| `compaction_threshold` | 80% of the window | decided as about 80%; the history probe |
| `shell_timeout` | 2 minutes | provisional; the tool-output probe |
| `check_timeout` | 5 minutes, which the default reserve's 5 minutes holds (../domain/budget.md, 1) | provisional |
| `tool_deadline` | 20 minutes, the most a command's call may ask for | provisional; the tool-output probe |
| `job_poll` | 30 s, a `job` call's wait when it asks for none | provisional; the tool-output probe's `background` |
| `job_wait` | 5 minutes, the longest yield or wait a call may ask for | provisional; the polls of real runs |
| `parallel_tools` | 4 adjacent reads run at once | provisional; the parallel-read probe |
| `group_stop` | 1 s for each of its three steps | provisional; the process-tree probe |
| `close` | 5 s | provisional |
| `write_deadline` | 30 s for one write of the event stream | provisional |
| `wrap_up` | 60 s, a child's final turn's allowance at most | provisional; the budget probe's `wrap-up` |
| `cancel_grace` | 10 s | provisional |
| `exit_grace` | derived: the agent's teardown bound, two `close`s, the event stream's `write_deadline` and its cancels' settling, plus one more `close` as a margin (skein's `runtime/shell.md`, section 12; ../protocol/limits.md, section 7); a setting below the bound is refused | decided as derived; the margin provisional |
| `sign_in_time` | 10 minutes, a browser sign-in's deadline (`commands.md`, 5) and skein's `oauth/README.md` sign-in deadline; no profile value: the product's client limits fix it, and `smith check` does not print it | provisional |
| `refresh_lead` | 60 s before the access token expires | provisional; the `credentials` probe |

### 2.6 Instructions

The task role is smith's own instructions: the role a run is given when
settings set no `instructions` (1.2; ../domain/run.md, section 3.1).

- **One text, at most 4 KiB,** the same for every model, and for chats
  and tasks alike.
- **Its topics, in this order:**
  1. carrying the request to its end, within its scope, by the plan or
     the steps it gives, in their order, and never by what it rules out;
  2. finding files, with `list` and `search` first, then reading the
     lines the work needs: the documents the request names (its task,
     its plan, the guides it points to) read together in the first
     response, each from its start in the default window; code, a
     window from where a search found it, one window wide enough for
     the part the work needs rather than several small ones, and a
     whole file only when the work needs all of it;
  3. batching in one response every call that waits on no other's
     answer: reads, lists, searches and commands that change nothing,
     with a running job's poll beside them; and a change's edits,
     across its files, one `edit` per file in one response, then the
     command that checks them;
  4. changing files with `edit` and `write`, never through the shell,
     the smallest change first, with every replacement one file needs
     in one `edit`, and an existing file written whole only when most
     of it changes, as code the repository's
     reviewer would accept by its guides, with no public item, lint
     exception or unreachable path the change does not need;
  5. running the tests the change needs, each check once on the same
     files, and long commands as jobs whose polls go beside other work
     (../domain/tools.md, section 5);
  6. not reverting changes the model did not make;
  7. the report that `finish` carries, as soon as the work is done and
     checked, with no reading or running for the report alone;
  8. that a compaction does not end the work.
- **It names only smith's own tools** (../domain/run.md, section 5.1), and
  says nothing that the run's own sections of the system text already
  say (../domain/run.md, section 3.3).
- **Its words are code,** in `smith-local-shell`; its topics are this
  list. A topic added, dropped or moved is a change to this
  section.
- **Its words cost requests.** Each step the words ask of the model in a
  response of its own is a round trip to the provider. The words ask for
  no such step that the work does not need: what can go together goes
  together, and a command is made a job only when it runs long. A change
  to them is proven by the `compare` suite, or, on the way to a verdict,
  by the `parity-change` suite when its run can tell the change's
  expected effect from none, and otherwise by the counts it targets
  (../benchmarks/README.md, section 4.2, and ../benchmarks/codex-parity.md, section 6).
- **It is a run's instructions,** so it counts where a charter's
  instructions count: in the window's host part (../protocol/limits.md,
  3.9). The startup check counts it as written (../protocol/limits.md,
  section 4, `host-part-holds-the-role`): its 4 KiB is the text's own
  cap, never a reservation in the host part.
