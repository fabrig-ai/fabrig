# tui-connected: the TUI's own contracts

The shapes the temper tasks of part `tui` share, as binding as
`notes/contracts.md` (whose sections 1 and 6 they extend). The design is
`temper/docs/design/client/README.md`, 6.11 to 6.16. A body may differ
from a sketch; a name, a public type or a signature may not without a
question. The words and the paints below are what the goldens print.

## 1. `temper-client-tui`'s surface

Step code under the subset, `no_std` with `alloc`, `#![forbid(unsafe_code)]`.

```rust
// lib.rs: the TUI's pages on temper's client root (temper/docs/design/client/README.md, 6.11 to 6.16).
pub mod look;     // the TUI's owner, its regions' and tokens' names
pub mod words;    // its own phrases, beside the view's
pub mod theme;    // Scheme, theme
pub mod limits;   // limits
mod state;
mod event;
mod view;
pub use event::Event;
pub use limits::limits;
pub use state::{Refusal, Signing, Told, Tui};
pub use theme::{Scheme, theme};
pub use view::{View, build, event};
```

### 1.1 `look`

```rust
pub const OWNER: skein_ui::Owner = skein_ui::Owner { number: 65_538 };   // 65_536 and 65_537 are taken
pub const TOP: skein_ui::Region = skein_ui::Region { owner: OWNER, number: 0 };     // tui-top
pub const BOTTOM: skein_ui::Region = skein_ui::Region { owner: OWNER, number: 1 };  // tui-bottom
pub const TITLE: skein_ui::Style = skein_ui::Style { owner: OWNER, token: 0 };      // tui-title
pub const HINT: skein_ui::Style = skein_ui::Style { owner: OWNER, token: 1 };       // tui-hint
pub const LIVE: skein_ui::Style = skein_ui::Style { owner: OWNER, token: 2 };       // tui-live
pub const WAITING: skein_ui::Style = skein_ui::Style { owner: OWNER, token: 3 };    // tui-waiting
pub const DOWN: skein_ui::Style = skein_ui::Style { owner: OWNER, token: 4 };       // tui-down
pub const REFUSED: skein_ui::Style = skein_ui::Style { owner: OWNER, token: 5 };    // tui-refused
pub const NAMES: skein_ui::look::Names<'static>;   // prefix b"tui", the names above in token and number order
/// The bindings' tokens, under OWNER.
pub const LINK: skein_lib::Token;   // Token::new(1): the link's field
pub const QUIT: skein_lib::Token;   // Token::new(2): "q to quit"
```

### 1.2 `state`

```rust
/// The TUI's own state beside the client root (client/README.md, 6.14).
pub struct Tui { /* address, field, signing, signed_out, unkept, linked, openings, revision: private */ }
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Signing {
    Idle,
    Refused(Refusal),          // the TUI's own refusal of what was pasted; nothing sent
    Asked,                     // a sign-in by a link sent, no answer yet
    Answered { lost: bool },   // its exchange answered (false) or lost (true), the link still signed out
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Refusal {
    NotALink,
    OtherAddress { address: Box<[u8]> },   // the link's HOST:PORT
}
/// What the host tells the state; each is one step.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Told {
    Typed { value: Box<[u8]> },   // the field's value, as the view's event gave it
    Refused { refusal: Refusal },
    Asked,                        // a SignInLink exchange went down: the field emptied
    Answered { lost: bool },      // that exchange's Reply (false) or Lost (true)
    Link { link: jig_client::Link },   // the link's state after a report
    Kept { written: bool },       // the credential's write ended
}
impl Tui {
    #[must_use] pub fn new(address: &[u8]) -> Tui;      // HOST:PORT; Idle, empty field, one opening
    #[must_use] pub fn address(&self) -> &[u8];
    #[must_use] pub fn field(&self) -> &[u8];
    #[must_use] pub const fn signing(&self) -> &Signing;
    #[must_use] pub const fn signed_out(&self) -> bool;  // signed out after a sign-in, until signed in again
    #[must_use] pub const fn unkept(&self) -> bool;      // the credential's last write failed
    #[must_use] pub const fn openings(&self) -> u64;     // times the sign-in screen opened: its focus request's count
    #[must_use] pub const fn revision(&self) -> u64;     // counts every change, for the view's build
    pub fn step(&mut self, told: Told);
}
```

The steps:

| Told | Does |
|---|---|
| `Typed` | the field takes the value; `Refused` and `Answered` become `Idle`; `Asked` stays |
| `Refused` | `Refused(refusal)`; the field keeps its value |
| `Asked` | `Asked`; the field emptied (one own write) |
| `Answered` | `Answered { lost }` where it was `Asked`; nothing otherwise |
| `Link` not `SignedOut` | `Idle`, the field emptied, linked, `signed_out` false |
| `Link` `SignedOut`, linked before | `Idle`, the field emptied, not linked, `signed_out` true, one more opening |
| `Link` `SignedOut`, not linked | nothing |
| `Kept` | `unkept` is `!written` |

Every step that changes something adds one to the revision.

### 1.3 `event`

```rust
/// What the view makes of a person's act (client/README.md, 6.14).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Typed { value: Box<[u8]> },   // Input on the link's field
    SignIn,                       // Submit on the link's field
    Quit,                         // Press on "q to quit"
}
```

### 1.4 `view`

```rust
pub struct View { /* skein_ui::View, Words, the shown count built: private */ }
impl View {
    #[must_use] pub fn new(limits: &skein_ui::Limits, words: crate::words::Words) -> View;
    #[must_use] pub fn tree(&self) -> &skein_ui::Tree;
}
/// The screen the root's link and `me` and the TUI's state call for (client/README.md, 6.14).
pub fn build(view: &mut View, tui: &Tui, root: &temper_client_domain::Domain, out: &mut skein_lib::Queue<skein_ui::Patch>);
/// The event of an act on a node, reading the view alone: the host's act and its recorder's.
#[must_use] pub fn event(view: &View, node: skein_ui::NodeId, act: skein_ui::Act) -> Option<Event>;
```

`build` builds when the root's `temper_client_domain::shown` or the
TUI's revision moved since the last build, their saturating sum as the
`shown` it gives `skein_ui::View::finish`.

### 1.5 `words`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Word { /* section 3's own phrases */ }
pub struct Words { /* the TUI's phrasebook, the view's Words: private */ }
impl Words {
    /// The TUI's English phrases under OWNER, its holes bounded by the domain's limits, beside the view's words.
    pub fn english(view: temper_client_view::words::Words, domain: &temper_client_domain::Limits) -> Result<Words, skein_ui::phrases::Invalid>;
    #[must_use] pub fn say(&self, word: Word, holes: &[skein_ui::phrases::Hole]) -> Box<[u8]>;
    #[must_use] pub fn view(&self) -> &temper_client_view::words::Words;
}
```

### 1.6 `theme` and `limits`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme { Dark, Light }
/// The TUI's theme for a scheme, valid (client/README.md, 6.15).
#[must_use] pub fn theme(scheme: Scheme) -> skein_ui_screen::Theme;

/// The TUI's tree's bound: either screen at the domain's most (`me`'s projects at
/// `client.domain.projects`, each name at `client.domain.text`) and the link's field at `field` bytes.
#[must_use] pub fn limits(domain: &temper_client_domain::Limits, field: u32) -> Option<skein_ui::Limits>;
```

## 2. The trees

Both screens are one `Page`. `Main` carries the region `TOP`, `ContentInfo`
the region `BOTTOM`; nothing else is placed. "own" is a phrase of
section 3, "view" one of `temper_client_view::words::Word`.

**Signing in**, while `root.client().link().state` is `SignedOut`:

```
Page
  Main [TOP]
    Heading 1          own Title                                  [TITLE]
    Paragraph          view SignedOutMain ("Signed out.")         [DOWN]      only while tui.signed_out()
    Paragraph          own SignIn(address)
    Paragraph          own Paste                                  [HINT]
    TextField          name own Link; entry Secret; SubmitsOnEnter; value tui.field() (written: the state's own writes);
                       placeholder own LinkShape(address); bound LINK; focus request tui.openings()
    Paragraph          the sign-in line, below                    [its token] only where there is one
  ContentInfo [BOTTOM]
    Paragraph          own QuitKeys ("Ctrl+C to quit")            [HINT]
    Status             own State(view SignedOut)                  [DOWN]
```

The sign-in line, from `tui.signing()` and `root.client().link().not_signed_in`:

| Signing | `not_signed_in` | Line | Token |
|---|---|---|---|
| `Idle` | any | none | |
| `Refused(NotALink)` | any | own NotALink | REFUSED |
| `Refused(OtherAddress { address })` | any | own OtherAddress(address, address) | REFUSED |
| `Asked` | any | own SigningIn | WAITING |
| `Answered { lost: true }` | any | own NoAnswer | WAITING |
| `Answered { lost: false }` | `Busy` | own Busy | WAITING |
| `Answered { lost: false }` | `NotReady` | own NotReady | WAITING |
| `Answered { lost: false }` | `LinkUsed`, `LinkExpired`, `LinkUnknown` | view SignInLinkUsed, SignInLinkExpired, SignInLinkUnknown | REFUSED |
| `Answered { lost: false }` | `Full { sign_ins, earliest, link_until }` | view SignInFull (Number, Wall, Wall) | REFUSED |
| `Answered { lost: false }` | none | own Refused | REFUSED |

**The first screen**, any other link state; `me` is `root.pages().me(root.client())`:

```
Page
  Main [TOP]
    Heading 1          own Title                                  [TITLE]
    Paragraph          own Connected(address, me.party.name)  |  own Connecting(address) without me  |  view OutOfDateMain when OutOfDate
    Paragraph          own Projects(names joined by " · ")  |  view NoProjectYet      only with me, not OutOfDate
    Alert              own NotKept                                [REFUSED]   only while tui.unkept()
  ContentInfo [BOTTOM]
    Button             name own Quit ("q to quit"); bound QUIT; shortcut q, Modifier::Plain   [HINT]
    Status             own State(the state's word)                [its token]
```

The state's word and token, as `temper-client-view`'s frame says them:
`Live` with `me`: view Live, LIVE; `Starting`, or `Live` without `me`:
view Connecting, WAITING; `Offline`: view Offline, DOWN; `OutOfDate`:
view OutOfDate, DOWN; `SignedOut`: view SignedOut, DOWN.

## 3. The TUI's own phrases

English, under `look::OWNER`; `{}` is a `Name` hole.

| Word | Text |
|---|---|
| `Title` | `temper` |
| `State` | `● {}` (the view's state word) |
| `Quit` | `q to quit` |
| `QuitKeys` | `Ctrl+C to quit` |
| `SignIn` | `Sign in this terminal to {}` |
| `Paste` | `Paste the link temper printed, then Enter.` |
| `Link` | `Link` |
| `LinkShape` | `http://{}/jig/v1/sign-in/link?secret=…` |
| `NotALink` | `That is not a sign-in link: paste the whole link temper printed.` |
| `OtherAddress` | `That link is for {}. To sign in there, run temper http://{}.` |
| `SigningIn` | `Signing in…` |
| `Busy` | `Signing in: the engine is busy; trying again.` |
| `NotReady` | `Signing in: the engine is starting or stopping; trying again.` |
| `NoAnswer` | `Signing in: no answer from the engine; trying again.` |
| `Refused` | `This link was refused. Use the fresh one temper printed.` |
| `Connected` | `Connected to {} as {}` |
| `Connecting` | `Connecting to {}` |
| `Projects` | `Projects: {}` |
| `NotKept` | `This sign-in could not be kept: the next temper asks for a link again.` |

The view's words it says, `temper_client_view::words::Word`: `Live`,
`Connecting`, `Offline`, `SignedOut`, `OutOfDate`, `SignedOutMain`
(`Shell::Terminal`: "Signed out."), `SignInLinkUsed`, `SignInLinkExpired`,
`SignInLinkUnknown`, `SignInFull`, `NoProjectYet`, `OutOfDateMain`.

## 4. The theme's values

Colours from `temper.css`'s chat terminal (`browser.md`, 2.5):

| | bg | ink | dim | faint | sel | accent | good |
|---|---|---|---|---|---|---|---|
| Dark | `#191816` | `#e8e3da` | `#9a9388` | `#6a655c` | `#2a2825` | `#e08a68` | `#8fc28f` |
| Light | `#fffefd` | `#34312c` | `#7a736a` | `#b5aea3` | `#f4f0e9` | `#b8664e` | `#3f7d4e` |

`theme(scheme)`, each paint `Paint::NONE` but what is named:

- `base`: fore ink, back bg; `focus`: back sel; `placeholder`: fore faint.
- `regions`: `TOP` at `Place::Top`, `Arrange::Lines`, border corners
  `['╭', '╮', '╰', '╯']`, across `'─'`, down `'│'`, its paint fore faint,
  `title: true`, pad 1, paint none; `BOTTOM` at `Place::Bottom`,
  `Arrange::Spread`, no border, pad 1, paint none.
- `styles`: `TITLE` fore accent and bold; `HINT` fore dim; `LIVE` fore
  good; `WAITING` fore dim; `DOWN` fore accent; `REFUSED` fore accent.
- `glyphs`: focus `'❯'`, mask `'•'`.

## 5. `temper-client-shell-tui`'s surface

Ordinary Rust, `binary` profile, its own `clippy.toml` the canonical
`ordinary` variant (`notes/contracts.md`, section 1).

```rust
// lib.rs: the TUI's host and its loop (temper/docs/design/client/README.md, 6.13 and 6.16).
pub mod host;     // notes/contracts.md, section 6
pub mod limits;
pub mod run;      // added by tui-run

// limits.rs
pub const MOST: skein_ui_screen::Size = skein_ui_screen::Size { rows: 200, columns: 500 };
pub const PASTE: u32 = 512;                                        // a paste's most bytes, the link's field's bound
pub const INTERVAL: skein_lib::Duration = skein_lib::Duration::from_millis(20);
#[derive(Clone, Debug)]
pub struct Limits {
    pub native: temper_client_shell_terminal::limits::Limits,      // native()'s: the client's, its connections', io's
    pub view: skein_ui::Limits,                                    // temper_client_tui::limits(&native.client, PASTE)
    pub most: skein_ui_screen::Size,
    pub paste: u32,
    pub interval: skein_lib::Duration,
}
#[must_use] pub fn tui() -> Limits;
#[must_use] pub fn valid(limits: &Limits) -> bool;
// added by tui-run, which has skein_ui_screen_terminal::Settings (screen-terminal's):
#[must_use] pub fn terminal(limits: &Limits, depth: skein_ui_screen_terminal::Depth) -> skein_ui_screen_terminal::Settings;

// host.rs, beside notes/contracts.md, section 6
pub struct Settings { pub limits: crate::limits::Limits }

// run.rs
pub struct Settings {
    pub engine: skein_io::Addr,
    pub directory: Box<[u8]>,                          // the directory's last component, b"tui"
    pub address: Box<[u8]>,                            // HOST:PORT
    pub scheme: temper_client_tui::Scheme,
    pub depth: skein_ui_screen_terminal::Depth,
    pub size: skein_ui_screen::Size,                   // the window's, from raw_input
    pub limits: crate::limits::Limits,
}
pub struct Run { /* the host, the native shell, the terminal, the signals' io: private */ }
impl Run {
    /// # Errors
    /// The descriptor io could not adopt.
    pub fn new(settings: Settings, root: skein_io::kernel::Fd, user: u32, seed: u64,
               input: skein_io::kernel::Fd, output: skein_io::kernel::Fd, signals: skein_io::kernel::Fd)
        -> Result<Run, skein_io::kernel::Fd>;
    /// The address `Duties::address` gave, for the binary to keep (client/README.md, 6.12).
    #[must_use] pub fn signed_in(&self) -> Option<&[u8]>;
    /// The native shell's files refused the directory.
    #[must_use] pub const fn refused(&self) -> bool;
}
impl skein_shell::Host for Run { /* exit: Code(0) once closed and drained, Code(1) once refused and drained */ }
```

## 6. The directory and the binary

- `$XDG_STATE_HOME/temper/tui` (absolute `XDG_STATE_HOME`), else
  `$HOME/.local/state/temper/tui` (absolute `HOME`), else refused.
- Files: `token` and `store`, `jig-client-shell-native`'s; `address`,
  the bytes `http://HOST:PORT` alone, read with
  `skein_shell::read_private_file(.., 64, user)`, written with
  `skein_shell::store_file` (`StoreDirectory::Private { name: b"tui", user }`,
  `Expect::Any`) after `drive` returns.
- The binary's module `temper/crates/temper/src/tui.rs` (`tui-command`):

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum Refused { NotATerminal, NotAnAddress(Box<[u8]>), Link, NotLoopback, Directory(String) }
impl Refused { #[must_use] pub fn line(&self) -> String; }   // client/README.md, 6.11's lines, each ending "\n"
/// `temper ADDRESS`'s argument, through `open::target`: an address on loopback, never a link.
pub fn address(argument: &[u8]) -> Result<std::net::SocketAddr, Refused>;
/// The TUI's directory from the environment.
pub fn directory(environment: &[(Box<[u8]>, Box<[u8]>)]) -> Result<std::path::PathBuf, Refused>;
/// The `address` file's bytes as an address, if they are one.
#[must_use] pub fn kept(bytes: &[u8]) -> Option<std::net::SocketAddr>;
#[must_use] pub fn scheme(environment: &[(Box<[u8]>, Box<[u8]>)]) -> temper_client_tui::Scheme;
#[must_use] pub fn depth(environment: &[(Box<[u8]>, Box<[u8]>)]) -> skein_ui_screen_terminal::Depth;
pub(crate) fn run(address: Option<Box<[u8]>>, clock: &skein_shell::Clock) -> u8;
```

`crate::Command` gains `Tui { address: Option<Box<[u8]>> }`.
