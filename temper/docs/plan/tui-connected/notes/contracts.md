# tui-connected: the contracts

The shapes that cross tasks, as this run's binding calls. The design is
`skein/docs/design/ui-screen.md` for skein's side and
`temper/docs/design/client/README.md`, 6.11 to 6.16, for temper's; where
this note is more exact than the design (a signature, a field's type), it
binds. A body may differ from a sketch; a name, a public type or a
signature may not without a question.

## 1. The crates

| Crate | Path | Profile (`tools/lints/lints.py --set`) | Clippy | Depends on |
|---|---|---|---|---|
| `skein-ui-screen` | `skein/crates/skein-ui-screen` | `[lints] workspace = true` (step), as `skein-ui-text` | the tree's `skein/clippy.toml` | `skein-ui`, `skein-lib` |
| `skein-ui-screen-terminal` | `skein/crates/skein-ui-screen-terminal` | `[lints] workspace = true` (step), as `skein-ui-text` | the tree's | `skein-ui-screen`, `skein-ui`, `skein-lib`, `skein-io` |
| `skein-ui-screen-tests` | `skein/tests/ui-screen` | `harness`, as `skein/tests/ui-text` | `skein/tests/clippy.toml` | `skein-ui-screen-terminal`, `skein-ui-screen`, `skein-ui`, `skein-lib`, `skein-io`, `skein-sim` |
| `temper-client-tui` | `temper/crates/temper-client-tui` | `[lints] workspace = true` (step), as `temper-client-view` | the tree's `temper/clippy.toml` | `skein-ui`, `skein-ui-screen`, `skein-lib`, `jig-client`, `temper-client-domain`, `temper-client-view` |
| `temper-client-shell-tui` | `temper/crates/temper-client-shell-tui` | `binary`, with `tools/lints/clippy/ordinary.toml` as its `clippy.toml`, as `temper-client-shell-terminal` | its own | the above, `skein-ui-screen-terminal`, `skein-io`, `skein-shell`, `jig-client-protocol-http`, `jig-client-shell-native`, `temper-client-shell-terminal` (for its `limits`) |

Every crate is `no_std` with `alloc` but `temper-client-shell-tui` and
the test crate, and forbids `unsafe`. A new crate is a member of the root
`Cargo.toml` and an entry of its `[workspace.dependencies]`, as its
siblings are.

## 2. `skein-ui-screen`

```rust
// lib.rs: the full-screen shell's renderer (ui-screen.md, sections 3 to 5).
pub use theme::{Arrange, Border, Glyphs, Paint, Place, RegionLook, Rgb, StyleLook, Theme};
pub use frame::{Cell, Frame, Position, Size};
pub use focus::Focus;
pub use render::{render, width};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Rgb { pub red: u8, pub green: u8, pub blue: u8 }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Paint {
    pub fore: Option<Rgb>,
    pub back: Option<Rgb>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
}
impl Paint {
    pub const NONE: Paint;                                  // no colour, no attribute
    #[must_use] pub fn over(self, under: Paint) -> Paint;       // self's colours where set, attributes or'ed
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place { Top, Body, Bottom }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arrange { Lines, Spread }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Border {
    pub corners: [char; 4],    // top left, top right, bottom left, bottom right
    pub across: char,
    pub down: char,
    pub paint: Paint,
    pub title: bool,           // the region's first level-1 heading in the top border
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RegionLook {
    pub region: skein_ui::Region,
    pub place: Place,
    pub arrange: Arrange,
    pub border: Option<Border>,
    pub pad: u16,
    pub paint: Paint,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StyleLook { pub style: skein_ui::Style, pub paint: Paint }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Glyphs { pub focus: char, pub mask: char }

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Theme {
    pub base: Paint,
    pub focus: Paint,
    pub placeholder: Paint,
    pub regions: Box<[RegionLook]>,
    pub styles: Box<[StyleLook]>,
    pub glyphs: Glyphs,
}
impl Theme { #[must_use] pub fn valid(&self) -> bool; }

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Size { pub rows: u16, pub columns: u16 }
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Position { pub row: u16, pub column: u16 }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Cell { pub glyph: char, pub paint: Paint }   // '\0' follows a wide glyph
impl Cell { pub const BLANK: Cell; }                    // ' ' in Paint::NONE

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Frame { /* most, size, cells, cursor, scroll: private */ }
impl Frame {
    #[must_use] pub fn new(most: Size) -> Option<Frame>;  // size 0 by 0; a u32 counts any most's cells (screen-surface's question)
    #[must_use] pub const fn most(&self) -> Size;
    #[must_use] pub const fn size(&self) -> Size;
    /// Clears to `size` (at most `most`, each side), every cell `fill`; the cursor none.
    pub fn clear(&mut self, size: Size, fill: Cell);
    #[must_use] pub fn row(&self, row: u16) -> Option<&[Cell]>;
    /// Writes one cell; false past the size.
    pub fn put(&mut self, at: Position, cell: Cell) -> bool;
    #[must_use] pub const fn cursor(&self) -> Option<Position>;
    pub fn set_cursor(&mut self, at: Option<Position>);
    #[must_use] pub const fn scroll(&self) -> u32;        // the body's first row shown, as laid
    pub fn set_scroll(&mut self, scroll: u32);
    #[must_use] pub fn text(&self) -> Box<[u8]>;          // ui-screen.md, section 4
    #[must_use] pub fn print(&self) -> Box<[u8]>;         // ui-screen.md, section 4
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Focus {
    pub node: Option<skein_ui::NodeId>,
    pub caret: u32,     // a byte offset in the focused field's value, at a character's boundary
    pub scroll: u32,    // the body's first row shown
    pub honoured: u64,  // the last focus request (Node::focus) honoured
}

/// ui-screen.md, section 5: lays `tree` into `frame` at `size` (cut to `frame.most()`).
/// Honours a newer focus request, drops a focus whose node is gone, and keeps
/// the focused node in view by moving `focus.scroll`. `theme` is valid.
pub fn render(tree: &skein_ui::Tree, theme: &Theme, size: Size, focus: &mut Focus, frame: &mut Frame);
/// ui-screen.md, 5.1: 0, 1 or 2 cells.
#[must_use] pub fn width(glyph: char) -> u8;
```

## 3. `skein-ui-screen-terminal`

```rust
// lib.rs: the full-screen shell's keys, drawing and terminal (ui-screen.md, sections 6 to 8).
pub use keys::{Decoder, Key};
pub use command::{Command, command};
pub use draw::{Depth, ENTER, LEAVE, draw, draw_bound};
pub use terminal::{Settings, Terminal};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Key {
    Char(char),
    Paste(Box<[u8]>),
    Enter, Tab, BackTab, Backspace, Delete,
    Up, Down, Left, Right, Home, End, PageUp, PageDown,
    Escape,
    Control(u8),   // b'a' to b'z'
    Alt(char),
}

pub struct Decoder { /* partial sequence, paste in progress: private */ }
impl Decoder {
    #[must_use] pub fn new(paste: u32) -> Decoder;              // the most bytes of a paste
    /// One read's bytes; a lone ESC at its end is Escape (ui-screen.md, 6.1).
    pub fn read(&mut self, bytes: &[u8], keys: &mut skein_lib::Queue<Key>);
}

#[derive(PartialEq, Eq, Debug)]   // no Clone: skein_ui::Act has none, and no hand-written impl (programming-model.md, 10.3)
pub enum Command {
    Act { node: skein_ui::NodeId, act: skein_ui::Act },
    Moved,    // the focus, its caret or the scroll moved: draw again
    Close,
    Nothing,
}
/// ui-screen.md, 6.2.
pub fn command(key: &Key, tree: &skein_ui::Tree, focus: &mut skein_ui_screen::Focus, limits: &skein_ui::Limits) -> Command;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Depth { True, Indexed }
pub const ENTER: &[u8];  // ESC[?1049h ESC[?25l ESC[?2004h ESC[?7l ESC[2J ESC[H
pub const LEAVE: &[u8];  // ESC[?2004l ESC[0m ESC[?7h ESC[?25h ESC[?1049l
/// ui-screen.md, section 7: appends to `out` the bytes from `last` (none: from nothing) to `next`.
pub fn draw(last: Option<&skein_ui_screen::Frame>, next: &skein_ui_screen::Frame, depth: Depth, out: &mut skein_lib::List<u8>);
#[must_use] pub fn draw_bound(most: skein_ui_screen::Size) -> Option<u32>;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub interval: skein_lib::Duration,
    pub depth: Depth,
    pub most: skein_ui_screen::Size,
    pub paste: u32,
}
pub struct Terminal { /* private */ }
impl Terminal {
    #[must_use] pub fn operations(settings: &Settings) -> Option<u32>;
    #[must_use] pub fn new(settings: Settings, size: skein_ui_screen::Size) -> Option<Terminal>;
    /// Writes ENTER first. Gives back the fd io refused.
    pub fn adopt(&mut self, input: skein_io::kernel::Fd, output: skein_io::kernel::Fd) -> Result<(), skein_io::kernel::Fd>;
    #[must_use] pub fn owns(op: skein_lib::Token) -> bool;
    pub fn completions(&mut self) -> &mut skein_lib::Queue<skein_io::kernel::Complete>;
    pub fn submissions(&mut self) -> &mut skein_lib::Queue<skein_io::kernel::Submit>;
    pub fn keys(&mut self) -> &mut skein_lib::Queue<Key>;
    pub fn resized(&mut self);                            // io's Resized, heard by the host
    #[must_use] pub const fn size(&self) -> skein_ui_screen::Size;
    pub fn draw(&mut self, frame: &skein_ui_screen::Frame);  // the host's newest frame
    #[must_use] pub const fn ended(&self) -> bool;        // the input's end was read
    pub fn close(&mut self);
    #[must_use] pub const fn closed(&self) -> bool;       // LEAVE written, nothing more
    #[must_use] pub fn next_deadline(&self) -> Option<skein_lib::Time>;
    #[must_use] pub fn work_pending(&self, now: skein_lib::Time) -> bool;
    pub fn iterate(&mut self, now: skein_lib::Time);
}
```

Its io tag is 1 (`NAMESPACE: u8 = 1`), free beside the text shell's 4
and the hosts' bits 61 and 63; a host that composes it routes by
`Terminal::owns`.

## 4. The panic's way out

```rust
// skein-fail-stop, lib.rs
/// What the hook runs once, before its report (programming-model.md, section 2).
pub type Restore = Box<dyn Fn() + Send + Sync>;
/// Sets the restoration, or takes it back with None. The hook takes it out before
/// running it, so it runs at most once.
pub fn before_report(restore: Option<Restore>);

// skein-shell, terminal.rs, impl Kept
/// shell.md, 6.3: writes `leave` to `output` with blocking writes, then sets the kept
/// modes back on the kept descriptor. Errors are ignored: it runs inside a panic.
#[must_use]
pub fn on_abort(&self, output: Fd, leave: &'static [u8]) -> Box<dyn Fn() + Send + Sync>;
```

skein-shell does not depend on skein-fail-stop: the binary hands the one
to the other.

## 5. The simulated window

```rust
// skein-sim, impl Sim
/// A window size for one of `pid`'s descriptors; Op::Window on it answers Done::Window(size).
pub fn set_window(&mut self, pid: Pid, fd: Fd, size: skein_io::kernel::Size);
/// Signalled::WindowChanged on `pid`'s signal source `fd`, in order with its other signals.
pub fn deliver_window_changed(&mut self, pid: Pid, fd: Fd);
```

## 6. The TUI's host seam

`temper-client-shell-tui`'s `host` module. The real loop (`run`), the
client's joint world and M3's recorder and replay all call it, and
nothing else of the host (`temper/docs/design/client/README.md`, 6.13).

```rust
pub struct Start {
    pub size: skein_ui_screen::Size,
    pub scheme: temper_client_tui::Scheme,          // Dark | Light
    pub address: Box<[u8]>,                         // "HOST:PORT", the clients' listener on loopback
    pub kept: Vec<jig_client::Record>,              // the small store's records
    pub credential: Option<skein_lib::Secret>,      // DIRECTORY/token's, if any
}
pub enum Report {
    Start(Start),
    Due,
    Key(skein_ui_screen_terminal::Key),
    Resize(skein_ui_screen::Size),
    Heard(jig_client::Event),                       // the transport's and the small store's, as the native shell gives them
    Kept { written: bool },                         // the credential's write ended
}
#[derive(Default)]
pub struct Duties {
    pub asked: Vec<jig_client::Request>,            // to the native shell, in order
    pub keep: Option<Option<skein_lib::Secret>>,    // the credential to write; Some(None) erases it
    pub address: Option<Box<[u8]>>,                 // the address signed in to, to keep
    pub open: Option<Box<[u8]>>,                    // an address for a browser: none in M1
    pub close: bool,                                // the shell closes its terminal and its transport
}
pub struct Settings { /* the TUI's limits and its view's: temper-client-shell-tui's `limits` */ }
pub struct Host { /* the root, the TUI's state and view, the focus, the frame: private */ }
impl Host {
    #[must_use] pub fn new(settings: &Settings, seed: u64, now: skein_lib::Time, wall: skein_lib::Wall) -> Host;
    pub fn clocks(&mut self, now: skein_lib::Time, wall: skein_lib::Wall);
    pub fn report(&mut self, report: Report) -> Duties;
    /// The frame for the tree as it is now, built and laid out.
    pub fn frame(&mut self) -> &skein_ui_screen::Frame;
    #[must_use] pub fn tree(&self) -> &skein_ui::Tree;
    /// The event the view makes of a key, without giving it (M3's recorder).
    #[must_use] pub fn event_of(&self, key: &skein_ui_screen_terminal::Key) -> Option<temper_client_tui::Event>;
    #[must_use] pub fn deadline(&self) -> Option<skein_lib::Time>;
}
```

`Report::Start` comes once, first: before it, every other report is the
host's defect (a panic). `Duties::close` is set once, and every report
after it but `Heard` and `Kept` does nothing.
