# The TUI's host and seam

The TUI's host and seam are split from [the TUI](README.md).

## 1. Its host and the seam

- **One seam,** shaped as the browser's host's (`../web/journeys.md`, section
  2): `temper-client-shell-tui`'s `host::Host`. What reaches it is a
  report, what it asks back is its duties, and it lays the frame. The
  loop (4), the client's joint world (`../testing.md`) and the TUI's
  recorder and replay (`../../draft/tui.md`, section 5) all call it, and
  nothing else of it.
- **It holds** temper's client root, `temper-client-domain`'s `Domain`,
  linked, as `temper-box open`'s native client holds it (`../line.md`, 2); the TUI's
  state and view (`pages/README.md`); the focus; and the frame. It holds no io, no
  file, no clock and no randomness of its own: what it does is a
  function of its reports, its clocks and its seed.

  ```rust
  pub struct Start {
      pub size: skein_ui_screen::Size,
      pub scheme: Option<temper_client_tui::Scheme>,  // the terminal's word, or none (6.15)
      pub background: Option<skein_ui_screen::Rgb>,  // the terminal's answer to OSC 11 (6.15)
      pub address: temper_client_tui::Origin,         // the origin (1)
      pub local: bool,                                // an engine's link file to read (6.12)
      pub boxed: bool,                                // on a box (6.12)
      pub kept: Vec<jig_client::Record>,              // the small store's records, `theme` among them (6.15)
      pub credential: Option<skein_lib::Secret>,      // DIRECTORY/token's
      pub note: Option<temper_client_tui::Note>,      // DIRECTORY/sign-in's, for this address
      pub plain: bool,                                // a terminal that draws no mark (`pages/README.md`, 3)
      pub version: Box<[u8]>,                         // the build, as `temper --version` prints it after "temper "
  }
  pub enum Report { Start(Start), Due, Key(skein_ui_screen_terminal::Key), Resize(skein_ui_screen::Size),
      Heard(jig_client::Event), Kept { written: bool }, Opened { opened: bool },
      Local { link: Option<skein_lib::Secret> },      // the engine's link file, read
      Handshake { failed: Option<temper_client_tui::Handshake> }, // the native shell's last TLS failure
      Listening { port: Option<u16> },                // the loopback listener's port, or none (`pages/sign-in.md`, 4)
      Returned(loopback::Returned) }                  // one request the loopback listener read
  pub enum Handshake { Untrusted, Expired, NotYetValid, OtherName, Failed } // temper_client_tui's
  pub struct Duties {
      pub asked: Vec<jig_client::Request>,            // to the native shell, in order, the TUI's `theme` record among them
      pub keep: Option<Option<skein_lib::Secret>>,    // the credential to write; Some(None) erases it
      pub note: Option<temper_client_tui::Note>,      // the origin's sign-in to write (6.12)
      pub address: Option<temper_client_tui::Origin>, // the origin signed in to (6.12)
      pub local: bool,                                // the engine's link file to read (6.12)
      pub open: Option<Box<[u8]>>,                    // an address for a browser
      pub listen: bool,                               // the loopback listener to open
      pub unlisten: bool,                             // the loopback listener to close
      pub answer: Option<loopback::Answer>,           // the answer to this report's `Returned`
      pub close: bool,
  }
  // temper-client-shell-tui's `loopback` (`pages/sign-in.md`, 4)
  pub struct Returned { pub get: bool, pub target: Box<[u8]>, pub host: Option<Box<[u8]>> }
  pub enum Answer { Redirect { to: Box<[u8]> }, NotFound }
  pub fn grant(returned: &Returned, port: u16) -> Option<skein_lib::Secret>; // the one path's grant, or none
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
  the note as the binary read it for this address (2), whether the
  terminal draws the header's mark, `plain` where `TERM_PROGRAM` is
  `Apple_Terminal`, and the build the header names, `temper-version`'s
  version and commit as `temper --version` prints them after "temper ",
  "0.0.0 (b948f40d6df1)", which the binary hands down, since no library
  links `temper-version` (`pages/README.md`, 3); `Local`,
  the engine's link file read for the duty `local`, its bytes or none;
  `Due`, the root's
  deadline reached; `Key`, one key as skein's decoder made it (skein's
  `ui/screen.md`, 6.1); `Resize`, the window's new size; `Heard`, one of
  the transport's or the small store's events, as the native shell gives
  them; `Kept`, the credential's write ended, written or not; `Opened`,
  the browser's opener ended, done or not (`../sign-in.md`, 1.1);
  `Handshake`, the native shell's last TLS failure changed (*The
  handshake*, below); `Listening`, the loopback listener the duty
  `listen` asked for listening at its port, or none where the run has
  no opener or couldn't listen; `Returned`, one request that listener
  read, its method `GET` or not, its target and its `Host` as sent, at
  most `limits::RETURN_HEAD` (`pages/sign-in.md`, 4). Any report before
  `Start` is the host's defect.
- **The handshake** (`../../draft/tui-remote.md`, 4): at an `https://`
  origin, the run reads the native shell's `Shell::refused` after each
  iteration (jig's `protocol/client/README.md`, section 9) and, when it
  changed, reports it as the TUI words it, `temper-client-tui`'s
  `Handshake`: skein-tls's `Certificate::Issuer` or `Invalid`,
  `Untrusted`; `Expired`; `NotYetValid`; `Name`, `OtherName`; any other
  TLS error, `Failed`; none once a handshake succeeded. The host keeps
  it in the TUI's state, and while it holds a failure the link's state
  says why (`pages/README.md`, *The link's state*). A connection that
  fails below TLS is no handshake's, and says "Offline" alone, as on
  loopback. At an `http://` origin the run reports none.
- **The duties** of each report: the root's requests for the native
  shell, in order, each routed as `temper-box open` routes them (`../line.md`, 2), and
  after them the TUI's `theme` record to save, when the person took a
  theme (`theme.md`, *The theme chosen*); the
  credential to keep, or none to erase it; the note to write, when the
  sign-in's standing changes (`pages/README.md`); the address signed in to, once, the
  first time the link is live with a party; the engine's link file to
  read, each time the TUI signs in from it (`pages/README.md`); an address for a
  browser to open, once for each approval asked (`../sign-in.md`, 1.1); the
  loopback listener to open, for each one-click ask, and to close, once
  its ask ends (`pages/sign-in.md`, 4); the answer to a `Returned`
  report's request, in that report's duties, which the run writes to
  that request's connection; and closing, once.
  After closing, every report but `Heard` and `Kept` does nothing, so
  the native shell's last write still lands.
- **The note:** after each report, once the link is live with a party,
  an expiry it names (`link().expires`, jig's `client/domain/client/watches.md`,
  2) that differs from the note's `Until` is a duty to note `Until`
  it. A sign-in judged ended (`pages/README.md`) is a duty to note `SignedOut`, once.
- **The credential:** after each report the host reads it from
  `jig-client`'s `link()`, through the root. One that differs from the
  one kept is a duty to keep it, before the launchpad is drawn on it;
  none, while the link is signed out by any means (a sign-out
  elsewhere, a sign-in ended, a held credential the engine does not
  know), is a duty to erase it. A sign-in kept by the engine leaves the
  credential as it was, and writes nothing (`../sign-in.md`, section 1).
- **A key** goes through skein's key reader (skein's `ui/screen.md`,
  6.2) on the tree and the focus the last frame laid, so that a key
  acts on what the person saw: an act on a node becomes the TUI's event
  through its view, if the view names one (`pages/README.md`); the focus moved draws
  again; Ctrl+C closes. Escape, which no node of the TUI's names, makes
  the event the view says while it draws a list, `Dismiss` on a
  command's list and `Aside` on the chat's question or a talk's proposal,
  or while it draws a
  left-off chat selected, `LeftOff { task: None }`, or on a talk with its
  box empty, `Leave`, and nothing
  otherwise (`temper_client_tui::escape`). Up and Down go first to the
  view (`temper_client_tui::arrow`, with `Arrow::Up` or `Arrow::Down`):
  while it draws the launchpad's left-off chats with the box empty and
  no list under it, they make its `LeftOff` event and skein's key reader
  is not asked (`pages/launchpad.md`, 4); otherwise they go to the
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
    stays there and the TUI draws the launchpad with no project (`pages/launchpad.md`, 7),
    or in front of a box the welcome (`box.md`);
  - *where the root asks:* a `Request::Show` or a replacing one, a chat
    started, a chat chosen from `/chats`, a role's talk chosen by `/talk`
    (`Talk { project, task }`, `pages/talk.md`), is followed at once as `Went`
    to that address, as the browser's host follows it, within the same
    report; an address the TUI draws no page for, `Ask { project }` after
    Start over among them, is followed to `Home` of its project, or of
    the root's;
  - *from a talk:* Esc on its empty box with no list standing, or `/exit`
    sent whole there, `Went(Home { project })`, the talk's project, which
    leaves the talk, with the notice of a talk left (`pages/talk.md`,
    section 1, *Leaving*);
  - *by `/new`:* `Went(Home { project })`, the root's project;
  - *not for a place not yet started* (`pages/README.md`, 2, *Not yet
    started*): it is the TUI's state over the root's page, which stays
    where it was until the place's first words are sent; the root's
    going to any other page ends it, but the `NewChat` or `Home` its own
    start passes through;
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
- **The launchpad's reads:** right after that first `LookKept` the host
  gives the root `Event::Launchpad`, once, so that the root's Home
  reads across the person's projects (`pages/launchpad.md`, 2). No other shell
  gives it.
- **The TUI's events** and what the host does with each, by screen (`pages/README.md`; `pages/launchpad.md`):

  | Event | Does |
  |---|---|
  | `Typed { value }` | while the sign-in screen is drawn, the link's field, as before; else the box, whenever it is drawn (`pages/README.md`, 1, *Typed before the sign-in lands*): on the launchpad with a project `Act::ChatWords { words }`, keeping the left-off selection while the words begin with "/" and clearing it otherwise; with no project, the welcome and "Connecting" among them, the TUI's command composer; on a chat or a talk `Act::MessageWords { words }`. Typed while the theme's list stands, it lowers the list too |
  | `SignIn` | signing in's Enter, as before; at a name, the sign-in screen's "Ask again" button pressed (`pages/sign-in.md`, 3), which asks the one click again where there is an opener |
  | `TypeCode` | at a name, the sign-in screen's "Type a code instead" pressed while a one-click ask waits (`pages/sign-in.md`, 3): the duty `unlisten`, and a typed code's ask, `Event::AskApproval { port: None }`, for which no tab opens |
  | `Send` | the box's Enter (`pages/README.md`, 1, *The box*): before the sign-in lands, the notice "Still signing in" and nothing else, the words kept (`pages/README.md`, 1, *Typed before the sign-in lands*); words that begin a command the TUI offers press it, as `Command` or `Own`, `/exit` whole as `/quit`, or on a talk as `Leave`; the name of `/draft`, `/change`, `/ask` or `/tell`, where the screen offers it, followed by white space alone, as that name pressed, `Own { command }`; in a place not yet started, words that begin no command as that place's `NAME WORDS` on the screen under it, and empty words or white space alone nothing (`pages/README.md`, 2, *Not yet started*); words that begin with "/", hold no white space and begin none, `terminal::Act::Dismiss` and the TUI's notice of an unknown command (`pages/README.md`, 1, *The notice*); the chat's question standing, `needs::Act::OpenReply`, `Act::CardWords` and `needs::Act::SendReply` for its object, then `Act::MessageWords` with no words; with no project, nothing; else `Act::StartChat` on the launchpad, `Act::Send` on a chat or a talk |
  | `Command { command }` | `Exit`, the TUI's `/quit`, closes (4); `New` goes to the launchpad; any other `Act::Terminal(terminal::Act::Run { command })` |
  | `Act { act }` | a list's choice, the root's act as the view bound it: `choice::Act::Model` or `Effort`, `Act::Go`, `Act::Confirm` or `Cancel`, `needs::Act::Choose`; `Act::Confirm` taken on a chat's `/close` keeps the chat as closed by the person, and the root's answer done to that close goes to the launchpad, `Went(Home { project })`, with the chat closed's notice (`pages/chat.md`, 1, *Closing*); and Enter on the launchpad's empty box with a left-off chat selected, `Act::Go` to its `Chat { task }` (`pages/launchpad.md`, 4) |
  | `Aside { object }` | "Tell it something else", or Esc or words typed on the chat's question: the TUI sets that question aside (`pages/README.md`); "Not now", Esc or words typed on a talk's proposal: the TUI sets that card aside (`pages/talk.md`) |
  | `Accept { object, revision }` | "Accept it" on a talk's proposal: the root's `Act::Decide { object, revision, act: Accept }`, then `Act::Confirm` (`pages/talk.md`, 1, *A proposal in place*) |
  | `Leave` | Esc on a talk's empty box with no list, or `/exit` sent whole on a talk: the notice of a talk left while the person's talk is open, then `Went(Home { project })`, the talk's project (`pages/talk.md`, 1, *Leaving*) |
  | `Dismiss` | Esc on a command's list: `Act::Terminal(terminal::Act::Dismiss)`, clearing the left-off selection too |
  | `LeftOff { task }` | Up or Down on the launchpad's left-off rows, `Some(TASK)` the row they select, or Esc there, `None`: the TUI's state keeps the selection (`pages/launchpad.md`, 4) |
  | `Own { command }` | one of the TUI's own commands pressed or sent alone (`pages/README.md`, *The TUI's own commands*), `Own::Draft`, `Change`, `Ask` or `Tell` going to its place not yet started, the box's words cleared, the left-off selection cleared and a walk ended (`pages/README.md`, 2, *Not yet started*), `Brief` the note, `Needs` the walk, `Open` the object in focus opened, `Web`, in front, the box's printed link opened while live, else the web's Home (`box.md`, *`/web`, temper in the browser*), `Talk` its list opened, a choice there `Act::Go` to the role's `Talk`, `Theme` the theme's list opened (`pages/README.md`, *`/theme`*); `n` on the launchpad's empty box makes `Own { command: Needs }` |
  | `Back` | Esc on a place not yet started's empty box with no list standing: the place ends, starting nothing, and a root its refused start left at `NewChat` goes to `Home { project }` (`pages/README.md`, 2, *Not yet started*) |
  | `Theme { choice }` | on the theme's list, `Some(choice)` taken: the host draws in it, keeps it (`theme.md`, *The theme chosen*) and gives its notice; `None`, Esc there: the list lowered |
  | `Open { address }` | a page opened through the opener, the whole address the view made: Enter on the selected goal, a sign-off's "Review on FORGE ↗", "Open it on the web" (`pages/launchpad.md`, 4 and 6) |
  | `Walk { object, step }` | a choice on the walk's prompt for the need `object` (`pages/launchpad.md`, 6): `Choose { words }`, `needs::Act::Choose`; `Open { address }`, the page opened as by `Open`; `Next`; each then marks the need shown |
  | `EndWalk` | Esc on the walk's prompt: the walk ends |

  `Send` on the launchpad also takes `/change WORDS`, `/ask WORDS`,
  `/talk WORDS` and
  `/chats N` before the root's commands (`pages/README.md`, 2, *The TUI's own
  commands*), and, with the walk's question set aside, answers it in
  the box's words as a chat's question's are, then marks it shown. On
  the launchpad, a chat and a talk it takes `/tell WORDS` before them
  too, and on a talk it sends the box's other words as the talk's
  (`pages/talk.md`).
  `Duties.open`, which opens the approval's tab, carries a page's
  address the same way, and in front `/web`'s printed link, the one
  secret it ever carries (`box.md`, *`/web`, temper in the browser*).
  Every address the TUI opens is its origin's,
  `Origin::print` and then the page's path: the approval's tab
  (`../sign-in.md`, 1.1), `/open` and the walk's "Open it on the web" (a
  sign-off's "Review on FORGE ↗" keeps the forge's own address), so
  that at a name a page opens on the deployment the TUI reads, `https://dev.fabrig.ai/chats/T2` (`../../draft/tui-remote.md`, 3).

  A key that makes `Act`, `Command`, `Aside`, `Accept`, `Dismiss`, `Leave`, `LeftOff`,
  `Own`, `Back`, `Open`, `Walk`, `EndWalk` or `Theme` marks the frame it acts on
  in a recording, and typing marks none
  (`journeys.md`, 3.3). Words typed on the launchpad, and leaving
  it, clear the selection.
- **The notice's time:** the TUI's notice (`pages/README.md`, 1, *The notice*) stands
  `limits::NOTICE`, 6 seconds, from the report that gave it, a newer
  one replacing it and its time. The host keeps its end, which
  `Host::deadline` counts among the times it gives, the soonest first;
  a `Due` at or past the end lowers the notice. So a recording keeps it as any turn's deadline
  (`journeys.md`, 3.2), and a replay lowers it at the same turn.
- **The seam's shape:** `Start` carries the start's facts, the
  terminal's background, whether the TUI is on a box and whether it is
  in front of one among them (`box.md`; the user's requests of 2026-11-08);
  `Start.address` is the origin and `Report` gains `Handshake`
  (`../../draft/tui-remote.md`, 3 and 4); `Report` gains `Listening`
  and `Returned`, and `Duties` gains `listen`, `unlisten` and `answer`,
  the loopback return's (`../../draft/tui-remote.md`, 5.1;
  `pages/sign-in.md`, 4);
  `Report` gains `Front` (`box.md`) and, in front, `Lent`, and `Duties`
  gains `lend` and `unlend`, its one lent terminal's (`box.md`, *Lending
  its terminal*); the root is given `Event::Lends` once in front. Following a
  `Show`, the look, `/theme`'s list and the theme taken, `/new`, `/quit`
  and the notice's time are
  the host's own steps on the root and the TUI's state, and closing is
  the duty `close` it has; the `theme` record crosses to the run as a
  `Save` among `asked` (`theme.md`, *The theme chosen*), the root is given
  `Event::Launchpad` once (*The launchpad's reads*), and nothing else new
  crosses.
- **Shaped for the recording now:** a recording is the start's facts,
  then turn by turn the clocks, each report with its duties, each key
  with its event, and a mark for each frame the person saw; a replay
  gives the same reports to a fresh host and finds the same duties and
  frames. The browser to open is a duty now, so that the browser's
  approval (`../../draft/tui.md`, 4.2) and the recording add no field to
  the seam.

## 2. Open questions

None.
