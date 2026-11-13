# temper-redeploy: arguments

Generated from temper-redeploy's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

## Internal

### `temper-redeploy stage`

- **Set:** internal
- **Forms:** `DESCRIPTION COMMIT`
- Stage each part's binaries without stopping or swapping anything.
- **Arguments:** `DESCRIPTION`: The deployment's JSON description.; `COMMIT`: Twelve lowercase hexadecimal digits naming the build.
- **Notes:** A staged build answering COMMIT is left as it is.

### `temper-redeploy run`

- **Set:** internal
- **Forms:** `DESCRIPTION COMMIT --record FILE [--workers | --together]`
- Stage, snapshot, redeploy and referee, writing a record throughout.
- **Arguments:** `DESCRIPTION`: The deployment's JSON description.; `COMMIT`: Twelve lowercase hexadecimal digits naming the build.
- **Flags:** `--record FILE`, `--workers`, `--together`
- **Notes:** The engine alone by default. Exit 0 only when done and the referee passes; otherwise 1.

### `temper-redeploy referee`

- **Set:** internal
- **Forms:** `DESCRIPTION RECORD`
- Read a record and referee against fresh tasks, without writing the record.
- **Arguments:** `DESCRIPTION`: The deployment's JSON description.; `RECORD`: A redeploy's JSON record.
- **Notes:** Exit 0 on pass, 1 on fail.

### `--record FILE`

- **Set:** internal
- **Required.**
- The new redeploy's JSON record; refused if it exists.
- **Notes:** One record per redeploy, never overwritten.

### `--workers`

- **Set:** internal
- Restart workers in turn after the engine.
- **Notes:** Excludes --together.

### `--together`

- **Set:** internal
- Restart every part together for a breaking change.
- **Notes:** Excludes --workers.
