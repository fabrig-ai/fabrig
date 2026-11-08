# tui-link-colours: the shapes that cross tasks

Binding for every task that cites it. The design is
`temper/docs/design/client/README.md`, 6.11 to 6.16 and 7.1;
`temper/docs/design/client/tui/journeys.md`, 3.2, 5, 6.4 and 8.2;
`skein/docs/design/ui-screen.md`, 3, 6.1 and 8; `jig/docs/design/client/domain/client.md`,
8.2; `temper/docs/design/protocol/setup.md`, 7.2. Where this note and the design differ,
the design wins and the report says so.

## 1. jig: the sign-in's expiry (`jig-expiry`)

```rust
// jig-client-protocol-link, beside `credential()` (sign_in.rs):
impl Link { pub fn expires(&self) -> Option<skein_lib::Wall>; }
// jig-client (client.rs):
pub struct LinkView<'a> { /* as now */ pub expires: Option<skein_lib::Wall> }
```

Set from `SignedIn`'s `expires` (wall nanoseconds, `Wall::from_nanos`) at both of the link
layer's `SignedIn` answers, the link's sign-in (token or none) and the approval's take;
none at `Held(credential)`; dropped wherever the credential is dropped.
`Signing::SignedIn { party }` keeps its shape.

## 2. skein: the terminal's background (`skein-background`)

```rust
// skein_ui_screen_terminal
pub const ASK_BACKGROUND: &[u8] = b"\x1b]11;?\x1b\\";
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Background { Waiting, Answered(skein_ui_screen::Rgb), Unanswered }
pub struct Settings { pub interval, pub depth, pub most, pub paste, pub background: Option<skein_lib::Duration> }
impl Terminal { pub fn background(&self) -> Background; }
impl Decoder {
    pub fn answers(&mut self);                       // ESC ] begins an answer from now on
    pub fn background(&self) -> Option<Option<skein_ui_screen::Rgb>>; // first 11 answer: readable or not
}
```

`Key` and `Decoder::new` keep their shapes. No skein crate of `fail-stop`, `heap`, `io`,
`json`, `lib`, `scratch`, `shell`, `sim` or `world` changes.

## 3. temper-client-tui (`tui-local`, `tui-colours`)

```rust
/// DIRECTORY/sign-in for one address (README.md, 6.12).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Note { Until(skein_lib::Wall), SignedOut }
impl Note {
    /// `HOST:PORT until NANOS\n` or `HOST:PORT signed-out\n`.
    pub fn print(self, address: &[u8]) -> Box<[u8]>;
    /// ADDRESS's note, or none: another address's, absent, unreadable, past `Note::MOST`.
    pub fn read(bytes: &[u8], address: &[u8]) -> Option<Note>;
    pub const MOST: u32 = 64;
}

/// The sign-in from the engine's link, as the view draws it (README.md, 6.14).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Local { Idle, Signing, Ended }
// Told gains `Local { local: Local }`; Tui gains `pub const fn local(&self) -> Local`.

impl Scheme { pub const fn of_background(background: skein_ui_screen::Rgb) -> Scheme; } // tui-colours
```

`Note`, `Local` and `Scheme` are exported from the crate's root. `NANOS` is decimal, no
sign, no leading zero but `0`; `HOST:PORT` is the host's `Start.address` bytes. The
palette (`theme::palette`) becomes six colours, in this order: `bg`, `ink`, `dim`,
`faint`, `accent`, `good`.

## 4. The host's seam (`host-local`)

```rust
pub struct Start { pub size, pub scheme, pub address, pub local: bool, pub kept, pub credential,
    pub note: Option<temper_client_tui::Note> }
pub enum Report { /* as now */, Local { link: Option<skein_lib::Secret> } }
#[derive(Default)]
pub struct Duties { pub asked, pub keep, pub note: Option<temper_client_tui::Note>, pub address,
    pub local: bool, pub open, pub close }
// host.rs constants
pub const LOCAL_READS: u8 = 3;                       // one need's reads of the file
pub const LOCAL_AGAIN: skein_lib::Duration = 1 s;    // between them
pub const EXPIRY_SLACK: skein_lib::Duration = 60 s;  // an expiry less this is an expiry
```

## 5. The run (`run`)

```rust
pub struct Settings { /* as now, but */ pub scheme: Option<temper_client_tui::Scheme>,
    pub link_file: Option<Box<[u8]>>,                // the engine's link file's path, bytes
    pub note: Option<temper_client_tui::Note>,        // as the binary read it
    pub files: Option<skein_io::kernel::Fd> }         // the TUI directory's parent, a root for `sign-in`
// limits.rs
pub const BACKGROUND: skein_lib::Duration = 200 ms;
```

`Start.local` is `link_file.is_some()`. The note is written as `TUI_DIRECTORY/sign-in`,
`Note::print(address)`.

## 6. The engine's link file (`box-side`, read by `binary-tui`)

```rust
// temper/crates/temper/src/link_file.rs, `pub mod link_file` in lib.rs
pub const MOST: u32 = 4096;
/// The file for a data directory: Linux `RUNTIME/temper/printed-link-KEY`, RUNTIME
/// absolute, else none; macOS `DATA/printed-link`.
pub fn path(data: &[u8], runtime: Option<&[u8]>, linux: bool) -> Option<std::path::PathBuf>;
/// The deployment the environment names, as `temper box` with no flag resolves it.
pub fn deployment(environment: &[(Box<[u8]>, Box<[u8]>)], working: &[u8]) -> Option<Box<[u8]>>;
pub fn keep(path: &std::path::Path, url: &[u8], user: u32) -> Result<(), String>;
pub fn forget(path: &std::path::Path);
pub fn read(path: &std::path::Path, user: u32) -> Option<skein_lib::Secret>;
```

KEY: the first 8 bytes of `skein_lib::sha256::digest(data)`, 16 lowercase hex digits.
The file's bytes: the printed link's URL as printed, then `\n`; `read` gives them without
the line feed. `keep`'s error names the path and why, never the URL.

## 7. The TUI's journey records (`host-local`)

`journeys.md`, 3.2: start facts `local` (no fields) after `address`, `noted until NANOS`
or `noted signed-out` after `credential`; report `found` with the bytes, or `found none`;
duties `note until NANOS` or `note signed-out` after `keep`, `read` (no fields) after
`signed`. Version stays `2`; a recording without `local` or `noted` is a start with
neither.

## 8. The box's block (`box-side`)

The first block's footer gains one last line, on stderr:
`In a terminal: WORD http://HOST:PORT\n`, WORD `printed::running_word`'s, HOST:PORT the
link URL's origin. Fresh links print as now.
