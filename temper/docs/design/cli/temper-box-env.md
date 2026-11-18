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

### `TEMPER_INTERNAL_SCRIPTED_LISTENER`

- **Set:** internal
- **Reads**
- **Form:** SOCKET
- **Absent:** Choose loopback port 8370 when free, otherwise a kernel-assigned port.
- The scripted model's listener retained by the supervising box.
- **Notes:** The box passes the chosen address before composing its worker and hands the listener to the fake LLM.
