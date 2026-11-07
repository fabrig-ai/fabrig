+++
id = "screen-keys"
part = "screen"
kind = "feature"
needs = ["screen-surface"]
files = [
  "skein/crates/skein-ui-screen-terminal/**",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 130
+++

# skein-ui-screen-terminal: keys, the key reader and the drawing

## Goal

The full-screen shell's terminal crate, with its pure parts: the decoder
that turns the input's bytes into keys, the key reader that turns a key
into an act on a node, a move of the focus or closing, and the drawing
that turns a frame into the bytes that take the screen to it. The io
terminal is `screen-terminal`'s; an application's host can already read
keys and lay frames with this task and `screen-render`.

`skein/docs/design/ui-screen.status.md` is not yours: tasks beside this
one run in parallel, and `screen-status` closes its entries once all have
merged.

Acceptance:

1. `skein/crates/skein-ui-screen-terminal` exists as a member of the root
   `Cargo.toml` and an entry of its `[workspace.dependencies]`, `no_std`
   with `alloc`, `#![forbid(unsafe_code)]`, `[lints] workspace = true` as
   `skein-ui-text`, depending on `skein-ui-screen`, `skein-ui`, `skein-lib`
   and `skein-io` (io for `screen-terminal`; add it now so the manifest
   doesn't move twice).
2. It exports `Key`, `Decoder`, `Command`, `command`, `Depth`, `ENTER`,
   `LEAVE`, `draw` and `draw_bound`, with the names and signatures of
   `temper/docs/plan/tui-connected/notes/contracts.md`, section 3. It does
   not yet export `Settings` or `Terminal`.
3. **The decoder** does what `skein/docs/design/ui-screen.md`, 6.1, says,
   every row of its table, split across reads, bounded, a paste's bytes
   kept up to the most at a character's boundary.
4. **The key reader** does what 6.2 says, every row of its table: the
   focus order over bound, enabled nodes of the roles a person acts on
   (`ui-text.md`, section 6's numbered roles), wrapping at both ends; the
   caret moved by a whole character; an `Input`'s value built from the
   tree's value and the caret, refused past the limits' text of a node;
   shortcuts as `ui.md`, 6.1, says (a plain one never while a text field
   or area has the focus; Escape in a dialog, then the innermost form
   holding the focus); Ctrl+C closing; the body's scroll by its rows less
   one, which the key reader learns from the focus's last frame (option
   below).
5. **The drawing** does what section 7 says: a first frame (or one of
   another size) cleared and written whole, a changed frame by its changed
   runs, paints changed only where they change, colours at both depths,
   the cursor last; `draw_bound(most)` the most bytes any `draw` writes
   for frames of `most`, `None` past a u32. `ENTER` and `LEAVE` are the
   bytes of section 8 exactly.
6. Step tests in `src/tests/` (tables): the decoder's table, each
   sequence whole and split at every byte, a lone `ESC` at a read's end
   and one followed in the next read, an unknown sequence dropped, a
   sequence of 17 bytes dropped, a paste past its most cut at a character;
   the key reader's table, each row of 6.2 on its kind of focus, a disabled
   node skipped, a shortcut while a field has the focus not pressed, Escape
   in a dialog and in a form, an `Input` at its bound not made; the
   drawing's rows of acceptance 5, the nearest indexed colours of a few
   colours (pure red, a gray, the palette's `#e08a68`), and `draw_bound`
   held by a frame of `most`'s cells, each a different paint from the last
   frame's.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, sections 1 to 3:
  they bind.
- `skein/docs/design/ui-screen.md`, sections 6, 7 and 8 (`ENTER`,
  `LEAVE`), and 5.4 (the focus).
- `skein/docs/design/ui.md`, section 6 (the acts each role takes) and 6.1
  (shortcuts, Escape).
- `skein/crates/skein-ui-text/src/command.rs`: the text shell's command
  reader, the sibling of this key reader (its acts per role, its `Input`
  then `Submit`).

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:** none; nothing names the crate yet.

**Options, each with its default:**

- The body's rows for `PageUp` and `PageDown`: the key reader takes no
  frame, so the default is a scroll by a fixed 10 rows less one, named
  `PAGE` with a comment citing 6.2; if a `rows` field on `Focus` would be
  needed, that is `skein-ui-screen`'s surface: don't add it, keep the
  constant, and say so in the report.
- A paste into a text field: line breaks become spaces and trailing ones
  are dropped (6.2); `\r\n` is one break.
- The drawing's cursor moves: absolute `ESC [ ROW ; COLUMN H` for each run
  is the default; relative moves only if a test shows they are needed.
- Reset of paints between runs: `ESC [ 0 m` then the run's paint, the
  default; incremental attribute changes only if you keep `draw_bound`
  exact.

## Steps

1. The crate, its `lib.rs` citing ui-screen.md at module level, the
   modules `keys`, `command` and `draw`.
2. The decoder: a small state machine over bytes; its partial sequence
   bounded at 16 bytes; a paste collected up to its most.
3. The key reader: the focus order, the role's act, the field's edits as
   whole values, the shortcuts and Escape, closing.
4. The drawing and its bound.
5. The tests (acceptance 6). Bite: drop `BackTab` from the decoder once,
   let a plain shortcut press while a field has the focus once, and draw
   an unchanged cell once; see their rows fail, and restore.
6. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-ui-screen-terminal --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-ui-screen-terminal --lib`. Commit
   ("skein-ui-screen-terminal: keys decoded and read as acts, frames drawn
   as bytes"), push.

## Report

As the implementer's text says, and: how `PageUp` learns its rows, the
cursor moves chosen, and the tests' time.

## Answers

### 2026-11-07: Resume after screen-surface pushes skein-ui-screen (recommended), or supply another approved dependency commit? The required crate is absent, so this task cannot compile or verify its acceptance. (tui-connected/screen-keys-1, foreman tui-connected/screen-keys/foreman-1)

Resume after screen-surface supplies skein-ui-screen.
The spec declares `needs = ["screen-surface"]`.
Acceptance 1 requires “depending on `skein-ui-screen`, `skein-ui`, `skein-lib` and `skein-io`.”
The next run merges the completed dependency, repairs against its binding contracts, and completes the required checks and mutation bites before committing and pushing.
The partial implementation is not acceptance-complete.
