# tui-box-polish: the contracts

The shapes that cross this plan's tasks, binding as written. The design
they implement is `temper/docs/design/client/README.md`, 6.13 to 6.17,
and `skein/docs/design/ui-screen.md`, sections 3 and 5.4. Each names
the task that makes it.

## skein-ui-screen (`focused-value`)

```rust
pub struct StyleLook {
    // ... as now, then, after `active_name`:
    /// On a focused text field or text area carrying this token: the paint
    /// laid over the node's own paint on its value and its caret's cell, in
    /// place of the focus's paint (ui-screen.md, 3 and 5.4).
    pub focused_value: Option<Paint>,
    // ... `spaced` as now
}
```

`temper-client-tui`'s `theme.rs` sets it in `style()`, `None`, and
`box-ink` names `Some(Paint::NONE)` on `look::FIELD`.

## temper-client-tui

The notice (`notice`), `temper/docs/design/client/README.md`, 6.14,
*The notice*; in `state.rs`, exported from the crate's root beside `Told`:

```rust
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Notice {
    Unknown { words: Box<[u8]> },   // the box's words, its "/" first
    SigningIn,                      // "Still signing in"
    // `theme-list` adds: Theme { choice: theme::ThemeChoice, terminal: Option<Scheme> },
}
pub enum Told { /* ... */ Notice { notice: Option<Notice> }, /* replaces Unknown { words } */ }
impl Tui { pub fn notice(&self) -> Option<&Notice>; /* replaces unknown() */ }
// limits.rs
pub const NOTICE: skein_lib::Duration = skein_lib::Duration::from_secs(6);
```

The closing command's name (`quit`), `README.md`, 6.14, *`/quit`*; in
`event.rs`, exported beside `LOG_COMMAND`:

```rust
pub const QUIT: &[u8] = b"/quit";
pub const EXIT: &[u8] = b"/exit";   // sent whole, the same command, never listed
/// The TUI's name of a root command: QUIT for Command::Exit, else command.name().
pub fn command_name(command: Command) -> &'static [u8];
```

The theme chosen (`theme-start`), `README.md`, 6.15, *The theme
chosen*; in `theme.rs`, reached as `temper_client_tui::theme::ThemeChoice`
and `temper_client_tui::theme::THEME_KEY` (the crate root's `lib.rs`
is not edited for it):

```rust
pub const THEME_KEY: &[u8] = b"theme";
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeChoice { Auto, Light, Dark }
impl ThemeChoice {
    pub const ALL: [ThemeChoice; 3] = [ThemeChoice::Auto, ThemeChoice::Light, ThemeChoice::Dark];
    pub const fn print(self) -> &'static [u8];          // b"auto", b"light", b"dark"
    pub fn read(bytes: &[u8]) -> ThemeChoice;           // anything else: Auto
    pub const fn scheme(self, terminal: Option<Scheme>) -> Scheme; // Auto: terminal, else Dark
}
```

The theme's list (`theme-list`), `README.md`, 6.13 and 6.14, *`/theme`*:

```rust
pub enum Own { Change, Ask, Brief, Needs, Open, Theme }       // ALL gains Theme, name b"/theme"
pub enum Event { /* ... */ Theme { choice: Option<theme::ThemeChoice> } }
pub enum Told { /* ... */ Theme { choice: theme::ThemeChoice, terminal: Option<Scheme> }, ThemeList { open: bool } }
impl Tui {
    pub fn theme_choice(&self) -> theme::ThemeChoice;   // Auto until told
    pub fn terminal_scheme(&self) -> Option<Scheme>;
    pub fn theme_list(&self) -> bool;
}
```

Typing before the sign-in lands (`typing`), `README.md`, 6.14, *Typed
before the sign-in lands*; in `view.rs`, exported with the view's names:

```rust
/// The sign-in screen is what the view draws: the link signed out and no sign-in from a link under way.
pub fn sign_in_drawn(root: &Domain, tui: &Tui) -> bool;
/// The sign-in has not landed: the link signed out, or `me` not yet read.
pub fn landing(root: &Domain) -> bool;
```

## temper-client-shell-tui

`theme-start`, `README.md`, 6.13 and 6.15:

```rust
pub struct Start { /* ... */ pub scheme: Option<temper_client_tui::Scheme>, /* the terminal's word, or none */ }
```

`run::Settings.scheme` stays `COLORFGBG`'s `Option<Scheme>`; the run's
`Start.scheme` is it, else `Scheme::of_background` of the terminal's
answer, else `None`. The recording's `screen` record writes and reads
`dark`, `light` or `none` (`temper/docs/design/client/tui/journeys.md`,
3.2); its facts' scheme is `Option<Scheme>`.

The host keeps its theme in `host.rs`:

- At `Start` it takes the record keyed `THEME_KEY` out of `Start.kept`
  before the rest reach the root (`ThemeChoice::read` of its bytes,
  Auto with none), and gives the root `Event::LookKept` with
  `choice.scheme(start.scheme)`.
- `fn keep_theme(&mut self, choice: ThemeChoice, duties: &mut Duties)`:
  the choice becomes the host's, the root is given `Event::LookKept` with
  the look it holds but its theme, `choice.scheme(terminal)`, and
  `jig_client::Request::Save` of the one record `THEME_KEY`,
  `choice.print()` is pushed to `duties.asked` after the report's
  routed requests.
- Writes in order: every `Save` and `Erase` leaving in `duties.asked`,
  the root's and the host's, is counted in the order asked; each
  `jig_client::Event::Stored` or `NotStored` heard answers the oldest,
  and only an answer to a root's write reaches the root.

`notice`, `README.md`, 6.13, *The notice's time*: the host keeps the
notice's end, `Option<Time>`; `fn notice(&mut self, notice: tui::Notice)`
tells it and sets the end to `now + limits::NOTICE`; `Host::deadline`
counts the end with its other times; a `Due` at or past it tells
`Told::Notice { notice: None }`.
