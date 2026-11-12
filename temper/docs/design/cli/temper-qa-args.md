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

### `temper-qa pass outcome`

- **Set:** internal
- **Forms:** `CASE RESULT NOTES [--finding N]...`
- Record a case's outcome at once; the session's last outcome is the one its end reads.
- **Arguments:** `CASE`: A case assigned to the session's pass.; `RESULT`: passed, failed or blocked.; `NOTES`: One argument, nonempty for blocked, saying why.; `[--finding N]...`: A finding of this case; repeat for each finding. A failed case names at least one.

### `temper-qa pass finding`

- **Set:** internal
- **Forms:** `FILE`
- Record a finding, copying its evidence and linking it to a matching open finding.
- **Arguments:** `FILE`: A finding JSON file, or - to read stdin.
- **Notes:** Evidence: at most 16 regular files, each at most 16 MiB, with distinct file names. An absent area takes the surface's configured area, else owner.

### `temper-qa pass check`

- **Set:** internal
- **Forms:** `FINDING RESULT NOTES`
- Check an open finding of the pass in a resumed session.
- **Arguments:** `FINDING`: An open finding of the session's pass.; `RESULT`: holds or gone.; `NOTES`: One argument with the check's notes, possibly empty.

### `temper-qa pass note`

- **Set:** internal
- **Forms:** `WORDS`
- Record what the pass noticed off the green path.
- **Arguments:** `WORDS`: One nonempty argument.

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
