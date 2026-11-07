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
- **`tui`** (`temper/docs/design/client/README.md`, 6.11 to 6.16, and
  `notes/tui.md`): `temper-client-tui` (the TUI's state, words, view and
  theme) and `temper-client-shell-tui` (its host and its loop); bare
  `temper` and `temper ADDRESS`; the client world's stories.
  `temper-client-domain` and `temper-client-view` do not change: the TUI
  reads the root's `me` and link and says the view's words through their
  public items.

## Tasks so far

| Task | Part | Needs | What |
|---|---|---|---|
| `fail-stop-restore` | screen | | the panic's restoration: skein-fail-stop's `before_report`, skein-shell's `Kept::on_abort` |
| `sim-window` | screen | | the simulated machine's window and window change |
| `screen-surface` | screen | | `skein-ui-screen`'s theme, frame, print, focus and width |
| `screen-render` | screen | screen-surface | the renderer: bands, framed regions, spread rows, the body's scroll, fields and the focus, the first screens' roles |
| `screen-keys` | screen | screen-surface | `skein-ui-screen-terminal`: the key decoder, the key reader, the drawing, `ENTER` and `LEAVE` |
| `screen-terminal` | screen | screen-keys, sim-window | the terminal on io, and `tests/ui-screen` |
| `screen-roles` | screen | screen-render | every other role, as the text shell says it |
| `screen-status` | screen | every screen task | skein's statuses closed from the trunk's code |
| `tui-view` | tui | screen-surface | `temper-client-tui`: the TUI's state, words, two screens, themes and limits |

## Critical path, as it stands

`screen-surface` → `screen-render` and `screen-keys`, side by side → the
TUI's host → its joint-world stories, and `screen-terminal` → the TUI's
loop → the command. The temper batch names its tasks.

## Overlaps

- `temper/docs/design/client/README.md` and `README.status.md`: this plan
  writes the TUI's sections there; look-round-17 (`terminal-redraw`) edits
  `README.status.md`, and tui-approval will write the approval's part.
  Each adds its own lines.
- `temper/docs/design/draft/tui.status.md` and `tui.execution.md`: this plan
  closes M1's lags and sets M1's state; tui-approval closes M2's.
- No code file of another draft's list is touched (`notes/common.md`).
- Inside this plan: `fail-stop-restore` and `screen-surface`, released
  together, each edit one line of `skein/docs/design/ui-screen.status.md`,
  so the second to merge meets the first's edit there. Every later task
  leaves the statuses to `screen-status`, so no other two tasks share a
  line.
- `screen-surface` asked whether `Frame::new` can refuse a `most` whose
  cells pass a u32 (two u16 sides never do): the design says a u32
  always counts them, and `Frame::new` returns a `Frame`, never refusing
  (the answer call, 2026-11-07); `screen-terminal` makes its frames so.
