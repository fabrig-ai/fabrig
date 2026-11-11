# temper: environment

Generated from temper's table (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `XDG_STATE_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.local/state when HOME is absolute.
- Base for a setting's state directory, before the tool's name.
- **Notes:** Only absolute paths are used.

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

### `COLORFGBG`

- **Set:** public
- **Reads**
- **Form:** COLOURS
- **Absent:** Use the default terminal background.
- Terminal foreground and background colours.

### `COLORTERM`

- **Set:** public
- **Reads**
- **Form:** NAME
- **Absent:** Use the terminal's indexed colours.
- Terminal true-colour capability.

## Internal

### `TEMPER_INTERNAL_SITE`

- **Set:** internal
- **Reads**
- **Form:** ORIGIN
- **Absent:** Use https://fabrig.ai (port 443).
- Loopback HTTPS origin for a test's temper-box download.
- **Notes:** Only https://127.0.0.1:PORT, https://[::1]:PORT or https://localhost:PORT, with nothing after the port.
