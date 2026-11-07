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
