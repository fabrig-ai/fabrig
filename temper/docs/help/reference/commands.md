# temper's commands

Kind: reference

Every command of temper's binaries, a section each: its usage, what it does, its arguments and its notes. A command two binaries share is written under the first and named under the others.

## temper

temper's TUI, on an engine or in front of a box.

### temper

`temper [ADDRESS]`

Run temper's full-screen terminal client, the TUI, on an engine; temper help tui explains closing and signing out.

- `ADDRESS`: Engine listener, http://HOST:PORT on loopback; defaults to the engine link file's address for this environment, else the TUI's last sign-in address, else http://127.0.0.1:8367.

Needs a terminal on stdin and stdout; without one, bare temper says it has no command. Its sign-in link is pasted on its screen, never given. Its directory is $XDG_STATE_HOME/temper/tui, else ~/.local/state/temper/tui. `/exit` or Ctrl+C closes it without signing out; on the sign-in screen, use Ctrl+C. `q` types a character on every screen.

### temper box

`temper box [ARGUMENT...]`

Run this build's temper-box, fetching it if needed.

- `ARGUMENT...`: Arguments for temper-box, passed on unchanged.

Finds temper-box beside this executable, then in the cache, then at the download site. A checkout build downloads none: build it with cargo build -p temper-box. Runs the box in this process's place, including in a terminal.

### temper help

`temper help [NAME | pages | --search WORDS...]`

Print this, a page, the pages, or a search.

- `NAME`: A page's name, as the index and the pages give it.
- `WORDS`: Words that a section must hold every one of, case folded: at most 8, each at most 32 bytes.

Reads no directory and no configuration, so it works before temper init. A name that is no page, or a search that matches nothing, exits 1. A search prints at most 20 hits, a line each, NAME: HEADING: LINE, then "and more; narrow the words" when more matched.

### temper ctl

`temper ctl DIR GROUP VERB [ARGUMENT...] [--json]`

`temper ctl DIR me|inbox|sign-in|sign-out|pending [ARGUMENT...] [--json]`

`temper ctl DIR --call FILE [--json]`

`temper ctl help [GROUP [VERB]]`

Act in a deployment as one party, one call a process: the agents' command line. Internal: it may change without notice.

- `DIR`: ctl's private directory for one deployment and one party: its sign-in, its kept requests and the engine's address.
- `GROUP`: A group of tools, a noun of temper's or one of the client's own names; temper ctl help lists them.
- `VERB`: One of the group's tools; temper ctl help GROUP lists them.
- `FILE`: One call as JSON, {"tool": "GROUP VERB", "arguments": {...}}, or - for stdin.

temper ctl help prints how to work through it, then the groups. A call holds one of the engine's streams while it runs, a wait for its whole span; past the engine's bound a call says so.

### temper internal fetch-box

`temper internal fetch-box [--cache DIR]`

Find this build's temper-box, fetching it if needed, for temper box. Internal: it may change without notice.

Writes progress lines, then found PATH on stdout; failures on stderr, exit 1. Reads no terminal.

## temper-engine

the deployment's engine, with the owned forge's commands.

### temper-engine serve

`temper-engine serve [DIRECTORIES]`

Run an engine alone's document until stopped; temper-worker serve runs a worker alone, and temper-box serve runs an engine and its worker in one process.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.

Its stdout carries only sign-in links, one a line; its stderr, its startup notes and the reason of a failure. A first termination signal settles the engine's forge writes and drains it; a second aborts. Exit 0 after a drained stop, 1 on a refused start or any failure. Refuses a worker alone's document, naming temper-worker serve, or an engine and its worker in one process, naming temper-box serve.

### temper-engine versions

`temper-engine versions`

Print this binary's patch levels, its store's and families' versions, and its wires, one line each.

Reads no configuration and no store.

### temper-engine help

`temper-engine help [NAME | pages | --search WORDS...]`

As temper help, above.

### temper-engine forge fetch

`temper-engine forge fetch [DIRECTORIES] FORGE`

Download an owned forge's Forgejo and runner binaries into the cache. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- `FORGE`: A forge's number in the configuration.

Fetches only the binaries the forge's setup gives as "fetched", at temper's pinned versions, for amd64 or arm64; another machine is refused, naming its architecture. It says how far each binary has gone on its stdout, a line at each percent. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.

### temper-engine forge prepare

`temper-engine forge prepare [DIRECTORIES] FORGE`

Prepare an owned forge before its Forgejo first runs. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- `FORGE`: A forge's number in the configuration.

A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.

### temper-engine forge run

`temper-engine forge run [DIRECTORIES] FORGE forgejo|runner`

Run an owned forge's Forgejo or its runner. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- `FORGE`: A forge's number in the configuration.

Checks the binary and the preparation, and takes the forge's lock, first. Run alone, it replaces itself by the program, whose exit status is then its own; given the forge's listener, as the box gives it, it runs the program as its child. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.

### temper-engine forge sign-in

`temper-engine forge sign-in [DIRECTORIES] FORGE`

Make what people need to sign in to temper through a Forgejo, and print the Forgejo command that registers temper. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- `FORGE`: A forge's number in the configuration.

For a Forgejo of application.sign_in.forges, on temper's host, before temper serve. Writes the provider's key and the forge's client secret where they are missing, private, and leaves any already there; prints Forgejo's add-oauth command, never running it.

### temper-engine forge source

`temper-engine forge source [DIRECTORIES] FORGE`

Add the box’s public temper sign-in source to its owned Forgejo. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- `FORGE`: A forge's number in the configuration.

Lists the sources and adds temper when absent, under the setup deadline. Prints source added or source failed WHY and exits 0 either way: a failure keeps the box running with plain links.

### temper-engine forge remove

`temper-engine forge remove [DIRECTORIES] FORGE`

Remove a stopped box slot's forge directory, keeping the cache. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- `FORGE`: A forge's number in the configuration.

Reads the slot's kind from box/groups/FORGE and refuses a directory outside data or in the cache.

### temper-engine internal forge-child

`temper-engine internal forge-child --parent PID --signal kill|terminate -- PROGRAM ARGS...`

Run an owned forge's program, sent SIGNAL when its parent PID ends. Internal: it may change without notice.

- `PID`: The process whose end ends the program.
- `PROGRAM`: The program to run, with its ARGS.

## temper-worker

the deployment's worker: a worker alone, and its programs' sign-in.

### temper-worker serve

`temper-worker serve [DIRECTORIES]`

Run a worker alone's document until stopped; temper-engine serve runs an engine alone, and the box's temper-box serve runs an engine and its worker in one process.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.

Its stderr carries its startup notes, its linked and lost lines and the reason of a failure. A first termination signal runs its worker's ending; a second aborts. Exit 0 after its ending, 1 on a refused start or any failure. Refuses an engine alone's document, naming temper-engine serve, or an engine and its worker in one process, naming the box's temper-box serve.

### temper-worker program-login

`temper-worker program-login [DIRECTORIES] NAME`

Sign a program source in on this host, through the program's own sign-in.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- `NAME`: The program source's name: 1 to 32 of a to z, 0 to 9 and -.

Checks the configuration and the agent's, makes the source's directory, then replaces itself by the program's sign-in, whose exit status is then its own. The engine need not run.

### temper-worker help

`temper-worker help [NAME | pages | --search WORDS...]`

As temper help, above.

## temper-agent

the deployment's agent, which its worker runs for each run.

### temper-agent help

`temper-agent help [NAME | pages | --search WORDS...]`

As temper help, above.

### temper-agent

`temper-agent AGENT.json`

Run one agent for a worker's run, on smith's agent configuration. Internal: it may change without notice.

- `AGENT.json`: The agent's configuration, as the worker's agent.configuration names it.

### temper-agent --list

`temper-agent --list AGENT.json`

Read one endpoint's model list, on smith's agent configuration. Internal: it may change without notice.

- `AGENT.json`: The agent's configuration, as the worker's agent.configuration names it.

### temper-agent --mcp-relay

`temper-agent --mcp-relay 127.0.0.1:PORT`

Relay a program's MCP connection to its agent. Internal: it may change without notice.

- `PORT`: The agent's relay listener on loopback; the secret is in SMITH_MCP_SECRET.

### temper-agent --program-child

`temper-agent --program-child PARENT PROGRAM ARGUMENTS...`

Run a program as the agent's child, which ends with its parent. Internal: it may change without notice.

- `PARENT`: The agent's process id.
- `PROGRAM`: The program's path, with its ARGUMENTS.

### run as claude

`claude ARGS...`

Run the Claude Code launcher with its delegated arguments. Internal: it may change without notice.

## temper-box

temper on this machine, as one box.

### temper-box

`temper-box [--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]`

Run a deployment as one box: the setup steps of each forge it sets up, then temper-box serve, each a child process.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.

In a terminal, with --echo none, the box runs the TUI in front, signed in, with its link shown there. The block, --fake-backends lines and errors go to the box log while it runs; quitting the TUI stops the box. Steps run in order, each restarted when it exits until it has exited too often within its window, when the box gives up, stops the others and exits 1; they stop in reverse order. Its stdout is its sign-in links, each line whole; its stderr, the block's other lines and errors, and what --echo chooses; the box and each step log under the data directory.

### temper-box init

`temper-box init [DIRECTORIES]`

Write a deployment that starts, with temper's default configuration, into the three directories.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.

Alone, init chooses two ports on `127.0.0.1`, preferring 8367 for clients and 8371 for webhooks. With `--box`, it adds the welcome, owned Forgejo and playground: five ports, preferring 8367 for clients, 8368 for owned Forgejo, 8369 for the fake forge, 8370 for the demo LLM and 8371 for webhooks. Taken ports fall back to the kernel. It writes matching addresses in `temper.json` and `agent.json`.

To choose ports for another box, leave the first running and give init
separate empty configuration, data and cache directories. No port flag
or coordinated edits are needed. See temper help box, step 1.

The saved addresses are fixed: later starts reuse them and refuse an
occupied address. A separate `temper-box init` closes its reserved listeners
on exit. On empty directories, `temper-box` runs init itself and keeps
all five reservations through startup; use that first-run path to avoid
a gap between selection and startup.

Refused, changing nothing, when the configuration directory already holds temper.json.

### temper-box serve

`temper-box serve [DIRECTORIES]`

Run the deployment's services, as its configuration's shape names them, until stopped.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.

Its stdout carries only sign-in links, one a line; its stderr, its startup notes and the reason of a failure. A first termination signal settles the engine's forge writes and drains it; a second aborts. Exit 0 after a drained stop, 1 on a refused start or any failure.

### temper-box program-login

`temper-box program-login [DIRECTORIES] NAME`

As temper-worker program-login, above.

### temper-box help

`temper-box help [NAME | pages | --search WORDS...]`

As temper help, above.

### temper-box versions

`temper-box versions`

As temper-engine versions, above.

### temper-box forge fetch

`temper-box forge fetch [DIRECTORIES] FORGE`

As temper-engine forge fetch, above.

### temper-box forge prepare

`temper-box forge prepare [DIRECTORIES] FORGE`

As temper-engine forge prepare, above.

### temper-box forge run

`temper-box forge run [DIRECTORIES] FORGE forgejo|runner`

As temper-engine forge run, above.

### temper-box forge remove

`temper-box forge remove [DIRECTORIES] FORGE`

As temper-engine forge remove, above.

### temper-box forge source

`temper-box forge source [DIRECTORIES] FORGE`

As temper-engine forge source, above.

### temper-box forge sign-in

`temper-box forge sign-in [DIRECTORIES] FORGE`

As temper-engine forge sign-in, above.

### temper-box ctl

`temper-box ctl DIR GROUP VERB [ARGUMENT...] [--json]`

`temper-box ctl DIR me|inbox|sign-in|sign-out|pending [ARGUMENT...] [--json]`

`temper-box ctl DIR --call FILE [--json]`

`temper-box ctl help [GROUP [VERB]]`

As temper ctl, above.

### temper-box open

`temper-box open DIR LINK_OR_ADDRESS`

Open a person's terminal client on a deployment. Internal: it may change without notice.

- `DIR`: The client's private directory, never one that ctl uses.
- `LINK_OR_ADDRESS`: A sign-in link temper serve printed, or the address of an engine the directory is signed in to.

For people; an agent works through temper ctl.

### temper-box internal agent

`temper-box internal agent AGENT.json`

Run one agent for a worker's run, on smith's agent configuration. Internal: it may change without notice.

- `AGENT.json`: The agent's configuration, as the worker's agent.configuration names it.

### temper-box internal agent-list

`temper-box internal agent-list AGENT.json`

Read one endpoint's model list, on smith's agent configuration. Internal: it may change without notice.

- `AGENT.json`: The agent's configuration, as the worker's agent.configuration names it.

### temper-box internal serve

`temper-box internal serve [DIRECTORIES] --box`

Run temper-box serve as the box's step, on the listeners and the pipe the box gives it. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.

### temper-box internal forge-child

`temper-box internal forge-child --parent PID --signal kill|terminate -- PROGRAM ARGS...`

As temper-engine internal forge-child, above.

### temper-box internal tui

`temper-box internal tui --box ADDRESS`

Run the TUI in front of the box that started it, signed in by the link the box hands it. Internal: it may change without notice.

- `ADDRESS`: The box's engine, http://HOST:PORT, on loopback.

### temper-box fake llm

`temper-box fake llm [DIRECTORIES] [--ready PREFIX] [--seed N] [--observations FILE]`

`temper-box fake llm --document FILE [--ready PREFIX] [--seed N] [--observations FILE]`

Serve a fake LLM provider: the demo's, from the configuration, or one from a JSON document. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.

### temper-box fake forge

`temper-box fake forge [DIRECTORIES] FORGE [--ready PREFIX] [--seed N] [--observations FILE]`

`temper-box fake forge --document FILE [--ready PREFIX] [--seed N] [--observations FILE]`

Serve a fake forge: forge FORGE of the configuration, or one from a JSON document. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- `FORGE`: A forge's number in the configuration.

### temper-box fake forge check

`temper-box fake forge check [DIRECTORIES] FORGE OBSERVATIONS STORE`

Judge a fake forge's observations against a stopped deployment's store. Internal: it may change without notice.

- `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- `FORGE`: A forge's number in the configuration.
- `OBSERVATIONS`: The observations file the fake forge wrote.
- `STORE`: A copy of the deployment's store.

### temper-box fake oauth

`temper-box fake oauth --document FILE [--ready PREFIX] [--seed N]`

Serve a fake OAuth issuer from a JSON document. Internal: it may change without notice.

### temper-box fake git

`temper-box fake git ARGS...`

Run temper's fake git with git's arguments, as the demo's git does. Internal: it may change without notice.

### run as git

`git ARGS...`

Run temper's fake git with git's arguments, as the demo's git does. Internal: it may change without notice.

### run as claude

`claude ARGS...`

As temper-agent's run as claude, above.

### temper-box --mcp-relay

`temper-box --mcp-relay 127.0.0.1:PORT`

As temper-agent --mcp-relay, above.

### temper-box --program-child

`temper-box --program-child PARENT PROGRAM ARGUMENTS...`

As temper-agent --program-child, above.

### temper-box --list internal agent-list

`temper-box --list internal agent-list AGENT.json`

Read one endpoint's model list, on smith's agent configuration. Internal: it may change without notice.
