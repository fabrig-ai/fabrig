# temper-box: environment

Generated from temper-box's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `TEMPER_JOB`

- **Set:** public
- **Sets**
- **Form:** DIR
- **Absent:** Set when a worker runs a job's scripts.
- The job's directory, set for its scripts.

### `COLORTERM`

- **Set:** public
- **Reads**
- **Form:** NAME
- **Absent:** Use the terminal's indexed colours.
- Terminal true-colour capability.

### `COLORFGBG`

- **Set:** public
- **Reads**
- **Form:** COLOURS
- **Absent:** Use the default terminal background.
- Terminal foreground and background colours.

### `DISPLAY`

- **Set:** public
- **Reads**
- **Form:** ADDRESS
- **Absent:** On Linux, use WAYLAND_DISPLAY to identify a graphical session.
- The X display, when opening a browser on this host.

### `WAYLAND_DISPLAY`

- **Set:** public
- **Reads**
- **Form:** NAME
- **Absent:** On Linux, use DISPLAY to identify a graphical session.
- The Wayland display, when opening a browser on this host.

### `SSH_CONNECTION`

- **Set:** public
- **Reads**
- **Form:** CONNECTION
- **Absent:** Check SSH_TTY and the display variables.
- An SSH session prevents opening a browser on the remote host.

### `SSH_TTY`

- **Set:** public
- **Reads**
- **Form:** TTY
- **Absent:** Check SSH_CONNECTION and the display variables.
- An SSH terminal prevents opening a browser on the remote host.

### `XDG_STATE_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.local/state when HOME is absolute.
- Base for a setting's state directory, before the tool's name.
- **Notes:** Only absolute paths are used.

## Internal

### `TEMPER_INTERNAL_FAKE_BOX_PORTS`

- **Set:** internal
- **Reads**
- **Form:** PORTS
- **Absent:** Reserve available ports.
- Choose the fake box's nine listener and peer ports.
- **Notes:** Nine comma-separated nonzero ports; used only for a new sandbox.

### `TEMPER_INTERNAL_RECORD`

- **Set:** internal
- **Reads**
- **Form:** VALUE
- **Absent:** Record nothing.
- Record the TUI into its directory.
- **Notes:** Any nonempty value enables recording; an empty value is unset.
