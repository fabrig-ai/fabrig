# temper's configuration

Draft, 2026-11-16, from the user's words that day (`notes.md`, section 21). This draft covers what a person writes to run temper, and where everything else lives. It does for the configuration what `cli.md` did for the command line: the files a person writes become simple and mostly human-readable, and the complexity moves into temper, where a person changes what's needed by asking temper (`ask-temper.md`). It builds on `configuring.md`, which moved how work runs into the store's settings, and finishes that move. The limits, the LLM providers, the agents' figures, the machine's places and the list of workers leave the file too, and what's left is a few lines of TOML. The workers' lifecycle comes with them, since it shapes the files: each worker becomes a setting an owner adds, changes or removes without a restart (6.7). Its facts come from a read-only survey of main at `5a3717cde3`, and from dev.fabrig.ai's documents, read that day without their secrets (section 2).

## 1. In one page

- **Every value has one home** (section 3):
  - the file holds what a start needs before it opens its store, and what belongs to the machine;
  - the store holds everything else a person may change, as settings, changed by asking temper;
  - everything else temper derives from those, or keeps as a constant, and writes nowhere.
- **The file is TOML, and a few lines long** (section 4). It's `temper.toml`, flat, with comments and names a person reads. A box's stock file is one line, and a worker's one or two. No part of it mirrors how the code is layered, as jig's `engine`, `workers` and `application` do today.
- **There's one stock and no profiles** (section 5). The binary's figures cover twice the scale of fabrig's own development, as dev.fabrig.ai ran it on 2026-11-16. `small` and `standard` leave the file and the help, and `small` stays for tests alone. Raising a ceiling costs nothing at rest, since temper's containers grow only as they're used. No stock figure is a test's.
- **Limits are settings** (6.1). An owner changes one by asking temper: a raise applies at once, a lowering at the next start. A limit reached names its setting, and Ask temper offers the raise. Nothing asks a person to edit a file and restart.
- **What can be derived is never written** (section 7). Every figure that a start's check ties to another is computed from its source: a turn's carriers, an inbox's bytes, a worker's workspaces and git operations, the deployment's places. No start is refused because two numbers in a file disagree.
- **The workers' and agents' figures come from the engine** (6.3). A worker's file says where its engine is. Its limits, its agents' profile and their endpoints come with its opening, so nobody keeps two copies of anything.
- **Workers are settings** (6.7). Each has its name, its places and its secret's digest. An owner adds, changes or removes one by asking temper or with `temper ctl worker`, and the engine isn't restarted. A worker is down when its link is lost, as today.
- **The LLM providers and the machine's places move to the store** (6.2, 6.4). The stock's providers are defaults, and one added or changed is a setting. An owned or fake forge's place is chosen when the forge is set up, and kept in its record.
- **Upgrades are cheap, for now** (section 8). No build reads a member that moved: each milestone lands compatible with dev.fabrig.ai, or with a few steps for its operators, which its row in the execution file lists. Upgrades take more care once v0.1.0 has users.
- **Each file has one table** (4.5). Help, the reference pages and `init` are rendered from it, and code reads the file only through it.

## 2. What there is

From a read-only survey of main at `5a3717cde3`, 2026-11-16, and dev.fabrig.ai's documents, read that day through a filter that kept only their limits, slots, graces, profiles and member names.

- **The stock is long.** `temper-box init --box` writes a `temper.json` of 631 lines and an `agent.json` of 100; `temper-box init` alone writes 260 and 82. Of the box's file, about 340 lines are the playground's scripted model and about 140 the LLM providers. Its top level is jig's three parts, `engine`, `workers` and `application` (`../protocol/configuration/README.md`, section 2), so a person meets the code's layering before what they came to set.
- **Overrides are everywhere.** The stock carries 40 overrides beside its profile (`temper/docs/examples/owned.json`), many of them figures derived by hand, such as `"hub.turn_bytes": 16803810`. dev.fabrig.ai's engine document overrides 52 limits (29 of jig's, 23 of temper's), and each of its three workers' documents 27: 133 in all.
- **There are copies, and they drift:**
  - **Unread overrides.** A worker applies only four of the 18 `application` overrides in dev's workers' documents (the checkout's message, workspaces and local timeout, and the git component's operations). It accepts the rest unread (`temper/crates/temper-service-core/src/limits.rs`). dev's workers say `forge.client.answer_bytes` is 255,570 where the engine says 1,048,576, and nothing notices.
  - **The agent's figures.** The agent's 11 profile figures and 10 endpoint limits are copied on every worker. Its endpoints copy the engine's `engine.llm`, and the start compares their names alone, not their addresses, transports or timeouts.
  - **Slots.** A worker's slots are written five times: its `slots`, its `hub.slots` and `fleet.slots`, its `checkout.workspaces` and its `git_component.operations`. The engine's `fleet.slots` and `hub.slots` repeat them again. The engine admits each worker at the smaller of its hello's slots and its own `fleet.slots`, per worker, not summed, so dev's 36 could be 12.
- **Some figures are coupled.** At least 12 figures follow from two sources by arithmetic. The sources are the largest tool payload (dev's is 63,740 bytes) and the agent's largest turn, which smith derives from its profile (dev's is 18,431,842). The coupled figures:
  - a hub's turn;
  - the core's resume, the turn plus 4;
  - the hub's transcript, from the resume;
  - the queue of eight turns;
  - the loads' bytes and reads;
  - a message's and a result's bytes, the payload less 2;
  - the inbox, 82 of those;
  - the forge's body, twice an op plus 21.

  A start refuses a file whose carrier is short, naming it, and the person fixes it by doing the sum.
- **Many defaults are below real use.** 43 limits' stock values are below dev's. For example:
  - tasks, 128 against 384;
  - placed holds, 16 against 256;
  - client streams, 8 against 32;
  - the brief's gathering, 30 s against 300 s;
  - the checkout's local timeout, 10 s against 60 s.

  Many limits cite a test world as their source in their comments, the forge's pending, calls, window, op bytes, rows, inbox and entries among them (`temper/crates/temper-engine-service-vocabulary/src/limits.rs`). On 2026-11-15 the forge outbox's 8 entries held two changes for good, and the hold told the person to raise the limit and restart.
- **Profiles buy little.** `small` and `standard` are sets of admission ceilings. temper's maps, lists, slabs and queues allocate nothing until they're used (`skein/crates/skein-lib/src/map.rs` and its siblings), so a ceiling costs memory only as work fills it.
  - What allocates ahead does so per connection or per child: a TLS client's buffers (about 20 KB for a forge, 133 KB for a sign-in's), and 512 bytes per git child.
  - An absent profile refuses the start.
  - dev's engine runs `standard` with its 52 overrides. With 17 live goals it held 38 MB resident.
- **The memory budget isn't the process's memory.** `engine.memory_bytes`, 1 GiB in the stock, meters the transcripts and turns the root holds: a task waits to prepare while they're past it. A worker's `memory_bytes` is read by nothing.
- **Nothing in the file changes live.** Each component copies what it's given when it's constructed. The store's settings already change live: a change is accepted while temper runs, and each kind is read where it's used (jig's `domain/engine/settings.md`, section 5). temper declares 13 kinds, but none holds a limit, a listener or an endpoint, so Ask temper can change none of them.
- **`init` already finds what it writes.** It finds its own binary, `git` and `claude` on `PATH`, picks free ports, and works out the agent's paths and arguments (`temper/crates/temper-init/src/lib.rs`). It then writes each into the file, where it stays.

## 3. Where each value lives

Every value temper reads has one of four homes, and its file's or kind's table (4.5, 6.1) says which.

| Home | What | Changed by | Applies |
|---|---|---|---|
| **The file** | What a start needs before it opens its store, and what belongs to the machine: where it listens, its public address, where its engine is, a program it can't find | Whoever runs the machine, by editing it | At the next start |
| **The store** | Everything else a person may want to change: limits, LLM providers and endpoints, the agents' figures, the workers and their places, the memory budget, the welcome. These sit beside the charters, rules and policies already there (`configuring.md`, section 3) | An owner, by asking temper, on the settings page or with `temper ctl`, as a decision (`../protocol/configuration/settings.md`) | At once, or at the next start where the setting's row says so |
| **Derived** | Every figure that follows from others: carriers, workspaces, the deployment's places, the stop bound | Nobody: temper computes it from its sources | When its sources change |
| **Constant** | What nobody has needed to change: the graces, the heartbeat, the deployment's seed, the forge connector's number, the playground's scripts | A landing | At the next build |

- **The file's rule:** a member is in the file only when the store can't hold it. Either the start needs it before the store is open, or it describes this machine rather than the deployment. Anything else a person may change is a setting.
- **A setting's rule:** a running engine takes it at once, unless something was sized, bound or agreed with it at start. Then its row says "at the next start", and the deployment's page shows the setting waiting for one (6.1).
- **A constant's rule:** it becomes a setting the first time someone needs to change it, by a landing. No constant is a file member just in case.
- **Defaults are the binary's.** The file and the store hold only what departs from them. `init` writes no default, and the settings page shows a default as one.

## 4. The file

### 4.1 Its format

- **It's TOML:** `temper.toml` in the configuration directory (`../protocol/configuration/directories.md`), read whole at start and strictly. An unknown key, a wrong type or an out-of-bounds value refuses the start, naming the key and its line.
- **It's flat, named by what a person means:** `listen`, `public`, `engine`, `owner`. A table appears only where a group means something of its own: `[sign_in.google]`, `[programs]`, `[environment]`, `[internal]`.
- **Its values are plain.** An address is `"127.0.0.1:7100"`, and a path a string, typed by its directory as today (`../protocol/configuration/directories.md`). No key takes a duration or a size. One that comes to follows smith's `config.toml`, whole seconds with the unit in the key's name (smith's `shell/settings.md`, 1.2). The JSON's tagged forms (`{"socket": ...}`, `{"in_process": ...}`, integers named `_ms`) are the code's, not the file's.
- **It's read by the `toml` crate,** already the workspace's for smith's `config.toml`, through its parser alone (`toml::de`), which needs neither serde nor its writer. Its errors and spans give each refusal its line. temper uses TOML nowhere else, and `init` writes the file as text from the table (4.5).
- **It's written once.** `init` writes each line with its comment. Nothing temper does writes the file after `init` (`upgrades.md`, section 10).
- **It holds no secret,** as today. A secret is a file that the file names, in the data directory, at mode 0600 (`../protocol/configuration/README.md`, section 5).
- **Its shape is its binary's,** as each binary refuses a document that isn't its own (`deployment-binaries.md`, section 4): `temper-box` reads a standalone file, engine and worker in one process; `temper-engine` an engine alone's; `temper-worker` a worker alone's. Each binary has its own table (4.5), so a key of another shape is unknown and refuses the start, naming its line. The cluster's tests on one machine keep their loopback shapes (`../protocol/cluster.md`, section 1).

### 4.2 A box's

What `temper-box init --box` writes:

```toml
# temper's configuration. `temper-box help config` says what each line does.
# Everything else is changed by asking temper.

listen = "127.0.0.1:7100"   # where the client connects
```

- **`listen`** is written because `init` chose it, a free port on loopback. The webhooks' listener is the next port on the same host unless the file says otherwise.
- **The worker is in the box,** with 4 places, its root `workspaces` in the data directory, its `git` and `claude` found on `PATH` at each start, and its agent temper's own binary. A box that wants more places asks temper, as for any worker (6.7).
- **The owned Forgejo and the playground's fake forge** take their places when they're set up, each on a free port that its record then keeps (6.4).
- **The first owner** is the box's user, by the login `init` reads, as today (`../protocol/setup/init.md`, section 2), so the file names none.
- **The welcome** shows on a deployment made by `init --box`, as today. It's a setting from the first start on (6.5).

### 4.3 An engine's and a worker's

An engine alone's file, as a person writes it from `temper-engine help config` (a cluster's `init` isn't designed: `deployment-binaries.md`, section 3):

```toml
# temper's engine. `temper-engine help config` says what each line does.
# Everything else is changed by asking temper.

listen  = "127.0.0.1:7100"              # the clients' listener, behind your proxy
public  = "https://temper.example.com"  # where people reach it
workers = "10.0.0.1:7200"               # where workers connect, on your private network
owner   = "sam@example.com"             # the first owner, read at the first start alone

[sign_in.google]
client = "1234.apps.googleusercontent.com"
secret = "secrets/google.secret"         # in the data directory, mode 0600
```

A worker alone's:

```toml
# temper's worker. `temper-worker help config` says what each line does.

engine = "10.0.0.1:7200"   # where its engine listens for workers
name   = "w1"              # the name its engine knows it by, if not the host's
```

- **The engine's file lists no worker.** Workers are settings (6.7), and the file says only where they connect.
- **A worker's name** is its host's name unless `name` says otherwise. Its secret is `secrets/worker.secret` in the data directory, and its root `workspaces` there. Its places are its setting's, so its file has no `slots`.
- **The workers' channel stays as today:** jig's link in plaintext on the workers' listener, between hosts over a private network (`../protocol/cluster.md`, section 3). Carrying it over the public address with TLS is a design of its own. It would take `workers` out of the engine's file and give a worker's `engine` a URL, and change nothing else here.
- **The engine's file holds no limit, provider, place or grace.**

### 4.4 Every key

The public keys, which help lists:

| Key | Shapes | Default | What it says |
|---|---|---|---|
| `listen` | standalone, engine | 127.0.0.1:7100 | the clients' listener |
| `public` | standalone, engine | none: loopback only | the address people reach, through a proxy that holds its TLS |
| `webhooks` | standalone, engine | the next port after `listen`'s | the forge's webhooks' listener |
| `owner` | standalone, engine | a box's user | the first owner's address, read at the first start alone; later owners are the store's |
| `[sign_in.google]` `client`, `secret` | standalone, engine | none | Google's sign-in, its client and its secret's file |
| `workers` | engine | none | the workers' listener, on a private network |
| `engine` | worker | none | its engine's workers' listener |
| `name` | worker | the host's name | the name its setting has (6.7) |
| `[programs]` `git`, `claude`, `forgejo`, `runner` | any | found on `PATH` at each start, or fetched as today | a program that isn't found, by its path |
| `[environment]` | standalone, worker | the worker's own `PATH` and `HOME` | variables its agents' programs get, by name |

- **Three sets, as the command line's** (`cli.md`, 3.3): the public keys above; internal ones under `[internal]`, which help doesn't list, such as `bundle`, a client bundle of another build; and test ones, which only tests write, under `[test]`.
- **A key whose value equals its default** is accepted, and `init` never writes one.

### 4.5 One table per file

- **One table declares every key** of a file, in the crate that reads it, as `cli.md`'s table does for a binary's commands (`cli.md`, 3.5). Each row gives its key, its set, its type and bound, its default, its shapes, and a line on what it says.
- **Everything is rendered from it.** Help (`temper-box help config`, `temper-engine help config`, `temper-worker help config`) and the reference page `configuration` come from the table, replacing `configuration` and `configuration-engine` (`../protocol/documentation.md`, section 8). So does `init`, which writes each line with its comment.
- **Code reads the file only through the table.** A check holds it, as `cli.md`'s holds a binary's arguments (`cli.md`, 3.8).
- **A member that moved is refused as unknown,** naming it, and the `configuration` page lists where each went (6.5, section 8).

## 5. The stock

### 5.1 One stock

- **The binary's figures are one set, the stock.** The file names no profile. Help and the reference pages print the stock's figures where they print `small`'s today (`../protocol/documentation.md`, section 8).
- **`small` becomes a fixture.** A test that needs small figures takes them from a fixtures crate beside the limits' crate, as `cli.md`, 3.2 makes fixtures crates, never from a profile that a document names. jig's and smith's profiles likewise stop being a document's choice: temper's stock is jig's and smith's figures with temper's over them.
- **Each limit's row in its table** gives its stock figure, what it was sized from, its unit, whether a raise and a lowering apply at once or at the next start (6.1), and a line.

### 5.2 Sized from what

- **Counts at twice dev.fabrig.ai's figures** where those are above `standard`'s, and at `standard`'s elsewhere, rounded up to a round number. Counts are the tasks, holds, inboxes, charters, executors, changes, subscriptions, resources and client streams. dev's 17 live goals become a stock of 40, and its 384 tasks become 768.
- **A count a format caps stays at the cap,** never below dev's, while nothing needs more: a worker's `fleet.slots` at 40, the most the link's 32-bit output cap holds with the stock's resume window, and `forge.landing_members` at 5, the worker wire's `Prefixes` within the channel's 1 MiB frame (`../protocol/configuration/limits.md`).
- **`forge.changes` is 768, its store raised for it.** A tier row holds up to `2 * forge.changes + 1` landings, and the store's ceiling of 759 caps `forge.changes` at 379, dev's own figure (`../protocol/store.md`, 4). That ceiling is set by the row's worst case, which passes the store's 32-bit length at 1,537 only because each blamed landing may copy 1,024 failing tests' ids of 4 KiB. So the store holds a failing test's id within 1 KiB, and the tier records' lists of landings hold 1,537. The stock's `forge.tiers.test_id_bytes` is 512, twice the longest id today (about 221 bytes, a fuzzy case's), and the start refuses one past 1 KiB. Nothing stored is lost: dev's ids are within its 256 bytes.
- **Times at dev's,** since they're the work's pace, not its scale: the brief's gathering 300 s, the checkout's local timeout 60 s, a tool's deadline an hour, a check's timeout 300 s, an endpoint's output 48,000 tokens.
- **The tool payload at 64 KiB,** above dev's 63,740 bytes. Every carrier follows from it and from the agent's profile (section 7).
- **Two figures, for whoever needs more.** The counts that scale with work in flight are written in code as formulas of `live_goals`, the most goals live at once, which the stock sets at 40. The carriers are written as formulas of `tool_payload`, the largest piece a tool writes. Both figures are settings (6.1), so "plan for 100 live goals" is one decision. Places, every worker's together, are the sum of the workers' settings' places (6.7), never written.
- **No stock figure is a test's.** Each stock figure's constant says what it was sized from: a figure of dev's, a formula, or a design's bound. A figure whose comment cites a test world is sized again. A test that needs another figure states it in its fixture.
- **The stock covers dev.fabrig.ai.** A focused test holds every stock figure at or above the deployment figures this draft records in the limits' crate, dev.fabrig.ai's of 2026-11-16, named as fabrig's own development.
- **The sign-in's lifetime stays a day.** dev's seven days isn't scale but its automations' sign-ins, so dev keeps it as an override with its reason (6.1).

### 5.3 Memory

- **At rest, raising the stock costs nothing:** its ceilings are admissions, not allocations (section 2). The engine at rest stays where dev's is, a few tens of MB. What allocates ahead is per connection and per child, and its figures don't change.
- **Under load, memory follows the work held.** The root's budget, `memory_bytes`, leaves the file. It becomes a setting, `transcript_memory`, 1 GiB in the stock, taken at once: the memory the runs' transcripts and turns may take while work prepares, not the process's. New work waits while it's full. Its name says what it bounds, so nobody reads it as temper's own memory. The deployment's page shows the held figure against the budget.
- **A worker's `memory_bytes`,** read by nothing, goes.

## 6. Settings for what moved

### 6.1 Limits

- **Two kinds, one page.** jig's core gains a kind for its own limits, so its reference binary has them too, and temper declares one for its own (jig's `domain/engine/settings.md`, section 5). Each kind's value holds its figures (`live_goals`, `tool_payload`) and named overrides, each override a name, a value and a reason of at most 120 bytes. One page and one proposal show both kinds, as one list by name.
- **A raise applies at once, a lowering at the next start.** Every count ceiling and every time bound takes a raise at once: each child holding a copy takes the new figure through the settings' change, and raising a lazily grown container's ceiling changes nothing else. A lowering waits for the next start, where the restore refuses one below what the store holds, naming it, as settings already do (jig's `domain/engine/settings.md`, section 7). A wire's bound (a frame's, an opening's term) and anything sized or bound at start take either at the next start. Each limit's row says which applies.
- **The settings page shows every limit:** its stock, its value, its override's reason, and "at the next start" for one that waits.
- **Nothing the store needs comes from the store.** The store's own bounds, and what reads it, are derived from the stock alone (section 7), so a start never needs a setting to open the store that holds it.

### 6.2 LLM providers and endpoints

- **The stock's providers are defaults:** OpenAI and Anthropic, with their sign-ins, keys, programs, endpoints, and the models' keep and drop lists. `engine.llm` leaves the file. This is the design `../protocol/configuration/settings.md`, section 1 leaves for later.
- **A provider added, changed or removed is a setting,** such as a local model's endpoint on loopback. It's jig's kind, decided by an owner, and applies at the next start, since the LLM connection builds its endpoints at start.
- **The playground's scripted provider is the box's constant,** with its scripts in the binary, its port chosen and its key made at the playground's start. `demo` leaves the file.

### 6.3 The workers' and agents' figures

- **A worker's figures come from its engine:** the hub's, the checkout's and the git component's limits, and its agents' profile and endpoints. They arrive in the answer to its opening. The workers' wire moves its version, declared breaking, so an engine and its workers move together (`../protocol/wires.md`, 3.2; section 8). A worker pins them for its agents as today's file is pinned (`../protocol/configuration/README.md`, section 4).
- **No person writes `agent.json`.** temper writes the pinned configuration in the worker's data directory, from what the engine sent and what the machine found: its programs and its environment.
- **A worker's places are its setting's** (6.7). The engine admits each worker at them and sends them with its figures, and the worker derives its workspaces, git operations and hub slots from them (section 7). A change reaches the worker at its next opening, without restarting the engine.
- **A worker's document loses its copies:** its limits, its `agent` and its `agent.json` are refused by name, and its operators remove them (section 8).
- **The agents' figures are a setting** of temper's: the profile's declared figures and policy, and an endpoint's limits. A change reaches each worker at its next opening, so its row says "at each worker's next start".

### 6.4 The machine's places

- **A forge's place is chosen when it's set up.** That covers an owned or fake forge's listener and remote, and the owned Forgejo's runner: a free port on loopback and binaries fetched, unless the person setting it up gives others. The forge's record keeps them, as it keeps them once made today (`../protocol/forge/engine.md`, section 1). `machine` leaves the file.
- **The owned forge's person** (its login and email) is the box's owner's, derived. The owned Forgejo's sign-in through temper (`engine.clients.provider`) is made with the owned forge.
- **GitHub's address is github.com's.** A fake GitHub's is a test's.

### 6.5 The rest

| Today's member | Becomes |
|---|---|
| `welcome` | a setting, its default from how the deployment was made: on for `init --box` |
| `engine.memory_bytes` | the setting `transcript_memory` (5.3) |
| `engine.hosts.workers` | the workers' settings (6.7) |
| `engine.clients`' `heartbeat_ms`, `link`, `approval` | constants, the link and the approval on |
| a worker's `graces` | constants as `temper-box init` writes them; the engine's fleet grace derived above the workers' stop bound (section 7) |
| `programs.check_interval_ms` | a constant |
| `engine.acts`, `engine.connectors`, `engine.providers`, `engine.deployment` | constants |
| `engine.owners` | `owner`, at the first start alone (4.4) |
| `engine.projects`, `application.landing`, `application.charter` | made at runtime, or settings already (`configuring.md`, section 3) |
| `application.clients`' `listener` and `public` | `listen` and `public` |
| `application.clients.bundle` | `[internal] bundle` |
| `application.workers`' `root` and `git` | the data directory's `workspaces`, and `git` found on `PATH` |
| a worker's `agent` | derived: temper's own binary, its pinned configuration temper's to write (6.3) |

### 6.6 Asking temper

- **Ask temper can reach the new kinds.** It reads them on its reference pages (`settings-fields`, and the stock's `limits` page) and proposes a change as it does for any kind (`ask-temper.md`, section 5), for an owner to accept.
- **A limit reached names its setting.** The hold or refusal names the setting, its value and what wanted more, and says to ask temper to raise it. Needs you holds a proposal of the raise, ready to accept. The held work goes on when room frees or the setting rises, never only at a restart.

### 6.7 Workers

- **Each worker is a setting** of a kind jig's core gains, `worker`, numbered as models are, a number never given to another name (jig's `domain/engine/settings.md`, section 1). Its value is the worker's name, its places and its secret's digest. The kind's bound is the fleet's configured workers, whose stock covers dev's three with room.
- **An owner changes them as any setting:** by asking temper, on the settings page, or with ctl's shorthands, each one settings change:

  | Command | Does |
  |---|---|
  | `temper ctl worker add NAME --places N --digest HEX` | adds a worker |
  | `temper ctl worker places NAME N` | changes its places |
  | `temper ctl worker remove NAME` | removes it |
  | `temper ctl worker list` | each worker, its places, whether it's up, since when, and its runs |

  A worker runs agents with the deployment's accounts, so adding one is always an owner's decision. Ask temper proposes one and never makes it.
- **Admission reads the settings.** At each opening the engine takes a worker's name and secret against its setting's digest, as it takes them against a digest file today (`../protocol/cluster.md`, section 3). A name with no setting is refused, `not-admitted`, and the worker redials as it does today, so a worker started before its setting links once it's added. The engine isn't restarted for any of it.
- **The secret stays on the worker.** Its operator makes it on the worker's host, as `../../help/server.md`, section 2 does today, and gives the setting its digest. A digest isn't a secret, so it may travel as an argument and show on the settings page.
- **Places apply at the worker's next opening.** The engine admits a worker at the smaller of its setting's places and its hello's slots, until the worker takes its places from the engine (6.3). A linked worker's new places wait for its next opening: its operator restarts it, and its drain lets its runs end (`redeploys.md`, 6.1).
- **Removing a linked worker is refused,** naming it. Its operator stops it first, and its runs end interrupted, as at any stop. A worker that isn't linked is removed at once: if it was lost, its runs were placed again after the engine's grace, as today.
- **Up or down is its link's state,** as the fleet keeps it today: up while linked, down once its link is lost. `worker list` and the settings page show which.
- **A box's worker is the setting `local`,** made at the box's first start with 4 places and no digest, since it opens in process. It can't be removed.
- **Left for later:** a join that makes the secret and gives its digest by a one-use token; the channel over the public address (4.3); places proposed from a machine's CPUs and memory. None of them changes the files beyond `workers` and `engine`.

## 7. Derived, never written

- **Formulas move from the start's checks into the stock's code.** Each figure with a source is computed from it, at the least that passes today's check, with its margin in the code:
  - **the agent's turn** comes from its profile, as smith derives it;
  - **the turn's carriers** come from the turn: a hub's turn at least the turn; the core's resume the turn plus 4; the hub's transcript from the resume; the turn queue the agents' unacknowledged turns times the turn; the loads' bytes, reads and replies; the output's bound;
  - **the payload's carriers** come from the payload: a message and a result 2 bytes less, an inbox the agent's inbox times that, the checkout's message at least the payload, the forge's body twice an op plus 21.
- **Places:** a worker's hub slots, workspaces and git operations are its places. The engine admits each worker at its setting's places and has no hub slots of its own. The engine's limits on workers, those configured and those in contact, follow from the workers' settings' count, and the limits that scale with places from their places' sum.
- **Clients' carriers:** the exchanges and calls are the larger stream bound plus four, as jig's rule has them (jig's `protocol/limits.md`).
- **Graces:** a worker's stop bound comes from its graces, the engine's fleet grace above it, and a unit's stop timeout above that, as `temper help server` gives it.
- **An override of a derived figure** is still possible by name, with its reason, for a case its source doesn't cover. One below its derivation refuses the start, naming both.
- **The checks stay, but check the code.** Each formula has a focused test that its result passes today's check, at the stock and at twice it, so the checks hold temper's derivations rather than a person's arithmetic.

## 8. Upgrades

By the user's word of 2026-11-20, the configuration crosses a build as `upgrades.md`, section 10, has it. Until then this draft departed from it, an upgrade being cheap rather than smooth until v0.1.0 has users (`notes.md`, section 21), and its milestones landed with their operators' steps.

- **Each build reads its own members and the previous release's.** A member that moves or is renamed is read from its old place too, with one warning naming the new one. Once dev.fabrig.ai runs a build that reads the new form, the old place is refused as unknown, naming it, and the `configuration` page says where each went (6.5): as for the store, no code is kept for a configuration older than dev.fabrig.ai's (`store-floor.md`).
- **The same holds for a worker's configuration and the small files a build reads at its start:** a secret's file renamed, an environment entry moved, the forge's setup file.
- **A change no plan derives,** such as a member split in two or a unit changed, brings a written converter in the build, tested against a copy of dev.fabrig.ai's configuration, and never an operator's step.
- **The registered store's changes use defaults, former names or a written job,** with no version move or patch (jig's `protocol/store/layouts.md`).

## 9. Acceptance

Plans use the focused and fuzzy suites alone. Stories in slower tiers are written and compiled in their plans, and run after landing.

- **The file:** each shape's stock file starts its shape; a key of another shape, an unknown key, or a bad value is refused, naming its line; help and the `configuration` page are rendered from the table; a check holds every read of the file to the table.
- **The stock:** every figure is at or above the deployment figures recorded in the limits' crate. No constant's comment cites a test world as its source.
- **Derivations:** each formula's figure passes today's check at the stock and at twice it.
- **Limits as settings:** in a world, work held on a count goes on once a raise is accepted, with no restart; a lowering waits for the next start, and a restore refuses one below what's held, naming it.
- **A limit reached** names its setting, and Needs you holds Ask temper's proposal of the raise.
- **Workers' figures:** an engine sends a worker its figures at its opening, and a worker with the file of 4.3 runs a goal's work.
- **Workers as settings:** in a world, a worker added as a setting links with no restart of the engine; a name with no setting, or a wrong secret, is refused `not-admitted`; new places apply at the worker's next opening; removing a linked worker is refused, naming it; `worker list` shows a lost worker down.
- **Upgrades:** a member that moved is refused, naming it; once the TOML file lands, `temper.json` is refused by name.

## 10. Order

For the execution file (`config.execution.md`), each milestone its own plan, started as soon as what it needs has landed, and in parallel where nothing ties them. The file's format changes last, so that it changes once.

1. **At once, in parallel:** one stock and derived figures (5, 7); workers as settings (6.7); the workers' and agents' figures from the engine (6.3); places at set-up, and the rest (6.4, 6.5).
2. **Then:** limits and the memory budget as settings (5.3, 6.1, 6.6), once the stock has landed; LLM providers in the store (6.2), once the workers' figures have.
3. **Last:** the TOML file, its tables, help, reference and `init` (section 4).

## 11. Open questions

- **A raise taken at once by every child** (6.1). Each limit's milestone checks the children that hold a copy. Default: a child that can't take a raise at once makes its limit "at the next start".
- **The memory budget's stock** (5.3): 1 GiB as today, or derived from the machine's memory. Default: 1 GiB, a ceiling of work held, which a deployment at rest doesn't approach.
- **An override's reason** (6.1): required or optional. Default: required, so a later reader knows why a figure departs.
- **The webhooks' listener** (4.4): the next port, or a path on the clients' listener. Default: the next port.
