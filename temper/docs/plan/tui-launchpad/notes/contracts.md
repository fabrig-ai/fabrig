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
- **Added after the first release, compatibly:** the task `screen-label` adds
  `StyleLook::label: bool` (true lays a text field's or area's name before its value, as
  now; false lays its value or placeholder alone after its focus columns), and fills
  `temper-client-tui`'s literals with `label: true`.
- **Added with the third batch, compatibly:** the task `screen-entries` adds to
  `StyleLook` `mark: Option<Mark>`, `described: Option<Paint>` and `spaced: bool`, with
  `pub struct Mark { pub glyph: char, pub paint: Paint }` (`Clone, Copy, PartialEq, Eq,
  Debug`), exported from `skein_ui_screen`, and fills `temper-client-tui`'s literals
  with `mark: None, described: None, spaced: false`. `Theme::valid` checks a mark's glyph
  as it checks the focus mark's.
- What temper's theme sets them to is section 3's.

## 2. The view's shared surface (`view-shared`, `view-lists`; `temper/docs/design/client/README.md`, 6.14, *Its words*)

A new public module of `temper-client-view`, `src/shared.rs`, `pub mod shared;` in
`lib.rs`: words and data with no tree, each a function the browser's own builder then
builds from, so that the browser and the TUI say one thing alike. Every name below is
`temper_client_view::shared::NAME`; `Words` is `temper_client_view::words::Words`, `c` is
`jig_client::contract`, `Domain` is `temper_client_domain::Domain`.

```rust
// view-shared
pub use crate::pages::terminal::words::TerminalWord;   // now `pub`, its variants as they are
pub fn say(w: &Words, word: TerminalWord) -> Box<[u8]>;
/// Home's headline as the browser's Home says it (README.md, 3.5, *the headline*), and the
/// wall at which it next changes (a minute's tick of "The last change landed SPAN ago").
pub fn home_headline(w: &Words, name: &[u8], facts: &temper_client_domain::pages::home::Facts,
    now: skein_lib::Wall) -> (Box<[u8]>, Option<skein_lib::Wall>);
/// A chat row's state as Home's Chats row words it (`pages::chats::status_name`).
pub fn chat_state(line: &c::TaskLine, me: &c::Me, w: &Words) -> Box<[u8]>;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NeedKind { Question, Other }
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NeedLine { pub kind: NeedKind, pub words: Box<[u8]> }   // "TITLE: SHORT", as Home's lines
pub fn need_line(need: &temper_client_domain::pages::needs::Need<'_>, w: &Words) -> NeedLine;
pub use crate::pages::chat_line::Run;                 // now `pub`, with `pub` with_capacity, call, result, has_call, is_empty, line
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RunLine {
    pub name: Box<[u8]>,                 // "Listed changes, read 3 changes"; cut at 256 as now
    pub description: Option<Box<[u8]>>,  // "6 s", "1 failed · 29 s", "failed: WORDS · 29 s", "reading change 2…"
    pub failed: u64,                     // calls failed in the run
    pub running: bool,                   // the line of the run under way
}
impl<'a> Run<'a> {
    /// The line `flush` draws, and the run emptied as `flush` empties it; none when empty and
    /// nothing is under way. `flush` becomes `line` then the browser's nodes.
    pub fn line(&mut self, current: Option<&c::CurrentCall>, running: bool, w: &Words,
        terminal: bool) -> Option<RunLine>;
}
/// The running line's words while the browser's terminal draws one (`terminal::chat_running`).
pub fn running(root: &Domain, w: &Words) -> Option<Box<[u8]>>;
/// The picker's closed line, "Claude Opus 5.5 · high" (`choice::picker::closed_line`).
pub fn closed_line(root: &Domain, task: Option<&c::Task>, w: &Words) -> Box<[u8]>;
/// A chat's short heading (`pages::chat::short_title`).
pub fn heading(title: &[u8]) -> Box<[u8]>;
pub fn description(command: temper_client_domain::pages::terminal::Command) -> &'static [u8];
pub fn offered(root: &Domain, task: Option<&c::Task>,
    command: temper_client_domain::pages::terminal::Command) -> bool;
/// A note's words: `Unknown` and `Closed` as the browser's; `Font` as the browser's;
/// `Help` the browser's own table (the TUI writes its own help).
pub fn note(note: &temper_client_domain::pages::terminal::Note) -> Box<[u8]>;

// view-lists
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Choice {
    pub name: Box<[u8]>,                           // "Claude Opus 5.5", "high", "T9"
    pub words: Box<[u8]>,                          // "your subscription · default", a chat's title; may be empty
    pub selected: bool,                            // the current one
    pub act: temper_client_domain::pages::Act,     // what taking it does
}
/// A command's list head: "Model. A change applies from the next turn.", "Effort. …", "Chats".
pub fn list_head(listing: temper_client_domain::pages::terminal::Listing) -> &'static [u8];
/// `/model`'s models with a model; a provider with none (the browser's "Connect a plan or key ›") left out.
pub fn models(root: &Domain, task: Option<&c::Task>, w: &Words) -> Box<[Choice]>;
pub fn efforts(root: &Domain, task: Option<&c::Task>, w: &Words) -> Box<[Choice]>;
/// `/chats`: the person's live chats, newest activity first, the open one left out.
pub fn chats(root: &Domain, w: &Words) -> Box<[Choice]>;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Confirmation { pub head: &'static [u8], pub take: &'static [u8], pub keep: &'static [u8] }
/// `/close`'s or `/start-over`'s list while the browser's terminal draws it (`commands::confirms`,
/// its confirmation not sent); its acts are `Act::Confirm` and `Act::Cancel`.
pub fn confirmation(root: &Domain) -> Option<Confirmation>;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QuestionChoice { pub words: Box<[u8]>, pub recommended: bool }
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Question {
    pub object: c::ObjectKey,
    pub head: Box<[u8]>,                     // its question line, parsed
    pub choices: Box<[QuestionChoice]>,      // in their parsed order
    pub primary: Option<usize>,              // the recommended one's index, as `Parsed::primary`
}
/// The first question the browser's terminal draws under a waiting chat's box
/// (`terminal::question::under`'s choice of card), whatever the look.
pub fn question(root: &Domain, w: &Words) -> Option<Question>;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DelegationLine { pub kind: Box<[u8]>, pub title: Box<[u8]>, pub state: Box<[u8]> }
/// A delegated task's one line as the browser's terminal card says it ("T7 · Change", ...).
pub fn delegation_line(task: &c::Task, w: &Words) -> DelegationLine;
/// A proposal's title as the browser's terminal card line says it; none for another card.
pub fn proposal_line(card: &temper_client_domain::pages::inbox::Card, w: &Words) -> Option<Box<[u8]>>;
/// What the browser draws under its composer for the open page's last send: a chat's start
/// refused on `Home` or `Ask`, a message refused or failed on a chat, the waiting
/// question's answer refused; none otherwise.
pub fn refusal(root: &Domain, w: &Words) -> Option<Box<[u8]>>;
```

- Each browser builder that says the same thing builds from its function: `chat_line::Run::flush`
  from `line`; `terminal::chat_running` from `running`; `terminal::under`'s quiet line from
  `say` and `closed_line`; `commands::under` from `description`, `offered`, `list_head`,
  `confirmation` and `chats`; the picker's terminal lists from `models` and `efforts`;
  `terminal::question::under` from `question`; the terminal's delegation card from
  `delegation_line`. The browser's trees are byte for byte what they were: its tests
  and goldens unchanged are the check.
- **Added for `tui-chat`,** the one writer of `temper-client-view` in its wave:
  `pub fn working(root: &Domain, w: &Words) -> bool`, the browser's `chats::working` for
  the open chat (its status active and its words not begun), which a run's line under way
  and the three dots read.
- `see`'s phrases are `chat_line`'s table (README.md, 3.5, *its calls*), read from
  `calls::touched`'s `Touched::Other { argument }`, the address; no domain change.

## 3. `temper-client-tui` (`tui-surface`; README.md, 6.13 to 6.15)

```rust
// event.rs: `Quit` goes; "q to quit" binds `Command { command: Exit }` until `tui-foot`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Typed { value: Box<[u8]> },                                   // the screen's field: the link, or the box
    SignIn,
    Send,                                                         // the box's Enter
    Command { command: temper_client_domain::pages::terminal::Command },
    Act { act: temper_client_domain::pages::Act },                // a list's choice
    Aside { object: jig_client::contract::ObjectKey },            // the chat's question set aside
    Dismiss,                                                      // Esc on a command's list
}
// state.rs: `Told` gains
    Size { columns: u16 },                                        // at Start and each Resize
    Went,                                                         // a screen opened: the box asks the focus, its value written
    Cleared,                                                      // the box's words changed by the host's act
    Aside { object: Option<jig_client::contract::ObjectKey> },
    Unknown { words: Option<Box<[u8]>> },                         // the TUI's note of a command it doesn't offer
// and `Tui` gains
    pub const fn columns(&self) -> u16;
    pub const fn aside(&self) -> Option<jig_client::contract::ObjectKey>;
    pub fn unknown(&self) -> Option<&[u8]>;
    pub const fn focus(&self) -> u64;                             // the box's focus request count
    pub const fn box_written(&self) -> u64;                       // the box's value's write count
// view.rs
pub fn escape(view: &View) -> Option<Event>;                       // what Esc makes in the tree last built
impl View { pub const fn words(&self) -> &crate::words::Words; }   // the host's for `shared::question`
/// The commands the TUI offers on the root's page, in `Command::ALL`'s order (README.md,
/// 6.14, *The commands*, *Offered*): `shared::offered` narrowed by the screen.
pub fn offered(root: &temper_client_domain::Domain, words: &crate::words::Words)
    -> Box<[temper_client_domain::pages::terminal::Command]>;
// limits.rs: the tree's bound holds a chat page
pub fn limits(domain: &temper_client_domain::Limits, view: &skein_ui::Limits, field: u32)
    -> Option<skein_ui::Limits>;                                   // each field the larger of the screens' and `view`'s
```

- `Tui`'s `revision` moves on every `Told` that changes what it holds, as now. `columns`
  is 0 until told, and a part leaves nothing out for a narrow row on 0.
- `offered`, `escape`, `View::words` and the `Event`s are exported from the crate's root
  with `View`, `build` and `event`.
- **The view's parts:** `src/view.rs` keeps `View`, `build`, `event`, `escape`, signing in
  and the bindings; one module a part, `src/view/launchpad.rs` (`tui-launchpad`),
  `src/view/foot.rs` (`tui-foot`), `src/view/chat.rs` (`tui-chat`), each with `pub(crate)
  struct` of its data (`Launchpad`, `Foot`, `Chat`), `pub(crate) fn read(root: &Domain,
  tui: &Tui, words: &Words, wall: Wall) -> PART` and `pub(crate) fn draw(b: &mut Builder,
  part: &PART, words: &Words, binds: &mut Binds)`, `foot::draw` returning the
  `Option<Event>` Esc makes. A part's data fields are its task's own. `view.rs`:

  ```rust
  pub(crate) enum Body { Launchpad(launchpad::Launchpad), Chat(chat::Chat) }
  pub(crate) struct Signed { pub body: Body, pub foot: foot::Foot }
  /// The signed-in tree from data alone: the page, the body's top, the body, the foot.
  pub(crate) fn signed(view: &mut View, signed: &Signed, out: &mut Queue<Patch>);
  pub(crate) struct Binds { /* List<Event> */ }
  impl Binds { pub(crate) fn bind(&mut self, event: Event) -> skein_ui::Binding; }
  ```

  `build`, signed in: `Body::Chat(chat::read(..))` on `Chat { .. }`, else
  `Body::Launchpad(launchpad::read(..))`, and `foot::read(..)`, then `signed`. A part's
  goldens build `Signed` from data and lay it with `signed`, through
  `src/tests/goldens.rs`'s `check`. Binding tokens: under `look::OWNER`, `Binds`' from 4,
  reset at each build; `event` decodes a bound press to a clone of its `Event`, and the
  box's input to `Typed` and its submit to `Send`; `View` keeps the `Event` Esc makes.
- **The names** (`look.rs`), numbers after today's:

  | Region | Number | | Style | Number |
  |---|---|---|---|---|
  | `tui-top` `TOP` | 0 | | `tui-title` `TITLE` … `tui-refused` `REFUSED` | 0 to 5 |
  | `tui-bottom` `BOTTOM` | 1 | | `tui-name` `NAME` | 6 |
  | `tui-chat-top` `CHAT_TOP` | 2 | | `tui-faint` `FAINT` | 7 |
  | `tui-sections` `SECTIONS` | 3 | | `tui-address` `ADDRESS` | 8 |
  | `tui-working` `WORKING` | 4 | | `tui-byline` `BYLINE` | 9 |
  | `tui-box` `BOX` | 5 | | `tui-head` `HEAD` | 10 |
  | `tui-list` `LIST` | 6 | | `tui-signal` `SIGNAL` | 11 |
  | | | | `tui-star` `STAR` | 12 |
  | | | | `tui-said` `SAID` | 13 |
  | | | | `tui-agent` `AGENT` | 14 |
  | | | | `tui-run` `RUN` | 15 |
  | | | | `tui-run-failed` `RUN_FAILED` | 16 |
  | | | | `tui-dots` `DOTS` | 17 |
  | | | | `tui-running` `RUNNING` | 18 |
  | | | | `tui-end` `END` | 19 |
  | | | | `tui-field` `FIELD` | 20 |
  | | | | `tui-list-head` `LIST_HEAD` | 21 |
  | | | | `tui-option` `OPTION` | 22 |
  | | | | `tui-note` `NOTE` | 23 |
  | | | | `tui-spaced` `SPACED` | 24 |

  Bindings: `LINK` 1 and `QUIT` 2 as now, `QUIT` decoded to `Command { command: Exit }`
  and gone once nothing binds it (`tui-foot`); `BOX` 3 (the box: `Typed`, `Send`); `Binds`
  from 4.
- **The theme** (`theme.rs`), README.md, 6.15's tables: `palette(scheme)` is seven colours,
  `bg`, `ink`, `dim`, `faint`, `accent`, `good`, `sel` in that order, `sel` `#2a2825` dark
  and `#f4f0e9` light; `Theme::active` is `Paint { back: Some(sel), ..Paint::NONE }`; the
  box's region's `Border { corners as tui-top's, across: '─', down: '│', paint: faint,
  title: false, sides: false }`; `tui-address` and `tui-byline` `align: Align::End`;
  `tui-said` `band: true` with its paint `back: Some(sel)`; `tui-field` `label: false`;
  `tui-end` `Paint { reverse: true, .. }`; `tui-said`'s mark `Mark { glyph: '❯', paint:
  dim }`, `tui-agent`'s, `tui-dots`' and `tui-running`'s `✦` in the accent, `tui-run`'s `▸`
  in dim; `tui-option` bold with `described: Some(dim)`; `tui-spaced` `spaced: true`;
  every other look `sides: true`, `align: Align::Start`, `band: false`, `label: true`,
  `mark: None`, `described: None`, `spaced: false` (section 1).
- **Its words** (`words.rs`), the TUI's own, beside today's (their English is
  README.md, 6.14's): `Separator` " · ", `Between` " / ", `Byline` "written by the status
  agent " and a clock, `LeftOff` "Where you left off", `NeedsYou` "Needs you", `Question`
  "?", `OtherNeed` "•", `NoProject` "Setting temper up makes your first project: open
  http://" ADDRESS " in a browser.", `Needing` "N needs you" (one) and "N need you"
  (other), a plural phrase, `NeedingCut` "N+ need you", `AskTemper` "Ask temper",
  `Message` "Message", `Reply` "Reply", `Ended` "/new starts another chat", `Commands`
  "Commands", `Exit` "Close temper", `KeyEnter` "Enter sends", `KeyTab` "Tab completes a
  command", `KeyPage` "PgUp and PgDn read back", `KeyEsc` "Esc closes a list", `KeyClose`
  "Ctrl+C closes temper", `Else` "Tell it something else", `Dots` "…", `Numbered` N ". ".
  `Quit`, `Connected` and `Projects` stay until the task that stops drawing them
  (`tui-foot`, `tui-launchpad`) deletes them.

## 4. The host and the TUI's events (`host-launchpad`; README.md, 6.13; `tui/journeys.md`, 3.3)

- `Start`, `Report` and `Duties` keep their shapes; `limits::Limits` keeps its fields.
- **Navigation:** the root's start stays `Address::Inbox { filter: Filter::ALL }`; right
  after `Start`'s `Event::Start`, `Event::LookKept { look: Look { terminal: true, theme,
  font: Font::DejaVu } }` (`theme` `Theme::Dark` or `Light` by `Start.scheme`). After each
  report's routing, once: while the root's address is the start's and `me` names a
  project on a live link, `Event::Went { place: Place::Address(Home { project }) }`, the
  first project in `me`'s order. A `Duty::Show { address, replace }` is followed in the
  same report as `Went` to it, `Ask { project }` (and any address but `Home`, `Chat` and
  the start's) to `Home` of its project, else the root's. Each `Went` tells `Told::Went`.
- **The theme drawn** is `theme(scheme)` of the root's `pages().look().theme` after each
  report.
- **The events,** README.md, 6.13's table, and their prints in a recording, the
  compact `Debug` of `temper_client_tui::Event` (`format!("{event:?}")`):

  | Event | Print |
  |---|---|
  | `Typed` | `Typed { value: [104, 105] }` |
  | `SignIn` | `SignIn` |
  | `Send` | `Send` |
  | `Command` | `Command { command: Help }` |
  | `Act` | `Act { act: Choice(Model { index: 0, account: Some(3) }) }`, `Act { act: Go { address: Chat { task: 9 } } }`, `Act { act: Confirm }`, `Act { act: Needs(Choose { object: Task(9), words: [67, 111] }) }` |
  | `Aside` | `Aside { object: Task(9) }` |
  | `Dismiss` | `Dismiss` |

  No record kind is added: they are the `key` record's event bytes (`tui/journeys.md`,
  3.2), and a key with `Typed` still marks nothing.
- **Escape:** a `Key::Escape` the key reader makes nothing of is `tui::escape(&view)`'s
  event, given as any event; `Host::event_of` says the same.
- **`Told::Size`** at `Start` and each `Resize`; **`Told::Cleared`** after each act of the
  host's that empties or replaces the box's words (a send, a command, `Dismiss`, the
  question's answer); **`Told::Aside`** for `Aside`, and `None` once the root's question
  differs from the one aside; **`Told::Unknown`** for a command the TUI doesn't offer, and
  `None` at the next send or `Went`.

## 5. The domain's Home and the status entries (`domain-home`; README.md, 3.5, *Home*, *In the TUI*)

- `pages.rs`'s `start_chat`: `Address::Home { project }` as `Address::Ask { project }`,
  through `Chats::ask_chat` with Home's composer words and the page's choice.
- `pages/choice.rs`'s refresh: with no task, `Address::Home { .. }` refreshes against
  `chats::ASKING_TEMPLATE`, as `Address::Ask { .. }` does.
- `pages/terminal.rs`'s `terminal_send`: where the look is the terminal's, a send on
  `Address::Home { .. }` reads command words as on the pages `draws` names; `draws` itself
  is unchanged (the browser's terminal draws no Home).
- No new public item.
- **Status entries:** `temper/docs/design/client/README.status.md`'s nine **Not yet**
  entries of this plan stand at the head of *Gaps and departures*, a blank line between
  two. Each task deletes its own entry's lines alone, never a blank line, so that tasks
  running at once touch no neighbour's lines; `joint`, the last, deletes its own, the blank
  lines left between the plan's entries, and the summary's "beyond the **Not yet**
  entries of the launchpad, its commands and a chat below".
