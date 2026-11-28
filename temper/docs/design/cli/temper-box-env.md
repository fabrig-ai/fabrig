# temper-box: environment

Generated from temper-box's table (`skein/docs/design/runtime/shell-table.md`, 1.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

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

### `TEMPER_CLOSE_OUTDATED`

- **Set:** public
- **Sets**
- **Form:** 3
- **Absent:** The close command runs without the outdated ask.
- Asks the close command to report an outdated finding with exit status 3.

### `TEMPER_CHECKPOINT`

- **Set:** public
- **Sets**
- **Form:** DIRECTORY
- **Absent:** Set for every held command; an inherited value is replaced.
- The held checkpoint's absolute directory, set for checkpoint --hold's child.
- **Notes:** The checkpoint lock stays held until the child is closed.

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
