# The TUI

## 1. The TUI: `temper [ADDRESS]`

The TUI (`../../draft/tui.md`) is a person's full-screen terminal client:
temper's client root, a view of its own and skein's full-screen shell
(skein's `ui/screen.md`), over the native transport `temper-box open` uses.
Its first pass (`../../draft/tui.md`, section 4) signs in from the local
engine's link file, with a pasted link or the browser's approval as
fallback (`../README.md`, 4.1); once signed in it draws, live, a launchpad, the commands
of the browser's chat terminal and a chat read as that terminal reads
it (`pages.md`). `temper-box open` stays the tests' line client
(`../line.md`, 9).

- **Bare `temper`** runs the TUI when stdin and stdout are both
  terminals (skein-shell's `terminal_modes` answers for descriptors 0
  and 1), at the address of the engine's link file its environment
  names, when the binary reads one there (2, *The engine's link, kept
  at rest*), else at the address its directory keeps (2), else at
  `http://127.0.0.1:8367`, the clients' port `temper-box init` prefers
  (`../../../help/reference/commands.md`). Otherwise it prints the line it
  printed before the TUI, `temper: no command; temper --help lists
  them`, and exits 1.
- **`temper ADDRESS`** names the engine's clients' listener,
  `http://HOST:PORT`, with or without a final `/`, HOST an IP address on
  loopback, read as `temper-box open` reads an address (`../line.md`, 1). A box whose
  port fell back to another is named by its address.
- **The link is never an argument,** as `temper ctl`'s isn't
  (`../ctl/README.md`, 6.1): the TUI takes it pasted on its sign-in screen
  (`pages.md`), and refuses it as ADDRESS.
- **One argument at most,** an address: a first word that is no
  address and no command is no command, as before
  (`temper: no command WORD; temper --help lists them`).
- **In front of a box, too:** `temper box` in a terminal runs the TUI
  at once and `temper-box --front` behind it, signed in by a link the box
  hands it (`box.md`).
- **Exit status:** 0 once the TUI closed (4); 2 when its arguments are
  refused, the table's rows about ADDRESS and its count; 1 when it
  cannot start otherwise (`../../cli/README.md`, 4 and 7.4); each with
  one line on stderr. Its refusals name `temper`:

  | Cause | stderr |
  |---|---|
  | ADDRESS given, stdin or stdout not a terminal | `temper: the TUI needs a terminal on stdin and stdout` |
  | ADDRESS not an `http://` address, or HOST not an IP address and port | `temper: not an address: ARG` |
  | ADDRESS a sign-in link | `temper: a sign-in link is never an argument: run temper http://HOST:PORT and paste it` |
  | HOST not on loopback | `temper: the engine's address must be on loopback` |
  | two arguments or more | `temper: invalid arguments` and the usage, `usage: temper [ADDRESS]` |
  | the directory unresolved or refused (2) | `temper: DIRECTORY: ` and what was refused |

  A panic aborts the process (fail-stop), the terminal left as it was
  found (4).
- **The client's binary is `temper`** (`../README.md`, section 2;
  `../../cli/README.md`, 7.4): the table above is its bare form's, and
  an ADDRESS that isn't one is refused, exit 2, as its table's refusals
  are. Its `temper-client-shell` captures arguments, environment and
  working directory once and gives them to the TUI's start. Its
  directory, its `address` and its `sign-in` note are the same files (2)
  whether it runs on an engine or in front of a box (`box.md`), and it
  reads the engine's link file (`../../protocol/binaries.md`).

## 2. Its directory

- **Where:** `$XDG_STATE_HOME/temper/tui` where `XDG_STATE_HOME` is an
  absolute path, else `$HOME/.local/state/temper/tui` (skein-shell's
  `Base::State` for the tool `temper`, then `tui`); with neither, the
  start is refused. It is the TUI's alone, never `temper-box open`'s or
  `temper ctl`'s, and no argument names it.
- **Private,** as `temper-box open`'s (`../line.md`, 1): the binary makes it and its
  missing parents `0o700` and refuses it unless it is a directory of
  this user's with no group or other bits (skein's `runtime/io/files.md`, 3), then
  opens its parent as a root and gives `tui` to
  `jig-client-shell-native`, as `temper-box open` gives DIRECTORY's last
  component.
- **What it holds:**
  - `token`, the bearer credential, and `store`, the small store, both
    the native shell's files, kept as `../README.md`, section 4 says;
  - `address`, the address the TUI last signed in to, `http://HOST:PORT`
    and nothing else, `0o600`. The binary reads it once at start, at
    most 64 bytes; one absent, empty, unreadable or not an address
    reads as none, and the default holds. Once the TUI has closed, if it
    signed in at an address the file did not hold, the binary writes
    the file whole through skein's io (skein-shell's `store_file`, a
    synced temporary replacing it).
  - `sign-in`, the note of how the TUI's last sign-in at an address
    stands, one line, `0o600`: `HOST:PORT until NANOS`, signed in there
    until that wall time in nanoseconds, as the sign-in's answer said
    (jig's `client/domain/client/watches.md`, 2); or `HOST:PORT signed-out`,
    its sign-in there ended before that time, so that only a person's act
    signs it in there again (`pages.md`, *Expired is not signed out*). It is
    `temper-client-tui`'s `Note`, printed by `Note::print` and read by
    `Note::read`. The binary reads it once at start, at most 64 bytes: a
    line for another address than the TUI's, or one absent, unreadable
    or past its bound, reads as none. The run writes it whole at each
    duty `note` (3), through skein-io's file layer on its own io, as
    the recorder saves (`journeys.md`, 3.1): a synced temporary
    replacing it. A write that fails says nothing, and the next note is
    written whole again.
- **One sign-in at a time:** the token is the engine's it was made at.
  Started at another address, the TUI offers it there, the engine
  there refuses it, and the TUI erases it and asks for that engine's
  link (`pages.md`).
- **The engine's link, kept at rest.** While the printed link is
  enabled (`../README.md`, section 4), the running engine, `temper-box serve` alone or under
  `temper box`, keeps its live local link, a one-time secret of its own
  beside the printed link's (jig's `protocol/client/sign-in.md`, *The
  local link*), with the address, in a private file, so that a TUI on
  the same machine signs in with nothing to paste (`pages.md`). The file never
  holds the printed link's secret: a TUI's sign-in spends the file's
  link alone, and the printed link stays live for the person's browser,
  never touched, printed again or spent by a TUI. The file's secret is
  otherwise held only in jig's memory: never on serve's stdout or
  stderr, nor in a log; the file is never in the configuration
  directory, which holds no secret.
  - *Where:* on Linux, `$XDG_RUNTIME_DIR/temper/printed-link-KEY`, where
    `XDG_RUNTIME_DIR` is an absolute path (a tmpfs, never backed up);
    with none, no file is kept and the TUI reads none. On macOS,
    `DATA/printed-link`, in the deployment's data directory. KEY is the
    first 8 bytes of the SHA-256 of the data directory's absolute path,
    as 16 lowercase hex digits. One function, `temper-shell`'s
    `link_file::path`, makes it for the engine and for the TUI.
  - *Its deployment:* the engine keys it by the data directory it runs
    on, however it was named; the TUI by the deployment its environment
    names, resolved as `temper box` with no flag resolves it
    (`../../protocol/configuration/directories.md`): `TEMPER_DATA`, else the
    XDG default. No flag names another: the command line's flags are
    designed with the user later.
  - *Written* whole, `0o600`, a synced temporary replacing it, in a
    directory the engine makes `0o700` with its missing parents and
    refuses unless it is this user's with no group or other bits, as the
    TUI's own (above): the local link's URL, made as serve makes the
    printed link's, then a line feed. Serve writes it at each `LocalLink`
    its startup hears, once its clients' listener is up, and prints none
    of it: at its start, and each time the component draws the next,
    after a TUI spent the last or a sign-in found it expired. A `Link` is
    printed alone and never written, and `temper box`'s first secret,
    the printed link's (`../../protocol/setup/box.md`, 2), is never written
    either. A write that fails is one line on stderr, `temper-box serve: link
    file PATH: WHY`, naming no secret, and serve goes on.
  - *Removed* when serve exits, by any way out it has. An engine killed
    outright leaves it, and its link then grants nothing (below).
  - *Read* as a private file of this user's, at most 4,096 bytes
    (skein-shell's `read_private_file`): absent, unreadable, too large or
    not private reads as none. The binary reads it once at start, for
    bare `temper`'s address (1) and, whatever the address, for
    whether the TUI is on a box; the run reads it at each duty `local`
    (3), for the sign-in (`pages.md`).
  - *On a box:* the file read at start holds a link, as
    `jig-client-protocol-http`'s `printed` reads it, whose `HOST:PORT`
    is the TUI's address: the engine there is this machine's temper box,
    and the TUI names its place "box" (`pages.md`, *The rule above the box*).
    The binary keeps the answer alone, `run::Settings.boxed`, and drops
    the secret. `temper-box serve` run alone writes the same file and reads
    as a box too: the file does not tell them apart. With no file, or a
    link for another address, the TUI is not on a box. The binary opens a separate root for
    the link's parent; the run resolves only its filename beneath that
    root and closes the descriptor on shutdown. A parent that cannot be
    opened reads as no link, without refusing the TUI's state directory.
  - *A stale file grants nothing.* Its address and its secret come from
    one file, written by the engine that holds the listener. A restart
    forgets links, and a spent link is dead (jig's `protocol/client/sign-in.md`
    ): a link that another engine answers, or that none
    answers, signs nothing in, and the TUI falls back (`pages.md`).
  - *The risk, accepted by the user, 2026-11-07:* a live owner link,
    one-time and live at most 2 hours, at rest and readable by any
    process running as the person, the box's own agents among them.
    Those can already read the TUI's credential (`token`, above), so this
    adds little. The file's link is its own, the user's pick of
    2026-11-08, so it adds nothing to the printed link's exposure, and a
    `temper` started before the person opens the printed link leaves that
    link to the browser. *The way back:* the file holds the address
    alone, and the sign-in takes one step, the paste or the browser's
    approval.

## 3. Its host and the seam

- **One seam,** shaped as the browser's host's (`../web/journeys.md`, section
  2): `temper-client-shell-tui`'s `host::Host`. What reaches it is a
  report, what it asks back is its duties, and it lays the frame. The
  loop (4), the client's joint world (`../testing.md`) and the TUI's
  recorder and replay (`../../draft/tui.md`, section 5) all call it, and
  nothing else of it.
- **It holds** temper's client root, `temper-client-domain`'s `Domain`,
  linked, as `temper-box open`'s native client holds it (`../line.md`, 2); the TUI's
  state and view (`pages.md`); the focus; and the frame. It holds no io, no
  file, no clock and no randomness of its own: what it does is a
  function of its reports, its clocks and its seed.

  ```rust
  pub struct Start {
      pub size: skein_ui_screen::Size,
      pub scheme: Option<temper_client_tui::Scheme>,  // the terminal's word, or none (6.15)
      pub background: Option<skein_ui_screen::Rgb>,  // the terminal's answer to OSC 11 (6.15)
      pub address: Box<[u8]>,                         // HOST:PORT
      pub local: bool,                                // an engine's link file to read (6.12)
      pub boxed: bool,                                // on a box (6.12)
      pub kept: Vec<jig_client::Record>,              // the small store's records, `theme` among them (6.15)
      pub credential: Option<skein_lib::Secret>,      // DIRECTORY/token's
      pub note: Option<temper_client_tui::Note>,      // DIRECTORY/sign-in's, for this address
  }
  pub enum Report { Start(Start), Due, Key(skein_ui_screen_terminal::Key), Resize(skein_ui_screen::Size),
      Heard(jig_client::Event), Kept { written: bool }, Opened { opened: bool },
      Local { link: Option<skein_lib::Secret> } }     // the engine's link file, read
  pub struct Duties {
      pub asked: Vec<jig_client::Request>,            // to the native shell, in order, the TUI's `theme` record among them
      pub keep: Option<Option<skein_lib::Secret>>,    // the credential to write; Some(None) erases it
      pub note: Option<temper_client_tui::Note>,      // DIRECTORY/sign-in to write (6.12)
      pub address: Option<Box<[u8]>>,                 // the address signed in to (6.12)
      pub local: bool,                                // the engine's link file to read (6.12)
      pub open: Option<Box<[u8]>>,                    // an address for a browser
      pub close: bool,
  }
  impl Host {
      pub fn new(settings: &Settings, seed: u64, now: Time, wall: Wall) -> Host;
      pub fn clocks(&mut self, now: Time, wall: Wall);
      pub fn report(&mut self, report: Report) -> Duties;
      pub fn frame(&mut self) -> &skein_ui_screen::Frame;
      pub fn tree(&self) -> &skein_ui::Tree;
      pub fn event_of(&self, key: &skein_ui_screen_terminal::Key) -> Option<temper_client_tui::Event>;
      pub fn deadline(&self) -> Option<Time>;
  }
  ```

- **The reports:** `Start`, once and first, with the window's size, the
  scheme the terminal says, or none where it says none, the terminal's
  background where it answered (`theme.md`), the
  address, whether the TUI has an engine's link file to read
  (on Linux, `XDG_RUNTIME_DIR` absolute; on macOS, always), whether it
  is on a box (2, *On a box*), the small
  store's records and the credential as the native shell loaded them,
  and the note as the binary read it for this address (2); `Local`,
  the engine's link file read for the duty `local`, its bytes or none;
  `Due`, the root's
  deadline reached; `Key`, one key as skein's decoder made it (skein's
  `ui/screen.md`, 6.1); `Resize`, the window's new size; `Heard`, one of
  the transport's or the small store's events, as the native shell gives
  them; `Kept`, the credential's write ended, written or not; `Opened`,
  the browser's opener ended, done or not (`../README.md`, 4.1). Any report
  before `Start` is the host's defect.
- **The duties** of each report: the root's requests for the native
  shell, in order, each routed as `temper-box open` routes them (`../line.md`, 2), and
  after them the TUI's `theme` record to save, when the person took a
  theme (`theme.md`, *The theme chosen*); the
  credential to keep, or none to erase it; the note to write, when the
  sign-in's standing changes (`pages.md`); the address signed in to, once, the
  first time the link is live with a party; the engine's link file to
  read, each time the TUI signs in from it (`pages.md`); an address for a
  browser to open, once for each approval asked (`../README.md`, 4.1); and closing, once.
  After closing, every report but `Heard` and `Kept` does nothing, so
  the native shell's last write still lands.
- **The note:** after each report, once the link is live with a party,
  an expiry it names (`link().expires`, jig's `client/domain/client/watches.md`,
  2) that differs from the note's `Until` is a duty to note `Until`
  it. A sign-in judged ended (`pages.md`) is a duty to note `SignedOut`, once.
- **The credential:** after each report the host reads it from
  `jig-client`'s `link()`, through the root. One that differs from the
  one kept is a duty to keep it, before the launchpad is drawn on it;
  none, while the link is signed out by any means (a sign-out
  elsewhere, a sign-in ended, a held credential the engine does not
  know), is a duty to erase it. A sign-in kept by the engine leaves the
  credential as it was, and writes nothing (`../README.md`, section 4).
- **A key** goes through skein's key reader (skein's `ui/screen.md`,
  6.2) on the tree and the focus the last frame laid, so that a key
  acts on what the person saw: an act on a node becomes the TUI's event
  through its view, if the view names one (`pages.md`); the focus moved draws
  again; Ctrl+C closes. Escape, which no node of the TUI's names, makes
  the event the view says while it draws a list, `Dismiss` on a
  command's list and `Aside` on the chat's question, or while it draws a
  left-off chat selected, `LeftOff { task: None }`, and nothing
  otherwise (`temper_client_tui::escape`). Up and Down go first to the
  view (`temper_client_tui::arrow`, with `Arrow::Up` or `Arrow::Down`):
  while it draws the launchpad's left-off chats with the box empty and
  no list under it, they make its `LeftOff` event and skein's key reader
  is not asked (`pages.md`, *Where you left off*); otherwise they go to the
  key reader as any key. `Host::event_of` gives the event a key
  would make, without giving it.
- **The frame:** `Host::frame` lays the view's tree with skein's
  renderer at the window's size, in the theme of the scheme the root's
  look names and the start's background, `temper_client_tui::theme(scheme,
  background)` (*The look*, below; `theme.md`), with the host's focus (skein's
  `ui/screen.md`, section 5). After each report the host has already
  built its view from the root and the TUI's state, so `Host::tree` is
  the tree the next key reads.
- **Its pages:** the root starts at `Inbox { filter: ALL }`, as `temper-box
  open`'s does, and the host moves it:
  - *to the launchpad,* `Went(Home { project })`, the first of `me`'s
    projects in `me`'s order, after the report in which the link is live
    with `me` read and naming a project while the root is still at its
    start's address; until then, and while `me` names none, the root
    stays there and the TUI draws the launchpad with no project (`pages.md`),
    or in front of a box the welcome (`box.md`);
  - *where the root asks:* a `Request::Show` or a replacing one, a chat
    started, a chat chosen from `/chats`, is followed at once as `Went`
    to that address, as the browser's host follows it, within the same
    report; an address the TUI draws no page for, `Ask { project }` after
    Start over among them, is followed to `Home` of its project, or of
    the root's;
  - *by `/new`:* `Went(Home { project })`, the root's project;
  - *back to the start's address,* `Went(Inbox { filter: ALL })`, after
    the report in which `me` no longer names the root's project, the
    playground's reset among them (`box.md`), so that the next project `me`
    names is gone to as at the start.
  Every other duty of the root's but its requests to the native shell,
  `KeepLook` and `Screen` among them, is the browser's and is dropped,
  as `temper-box open` drops them.
- **The look:** right after `Start` the host gives the root `Event::LookKept
  { look: Look { terminal: true, theme, font: Font::DejaVu } }`, `theme`
  the scheme of the theme kept (`theme.md`, *The theme chosen*), and again
  with the scheme of each theme the person takes, so that the root's terminal machinery is the TUI's
  as a browser's chat terminal's is (`../pages/chat/terminal.md`, 1): commands
  sent whole, the lists they open, the notes, a confirmation as a list,
  a chat's question read while it waits on the person. After each
  report, a theme of the root's look other than the one drawn is the
  theme drawn from then on (`theme.md`).
- **The TUI's events** and what the host does with each, by screen (`pages.md`):

  | Event | Does |
  |---|---|
  | `Typed { value }` | while the sign-in screen is drawn, the link's field, as before; else the box, whenever it is drawn (`pages.md`, *Typed before the sign-in lands*): on the launchpad with a project `Act::ChatWords { words }`, keeping the left-off selection while the words begin with "/" and clearing it otherwise; with no project, the welcome and "Connecting" among them, the TUI's command composer; on a chat `Act::MessageWords { words }`. Typed while the theme's list stands, it lowers the list too |
  | `SignIn` | signing in's Enter, as before |
  | `Send` | the box's Enter (`pages.md`, *The box*): before the sign-in lands, the notice "Still signing in" and nothing else, the words kept (`pages.md`, *Typed before the sign-in lands*); words that begin a command the TUI offers press it, as `Command` or `Own`, `/exit` whole as `/quit`; words that begin with "/", hold no white space and begin none, `terminal::Act::Dismiss` and the TUI's notice of an unknown command (`pages.md`, *The notice*); the chat's question standing, `needs::Act::OpenReply`, `Act::CardWords` and `needs::Act::SendReply` for its object, then `Act::MessageWords` with no words; with no project, nothing; else `Act::StartChat` on the launchpad, `Act::Send` on a chat |
  | `Command { command }` | `Exit`, the TUI's `/quit`, closes (4); `New` goes to the launchpad; any other `Act::Terminal(terminal::Act::Run { command })` |
  | `Act { act }` | a list's choice, the root's act as the view bound it: `choice::Act::Model` or `Effort`, `Act::Go`, `Act::Confirm` or `Cancel`, `needs::Act::Choose`; and Enter on the launchpad's empty box with a left-off chat selected, `Act::Go` to its `Chat { task }` (`pages.md`) |
  | `Aside { object }` | "Tell it something else", or Esc or words typed on the chat's question: the TUI sets that question aside (`pages.md`) |
  | `Dismiss` | Esc on a command's list: `Act::Terminal(terminal::Act::Dismiss)`, clearing the left-off selection too |
  | `LeftOff { task }` | Up or Down on the launchpad's left-off rows, `Some(TASK)` the row they select, or Esc there, `None`: the TUI's state keeps the selection (`pages.md`, *Where you left off*) |
  | `Own { command }` | one of the TUI's own commands pressed or sent alone (`pages.md`, *The TUI's own commands*), `Own::Change` or `Ask` leaving its name and a space in the box, `Brief` the note, `Needs` the walk, `Open` the object in focus opened, `Theme` the theme's list opened (`pages.md`, *`/theme`*); `n` on the launchpad's empty box makes `Own { command: Needs }` |
  | `Theme { choice }` | on the theme's list, `Some(choice)` taken: the host draws in it, keeps it (`theme.md`, *The theme chosen*) and gives its notice; `None`, Esc there: the list lowered |
  | `Open { address }` | a page opened through the opener, the whole address the view made: Enter on the selected goal, a sign-off's "Review on FORGE ↗", "Open it on the web" (`pages.md`) |
  | `Walk { object, step }` | a choice on the walk's prompt for the need `object` (`pages.md`, *Walking what needs you*): `Choose { words }`, `needs::Act::Choose`; `Open { address }`, the page opened as by `Open`; `Next`; each then marks the need shown |
  | `EndWalk` | Esc on the walk's prompt: the walk ends |

  `Send` on the launchpad also takes `/change WORDS`, `/ask WORDS` and
  `/chats N` before the root's commands (`pages.md`, *The TUI's own
  commands*), and, with the walk's question set aside, answers it in
  the box's words as a chat's question's are, then marks it shown.
  `Duties.open`, which opens the approval's tab, carries a page's
  address the same way.

  A key that makes `Act`, `Command`, `Aside`, `Dismiss`, `LeftOff`,
  `Own`, `Open`, `Walk`, `EndWalk` or `Theme` marks the frame it acts on
  in a recording, and typing marks none
  (`journeys.md`, 3.3). Words typed on the launchpad, and leaving
  it, clear the selection.
- **The notice's time:** the TUI's notice (`pages.md`, *The notice*) stands
  `limits::NOTICE`, 6 seconds, from the report that gave it, a newer
  one replacing it and its time. The host keeps its end, which
  `Host::deadline` counts among the times it gives, the soonest first;
  a `Due` at or past the end lowers the notice. So a recording keeps it as any turn's deadline
  (`journeys.md`, 3.2), and a replay lowers it at the same turn.
- **The seam's shape:** `Start` carries the start's facts, the
  terminal's background, whether the TUI is on a box and whether it is
  in front of one among them (`box.md`; the user's requests of 2026-11-08);
  `Report` gains `Front` (`box.md`), and `Duties` gains no field. Following a
  `Show`, the look, `/theme`'s list and the theme taken, `/new`, `/quit`
  and the notice's time are
  the host's own steps on the root and the TUI's state, and closing is
  the duty `close` it has; the `theme` record crosses to the run as a
  `Save` among `asked` (`theme.md`, *The theme chosen*), and nothing else new
  crosses.
- **Shaped for the recording now:** a recording is the start's facts,
  then turn by turn the clocks, each report with its duties, each key
  with its event, and a mark for each frame the person saw; a replay
  gives the same reports to a fresh host and finds the same duties and
  frames. The browser to open is a duty now, so that the browser's
  approval (`../../draft/tui.md`, 4.2) and the recording add no field to
  the seam.

## 4. Its loop and closing

- **One host on skein's loop** (skein-shell's `drive`),
  `temper-client-shell-tui`'s `run::Run`: the host (3);
  `jig-client-shell-native`'s `Shell`, its transport and its files, as
  `temper-box open`'s; skein's full-screen terminal on stdin and stdout
  (skein's `ui/screen.md`, section 8); and SIGINT, SIGTERM and SIGWINCH
  from skein-shell's `open_signals` with `Watch::TerminationAndWindow`,
  adopted by an io of its own, which in front of a box reads the box's
  lines too (`box.md`). A completion goes by its tag: the
  terminal's (`Terminal::owns`), the signals' (bit 61), else the native
  shell's.
- **Its settings,** `temper-client-shell-tui`'s `limits`: the client's
  and its connections' from `temper-client-shell-terminal`'s
  `limits::native()`; the TUI's tree's from `temper-client-tui`'s
  `limits`, at least the browser view's tree's (`../view.md`, 4), since the TUI
  draws no more of a chat than the chat's page holds, and at least its
  sign-in's; a frame of at most 200 rows by 500 columns; a paste of at
  most 512 bytes, the link's field's bound, which a paste into the box
  keeps too (`../README.md`, section 5); a frame drawn at most every
  20 ms; the terminal's background, asked at every start, waited for at
  most 200 ms, `limits::BACKGROUND` (`theme.md`).
- **Its run's settings** (`run::Settings`), the binary's: beside the
  engine, the directory, the address, the depth, the size, the limits
  and the opener, the scheme where `COLORFGBG` decided it, or none for
  the terminal's answer to decide (`theme.md`); the path of the engine's link
  file, or none where the platform keeps none (2); whether the TUI is
  on a box, `boxed` (2, *On a box*); and the note the binary
  read (2).
- **Each iteration:** the native shell iterates; once it has loaded the
  token and the small store, and the terminal's background is no longer
  awaited (`Terminal::background`, skein's `ui/screen.md`, section 8),
  `Start`, at the terminal's size, in the scheme chosen, with the
  terminal's background where it answered (`theme.md`) and whether the TUI is
  on a box (2); then the
  credential's write ended, the transport's events, the window's size
  when it changed, the keys, and the root's deadline when it is due,
  each a report whose duties are carried at once: the requests to the
  native shell, the credential kept or erased, the note written (2),
  the address noted for the binary, the engine's link file read and
  reported at once as `Local`, closing. Once anything was reported, the
  host's frame
  goes to the terminal, which draws the newest at its pace. No key is
  taken and no frame drawn before `Start`.
- **Time and the seed** are skein-shell's, as `temper-box open`'s (`../line.md`, 2).
- **Closing:** `/quit` (or `/exit`, `pages.md`, *`/quit`*), Ctrl+C anywhere,
  the input's
  end, SIGINT or SIGTERM; in front of a box, the end of the box's lines
  too, and `/quit` and Ctrl+C ask first while work runs (`box.md`). The TUI
  doesn't sign out. The terminal
  finishes the write it began and writes its `LEAVE`; the native shell
  finishes the small store's last write, then closes its connections
  and files; the binary sets the terminal's modes back, writes the
  address (2) and exits 0. What is pending stays in the small store,
  as `temper-box open`'s does (`../line.md`, 6).
- **Its startup** is the binary's (skein's `ui/screen.md`, section 8):
  the input set raw with the interrupt keys passed as bytes
  (`Keys::Pass`), so that Ctrl+C is a key; then the restoration
  skein-shell's kept modes make, `Kept::on_abort(stdout, LEAVE)`, given
  to skein-fail-stop's hook before the loop and taken back after it,
  before the modes are set back, so that a panic leaves the terminal as
  it was found and its report reaches the person's own screen (skein's
  `../foundation/programming-model.md`, section 2).
- **A directory the native shell's files refuse** after the binary
  checked it: the run closes, its `LEAVE` written, and the binary prints
  `temper: DIRECTORY: cannot open its files` and exits 1.
- **macOS** builds it with skein-shell's and skein-io's portable parts
  alone.

Its tests are `../testing.md`'s; its later passes, `../README.md`, section 5's.

## 5. Open questions

- **The TUI's later passes** (`README.md`, 1 to 3 and `pages.md` and `theme.md` and `README.md`, 4 and `../README.md`, 4.1): beyond the
  launchpad as drawn, its commands, the walk of what needs the person
  and a chat, a design session among them (`pages.md`): talking with a role, `/talk`, `/tell` and the launchpad's team
  lines, T3's (`../../draft/surfaces.md`, 11.1, 11.3 and 11.4); a need
  from elsewhere on a chat, the line above its box; the web's hand-off,
  a page's terminal command to copy; and choosing a project from a
  checkout (`../../draft/tui.md`, sections 2 and 3). Default: refine each
  on the recorded frames before designing it.
- **The launchpad's and a chat's later parts** (`pages.md`): the brief, *Where
  you left off* and Needs you across the person's projects, once the
  root reads Home across them (`../../draft/surfaces.md`, 3); a newest goal
  of the person's that Home's goals read, by priority, does not hold,
  which needs a read of the person's goals by activity; a run opened to its
  rows; earlier pages read as the conversation's head nears, and "Latest
  ↓", once skein's screen reports a log's edge (skein's `ui/screen.md`,
  section 10); a card at the call that made it, and its acts, Accept
  among them; a close that asks its delegates' fates; a new line in the
  box, which a terminal's Shift+Enter cannot ask; a paste into the box
  past the link's 512 bytes. Owner: temper/tui.
  Default: each when the user asks for it on a recorded frame; until then the web does what the TUI leaves out.
- **The TUI's command-line flags** (1 and 2): choosing another
  deployment and overriding local sign-in or the scheme wait for the
  user's command-line design. The environment names the local deployment.
- **On a box, told by the box** (2, *On a box*; `pages.md`, *The rule
  above the box*): in front, the box tells the TUI it is one (`box.md`, *The
  place*); bare `temper` reads the engine's link file, which names an
  engine of this machine, so `temper-box serve` run alone reads as a box
  too, where the user's request of 2026-11-08 names "box" for a temper
  box. Default: the file, for bare `temper`. Owner: temper/tui.
- **The TUI's design location:** `README.md`, sections 1 to 3 and `pages.md` and `theme.md` and `README.md`, section 4 and `../README.md`, section 4.1 stay
  here; moving them under `client/tui/`, as `../../README.md`, section 6
  assigns them, is a later cleanup that must preserve their citations.
- **The TUI after landing** (`README.md`, 1 to 3 and `pages.md` and `theme.md` and `README.md`, 4): on a fresh box, start bare
  `temper` in another terminal before opening the printed link and read
  the launchpad with nothing pasted, then open the printed link in the
  browser and see it sign in, with no fresh link printed (2); check
  renewal after expiry and refusal to renew after a
  sign-out noted before the expiry slack. With no usable engine's link
  file, paste the printed link, then sign in through the browser's
  approval and check refusal and asking again
  (`../../draft/tui.execution.md`, M1 and M2); ask temper a question and
  read the chat as it is written. Check the terminal's own
  colours and background answer in emacs vterm, the band under the
  person's words on its ground and the dim words where no answer comes,
  the left-off chats reached by Up and Down, `/effort` on the launchpad,
  a Markdown list's "•", the chat's title in the rule above the box with
  "box" as its place, and the runtime on
  macOS. Walk the launchpad as drawn against its mockups
  (`tui/whole-story/launchpad.html` and `pair/` beside it): the brief
  and `/brief`, *Where you left off* with a goal among its rows, each
  mark under Needs you and its walk by `n` and `/needs`, `/change` and
  `/ask` with words, `/chats N`, and `/open` opening the browser on the
  page in focus; and a reply's views printed in a chat beside the web's
  drawing of them (`../../draft/surfaces.md`, 11.1, 11.4 and 11.5). The
  user takes the recorded frames, `launchpad-drawn`'s among them, and
  refines the look on them
  (`journeys.md`, sections 9 and 10), and the look requests
  `tui-launchpad` and `tui-chat-title-below` are checked against them
  (`../web/look-requests.md`, sections 4 and 5). Simulated closing, client-world
  stories and a recorded pseudo-terminal establish no real terminal's
  look or macOS runtime.
- **The TUI in front of a box after landing** (`box.md`): run `temper box`
  in a real terminal, Linux and macOS, and check the foreground taken
  and given back, Ctrl+C with and without work, a restart and a give up
  on the quiet line, `/log`, and the welcome's link copied whole from
  its wrapped rows; the look request `tui-box-front` is checked against
  its frames, the light theme among them (`../web/look-requests.md`,
  sections 4 and 5). The user's requests of 2026-11-09 are checked
  there too, in emacs vterm at 150 by 39 on a brand-new box: keys typed
  on the welcome before the sign-in lands drawn at once and kept, Enter
  then saying "Still signing in", the box's words in the terminal's ink
  after an accent "❯", `/theme`'s list with "dark: your terminal doesn't
  say" and the choice starting the next run, `/quit` and `/exit`, and a
  notice standing 6 seconds on the quiet line, in a narrow window too
  (`pages.md` and `theme.md`). Left open: the question counts the TUI's project's
  chats alone, not work in other projects; two steps restarting at once
  show the one last said; a box over SSH is the person's tunnel, with
  no front of its own. Default: so, until the user asks otherwise on a
  recorded frame.
- **The TUI's address kept at closing** (2): the binary writes it once
  the loop has ended, so a TUI killed outright keeps the address it
  read. Default: so, though the run now writes the note at once with
  file io of its own; moving the address there waits for a need.
- **The TUI's focus in reverse video** (`theme.md`): the mark and the accent
  show the focus. Default: so; reverse video is the fallback if the
  frames show they can't, a look request's to ask.
- **A sign-out made in the TUI** (`pages.md`): the first pass has none, so its
  note's `SignedOut` comes from a sign-in ended elsewhere. Default: a
  later sign-out of the TUI's writes the same note.
- **An engine at the same address that never made the TUI's sign-in**
  (`pages.md`): read as a sign-in ended, so the TUI waits for a person's act
  there. Default: so, the safe side, until a deployment's own name
  tells engines apart.
- **Expiry and revocation at the same time** (`pages.md`): the engine answers
  both as signed out. The note's last minute is treated as expiry, so a
  revocation in that interval can renew automatically; an absent or
  unreadable note also permits a local try. Distinguishing these ends
  reliably needs an engine answer that names the reason. Default: the
  note and its minute's slack, with this limit kept explicit.
- **The TUI while offline** (`pages.md`): the launchpad and a chat keep what
  they last read while the state says "Offline". Default: so, its words
  refined on the recording.
