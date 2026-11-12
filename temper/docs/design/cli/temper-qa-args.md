# temper-qa: arguments

Generated from temper-qa's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

## Internal

### `temper-qa run`

- **Set:** internal
- **Forms:** ``
- Watch main and triage its landings, one service per QA home.
- **Notes:** Holds run.lock for its life; reads qa.json once at start.

### `temper-qa triage none`

- **Set:** internal
- **Forms:** `WHY`
- Answer no QA for the session's landing, with a line saying why.
- **Arguments:** `WHY`: One nonempty argument saying why no QA is needed.

### `temper-qa triage case`

- **Set:** internal
- **Forms:** `FILE`
- Queue a checked case, or join the queued case for its surface and page.
- **Arguments:** `FILE`: A case JSON file, or - to read stdin.

### `temper-qa show`

- **Set:** internal
- **Forms:** ``
- Print the ledger's landings, cases, builds and sessions, oldest first.

### `temper-qa lines`

- **Set:** internal
- **Forms:** ``
- Print the ledger's lines, oldest first.
