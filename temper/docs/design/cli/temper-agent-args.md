# temper-agent: arguments

Generated from temper-agent's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `temper-agent help`

- **Set:** public
- **Forms:** `[NAME | pages | --search WORDS...]`
- Print this, a page, the pages, or a search.
- **Arguments:** `NAME`: A page's name, as the index and the pages give it.; `WORDS`: Words that a section must hold every one of, case folded: at most 8, each at most 32 bytes.
- **Notes:** Reads no directory and no configuration, so it works before temper init. A name that is no page, or a search that matches nothing, exits 1. A search prints at most 20 hits, a line each, NAME: HEADING: LINE, then "and more; narrow the words" when more matched.

## Internal

### `temper-agent`

- **Set:** internal
- **Forms:** `AGENT.json`
- Run one agent for a worker's run, on smith's agent configuration.
- **Arguments:** `AGENT.json`: The agent's configuration, as the worker's agent.configuration names it.

### `temper-agent --list`

- **Set:** internal
- **Forms:** `AGENT.json`
- Read one endpoint's model list, on smith's agent configuration.
- **Arguments:** `AGENT.json`: The agent's configuration, as the worker's agent.configuration names it.

### `temper-agent --mcp-relay`

- **Set:** internal
- **Forms:** `127.0.0.1:PORT`
- Relay a program's MCP connection to its agent.
- **Arguments:** `PORT`: The agent's relay listener on loopback; the secret is in SMITH_MCP_SECRET.

### `temper-agent --program-child`

- **Set:** internal
- **Forms:** `PARENT PROGRAM ARGUMENTS...`
- Run a program as the agent's child, which ends with its parent.
- **Arguments:** `PARENT`: The agent's process id.; `PROGRAM`: The program's path, with its ARGUMENTS.

### run as `claude`

- **Set:** internal
- **Forms:** `ARGS...`
- Run the Claude Code launcher with its delegated arguments.
