# The protocol layer

Provisional, 2026-10-03, revised 2026-11-02. temper's protocol layer:
how temper composes jig's protocol layer with components of its own (its
forge's, its checkout's, its client's) into the processes of one binary.
jig owns the layer's shape (jig's `protocol/README.md`) and the services'
(jig's `protocol/service.md`); this document says what temper adds and
the crates it is made of. The mechanics are those of skein's
`programming-model.md` (sections 2, 4 and 9); the entities it carries are
those of `../domain/`. Each boundary of temper's own is designed in a
document of its own (section 2). What is open across them is listed in
section 14. The cluster, temper as several processes of one machine for
its tests, is section 15.

## 1. In one page

- **Translation, and nothing else.** The protocol layer turns bytes into
  the domains' entities and back, and decides nothing about the domain
  (programming-model.md, section 4). The domain decides deadlines,
  retries and what a failure means; the protocol layer runs the call, its
  timer and its one attempt.
- **jig's protocol layer, composed.** jig ships the components for its
  own vocabularies: the store, the workers' channel, LLM accounts,
  clients, configuration (jig's `protocol/README.md`, section 5).
  temper's services own them as children beside smith's, skein's and
  temper's own components, and route between them, io and the roots, as
  jig's reference service does (jig's `protocol/service.md`, section 2).
- **What temper adds:** the forge's protocol (skein's forge component,
  with its webhooks' listener, handed temper's values and routed to the
  root and the forge connector); the checkout's io
  (the git component); its parts' codecs on jig's channel; its families
  in jig's store; its git identities; its host tools' translation; its
  client (its wire, its client root and view on `jig-client`, its
  terminal); the clients' listener, on which it mounts jig's clients'
  component and serves its forge routes; the setup host, skein's, in the
  engine (forge.md, 2.6); and the forge's setup's commands outside the
  services (setup.md). Each is a crate of its own (section 5).
- **Foreign protocols are stacks of skein's machines.** Forgejo's REST
  API and GitHub's are skein's forge kit, each a dialect of it, over
  skein's HTTP client and JSON (skein's `forge.md`); git's arguments in and text out are temper's decoders
  over io's processes. temper's records on jig's channel, in jig's store
  and on the client's wire are generated from schemas by skein's
  generator.
- **Standalone first.** `temper serve` runs the engine and one worker in
  one process, on one loop, joined by jig's in-process address; the
  worker's agents are child processes of temper's own binary (`temper
  internal agent`); a person's client reaches the engine on loopback;
  the forge is polled, and hinted by its webhooks on loopback when they
  are configured (section 8; forge.md, section 5). Hosts apart,
  webhooks from another host, the forge as a sign-in provider and the
  forge's client documents come after (section 14). The browser is a
  person's client on loopback too (`../client/browser.md`).
- **The cluster, for tests.** The same `temper serve`, on a
  document of another shape, is an engine alone or a worker alone. One
  engine alone and its workers alone, each a process of its own on one
  machine, joined over loopback in plaintext, with the fakes as
  processes beside them, are the cluster that temper's process tier
  runs from one description (section 15). It is no
  deployment's shape: a box keeps its one worker inside `temper serve`.
- **One binary:** `serve`, `internal agent`, `open`, `program-login`,
  the forge's setup's `forge prepare`, `forge run` and `box`, and the
  agents' `ctl`, which `../client/tools.md` designs (section 4).
- **No retries below the domain.** A call goes out once and ends once,
  with the failure the domain's vocabulary defines (section 5).
- **The forge's load follows its changes.** The calls a pass makes grow
  with what changed on the forge, never with its history, its labels or
  the items temper holds, and the worlds count them by HTTP request
  (forge.md, section 6).
- **Secrets never reach a domain.** A domain names a credential by number
  and generation; values live in the protocol layer's tables and in jig's
  secrets' store: LLM accounts' as jig keeps them, and each forge's
  records under temper's tag, from which its access and its git identity
  are made (section 10; credentials.md).
- **Tested at every tier:** codecs and parsers against golden bytes and
  transcripts of real peers; the forge component and the git component in
  worlds of their own; temper's client joined to the engine's root; the
  one-process world on skein's simulator; the real binary with an owned
  Forgejo, its runner and real git; a live tier with a real model
  (section 13; `../domain/testing.md`).

## 2. Reading order

1. **jig's `protocol/README.md` and `protocol/service.md`:** the layer
   temper composes, the service parts jig ships for it, and the routing
   temper copies.
2. **README.md:** this overview.
3. **configuration.md:** what `temper serve` and its other commands are
   told at start.
4. **credentials.md:** the secrets temper's processes hold: LLM accounts
   as jig's, the forges' records in jig's secrets' store, the forges'
   access, git identities, and the values clients' requests carry.
5. **channel.md:** temper's parts on jig's channel, and a git identity's
   value.
6. **store.md:** temper's families in jig's store.
7. **llm.md:** where LLM calls are made, host tools, the charter and a
   run's result.
8. **git.md:** the checkout's io: the git component.
9. **forge.md:** the forge's protocol layer on skein's forge kit: what
   temper hands the forge component, the setup host, its marks, its
   webhooks' address, and what it relies on of Forgejo and GitHub.
10. **setup.md:** the forge's setup: connected, managed and owned, the
    setup host in the engine, and `temper box`.
11. **sign-in.md:** people's sign-in with Google on a deployment's
    public address: its members, routes, endings and binding.
12. **`../client/README.md`:** temper's client: its client root on `jig-client`,
    its pages, the forge's client family, its view, the terminal, and the
    clients' listener of temper's server; then **`../client/tools.md`**, `temper
    ctl`, the agents' tools.
13. **`../domain/testing.md`:** the tiers below the domain.
14. **`../client/browser.md`:** the browser, the second shell of
    temper's client.
15. **documentation.md:** the user documentation compiled into the
    binary, `temper help`, and agents' `read_docs`.
16. **tooling.md:** the contract with a project's own `prepare` and
    `key` commands, run as jobs on a worker for a landing by turn.

## 3. Names

jig's names keep their meaning here (jig's `protocol/README.md`, section
3; jig's `domain/README.md`, section 4): the hub, the core, the fleet,
the worker's link, credential grant, an application's credential,
payload version, profile, the clients' component, the default HTTP
mapping, the reference service (`jig-test-service`), the reference
binary (`jig-test`).

| Name | What it is |
|---|---|
| standalone | one process on one person's machine: `temper serve` with the engine and one worker joined in process (section 8) |
| engine alone | `temper serve` on a document with an engine and no worker, its workers' listener on loopback (15.1) |
| worker alone | `temper serve` on a document with one worker and no engine, dialling an engine alone on loopback (15.1) |
| the cluster | one engine alone and its workers alone on one machine, with the fakes as processes, as temper's process tier runs it (section 15) |
| the cluster's description | `crates/temper/tests/fixtures/cluster/`: the cluster's processes and their documents, read by the process tier (15.5) |
| the engine's service, the worker's service | each `iterate` with its root and protocol layer, in `temper-service` (sections 6 and 7) |
| the routing | the protocol layer's code between io, components and the root (jig's `protocol/service.md`, section 2) |
| the forge component | skein's `skein-forge-connection` (forge.md; skein's `forge-connection.md`) |
| the git component | `temper-worker-protocol-git` (git.md) |
| temper's parts | the workspace items, the saved-work branch, the work, the preparation (channel.md) |
| payload version, store version, client version | temper's three own versions: `PAYLOAD_VERSION`, `STORE_VERSION`, `CLIENT_VERSION` (section 9) |
| temper's families | the forge connector's records in jig's main store (store.md) |
| a forge's records | its credentials, ids and accepted installations: rows of jig's secrets' store under temper's tag (credentials.md, 2.1) |
| git identity | one of temper's application credentials, one per forge: Forgejo's git token, or a GitHub App's installations' tokens, with a login and an author (credentials.md, section 4) |
| the identities' table | the protocol layer's table of git identities' values, on the engine and on each worker |
| a forge's access | what the forge component authenticates with: Forgejo's API token, or a GitHub App's id and key and its installations' tokens; the component's only (credentials.md, section 5) |
| the setup host | skein's in-process host for the setup's plans, a child of the engine's protocol layer (forge.md, 2.6) |
| the forge's client family | temper's client side of the forge connector: its words, and the adoption form (`../client/README.md`, section 4) |
| the terminal shell | `temper-client-shell-terminal` (`../client/README.md`, section 6) |
| the browser shell | `temper-client-shell-web` (`../client/browser.md`, 2.1) |
| the bundle | the browser shell's module, glue and stylesheets, skein's bundler's output, which the binary holds and the clients' listener serves (`../client/browser.md`, 2.2 and 2.6) |
| the tools | the agents' interface to temper, `temper ctl`: `temper-client-tools` and `temper-client-shell-cli` (`../client/tools.md`, section 4) |
| the clients' listener | `temper-service`'s listener for clients, on which jig's clients' component is mounted with jig's default HTTP mapping, under `/jig` (`../client/README.md`, section 8) |
| the fake git program | git as the worker runs it, faked: its core `crates/temper-fake-git`, `temper fake git`, and its hosted half `crates/temper-fake-git-hosted` (`../domain/testing.md`, 4.3) |
| the throwaway Forgejo | a Forgejo started by a test script on loopback, with SQLite in a scratch directory, removed after (`../domain/testing.md`, 2.6) |
| locate | the forge connector's read of a repository's id by owner and name (skein's `forge.md`, 5.2) |
| the webhooks' listener | the forge component's own listener for Forgejo's signed deliveries, plaintext on loopback, one path per forge (skein's `forge-connection.md`, section 6; forge.md, section 5) |

The forge's setup's names are setup.md's: connected, managed and owned
(section 2), the forge's directory (section 3), reconcile, a plan of the
setup host's (section 5),
the pin (6.1), the preparation (6.3), the box and its steps (section 7).

## 4. The processes and the binary

The box's binary, `temper` (`crates/temper`), ordinary Rust: a shell,
copied from jig's reference binary, the whole product with every
command; a deployment runs a binary per process (4.1). Its commands, sorted by who runs them
(`../draft/agent-docs.md`, 7.1), `temper ctl`'s as `../client/tools.md`
designs them:

```
documentation  temper help [NAME | --search WORDS...]
deployment     temper init [DIRECTORIES] [--box]
               temper serve [DIRECTORIES]
               temper box [DIRECTORIES] [--echo none|steps] [--fake-backends [--bundle DIR]] [--no-tui]
               temper versions
               temper upgrade --check DIRECTORY | --back
               temper program-login [DIRECTORIES] NAME
               temper forge fetch|prepare|sign-in|remove [DIRECTORIES] FORGE
               temper forge run [DIRECTORIES] FORGE forgejo|runner
clients        temper [ADDRESS]
               temper ctl DIRECTORY GROUP VERB [ARGUMENT...] [--json]
               temper ctl DIRECTORY --call FILE [--json]
               temper ctl help [GROUP [VERB]]
               temper open DIRECTORY LINK_OR_ADDRESS
spawned        temper internal agent AGENT.json
               temper internal serve [DIRECTORIES] --box
               temper internal forge-child --parent PID --signal kill|terminate -- PROGRAM ARGS...
               temper internal tui --box ADDRESS
test doubles   temper fake llm|forge|oauth|git ...
```

- **Every command is a row of the binary's table,** as ctl's tools are
  (`../client/tools.md`, 5.1): its words, its usage, its line, its
  arguments with a line each, and its notes. The parser, `temper
  COMMAND --help`, `temper --help` (the rows by who runs them, a line
  each) and the reference page `commands` derive from it.
- **What temper spawns itself** is under `temper internal`, absent from
  `temper --help`'s first groups and from every page but `commands`; a
  person or an agent never runs it.
- **The deployment's client, `temper-client`** (`crates/temper-client`;
  `../draft/deployment-binaries.md`, section 4 and section 9's first item), a binary of its
  own for a person's machine or an automation's. Its table holds three of
  `temper`'s rows, the same rows run by the same code (section 5):

  ```
  documentation  temper-client help [NAME | --search WORDS...]
  clients        temper-client [ADDRESS]
                 temper-client ctl DIRECTORY GROUP VERB [ARGUMENT...] [--json]
                 temper-client ctl DIRECTORY --call FILE [--json]
                 temper-client ctl help [GROUP [VERB]]
  ```

  - Its parser, `temper-client --help`, `temper-client COMMAND --help` and
    its refusals derive from its table as `temper`'s do from its own,
    each line naming `temper-client` where `temper`'s name `temper`. The
    rows' own words, their lines, arguments and notes, are the box's and
    name `temper`; so do `ctl`'s own help and its lines (`temper ctl: `),
    which no binary changes.
  - **`temper-client --version`** prints one line, `temper-client VERSION
    (COMMIT)`, as `../release.md`, section 3, gives every binary's: VERSION
    and COMMIT `temper-version`'s (4.1). It is a flag, as `--help` is, and
    no row; `version` is no command of its.
  - A first word that is no row of its table, no address and no flag is
    refused as `temper`'s are, `temper-client: no command WORD;
    temper-client --help lists them`, exit 1: `box`, `serve`, `open` and
    `internal` among them.
  - **It links** the client's crates but the web shell and the bundle,
    jig's client crates, skein's, and `temper-shell` (section 5), which
    every binary of temper's starts from; none of the engine's, the
    worker's, smith's agent, the box or the forge's setup. Built alone,
    as the release job builds it (`cargo build -p temper-client --bin
    temper-client`), `temper-protocol-records` is built without its
    `forge` feature.
  - **Its file** is the Cargo target `temper-client`. The name a
    deployment's people run it by is the user's to choose
    (`../draft/deployment-binaries.status.md`, Open), and the release job
    names the published file by it.
  - **Its story,** in the process tier: `temper-client ctl` signs in to an
    engine with the link it printed and reads `me`, as `temper ctl` does
    (`../draft/deployment-binaries.md`, section 10).

`DIRECTORIES` is `[--config DIR] [--data DIR] [--cache DIR]`, given
anywhere among the command's arguments, each with its variable and its
default; the document is `temper.json` in the configuration directory
(configuration.md, section 12).

- **`temper serve`:** reads the configuration (configuration.md,
  section 2) and runs the services its document's shape names, in one
  process on skein's `Host` and `drive` (jig's `protocol/service.md`,
  sections 4 and 10): standalone, the engine's and the worker's, the
  store in the deployment's directory; an engine alone, the engine's
  with its store and no worker; a worker alone, the worker's, with no
  store (15.1).
  - Its stdout carries only sign-in links, one per line: each the
    secret jig's clients' component gives, in jig's default mapping's
    link route under temper's prefix (jig's `protocol/client.md`,
    section 8; `../client/README.md`, section 7); its stderr carries startup
    notes, the lines of 15.3 (each TCP listener once bound, and a worker
    alone's link at each opening) and the reason of a failure. A worker
    alone prints no sign-in link.
  - `--box`, given by `temper box` alone, as `temper internal serve
    --box`, adopts the box's pipe, the two fds after the listeners the
    box gives it (setup.md, 7.1). Given listeners, `LISTEN_FDS` of them
    from fd 3, its clients' and webhooks' listeners take them by their
    addresses (skein's `shell.md`, 6.6).
  - A first termination signal settles the engine's forge writes, then
    drains it (`../domain/root.md`, 9.2; jig's `protocol/service.md`,
    section 5), and a worker alone's runs its worker's ending (jig's
    `protocol/service.md`, section 6); a second turns either into an
    abort, which keeps the reason of a failure that started the drain
    (a lost webhooks listener's, forge.md, 2.3). A write the settling
    left without its outcome is printed as the drained engine ends, one
    line each, and the exit is still 0:
    `temper serve: stopped before task {task}'s forge write {entry} settled; the next start settles it`.
  - A start refused because its patches would reshape a write in flight
    (jig's `protocol/store.md`, 10.3, `InFlight`) prints jig's words,
    then a line for each write its keys name, from temper's `Flight`
    key, at most 64, then how many more, and exits 1:
    `temper serve: task {task}'s forge write {entry} is still in flight; start the build before this one and stop it to settle it`;
    `temper serve: and {n} more forge writes in flight`.
  - A start refused at a step of its restart (jig's `domain/engine.md`,
    section 6; `../domain/root.md`, 9.1), which only a store that does
    not decode or a live set past the limits can be, prints the refusal
    in the restart's own words, `temper serve: ` then the step and what
    it refused (`restore tasks: live tasks past limits.tasks`), and
    exits 1. The catch-all `unknown reference` is left for a refusal nothing
    names. The refused process ends whole: it closes every listener it
    holds, the box's given ones too, whether or not a component took
    one, and its ended root settles each worker's pending release at
    once, so neither a listener nor a worker keeps it open.
  - Under the box, a refused start is a long step's exit like any other
    (setup.md, 7.2): restarted after its backoff, its refusal printed
    at each exit, the box giving up, exit 1, past `box.restarts` within
    `box.window_ms`. No refusal is told apart as lasting. The box reads
    no step's reasons, and a retry is a restart, which makes nothing
    twice; a cause repaired during the backoff (a setting, the disk)
    lets the next attempt start.
  - A link a start cannot make (a writer's bug, jig's
    `domain/engine.md`, section 6) is a panic: serve's fail-stop report
    names the record and the link, with its backtrace (skein's
    `shell.md`, 6), and under the box it is a long step's exit like any
    other.
- **`temper init`:** writes a deployment that starts, the default
  document among it, into the three directories, refusing when the
  configuration directory holds a document (setup.md, 8.2).
- **`temper internal agent AGENT.json`:** smith's agent shell,
  `smith_agent_shell::run_decoded`, on smith's agent configuration as
  temper reads it, a `"fetched"` program naming its launcher
  (configuration.md, section 5; setup.md, 6.6). The worker's protocol layer spawns it,
  one process per run; a person never does.
- **smith's agent modes,** `temper --mcp-relay 127.0.0.1:PORT` and
  `temper --program-child PARENT PROGRAM ARGUMENTS...`: temper's binary
  is its agents' own executable, so the agent shell starts each program
  process through it, its status check and each print run, and the
  program starts its MCP relay through it (smith's
  `protocol/programs.md`, 5.1 and 5.3). `main` gives either flag, as
  the first argument, to smith's agent shell,
  `smith_agent_shell::relay::main` and
  `smith_agent_shell::program_child::main`, before it reads the
  binary's table, as `smith-agent` does. Neither is a row of the table,
  in `temper --help` or on a page; a person never runs them.
- **`temper [ADDRESS]`:** a person's full-screen terminal client, the
  TUI (`../client/README.md`, 6.11). Bare `temper`, with stdin and
  stdout both terminals, runs it at the address it last signed in to,
  else at `http://127.0.0.1:8367`; with either not a terminal, it prints
  `temper: no command; temper --help lists them` and exits 1, as before
  the TUI. ADDRESS is the engine's clients' listener on loopback; the
  sign-in link is pasted on the TUI's screen, never given. Its
  directory is its own, under the person's state directory
  (`../client/README.md`, 6.12). It reads no configuration file.
- **`temper open DIRECTORY LINK_OR_ADDRESS`:** a person's terminal client
  (`../client/README.md`, 6.1). It reads no configuration file.
- **`temper ctl DIRECTORY ...`:** the agents' tools, one call per
  process, over the same native transport as `temper open`
  (`../client/tools.md`, section 8). It reads no configuration file.
- **`temper versions`:** what this binary speaks and keeps, one line
  each: every owner's patch level (none before the first release), its
  families' and the store's versions, and its wires, as jig's store
  keeps them (jig's `protocol/store.md`, 5.3). It reads no
  configuration and no store.
- **`temper upgrade --check DIRECTORY`:** a rehearsal (jig's
  `protocol/store.md`, 10.5) over DIRECTORY, a stopped deployment's
  data directory or a copy of one such as last night's backup, which it
  never writes: it works in a scratch copy under the cache directory,
  with the configuration `temper serve` would read; the open patches
  and checks as a start would, restores, and stops before its switch
  step. It prints a line per patch with its counts and time, the time
  of the whole, and the refusal a start would meet, in a start's words,
  or that the patches and the restores pass, saying that what follows
  the switch is not rehearsed; then whether the workers may stay: yes
  when the copy's wires equal this binary's and no pending patch is
  marked fresh, no saying which differs, unknown when the copy keeps
  none (`../draft/upgrades.md`, sections 3.1 and 9). Exit 0 when the
  patches and restores pass, 1 when a start would be refused there.
- **`temper upgrade --back`:** makes the previous generation active
  again (jig's `protocol/store.md`, 10.5), for a binary that patched and
  failed before its first commit of its own: the deployment stopped,
  and the old binary started after it. Refused, changing nothing, once
  the patched generation has committed, with no previous generation, or
  while a process holds the deployment, each in words that say so; past
  the first commit, going back is restoring a backup
  (`../draft/upgrades.md`, 3.2).
- **A start that patches** prints a line per patch as it ends:
  `temper serve: patched {owner} {patch}: {records} records in {commits} commits`.
- **`temper program-login NAME`:** a program's login for
  the program source NAME, on this host: it checks the document and the
  agent's configuration, makes the source's configuration directory, and
  replaces itself by the program's own sign-in (credentials.md, section
  3). The engine need not run.
- **The forge's setup** (setup.md), FORGE a forge's number, as its
  record has it (forge.md, 2.1) and the Forges page shows it; reconcile
  is the engine setup host's plan (forge.md, 2.6). The box knows kinds,
  not forges: `fetch`, `prepare` and `run` read the forge's kind and
  setup entry from its group's file, `<data>/box/groups/FORGE`, which
  `temper serve` writes as it asks for the group (setup.md, 7.1), a
  forge with none taking the machine's `owned` part (setup.md, 7.4), and
  each reads and checks the whole document as `temper serve` does
  (configuration.md, section 8):
  - `temper forge prepare`: an owned forge's preparation, before its
    Forgejo runs; it ends when done (setup.md, 6.3);
  - `temper forge run`: checks the binary and the preparation, takes
    the forge's lock, and replaces itself by `forgejo web` or
    `forgejo-runner daemon`, whose process a host supervises (setup.md,
    6.4);
  - `temper forge remove`: an owned or fake forge's directory under
    data removed, `forge-N` or the playground's, by the kind its group's
    file names: the box's remover step, run once a removed forge's group
    has stopped (setup.md, 7.1; forge.md, 2.3 and 2.8). It refuses a
    path outside data or one within the cache, which it keeps,
    Forgejo's binaries with it, and a directory already gone is no
    failure; it reads no document and touches no store, so a forge is
    removed from the client, never by this command alone;
  - `temper box`: its steps as child processes, `temper serve` and the
    groups it has started for forges, each by its kind (setup.md, 7.1),
    in order, restarted when they exit and stopped in reverse order; its
    stdout is `temper serve`'s sign-in links, each line whole; its
    stderr, a terminal's, the box's own lines alone, each step's lines
    in its log under the data directory, and on stderr too, prefixed by
    the step's name, with `--echo steps` (setup.md, 7.2).
- **Exit status:** 0 after a drained shutdown, a worker alone's ending
  or a finished command; 1 on
  any failure (a refused start, a usage error, a failed commit, an
  abort, a failed step of the setup, the box giving up), the reason on
  stderr. smith's agent shell exits 0 or 1 the same way; `temper ctl`'s
  statuses are `../client/tools.md`, 6.3; `temper forge run`'s is Forgejo's or
  the runner's own, and `temper program-login`'s the program's sign-in's
  once it has replaced itself.

Standalone, one person's machine:

```
temper serve                           one process, one loop: temper-service's system module
├── the engine's service               section 6
│   ├── listens: the in-process name "workers"; the clients' listener and, when configured, the webhooks' listener, TCP on loopback
│   └── connects: each kept forge's API (HTTP; TLS unless the forge is on loopback)
├── the worker's service               section 7
│   └── dials: the in-process name "workers"
├── temper internal agent AGENT.json   a child process per run, in a process group of its own
└── git ...                              a child process per git invocation, in a process group of its own (git.md, section 7)
temper open DIRECTORY LINK_OR_ADDRESS  a process of its own, reaching the engine on loopback
temper ctl DIRECTORY GROUP VERB ...    a process per call, reaching the engine on loopback
temper program-login NAME              run by hand on a host, replaced by the program's sign-in
temper versions                        prints this binary's levels, versions and wires
temper upgrade --check DIRECTORY       rehearses an upgrade over a copy; switches nothing
temper upgrade --back                  makes the previous generation active again, before the patched one's first commit
```

A box runs `temper serve` as one of its steps, after the forge's setup's
commands, each a child of its own (setup.md, 7.1). The cluster's
processes are 15.2's.

### 4.1 The deployment's binaries

A deployment's processes each run a binary of their own
(`../draft/deployment-binaries.md`, section 4): the engine's,
`temper-engine` (4.2), and the worker's, the agent's and the client's,
each written here by its own milestone
(`../draft/deployment-binaries.execution.md`). The box keeps `temper`,
which links every one of them and keeps every command above. What holds
of each:

- **Its own table,** rows as above, from which its parser, its
  `--help`, its `COMMAND --help` and its usage lines derive, rendered
  under its own name by `temper-shell`'s `commands` (section 5). A row
  two binaries share, the same words run by the same code, is one
  constant both tables hold. `help`, the pages, is every binary's.
- **`--version`** prints `NAME VERSION (COMMIT)` before the table is
  read (`../release.md`, section 3).
- **A binary refuses a document that isn't its own,** before any other
  check, by the document's nearest shape (15.1), naming the binary that
  runs it; the standalone shape is the box's `temper serve`'s.
- **What it links is held by a test.** `tests/boundary/tests/binaries.rs`
  reads, for each binary's package, its closure as `cargo tree -p
  PACKAGE -e normal --locked --offline` gives it, and fails naming each
  crate of the binary's excluded set it finds there, and each named
  exception it no longer finds, so that the list only shrinks. A row
  per binary: its package, the crates it excludes, by name or by a
  prefix ending in `-`, and its exceptions, each with its reason. An
  exception is a cut that costs more than it gives.
- **Code two binaries share is a crate each links,** never one binary's
  package another links: `temper-shell` (the commands' table and its
  renderings, the directories, `help`, the engine's link file) for every
  binary, `temper-engine-shell` for the engine's commands, which the box
  links too.
- **A binary's features are its package's.** `temper-service`'s worker
  half is its `worker` feature (section 5), which the box's binary and
  the worlds turn on and `temper-engine` leaves off. Cargo unifies the
  features of what one invocation builds, so the gate, which builds the
  workspace whole, builds the service with it, and only a binary's own
  build (`cargo build -p PACKAGE --bin NAME`, the release job's) and its
  package's own tests build it without; a break there shows at that
  build (`../draft/deployment-binaries.md`, section 8).

### 4.2 `temper-engine`

The engine's host's binary, `crates/temper-engine`, ordinary Rust: the
engine and its web client, with the owned forge's commands:

```
documentation  temper-engine help [NAME | --search WORDS...]
deployment     temper-engine serve [DIRECTORIES]
               temper-engine versions
               temper-engine upgrade --check DIRECTORY | --back
               temper-engine forge fetch|prepare|sign-in|source|remove [DIRECTORIES] FORGE
               temper-engine forge run [DIRECTORIES] FORGE forgejo|runner
spawned        temper-engine internal forge-child --parent PID --signal kill|terminate -- PROGRAM ARGS...
               temper-engine --version
```

- **Each command is the box's of the same words,** run by the same
  code, `temper-engine-shell`'s, with the same exits, but for what
  follows. Its `help` and its `forge` and `internal forge-child` rows
  are the constants the box's table holds too; its `serve` row is its
  own, saying it serves an engine alone's document.
- **Its lines name it:** serve's and the forge's lines, refusals and
  hints say `temper-engine serve: …`, `temper-engine forge VERB: …` and
  `run temper-engine forge prepare …` where the box's say `temper`.
- **An engine alone's document only.** `serve` and each `forge` command
  that reads the document (all but `remove`) read its nearest shape
  first (15.1) and, for any other, print one line and exit 1, before any
  other check, `COMMAND` the command's words:
  - a worker alone's:
    `temper-engine COMMAND: temper.json holds a worker alone; temper-worker serve runs it`;
  - standalone's:
    `temper-engine COMMAND: temper.json holds an engine and its worker in one process; the box's temper serve runs it`.

  A document whose shape can't be read, not a JSON object, goes on to
  the checks `temper serve` makes, and is refused in their words.
- **`versions`** prints, one line each, in this order, and exits 0,
  reading no configuration and no store:
  - `level OWNER LEVEL` for `skein`, `smith`, `jig` and `temper`, the
    binary's patch level of each (jig's `protocol/store.md`, 10.5,
    `levels`), `none` before the owner's first patch;
  - `store VERSION`, jig's store's component version, then `family
    NAME VERSION` for jig's families, `core`, `tasks`, `people`,
    `notes`, `accounts` and `settings`, and `temper`'s, `STORE_VERSION`
    (section 9; jig's `format::current`);
  - `wire NAME VERSION` for each wire the engine speaks to its workers,
    in the order `temper-service`'s `store::WIRES` gives them.
- **`upgrade`** is section 4's `temper upgrade`, on this binary.
- **No command is an unknown command:** `temper-engine: no command WORDS;
  temper-engine --help lists them`, the words given as far as a row
  begins with them, and exit 1, as the box's binary says it.
- **The bundle is in it,** as in the box's (`../client/browser.md`,
  2.6): of a deployment's binaries, only the engine's serves the web
  client.
- **What it links** (4.1): its row in `binaries.rs` excludes the
  worker's domain, `temper-worker-`, `jig-protocol-worker-agents`,
  `jig-protocol-worker-link` and smith's host half, `smith-host-`, and
  smith's agent: `smith-agent-`, `smith-local-`, `smith-inline-agent`,
  `smith-domain`, `smith-domain-`, `smith-protocol-channel`,
  `smith-protocol-events`, `smith-protocol-llm`,
  `smith-protocol-machine`, `smith-protocol-program` and
  `smith-events`. Its exceptions:
  - `smith-domain`, `smith-domain-run`, `smith-domain-session` and
    `smith-domain-tools`: smith's agent domain, sans-io, whose
    vocabulary jig's charter (`jig-protocol-charter`) and the engine's
    translation (`temper-engine-smith`) speak, the charter's run,
    `Start`, `Event` and `Transcript`, with no io, process or LLM of its
    own. Cutting them would move that vocabulary into a smith crate of
    its own, a change of smith's that gives the engine no io or process
    less.

  smith's codecs and shared translation, `smith-charter`,
  `smith-channel`, `smith-transcript` and `smith-protocol-translate`,
  and the channel's shared wires, `jig-protocol-worker-wire` and
  `temper-protocol-worker-wire`, are the engine's as much as the
  worker's, and are not excluded.
- **Its package** holds its table and its `main` alone; the commands'
  code is `temper-engine-shell`'s and `temper-shell`'s, `--version`'s
  `temper-version`'s (section 5). Its step tests, one test binary, run
  the built binary.

## 5. Components and crates

**jig's,** composed as they are (jig's `protocol/README.md`, section 5):

| Crate | In | What temper uses it for |
|---|---|---|
| `jig-protocol-config` | engine, worker | jig's parts of the configuration (configuration.md, section 2) |
| `jig-protocol-store` | engine | the store, temper's families beside jig's in the same commits (store.md) |
| `jig-protocol-worker-wire` | engine, worker | the channel's records; temper's parts ride in them as bytes (channel.md) |
| `jig-protocol-engine-workers` | engine | the worker's link, the engine's side: opening, the payload version, grants' values filled in |
| `jig-protocol-worker-link` | worker | the worker's link, the worker's side: dialling, grants' values taken out |
| `jig-protocol-worker-agents` | worker | the program sources and the worker route, between smith's host half and the link (jig's `protocol/service.md`, 11.2) |
| `jig-protocol-accounts` | engine | LLM accounts on skein's accounts driver: signed-in records and keys from jig's secrets' store, a loopback listener for each sign-in's redirect, the access tokens and keys lent (credentials.md, section 3) |
| `jig-client-protocol-wire` | engine, client | jig's records for clients; the forge's client family's records ride in them as bytes (`../client/README.md`) |
| `jig-protocol-engine-clients` | engine | the clients' component: jig's messages with their credentials, the printed link's secret, sign-ins, credentials (`../client/README.md`, section 8) |
| `jig-protocol-engine-clients-http` | engine | jig's default HTTP mapping, its engine end, with its example guards, mounted on temper's clients' listener under `/jig`, and the server part that listener stands on (jig's `protocol/service.md`, 11.4) |
| `jig-client` | client | the client temper's client root holds: the link, watches by handle, the request table, kept work, families; jig's helpers (`../client/README.md`, section 3) |
| `jig-client-protocol-link` | client | `jig-client`'s half toward its transport: jig's messages and the small store's records as bytes (jig's `protocol/client.md`, 4.3) |
| `jig-client-protocol-http` | client | jig's default HTTP mapping, its client end, under `jig-client-shell-native` and `jig-client-shell-web` |
| `jig-client-shell-web` | client | the browser shell's adapter over skein's fetch, and the printed link's sign-in part (`../client/browser.md`, 2.1) |
| `jig-protocol-transcript` | client | turns' bytes, decoded below `jig-client` |
| `jig-client-shell-native` | client | `temper open`'s and `temper ctl`'s transport: HTTP on io, the credential `jig-client` holds sent as a bearer, the small store's file in its directory |

temper does not use `jig-protocol-charter` or `jig-protocol-tools`: its
root takes its own `Tool` vocabulary, translated by `temper-engine-smith`
(llm.md, sections 4 and 5). Nor does it use jig's reference UI
(`jig-client-view`, `jig-client-test-shell-web`, nor the look of
`jig-client-shell-web`): temper's pages are its own,
on skein's UI kit (`../client/README.md`, section 5).

**smith's:** `smith-host-domain`, the worker root's agent child;
`smith-host-protocol`, smith's host half, one channel per agent process
over its pipes (smith's `protocol/hosts.md`, sections 2 and 3);
`smith-protocol-channel`, `smith-channel`, `smith-charter` and
`smith-transcript`, smith's codecs, which the routing calls to decode and
encode smith's payloads; `smith-agent-shell`, the `agent` command; and
`smith-local-service`'s profile, from which `temper serve` reads the
agent's host limits at start, as jig's reference binary does.

**skein's:** `skein-lib` and `skein-io` everywhere, lib's HMAC-SHA256
checking the deliveries' signatures (skein's `lib.md`, 6.2), io's
processes under the box; `skein-shell`, the binary's startup kit
(`read_file`, `read_private_file`, seeds, `Host`, `drive`);
`skein-json`, configuration and the forges' documents; `skein-http`, the
forge component's client and its webhooks' listener's server, the setup
host's client, and the fake forge's server; the forge kit,
`skein-forge` (Forgejo's and GitHub's documents and call plans), `skein-forge-client`,
`skein-forge-connection` (the forge component) and `skein-forge-setup`
(the setup host and its plans), with the fake forge and the throwaway Forgejo in its worlds
(skein's `forge.md`, section 2);
`skein-tls`, a forge's TLS; `skein-codec` and its generator, every codec
crate; `skein-kv`, under jig's store and temper's key encoders;
`skein-channel`, under jig's channel; `skein-oauth-accounts`, under
jig's accounts; `skein-ui`, under temper's view, and
`skein-ui-text`, the terminal's text shell. In the worlds: skein's
simulator, fake machine, fake LLM peer, fake issuer and fake person.

**temper's own,** each step code under the subset but the binaries,
`temper-shell`, the terminal's host, `temper ctl`'s host,
`temper-forge-setup-shell`, the box's host and the fake git program's
core:

| Crate | Kind | Holds | Depends on |
|---|---|---|---|
| `crates/temper-service` | services | the engine's and the worker's services and their one-process composition, in an engine, a worker and a system module, as jig's reference service is: each `iterate`, the routing, the clients' listener on jig's server part, with jig's clients' component and its default HTTP mapping mounted, temper's forge routes, and the browser's page and bundle served (`../client/browser.md`, 2.2), startup from configuration's bytes, start and end; temper's profile `small`. The worker's half is its `worker` feature (4.1), on by default and in the box's binary and the worlds: with it off, the worker's types are uninhabited, its modules and crates absent, and a start whose document's nearest shape (15.1) isn't an engine alone is refused; the engine's half names no worker crate, its limits of the worker's checkout and git component its own mirrors, which the worker's half converts | the roots, `temper-engine-smith`, `temper-protocol-brief`, jig's components, smith's codecs and `smith-protocol-translate`, temper's codec crates and components, `skein-forge-connection`, `skein-io`, `skein-json`; with `worker`, the worker's root, checkout and git component, `jig-protocol-worker-link`, `jig-protocol-worker-agents` and smith's host half |
| `crates/temper` | binary | the box's binary: the commands of section 4, copied from jig's reference binary; for each that reads the document, `temper-service`'s startup, with `machine` and the setup's and the box's limits checked by their own crates (configuration.md, section 2); `serve` and the forge's commands are `temper-engine-shell`'s, given the worker's half of a start, its agent, root, program sources and launcher, which this binary holds | `temper-service` with `worker`, `temper-engine-shell`, `temper-shell`, `skein-shell`, `smith-agent-shell`, `smith-local-service`'s profile, `skein-oauth-accounts`, `jig-client-shell-native`, the client crates, `temper-forge-setup-shell`, `temper-box`, `temper-client-bundle` |
| `crates/temper-client` | binary | `temper-client`, the deployment's client (section 4): its table of three rows, its parser and help, and `--version`; the TUI and `ctl` it runs are its shells' (`../client/README.md`, section 2) | `temper-shell`, `temper-version`, `temper-client-shell-tui`, `temper-client-shell-cli`, `skein-shell`, `skein-fail-stop` |
| `crates/temper-forge-setup` | step code | the forge's setup, but for reconcile, which is skein's (`skein-forge-setup`): its types and checks, and the setup mapped onto reconcile's given values, with the kit's check of them; the preparation's machine; `app.ini` and the runner's files; the processes' specs; the configuration's steps, `steps` and `Step`. Names no host | setup.md, section 11 |
| `crates/temper-forge-setup-shell` | hosts | ordinary Rust: the hosts of `temper forge prepare` and `run`: commands, files, the forge's locks and the exec (setup.md, 6.4); HTTP on skein-http over skein-io | setup.md, section 11 |
| `crates/temper-box` | machine, host | the box's machine, step code, and its host, ordinary Rust, on skein-io's processes; nothing of the setup but its steps (setup.md, 7.3) | setup.md, section 11 |
| `crates/temper-fake-git` | program | the fake git program's core, ordinary Rust, which the binary runs as `temper fake git`: its checks, answers, step core, replay and two remotes (`../domain/testing.md`, 4.3) | setup.md, section 11 |
| `crates/temper-fake` | step code | the fake backends' documents and seeds, for `temper fake forge`, `temper fake llm` and the box's fake backends (setup.md, 7.5 and 8.4; configuration.md, 12.1); no io | skein's fake forge and fake LLM, `skein-forge-setup`, `temper-forge-setup`, `temper-protocol-store`, `temper-engine-domain-forge` |
| `crates/temper-init` | step code | the files `temper init` writes, the default document among them (setup.md, 8.2 and 8.3) | `skein-lib`, `skein-json`, `skein-forge-setup` |
| `crates/temper-worker-protocol-git` | component | the git component: each `git::Op` of `temper-worker-domain-checkout` as a fixed sequence of git invocations, their arguments, environment, output and failures; the operations in flight, their processes, pipes and deadlines; `Make` as file operations (git.md) | `skein-lib`, `skein-io`, the checkout's vocabulary |
| `crates/temper-protocol-worker-wire` | codecs | temper's parts on jig's channel, and a git identity's value, from a schema; `PAYLOAD_VERSION` (channel.md) | `skein-lib`, `skein-codec` |
| `crates/temper-protocol-store` | codecs, keys | temper's families: key encoders, codecs from a schema, `STORE_VERSION`, the restart's interval (store.md) | `skein-lib`, `skein-codec`, `skein-kv`, `jig-protocol-store`, the forge's records |
| `crates/temper-protocol-records` | codecs | the forges' records in jig's secrets' store and a pasted bundle, from `schema/secrets-v1.schema`, the family `temper-secrets` (section 9; credentials.md, 2.1) | `skein-lib`, `skein-codec`, the forge kit's vocabularies, `jig-protocol-config` |
| `crates/temper-client-protocol-wire` | codecs | the forge's client family's records: the adoption request and its outcome, the small store's record of a pending one; `FORGE`, the forge connector's number, 1; `Owner`, temper's owners; `CLIENT_VERSION` (`../client/README.md`, 4.6) | `skein-lib`, `skein-codec` |
| `crates/temper-engine-smith` | translation | smith's start from an assignment, host calls decoded into the root's `Tool` vocabulary, answers rendered, results and turns read; called by the engine's routing, keeping no state (llm.md); the brief fitted by `temper-protocol-brief` | the root's and its forge children's vocabularies, jig's and smith's domain vocabularies, `temper-protocol-brief`, `skein-lib`, `skein-json` |
| `crates/temper-client-domain` | client root | `jig-client` held, temper's pages, the forge's client side (`../client/README.md`, section 3) | `../client/README.md`, section 2 |
| `crates/temper-client-view` | view | temper's pages as skein-ui's tree, with the forge's view (`../client/README.md`, section 5) | `../client/README.md`, section 2 |
| `crates/temper-client-shell-terminal` | shell | the terminal: its host ordinary Rust, over skein's text shell and jig's native transport, and how an argument names an engine's address or its link (`../client/README.md`, section 6) | `../client/README.md`, section 2 |
| `crates/temper-client-shell-web` | shell | the browser: a `cdylib` for wasm32, ordinary Rust, over skein's browser parts and jig's browser adapter (`../client/browser.md`, 2.1) | `../client/README.md`, section 2 |
| `crates/temper-client-bundle` | data | the browser's bundle in the binary: its build script builds `temper-client-shell-web` for wasm32 and skein's bundler in a nested cargo, runs the bundler and lists the bundle's files; its library includes them, or nothing under `TEMPER_BUNDLE=none` (`../client/browser.md`, 2.6) | nothing; its script, `std` alone |
| `crates/temper-client-tools` | client root, rendering | the agents' tools: the table, the runner, marks and conditions, the rendering, the tools' root (`../client/tools.md`, section 4) | `../client/tools.md`, section 4 |
| `crates/temper-client-shell-cli` | shell | `temper ctl`: its host ordinary Rust, one call per process over jig's native transport, and its start, which `temper ctl` and `temper-client ctl` run (`../client/tools.md`, section 8) | `../client/tools.md`, section 4 |
| `crates/temper-settings-form` | codecs | the settings file's form and words and `FIELDS`, shared by the client and the engine (`../client/tools.md`, 5.3; documentation.md, section 8) | `skein-lib`, `skein-json`, `skein-codec`, `jig-client-protocol-wire` |
| `crates/temper-docs` | data | the help pages compiled in, with their paging and search, of no side (documentation.md, sections 2 and 3) | `skein-lib` |
| `crates/temper-engine` | binary | the engine's binary, its table and `main` (4.2) | `temper-engine-shell`, `temper-shell`, `temper-version`, `temper-service` without `worker`, `jig-protocol-store` and `temper-protocol-store` (for `versions`), `skein-shell`, `skein-io`, `skein-fail-stop` |
| `crates/temper-engine-shell` | hosts | ordinary Rust: `serve`'s host and its start for any document, the worker's half given by the binary that holds one (`serve::WorkerHalf`), none by `temper-engine`; the forge's commands, beside `temper-forge-setup-shell`'s hosts; the bundle read or embedded; the GitHub, Google and accounts runtimes; the rows both tables hold (4.1); `setup`, the document's check that `serve` and the forge's commands make, the box's limits and the `demo` member included | `temper-service`, `temper-shell`, `temper-forge-setup`, `temper-forge-setup-shell`, `temper-box` (the box's limits), `temper-fake` (the `demo` member), `temper-client-bundle`, `skein-shell` |
| `crates/temper-shell` | shell | ordinary Rust, of no side: the commands' table, `Row` and its renderings under a binary's name (4.1), and the shared `help`, `[ADDRESS]` and `ctl` rows; the directories and their settings; `help` over `temper-docs`; the engine's link file, written by serve and read by the TUI (`../client/README.md`, 6.12) | `skein-shell`, `skein-io` (a kernel error's number, in a file's refusal), `jig-protocol-config`, `temper-docs`, `skein-lib` |
| `crates/temper-version` | data | `VERSION` and `COMMIT` (`../release.md`, section 3): its build script reads git, its library holds both; linked by the binaries' packages alone | nothing; its script, `std` alone |
| `crates/temper-protocol-brief` | translation | a charter's brief fitted to the room its run leaves, cut from its last section with each cut's line (`../domain/agent.md`, 4.3), for the engine's translation and the worker's service alike | `smith-domain-run`, `skein-lib` |

**temper's test crates, worlds and tiers** are `../domain/testing.md`'s
(sections 2, 4 and 7): the fake git program, the worlds below the
domain, the process and live tiers, and the tool that records git's
transcripts (git.md, section 9). The fake forge's protocol layer, the
throwaway Forgejo and the tool that records Forgejo's transcripts are
skein's (`fake-forge.md`, sections 4, 6 and 7).

**What temper's components share** is what every component of jig's
does (jig's `protocol/README.md`, section 1; programming-model.md,
sections 4 and 9): each a child of the protocol layer with its own
limits, deadlines and kernel submission edges; its
connections or invocations its own entities, bound to calls by echoed
tokens; one exchange in flight per connection, with an output cap and a
room check before the next is parsed; a call with no room refused before
anything is set aside for it; each call's deadline the domain's, and
each connection's or invocation's its own, armed from its limits. Two
rules every temper component keeps:

- **No retries below the domain.** A call goes out once and ends once,
  with the failure the domain's vocabulary defines: for the forge,
  unavailable only when the call provably never left, and timed out
  otherwise (skein's `forge.md`, 5.4). A retry below the domain hides its cost from
  the budget, and adds load just when the peer is struggling.
- **Limits at the entrances.** The worker's `temper-worker-domain::valid`
  checks entrance constraints and count representability at start
  (configuration.md, section 7). No heap sum admits a component, and
  nothing is summed with jig's for admission.

## 6. The engine's service

In the shape of jig's `protocol/service.md`, section 2:

```
temper-service, engine module           iterate: io ↔ components ↔ root
├── temper-engine-domain                the root, with the journal (../domain/root.md)
└── the protocol layer
    ├── jig-protocol-store              commits, loads, transcripts, secrets; temper's families (store.md)
    ├── jig-protocol-engine-workers     the worker's link; temper's parts as bytes (channel.md)
    ├── jig-protocol-accounts           LLM accounts (credentials.md, section 3)
    │   └── skein-oauth-accounts
    ├── the clients' listener           temper's, on jig's server part: TCP on loopback (../client/README.md, section 8)
    │   └── jig-protocol-engine-clients-http   jig's default HTTP mapping, under /jig, with its example guards
    ├── jig-protocol-engine-clients     the clients' component: jig's messages; the forge's client family as bytes (../client/README.md, section 8)
    ├── skein-forge-connection          the forge component: every kept forge's API; the webhooks' listener, TCP on loopback (forge.md)
    ├── the setup host                  skein-forge-setup's in-process host: the setup's plans (forge.md, 2.6)
    ├── the identities' table           git identities' values (credentials.md, section 4)
    └── timers                          the root's and every component's deadlines
```

Starting, ending, `iterate`'s passes, room and timers are jig's
(jig's `protocol/service.md`, sections 3 to 5). What temper adds:

- **The forge component,** speaking each forge's API, Forgejo's REST API
  under `/api/v1` or GitHub's in skein's dialect, polled at the domain's
  cadences (skein's `forge.md`, section 5). The root's
  `Request::Forge { call, repository, op }` goes to it; it answers each
  call with exactly one `client::Event::Answered { call, cost, result }`,
  which the routing hands the root as a forge event
  (`../domain/root.md`, 5.1). An event the root cannot take yet comes
  back to the component, which holds it and offers it again (jig's
  `protocol/service.md`, section 2, room); one the root returns during
  its drain stays with the component until it closes (forge.md, section
  2; `../domain/root.md`, 9.2). Each forge's access is the component's
  alone (credentials.md, section 5).
- **A forge's start check** (credentials.md, section 5; skein's
  `forge-connection.md`, section 3): when a forge refuses its access, or
  it answers as a user other than the forge's own, that forge stops,
  never the engine: the component gives `Stopped { forge, why }`, the
  deployment's page names it with Replace (`../client/forge.md`, 5.4),
  and its calls are answered `NotConnected`; the engine and every other
  forge run on (forge.md, 2.3). Records that do not fit the configured
  forge leave it not connected, named the same way (credentials.md,
  2.1). A failure of jig's secrets' store itself stops the engine
  (credentials.md, section 5).
- **The setup host,** skein's in-process host component, beside the
  forge component: the routing runs its plans (reconcile at each start
  of a managed or owned Forgejo, a connect's, an adoption's access) and
  carries what passes between it, the root and the forge's records. It
  holds an admin's token or a GitHub App's key only while a plan runs
  (forge.md, 2.6; setup.md, the setup host in the engine). On close,
  the setup adapter retains closing requested separately from host
  `Close` sent; it drains host output independently of active notes,
  store, forge and events gates, sends `Close` once bounded host room
  permits, and completes only on actual `Closed` after io settlement
  (setup.md, 5.1 and 13.1). No application note-pop order is required;
  active-phase gates and skein's host/caller semantics remain in force.
- **The webhooks' listener,** the forge component's own, when
  `application.webhooks` names it: each delivery verified as it streams,
  answered, then made a hint, which the routing hands the root as a forge
  event (forge.md, sections 2 and 5). A hint is never a fact: polling
  finds what a lost delivery told of (forge.md, section 6).
- **temper's parts.** As it encodes an assignment, the routing encodes
  the root's `ForgeWorkspace` as temper's workspace items, with
  `temper-protocol-worker-wire`, making each repository's remote from its
  forge's `git` base and the repository's owner and name
  (forge.md, 2.1), with the owner's account id on a GitHub
  forge, so workers need no forge configuration.
  As it decodes an answer, it decodes the work and the preparation into
  the root's `Parts` (channel.md, sections 4, 5 and 7;
  `../domain/root.md`, 7.4).
- **temper's families.** The root's forge records go to the store under
  jig's application tag, encoded by `temper-protocol-store`, in the same
  commits as jig's; the forge's restart step loads them as one interval
  (store.md).
- **The identities' table,** filled from each Forgejo forge's records at
  start and from each GitHub forge's minted installation tokens as they
  come, keeping each identity's two newest generations; the routing puts
  each value in the table, then gives the root one `Event::Credential`
  for it, before its restart for a Forgejo forge's, and
  `jig-protocol-engine-workers` fills grants' values from the table
  through the routing (credentials.md, section 4; `../domain/root.md`,
  7.1).
- **Host tools, the charter and results through `temper-engine-smith`,**
  called in the routing, with smith's codecs, in the step that routes
  them, keeping no state (llm.md, sections 4 and 5; `../domain/root.md`,
  5.2).
- **The clients' listener.** The service owns it, and mounts jig's
  clients' component on it with jig's default HTTP mapping under `/jig`,
  with the mapping's example guards (`../client/README.md`, section 8; jig's
  `protocol/service.md`, section 2). A path jig's component does not claim
  goes to temper's forge routes, `/forges/...`, a GitHub connect's
  browser steps, each a setup host's run (`../client/forge.md`, 9.1), then
  to the browser's page and bundle; every other path, and a `GET` of the
  printed link's route, is answered not found.
- **The client's application wire.** The forge's client family's
  requests are decoded by `temper-client-protocol-wire` in the routing
  and answered by the root; every other part of the forge's is answered
  as unknown (`../client/README.md`, sections 4 and 8). An adoption names a
  forge, an owner and a name; the routing completes it with the forge's
  host, the host of its configured `git` base with `:port` when it names
  one (configuration.md, section 3), and hands the root the party's keyed
  request; the root locates the repository, then makes jig's adoption
  request on the party's behalf (`../domain/root.md`, 5.4;
  `../domain/forge.md`, section 4).
- **Periods,** from the smallest source, as jig's reference service gives
  them (configuration.md, section 8).

## 7. The worker's service

In the shape of jig's `protocol/service.md`, section 6:

```
temper-service, worker module           iterate
├── temper-worker-domain                the worker's root: jig's hub, smith's host domain, the checkout
└── the protocol layer
    ├── jig-protocol-worker-link        the link to the engine; temper's parts as bytes (channel.md)
    ├── smith-host-protocol             one channel per agent process, over its pipes; LLM grants filled into its start
    ├── jig-protocol-worker-agents      the program sources and the worker route, between smith's host half and the link
    ├── agent processes                 temper internal agent, each in a process group of its own
    ├── the identities' table           git identities' values, from grants (credentials.md, section 4)
    └── temper-worker-protocol-git      the git component: the checkout's io (git.md)
```

No store; starting, ending and smith's payloads are jig's (jig's
`protocol/service.md`, section 6). What temper adds:

- **Agents run as `temper internal agent`:** the agent program is
  temper's own binary, with the arguments configuration gives
  (configuration.md, section 5), its environment holding no credential.
- **temper's parts.** The routing decodes the workspace items into the
  root's `wire::Workspace` before the root sees the assignment, and
  encodes the root's `wire::Work` and `wire::Preparation` beside an
  answer (channel.md, sections 4, 5 and 7).
- **Credentials.** `jig-protocol-worker-link` takes grants' values out:
  LLM grants go to smith's host half in its form, a credential's value
  with its scheme or a program source's name (jig's `protocol/llm.md`,
  section 6); git identities' go into the identities' table,
  from which the routing hands the git component an identity's value
  with each operation that names it; the component never reads the table
  (git.md, sections 2 and 5).
- **The checkout's io.** Each `git::Op` the root asks for goes to the git
  component, which answers it with exactly one terminal; `Make` is file
  operations under the worker's root directory (git.md, section 4).
- **Local channel refusals** use jig's public `u16` constants
  `REFUSED_LIMITS = 2` and `REFUSED_FRAMING = 3` from
  `jig-protocol-worker-link`, including diagnostic matches (jig's
  `protocol/channel.md`, 10.1; skein's `channel.md`, 5.1). These are
  channel reasons, separate from jig's application reasons at 256 and
  above. Existing worker diagnostic tests check those matches; the
  wire and `Closed` facts keep their values.
- **Starting:** configuration, the components, the worker's root
  directory opened, then the link dialled at once (jig's
  `protocol/service.md`, section 6).

## 8. One process

Standalone is jig's one process (jig's `protocol/service.md`, section 10):

- **One service, one loop.** `temper-service`'s system module owns the
  engine's protocol layer and the worker's as children, each with its own
  root and journal, sharing io and nothing else; nothing crosses between
  them but bytes.
- **Joined by the in-process name `workers`:** the engine's workers'
  listener and the worker's link (configuration.md, section 2). The
  link's opening, authentication, liveness, relays and grants run as
  between hosts, in plaintext, with no TLS.
- **A client on loopback,** over TCP to the clients' listener: `temper
  open`, signing in by the link `temper serve` prints at start, or
  `temper box` before it starts serve (`setup.md`, 7.2).
- **Agents and git as child processes,** each in a process group of its
  own, which a stop signals whole (jig's `protocol/README.md`, section
  10); containment after a forced stop remains open (section 14). An
  agent's program, `claude`, is the agent's child in a group of its own
  (smith's `protocol/programs.md`, section 2), so killing an agent's
  group leaves its program running: only the agent's own stop ends it.
- **The forge outside,** reached over HTTP, and over TLS when it is not
  on loopback; its deliveries reach the webhooks' listener on loopback,
  so a forge on another host polls alone (forge.md, section 5).
- **The core knows its workers are in process:** the engine's workers'
  listener is the in-process name, from which jig's configuration tells
  the core so (jig's `protocol/configuration.md`, section 3;
  `../domain/root.md`, section 2).
- **Starting and ending together:** configuration for both, then the
  engine's start, then the worker's. A signal drains the engine; once it
  has drained, the service stops the worker, with no grace for its turns
  and answers: its runs are cancelled, each agent process ending its
  program before it exits, within smith's cancel, then kill, and the
  service closes the worker once no agent process is left, its git
  invocations' groups killed as io closes them (jig's
  `protocol/service.md`, section 10). The service exits only after. A
  second signal closes the worker at once, its agent processes' groups
  killed, and a program they started outlives them until agents run in
  contained trees (section 14). No worker outlives the engine, so the
  engine, on hearing the signal, settles the worker's agent claims
  as interrupted (jig's `domain/engine.md`, 6.1): a stop spends no try
  of a run's task, a coding run's among them, and moves none of its
  change's counts; the task is due again at the next start, its next
  attempt resuming from the turns it committed and starting from the
  work it saved last (`../domain/worker.md`, 4.2 and 5). After a crash,
  the next start settles them at once, lost or not started (jig's
  `domain/engine.md`, section 6; `../domain/root.md`, 9.1). Job claims
  remain for restart recovery even after a drain, since their effects
  may have happened.
- **Its world** is `tests/one-process` (`../domain/testing.md`, 2.5).

### 8.1 A long pass

One thread runs every stage of a pass, so a stage that takes long holds
all the others, and a client sees only that nothing answered. The service's
log says when that happened; nothing else about the loop changes.

- **The stages,** in the order a pass runs them, are `io` (io's resumes,
  completions and timers), `route` (its events to their owners), `engine`,
  `worker` and `out` (closes, the box's pipe, the components' requests,
  printed links, the publish and the reclaim). `temper-service`'s
  `iterate` runs them, and `iterate_stage` runs one. The binary's adapter
  adds its own two: `names` (a forge's name resolved) and `drain` (handover
  files, operator lines and links), the only stages that may block in a
  system call.
- **The clock.** Every stage still sees the pass's one `now`. Only the
  binary's adapter reads the monotonic clock between stages, and those
  times reach `temper-service`'s `long_pass::Watch` alone, a step that
  takes them as arguments and decides nothing but whether to write a line.
  A world that calls `iterate` has no watch.
- **The bound is 1 s,** `long_pass::LONG_PASS`, on a whole pass: no stage
  can pass it without its pass passing it. It is a thirtieth of `temper
  ctl`'s deadline (`../client/tools.md`, section 12), so a stall big enough
  to cost a call that deadline shows at once; and across 1,543 `temper ctl
  me` reads on running boxes under a load of up to 27 on 8 CPUs, the longest
  whole read was 1.95 s, so a healthy box writes few of these lines or none.
  A pass of exactly the bound writes nothing.
- **The line** is one stderr line, which the box keeps in `serve.log`
  (`setup.md`, 7.2), naming the pass, its longest stage and the bound,
  with no payload:
  `temper serve: long pass: 1250 ms, engine 1200 ms of it, past the 1000 ms bound`.
- **At most one line in 10 s,** `long_pass::LINE_EVERY`. The first long pass
  writes at once. Those after it within the 10 s are held as the longest
  of them and a count, and the first pass past the 10 s, long or not,
  writes that, ending `; 3 more since the last line`, the count being the
  long passes it does not describe. So a long stall never hides behind a
  shorter one. A held line left at the service's end is written at its exit.
- **What it costs when nothing is slow:** seven clock reads and a few
  comparisons in a pass, no allocation and no write.
- **Its tests** hold the bound, the rate and the words on supplied times:
  `temper-service`'s `tests/long_pass.rs`, and the line's words beside
  `note_line` in `temper`'s `host.rs`.

### 8.2 When the core does not answer

The engine's service hands a party's input to the root only while its
gate is open: the root started and its decisions open
(`../domain/root.md`, 9.1), the forges' records loaded and connected,
the first periods given (configuration.md, section 8) and the forges'
names resolved (forge.md, 2.2). While the gate is shut, the clients'
component answers a party's input not ready and a public read busy, at
once. A watch refused while the gate is shut says not ready, distinct
from a full watch room. An input the root hands back at its watermark
waits in the service for room (`../domain/root.md`, section 4). The
client names not ready when its answer carries that refusal; the
service's log names what the gate waits on. No gate reason travels on
the client wire (section 14).

- **The line.** When the gate has kept a party's input from the root for
  1 s or more, answered not ready or held, the service writes one stderr
  line, which the box keeps in `serve.log` (setup.md, 7.2):
  `temper serve: the core is not answering: REASON`. It writes it again
  only when the reason changes, and once the gate passes a party's
  input again after such a line, `temper serve: the core answers
  again`. The 1 s is 8.1's bound, a thirtieth of `temper ctl`'s
  deadline. The time is the service's clock, judged as each party's
  input arrives, so a deployment no client calls writes nothing, and
  the line needs no timer of its own.
- **Its reasons,** in the order the gate tests them, the first that
  holds:

  | The gate | REASON |
  |---|---|
  | the root not yet opened | `starting: STEP`, the start's step in 9.1's words: `loading the core's records`, `loading the inbox`, `loading the forge's records`, `linking the forge's records`, `sweeping`, `switching the store`, `committing the wires`, `adopting runs`, `reading the forge afresh`, `settling the forge's outbox`, `opening decisions` |
  | the forges' records not yet loaded and connected | `loading the forges' records` |
  | the first periods not yet given | `opening the first periods` |
  | the forges' names not yet resolved | `resolving the forges' names` |
  | the root's commits past its window, waiting on the store | `waiting for the store to make commit N durable, the oldest of M in flight` |
  | the root at its watermark | `at its watermark: B of W in flight` |

  A root whose journal has stopped is not among them: it ends the
  engine (below).
- **A stopped journal ends the engine.** The root's `Stop` with what
  stopped its journal (`../domain/root.md`, section 4) ends `temper
  serve` with exit 1 and one line, `temper serve: the engine's journal
  stopped: WHY`, WHY one of `the store answered commit N, with S sent
  and D durable`, `a decision past the store's E writes`, `the commit
  counter at its end`. A commit the store failed keeps its own line,
  `temper serve: commit N failed`, and a start's refusal its own lines
  (section 4).
- **Bounded, and free when the core answers:** at most 240 bytes, no
  request's words and no secret; one reason held, and a comparison as
  each party's input arrives.
- **Its tests** hold the 1 s, the change of reason, the line on the gate
  passing again and each reason's words, on a service stepped with
  supplied times, in `temper-service`'s tests; the journal's stop and
  its line beside `note_line` in `temper`'s `host.rs`.

## 9. Versions

jig's rules hold (jig's `protocol/README.md`, section 7): one version at a
time before release, a reader that refuses another and says so, and an
upgrade that stops the engine and its workers together. jig's versions
(its channel's, its store's format and families', its clients' per
owner) and smith's (its charter's, transcript's and result's) are as
jig's `protocol/README.md`, section 7 and smith's `protocol/README.md`,
section 8 give them. temper's sit beside them:

| Version | Owner | Covers | Checked |
|---|---|---|---|
| `PAYLOAD_VERSION` | `temper-protocol-worker-wire` | temper's parts: the workspace items, the saved-work branch, the work, the preparation; a git identity's value; and a delivery's evidence, smith's record as the smith temper links encodes it | the worker's credential at the link's opening: another is refused as *application* (channel.md, section 3) |
| `STORE_VERSION` | `temper-protocol-store` | temper's families | at the store's opening: a store written by another refuses to open (store.md, section 4) |
| `CLIENT_VERSION` | `temper-client-protocol-wire` | the forge's client family's records | per owner, as jig's: the engine serves exactly its own, and `temper open`, from the same binary, speaks it (`../client/README.md`, 4.6) |
| keys' markers | the forge connector | the keys temper writes into forge bodies | the key's version, its first byte; the marker has none of its own (forge.md, section 4) |
| configuration | none | — | unknown members refuse the start (configuration.md, section 2) |

- **The baseline is the first release** (skein's `codec.md`, section 5).
  Until it, each of temper's three versions keeps its current number.
  A change a reader would notice changes the schema, its generated code
  and its one set of golden bytes in place, including a repin of smith
  that changes a delivery's record (channel.md, section 3). Versions move
  at a release, not at a landing. A store from an older build is started
  afresh when its families change: its unchanged version does not tell
  the layouts apart. A reader still refuses another version.
- **Both channel components are given `PAYLOAD_VERSION`** when they are
  made; neither knows what it covers.
- **The store keeps the wires** the last start spoke (jig's
  `protocol/store.md`, 5.3, `Wires`): temper's shell hands jig's store
  `PAYLOAD_VERSION` as `temper-payload` beside jig's and smith's, so the
  next binary's rehearsal can say whether the workers may stay
  (`../draft/upgrades.md`, section 9).
- **What the first release freezes.** From it, every byte of temper's a
  deployment keeps must stay readable by every later build
  (`../draft/upgrades.md`, 4.1, 6.3, 10 and 11). Before it, in place, each
  in one change with its golden bytes:
  - **The forges' secrets from a schema:** temper's family
    `temper-secrets` (`temper-protocol-records`, `schema/secrets-v1.schema`),
    a versioned record per row of jig's secrets' store under temper's tag
    (credentials.md, 2.1): an enum of records, one per item (the three
    tokens and the webhook's secret each a `secret N`; the user and the
    app's id `u64`s, big-endian as every schema's; the app's slug a
    `text N`; its key a `secret N` holding its DER; the accepted
    installations a list; the origin its kind and address). The
    hand-written envelope and its little-endian numbers go. temper's set
    names the family at the baseline, `map: Secrets`, its keys temper's
    numbers below 2^63 (jig's `protocol/store.md`, section 9). A pasted
    bundle is a carried value, jig's from 2^63 up, kept only until the
    request that carries it is used (credentials.md, 2.2): its form is
    `temper-secrets`' `Pasted`, versioned, and one of another version is
    refused by name and pasted again, never patched.
  - **temper's store says what it holds** (store.md, section 4): a
    retained refusal `holds jig-store-people Refusal`; each list of
    answers' items, comments, reviews, statuses, remarks and files its
    `pieces`; each list of raw names (a branch's segments, protection
    contexts, labels, branch names) a declared item type and its
    `pieces`, its bytes moving in place where they must.
  - **An effect's key is an identity, never patched:** kept as written,
    in the store and in the forge's markers, compared byte for byte.
    Its first byte is its version (skein's `forge-client.md`, 7.1); from
    the first release the connector reads every version it has written,
    and a purpose's tag keeps its meaning for good, a new meaning taking
    a new tag (forge.md, section 4). The marker's wrapper and the bot's
    marker are fixed forms.
  - **Small files** (`../draft/upgrades.md`, section 10): the forge's setup
    `state`, versioned JSON, is read in every version from the first
    release on and written in the current at its next write; the
    preparation writes it before the engine starts, so it follows the
    forge's own upgrade, whose going back is its dump (setup.md, 6.3).
    The worker's secret and digest (hex digits and a newline), the
    scripted model's key, the forge's sign-in client secret (setup.md,
    6.8), the box's group files and started markers (setup.md, 7.1) and the admin token's
    handover are fixed forms, never versioned: a change is a new file
    name, read beside the old one for as long as a release may find it.
    The engine writes no small file before its start's wires are durable
    (jig's `protocol/store.md`, 5.3), so going back finds the engine's
    as they were; the preparation's `state` and an owned Forgejo's data
    are the forge's, and going back past a preparation that migrated it
    needs its dump (`../draft/upgrades.md`, 3.2 and 11).
  - **Golden bytes** pin each fixed form, each schema's records and the
    key's frame and marker, so a change fails a test naming it.

  Outside it: what an external program reads (Forgejo's and the
  runner's files, which their own upgrade carries), the fake backends'
  sandbox, logs, caches and worker roots.
- **The baseline is v0.1.0's store:** its
  disk format (skein-kv's frames and snapshot, the transcripts'
  framing, the deployment's layout) every later build reads as it finds
  it, and its records every later build patches by itself as it starts
  (`../draft/upgrades.md`, section 3). So the tag's commit makes, in one
  set across skein, smith, jig and temper:
  - **every stored family's text kept:** each owner's crate run once
    with `--baseline` (skein's `codec.md`, 12.1), its current texts its
    first kept versions, its level 0, no patch: skein-oauth's
    `skein-oauth`; smith-transcript's turn family; jig's store families
    (the core, tasks, people, notes, accounts, settings, the component's
    and `jig-store-secrets`) and `jig-accounts`; temper's `temper-store`
    and `temper-secrets`. No version moves at it;
  - **the real patch sets:** each owner's `PatchSet` from its crate's
    generated module, its families with their maps, key ranges, roots,
    the frames' families, smith's fresh word and temper's `Flight`
    range, handed by `temper-service`'s store shell in place of
    `PatchSets::NONE` (jig's `protocol/store.md`, 10.3). A store v0.1.0
    makes records every owner at level 0; one from a build before it,
    level none, is refused as older, made again (jig's
    `protocol/store.md`, 5.3);
  - **the gate's check from then on:** each repository's gate runs the
    patch command's check over each owner crate (skein's `codec.md`,
    12.7), so a stored change with no patch, an edited kept text or a
    landed patch changed fails it;
  - **the baseline's golden store,** made by the tag's binary from a
    story (a project, a person and an account, a goal's change landed,
    a chat paused, a task mid-run, a forge's records and secrets): the
    deployment's data directory whole, its store, secrets and
    transcripts, with its configuration and small files, kept for good
    beside the one-process world (`../domain/testing.md`, 2.5), never
    regenerated. Every later build starts on it and asserts what draft
    13 asks: identities and counters, policies, kept outcomes, old turns
    read in the current version (`../draft/upgrades.md`, section 13).
- **Before the tag:** every stored record with its kept form, a
  delivery's evidence among them (jig's `protocol/store.md`, section
  16, still open); the golden store's story and its later builds'
  assertion, as a test; the rehearsal below. The patch command's
  modules and the sets built from them exist only once `--baseline` has
  run (skein's `codec.md`, 12.1 and 12.3), and its check reports every
  family unkept until then (12.7), so the sets, their handing and the
  gates' check are the tag's set's, with the kept texts.
- **The baseline rehearsed before the tag:** that set made on a branch
  that never lands, from the day's schemas, its sets handed to the
  one-process world over a store a story made: the plan accepts every
  real set (at most 64 families, names at most 64 bytes, key ranges
  disjoint and each family's own, holds acyclic, one frames' family per
  slot, the manifest within its bound), a start at level 0 opens, a
  start of the build before is refused as older, and nothing is
  patched. What it finds is fixed on `main` before the tag, so the
  tag's set only repeats it.

## 10. Secrets

The secrets temper's processes hold, where each comes from, who holds it
and what it reaches are credentials.md's (section 2); the files they are
read from are configuration.md's (section 6). Across the layer:

- **Names in the domain, values beside it** (jig's `protocol/README.md`,
  section 8): no domain state, fact, record of the main store or log
  holds a value. Values move between components through the routing,
  never through a root.
- **temper's forge secrets are in jig's secrets' store:** each forge's
  records (its credentials, ids and accepted installations) under
  temper's tag, read whole at start and written only by the engine's
  protocol layer (credentials.md, 2.1), and the values clients' requests
  carry, kept by jig until their run ends (credentials.md, 2.2). The
  files read at start are configuration.md's (section 6).
- **Nothing crosses a host** in standalone: the link is in process, the
  client and the webhooks on loopback, and a forge off the machine is
  reached over TLS only (configuration.md, section 3).
- **The forge's setup's secrets:** the engine's setup host holds an
  admin's token or a GitHub App's key only while a plan runs (setup.md,
  the setup host in the engine; credentials.md, section 5). The one setup
  file `temper serve` reads is an owned forge's handover `admin_token`,
  read once and removed after its `TakeOver` is done (forge.md, 2.6;
  configuration.md, section 6).

## 11. What temper uses from jig, smith and skein

temper's protocol layer uses jig, smith and skein as they are, at the
revisions temper's `Cargo.lock` pins.

- **jig:** the components of section 5, with the service parts jig ships
  (its `protocol/service.md`, section 11), composed; the reference
  service's routing and the reference binary, copied; `jig-client` and
  its native transport; jig's client test kit,
  its check, faults and attack stories, for temper's client tiers; the
  conformance kit, as the domain's worlds use it.
- **smith:** the agent shell, the host domain and host half, the codecs;
  the agent process over skein's LLM client, and its program backend,
  Claude Code (smith's `protocol/programs.md`); smith's fake `claude`, in
  the worlds.
- **skein:** io with its in-process names, processes and process groups;
  the startup kit and its private reads; lib's SHA-256 and its HMAC;
  JSON; HTTP's client and server; TLS's client; the codec generator; the
  key-value store; the framed channel; the accounts driver, under jig's
  accounts; the UI kit and its text shell; the simulator, the fake
  machine, the fake LLM peer, the fake issuer and the fake person.
- **What is temper's own:** its presentation, since presentation is the
  application's (jig's `client/domain/client.md`), on skein's UI kit and
  text shell (`../client/README.md`, sections 5 and 6); and, where something
  would be better upstream, as jig's design allows, the fake git
  program, on skein's fake checkout and fake forge; and `temper program-login`, the
  application's command a program source's holder runs on each host
  (jig's `domain/accounts.md`, section 5).
- **Not used:** `jig-protocol-charter` and `jig-protocol-tools` (llm.md);
  agents in a host's own process (`smith-inline-agent`; section 14);
  jig's reference UI (section 5).

## 13. Testing

`../domain/testing.md` applies skein's strategy (testing-strategy.md) to
temper: its tiers, fakes, layout and budgets. What checks this layer:

- **Step tests:** every codec against golden bytes, at its bounds and by
  fuzz targets; each translation both ways; git's argument and
  environment builders, and its output parsers against transcripts of
  real git (`tools/git-transcripts`); `skein-forge`'s decoders against
  transcripts of the throwaway Forgejo (skein's conformance,
  `fake-forge.md`, section 7);
  temper's pages as skein's text shell prints them (`../client/README.md`,
  section 10).
- **Machine worlds:** the forge component against a scripted HTTP peer
  (skein's `forge-connection.md`, section 10); the git component against
  scripted process ends and outputs; the terminal's host against a
  scripted stdin and stdout.
- **The forge's protocol world,** `tests/engine/protocol-forge`: the
  forge connector world's stories through bytes, over in-memory streams
  cut at random, with the cost checks by HTTP request (`../domain/testing.md`,
  2.2).
- **The client's joined world,** `tests/client`: temper's client root
  and view, with skein's fake person and jig's check, joined to temper's
  engine root (`../domain/testing.md`, 2.1 and 6.3).
- **The one-process world,** `tests/one-process`: `temper-service` on
  skein's simulator, killed and restarted at drawn moments, drained by a
  shutdown. Its referee: nothing a party, the worker or the forge saw
  depended on a lost commit; a drained shutdown ends with nothing
  waiting for a commit; no secret's value in a domain's state, a store
  record or an agent process's environment (`../domain/testing.md`, 2.5).
- **Reconcile's world** is skein's: reconcile against the fake forge's
  protocol layer, serving the setup's routes (skein's `forge-setup.md`,
  section 7; setup.md, 13.2).
- **The process tier and the live tier:** the real binary end to end,
  outside the two suites: a chat answered, a change landed on an owned
  Forgejo with CI and webhooks, the binary killed and restarted, `temper
  box`, `temper program-login` over smith's fake `claude`, the cluster
  with a task on each of two workers alone and a worker lost (15.6);
  then a real model (`../domain/testing.md`, 2.6 and 2.7; setup.md,
  13.4).

## 14. Open questions

Each document keeps its own; these cross them:

- **The gate's reason on the client wire** (8.2): a distinct not-ready
  watch refusal lets a client distinguish startup or an unanswering core
  from watch capacity, but carries no reason. Sending the start's step
  or the gate's other reason needs jig's vocabulary, wire and client
  link to retain it. Default: the client names not ready and points to
  serve's log; the log alone names why (`../client/tools.md`, section 14).
- **The TUI's later passes** (section 4): the browser's approval,
  recording and post-landing terminal walk stay with the client's
  design (`../client/README.md`, section 11).
- **Names while running.** io connects to addresses, resolved at
  startup, while a forge's address may move behind its CDN, and a name
  that does not resolve at the start leaves its forge unreachable for
  that run, not the start refused (setup.md, 8.3). Default: a restart
  resolves names again; skein's DNS client, or skein-forge-connection
  resolving at its connect, would mend it (a skein addition).
- **The kernel's share of a pass.** A long pass (8.1) times what the
  service runs, not the reap and the submit around it, since the submit
  holds the loop's wait, and `skein-shell`'s `drive` keeps both. A stall
  there writes no line. Default: so, until a walk's log leaves a stall
  unexplained.
- **HTTP/1.1 only.** Forgejo serves it; a forge that required HTTP/2
  would need skein to grow it. Default: HTTP/1.1.
- **Generating the routing** (jig's `protocol/service.md`, section 9).
  Default: copied from jig's reference service and filled.
- **Supervision** (jig's `protocol/service.md`, section 9): what
  restarts `temper serve` after a failed commit or a crash. Default:
  nothing of `temper serve`'s own; a box restarts it (setup.md, 7.2),
  and otherwise the person or the machine's
  service manager starts it again, and it recovers from the last
  durable commit.
- **Agents in the worker's process** (`smith-inline-agent`), with no
  containment, for one person's machine (jig's `protocol/service.md`,
  section 10). Default: agent processes, as `temper internal agent`.
- **Programs after a forced agent stop** (section 8): killing an agent's
  group leaves a program in its own group running, even in standalone.
  Ending both needs contained trees or a parent-death signal for the
  program (setup.md, section 15). Default: the orderly stop lets agents
  end their programs; a second signal or forced close has no such guarantee.
- **After standalone,** each designed where it is named, and needed only
  beyond one person's machine: webhooks from another host, over TLS
  (forge.md, section 5); hosts apart: TLS on the engine's listeners,
  containment for shared hosts, a worker alone on another host (jig's
  `protocol/README.md`, section 10; on one machine it is the cluster's,
  section 15); the forge as people's sign-in
  provider (credentials.md, section 8); the forge's client documents and
  panels (`../client/forge.md`); and what the forge's setup leaves for later
  (setup.md, section 15). Default: standalone.
- **An engine restarted under live workers.** The engine announces its
  identities' values at generation 1 at each start, sound only while no
  worker outlives it (credentials.md, section 4). Default: the cluster
  restarts whole, its workers stopped before its engine (15.4);
  generations kept across starts come with hosts apart (credentials.md,
  section 8).
- **`temper init` for the cluster's shapes.** Default: none; `temper
  init` writes standalone, and the cluster's documents are its
  description's templates (15.5). Hosts apart would want it.
- **Limits across processes.** Each process checks its own document
  (configuration.md, section 7); what holds only between an engine's
  limits and a worker's, the cluster's one template holds, writing the
  same `application.limits` into each of its documents, and jig's link
  holds the rest at its opening, by its terms and the hello's bounds
  (15.3). Default: so; a digest of the limits at the
  link's opening, beside the payload's version, comes with hosts apart.
- **The description's Rust reader's home.** Default: temper's process
  tier's support (15.5); skein-world's, once a second product runs a
  cluster of its own.
- **The box's members in the engine's closure** (4.2): `temper-engine`
  checks a document's `box.` limits and `demo` member as the box's
  binary does, so it links the box's machine and the fakes' crates,
  `temper-box` and `temper-fake`, with none of their commands (section
  5). Default: so; a check of those members in a crate of no side would
  cut both from its closure.
- **The box's `versions` and `upgrade`** (section 4): the draft gives
  both to `temper-engine` (4.2) and has the box keep the commands it
  has (`../draft/deployment-binaries.md`, sections 3 and 9), and the box
  has neither yet. Default: the box's table keeps both rows as section 4
  designs them, until the draft's M5 settles the box's table
  (`../draft/deployment-binaries.execution.md`).

## 15. The cluster

The cluster is temper on several processes of one machine: one engine
alone, its workers alone, and the fakes, each a process of its own,
joined over loopback. It is a primitive of temper's tests, and the
shape of the case of `../domain/testing.md`, 5.3 on fakes: an engine and
its workers, one of them added later. It is no deployment's: `temper init` writes standalone, and a box keeps its one
worker inside `temper serve` (setup.md, 7.1). TLS, containment and other
hosts are later (15.7).

### 15.1 Its shapes

`temper serve` takes its shape from its document's members, and from
nothing else: no flag, no command of its own. A deployment's binary
serves its own shape alone and refuses the others by their nearest
shape, below (4.1): `temper-engine serve` an engine alone (4.2).

| Shape | `engine` | `workers` | `application` |
|---|---|---|---|
| standalone | jig's engine part; `hosts.listener` `[{"in_process": "workers"}]` | one, whose `engine` is `{"in_process": "workers"}` | every member; `workers`, one entry |
| engine alone | jig's engine part; `hosts.listener` one socket address on loopback, in plaintext, its port 0 or fixed; `hosts.workers` the cluster's workers, each with its digest | `[]` | every member; `workers` `[]` |
| worker alone | absent | one, whose `engine` is a socket address on loopback, in plaintext | `workers`, one entry, and `limits`, alone |

- **Never mixed.** A document of none of these shapes is refused, naming
  the path that departs from the nearest: an engine with both a worker
  and a socket listener, a listener list with an in-process name and a
  socket, a worker alone with any other member. An in-process listener
  tells jig's core that no worker outlives the engine, so it settles the
  claims of every worker at once on a restart (jig's `domain/engine.md`,
  section 6): a socket's workers among them would lose their runs.
  The nearest is read from two members: a document without `engine` is
  nearest a worker alone; one whose `engine.hosts.listener`'s first
  entry is a socket address, an engine alone; any other, standalone.
- **Each shape's checks** (configuration.md, section 2): standalone's,
  as section 2 gives them; an engine alone's the same, but for its
  listener and its workers, by the table's rule, and the root's for
  every shape with an engine; a worker alone's, its worker part's, its
  entry of `application.workers` (configuration.md, section 4) and the
  limits' checks its side holds (configuration.md, section 7).
- **The service** has an optional engine and an optional worker:
  standalone both, an engine alone no worker, and a worker alone no
  engine, whose loop, ending and exit are its worker's alone, with no
  store (sections 6 and 7; jig's `protocol/service.md`, section 6).
- **Three directories for each process,** its own (configuration.md,
  section 12): an engine alone's data holds the deployment's directory,
  the store and the secrets' store among it, and each worker's digest; a
  worker alone's, its secret, its root, where its workspaces are, and
  its programs directory. No two processes share one.
- **Memory by its service:** `engine.memory_bytes` meters the engine's
  held transcripts and turns (`../domain/root.md`, 4.1). Memory budgets
  are not startup heap sums (configuration.md, section 7); io's sockets
  belong to their own service.
- **`temper program-login`** takes a worker alone's directories as it
  takes standalone's, and reads only the worker's part: each worker
  signs its programs in on its own, and none is copied to another
  (credentials.md, section 3).

### 15.2 Its processes

```
fake forge   temper fake forge ... --ready PREFIX    (setup.md, 8.4)
fake llm     temper fake llm ... --ready PREFIX
engine       temper serve DIRS     after both fakes: an engine alone
w1, w2       temper serve DIRS     after the engine: each a worker alone
  agents, git                      its children, each a process group
```

- **Started in order, stopped in reverse:** the fakes; the engine, once
  both fakes are ready; each worker, once the engine is (15.3). A worker
  may start later than the others, as s2 does in the case of
  `../domain/testing.md`, 5.3.
- **Every address from a ready line:** each listener binds port 0 on
  127.0.0.1, and its process prints what it got; the description's
  templates carry it to the processes after it.
  The engine's document names the fakes' addresses as their ready lines
  gave them, and each worker's names the engine's workers' listener as
  its `listening workers` line gave it.
- **Workers are the engine's configured positions,**
  `engine.hosts.workers`, each with its own name and secret (jig's
  `domain/engine.md`, section 8). A worker started later is a position
  the engine knows from its start, empty until that worker links; a
  position added is a new engine document, so the cluster's restart
  (15.4).
- **The fakes are setup.md, 8.4's,** each a process of its own with its
  ready line. A worker's git is the program an assignment's repositories
  name, `temper fake git` for a fake forge (channel.md, section 4): the
  workers share the fake forge's machine, so its program's path and its
  loopback listener serve them as they serve standalone's worker.

### 15.3 Joining, and its lines

- **jig's link, in plaintext over loopback** (jig's
  `protocol/channel.md`, section 10): the worker dials the engine's
  workers' listener; the engine takes its name and its secret against
  that worker's digest, then its hello: its capability, its configured
  position, its capacity and a stop bound strictly below the engine's
  grace, `fleet.grace_ms`. The worker redials, jittered and doubling,
  until it links.
- **Secrets are the description's fixtures:** each worker's secret, and
  its digest in the engine's data, a fixed pair the description holds
  and renders into each process's directories (15.5); a test's, never a
  deployment's, and never copied from one.
- **Short graces:** the description's are seconds, each worker's stop
  bound below the engine's grace, so a lost worker's runs are placed
  again within a test's time.
- **Lines on stderr,** each one line, flushed once what it says holds;
  the cluster's contract, read by their literal prefixes:

  | Line | Printed by | When |
  |---|---|---|
  | `listening <listener> <address>` | each shape with an engine, for `clients`, `webhooks` and a socket's `workers` | once that listener is bound, with the address bound (`listening workers 127.0.0.1:41733`; an IPv6 one bracketed, `[::1]:41733`) |
  | `linked <worker> <address>` | a worker alone | at each opening of its link: the engine has taken its secret and its hello is sent, with the engine's address |

  A refused opening or hello is a note on the worker's stderr with jig's
  reason, and the worker redials, as a worker between hosts would; it
  never exits for one. The note reads `refused <worker> <address>
  <reason>`, the address the engine's as `linked` gives it, the reason
  the engine's refusal as jig's link names it (`unauthorized`,
  `application`, `busy`, `replaced`, `not-admitted`, `stopping`), a
  peer's code jig doesn't name as `code-<n>`, or the worker's own
  (`terms`, `malformed`); the engine's text, which jig does not keep,
  is not in it. A refusal is noted once until its reason changes or
  the worker links: jig's link backs the redials off, and a repeat of
  the same refusal prints nothing. The sign-in link stays on stdout
  (section 4).

  The engine says why a hello is refused (jig's `domain/engine.md`,
  section 8): the refusal's text, at most 256 bytes, is the fleet's reason
  in words, and the engine writes the same words on its own stderr,
  `temper serve: hello refused channel=<n>: <words>`. An agent's shortfall
  reads as a start's problem for it does, after its charter and model
  where it has them (`charter 11: needs 64 turns; the agent's
  profile.policy.max_turns allows 32`), so a charter set after the start,
  which no start's check saw, says the same. A refusal the fleet gives no
  reason for reads `not admitted`. A repeat of the last words prints
  nothing: a worker redials for good, and one line says it until the words
  change.
- **Each line's address is its listener's bound one:** the clients'
  server keeps its own, as the sign-in link reads it; the workers'
  routing keeps the one io's `Listening` event gives; and the webhooks'
  comes up from skein-forge-connection as its `listening` event (skein's
  `forge-connection.md`, 6.4).
- **Ready**: the engine at its `listening` lines
  and its sign-in link; a worker at its `linked` line. A task created
  before a worker's hello has reached the fleet waits for a free slot,
  so no ready line need say more.

### 15.4 Stopping, and what fails

- **Stopped in reverse**: each worker first, by
  its first termination signal, which runs its worker's ending: its runs
  stopped, and done once the engine has every turn and answer, or past
  its grace (jig's `protocol/service.md`, section 6), exiting 0; then
  the engine, drained, exiting 0; then the fakes. A process not ended
  within its cost class's `stop_seconds` (15.5) is killed, its tree with it,
  and the stop fails.
- **No survivors:** each process in a cgroup of its own, its agents and
  git within it; a stop is done once its cgroup is empty, and a process
  left in it is a survivor, killed and reported.
- **A worker lost,** its process killed mid-run, then its tree (the
  worker first, so it cannot report its agent's end as an agent's
  failure): the engine's fleet keeps
  its runs for its grace, then they are lost and their tasks retry, on a
  worker with a free slot (jig's `domain/engine.md`, section 8). The
  lost attempt is fenced, so nothing it may still have sent counts, and
  nothing lands twice.
- **A worker hung,** its tree stopped by `SIGSTOP` past the engine's
  grace, then continued: its run has been placed again; it links again,
  and the attempt it held, fenced, lands nothing.
- **A worker restarted** on its own directories links again at its
  position and takes new runs.
- **The engine ends the cluster.** An engine that exits, or is killed,
  under live workers ends the cluster's run: it is never restarted under
  them (section 14). Its workers, past their grace, cancel their runs
  and are stopped; an agent whose worker has gone sees its pipes close,
  and ends.

### 15.5 Its description

- **One fixture, `crates/temper/tests/fixtures/cluster/`,** whose
  description, `experiment.json`, has the format `lab-experiment/1` and
  the cost label `temper-m1-calibration`. Each process keeps its `argv`, whole `env`, `ready`
  lines (15.3), `after`, `memory_bytes` (15.1) and `files`: its
  document rendered from a template, and its synthetic secret or digests.
  Templates name addresses by `{proc.P.ready.C}` and directories by
  `{proc.P.dir}`. The description is canonical JSON, with integers
  within ±(2^53−1); each process's `memory_bytes` matches its document.
- **The process tier's reader,** `crates/temper/tests/support/cluster.rs`:
  templates rendered; processes started by skein-world's `Binary`, in
  cgroups, by `after`, ties in name order, with no seed; ready lines
  awaited on streams; processes stopped in reverse, a termination signal,
  then the stop bound, then the tree killed. It reads none of the
  description's `actions`, `faults`, `collect` or `checks`, which are
  empty.
- **The reader's bindings:** `{artifact.temper}` is the absolute
  binary Cargo names to the test (`CARGO_BIN_EXE_temper`);
  `{artifact.git}` is its fake-git link; `{experiment}` is this
  fixture's absolute directory; `{proc.P.dir}` is P's scratch
  directory, made `0700`; and `{proc.P.ready.C}` is a capture once
  P is ready. A seed, run directory, browser or other unknown binding
  fails the test.
- **Bounds:** readiness is 120 seconds per process from its
  start, stopping 20 seconds per process. The stop bound stays above
  each worker's stop bound and the engine's drain. The reader's
  constants and the fixture's memory declarations are tested.
- **The fake forge's setup** is `temper serve`'s handover and reconcile
  at start, from records (setup.md, 6.5). The fixture holds the fake
  forge, fake LLM, engine and workers; a story supplies its fake setup
  (`../domain/testing.md`, 2.6). Its tasks are chats, its referee checks
  their ends on named workers and no survivor. Each chat keeps a worker's
  slot by an idle threshold and ordinary time past the story's deadline,
  never by timing. The forge starts before the engine.
- **The reader is temper's** until a second product needs one
  (section 14). Shared data keeps its process stories on one fixture.
- **Cgroups, or no cluster:** several processes observed by one test
  need skein-world's cgroup trees, where the user's service manager
  delegates them; walking refuses a second observer. Without them, the
  cluster's tests fail at their start, saying why.

### 15.6 Its tests

Each where `../domain/testing.md` puts it (2.4 to 2.6):

| Where | What it shows |
|---|---|
| step tests | each shape read and checked, a document of none refused, naming its path; each line of 15.3 as printed |
| the engine's domain | two workers at positions 0 and 1, one slot each: two tasks, one placed on each; one lost, its run placed on the other past the grace |
| the real loop | an engine alone and two workers alone, three services on one ring with their own directories, the workers dialling the engine's listener on loopback: a task on each runs to its end |
| the process tier | `crates/temper/tests/fixtures/cluster/` on real processes, as below, and case 4.6's restart boundary (`../domain/testing.md`, 5.3 and 2.6) |

The process tier's first stories, one slot on each worker: the
cluster's fixtures give each worker one. Two tasks on two workers need
no more; the cluster's workers of several slots are later (15.7).


- **A task on each worker:** two tasks on the fakes, one run on w1 and
  one on w2, each to its end; then the cluster stopped, each process
  exiting 0, no survivor.
- **A worker lost:** w1 killed mid-run; its task's run is placed on w2
  past the grace, and lands once. Its process is killed first, then
  its tree, so the engine sees its link close, not an agent's end, and
  no silence is waited on. A waiting
  run's turn is committed only when the run ends, so the killed
  attempt's reply never reaches the engine; the retried run resumes
  from the chat's committed turns and answers anew, the reply is in
  them once, and the lost attempt's run ends as lost in the task's
  activity. The killed process's end is the story's, not a failure.
- **A worker hung:** w1 stopped past the grace, then continued; its run,
  placed on w2, lands once, and w1's attempt lands nothing. First, a
  stop past the engine's silence alone, continued within the fleet's
  grace, heals: w1 links again and the same attempt answers there,
  nothing placed twice. Then a stop held until the run is seen on w2
  (never a length of time): once continued, w1 links again, its
  listing of the fenced attempt is cancelled and what it sends of it
  dropped, and w1 takes a new run on its one slot, which shows it
  acted on the cancel, whose end sent the fenced attempt's held turn;
  the fenced attempt's activity and the chat's turns are as they were
  before the continue, and the reply is in them once.
- **A worker started later:** w2 started once w1's slot is taken; the
  second task waits, due and unplaced, with no attempt (the engine
  admits a run only once a slot is free), then runs on w2 once it
  links. A restarted worker links again at its position: w1, its run
  ended, is stopped alone, exits 0, and is started again on its own
  directories; it prints `linked` again, is the engine's w1 again, and
  takes a new run.
- **Every story waits on what shows each step,** read through the
  client and the processes' lines, never on a clock, within one bound;
  w2 starts only once w1 holds its run, so which worker takes a chat is
  never left to placement's order; and each chat is closed and ended
  before the cluster stops, since a worker's ending under a waiting
  run fails it into a retry.
- **A mixed document,** each shape's departures, refuses its start,
  exiting 1 and naming the path.

These are the case's multi-worker steps at their smallest
(`../domain/testing.md`, 5.3): 4.1's placement by free slots, 4.3's
worker added later, 4.6's hung worker.

- **The case's restart,** a story on the same
  description, the case's workers named ws, s1 and s2, its fixture
  applied to the engine's and the workers' documents, its acts by real
  `temper ctl`, and its oracle over what the processes observed
  (`../domain/testing.md`, 2.6): 4.6's restarted
  workstation, ws stopped under the director's waiting chat and started
  again on its own directories.

### 15.7 Later

- **A worker stopped apart from its engine** (15.4): its Shutdown
  cancel's agent answer counts as `Failed(Agent)`, spending an agent
  try while the engine stays up. Giving that restart the standalone
  stop's no-try interruption (section 8) is left for the cluster
  (jig's `protocol/service.md`, section 9).
- **The engine's refusal text on the worker** (15.3): the text is sent
  over the channel, but jig's worker link retains only the reason code
  for its note. Keeping the words for a worker's stderr needs a change
  through that link. Default: the engine prints the words and the
  worker prints its code, suppressing repeats as 15.3 says.
- **TLS on the workers' listener, contained process trees, and workers
  on other hosts,** with hosts apart (section 14): a worker alone on
  another machine is the same document, its `engine` in TLS (jig's
  `protocol/README.md`, section 10).
- **An engine restarted under live workers,** with generations kept
  across starts (section 14).
- **The cluster's workers of several slots,** as the case of
  `../domain/testing.md`, 5.3 has: a worker hosts an agent per slot
  (`../domain/worker.md`, section 6), and the cluster's fixtures and
  stories still give each of its workers one slot.
- **`temper init` for the cluster's shapes** (section 14).
