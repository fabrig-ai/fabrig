# temper-qa: arguments

Generated from temper-qa's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

## Internal

### `temper-qa run`

- **Set:** internal
- **Forms:** ``
- Watch main and triage its landings, one service per QA home.
- **Notes:** Holds run.lock for its life; reads qa.json once at start.

### `temper-qa build keep`

- **Set:** internal
- **Forms:** `COMMIT DIR`
- Every executable must print that commit as its version, the files are moved into the builds directory under the commit, a manifest names the commit and each file's checksum, and the ledger records the build.
- **Arguments:** `COMMIT`: The landed tip, 40 or 64 lowercase hexadecimal digits.; `DIR`: The staged build, on the builds directory's filesystem.

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

### `temper-qa pass setup`

- **Set:** internal
- **Forms:** `NAME`
- Restore a pass setup at its place, or make it again from the nearest restoring copy.
- **Arguments:** `NAME`: fresh, or a recipe of the pass's setups.
- **Notes:** The drive session must be closed. Walks use the place's build with --strict. Exit 4: the box failed; 5: stopped at a recipe line; 1: refused. Cache stays across restores.

### `temper-qa pass recipe`

- **Set:** internal
- **Forms:** `NAME FILE`
- Write a recipe in the pass's setups and prove it by walking from its parent.
- **Arguments:** `NAME`: A recipe name not among the build's setups; a pass-added recipe may be replaced.; `FILE`: A JSON object with after, lines, readme and optional scripts; - reads stdin.
- **Notes:** The drive session must be closed. A failed proof removes the recipe. Exits are pass setup's.

### `temper-qa show`

- **Set:** internal
- **Forms:** ``
- Print the ledger's landings, cases, builds, sessions, passes, findings and notes, oldest first.

### `temper-qa lines`

- **Set:** internal
- **Forms:** ``
- Print the ledger's lines, oldest first.

### `temper-qa report`

- **Set:** internal
- **Forms:** `PASS`
- Print a pass's JSON report from the ledger.
- **Arguments:** `PASS`: The pass number.
