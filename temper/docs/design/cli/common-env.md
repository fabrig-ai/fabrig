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
- **Read by:** temper-box, temper-engine

### `TEMPER_DATA`

- **Set:** public
- **Reads**
- **Form:** DATA
- **Absent:** default $XDG_DATA_HOME/temper
- **Twin of:** `--data`
- The deployment's data and secrets.
- **Read by:** temper-box, temper-engine

### `TEMPER_CACHE`

- **Set:** public
- **Reads**
- **Form:** CACHE
- **Absent:** default $XDG_CACHE_HOME/temper
- **Twin of:** `--cache`
- Binaries that can be fetched again.
- **Read by:** temper-box, temper-engine

### `PATH`

- **Set:** public
- **Reads**
- **Form:** DIRS
- **Absent:** Optional executables are not discovered.
- Search path for optional executables and child programs.
- **Read by:** temper-box, temper-engine

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
- **Read by:** temper-box, temper-engine

### `SSL_CERT_FILE`

- **Set:** public
- **Reads**
- **Form:** FILE
- **Absent:** Use the machine's certificate trust.
- Certificate trust file for HTTPS connections.
- **Read by:** temper-box, temper-engine

### `XDG_RUNTIME_DIR`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use the deployment data directory's link file.
- Runtime directory for the engine link file.
- **Read by:** temper-box, temper-engine

### `HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** A directory setting needs an absolute XDG base or an explicit value.
- Base for a setting's directory when its XDG base is unusable.
- **Notes:** Only absolute paths are used.
- **Read by:** temper-box, temper-engine

### `XDG_CONFIG_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.config when HOME is absolute.
- Base for a setting's configuration directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** temper-box, temper-engine

### `XDG_DATA_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.local/share when HOME is absolute.
- Base for a setting's data directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** temper-box, temper-engine

### `XDG_CACHE_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.cache when HOME is absolute.
- Base for a setting's cache directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** temper-box, temper-engine

## Internal

### `TEMPER_INTERNAL_WAIT_CHECK`

- **Set:** internal
- **Reads**
- **Form:** SWITCH
- **Absent:** Check in debug builds only.
- Check the service wait boundary.
- **Read by:** temper-box, temper-engine

### `TEMPER_INTERNAL_CUT_AT_LISTEN`

- **Set:** internal
- **Reads**
- **Form:** FILE
- **Absent:** Do not cut the start.
- Abort once at the client listener, creating this marker.
- **Notes:** Requires TEMPER_INTERNAL_WAIT_CHECK. An existing marker keeps the next start running.
- **Read by:** temper-box, temper-engine

### `TEMPER_INTERNAL_FORGE_NO_REGISTRATION`

- **Set:** internal
- **Reads**
- **Form:** SWITCH
- **Absent:** Register the first account.
- Use links only instead of registering a first account.
- **Read by:** temper-box, temper-engine
