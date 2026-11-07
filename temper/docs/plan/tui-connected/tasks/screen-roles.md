+++
id = "screen-roles"
part = "screen"
kind = "feature"
needs = ["screen-render"]
files = [
  "skein/crates/skein-ui-screen/**",
]
effort = "high"
minutes = 120
+++

# skein-ui-screen: every role laid out

## Goal

A shell maps every role, state and property (`skein/docs/design/ui.md`,
section 16, *Mapped whole*). `screen-render` lays the roles of an
application's first screens and lays every other as a group; this task
lays each of the others as `skein/docs/design/ui-screen.md`, 5.3, says:
with the text shell's words and marks, in cells.

`skein/docs/design/ui-screen.status.md` is not yours: tasks beside this
one run in parallel, and `screen-status` closes its entries once all have
merged.

Acceptance:

1. `skein_ui_screen::render` lays each role `screen-render` laid as a
   group as the text shell prints it (`skein/docs/design/ui-text.md`,
   section 5's table), with 5.3's differences: disclosure (unfolded),
   table, row, cell, column header, row header, term, description, tree
   and tree item, menu and menu item, choice group and choice, select and
   option, check, tabs and tab, log (whole, oldest first, without the text
   shell's `(up, down: more lines)`), progress, code, quote, figure,
   dialog, drawing (its fallback) and image (its name). States and
   properties follow as the text shell's words, a collection's count and
   an item's position among them, a column header's sort; a check's,
   choice's and option's mark is its state, as there.
2. The text shell's `every.txt` fixture tree (every role, every state,
   every property: `skein/crates/skein-ui-text/src/tests/fixtures.rs`)
   built again in this crate's fixtures, through skein-ui's builder, and
   laid at 80 by 24 with the fixture theme, kept as
   `src/tests/golden/every.txt` (a `Frame::print`), written by hand from
   5.3 and ui-text.md's section 5. Where it does not fit in 24 rows, a
   second golden at 80 by 72, `every-tall.txt`, holds the whole.
3. Focusable roles among them (choice, check, option, tab, menu item, a
   select's options) keep their focus columns, and a disabled select's
   options are laid with `(disabled)` and are not focusable, as the key
   reader's focus order (6.2) needs.
4. No role is laid as a group any more: the arm `screen-render` gave them
   is gone.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, section 2.
- `skein/docs/design/ui-screen.md`, 5.3 and 5.4.
- `skein/docs/design/ui-text.md`, section 5, all of it, and
  `skein/crates/skein-ui-text/src/render.rs`'s role arms, the reference.
- `skein/docs/design/ui.md`, 3.1, 3.2 and section 16.

## Owns and may change

**Owned:** `skein-ui-screen`'s `render` module and its helpers, its tests,
fixtures and goldens. `screen-render`'s goldens stay byte for byte; a
change there is a red, not stale.

**Readers and neighbours:** none outside the crate.

**Options, each with its default:**

- A table's columns: each row's cells joined by ` | ` on one line, as the
  text shell prints them, wrapped like any line; aligned columns are
  later's. If a row is wider than its room, wrap it as any line.
- A code block's lines are not wrapped but cut at the room, their first
  columns kept, since a wrapped code line misleads; say so in a comment
  citing 5.3.

## Steps

1. The roles of acceptance 1, one arm each, the words copied once into
   the module holding the text shell's (`screen-render`'s choice).
2. The fixture and goldens (acceptance 2). Bite: drop a check's mark once
   and see `every.txt` fail, and restore.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-ui-screen --all-targets -- -D warnings`; `heavy cargo
   nextest run -p skein-ui-screen --lib`; `heavy cargo check -p
   skein-ui-screen-terminal --tests` and, if they exist on the trunk,
   `-p temper-client-tui --tests` and `-p temper-client-shell-tui --tests`
   (their goldens use only `screen-render`'s roles: a change in them is a
   red). Commit ("skein-ui-screen: every role laid out as the text shell
   says it"), push.

## Report

As the implementer's text says, and: whether the every-role tree fits 24
rows, and the goldens' time.

## Answers

### 2026-11-07: Resume after screen-render pushes its renderer (recommended), or authorize this task to implement the missing prerequisite renderer too? (tui-connected/screen-roles-1, foreman tui-connected/screen-roles/foreman-1)

Resume after screen-render pushes its renderer; do not implement the missing prerequisite in screen-roles.
The task declares `needs = ["screen-render"]`.
Its goal says: “`screen-render` lays the roles of an application's first screens and lays every other as a group; this task lays each of the others”.
Acceptance 4 requires: “the arm `screen-render` gave them is gone.”
The checkout has no render module or export in `skein/crates/skein-ui-screen/src/lib.rs`; the prerequisite is absent.
The next run merges `origin/tui-connected/screen-render`, verifies its renderer, fixture theme and goldens are present, then implements screen-roles.
If the prerequisite remains absent, keep the task parked and report it without creating a replacement renderer.
