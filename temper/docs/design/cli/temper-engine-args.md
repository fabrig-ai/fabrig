# temper-engine: arguments

Generated from temper-engine's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `temper-engine serve`

- **Set:** public
- **Forms:** `[DIRECTORIES]`
- Run an engine alone's document until stopped; temper-worker serve runs a worker alone, and temper-box serve runs an engine and its worker in one process.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Its stdout carries only sign-in links, one a line; its stderr, its startup notes and the reason of a failure. A first termination signal settles the engine's forge writes and drains it; a second aborts. Exit 0 after a drained stop, 1 on a refused start or any failure. Refuses a worker alone's document, naming temper-worker serve, or an engine and its worker in one process, naming temper-box serve.

### `temper-engine versions`

- **Set:** public
- **Forms:** ``
- Print this binary's patch levels, its store's and families' versions, and its wires, one line each.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Reads no configuration and no store.

### `temper-engine help`

- **Set:** public
- **Forms:** `[NAME | pages | --search WORDS...]`
- Print this, a page, the pages, or a search.
- **Arguments:** `NAME`: A page's name, as the index and the pages give it.; `WORDS`: Words that a section must hold every one of, case folded: at most 8, each at most 32 bytes.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Reads no directory and no configuration, so it works before temper init. A name that is no page, or a search that matches nothing, exits 1. A search prints at most 20 hits, a line each, NAME: HEADING: LINE, then "and more; narrow the words" when more matched.

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

## Internal

### `temper-engine forge fetch`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Download an owned forge's Forgejo and runner binaries into the cache.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** Fetches only the binaries the forge's setup gives as "fetched", at temper's pinned versions, for amd64 or arm64; another machine is refused, naming its architecture. It says how far each binary has gone on its stdout, a line at each percent. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.

### `temper-engine forge prepare`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Prepare an owned forge before its Forgejo first runs.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.

### `temper-engine forge run`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE forgejo|runner`
- Run an owned forge's Forgejo or its runner.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** Checks the binary and the preparation, and takes the forge's lock, first. Run alone, it replaces itself by the program, whose exit status is then its own; given the forge's listener, as the box gives it, it runs the program as its child. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.

### `temper-engine forge sign-in`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Make what people need to sign in to temper through a Forgejo, and print the Forgejo command that registers temper.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** For a Forgejo of application.sign_in.forges, on temper's host, before temper serve. Writes the provider's key and the forge's client secret where they are missing, private, and leaves any already there; prints Forgejo's add-oauth command, never running it.

### `temper-engine forge source`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Add the box’s public temper sign-in source to its owned Forgejo.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** Lists the sources and adds temper when absent, under the setup deadline. Prints source added or source failed WHY and exits 0 either way: a failure keeps the box running with plain links.

### `temper-engine forge remove`

- **Set:** internal
- **Forms:** `[DIRECTORIES] FORGE`
- Remove a stopped box slot's forge directory, keeping the cache.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `FORGE`: A forge's number in the configuration.
- **Notes:** Reads the slot's kind from box/groups/FORGE and refuses a directory outside data or in the cache.

### `temper-engine internal forge-child`

- **Set:** internal
- **Forms:** `--parent PID --signal kill|terminate -- PROGRAM ARGS...`
- Run an owned forge's program, sent SIGNAL when its parent PID ends.
- **Arguments:** `PID`: The process whose end ends the program.; `PROGRAM`: The program to run, with its ARGS.
