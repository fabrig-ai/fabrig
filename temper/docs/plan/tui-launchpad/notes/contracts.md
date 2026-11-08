# tui-launchpad: the shapes that cross tasks

The run's binding calls. A name, field, type, word or record given here is spelled as it
is given; one this note doesn't give is the task's own, inside its files. Section 1 is
skein's (`skein/docs/design/ui-screen.md`); sections 2 to 5 are temper's
(`temper/docs/design/client/README.md`, 6.13 to 6.16); section 6 is the journeys'
(`temper/docs/design/client/tui/journeys.md`).

## 0. Decisions every task builds on

- **The launchpad is the client root at `Address::Home { project }`,** the project the
  first of `me`'s projects. Home already reads what the launchpad draws: the status
  agent's brief (`pages::home::facts(root, project, wall).brief()`), what needs the
  person (`facts.needs`, `pages().needs()`), and the person's live chats
  (`pages().chats().live(client)`), newest activity first. The TUI derives none of them
  again.
- **The root holds the terminal's look,** as a browser's root does in its chat terminal
  (`temper/docs/design/client/README.md`, 3.5, *A chat's terminal*): the TUI's host gives
  the root `Event::LookKept { look: Look { terminal: true, theme, font: Font::DejaVu } }`
  right after `Start`, `theme` its start's scheme. So the root's own terminal machinery
  is the TUI's: commands sent whole (`Command::named`), the lists a command opens
  (`pages().listing()`), the notes (`pages().notes()`), a confirmation as a list, and a
  chat's question read while it waits on the person. `/theme` flips `look().theme`, and
  the TUI draws in the scheme it names. The root's `KeepLook` and `Screen` requests are
  the browser's: the TUI's host drops them.
- **The domain's one change:** on `Home { project }`, the composer asks Ask temper, as
  `Ask { project }` does: the choice is refreshed against `chats::ASKING_TEMPLATE`,
  `Act::StartChat` creates the chat through `Chats::ask_chat`, always a new one, and a
  send's command words are the terminal's where the look is the terminal's. The browser's
  Home sends neither `ChatWords` nor `StartChat` and draws no picker, so it is unmoved.
- **The TUI's commands:** on the launchpad `/model`, `/new`, `/chats`, `/theme`, `/help`,
  `/exit`, in the domain's `Command::ALL` order and the browser terminal's words; on a
  chat also `/effort`, `/pause`, `/close` and `/start-over` where the domain offers them.
  Never `/draft`, `/goal` (drafting is left out) or `/font` (the browser's alone). `/new`
  opens the launchpad, whose next question starts a new Ask temper chat; `/exit` closes
  temper, sent whole or pressed, and never reaches the root.
- **Closing:** Ctrl+C anywhere, `/exit`, the input's end and the signals. Once signed in,
  the box has the focus and `q` types; the first screen's "q to quit" goes. The sign-in
  screen stays as it is.
- **The palette gains `sel`,** the browser terminal's (`#2a2825` dark, `#f4f0e9` light),
  for the two bands the frames draw: the person's words and the active option. It is the
  one background the TUI paints, by the user's request of 2026-11-08.

## 1. skein's full-screen shell (`skein/docs/design/ui-screen.md`, 3, 5 and 6.2)

```rust
// skein_ui_screen, theme.rs
pub struct Border {
    pub corners: [char; 4],
    pub across: char,
    pub down: char,
    pub paint: Paint,
    pub title: bool,
    pub sides: bool,      // false: a rule of `across` above and below, no corners, no sides
}
pub enum Align { Start, End }
pub struct StyleLook {
    pub style: Style,
    pub paint: Paint,
    pub align: Align,     // End: the node's lines flush with its room's end; in a border's title, at its end
    pub band: bool,       // the node's rows painted across its region's inner width
}
pub struct Theme {
    pub base: Paint,
    pub focus: Paint,
    pub placeholder: Paint,
    pub active: Paint,    // laid as a band across an options menu's active item
    pub regions: Box<[RegionLook]>,
    pub styles: Box<[StyleLook]>,
    pub glyphs: Glyphs,
}
// skein_ui_screen, focus.rs
pub struct Focus {
    pub node: Option<skein_ui::NodeId>,
    pub caret: u32,
    pub scroll: u32,
    pub honoured: u64,
    pub active: Option<skein_ui::NodeId>,  // the options menu's active item
    pub follow: bool,                      // the body shows its end as it grows
    pub ended: u64,                        // the last log end request honoured
    pub rows: u16,                         // the body's rows in the last frame laid
}
```

- `Align` and `Focus` keep their derives (`Clone, Copy, PartialEq, Eq, Debug`; `Focus`
  also `Default`: `follow` false, `ended` 0, `rows` 0, `active` none).
- The task `screen-looks` adds `Border::sides`, `Align`, `StyleLook::align` and
  `StyleLook::band`; the task `screen-lists` adds `Theme::active` and `Focus`'s four
  fields. Each fills the literals the compiler names in `temper-client-tui`
  (`src/theme.rs`, `src/tests/theme.rs`) with what draws as before: `sides: true`,
  `align: Align::Start`, `band: false`, `active: Paint::NONE`.
- What temper's theme sets them to is section 3's.
