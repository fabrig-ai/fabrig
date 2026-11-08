# tui-launchpad-2: the contracts between tasks

Binding for every task of the plan. Design: `temper/docs/design/client/README.md`,
6.12 to 6.15; `temper/docs/design/client/tui/journeys.md`, 3.2;
`skein/docs/design/ui-screen.md`, 5.2.

## 1. `temper-client-tui` (task `look` makes all of it)

```rust
// theme.rs
pub const BAND_STEP: u8 = 20;
/// The background moved one step toward the ink: each channel by BAND_STEP,
/// saturating, toward 255 where `Scheme::of_background(background)` is Dark,
/// toward 0 where it is Light (6.15, The band).
pub const fn band(background: Rgb) -> Rgb;
/// Six colours: bg, ink, dim, faint, accent, good, in that order; no `sel`.
pub const fn palette(scheme: Scheme) -> [(&'static str, Rgb); 6];
/// `background` is the terminal's answer to OSC 11, or None.
pub fn theme(scheme: Scheme, background: Option<Rgb>) -> Theme;

// place.rs, re-exported at the crate root as `place`
/// "box" (on a box, port 8367), "box:PORT" (on a box, another port),
/// else the address as given. `address` is HOST:PORT.
pub fn place(address: &[u8], boxed: bool) -> Box<[u8]>;

// event.rs: one new variant
pub enum Event { /* ..., */ LeftOff { task: Option<u64> } }

// view.rs, re-exported at the crate root as `Arrow` and `arrow`
pub enum Arrow { Up, Down }
/// The event Up or Down makes on the tree last built, or None for the
/// key reader to have the key. Task `look` returns None always; task
/// `left-off` gives it its body.
pub fn arrow(view: &View, arrow: Arrow) -> Option<Event>;

// state.rs
pub enum Told { /* ..., */ LeftOff { task: Option<u64> }, Boxed }
impl Tui {
    pub const fn left_off(&self) -> Option<u64>; // the selected left-off chat's task
    pub const fn boxed(&self) -> bool;           // on a box (6.12, On a box)
}
```

- `Told::LeftOff` sets the selection, `Told::Boxed` sets `boxed`; each
  changes the revision when it changes the state. `Told::Went` also
  clears the selection.
- Look names, appended after `tui-command` in `look::NAMES`' order, each
  a `pub const` in `look`:

  | Const | Name | Look (6.15) |
  |---|---|---|
  | `BULLETS` | `tui-bullets` | no paint; mark `•`, no paint |
  | `LEFT_OFF` | `tui-left-off` | no paint; mark `' '` (a space), no paint |
  | `PICKED` | `tui-picked` | accent over the band where there is one; `band` where there is one; mark `❯`, accent |
  | `RULE_TITLE` | `tui-rule-title` | no paint; aligned to the end |

  Task `chat-rule` later removes `STAR` (`tui-star`) and `CHAT_TOP`
  (`tui-chat-top`) and renumbers what follows them; every reader names
  the consts, never their numbers.

## 2. `temper-client-shell-tui` (task `seam` makes all of it)

```rust
// host.rs
pub struct Start {
    pub size: skein_ui_screen::Size,
    pub scheme: temper_client_tui::Scheme,
    pub background: Option<skein_ui_screen::Rgb>, // the terminal's answer, or None
    pub address: Box<[u8]>,
    pub local: bool,
    pub boxed: bool,                               // on a box
    pub kept: Vec<jig_client::Record>,
    pub credential: Option<skein_lib::Secret>,
    pub note: Option<temper_client_tui::Note>,
}
// run.rs: `run::Settings` gains `pub boxed: bool`.
// journey/record.rs: `journey::Facts` gains `pub background: Option<Rgb>` and `pub boxed: bool`.
```

- The host keeps both from `Start` (`seam`); task `arrows` hands them on:
  `tui::theme(scheme, background)` at each report, and `Told::Boxed`
  given to the TUI's state at `Start` when `boxed`.
- The recording's start facts: `background R G B` (three decimal numbers)
  after `screen`, when the start has one; `box` (no field) after `local`,
  when `boxed` (`tui/journeys.md`, 3.2). Absent, they read as none and
  false.

## 3. The binary (task `seam`)

- `temper/crates/temper/src/tui.rs` reads the engine's link file at
  start whatever the argument, and sets `Settings.boxed` when the link it
  holds, read by `jig-client-protocol-http`'s `printed`, names the TUI's
  address. The link's secret is dropped there.

## 4. The skein renderer (task `rule-title`)

- No type or signature changes: `skein_ui_screen::render` lays a title
  whose heading's children all align `End` with nothing at the border's
  start but its glyphs across, and, in a rule (a border with no sides),
  the end's part before two glyphs across (`skein/docs/design/ui-screen.md`,
  5.2). A box's titles are laid as before.
