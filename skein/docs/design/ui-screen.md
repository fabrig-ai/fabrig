# The full-screen shell

Provisional, 2026-11-07. The design of `skein-ui-screen` and
`skein-ui-screen-terminal`: skein-ui's shell for a person at a terminal
who works in it full screen. It lays the view's tree (ui.md, section 3)
out on a grid of cells, placing regions by their names (ui.md, section
8) in the colours, glyphs and borders of a theme the application gives,
and reads a person's keys as acts on nodes (ui.md, section 6). It says
what its parts are, the theme, how a frame is laid out, the keys, the
terminal and its ways out, and how it is tested. What is left for later
is in section 10, what is open in section 11.

## 1. In one page

- **The tree as a frame, a key as an act.** The renderer lays the whole
  tree out as a frame, a grid of cells the window's size; the key reader
  turns a key into one of skein-ui's acts on a node (ui.md, section 6),
  a move of the focus, or closing.
- **Every application's.** It names no application: an application's
  full-screen client is its state, its view and its theme, composed
  with this shell under its own host. Its look is the theme: colours,
  glyphs and borders for the application's style tokens and regions, and
  where each region goes (section 3).
- **Regions placed by name, never by order.** The theme places a
  region at the top, in the body, or at the bottom, framed or not, its
  children on lines or spread along one row. The rest of the tree is the
  body, which scrolls to keep the focus in view.
- **By role, as the text shell says it.** Each role is laid out with the
  text shell's words and marks (ui-text.md, section 5), wrapped at its
  region's width, without its numbers: here a person moves a focus and
  presses keys. States and properties stay words, since nothing a person
  must know is said by a colour alone (ui.md, section 8).
- **Keys, not lines.** Tab and the arrows move the focus, Enter and Space
  act on it, typed and pasted text edits the focused field, a node's
  shortcut presses it (ui.md, 6.1), and Ctrl+C closes.
- **The terminal left as it was found.** The terminal enters the
  alternate screen, hides the cursor and takes pastes whole; on every
  way out it writes the same bytes that undo it, a panic's included,
  and the startup's modes are set back (section 8).
- **Pure where it can be.** The renderer, the key reader, the key
  decoder and the drawing of a frame as bytes are step code, bounded by
  the most cells a frame holds; the terminal is ordinary Rust on skein's
  io, driven by the application's host.
- **Tested by golden frames,** printed as text with their paints, every
  key on every kind of focus, and the terminal on skein's simulated
  machine (section 9).

## 2. In skein

| Crate | Kind | Holds | Depends on |
|---|---|---|---|
| `skein-ui-screen`, `crates/skein-ui-screen` | step code under the subset, `no_std` with `alloc` | the theme (section 3), the frame and its print (section 4), the focus, the renderer (section 5) | `skein-ui`, `skein-lib` |
| `skein-ui-screen-terminal`, `crates/skein-ui-screen-terminal` | the key decoder, the key reader and the drawing: step code; the terminal: ordinary Rust, as shells are (programming-model.md, 10.2) | the keys (section 6), the drawing (section 7), the terminal (section 8) | `skein-ui-screen`, `skein-ui`, `skein-lib`, `skein-io` |
| `tests/ui-screen`, `skein-ui-screen-tests` | test crate | the terminal on the simulated machine (section 9) | |

Both forbid `unsafe`. The startup that sets the terminal's input raw and
keeps its modes is skein-shell's (shell.md, 6.3); the hook that leaves the
terminal at a panic is skein-fail-stop's (programming-model.md, section 2).

```
a person at a terminal, full screen
  ▲ bytes out: frames, entering, leaving    ▼ bytes in: keys, pastes; the window resized
skein_ui_screen_terminal::Terminal       the raw input as a read pipe, the output as a write pipe, on skein's io
  ▲ frames                                  ▼ keys; the window's size
the application's host                   its loop over skein-shell's drive: its steps, its view, the renderer, the key reader
```

It is one of skein-ui's shells (ui.md, section 16), beside the text shell
(ui-text.md) and the browser parts (ui-web.md). The fake person reads the
same tree, not a frame (ui.md, 17.1).

## 3. The theme

`skein_ui_screen::Theme`, which the application gives whole, with no
defaults. An application with a dark and a light look gives the one its
host chose (its own design says how).

- **A paint,** `Paint`: a foreground and a background colour, each a
  24-bit `Rgb` or none, and the attributes bold, dim, italic, underline
  and reverse. A paint laid over another (`Paint::over`) takes each
  colour it names and adds each attribute it sets; none leaves the one
  under it.
- **The base,** the paint of every cell before anything is laid on it.
- **A region's look,** `RegionLook`, at most one per region (ui.md,
  section 8):
  - *its place,* `Place`: `Top`, `Body` or `Bottom` (5.2);
  - *its arrangement,* `Arrange`: `Lines`, each child on lines of its
    own, or `Spread`, its children along one row (5.3);
  - *its border,* `Border` or none: four corners, a glyph across and one
    down, their paint, and whether the region's first level-1 heading is
    drawn in its top border, as its title;
  - *its padding,* the columns kept clear inside each side;
  - *its paint,* laid over the base for every cell of the region.
  A region with no look is laid in the body, as any other node is.
- **A style token's look,** `StyleLook`: its paint, laid over its
  region's on the node that carries the token and every node under it,
  the nearest token's last. A token with no look paints nothing.
- **The focus's paint,** laid last over the focused node's cells; the
  **placeholder's paint**, for a field's placeholder (5.4).
- **Its glyphs,** `Glyphs`: the focus mark drawn before the focused
  node, and the mask drawn for each character of a secret's value.
- **Checked:** `Theme::valid` refuses two looks for one region or one
  token, and a glyph that is not one cell wide (section 5) or is a
  control character. The renderer takes only a valid theme.

## 4. The frame

- **A grid of cells,** `Frame`: its size, `Size { rows, columns }`, and a
  `Cell` per row and column, each a glyph and its paint. A glyph two
  cells wide takes its cell and the next, which holds U+0000 and is
  drawn as nothing. The frame also holds the cursor's position, when a
  field has the focus (5.4).
- **Bounded at its making:** `Frame::new(most)` holds at most `most`'s
  cells, and the renderer lays a window larger than that in its top-left
  `most` alone. `None` for a `most` whose cells pass a u32.
- **Its text,** `Frame::text`: each row as UTF-8, a line per row, a wide
  glyph written once, the row's trailing spaces removed.
- **Its print,** `Frame::print`, a frame as text with its paints, for a
  golden and a diff: the text; a line `--`; a line per row with one
  letter per cell naming its paint, `.` for the base, `a` to `z` and
  then `A` to `Z` for the others in the order they first appear, and
  `?` past those; a line `--`; a line per letter, `a: fore #RRGGBB back
  #RRGGBB bold dim italic underline reverse`, naming only what is set;
  and the cursor, `cursor ROW COLUMN`, or `no cursor`.

## 5. How it lays out

`skein_ui_screen::render(tree, theme, size, focus, frame)`: from the
view's tree, a valid theme, the window's size and the shell's focus, the
frame. A pure function, step code under the subset, with no io and no
clock, writing into the frame it is given and allocating nothing past it.

### 5.1 The text

- **A character's width,** `skein_ui_screen::width`: two cells for East
  Asian Wide and Fullwidth characters (Unicode's `EastAsianWidth.txt`),
  none for a combining mark and the other zero-width characters, which
  are not drawn (section 10), and one for every other.
- **Nothing from the tree drives the terminal,** as in the text shell
  (ui-text.md, section 5): a control character and a byte that is not
  UTF-8 are laid as U+FFFD, a tab as a space; a newline starts a new
  line at the same indentation.
- **Wrapped at its room:** a line longer than its region's inner width
  breaks at its last space within it, else at the width, and goes on at
  the same indentation.

### 5.2 The bands

- **Three bands,** top to bottom: the top regions, the body, the bottom
  regions. The top regions are laid from the first row down in preorder;
  the bottom regions from the last row up, the last in preorder lowest;
  the body takes the rows between.
- **Room in that order:** the bottom regions first, then the top, then
  the body, each cut at what is left; a region cut shows its first rows.
  A frame is never laid past its size.
- **A framed region** spans the frame's width: its border, then its
  padding, then its content at the inner width that leaves. A region
  whose border draws its title has its first level-1 heading in its top
  border, after the first corner and one glyph across, between spaces,
  and not again inside: `╭─ temper ───╮`.
- **The body** is every node that is not in a placed region, in preorder,
  laid as lines at the frame's width. It scrolls: `Focus::scroll` is its
  first row shown. The renderer keeps the focused node's lines in view,
  moving the scroll the least it must, and never past the body's last
  row; the frame says the scroll it used, which the shell keeps.

### 5.3 The roles

- **Each node by its role, as the text shell prints it** (ui-text.md,
  section 5): the same words, marks, indentation, states and properties,
  with these differences:
  - *no numbers:* where the text shell prints `[n] `, nothing, and a
    node it numbers `[-]` is laid with its `(disabled)` state alone;
  - *no first line:* the text shell's `== TITLE == STATUS` is not laid;
    a heading is laid where it stands, unless a border draws it;
  - *the focus mark:* each node a person acts on (the roles the text
    shell numbers, ui-text.md, section 6) keeps two columns before it,
    which hold the theme's focus mark and a space while it has the focus,
    so the focus moving shifts no text;
  - *paints:* each cell takes the base, its region's paint, its tokens'
    looks, and the focus's paint while its node has the focus; a code
    run keeps its backticks.
- **A spread region** lays each child's text on one row, the first child
  from the row's start and the others from its end, two spaces between
  them, each child's paint its own; what does not fit is cut from the
  first child's end. It takes one row, whatever its children.
- **Not laid:** what the text shell does not print (ui-text.md, section
  5), but the placeholder, which the full-screen shell draws (5.4); a
  shortcut is the node's own words' to say.

### 5.4 Fields and the focus

- **The focus,** `Focus`: the focused node's id or none, the caret's
  byte offset in its value, the body's scroll and the last focus request
  honoured. The shell keeps it; the renderer and the key reader change
  it.
- **A focus request** (ui.md, 3.2) newer than the last honoured moves
  the focus to its node, the caret at its value's end. A focused node
  gone from the tree, or no longer one a person acts on, leaves the
  focus none.
- **A text field** takes one row: `NAME: ` and its value from the offset
  that keeps the caret in view; an empty value shows its placeholder in
  the placeholder's paint, if it has one. A secret's value is the
  theme's mask, once per character, and never its bytes. A text area
  wraps its value as lines (5.1).
- **The cursor** is shown at the caret while a field or a text area has
  the focus, and hidden otherwise.

## 6. Keys

### 6.1 Decoding

`skein_ui_screen_terminal::Decoder` turns the input's bytes into keys,
`Key`, step code bounded by the longest sequence it reads (16 bytes) and
the most bytes of a paste:

| Bytes | Key |
|---|---|
| a UTF-8 character but the controls below | `Char` |
| `\r`, or `\n` | `Enter` |
| `\t`; `ESC [ Z` | `Tab`; `BackTab` |
| `0x7f`, or `0x08` | `Backspace` |
| `ESC [ 3 ~` | `Delete` |
| `ESC [ A` to `D`, or `ESC O A` to `D` | `Up`, `Down`, `Right`, `Left` |
| `ESC [ H`, `ESC [ 1 ~`, `ESC O H`; `ESC [ F`, `ESC [ 4 ~`, `ESC O F` | `Home`; `End` |
| `ESC [ 5 ~`; `ESC [ 6 ~` | `PageUp`; `PageDown` |
| `0x01` to `0x1a` but the above | `Control` and its letter |
| `ESC` and a printable character | `Alt` and the character |
| `ESC` alone at the end of a read | `Escape` |
| `ESC [ 200 ~` ... `ESC [ 201 ~` | `Paste` and the bytes between, as one key |

- **Split across reads:** a sequence or a character begun in one read
  ends in the next; only `ESC` at a read's end with nothing after it is
  `Escape`.
- **Bounded:** a sequence that is none of these, or longer than 16 bytes,
  is dropped whole; a paste past the settings' most bytes keeps its first
  bytes up to that most, at a character's boundary, and drops the rest
  through its end.

### 6.2 The key reader

`skein_ui_screen_terminal::command(key, tree, focus, limits)`: from one
key, the view's last tree, the shell's focus and skein-ui's limits, one
command: an act on a node (skein-ui's `Act`), the focus or the scroll
moved, closing, or nothing. Pure, step code under the subset.

| Key | With the focus | Does |
|---|---|---|
| `Control` `c` | anywhere | closes (section 8) |
| `Tab`, `Down` | anywhere | the focus to the next node a person acts on, bound and enabled, in preorder, from the first after the last |
| `BackTab`, `Up` | anywhere | the focus to the one before, from the last after the first |
| `PageUp`, `PageDown` | anywhere | the body's scroll by its rows less one |
| `Enter` | a button, a link, a tab, a menu item, an option | `Press` |
| `Enter`, `Char` space | a check, a choice | `Toggle` |
| `Enter` | a text field or area that submits on enter | `Submit` |
| `Char` space | a button, a link, a tab, a menu item, an option | `Press` |
| `Char`, `Paste` | a text field or area | `Input` of its value with the text put in at the caret, the caret after it; in a text field a paste's line breaks are spaces, and its last ones are dropped |
| `Backspace`, `Delete` | a text field or area | `Input` of its value without the character before or after the caret |
| `Left`, `Right`, `Home`, `End` | a text field or area | the caret moved, by a character or to an end |
| `Char` a letter or a digit | none, or not a field | `Press` of the first enabled node naming it as a shortcut with no modifier (ui.md, 6.1) |
| `Control`, `Alt` and a letter or a digit | anywhere | `Press` of the first enabled node naming it with the primary modifier or with Alt (ui.md, 6.1) |
| `Escape` | anywhere | `Press` as ui.md, 6.1 says of Escape |
| any other | | nothing |

- **An act only where its role takes it** (ui.md, section 6), on a
  bound, enabled node, so the view never refuses a key's act for its
  role.
- **A value within its bound:** an `Input` whose value would pass the
  limits' text of a node is not made: the key does nothing.
- **The primary modifier** is Control: the terminal reads keys as
  Control and Alt on every platform (section 11).

## 7. Drawing

`skein_ui_screen_terminal::draw(last, next, depth, out)`: the bytes that
take the screen from the last frame drawn, or from nothing, to the next.
Step code, writing into the list it is given.

- **Only what changed:** each run of cells that differs from the last
  frame's is moved to and written, with the paints that change before
  it; with no last frame, or one of another size, the screen is cleared
  and every cell written.
- **Colours** as 24-bit `SGR 38;2` and `48;2` at depth `True`, and as the
  nearest of the 256 indexed colours, the 6×6×6 cube and the gray ramp,
  at depth `Indexed`; a colour that is none is the terminal's default.
- **The cursor** last: moved to the frame's cursor and shown, or hidden.
- **Bounded:** at most the bytes of a whole frame of `most`'s cells, each
  with its paint changed, which `skein_ui_screen_terminal::draw_bound`
  gives.

## 8. The terminal

`skein_ui_screen_terminal::Terminal`: ordinary Rust on skein's io, its
input adopted as a read pipe and its output as a write pipe (io.md,
section 3), its operations under a tag of its own so its host routes their
completions to it, as the text shell's terminal (ui-text.md, section 3).

- **Its settings,** `Settings`, given whole: the least time between two
  frames drawn; the colours' depth; the most cells of a frame; and the
  most bytes of a paste.
- **Its operations,** `Terminal::operations`, the most in flight at
  once, which the host's ring holds beside its own; none for settings it
  refuses.
- **Entering,** at adoption, before anything else: `ENTER`, the
  alternate screen (`ESC [ ? 1049 h`), the cursor hidden (`ESC [ ? 25
  l`), pastes bracketed (`ESC [ ? 2004 h`), no wrap at the last column
  (`ESC [ ? 7 l`), and the screen cleared.
- **Keys** are decoded as they are read and wait for the host
  (`Terminal::keys`); the terminal asks to read only once the host has
  taken every key it gave, a read's worth at once.
- **Frames:** the host gives each frame it lays (`Terminal::draw`). At
  most one is drawn per interval, the newest, as the bytes `draw` makes
  from the last drawn; one write at a time, each at most the output's
  room, so a frame larger than it goes in pieces before the next.
- **The window resized:** the host gives io's `Resized` to the terminal
  (`Terminal::resized`), which asks its input's window size (io.md,
  section 7) and gives the size to the host (`Terminal::size`); the next
  frame is drawn whole.
- **Closing** (`Terminal::close`): the key reader's Ctrl+C, the input's
  end, a termination signal the host hears, or the application's own
  close. The terminal starts no new frame, finishes the write it began,
  then writes `LEAVE`: pastes no longer bracketed (`ESC [ ? 2004 l`),
  the paints reset (`ESC [ 0 m`), wrap at the last column (`ESC [ ? 7
  h`), the cursor shown (`ESC [ ? 25 h`) and the alternate screen left
  (`ESC [ ? 1049 l`); then it is closed (`Terminal::closed`) and writes
  nothing more.
- **Its startup** is the host's binary's, with skein-shell's parts
  (shell.md, 6.3): the input set raw with the interrupt keys passed
  through as bytes (`Keys::Pass`), so that Ctrl+C is a key and Ctrl+Z
  and Ctrl+\\ stop nothing; the window's signal watched with the
  termination signals (`Watch::TerminationAndWindow`); and the full-screen
  shell run only when the input and the output are both terminals.
- **Every way out leaves the terminal as it was found.** Each closing
  above writes `LEAVE`, and once `drive` returns the binary's `main`
  sets the kept modes back (`Kept::restore`). A panic aborts (fail-stop),
  so it has a way of its own: once the input is raw, `main` gives
  skein-fail-stop the restoration skein-shell makes from the kept modes
  (`Kept::on_abort(output, LEAVE)`), which the hook runs before its
  report: `LEAVE` written to the output, then the modes set back, so the
  report reaches the person's own screen. `main` takes it back
  (`skein_fail_stop::before_report(None)`) before `Kept::restore`. Only a
  process killed outright leaves the terminal as it was set, as any
  program that sets it does.

## 9. Its tests

- **Golden frames** fix the renderer: frames' prints (section 4) kept as
  files beside the crate's step tests (`src/tests/golden/`) and changed
  only by hand, built from fixture trees through skein-ui's builder and a
  fixture theme, with no application's view:
  - a tree with every role, every state and every property, as the text
    shell's golden has it (ui-text.md, section 8), at 80 by 24;
  - regions placed at the top, framed with a title, and at the bottom,
    spread, around a body; the same in a frame too small for them;
  - wrapping, a wide glyph at a row's end, control characters and bytes
    that are not UTF-8;
  - a field focused with its caret, a secret masked, a value scrolled to
    its caret, a placeholder; the body scrolled to keep the focus;
  - `Theme::valid`'s refusals.
- **The keys:** the decoder's table, each sequence whole and split at
  every byte; the key reader's table, each key on each kind of focus,
  with a disabled node, a shortcut while a field has the focus, Escape in
  a dialog and in a form, and a value at its bound.
- **The drawing:** a first frame whole, one cell changed, a resize whole,
  the cursor shown and hidden, the indexed depth's nearest colours, and
  `draw_bound` for a frame at `most` with every paint changing.
- **The terminal** on skein's simulated machine, in `tests/ui-screen`
  (testing.md, section 6): `ENTER` first; keys split across reads; frames
  paced, one at a time, the newest drawn; a frame larger than the output's
  room in pieces; a resize asking the window and drawing whole; `LEAVE`
  last on each way out, closing, the input's end and a close while a
  frame is being written; nothing after it.
- **The panic's way out:** skein-fail-stop's fixture sets a restoration
  that writes a mark, panics, and its test finds the mark before the
  report (`tests/fail-stop`).
- **An application's full-screen client** is tested in its own tiers: its
  worlds play this shell's renderer and key reader through its host,
  never a terminal.

## 10. Later

- **Grapheme clusters:** a combining mark joined to the cell before it,
  and an emoji sequence as one glyph, in place of marks not drawn.
- **A log's edge near** (ui.md, section 6): the renderer measures which
  bound log's edge is in view and the shell reports `Near`; until an
  application draws a log, a log is laid whole.
- **A text area's rows:** `Up` and `Down` moving the caret by rows in a
  text area, in place of the focus.
- **The pointer:** a mouse's press on a node and its wheel on the body.
- **The terminal's own colours,** asked of it at start (its background's
  answer), so that an application need not choose dark or light.
- **Right-to-left text,** with the text shell's (ui-text.md, section 9).

## 11. Open questions

- **Apple's primary modifier:** a terminal gives no Command key, so the
  primary modifier is Control on every platform here (ui.md, 6.1 names
  Command on Apple's). Whether a shortcut with the primary modifier
  should answer to Control on macOS waits for an application that names
  one.
- **The shell's own words:** its states and entry kinds are the text
  shell's English words (ui-text.md, section 9).
