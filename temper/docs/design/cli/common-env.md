# temper: common environment

Generated from the tree's tables (`skein/docs/design/runtime/shell.md`, 14.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `TEMPER_CONFIG`

- **Set:** public
- **Reads**
- **Form:** CONFIG
- **Absent:** default $XDG_CONFIG_HOME/temper
- **Twin of:** `--config`
- Configuration, including temper.json.
- **Read by:** temper-box, temper-engine, temper-worker, temper

### `TEMPER_DATA`

- **Set:** public
- **Reads**
- **Form:** DATA
- **Absent:** default $XDG_DATA_HOME/temper
- **Twin of:** `--data`
- The deployment's data and secrets.
- **Read by:** temper-box, temper-engine, temper-worker, temper

### `TEMPER_CACHE`

- **Set:** public
- **Reads**
- **Form:** CACHE
- **Absent:** default $XDG_CACHE_HOME/temper
- **Twin of:** `--cache`
- Binaries that can be fetched again.
- **Read by:** temper-box, temper-engine, temper-worker, temper

### `PATH`

- **Set:** public
- **Reads**
- **Form:** DIRS
- **Absent:** Optional executables are not discovered.
- Search path for optional executables and child programs.
- **Read by:** temper-box, temper-engine, temper

### `TERM`

- **Set:** public
- **Reads**
- **Form:** NAME
- **Absent:** Use terminal output without terminal-specific links.
- Terminal type for links and terminal output.
- **Read by:** temper-box, temper-engine

### `LISTEN_FDS`

- **Set:** public
- **Reads**
- **Form:** N
- **Absent:** Open the configured listeners.
- Inherited listeners beginning at file descriptor 3.
- **Read by:** temper-box, temper-engine, temper-worker

### `SSL_CERT_FILE`

- **Set:** public
- **Reads**
- **Form:** FILE
- **Absent:** Use the machine's certificate trust.
- Certificate trust file for HTTPS connections.
- **Read by:** temper-box, temper-engine, temper-agent

### `XDG_RUNTIME_DIR`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use the deployment data directory's link file.
- Runtime directory for the engine link file.
- **Read by:** temper-box, temper-engine, temper-worker, temper

### `COLORTERM`

- **Set:** public
- **Reads**
- **Form:** NAME
- **Absent:** Use the terminal's indexed colours.
- Terminal true-colour capability.
- **Read by:** temper-box, temper

### `COLORFGBG`

- **Set:** public
- **Reads**
- **Form:** COLOURS
- **Absent:** Use the default terminal background.
- Terminal foreground and background colours.
- **Read by:** temper-box, temper

### `DISPLAY`

- **Set:** public
- **Reads**
- **Form:** ADDRESS
- **Absent:** On Linux, use WAYLAND_DISPLAY to identify a graphical session.
- The X display, when opening a browser on this host.
- **Read by:** temper-box, temper

### `WAYLAND_DISPLAY`

- **Set:** public
- **Reads**
- **Form:** NAME
- **Absent:** On Linux, use DISPLAY to identify a graphical session.
- The Wayland display, when opening a browser on this host.
- **Read by:** temper-box, temper

### `SSH_CONNECTION`

- **Set:** public
- **Reads**
- **Form:** CONNECTION
- **Absent:** Check SSH_TTY and the display variables.
- An SSH session prevents opening a browser on the remote host.
- **Read by:** temper-box, temper

### `SSH_TTY`

- **Set:** public
- **Reads**
- **Form:** TTY
- **Absent:** Check SSH_CONNECTION and the display variables.
- An SSH terminal prevents opening a browser on the remote host.
- **Read by:** temper-box, temper

### `HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** A directory setting needs an absolute XDG base or an explicit value.
- Base for a setting's directory when its XDG base is unusable.
- **Notes:** Only absolute paths are used.
- **Read by:** temper-box, temper-engine, temper-worker, temper

### `XDG_STATE_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.local/state when HOME is absolute.
- Base for a setting's state directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** temper-box, temper

### `XDG_CONFIG_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.config when HOME is absolute.
- Base for a setting's configuration directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** temper-box, temper-engine, temper-worker, temper

### `XDG_DATA_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.local/share when HOME is absolute.
- Base for a setting's data directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** temper-box, temper-engine, temper-worker, temper

### `XDG_CACHE_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.cache when HOME is absolute.
- Base for a setting's cache directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** temper-box, temper-engine, temper-worker, temper

## Internal

### `TEMPER_INTERNAL_WAIT_CHECK`

- **Set:** internal
- **Reads**
- **Form:** SWITCH
- **Absent:** Check in debug builds only.
- Check the service wait boundary.
- **Read by:** temper-box, temper-engine, temper-worker

### `TEMPER_INTERNAL_CUT_AT_LISTEN`

- **Set:** internal
- **Reads**
- **Form:** FILE
- **Absent:** Do not cut the start.
- Abort once at the client listener, creating this marker.
- **Notes:** Requires TEMPER_INTERNAL_WAIT_CHECK. An existing marker keeps the next start running.
- **Read by:** temper-box, temper-engine, temper-worker

### `TEMPER_INTERNAL_FORGE_NO_REGISTRATION`

- **Set:** internal
- **Reads**
- **Form:** SWITCH
- **Absent:** Register the first account.
- Use links only instead of registering a first account.
- **Read by:** temper-box, temper-engine

### `TEMPER_INTERNAL_COMMIT`

- **Set:** internal
- **Reads**
- **Form:** COMMIT
- **Absent:** The build script supplies unknown when git identity is unavailable.
- The build's commit and tracked-file dirty mark.
- **Notes:** Read at compile time from temper-version's build script alone; not a runtime override.
- **Read by:** temper-box, temper-engine, temper-worker, temper-agent, temper

### `TEMPER_INTERNAL_BUNDLE`

- **Set:** internal
- **Reads**
- **Form:** embed|none
- **Absent:** Embed the browser's bundle.
- Select whether the build embeds the browser's bundle.
- **Notes:** Read by temper-client-bundle's build script; none skips the nested browser build. Not a runtime override.
- **Read by:** temper-box, temper-engine

### `TEMPER_INTERNAL_DUMP_LLM`

- **Set:** internal
- **Reads**
- **Form:** DIR
- **Absent:** Do not dump model requests.
- Directory for agents' model request dumps.
- **Notes:** The box's worker reads it and sets it for its agents. An empty value disables dumping; a relative path is taken from the working directory.
- **Read by:** temper-box, temper-worker, temper-agent

### `TEMPER_INTERNAL_WORKSPACE_REUSE`

- **Set:** internal
- **Reads**
- **Form:** ANY
- **Absent:** Start with fresh worker workspaces.
- Reuse worker workspaces across restarts.
- **Notes:** Any nonempty value enables reuse; an empty value leaves it off. Never a shipped default.
- **Read by:** temper-box, temper-worker
