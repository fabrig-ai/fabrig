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

### `temper-qa pass open`

- **Set:** internal
- **Forms:** ``
- Start a holder for the restored setup and print the drive session's start.
- **Notes:** Requires setup and no live holder. Exit 0: ready; 3: store refused; 4: box failed or start exceeded 300 seconds; 1: refused.

### `temper-qa pass line`

- **Set:** internal
- **Forms:** `LINE`
- Give the open drive session one line and print its answer.
- **Arguments:** `LINE`: One drive line, as one argument.
- **Notes:** Prints the answer unchanged. Exit 0: ok; 5: failed line; 4: the session ended with the box's failure; 1: no holder.

### `temper-qa pass close`

- **Set:** internal
- **Forms:** ``
- End the drive session and print its kept lines.
- **Notes:** Waits for the holder's exit; prints the session's end and kept lines: followed by box/session/lines. Exit 0: clean stop; 4: box failed; 1: no holder.

### `temper-qa pass hold`

- **Set:** internal
- **Forms:** ``
- Hold the drive session on the place's socket until closed or the pass session ends.
- **Notes:** Owns drive.lock for its life and drive.sock at mode 0600. Watches the ledger every five seconds. Gives end, then kills only the drive session's process group past 70 seconds.

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

### `temper-qa tune`

- **Set:** internal
- **Forms:** ``
- Print figures and suggestions for tuning bounds and places, without recording anything.
- **Notes:** Session times and tokens, waits for builds, and waits for places in the last 7 days. See testing/qa/ledger.md, 6.
