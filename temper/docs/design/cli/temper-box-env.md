# temper-box: environment

Generated from temper-box's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `TEMPER_JOB`

- **Set:** public
- **Sets**
- **Form:** DIR
- **Absent:** Set when a worker runs a job's scripts.
- The job's directory, set for its scripts.

## Internal

### `TEMPER_INTERNAL_FAKE_BOX_PORTS`

- **Set:** internal
- **Reads**
- **Form:** PORTS
- **Absent:** Reserve available ports.
- Choose the fake box's nine listener and peer ports.
- **Notes:** Nine comma-separated nonzero ports; used only for a new sandbox.
