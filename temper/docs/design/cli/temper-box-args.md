# temper-box: arguments

Generated from temper-box's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `temper-box`

- **Set:** public
- **Forms:** `[--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]`
- Run a deployment as one box: the setup steps of each forge it sets up, then temper-box serve, each a child process.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`, `--echo none|steps`
- **Notes:** In a terminal, with --echo none, the box runs the TUI in front, signed in, with its link shown there. The block, --fake-backends lines and errors go to the box log while it runs; quitting the TUI stops the box. Steps run in order, each restarted when it exits until it has exited too often within its window, when the box gives up, stops the others and exits 1; they stop in reverse order. Its stdout is its sign-in links, each line whole; its stderr, the block's other lines and errors, and what --echo chooses; the box and each step log under the data directory.

### `temper-box init`

- **Set:** public
- **Forms:** `[DIRECTORIES]`
- Write a deployment that starts, with temper's default configuration, into the three directories.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Alone, init chooses two ports on `127.0.0.1`, preferring 8367 for clients and 8371 for webhooks. With `--box`, it adds the welcome, owned Forgejo and playground: five ports, preferring 8367 for clients, 8368 for owned Forgejo, 8369 for the fake forge, 8370 for the demo LLM and 8371 for webhooks. Taken ports fall back to the kernel. It writes matching addresses in `temper.json` and `agent.json`.

To choose ports for another box, leave the first running and give init
separate empty configuration, data and cache directories. No port flag
or coordinated edits are needed. See temper help box, step 1.

The saved addresses are fixed: later starts reuse them and refuse an
occupied address. A separate `temper-box init` closes its reserved listeners
on exit. On empty directories, `temper-box` runs init itself and keeps
all five reservations through startup; use that first-run path to avoid
a gap between selection and startup.

Refused, changing nothing, when the configuration directory already holds temper.json.

### `temper-box serve`

- **Set:** public
- **Forms:** `[DIRECTORIES]`
- Run the deployment's services, as its configuration's shape names them, until stopped.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Its stdout carries only sign-in links, one a line; its stderr, its startup notes and the reason of a failure. A first termination signal settles the engine's forge writes and drains it; a second aborts. Exit 0 after a drained stop, 1 on a refused start or any failure.

### `temper-box program-login`

- **Set:** public
- **Forms:** `[DIRECTORIES] NAME`
- Sign a program source in on this host, through the program's own sign-in.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `NAME`: The program source's name: 1 to 32 of a to z, 0 to 9 and -.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Checks the configuration and the agent's, makes the source's directory, then replaces itself by the program's sign-in, whose exit status is then its own. The engine need not run.

### `temper-box help`

- **Set:** public
- **Forms:** `[NAME | pages | --search WORDS...]`
- Print this, a page, the pages, or a search.
- **Arguments:** `NAME`: A page's name, as the index and the pages give it.; `WORDS`: Words that a section must hold every one of, case folded: at most 8, each at most 32 bytes.
- **Notes:** Reads no directory and no configuration, so it works before temper init. A name that is no page, or a search that matches nothing, exits 1. A search prints at most 20 hits, a line each, NAME: HEADING: LINE, then "and more; narrow the words" when more matched.

### `temper-box versions`

- **Set:** public
- **Forms:** ``
- Print this binary's patch levels, its store's and families' versions, and its wires, one line each.
- **Notes:** Reads no configuration and no store.

### `--config CONFIG`

- **Set:** public
- **Default:** $XDG_CONFIG_HOME/temper
- **Twin:** `TEMPER_CONFIG`
- Configuration, including temper.json.

### `--data DATA`

- **Set:** public
- **Default:** $XDG_DATA_HOME/temper
- **Twin:** `TEMPER_DATA`
- The deployment's data and secrets.

### `--cache CACHE`

- **Set:** public
- **Default:** $XDG_CACHE_HOME/temper
- **Twin:** `TEMPER_CACHE`
- Binaries that can be fetched again.

### `--echo none|steps`

- **Set:** public
- **Default:** none
- Show errors and a failed step's log, or every step's lines too.

## Internal

### `temper-box forge fetch`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Download an owned forge's Forgejo and runner binaries into the cache.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** Fetches only the binaries the forge's setup gives as "fetched", at temper's pinned versions, for amd64 or arm64; another machine is refused, naming its architecture. It says how far each binary has gone on its stdout, a line at each percent. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.

### `temper-box forge prepare`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Prepare an owned forge before its Forgejo first runs.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.

### `temper-box forge run`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE forgejo|runner`
- Run an owned forge's Forgejo or its runner.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** Checks the binary and the preparation, and takes the forge's lock, first. Run alone, it replaces itself by the program, whose exit status is then its own; given the forge's listener, as the box gives it, it runs the program as its child. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.

### `temper-box forge remove`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Remove a stopped box slot's forge directory, keeping the cache.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** Reads the slot's kind from box/groups/FORGE and refuses a directory outside data or in the cache.

### `temper-box forge source`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Add the box’s public temper sign-in source to its owned Forgejo.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** Lists the sources and adds temper when absent, under the setup deadline. Prints source added or source failed WHY and exits 0 either way: a failure keeps the box running with plain links.

### `temper-box forge sign-in`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Make what people need to sign in to temper through a Forgejo, and print the Forgejo command that registers temper.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** For a Forgejo of application.sign_in.forges, on temper's host, before temper serve. Writes the provider's key and the forge's client secret where they are missing, private, and leaves any already there; prints Forgejo's add-oauth command, never running it.

### `temper-box ctl`

- **Set:** internal
- **Forms:** `DIR GROUP VERB [ARGUMENT...] [--json]`, `DIR me|inbox|sign-in|sign-out|pending [ARGUMENT...] [--json]`, `DIR --call FILE [--json]`, `help [GROUP [VERB]]`
- Act in a deployment as one party, one call a process: the agents' command line.
- **Arguments:** `DIR`: ctl's private directory for one deployment and one party: its sign-in, its kept requests and the engine's address.; `GROUP`: A group of tools, a noun of temper's or one of the client's own names; temper ctl help lists them.; `VERB`: One of the group's tools; temper ctl help GROUP lists them.; `FILE`: One call as JSON, {"tool": "GROUP VERB", "arguments": {...}}, or - for stdin.
- **Notes:** temper ctl help prints how to work through it, then the groups. A call holds one of the engine's streams while it runs, a wait for its whole span; past the engine's bound a call says so.

### `temper-box open`

- **Set:** internal
- **Forms:** `DIR LINK_OR_ADDRESS`
- Open a person's terminal client on a deployment.
- **Arguments:** `DIR`: The client's private directory, never one that ctl uses.; `LINK_OR_ADDRESS`: A sign-in link temper serve printed, or the address of an engine the directory is signed in to.
- **Notes:** For people; an agent works through temper ctl.

### `temper-box internal agent`

- **Set:** internal
- **Forms:** `AGENT.json`
- Run one agent for a worker's run, on smith's agent configuration.
- **Arguments:** `AGENT.json`: The agent's configuration, as the worker's agent.configuration names it.

### `temper-box internal agent-list`

- **Set:** internal
- **Forms:** `AGENT.json`
- Read one endpoint's model list, on smith's agent configuration.
- **Arguments:** `AGENT.json`: The agent's configuration, as the worker's agent.configuration names it.

### `temper-box internal serve`

- **Set:** internal
- **Forms:** `[DIRECTORIES] --box`
- Run temper-box serve as the box's step, on the listeners and the pipe the box gives it.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- **Flags:** `--box`

### `temper-box internal forge-child`

- **Set:** internal
- **Forms:** `--parent PID --signal kill|terminate -- PROGRAM ARGS...`
- Run an owned forge's program, sent SIGNAL when its parent PID ends.
- **Arguments:** `PID`: The process whose end ends the program.; `PROGRAM`: The program to run, with its ARGS.

### `temper-box internal tui`

- **Set:** internal
- **Forms:** `--box ADDRESS`
- Run the TUI in front of the box that started it, signed in by the link the box hands it.
- **Arguments:** `ADDRESS`: The box's engine, http://HOST:PORT, on loopback.

### `temper-box fake llm`

- **Set:** internal
- **Forms:** `[DIRECTORIES] [--ready PREFIX] [--seed N] [--observations FILE]`, `--document FILE [--ready PREFIX] [--seed N] [--observations FILE]`
- Serve a fake LLM provider: the demo's, from the configuration, or one from a JSON document.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- **Flags:** `--document FILE`, `--ready PREFIX`, `--seed N`, `--observations FILE`

### `temper-box fake forge`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE [--ready PREFIX] [--seed N] [--observations FILE]`, `--document FILE [--ready PREFIX] [--seed N] [--observations FILE]`
- Serve a fake forge: forge FORGE of the configuration, or one from a JSON document.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Flags:** `--document FILE`, `--ready PREFIX`, `--seed N`, `--observations FILE`

### `temper-box fake forge check`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE OBSERVATIONS STORE`
- Judge a fake forge's observations against a stopped deployment's store.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.; `OBSERVATIONS`: The observations file the fake forge wrote.; `STORE`: A copy of the deployment's store.

### `temper-box fake oauth`

- **Set:** internal
- **Forms:** `--document FILE [--ready PREFIX] [--seed N]`
- Serve a fake OAuth issuer from a JSON document.
- **Flags:** `--document FILE`, `--ready PREFIX`, `--seed N`

### `temper-box fake git`

- **Set:** internal
- **Forms:** `ARGS...`
- Run temper's fake git with git's arguments, as the demo's git does.

### run as `git`

- **Set:** internal
- **Forms:** `ARGS...`
- Run temper's fake git with git's arguments, as the demo's git does.

### run as `claude`

- **Set:** internal
- **Forms:** `ARGS...`
- Run the Claude Code launcher with its delegated arguments.

### `temper-box --mcp-relay`

- **Set:** internal
- **Forms:** `127.0.0.1:PORT`
- Relay a program's MCP connection to its agent.
- **Arguments:** `PORT`: The agent's relay listener on loopback; the secret is in SMITH_MCP_SECRET.

### `temper-box --program-child`

- **Set:** internal
- **Forms:** `PARENT PROGRAM ARGUMENTS...`
- Run a program as the agent's child, which ends with its parent.
- **Arguments:** `PARENT`: The agent's process id.; `PROGRAM`: The program's path, with its ARGUMENTS.

### `temper-box --list internal agent-list`

- **Set:** internal
- **Forms:** `AGENT.json`
- Read one endpoint's model list, on smith's agent configuration.

### `--bundle BUNDLE`

- **Set:** internal
- The web client's bundle directory; a relative path is taken from the working directory.
- **Notes:** Requires --fake-backends; absent, use the configured bundle.

### `--fake-backends`

- **Set:** internal
- Run against temper's fake LLM provider, forge and OAuth issuer.
- **Notes:** An isolated deployment under DATA/box/fake-backends, sharing the cache.

### `--no-tui`

- **Set:** internal
- Run the box with no TUI in front, as on a pipe.

### `--box`

- **Set:** internal
- Also write the box's parts: the welcome, the owned Forgejo and the playground.

### `--document FILE`

- **Set:** internal
- The fake LLM's JSON document.

### `--ready PREFIX`

- **Set:** internal
- Print PREFIX and the bound address once.

### `--seed N`

- **Set:** internal
- **Default:** 0
- The domain and credential seed, a decimal u64.

### `--observations FILE`

- **Set:** internal
- Create the LLM's JSON lines query stream.
