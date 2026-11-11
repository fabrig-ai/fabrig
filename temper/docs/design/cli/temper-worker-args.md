# temper-worker: arguments

Generated from temper-worker's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `temper-worker serve`

- **Set:** public
- **Forms:** `[DIRECTORIES]`
- Run a worker alone's document until stopped; temper-engine serve runs an engine alone, and the box's temper-box serve runs an engine and its worker in one process.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Its stderr carries its startup notes, its linked and lost lines and the reason of a failure. A first termination signal runs its worker's ending; a second aborts. Exit 0 after its ending, 1 on a refused start or any failure. Refuses an engine alone's document, naming temper-engine serve, or an engine and its worker in one process, naming the box's temper-box serve.

### `temper-worker program-login`

- **Set:** public
- **Forms:** `[DIRECTORIES] NAME`
- Sign a program source in on this host, through the program's own sign-in.
- **Arguments:** `DIRECTORIES`: [--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.; `NAME`: The program source's name: 1 to 32 of a to z, 0 to 9 and -.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Checks the configuration and the agent's, makes the source's directory, then replaces itself by the program's sign-in, whose exit status is then its own. The engine need not run.

### `temper-worker help`

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
