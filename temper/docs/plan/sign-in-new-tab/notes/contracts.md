# sign-in-new-tab: contracts

The shapes that cross this plan's tasks, as its binding calls. The design is `temper/docs/design/client/README.md`, 3.5, `ConnectModel`, *The sign-in's tab*, *Its end* and *A reload while it waits*; section 7; and `skein/docs/design/ui-web.md`, section 10.

## The root: `temper-client-domain`

```rust
// src/boundary.rs
/// What the shell does with a sign-in's tab (client/README.md, 3.5, `ConnectModel`, The sign-in's tab).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tab {
    /// In the press's own turn, before the sign-in is carried; the provider names its words.
    Open { provider: u32 },
    /// The answer's address, its opener cut: once, from that answer.
    Point { to: Box<[u8]> },
    /// An answer with no address, or one the page has left.
    Close,
}
Request::Tab { tab: Tab }
Event::TabRefused
```

- Every root asks these: there is no browser's root. A shell that opens no tab drops them.
- The press's `Tab::Open` comes out before any `Request::Client` the same event asks.
- `Domain` holds `tab: Option<jig_client::Key>`, the key of the sign-in whose tab is open. `Told::Answered` for that key asks `Point` or `Close`, and clears it. `Event::TabRefused` clears it and sets the page's refusal, so no `Point` is asked.
- `ConnectModel` gains `pub const fn tab_refused(&self) -> bool`, cleared by a new press and by `Cancel`.

## The host: `temper-client-shell-web`

```rust
// src/host.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabAct {
    Open { words: Box<[u8]> },
    Point { to: Box<[u8]> },
    Close,
}
pub struct Duties { /* its fields */ pub tab: Vec<TabAct> }
pub enum Report { /* its variants */ TabRefused }
```

- `Host::duties` maps `Request::Tab`, in order, asking the view for `Open`'s words.
- The shell's `carry` carries `duties.tab` before `transport::carry`. `open_tab` answering `false` raises `Report::TabRefused` in the same turn.
- `Duties::departs()` stays as it is: a tab is no departure.

## The view: `temper-client-view`

```rust
// src/root.rs, beside `act_event` and `href`
/// The line a sign-in's tab holds until its address comes (client/README.md, 3.5).
#[must_use]
pub fn opening_tab(view: &View, root: &Domain, provider: u32) -> Box<[u8]>;
```

- The word is `Word::WelcomeTabOpening`, `Opening `, the plan's name, `’s sign-in…`: "Opening ChatGPT’s sign-in…", its apostrophe and ellipsis as temper's other words have them.
- `sign_in_wait::tab_refused` takes the page: `tab_refused(page: &ConnectModel) -> bool`.

## The recording: `temper-client-shell-web`'s `journey`

```rust
// src/journey/line.rs, each a word of its own in the file
Line::TabOpen(Box<[u8]>)   // tab-open WORDS
Line::TabPoint(Box<[u8]>)  // tab-point ADDRESS
Line::TabClose             // tab-close
Line::TabRefused           // tab-refused, a report, as `Due` is
```

`asked(duties)` writes the tab's lines first, in order. `is_ask` holds the first three; `Report::TabRefused` is read from the fourth.

## The small store: a sign-in's address

```rust
// temper-client-shell-web/src/records.rs
pub const SIGN_IN_KEY: &[u8] = b"sign-in";
pub struct SignIn { pub source: u32, pub address: Box<[u8]>, pub refused: bool }
pub fn encode_sign_in(sign_in: &SignIn) -> Box<[u8]>;   // `0` or `1`, the source in decimal, a space, the address
pub fn decode_sign_in(value: &[u8]) -> Option<SignIn>;

// temper-client-domain/src/boundary.rs
Request::KeepSignIn { source: u32, address: Box<[u8]>, refused: bool }
Event::SignInKept { source: u32, address: Box<[u8]>, refused: bool }
```

- The root asks `KeepSignIn` with an answered sign-in's `Tab::Point`, or in its place when its tab was refused.
- The host carries it as `Asked::Save`, counted in no `writes`, since jig's client asks no such write. It gives `SignInKept` right after `Event::Start`.
- `Request` (the page's form, `pages/accounts/request.rs`) reads the kept address and refusal only while `returned` is its source.
