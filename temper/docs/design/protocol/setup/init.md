# The configuration and `temper-box init`

- **One document,** `temper-box serve`'s, `temper.json` in the
  configuration directory (../configuration/README.md, section 2 and ../configuration/directories.md). Every
  `temper-box forge` command and `temper box` read it whole, check it
  as ../configuration/README.md, section 6 says, `application.machine` among it,
  and act on their part: they read no file but their own part's.
- **It names no forge.** A forge is a record of the forge connector,
  made by the act that sets it up or connects it, and numbered next when
  it is made (../forge/engine.md, 1 and 2): its kind, mode, place, git
  program, bot and first repository's intent are its record's, carried
  by the act that makes it, `SetUp` for an owned Forgejo (`owned.md`, 5, 7),
  `Connect` for a Forgejo elsewhere (`reconcile.md`, 5) or GitHub (`github.md`), and
  `StartPlayground` for the playground's fake one (`playground.md`). A first run's
  document names none (3).
- **The document keeps the machine's:** the clients' and the webhooks'
  listeners; in `application.machine` (../configuration/engine.md),
  what the forges a box can run take from this machine, the box's owned
  Forgejo's port and the fakes' ports among it, which `temper-box init --box`
  chooses and the box holds from its start (`box.md`, 1, `init.md`, 2); and the limits,
  whose forge members count the deployment's forges and repositories,
  never a sum over its forges (../forge/README.md, section 6).
- **`application.machine`,** each of its parts absent, or null, for
  none:

  ```
  "machine": {
    "owned": {
      "forgejo": "/opt/forgejo/forgejo",
      "runner": {"binary": "/opt/forgejo/forgejo-runner",
                 "labels": ["temper-host:host"], "capacity": 1,
                 "path": "/usr/local/bin:/usr/bin:/bin"},
      "listen": {"socket": "127.0.0.1:3000"},
      "remote": "http://127.0.0.1:3000",
      "person": {"login": "ada", "email": "ada@example.invalid"}},
    "fake": {
      "listen": {"socket": "127.0.0.1:8369"},
      "remote": "http://127.0.0.1:8369",
      "git": "demo/bin/git"},
    "github": null}
  ```

  | Member | Type | Bounds |
  |---|---|---|
  | `owned` | the box's owned Forgejo | its members below; the owned kind's group (`box.md`, 1) and the offer to set one up (`owned.md`, 5) need it |
  | `owned.forgejo`, `owned.runner.binary` | cache paths (../configuration/directories.md), or `"fetched"` | checked against the pins of `owned.md`, 1; `"fetched"` names temper's pinned build in the cache, which `temper-box forge fetch` writes (`owned.md`, 6) |
  | `owned.mirror` | absent, or `{"forgejo", "runner"}`, each an origin | each `https://HOST` or `https://HOST:PORT`, no path: where `temper-box forge fetch` asks for the pinned paths (`owned.md`, 6) |
  | `owned.public` | absent, or an origin | `https://HOST` or `https://HOST:PORT`, no path: the address people reach this Forgejo at, behind a proxy; written as `ROOT_URL` and `DOMAIN` (`owned.md`, 2); refused beside a set `owned.sign_in`, at that member's path (`../openid-provider.md`, 9) |
  | `owned.runner` | the runner's members, or null for none | `capacity` at least 1; `labels` at least one, at most 8 |
  | `owned.listen` | an address, jig's format | a socket address on loopback; unique among the deployment's listeners and the machine's ports |
  | `owned.remote` | an HTTP base | `http://<listen>`: the forge's git base |
  | `owned.person` | `{"login", "email"}` | a Forgejo login other than the bot's and `temper-admin`, and with `sign_in` set other than `work` too, its address not the bot's (`../openid-provider.md`, 9) |
  | `owned.sign_in` | a client of `engine.clients.provider`, or `null`; absent as `null` | the client a box's owned Forgejo signs in through (`../openid-provider.md`, 9) |
  | `fake` | the playground's fake forge | its members below; the fake kind's group (`box.md`, 1) and the playground's start (`playground.md`) need it, and `application.demo` beside it |
  | `fake.listen` | an address, jig's format | as `owned.listen` |
  | `fake.remote` | an HTTP base | `http://<listen>` |
  | `fake.git` | a data path | the playground's git, a link to `temper-box fake git` (`init.md`, 2, `playground.md`): the program the engine names in each of the fake forge's repositories' records, for the worker to spawn in place of `workers[].git` (../channel.md, section 4) |
  | `github` | GitHub's place, `{"api", "transport", "remote", "web", "git"}`, or null | null for github.com's (`github.md`, 1); a place only to point at a fake (`box.md`, 5) |

- **Runner slots:** change CI and tiers whose workflows use `temper-host`
  share the owned runner's capacity, including other repositories' jobs.
  `application.machine.owned.runner.capacity` has minimum and default 1
  (3). To run CI beside one single-job tier takes 2 slots; beside two
  such tiers takes 3. Multi-job workflows need a slot for each concurrent
  job. With only one slot a slow tier can keep CI waiting and prevent
  landings during its run. A capacity edit reaches an existing runner
  through re-preparation (`owned.md`, 3); stop the box, edit `CONFIG/temper.json`,
  then restart on the same configuration, data and cache directories.
- **A forge's setup entry,** what the setup host and the box's commands
  set an owned, fake or managed forge up with, is made from its record
  and, for an owned or fake forge, the machine's part of its kind: its
  bot, the record's, `temper` for an owned or fake one; an owned or fake
  forge's directory, `forge-N` or `playground-N` (`README.md`, section 3); an owned forge's binaries,
  runner, port, person and origins, `owned`'s; a fake forge's port and
  git, `fake`'s, its seed's owner `playground` and its one repository
  `widget`, landing on `main` (`playground.md`). A managed forge's is its record's
  alone: its records are made when the person connects as its admin
  (`reconcile.md`, 5), and its admin token's name is `admin`, not the owned forge's
  `temper-admin`. No entry names an org: an owned forge's org is its
  first repository's, made by `Create` (`owned.md`, 7).
- **A Forgejo connected** with its bot's tokens has no setup entry
  (`README.md`, section 4); GitHub takes none (`github.md`).
- **The page's actions are the engine domain's offers**
  (`../../domain/forge/adoption.md`, 1): "Set up a Forgejo here" while the
  machine names the box's owned Forgejo and none is kept (`owned.md`, 5), and
  "Connect as its admin", which makes a Forgejo elsewhere from the
  address the person gives (`reconcile.md`, 5). The client cannot infer these offers
  from the configuration.
- **The owned card's configuration** gives its data directory, Forgejo's
  and its runner's versions and the person CI jobs run as; the card says
  CI is unisolated and Forgejo stops with `temper box` (`owned.md`, 5;
  `../../client/forge/deployment.md`). After connect it offers "New repository",
  with the set up's organization and repository fields (`README.md`, section 9
  ), including after a restart. The card
  names the `person_password` path, never its value.
- **GitHub's connected card** shows the setup URL to set on GitHub
  (`github.md`, 3); before any accepted installation, the action is "Install the
  App", and after one, "Install on another account" (`github.md`, 4).
- **A forge's record names no credential:** its secret records hold
  them (`README.md`, section 3). An owned or fake forge's place is the machine's
  part's, written out, never derived: its `api` the port's address,
  plaintext, and its `git` the `remote`, `http://<listen>`, which a fake
  forge does not serve yet (`playground.md`).
- **A forge's number is its stable id** (../forge/engine.md, 1), given when it
  is made and never reused, even once the forge is removed: its secret
  records are kept under it; and records made for another kind or
  address leave a forge not connected, named, with nothing erased
  (`README.md`, section 3).
- **`webhooks`,** `application.webhooks`, `{"listener": address}`, on
  loopback; each Forgejo forge's secret is its `WebhookSecret` record
  (../configuration/engine.md; skein's `forge/README.md`, section 6).
- **The clients' listener at a fixed address,** for GitHub: GitHub's
  redirect and setup URLs name it, so its port is never 0 (`github.md`, 1).
- **`temper-box serve` checks `application.machine`,** and reads one file
  of a forge's setup, an owned or fake forge's handover (`README.md`, section 3).
  Under the box, as `temper-box internal serve --box`, it adopts the box's
  pipe, over which it asks for a forge's group, and for its removal
  (`box.md`, 1).
- **`application.demo`,** the scripted model's fake LLM, `llm` alone,
  or null for none (`playground.md`).
- **`application.welcome`,** `true` to show the welcome; absent, none
  (../configuration/engine.md).
- **An owned example** ships for a person, a test or QA's host to start
  from (1).
- **`temper-box init`** writes a deployment that starts (2), with the
  default document (3).

## 1. The owned example

The box document written by `temper-box init --box` and the owned example take
jig's `standard`'s `documents.lines`, sixteen entries for each creating
executor list, and its `tasks.charters`, twelve.
This holds the reference's eligible charters 1, 2, 4, 5, 6 and 7 and its
configured procedures without shortening either offer. The deployment's
effective limits must still fit its codecs.

`docs/examples/owned/` is a configuration directory (../configuration/directories.md
). Its document, `temper.json`, is a whole one, `engine`,
`workers` and `application`, for the box's owned Forgejo with a runner,
webhooks, one worker and two LLM providers, with no
LLM account: a person adds one from the client once the deployment has
started (../credentials.md, section 3). Beside it are the files it names
that hold no secret: the agent's configuration, `agent.json`, and the
charters, `charters/`, short texts a person edits. The files that hold
a secret, the worker's secret and its digest, are data paths and never
shipped: the walk `box` (`temper help box`) says how each is made in the data
directory. A person, a test or QA's host starts from a copy, edits the
agent's absolute paths to the copy's, and `temper box --config COPY`
runs it (`box.md`).

- **Its owned Forgejo,** the machine's `owned` (above), with a runner
  whose label is `temper-host:host`, at `127.0.0.1:3000`, and the person
  `ada`, whose org is `acme`; the document names no forge. The person
  sets it up from the client under the box (`owned.md`, 5): its `SetUp` makes
  forge 1, the first forge made, in
  `forge-1`, its bot `temper`, its place the machine's, `api` and `git`
  from `listen`, plaintext; no credential, since its records are made
  at `started`, from the handover (`owned.md`, 5).
- **Its landing rule** covers `acme/widget`'s `main`, and asks for CI,
  Actions on the owned runner, the head up to date with its base, and an
  agent's review, gate 1, at a clean head (../configuration/engine.md).
  `acme/widget` is a repository the person makes in the org on
  Forgejo's web, with Actions, then adopts (`owned.md`, 5).
- **Its owner may do the forge's work in the org,** and no more: the
  rules' ceiling, the project's ceiling, its role's authority and its
  projections each hold one grant on the org, `acme`, for each kind a
  change and its goal's issue use (`../../domain/forge/README.md`, section 4):
  read, push, open a pull request, merge, comment, create an issue, edit
  a pull request, update it from its base, close or reopen it, and edit,
  close or reopen an issue; 10, as the process tier's owner holds them.
- **Its LLM providers** (../configuration/README.md, section 2), written out
  whole: 1, OpenAI, its public Responses endpoint and gpt-6.1-sol, the
  first and so the charters' default, then gpt-6-luna, the cheapest,
  each with its window, output and prices, offering keys and ChatGPT's
  sign-in.
  The sign-in is a plan's (`signed_in_plan`, its manage-usage address
  `https://chatgpt.com/settings/usage`), on ChatGPT's public
  registration as jig's live sign-in uses it (jig's
  `domain/providers.md`, 2.2): the issuer `https://auth.openai.com`,
  the entry client `dynamic_agent_client`, the name hint `temper`,
  authorization at `/api/accounts/authorize`, the token at
  `/api/accounts/oauth/token`, revocation at
  `/api/accounts/oauth/revoke` and the published keys at
  `/.well-known/jwks.json`, all on `auth.openai.com` over TLS; the
  scopes `openid profile email offline_access resource.invoke
  chatgpt.tokens.use.direct`, the last of which must be granted; the
  resource `https://api.openai.com/v1`; the redirect
  `127.0.0.1:1455/auth/callback`, its port fixed, as the registration
  requires. A plan's runs use the same endpoint and model as a key's,
  with the grant as a bearer (jig's `protocol/llm.md`, 5.3). The charters
  allow the provider's models in order, the default the first whose LLM
  provider has a usable account (../llm.md, section 5; jig's
  `domain/tasks/README.md`, section 3).
- **Its agent's configuration** (../configuration/README.md, section 4),
  `owned/agent.json`, declares OpenAI's endpoint and model, and
  Claude Code's program, its path and its programs directory, a
  directory the person names; and takes `temper-box init`'s figures, sized
  for a project of fabrig's scale (../configuration/stock.md): smith's
  `standard`, with `declared` `memory` 2,147,483,648, `tool_payload`
  the largest smith admits at its model's output of 8,192 tokens (about
  10,662 bytes; ../configuration/settings.md, 7), `read_window` 256,
  `conversations` 26 (room for a run and 25 sub-agents, smith's
  `protocol/limits.md`, 2.2: a run's main conversation and its
  sub-agents', each a program session of its own on Claude Code, smith's
  `protocol/programs.md`, 5.10) and `calls_per_response` 48 (smith's
  `domain/session.md`, 5, and smith's `protocol/limits.md`, 2.3), and `policy`
  `max_turns` 800, `check_timeout` 60,000 ms, `group_stop` 10, `inbox` 82
  (a run's inbox, jig's `standard` `tasks.run_inbox`, which the fleet
  holds the agent to at its hello) and `unacknowledged` 8. The stock's
  charters run up to 800 turns;
  each permits modifying work with a 60,000 ms reserve, so the agent's
  check timeout must fit that reserve. smith's two-byte rendered
  separator makes the agent's advertised message bytes its payload less
  two, and the engine's message and result bounds are those
  (../configuration/stock.md). Its largest call is then its payload, the
  larger of that and its outcome (../configuration/limits.md,
  `standard` against the agent). smith's standard sizes with nothing
  declared refuse the example's model declaration at start.
  `temper-box init`'s agent takes the same figures but its endpoints and
  programs (2); its scripted model's window is `gpt-6-luna`'s, so its
  room is the same (3).
- **Its `engine.limits`** are jig's `standard` profile (../configuration/stock.md
  ) with the overrides ../configuration/limits.md asks so that the
  engine's documents fit temper's client, those its agent sets, and the
  machine's concurrency the box keeps, and no other:
  - `documents.rows` 2, temper's client's rows, and
    `documents.text_bytes` 512, the most document text its web and
    terminal clients receive: their wire takes the larger of their
    `text`, 512, and a hold's account, 512 (jig's
    `client/domain/contract/documents.md`, 3), and refuses a longer one. It is
    also the piece a client reads a task's text in (jig's
    `client/domain/contract/README.md`, 4), so it is as large as they take. A
    run's `read_task` has its own room, the agent's `tool_payload`, 10,662
    (`../../domain/routing.md`, 2);
  - `tasks.message_bytes` and `tasks.result_bytes`, the agent's
    advertised message bytes (above): the fleet holds the agent to the
    engine's message bound at its hello (jig's
    `protocol/limits.md`), and a result is a message in its
    parent's inbox; and `tasks.inbox_bytes`, `tasks.inbox_messages` at
    that bound, 874,120, so that a task's row stays within the journal's
    carrier (`root.journal.transcript_bytes`, below);
  - the machine's concurrency where `standard` sets another than the box
    had, the fleet's workers and the clients' streams and their carriers
    (../configuration/stock.md), each at what the box had. `standard`'s
    `fleet.slots`, 4, holds the worker's `slots` with no override.
  `standard` holds the owner's grants (16), their names' segments (8),
  the notes' scopes past them, the watch room and the reference's
  charters and executors, so none of small's overrides for them stays.
- **Its worker's `limits`** are jig's `standard` profile with those of
  the hub's that it, sized for any agent, sets past smith's agent
  (../configuration/limits.md, the worker's check against the agent),
  and no other: `hub.accounts` 4, the agent's accounts; `hub.run_calls`
  48, the agent's calls; and `hub.event_bytes`, the agent's advertised
  message bytes. `standard`'s `hub.slots`, 4, holds its `slots`, 4, so
  that two chats and a goal's agents run side by side
  (../configuration/stock.md).
- **Its `application.limits`** are temper's `standard` with
  `checkout.workspaces` and `git_component.operations` 4, the least the
  worker's slots take (`../../domain/worker/README.md`, section 5);
  `checkout.message_bytes` the agent's payload, so the checkout accepts
  the agent's largest call; `forge.client.op_bytes` the engine's result
  bound, which the root holds it to, and `forge_component.body_bytes`
  21,341, which holds that write and its marker; `forge.judge_projects`
  8, jig's `authority.projects` in `standard`; the forge's repositories
  and output (3); and the turn's bounds below.
- **Its turn's bounds derive from smith's largest turn,** T, the
  agent's `turn_body`, which its configuration derives (../configuration/limits.md
  , the turn's check). The model's reasoning allowance,
  `reasoning_item`, is 65,536 bytes: a plan's model's reasoning items
  stay within it in ordinary use, so a chat on a plan is not held for
  its reasoning, and a person may lower it, with these bounds
  (../configuration/README.md, section 4). Each bound is the least the turn's
  check takes, past jig's `standard` profile's:
  - both hubs, `engine.limits` and the worker's `limits`:
    `hub.turn_bytes` T; `hub.transcript_bytes`
    W + (B − 4) × ⌊W / 4⌋, where W is `core.resume_bytes` and B is
    `size_of::<Box<[u8]>>()`: the window and a box for each turn it can hold;
    `hub.turn_queue_bytes` 8T, the agent's eight unacknowledged turns
    (the worker's check, above);
  - the worker's `limits`: `io.output` a turn of T's frame on its link,
    header included, past the profile's 1,048,576, so that the worker's
    link is made (jig's `protocol/channel/README.md`, 8 and 10);
  - `engine.limits`: `core.resume_bytes` T + 4, one largest turn with
    its length, so that a chat's window resumes whole up to that size;
  - `application.limits`: `root.journal.transcript_bytes` T + 4, the
    resume setting's bound; `root.loads.transcript_bytes` T;
    `root.loads.bytes` and `root.loads.reply_bytes` T and a row's slot;
    `root.loads.read_bytes` 2T, the turn and room as large again for
    the evidence a read carries with it (../configuration/README.md, section 9).

  A test holds the example's and `temper-box init`'s figures to this
  derivation from their agent, and its writer rewrites them:

  ```sh
  SKEIN_CODEGEN_WRITE=1 cargo nextest run --locked -p temper --test suite -E 'test(startup::turn_bounds)'
  ```

  jig's `standard` store page admits the frame of any turn smith admits,
  up to 33,554,432 bytes, and the store's frame and the worker wire
  carry it (../configuration/settings.md, 7; jig's `protocol/store/transcripts.md`, 1).
  The other KV bounds stay as they are: a turn's body is a transcript
  file's, never a KV value. Startup refuses a figure past its wire
  field's ceiling.
- **Its `memory_bytes`** are deployment budgets, never declarations of
  a service's worst case. `engine.memory_bytes` meters the engine's
  held bytes (`../../domain/root.md`, 4.1); no start compares memory with
  a sum or names a deficit (skein/docs/foundation/programming-model.md, 6.3).
  - The forge's API token is within `standard`'s, as `small`'s,
    `forge_component.token_bytes`, 512, past which the start is
    refused; an owned forge's longest `git` base is `http://` with a
    loopback address and a port, `http://127.255.255.254:65535`,
    28 bytes.
  - The worker's agent's `program` and the configuration's path among
    its `arguments` fit Linux's `PATH_MAX`, 4,096 bytes; Claude Code's
    path and programs directory, which the host's half holds to check
    a program's login (smith's `protocol/programs.md`, section 4),
    fit the same bound.
  - Its other addresses and secrets, logins, emails, binaries and
    charters, and LLM providers keep jig's limits.

  Channel frames and queues use max(the checked credential value bound,
  the 32-byte source name, the generated Identity bound), independently
  on each side; configured keys include jig's 4,096-byte ceiling, and
  workers reserve it without provider declarations. The program paths
  are retained once, separately from the per-slot spawn command paths.
  Startup checks the example's limits with its strings at their bounds,
  without starting a service. Artificial paths are never opened. A
  separate shell test prepares the example with ordinary paths.
  Program declarations survive startup; runnable program grants still
  wait for smith's program backend and jig's program sources.
- **Three loopback addresses,** Forgejo's (`listen`, `127.0.0.1:3000`),
  the clients' listener's (`127.0.0.1:7100`) and the webhooks'
  listener's (`127.0.0.1:7101`), are what a host changes to run several
  boxes on one machine, with each box's directories.

## 2. `temper-box init`

`temper-box init [DIRECTORIES] [--box]` writes a deployment that starts,
from its three directories (../configuration/directories.md) and what it
finds on the machine. Alone, it writes a bare deployment: no welcome, no
playground and no Forgejo on this machine.

- **The box's parts, one flag:** `--box` adds them: the welcome's
  member; the owned Forgejo, its person and its port, the OpenID provider and
  its key (`../openid-provider.md`, 9); and the
  playground, `application.demo` with its `demo/` files and its two
  ports. `temper box`'s first run passes it (`box.md`, 1); a person may pass it
  too. The document keeps them as three members, so that splitting the
  flag later changes no format (`README.md`, section 9).
- **Refused,** exit 1, before it writes anything, naming the file: when
  the configuration directory holds `temper.json`, or the data
  directory holds `worker.secret`, `worker.digest` or, with `--box`,
  `demo/llm.key` or `demo/bin/git`. It never replaces a file; a person
  who starts again removes them.
- **Its directories:** the data and cache directories made and checked
  as every command makes and checks them (../configuration/directories.md
  ), and the configuration directory made the same way when absent;
  an existing configuration directory is taken as it is; and, with
  `--box`, the data directory's `demo/` and `demo/bin/`, made private
  (0o700).
- **Its files,** each created new, in this order, the document last, so
  that a configuration directory holding one holds them all, the two in
  `demo/` with `--box` alone:

  | File | Directory | Mode | What it is |
  |---|---|---|---|
  | `worker.secret` | data | 0o600 | 32 bytes from the kernel's random source, as 64 lowercase hexadecimal digits and a newline |
  | `worker.digest` | data | 0o600 | the SHA-256 of the secret's digits without its newline, as 64 lowercase hexadecimal digits and a newline, as the walk `box` (`temper help box`) makes it |
  | `provider.key` | data | 0o600 | with `--box`, the OpenID provider's signing key, made as `owned.md`, 8's first step, and kept if already there (`../openid-provider.md`, 9); no such file when it can't be made |
  | `demo/llm.key` | data | 0o600 | the scripted model's key (`playground.md`): 32 bytes from the kernel's random source, as 64 lowercase hexadecimal digits and a newline |
  | `demo/bin/git` | data | a link | the playground's git, `temper-box fake git` (`playground.md`): a symbolic link to the binary |
  | `agent.json` | configuration | 0o644 | smith's agent configuration (../configuration/README.md, section 4): the owned example's endpoints with their limits and its profile (1), and, with `--box`, the scripted provider's endpoint and limits (3); Claude Code's program, its `path` the `claude` found on `PATH` or, with none, `"fetched"` (`owned.md`, 6), and its programs directory, `programs` in the data directory; and its endpoint and limits (3), whether or not `claude` is on `PATH` |
  | `temper.json` | configuration | 0o644 | the default document (3) |

- **What it finds,** each written absolute:
  - *the binary,* the running `temper-box`'s own path: the worker's agent's
    `program`, with `arguments` `["internal", "agent",
    <configuration>/agent.json]` and `configuration` that same path
    (../configuration/README.md, section 4), and, with `--box`, the playground's
    git link's target;
  - *git,* the `git` found on `PATH`: the worker's `git`; with none,
    `null`, written without refusing, and init says so (../git.md, section
    3). With `--box`, the playground's link, `<data>/demo/bin/git`, is
    the machine's `fake.git` (`init.md`), so that the playground's git
    is `temper-box fake git` (`playground.md`) whatever git the machine has, or none;
  - *Claude Code,* the `claude` found on `PATH`: the `path` of the
    agent's configuration's program; with none, `"fetched"`, temper's
    pinned build, fetched when a command of its source first needs it
    (`owned.md`, 6), written without a word, since Claude is offered either way
    (`../../client/pages/README.md`);
  - *the person,* with `--box`, the machine's owned Forgejo's (3): its `login` the
    name of the effective uid's entry in `/etc/passwd` (`geteuid`, then
    that file; on macOS, whose users are not in it, what `/usr/bin/id
    -un` prints), the login the person signs in to their Forgejo with; its
    `email` `LOGIN@localhost`. With no entry, or a name that is not a
    Forgejo login, is one of Forgejo's reserved names (such as `admin`),
    or is the bot's, `temper-admin` or `work`, the first repository's
    default org (`owned.md`, 7), the login is `person`, and init says so: no owned
    forge is set up until the person asks, so the person's login is
    never a reason the first run does not start;
  - *the bundle:* none to find. `clients.bundle` is always `null`, the
    binary's own (`../../client/web/README.md`, 2.6): a temper built with its
    browser starts its first run with it. From a binary built without
    one (`TEMPER_INTERNAL_BUNDLE=none`), init says the browser is not served;
    the link still signs in a terminal (`temper-box open`), and a browser
    that opens it is answered the clients' listener's page for no
    bundle (`../../client/web/README.md`, 2.2), never a bare `404`;
  - *the clients' listener's port:* 8367, when a socket binds
    `127.0.0.1:8367`, else the port the kernel assigns to a bind of port
    0; the port written fixed, as GitHub requires (`github.md`, 1). A later start on a port taken is
    refused, naming it and the document, as any start is;
  - *the machine's ports,* with `--box`: 8368, the owned Forgejo's
    (3); 8369, the playground's fake forge's, and 8370, the scripted
    model's fake LLM's (`playground.md`), each by the same rule;
  - *the webhooks' listener's port:* 8371, by the same rule (3). The
    sockets, the clients', these three with `--box` and the webhooks',
    are held until all the ports are chosen, so that no two are the
    same; then `temper-box init` closes them, and `temper box`, whose first
    run ran init in its own process, keeps them, and holds each for its step
    from then on (`box.md`, 1), so that no port it wrote is free between init and
    the step's bind. The port is written fixed, since the setup host gives the
    hook's address from the document at its start (`reconcile.md`, 3); a listener
    bound to port 0, its hooks given the address it reports, is later.
- **Choosing ports for another box:** keep the first running and give
  `temper-box init --box` three separate empty directories. Its occupied preferred ports
  fall back by the rule above, and both documents name the chosen ports
  consistently, including the machine's remotes and the scripted LLM
  endpoint.
  The saved addresses stay fixed; a separate init releases its
  reservations when it exits, so it does not reserve them for a later
  box. A first-run `temper box` on empty directories keeps the
  reservations through startup. The walk `box`, step 1, and the
  `commands` page's init entry explain these choices (`temper help`).
- **What it says,** on stderr: `init: wrote <path>` for each file;
  `init: this temper has no browser (built with TEMPER_INTERNAL_BUNDLE=none):
  the browser is not served` from a binary built without its bundle;
  `init: no git on PATH: a real forge's repositories need git
  2.41 or later, set as workers[].git` when there is none; with
  `--box`, `init: LOGIN cannot be a Forgejo login: the owned Forgejo's
  person is person` when the login falls back (above) and `init: uid UID
  has no login in /etc/passwd: the owned Forgejo's person is person`
  with no login; `init: after starting, continue with ChatGPT or add an
  OpenAI API key from the accounts page`; and last, `init: start it with
  temper-box serve`, or with `--box` `init: start it with temper box`, with
  the flags init was given. Its stdout is empty; exit 0.
- **Nothing else:** no forge, no account, no credential, no repository
  and no store. The engine's first start makes its store; accounts,
  forges and repositories are the client's (../credentials.md, section 3;
  `../../client/forge/README.md`), and the playground's forge, account, project
  and repository its start's (`playground.md`).
- **Its code:** `temper-init`, step code (`README.md`, section 7): given the facts
  above, git's and `claude`'s paths or none among them, the box's parts' or none, those
  being the person's login, the machine's ports and the scripted key's
  32 bytes, the three directories and the secret's 32 bytes, it answers
  the files, each with its directory, its name, its bytes and whether it
  is private, and, with the box's parts, the person's login, fallen back
  to `person` or not; it reads, writes and binds nothing. The binary's `init` finds
  the facts, checks the refusals and writes the files, on skein-io.
- **Checked:** temper-init's step tests check that the box's `engine.llm` is the
  owned example's with the scripted provider (3), that the bare
  document is the box's less its parts, that the digest is
  the secret's, the files' names, directories and order, and the
  machine's part with the person's login found and fallen back. The binary's
  crate tests, on scratch directories, check that each document init writes
  passes ../configuration/README.md, section 6, as `temper-box serve` checks it, with
  the agent's configuration it writes, at the fixed ports and at kernel's
  ports, always distinct, with and without `claude` and git, the
  Anthropic provider, its program, endpoint and model in both documents
  either way, the program's `path` the `claude` found or `"fetched"`;
  a second init while the first holds its five listeners, on separate
  directories, writes disjoint ports and matching listener, remote
  and scripted LLM addresses in both documents; `clients.bundle` always
  `null`; each refusal writing nothing, the person's fallback login
  among them; the files' modes; and the link's target. A box on a fresh init,
  among those tests, reaches `temper-box serve`'s listening with no forge
  and every group dormant: no `forge-N` directory, no group's file and
  nothing in the cache. The process
  tier starts a box on empty directories (`testing.md`, 4), and a story of its own
  starts the shipped box the same way and reads its welcome's Model step,
  ChatGPT's and Claude's cards, with no `claude` on `PATH` (`testing.md`, 1).

## 3. The default document

Sized for a person's real work: a project of fabrig's scale, its own
development, with five times the largest figure seen
(../configuration/stock.md), so a fresh box runs such a project with no
override.

- **The box's parts,** `temper-box init --box`'s alone (2):
  `application.welcome` `true`; the machine's `owned`; and the
  playground: the machine's `fake`, `application.demo`, the scripted
  provider in `engine.llm` and in the agent's configuration, and the
  reset's grant in `engine.acts`. A document without them is the same
  document less those members: the machine part `github` null alone,
  `application.demo` null, `engine.llm` the owned example's with
  Anthropic's provider (below), and no `engine.acts`.
- **No forge** (`init.md`): a first run's store holds none, and the
  person makes each from the client, none needing a restart: the
  playground's fake one by trying it (`playground.md`), an owned Forgejo by setting
  it up under the box (`owned.md`, 5), GitHub or a Forgejo elsewhere by connecting
  it (`github.md`, `reconcile.md`, section 5). A start resolves each kept forge's `api` name
  once (../README.md, section 11); a name that does not resolve, GitHub's
  on a machine without DNS, refuses no start: that forge starts not
  connected, and each connect of it, restored or asked, is answered
  unreachable, naming the host, until a start that resolves it. The
  other forges start as they would. A forge so held keeps its connection
  and its setup entry, built with the address `127.0.0.1:0`, which no
  listener can hold, and temper sends neither a connect nor a setup
  request for it: the placeholder never reaches a socket.
- **Its machine part,** `application.machine` (`init.md`):
  - `owned`, the box's owned Forgejo: its `forgejo` and `runner.binary`
    `"fetched"` (`owned.md`, 6); its runner the owned example's,
    `temper-host:host`, capacity 1, and its `path`; its `listen`
    `127.0.0.1:<its port>`, 8368 (2), and its `remote`
    `http://127.0.0.1:<its port>`; and its person the one init finds,
    `LOGIN@localhost` (2). Set up, it is forge N in `forge-N`, its
    handover `forge-N/admin_token`; its access and git value are
    records, with no other credential file written or read;
  - `fake`, the playground's fake forge: its `listen`
    `127.0.0.1:<the fake forge's port>`, its `remote`, and its `git` the
    playground's link (2);
  - `github` null, GitHub at github.com's addresses (`github.md`).

- **The webhooks' listener,** `application.webhooks.listener`,
  `127.0.0.1:<its port>` (2). Each Forgejo's `WebhookSecret` is a
  record, drawn by the setup plan that makes its first hook (`reconcile.md`, 4).
  Adoption makes the repository's hook; the fake's start-time adoption hooks its seeded
  repositories (`playground.md`). GitHub's hints come from its App's delivery log.
- **Its LLM providers:** the owned example's `engine.llm` with no
  account and no model tables: the binary supplies stock model settings,
  each with its `shown`, `efforts` and `effort` (../configuration/stock.md);
  2, `anthropic`, on every document, whatever is on
  `PATH`, so that the Model step offers ChatGPT and Claude
  (`../../client/pages/README.md`): `signed_in_plan` false, `manage_usage`
  `https://claude.ai/settings/usage`, its models Anthropic's of that table,
  through Claude Code's `--model` (smith's `protocol/programs.md`, 5.1),
  and its one source Claude Code's program, `claude_code`; one
  endpoint, `anthropic`, naming that program, Anthropic's dialect, 2, at
  `api.anthropic.com:443` on TLS, with OpenAI's bounds; its models'
  prices are that table's, figures a subscription bills
  to nobody; the agent's configuration declares them too (2), and the
  `standard` profile's `documents.lines`, 16, holds the offer's three
  lines, so that no override is written; and 3, `demo`, the scripted
  provider, the playground's (`playground.md`): one endpoint,
  `demo`, at `127.0.0.1:<the fake LLM's port>`, plaintext, OpenAI's
  dialect on its public route; one model, `demo`, priced as the owned
  example's model is, 1 per 1,000,000 for input, cached and output (jig
  and smith refuse a price of 0); its window `gpt-6-luna`'s, 1,048,576,
  and its output 8,192, so the scripted model leaves every charter the
  room luna leaves it: smith's host part, at the owned threshold, 4/5,
  holds the largest stock charter's tools and the authorised brief with
  a whole spec (`../../domain/agent/tools.md` and `../../domain/agent/charter.md`, 5). The message bound is
  the agent's (1), and a request at the threshold stays within the
  fake's request bound; its source a key. The agent's configuration
  declares it too (2).
- **Its projects:** none. Its `engine.projects` is `[]`: jig makes
  each first owner a deployment owner with no project configured
  (jig's `domain/people/README.md`, 3), and the engine, the playground and the
  client run a deployment with no project. Projects are made at
  runtime: the playground's start makes the playground's, "Demo"
  (`playground.md`), and the welcome's Ready the person's first, "Hello" for
  `work/hello` (`../../client/pages/README.md`). A deployment whose
  first start seeded a project, project 1 as the document seeded it
  before, keeps it: no start removes or renames it. One policy, `engine.policies` 1 (jig's
  `protocol/configuration.md`, section 3), which each project made
  starts from: its owner's grants, the rules' ceiling and its
  projections the owned example's, on the pattern below in place of
  the org's. Each project's landing rule 1 covers every branch of each
  of its repositories, the pattern `{"segments": ["branch"], "last":
  "open", "value": ""}`, `enforce`d, with `ci`, `up_to_date` and gate
  1, an agent's review, and no approval: a change lands on green CI and
  an agent's review with no person in between, in every project, seeded
  or made, until its owners change the rule (../configuration/settings.md, 2 and ../configuration/stock.md
  ). Each project is made on that policy
  by jig's `MakeProject` (`playground.md`, `owned.md`, 7). The policy and the landing rule are
  the binary's stock settings, policy 1 and every project's rule 1, and
  the document names neither (../configuration/stock.md); `temper-box init`
  writes no `engine.policies`, `engine.rules` or `application.landing`.
- **Its acts:** `engine.acts` (jig's `protocol/configuration.md`,
  section 3) grants the forge connector, 1, the playground's reset's
  acts (`playground.md`) and a made repository's adoption
  (`../../domain/forge/adoption.md`, 5), `{"connector": 1, "may": ["remove_project",
  "remove_account", "adopt"]}`, and no other connector any. The
  document's first owner, whom the grant needs, is its `owners`' first.
  A document without the grant still serves: `temper-box serve` starts on
  it, and only the reset's acts are refused, *not offered*: each is
  kept, and named in serve's log with the grant to add to
  `engine.acts`, the one `temper-box init --box` writes, until a start's document
  grants it.
- **Its charters and templates** are the binary's stock
  (../configuration/stock.md): charters 1, 2, 4, 5, 6, 9, 10 and 11 and
  templates `chat`, `goal`, `goal for a draft` and `ask`, numbered
  1, 2, 4 and 5. Their model choices include the scripted model,
  `demo`, after their first choices and the rest of the real catalogue.
  A task with no model chosen takes the
  first of its charter's models whose provider has an account usable for
  its payer (jig's `domain/accounts.md`, 4.3), so in the playground,
  whose project's one account is the scripted one (`playground.md`), a chat and the
  coding agent of the change it delegates both run on the scripted
  model, with no pick. jig takes it at each run's check until the task
  keeps an account: a chat started before an account of a real
  provider stays on the model it took, and a chat started after takes
  the first of its charter's models with an account usable,
  `claude-opus-5-5` once a Claude login is. The composer's line says
  which, before the start, from jig's `resolved_default` preview in
  `me`, never by guessing from catalogue order (`../../client/pages/accounts.md`,
  2). The request keeps its omitted choice; the preview does not turn
  it into an explicit one. The charters, the
  templates and the forge's charter are the binary's stock settings,
  their models first choices and the rest of the catalogue
  (../configuration/stock.md), the scripted model last where `demo` is
  configured; the document names
  none of them (../configuration/stock.md), and a person changes them from
  "How work runs" (`../../client/pages/settings.md`, 2).
- **One worker,** in the process, with 4 slots (1), its `root` `workspaces` in the data
  directory, its `git` the machine's, or null (2); the deployment's directory
  `deployment` in the data directory; the clients' listener
  `127.0.0.1:<port>` (2).
- **Its sign-ins:** `engine.clients`' `link` and `approval` both
  `true`, in both documents and the owned example (1): the printed
  link (`box.md`, 2) and a terminal approved by the person's browser
  (`../../client/README.md`, 4.1), on the clients' loopback listener
  (../configuration/engine.md, clients). The box prints the link
  alone; the approval prints nothing.
- **Its limits:** the owned example's (1), the `standard` profiles,
  among them `forge.repositories`
  and `forge.client.repositories` 3, counts of the deployment's
  (../forge/README.md, section 6), and `forge.output` 82. Its `memory_bytes` are
  deployment budgets, as the owned example's are (1). Startup checks
  its limits with its strings at their bounds, the machine's
  `fake.git`, which the engine names to the worker, among them; the two
  literals of `temper-init`'s forgejo-elsewhere/demo document are
  checked the same way.
- **`application.demo`,** the scripted model's fake LLM (`playground.md`).
- **Its box's steps** (`box.md`, 1): `temper-box serve`; the fake kind's group, the
  fake LLM, a fake forge's preparation and the fake forge, dormant until
  the person tries the playground (`playground.md`); and the owned kind's group,
  dormant until the person sets up an owned Forgejo: its fetch, its
  preparation, Forgejo's web and the runner's daemon; the engine takes
  the handover and reconciles (`owned.md`, 5).

## 4. Open questions

- **A start holding the engine's documents to temper's clients** (`init.md`, 1):
  no start compares `documents.text_bytes` with what the web and terminal
  clients receive, so a deployment past 512 starts, and those clients
  refuse a document whose text is longer. A test holds `temper-box init`'s
  documents to them; a deployment's own override is held by nothing.

- **The deployment's own UX:** what `temper-box init` alone writes and says
  beyond `init.md`, 2, and how a server is set up. For now it is the cheapest
  bare deployment.

- **Flags of their own** (`--welcome`, `--playground`), when a
  deployment wants some of the box's parts and not others (`init.md`, 2).

- **Four agents on a laptop** (`init.md`, 1, 3): measure their combined resident
  memory, including any program an agent launches, after landing. The
  box-and-serve figures in `README.status.md` exclude agent processes and
  do not size four of them. Each agent declares 2 GiB, carried unenforced;
  the start checks slots and their related limits, without comparing that
  declaration with laptop memory. The default is four slots, reversible
  by the person; workspace disk use depends on the repositories cached
  in the four workspaces. The process, browser and live checks follow
  landing (`../../testing/README.md`, section 7).

- **Discovering models per account** (`init.md`, 3): provider-list reads and their
  retained names are the catalogue's lists step
  (`../../draft/model-catalogue.md`, sections 5 and 7). Default: the binary's
  stock and model settings added by name through Settings, Models or
  Ask temper; neither generated configuration document holds models.

- **The LLM providers out of the document,** each a design of its own;
  the scripted provider, the playground's (`playground.md`), may become the binary's
  stock first, its endpoint the fake LLM's port, the machine's.

- **Good defaults for a person's real work:** landing rules and the
  memory bound. Default: `init.md`, 3's, refined once a first run works; the
  charters and limits are sized for a project of fabrig's scale
  (../configuration/stock.md).

- **A task's text past what the clients receive** (`init.md`, 1). A client reads a
  spec in pieces of `documents.text_bytes`, 512, where the request
  `first-goal-fixes` of 2026-11-05 has 8,192. Going past 512 needs the web and
  terminal clients' `text` raised (jig's `client/domain/contract/README.md`, 4).
  Default: 512. The user's to choose, with no answer yet. A run's `read_task`
  is outside it where its worker or its process's local agent names a reply
  bound, reading in that room (`../../domain/routing.md`, 2).
