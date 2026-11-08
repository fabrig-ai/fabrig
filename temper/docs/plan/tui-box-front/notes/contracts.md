# tui-box-front: the contracts between tasks

Binding for every task of the plan: the names, shapes and wire forms that
cross a task's boundary. Build to them; a change to one is a question. The
design they come from: skein's `skein/docs/design/shell.md`, 6.3; jig's
`jig/docs/design/protocol/client.md`, 4.2 and section 8;
`temper/docs/design/protocol/setup.md`, 7.1 and 7.6;
`temper/docs/design/client/README.md`, 4.6 (`Forges`), 6.11, 6.13 and 6.17;
`temper/docs/design/client/tui/journeys.md`, 3.2 and 5.

## 1. skein-shell: the terminal's foreground (task `skein-foreground`)

```rust
// skein/crates/skein-shell/src/terminal.rs, re-exported at the crate root
/// Opens the process's controlling terminal, `/dev/tty`, for reading and
/// writing, close-on-exec, never making it a controlling terminal
/// (shell.md, 6.3). The caller owns the descriptor.
pub fn open_terminal() -> Result<Fd, Error>;
/// Makes the caller's process group the foreground group of `terminal`,
/// with SIGTTOU blocked for the call, so that a background group may take
/// it (shell.md, 6.3).
pub fn take_foreground(terminal: Fd) -> Result<(), Error>;
```

Linux and macOS alike. Neither is step code's: the binary calls them, as it
calls `raw_input`.

## 2. jig's clients' component (task `jig-sign-in`)

`jig_protocol_engine_clients` (`boundary.rs`, `component.rs`):

```rust
pub struct Settings {
    // ...as on main, then beside `link_secret`:
    /// The local link's first secret, drawn by the startup and given here
    /// (protocol/client.md, 8, *A secret given*).
    pub local_secret: Option<Secret>,
}
pub enum Unusable { Limits, Seed, LinkSecret, LocalSecret }
pub enum Request {
    // ...as on main, then after `LocalLink`:
    /// The printed link signed a person in: a sign-in made or kept by a
    /// printed link's secret, each time (protocol/client.md, 8).
    LinkSignedIn,
}
```

- `local_secret` is taken at start in the first local link's draw's place:
  its `link_span` runs from that taking, no `LocalLink` is said for it, and
  all of section 8 holds of it as of a drawn one; the next local link, after
  it is spent, is drawn and said as `LocalLink`. Any length but 32 bytes,
  or a secret equal to the given `link_secret`, is `Unusable::LocalSecret`.
  Where the printed link isn't enabled it is dropped, as `link_secret` is.
- `LinkSignedIn` is said when people answers signed in to a sign-in whose
  reserved secret was the printed link's (made, or a held sign-in kept),
  never for the local link, a `Mint`, a service's sign-in, an approval's
  take or any refusal. It carries nothing: no party, no secret.
- Every exhaustive `match` on `Request` gains the arm, as the compiler
  forces, in jig and temper alike; temper-service's arm does nothing until
  task `serve-sign-in` gives it its body.

## 3. The forges document says the printed link was used

- **The wire** (task `forges-linked`): `temper-client-protocol-wire`'s
  `Forges` record gains `linked: bool`, after `forges`
  (`schema/client-wire-v1.schema`, `versioned record Forges { forges:
  list 16 ForgeRow linked: bool }`, and its generated code). Meaning: the
  printed link has signed a person in since the engine started; the
  clients' component's memory alone, so false after a restart
  (`temper/docs/design/client/README.md`, 4.6).
- **The engine** (task `serve-sign-in`): `temper-service` keeps `linked`,
  false at start, true from the first `Request::LinkSignedIn`, and marks the
  forges document so that its watchers are delivered again
  (`temper-engine-domain`'s `mark_forges`); the document's encoding carries
  it.
- **The client** (task `domain-front`): `temper-client-domain`'s forge side
  keeps it with the rows:

  ```rust
  impl Side { pub fn linked(&self) -> Option<bool> } // reached as root.forge().linked(); None until the forges watch delivered
  // pages/welcome.rs
  /// The person signed in by the printed link: the forges document says so,
  /// or the welcome's decision is past its first step, or `me` names a
  /// project (temper/docs/design/client/README.md, 6.17).
  pub fn signed_in(root: &Domain) -> bool;
  ```

## 4. serve's standard input under the box (tasks `serve-sign-in`, `box-front`)

`temper internal serve DIRECTORIES --box` reads its standard input to its
end before it starts anything: none, 32 bytes (the printed link's first
secret) or 64 bytes (the printed link's first secret, then the local link's
first secret); any other length refuses its start,
`--box: stdin: expected the first link's secret`. The first 32 bytes become
jig's `link_secret`, the next 32 its `local_secret`
(`temper-service`'s `Seeds.local_secret: Option<Secret>`, beside
`link_secret`). The box writes 64 bytes when its TUI is in front, 32 when
not (`temper/docs/design/protocol/setup.md`, 7.1 and 7.6).

## 5. The front: the box's TUI child (tasks `box-front`, `binary-front`)

- **Its command:** `temper internal tui --box http://HOST:PORT`, HOST:PORT
  the printed link's own origin. On Linux the box spawns it through
  `temper internal forge-child --parent BOX --signal terminate --`, as every
  step; on macOS directly. Its environment is the box's.
- **Its descriptors:**

  | fd | What | Way |
  |---|---|---|
  | 0 | `/dev/null` | |
  | 1, 2 | pipes the box reads into the front's own log, `<data>/box/logs/tui.log`, as a step's lines, and into its tail | out |
  | 3 | the local link's URL, written whole by the box, then closed | in |
  | 4 | the box's lines (section 6), until the box exits | in |

- **Its terminal:** the front opens the controlling terminal
  (`skein_shell::open_terminal`), takes its foreground
  (`skein_shell::take_foreground`), and runs the TUI on that descriptor, its
  input and its output, as bare `temper` runs on 0 and 1. The box gives it
  no descriptor of its terminal.
- **The local link's URL** on fd 3 is made as the printed link's
  (`temper_service::clients::first_link_url`), at most 4,096 bytes, from the
  local link's first secret, which the box also writes after the printed
  link's on serve's stdin (section 4). Never argv, the environment or a file.
- **The box's side:** its exit, stop and give up with a front are
  `setup.md`, 7.6's. The front's exit is the box's stop, as a first
  termination signal's.

## 6. The box's lines (tasks `box-lines` writes, `tui-surface` reads)

One line each, a word, one space, a field that runs to the line feed (bytes,
spaces allowed, no escapes), then a line feed. A line is at most 8,192 bytes
with its line feed (`temper_box::front::LINE_BYTES` and
`temper_client_tui::BoxLine::MOST`): the box cuts a `log` field to fit at a
UTF-8 boundary, and the reader drops any line it can't read (an unknown
word, no field, past the bound).

| Line | Field | When |
|---|---|---|
| `link URL` | the printed link's URL, no OSC 8 | first, the first block's link; then each fresh link serve prints |
| `log TEXT` | one line as the box writes it, its time first, without its line feed | each line the box writes to `box.log`, and each line its terminal would carry without the front: a group's failure, a restart, a give up, each line of a failure's tail and its `Log: PATH` |
| `restarting STEP` | the step's name, as the terminal's lines name it (`forgejo-3`) | a long step exited and will be restarted (`Fact::Restarting`) |
| `running STEP` | the same | a step the box restarted has been spawned again |
| `gave-up STEP` | the same | the box gives up (`Fact::GivingUp`) |

The box never writes a `link` to a log, and no other line holds a secret.

## 7. The TUI's surface (task `tui-surface`)

`temper-client-tui`:

```rust
// box_line.rs, re-exported at the crate root as `BoxLine`
pub enum BoxLine {
    Link { url: skein_lib::Secret },
    Log { line: Box<[u8]> },
    Restarting { step: Box<[u8]> },
    Running { step: Box<[u8]> },
    GaveUp { step: Box<[u8]> },
}
impl BoxLine {
    pub const MOST: usize = 8192;
    /// One line without its line feed; None where section 6 drops it.
    pub fn read(line: &[u8]) -> Option<BoxLine>;
    /// The line without its line feed, as `read` takes it back.
    pub fn print(&self) -> Box<[u8]>;
}

// state.rs
pub enum Told { /* ...as on main, then: */ Front, BoxLine { line: BoxLine }, Quitting, Stay, Log }
pub enum BoxState<'a> { Running, Restarting { step: &'a [u8] }, GaveUp { step: &'a [u8] } }
impl Tui {
    pub const fn front(&self) -> bool;                     // in front of a box
    pub const fn printed(&self) -> Option<&skein_lib::Secret>; // the box's latest printed link
    pub fn box_log(&self) -> impl Iterator<Item = &[u8]>;  // the box's last lines, oldest first
    pub fn box_state(&self) -> BoxState<'_>;
    pub const fn quitting(&self) -> bool;                  // the quit question stands
    pub const fn log_shown(&self) -> bool;                 // /log's note stands
}

// event.rs
pub enum Event { /* ...as on main, then: */ Stop, Stay, Log }

// view.rs, re-exported at the crate root
/// Whether stopping the box would stop work under way, as the root reads it.
pub fn work_runs(root: &temper_client_domain::Domain, tui: &Tui) -> bool;
```

- `Told::BoxLine`, not `Told::Boxed`: `tui-launchpad-2`'s `look` adds a
  unit `Told::Boxed` (on a box) to the same enum.
- `Told::Front` marks the TUI in front of a box. `Told::BoxLine`: `Link`
  replaces the printed link; `Log` appends, the oldest dropped past
  `limits::BOX_LOG` (50); `Restarting` sets that step restarting; `Running`
  clears it for that step; `GaveUp` sets the box given up, for good.
  `Told::Quitting` raises the quit question; `Told::Stay` lowers it;
  `Told::Log` stands `/log`'s note until the next send or screen, as the
  unknown command's note. Each changes the revision when it changes the
  state.
- `Event::Stop` is "Stop the box" taken; `Event::Stay` "Don't quit" taken,
  or Esc on the question; `Event::Log` is `/log` pressed or sent, a command
  of the TUI's own that never reaches the root.

## 8. The host's seam (task `tui-seam`)

`temper-client-shell-tui`:

```rust
// host.rs
pub struct Start { /* ...as on main, then: */ pub front: bool }
pub enum Report { /* ...as on main, then: */ Front(temper_client_tui::BoxLine) }
// run.rs
pub struct Settings { /* ...as on main, then: */ pub front: Option<Front> }
pub struct Front {
    pub link: skein_lib::Secret, // the local link's URL the box handed on fd 3
    pub lines: skein_io::kernel::Fd, // the box's lines, fd 4
}
```

- `Start.front` is `settings.front.is_some()`; with it `Start.local` is true.
  The run answers the first duty `local` with `Report::Local { link:
  Some(link) }`, the handed link, and every later one as today, from the
  engine's link file or none.
- In front, the host tells the TUI `Told::Front` at `Start`, signs in from
  the handed link whatever credential it holds, and signs in by it with
  `held` the credential it holds, so that a held sign-in is kept.
- Each line the run reads from `lines` is `Report::Front(line)`, told as
  `Told::BoxLine`. The lines' end (the box gone) closes the run, as SIGTERM.
- In front, Ctrl+C and `/exit`: the question standing, close; else, work
  running (`work_runs`), `Told::Quitting`; else close. `Event::Stop` closes;
  `Event::Stay` is `Told::Stay`; `Event::Log` is `Told::Log`.
- **The recording** (`tui/journeys.md`, 3.2 and 5): the start fact `front`
  (no field) after `local`; the report record `box-line` with the line's
  bytes (`BoxLine::print`), replayed through `BoxLine::read`; a `box-line`
  record's `link` URL's secret is a *link* original, as `found`'s is.

## 9. The binary (task `binary-front`)

- `temper box [DIRECTORIES] [--echo none|steps] [--no-tui]` and `temper box
  [DIRECTORIES] --fake-backends [--bundle BUNDLE] [--no-tui]`.
- **The TUI in front** when all hold: no `--no-tui`; echo `none`; fds 0 and
  1 terminals (`skein_shell::terminal_modes`); `open_terminal` succeeds; and
  the box draws a first link (`box_command::first_link` gives one).
  Otherwise the box runs as today, byte for byte.
- `temper internal tui --box http://HOST:PORT` (a `Who::Spawned` row of
  `commands.rs`) reads fd 3 to its end (at most 4,096 bytes), opens and takes
  the terminal, and runs the TUI with `run::Settings.front` set, its
  directory, note, scheme and depth as bare `temper`'s; its exit as bare
  `temper`'s.
