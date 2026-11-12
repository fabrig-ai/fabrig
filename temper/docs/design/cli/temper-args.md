# temper: arguments

Generated from temper's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `temper`

- **Set:** public
- **Forms:** `[ADDRESS]`
- Run temper's full-screen terminal client, the TUI, on an engine; temper help tui explains closing and signing out.
- **Arguments:** `ADDRESS`: Engine listener, http://HOST:PORT on loopback; defaults to the engine link file's address for this environment, else the TUI's last sign-in address, else http://127.0.0.1:8367.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Needs a terminal on stdin and stdout; without one, bare temper says it has no command. Its sign-in link is pasted on its screen, never given. Its directory is $XDG_STATE_HOME/temper/tui, else ~/.local/state/temper/tui. `/exit` or Ctrl+C closes it without signing out; on the sign-in screen, use Ctrl+C. `q` types a character on every screen.

### `temper box`

- **Set:** public
- **Forms:** `[ARGUMENT...]`
- Run the TUI in front of this build's temper-box, fetching it if needed.
- **Arguments:** `ARGUMENT...`: Arguments for temper-box, passed on unchanged.
- **Flags:** `--config CONFIG`, `--data DATA`, `--cache CACHE`
- **Notes:** Finds temper-box beside this executable, then in the cache, then at the download site. A checkout build downloads none: build it with cargo build -p temper-box. In a terminal, runs the TUI in front of the box. Without a terminal, or with --echo steps, runs the box in this process's place.

### `temper help`

- **Set:** public
- **Forms:** `[NAME | pages | --search WORDS...]`
- Print this, a page, the pages, or a search.
- **Arguments:** `NAME`: A page's name, as the index and the pages give it.; `WORDS`: Words that a section must hold every one of, case folded: at most 8, each at most 32 bytes.
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

### `temper ctl`

- **Set:** internal
- **Forms:** `DIR GROUP VERB [ARGUMENT...] [--json]`, `DIR me|inbox|sign-in|sign-out|pending [ARGUMENT...] [--json]`, `DIR --call FILE [--json]`, `help [GROUP [VERB]]`
- Act in a deployment as one party, one call a process: the agents' command line.
- **Arguments:** `DIR`: ctl's private directory for one deployment and one party: its sign-in, its kept requests and the engine's address.; `GROUP`: A group of tools, a noun of temper's or one of the client's own names; temper ctl help lists them.; `VERB`: One of the group's tools; temper ctl help GROUP lists them.; `FILE`: One call as JSON, {"tool": "GROUP VERB", "arguments": {...}}, or - for stdin.
- **Notes:** temper ctl help prints how to work through it, then the groups. A call holds one of the engine's streams while it runs, a wait for its whole span; past the engine's bound a call says so.

### `temper internal fetch-box`

- **Set:** internal
- **Forms:** `[--cache DIR]`
- Find this build's temper-box, fetching it if needed, for temper box.
- **Notes:** Writes progress lines, then found PATH on stdout; failures on stderr, exit 1. Reads no terminal.
