# look-round-16: the shapes that cross its parts

Binding for every task that names them. A later batch adds sections; it changes none above it.

## 1. The kept store (skein-ui-web, `skein/docs/design/ui-web.md`, 10, *The kept store*)

```rust
// skein/crates/skein-ui-web/src/platform.rs, on `Platform`, after `remove`
/// The kept store's records, the browser's local storage (ui-web.md, 10); none by default.
fn read_kept(&self) -> Vec<crate::store::Record> {
    Vec::new()
}
/// Writes one kept record; refused by default.
fn write_kept(&mut self, _key: &[u8], _value: &[u8]) -> bool {
    false
}
/// Removes one kept record.
fn remove_kept(&mut self, _key: &[u8]) {}

// skein/crates/skein-ui-web/src/store.rs
impl Store {
    /// The small store, the tab's session storage, as `new` has always made it.
    pub fn new(prefix: &[u8], bound: Option<u64>) -> Self;
    /// The kept store, the browser's local storage, in the same form.
    pub fn kept(prefix: &[u8], bound: Option<u64>) -> Self;
}
```

- `read_all`, `write` and `remove` keep their signatures and reach the session methods for `new`'s store and the kept ones for `kept`'s. The encoding, the prefix and the bound are the same.
- `Browser` implements the three over `window.local_storage()`; every other platform keeps the defaults.

## 2. Full screen (skein-ui-web, `skein/docs/design/ui-web.md`, 11, *Full screen*)

```rust
// skein/crates/skein-ui-web/src/screen.rs
//! Full screen, one effect's part (skein/docs/design/ui-web.md, section 11).
#[cfg(target_arch = "wasm32")]
pub mod web;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// The page went into full screen, its Escape held where the browser lets it.
    Entered,
    /// The page left full screen, by the browser or by `leave`; once for each `Entered`.
    Left,
    /// A request the browser refused, or a page with no full screen.
    Refused,
}

/// The report a change of the document's full-screen element makes, from `was` to `now`.
#[must_use]
pub fn changed(was: bool, now: bool) -> Option<Report>;

// skein/crates/skein-ui-web/src/screen/web.rs
pub struct Screen<R> { /* the inbox, the wrap, whether the page is in full screen */ }
impl<R: 'static> Screen<R> {
    /// Listens for the document's `fullscreenchange`, once, for the page's life.
    #[must_use]
    pub fn new(inbox: Rc<Inbox<R>>, wrap: fn(Report) -> R) -> Self;
    /// Asked in the turn of a person's act: `requestFullscreen`, then, once in, Escape locked.
    pub fn enter(&self);
    /// Leaves full screen where the page is in it; nothing otherwise.
    pub fn leave(&self);
}
```

- `changed(false, true)` is `Entered`, `changed(true, false)` `Left`, and no change `None`.
- The keyboard is locked on `Entered` by `navigator.keyboard.lock(['Escape'])`, reached by reflection where the browser has it, its rejection ignored; unlocked on `Left`.

## 3. A field's options (skein-ui and skein-ui-web, `skein/docs/design/ui.md`, 3.2, *options*; `skein/docs/design/ui-web.md`, 6 and 7)

```rust
// skein/crates/skein-ui/src/tree.rs, on `State`, after `ToLogEnd`
/// A menu offering the choices of the nearest text field or area before it in its form.
Options,
```

- An owner builds it as any menu, in the form that holds its field, after the field:

  ```rust
  b.open(Role::Menu);
  b.name(b"Commands");
  b.state(State::Options);
  for each choice {
      b.open(Role::MenuItem);
      b.name(choice_words);          // what Tab writes into the field, for a completion
      b.description(its_help);       // optional, shown beside it
      if it_is_the_current_one { b.state(State::Selected); }
      b.bind(binding);               // pressed by Enter, a click or the text shell's number
      b.close();
  }
  b.close();
  ```

- The builder asserts `Options` on a `Menu` alone; `outline::check` faults a menu with `Options` whose innermost form holds no text field or text area before it.
- The web page keeps the active option (`Page`, beside `menu`), never the tree. An owner learns of it only through the item's `Press` or the field's `Input` that Tab's completion reports.
- The text shell prints it as any menu; the worker's platform has nothing to do for it.

## 4. The terminal's look (temper-client-domain, `temper/docs/design/client/README.md`, 3.5, *A chat's terminal*)

Made by `terminal-frame`; later tasks add to the module, as 5 and 6 say, and change none of this.

```rust
// temper/crates/temper-client-domain/src/pages/terminal.rs
//! A chat's terminal: the look a person chose, the same on every chat the browser opens
//! (temper/docs/design/client/README.md, 3.5, A chat's terminal).

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Look {
    pub terminal: bool,
    pub theme: Theme,
    pub font: Font,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Font {
    #[default]
    DejaVu,
    Plex,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    /// ">_": the chat's pages in the terminal.
    Enter,
    /// "Exit", `/exit`: the browser's look again.
    Leave,
    /// `/theme`: the other theme.
    Theme,
    /// `/font`: the other font.
    Font,
}

/// Whether the terminal draws the page at this address: a chat's, `NewChat` or `Configure`.
#[must_use]
pub fn draws(address: &crate::Address) -> bool;

// temper/crates/temper-client-domain/src/pages.rs
pub mod terminal;
pub enum Act { /* … */ Terminal(terminal::Act) }
impl Pages {
    /// The look a person chose (README.md, 3.5, A chat's terminal).
    #[must_use]
    pub fn look(&self) -> terminal::Look;
}
```

- `Pages` holds the `Look`, starting at its default; `Act::Terminal` sets it and changes nothing else in this part.
- The view draws the terminal where `look().terminal && terminal::draws(address)` and its shell is the browser (`crate::words::Shell::Browser`); it decides the shell, the domain never does.

## 5. The view's terminal (temper-client-view)

Made by `terminal-frame`; `terminal-commands`, `terminal-draft` and `terminal-cards` draw into it.

```rust
// temper/crates/temper-client-view/src/pages/terminal.rs
//! A chat's pages as a terminal (temper/docs/design/client/README.md, 3.5, A chat's terminal;
//! client/browser.md, 2.5, A chat's terminal).
mod words;   // the terminal's words, its own table, as `cards/words.rs` is

/// Whether this view draws the terminal: the browser, the look's `terminal`, a page it draws.
pub(crate) fn shown(root: &Domain, w: &Words) -> bool;
/// The page's main's tokens: TERMINAL, and TERMINAL_LIGHT or TERMINAL_PLEX by the look.
pub(crate) fn style(b: &mut Builder, look: Look);
/// The top line: where, the heading, the number; "Draft" (or "+ Draft") and "Exit".
pub(crate) fn top(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, at: Top<'_>);
pub(crate) struct Top<'a> { pub project: Option<u32>, pub heading: &'a [u8], pub name: &'a [u8], pub number: Option<u64>, pub draft: Draft }
pub(crate) enum Draft { Shown, Hidden, Add, None }
/// The running line above the box, while the chat works or waits (README.md, 3.5).
pub(crate) fn running(b: &mut Builder, words: Option<&[u8]>);
/// Below the box: its list where one is open (`terminal-commands`), else the quiet line.
pub(crate) fn under(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words, bindings: &mut Bindings, hint: Hint);
pub(crate) enum Hint { Commands, Paused }
/// The terminal's notes after the conversation (`terminal-commands` fills them).
pub(crate) fn notes(b: &mut Builder, root: &Domain, w: &Words);
```

- `pages::session::to_terminal(b, w, bindings)` draws ">_" in a browser's bar, bound to `Act::Terminal(terminal::Act::Enter)`; `chat.rs`, `new_chat.rs` and `configure.rs` call it where *Entered* says.
- The look tokens are `browser.md`, 2.5, *A chat's terminal*'s, in `look.rs` under `TERMINAL`, `TERMINAL_LIGHT`, `TERMINAL_PLEX`, `TERMINAL_TOP`, `TERMINAL_WHERE`, `TERMINAL_STAR`, `TERMINAL_PLACE`, `TERMINAL_SEP`, `TERMINAL_TASK`, `TERMINAL_ACTS`, `TERMINAL_BTN`, `TERMINAL_RUN`, `TERMINAL_LINE`, `TERMINAL_HINT`, `TERMINAL_HEAD`, `TERMINAL_NOTE` and `TO_TERMINAL`, their names `temper-terminal`, `temper-terminal-light`, … and `temper-to-terminal`.
