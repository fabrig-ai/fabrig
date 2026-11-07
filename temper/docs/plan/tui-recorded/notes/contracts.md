# tui-recorded: the binding calls

The shapes that cross tasks. Each task builds its half to this note; a
detail it leaves open is the task's, named in its report.

## 1. The TUI's surface (`temper-client-tui`), task `tui-surface`

The design is `temper/docs/design/client/README.md`, 7.1, *The TUI's flow*.

```rust
// state.rs: Tui gains what the root does not hold of the approval.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Browser {
    Opening,   // the host asked the opener; drawn as opened
    Opened,    // the opener exited 0
    None,      // no browser to open, or the opener failed: the address is printed
}
pub enum Told {
    // ...every existing variant, unchanged...
    Browse,                    // the host asked to open the approval's address (a new ask answered)
    Opened { opened: bool },   // the opener's report
}
impl Tui {
    pub const fn browser(&self) -> Option<Browser>; // None until the first Browse of this opening
    pub const fn pasted(&self) -> bool;             // a pasted link was sent since the last Browse
}
// lib.rs re-exports: Browser, approval_address, may_ask_again.

/// The approval page's address, from the TUI's own parts alone:
/// `http://` + address (HOST:PORT) + `temper_client_domain::address::print_approve(Some(code))`.
pub fn approval_address(address: &[u8], code: jig_client::ApprovalCode) -> Box<[u8]>;

/// Whether Enter in the empty Link field asks again: the link is signed out and the
/// approval ended (`not_signed_in` is `Approval(Refused | Expired | Used | Unknown)` or
/// `Full`), was not asked (`asked()` is `NotAsked(..)`), or a pasted link replaced it
/// (`pasted()`); never while `asked()` is `Asking`, or while the take waits unreplaced.
pub fn may_ask_again(tui: &Tui, root: &temper_client_domain::Domain) -> bool;
```

- `Told::Browse` sets `browser` to `Opening` and clears `pasted`;
  `Told::Opened { opened }` sets `Opened` or `None`; `Told::Asked` (a
  pasted link's exchange, as today) sets `pasted`; a new opening of the
  sign-in screen (`Told::Link` to signed out after a sign-in) clears
  both. Each change moves `revision`, as the existing steps do.
- `tui::Event` is unchanged: Enter in the field stays `Event::SignIn`;
  the host decides (section 2).
- **The view's clock and tick** (`temper/docs/design/client/README.md`,
  5.12, *The tick*), for "The code expires in SPAN.":

  ```rust
  pub fn build(view: &mut View, tui: &Tui, root: &Domain, wall: Wall, out: &mut Queue<Patch>);
  impl View { pub fn tick(&self) -> Option<Wall>; } // the next instant a word it drew changes
  ```

  `build` builds again when the root's shown count or the TUI's revision
  moved, or once `wall` has reached the tick; `tui-surface` adds the
  parameter and `tick` returning `None`, and `approval-lines` fills
  them.

## 2. The host's seam (`temper-client-shell-tui`'s `host`), task `approval-host`

```rust
pub enum Report { Start(Start), Due, Key(Key), Resize(Size), Heard(jig_client::Event),
    Kept { written: bool }, Opened { opened: bool } }
// Duties unchanged; `open` is now asked.
```

- **The ask:** after each report, when the root's link is signed out,
  `root.asked()` is `None`, and the TUI's `openings()` differs from the
  opening last asked for, the host steps `Event::AskApproval` and notes
  that opening. Nothing is asked before `Start`, or after closing.
- **The tab:** when `root.asked()` becomes `Asked { code, .. }` for an
  ask not yet opened, the duty `open` is `Some(approval_address(address,
  code))`, once, and the host tells `Told::Browse`.
- **`Report::Opened { opened }`** tells `Told::Opened { opened }`.
- **Enter:** `Event::SignIn` with the field empty and `may_ask_again`
  steps `Event::AskApproval` (the root's `asked` then replaces the take);
  otherwise it is checked as a link, as today.

## 3. The opener (`temper-client-shell-tui`'s `opener` and `run`; `temper`'s `tui`), task `approval-host`

```rust
// temper-client-shell-tui/src/opener.rs
pub struct Opener {
    pub program: Box<[u8]>,          // an absolute path
    pub env: Box<[Box<[u8]>]>,       // NAME=value, the TUI's own environment
}
/// The opener's command: `program` with the address as its one argument, no shell.
pub fn spawn(opener: &Opener, address: &[u8], root: skein_io::kernel::Fd) -> skein_io::kernel::Spawn;

// run.rs
pub struct Settings { /* ...existing fields... */ pub opener: Option<Opener> }
```

- The host's `deadline()` is the earlier of the root's and the view's
  tick, the tick taken from wall to the host's clock (`now + (tick -
  wall)`), so that a `Due` builds the span again.
- The run carries the duty `open`: with no opener, it reports
  `Opened { opened: false }` in the same iteration; else it spawns
  `spawn(..)` on its own io, the child's stdin, stdout and stderr none of
  the terminal's, and reports `Opened { opened }` at its exit, done on 0.
  Closing waits for no opener.
- The binary chooses it (`temper/crates/temper/src/tui.rs`):

  ```rust
  pub fn opener(environment: &[EnvironmentEntry], linux: bool, executable: &dyn Fn(&Path) -> bool)
      -> Option<temper_client_shell_tui::opener::Opener>;
  ```

  none when `SSH_CONNECTION` or `SSH_TTY` is set, or, when `linux`,
  neither `DISPLAY` nor `WAYLAND_DISPLAY` is; else `xdg-open` (Linux) or
  `open` (macOS) in the first absolute directory of `PATH` for which
  `executable` holds; none when none does. `run` calls it with
  `cfg!(target_os = "linux")` and a check of the file's mode.

## 4. The shared redaction and scan (`temper-client-shell-web`'s `journey`), task `originals`

```rust
pub enum Kind { Link, Stored, Typed, Given, Approval, Credential, Bearer }
impl Originals {
    /// The originals of the candidates given, then each given text, as `of` builds them.
    pub fn new(candidates: Vec<(Kind, Box<[u8]>)>, given: &[Box<[u8]>]) -> Originals;
}
// journey.rs re-exports it beside `redact`:
/// Replace every original's forms and digests in one field, as `redact` does in each; the places replaced.
pub fn redact_field(field: &mut Box<[u8]>, framing: Framing, originals: &Originals) -> u32;
// scan.rs, beside `bytes` (which stays, `original || shape`):
pub fn original(bytes: &[u8], originals: &Originals) -> bool;
pub fn shape(bytes: &[u8]) -> Option<Shape>;
```

- `Originals::of` collects its candidates and calls `new`; the browser's
  candidates and their order stay as they are.
- `tools/journeys`' `Added` counts gain the three kinds for a TUI
  journey's `add` (`tui-print`); the browser's counts and their lines
  stay as they are.

## 5. The TUI's file (`temper-client-shell-tui`'s `journey`, feature `record`), task `tui-lines`

The records are `temper/docs/design/client/tui/journeys.md`, 3.2.

```rust
pub enum Line {
    Journey { version: u32 }, Seed(u64), Clocks { now: Time, wall: Wall },
    Screen { size: Size, scheme: Scheme }, Address(Box<[u8]>),
    Stored { key: Box<[u8]>, bytes: Box<[u8]> }, Credential(Box<[u8]>),
    Turn { now: Time, wall: Wall }, Start, Due,
    Key { key: Key, event: Option<Box<[u8]>> }, Resize(Size),
    Reply { exchange: u64, bytes: Box<[u8]> }, Unanswered(u64),
    Streamed(Streamed), Gone, Saved, Unsaved,
    Kept { written: bool }, Opened { opened: bool },
    Send { exchange: u64, message: Sent, credential: Option<Box<[u8]>> },
    Stream(Option<Box<[u8]>>), Unstream,
    Save { key: Box<[u8]>, bytes: Box<[u8]> }, Erase(Box<[u8]>),
    Keep(Option<Box<[u8]>>), Signed(Box<[u8]>), Browse(Box<[u8]>), Quit,
    Deadline(Option<Time>), Mark(Option<Box<[u8]>>), End(Option<Box<[u8]>>),
}
pub enum Streamed { Record(Box<[u8]>), Heartbeat, Full { transport: bool, size: u32 }, SignedOut }
/// A message as sent: its kind and bytes; `SignInService` and `SignOut` hold none.
pub enum Sent { Authorize(Box<[u8]>), ApprovalAsk(Box<[u8]>), ApprovalRead(Box<[u8]>),
    ApprovalDecide(Box<[u8]>), SignInApproval(Box<[u8]>), Request(Box<[u8]>), Read(Box<[u8]>),
    Watch(Box<[u8]>), Unwatch(Box<[u8]>), StreamGone(Box<[u8]>), SignInLink(Box<[u8]>),
    SignInService, SignOut }
impl Sent { pub fn of(message: &jig_client::Message) -> Sent; }
#[derive(Debug, PartialEq, Eq)]
pub struct Unreadable { pub line: u32, pub why: &'static str }
impl Line {
    pub fn write(&self, out: &mut Vec<u8>);   // the line and its newline
    pub fn kind(&self) -> &'static str;       // the record's kind, as the file writes it
}
pub fn lines(recording: &[u8]) -> Result<Vec<Line>, Unreadable>;
```

- `Line`, `Streamed` and `Sent` derive `Clone`, `PartialEq`, `Eq`;
  `Line`'s `Debug`, if any, prints kinds and lengths alone, never bytes.
- An exchange is `skein_lib::Token::raw()`.

## 6. The recorder and the replay (`journey`), tasks `tui-replay` and `tui-record`

```rust
pub struct Facts { pub seed: u64, pub now: Time, pub wall: Wall, pub size: Size,
    pub scheme: Scheme, pub address: Box<[u8]>, pub kept: Vec<jig_client::Record>,
    pub credential: Option<Secret> }
impl Facts {
    /// The host as the run makes it, and the Start report it is given first.
    pub fn start(&self) -> (Host, Report);
}
/// The event the view makes of `key`, as a recording keeps it: its Debug print.
pub fn event(host: &Host, key: &Key) -> Option<Box<[u8]>>;
pub struct Recorder;
impl Recorder {
    pub fn new(facts: &Facts) -> Recorder;
    pub fn turn(&mut self, now: Time, wall: Wall);
    pub fn report(&mut self, host: &mut Host, report: Report) -> Duties;
    pub fn deadline(&mut self, deadline: Option<Time>);
    pub fn mark(&mut self, host: &mut Host);
    pub fn save(&self, host: &mut Host) -> Box<[u8]>;
}
pub struct Mark { pub frame: Frame, pub live: Option<Box<[u8]>>, pub key: Option<Key> }
pub struct Stale { pub turn: u32, pub line: u32, pub recorded: Box<[u8]>, pub asked: Box<[u8]> }
pub enum Refusal { Unreadable(Unreadable), Stale(Stale), Differs { mark: u32, line: u32 } }
pub struct Replay;
impl Replay {
    pub fn new(recording: &[u8]) -> Result<Replay, Unreadable>;
    pub fn next(&mut self) -> Result<Option<Mark>, Stale>;   // on to the next mark, or None at the end
    pub fn host(&mut self) -> &mut Host;
}
/// The first replay's proof: every mark's live print is the replay's; the recording
/// with its marks' prints dropped.
pub fn prove(recording: &[u8]) -> Result<Box<[u8]>, Refusal>;
```

- `Mark.key` is the key recorded after the mark, when the mark was taken
  before one (the print's link); `None` for the mark key's and the end.
- `Stale.recorded` and `asked` are lines as the file writes them; only
  the reprint, whose recordings passed the scan, prints them.
- `run::Settings` gains `pub record: bool` (`tui-record`): true has the
  run make a `Recorder` at `Start` and save to `recording` in its
  directory (`tui/journeys.md`, 3.1); `temper`'s `tui.rs` sets it to
  `cfg!(feature = "record")`.

## 7. The TUI's redaction and scan (`journey`), task `tui-redact`

```rust
pub fn fields_mut(line: &mut Line) -> Vec<(&mut Box<[u8]>, temper_client_shell_web::journey::Framing)>;
pub fn originals(lines: &[Line], given: &[Box<[u8]>]) -> temper_client_shell_web::journey::Originals;
pub struct Redacted { pub recording: Box<[u8]>, pub originals: Originals, pub places: u32 }
pub fn redact(recording: &[u8], given: &[Box<[u8]>]) -> Result<Redacted, Unreadable>;
/// The approval codes as shown, read from each `browse` address's `code=`.
pub fn codes(lines: &[Line]) -> Vec<Box<[u8]>>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place { Field, Frame { turn: u32 }, Printed }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found { pub line: u32, pub record: &'static str, pub place: Place, pub what: What }
/// Originals in the whole, shapes in a copy with each code cut.
pub fn scan_bytes(bytes: &[u8], codes: &[Box<[u8]>], originals: &Originals) -> Option<What>;
/// Every field of every line, then every frame the replay draws at each turn's end.
pub fn scan(recording: &[u8], originals: &Originals) -> Result<Option<Found>, Refusal>;
```

`What`, `Framing`, `Originals` and `Kind` are `temper-client-shell-web`'s
`journey` types (section 4).

## 8. The walk (`journey`), task `tui-walk`

```rust
pub enum WalkStep { See(Box<[u8]>), Key(Key), Mark, Approve, End }
pub fn walk(file: &[u8]) -> Result<Vec<WalkStep>, Unreadable>;
/// The first step, from 1, that the recording does not follow (tui/journeys.md, 8.1).
pub fn follows(walk: &[WalkStep], recording: &[Line]) -> Option<u32>;
/// A save replayed to its end: whether its frame's text holds `text`.
pub fn shows(save: &[u8], text: &[u8]) -> Result<bool, Refusal>;
/// A save's last `browse` address.
pub fn browsed(save: &[u8]) -> Result<Option<Box<[u8]>>, Unreadable>;
```

## 9. The print (`tools/journeys`), tasks `tui-print` and `tui-again`

- `temper_client_journeys::TUI_JOURNEYS`, beside `JOURNEYS`:
  `concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/design/client/tui/journeys")`.
- `temper-client-tui`'s `theme::palette(scheme: Scheme) -> [(&'static str, Rgb); 7]`,
  the names `bg`, `ink`, `dim`, `faint`, `sel`, `accent`, `good` in that
  order (`temper/docs/design/client/README.md`, 6.15), from which
  `theme` takes its colours.
- The program's files are `tui/journeys.md`, 6.2's names.
