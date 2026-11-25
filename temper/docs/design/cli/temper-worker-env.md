# temper-worker: environment

Generated from temper-worker's table (`skein/docs/design/runtime/shell-table.md`, 1.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `TEMPER_JOB`

- **Set:** public
- **Sets**
- **Form:** DIR
- **Absent:** Set when a worker runs a job's scripts.
- The job's directory, set for its scripts.

### `TEMPER_WORKER_DISK_BUDGET`

- **Set:** public
- **Reads**
- **Form:** RESERVE_BYTES,BYTES_PER_PLACE
- **Absent:** Use the configured places.
- Keep filesystem space for active work before admitting another checkout.
- **Notes:** A standalone worker samples available bytes on its data filesystem. Each occupied place reserves a full allowance.

## Internal
