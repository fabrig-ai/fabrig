# tui-connected: the plan

The TUI's M1 (`temper/docs/design/draft/tui.execution.md`, M1): skein's
full-screen shell, and temper's TUI on it, signed in by the box's pasted
link and drawing its first screen live. It carries the priority mark.

## Interfaces of skein's that change

- **skein-fail-stop:** gains `Restore` and `before_report`, a restoration
  its hook runs before the report (`fail-stop-restore`).
- **skein-shell:** `Kept` gains `on_abort` (`fail-stop-restore`).
- **skein-sim:** `Sim` gains `set_window` and `deliver_window_changed`
  (`sim-window`).

Nothing in skein's `heap`, `io`, `json`, `lib`, `scratch` or `world`
changes.

## Parts

- **`screen`** (`skein/docs/design/ui-screen.md`): two new crates,
  `skein-ui-screen` (the theme, the frame, the focus, the renderer: step
  code) and `skein-ui-screen-terminal` (the key decoder, the key reader,
  the drawing: step code; the terminal: ordinary Rust on io), and the test
  crate `tests/ui-screen`; the panic's restoration in skein-fail-stop and
  skein-shell; the simulated window.
- **`tui`** (`temper/docs/design/client/README.md`, 6.11 to 6.16; written
  in a later batch): `temper-client-tui` (the TUI's state, view and theme)
  and `temper-client-shell-tui` (its host and its loop); bare `temper` and
  `temper ADDRESS`; the client world's stories.

## Tasks so far

| Task | Part | Needs | What |
|---|---|---|---|
| `fail-stop-restore` | screen | | the panic's restoration: skein-fail-stop's `before_report`, skein-shell's `Kept::on_abort` |
| `sim-window` | screen | | the simulated machine's window and window change |
| `screen-surface` | screen | | `skein-ui-screen`'s theme, frame, print, focus and width |

## Critical path, as it stands

`screen-surface` → the renderer and the key reader, side by side → the
TUI's host → its joint-world stories and its loop → the command. The
later batches name them.

## Overlaps

- `temper/docs/design/client/README.md` and `README.status.md`: this plan
  writes the TUI's sections there; look-round-17 (`terminal-redraw`) edits
  `README.status.md`, and tui-approval will write the approval's part.
  Each adds its own lines.
- `temper/docs/design/draft/tui.status.md` and `tui.execution.md`: this plan
  closes M1's lags and sets M1's state; tui-approval closes M2's.
- No code file of another draft's list is touched (`notes/common.md`).
