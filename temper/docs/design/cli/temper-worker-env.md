# temper-worker: environment

Generated from temper-worker's table (`skein/docs/design/runtime/shell-table.md`, 1.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `TEMPER_JOB`

- **Set:** public
- **Sets**
- **Form:** DIR
- **Absent:** Set when a worker runs a job's scripts.
- The job's directory, set for its scripts.

### `TEMPER_CLOSE_REFUSAL`

- **Set:** public
- **Sets**
- **Form:** 3
- **Absent:** The close command uses its exit statuses from before typed refusals.
- Asks the close command to report a typed refusal with exit status 3.

### `TEMPER_CLOSE_RESULT_BYTES`

- **Set:** public
- **Sets**
- **Form:** BYTES
- **Absent:** The close checker uses 512 bytes, also used for values that are not whole decimals of at least 512.
- Maximum bytes in the close command's typed refusal, including its newline.

### `TEMPER_CLOSE_OUTDATED`

- **Set:** public
- **Sets**
- **Form:** 3
- **Absent:** The close command runs without the outdated ask.
- Asks the close command to report an outdated finding with exit status 3.

### `TEMPER_WORKER_DISK_BUDGET`

- **Set:** public
- **Reads**
- **Form:** RESERVE_BYTES,BYTES_PER_PLACE
- **Absent:** Use the configured places.
- Keep filesystem space for active work before admitting another checkout.
- **Notes:** A standalone worker samples available bytes on its data filesystem. Each occupied place reserves a full allowance.

## Internal
