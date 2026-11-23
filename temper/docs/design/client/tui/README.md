# The TUI

- `directory.md`: the TUI's directory and the engine's local link.
- `host.md`: the TUI's host and seam.
- `box.md`: the TUI in front of a box.
- `box-startup.md`: its stages and startup screens.
- `box-lending.md`: lending its terminal.
- `box-web.md`: opening the browser.
- `pages/README.md`: the TUI's state, shared words, foot and commands.
- `pages/sign-in.md`: signing in and renewing from the engine's link.
- `pages/refused.md`: the screen for a build the engine refuses.
- `pages/launchpad.md`: the launchpad, its rows and walking what needs you.
- `pages/chat.md`: a chat's conversation and its question.
- `pages/design-session.md`: the draft pane, reply choices and starting a goal.

## 1. The TUI: `temper [ADDRESS]`

The TUI (`../../draft/tui.md`) is a person's full-screen terminal client:
temper's client root, a view of its own and skein's full-screen shell
(skein's `ui/screen.md`), over the native transport `temper-box open` uses.
Its first pass (`../../draft/tui.md`, section 4) signs in from the local
engine's link file, with a pasted link or the browser's approval as
fallback (`../sign-in.md`, 1.1), and, at a deployment by name, over
HTTPS, by the browser's approval alone (`../../draft/tui-remote.md`;
`pages/sign-in.md`, 3); once signed in it draws, live, a launchpad, the commands
of the browser's chat terminal and a chat read as that terminal reads
it (`pages/README.md`). `temper-box open` stays the tests' line client
(`../line.md`, 9).

- **Bare `temper`** runs the TUI when stdin and stdout are both
  terminals (skein-shell's `terminal_modes` answers for descriptors 0
  and 1), at the address of the engine's link file its environment
  names, when the binary reads one there (2, *The engine's link, kept
  at rest*), else at the origin its directory keeps (2), an engine on
  loopback or a deployment by name, else at
  `http://127.0.0.1:8367`, the clients' port `temper-box init` prefers
  (`../../../help/reference/commands.md`). So on a machine with no box
  running, bare `temper` goes back to the deployment last used.
  Otherwise it prints the line it
  printed before the TUI, `temper: no command; temper --help lists
  them`, and exits 1.
- **`temper ADDRESS`** names the engine's clients' listener, its
  origin, in one of two forms, each with or without a final `/`
  (`../../draft/tui-remote.md`, 3):
  - `http://HOST:PORT`, HOST an IP address on loopback, read as
    `temper-box open` reads an address (`../line.md`, 1), so that a
    bearer never crosses a network in plaintext (`../sign-in.md`, 1). A
    box whose port fell back to another is named by its address.
  - `https://NAME[:PORT]`, a deployment's public origin, the one the
    person's browser opens (`../../protocol/sign-in.md`, 2.2): NAME a
    host name, labels of letters, digits and hyphens joined by dots,
    each 1 to 63 bytes, none starting or ending with a hyphen, the last
    not all digits, so that every NAME taken is one TLS names a server
    by (skein-tls's `Name`), compared in lower case, at most 253 bytes;
    PORT 1 to 65535, 443
    where none is given; no path, query, user or fragment.

  HTTPS takes a name, never an IP address, since a deployment's
  certificate is for its name; HTTP takes an IP address on loopback,
  never a name, `localhost` among them.
- **The origin,** `temper-client-tui`'s `Origin`, is the one value the
  start, the host, the view, the recording and the TUI's files name the
  engine by:

  ```rust
  pub enum Origin {
      Http(core::net::SocketAddr),          // http://HOST:PORT, HOST an IP address on loopback
      Https { name: Box<[u8]>, port: u16 }, // https://NAME[:PORT], NAME in lower case
  }
  pub enum Unaddressed { NotAnAddress, HttpName, HttpsAddress, OffLoopback }
  impl Origin {
      pub fn parse(bytes: &[u8]) -> Result<Origin, Unaddressed>;
      pub fn print(&self) -> Box<[u8]>;     // http://127.0.0.1:8367, https://dev.fabrig.ai, https://NAME:8443
      pub fn authority(&self) -> Box<[u8]>; // 127.0.0.1:8367, dev.fabrig.ai:443: HOST:PORT, the port always
  }
  ```

  `parse` is the one reader, of the argument, of the origin kept (2)
  and of a recording's (`journeys.md`, 3.2): it takes what `print`
  prints, a final `/` or none, NAME in any case and `:443` written out,
  and refuses the rest by `Unaddressed`'s cause (the table below). A
  sign-in link is told apart before it, as now. `print` leaves out an
  HTTPS port of 443; `authority` never does. The engine's own
  `Origin` (`temper-engine-service-vocabulary`) is the deployment's
  configured public origin, which no client crate links and which holds
  no loopback address the TUI reaches, so the TUI keeps its own.
- **At a name, resolved at start:** for an `https://` origin, after its
  directory (2), the binary resolves NAME once, before the loop, through
  the platform's resolver, `(NAME, PORT)` as `std`'s `ToSocketAddrs`
  takes it, as skein's shells resolve their configured peers (skein's
  `runtime/io/README.md`, section 3), and gives every address found, in
  the resolver's order, to the native shell, which tries them in order
  at each connection (jig's `protocol/client/shells.md`, section 1). It
  reads the machine's roots then, skein-shell's `read_trust_roots` with
  `Trust::Machine`, `SSL_CERT_FILE` from the environment it captured,
  else the distribution's bundle, and gives them as the native shell's
  `skein_tls::Config`, `http/1.1` offered by ALPN. A deployment whose
  addresses change while the TUI runs needs it started again
  (`../../draft/tui-remote.md`, 10).
- **The link is never an argument,** as `temper ctl`'s isn't
  (`../ctl/command-line.md`, 1.1): the TUI takes it pasted on its sign-in screen
  (`pages/README.md`), and refuses it as ADDRESS, in either of its forms
  (`../sign-in.md`, section 1).
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
  | ADDRESS neither form: not `http://` or `https://`, a NAME that isn't one, HOST with no port, a path, a query, a user or a fragment (`NotAnAddress`) | `temper: not an address: ARG` |
  | `http://` with a name, `localhost` among them (`HttpName`) | `temper: http:// reaches an engine on this machine alone: use https://NAME` |
  | `https://` with an IP address (`HttpsAddress`) | `temper: https:// needs the deployment's name` |
  | ADDRESS a sign-in link | `temper: a sign-in link is never an argument: run temper http://HOST:PORT and paste it` |
  | `http://` HOST not on loopback (`OffLoopback`) | `temper: the engine's address must be on loopback` |
  | two arguments or more | `temper: invalid arguments` and the usage, `usage: temper [ADDRESS]` |
  | the directory unresolved or refused (2) | `temper: DIRECTORY: ` and what was refused |
  | NAME not resolved at start, or resolved to no address | `temper: can't find NAME: WHY`, WHY the resolver's words or "no address" |
  | the machine's roots not read | `temper: the machine's trust roots: WHY`, WHY skein-shell's refusal |

  The last two exit 1, as the directory's do.

  A panic aborts the process (fail-stop), the terminal left as it was
  found (4).
- **The client's binary is `temper`** (`../README.md`, section 2;
  `../../cli/README.md`, 7.4): the table above is its bare form's, and
  an ADDRESS that isn't one is refused, exit 2, as its table's refusals
  are. Its `temper-client-shell` captures arguments, environment and
  working directory once and gives them to the TUI's start. Its
  directory, its `address` and each origin's directory are the same (2)
  whether it runs on an engine or in front of a box (`box.md`), and it
  reads the engine's link file (`../../protocol/binaries.md`).

## 2. Its directory

Moved to [`directory.md`](directory.md), section 1.

## 3. Its host and the seam

Moved to [`host.md`](host.md), section 1.

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
  engine, the native shell's `Engine` (jig's `protocol/client/shells.md`,
  section 1: `Plain` at an `http://` origin, `Tls` at a name, with the
  addresses resolved and the machine's roots, 1), the origin's
  directory (2), the origin, the depth, the size, the limits
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
  credential's write ended, the transport's events, the native shell's
  last TLS failure when it changed (3, *The handshake*), the window's size
  when it changed, the keys, and the root's deadline when it is due,
  each a report whose duties are carried at once, the root settled
  first (jig's `client/domain/root.md`, section 3): the requests to the
  native shell, the credential kept or erased, the note written (2),
  the address noted for the binary, the engine's link file read and
  reported at once as `Local`, the loopback listener opened or closed,
  a request's answer written, closing. Once anything was reported, the
  host's frame
  goes to the terminal, which draws the newest at its pace. No key is
  taken and no frame drawn before `Start`.
- **The loopback listener** (`pages/sign-in.md`, 4) is on the run's own
  io, the one that holds the signals: skein-io's `Listen` on
  `127.0.0.1` at port 0, its `Listening` the port reported, each
  `Accepted` socket bound while fewer than `limits::RETURNS` are open
  and rejected otherwise, and skein-http's server above each, which
  reads one request head; the run reports it as `Returned`, writes the
  answer the report's duties carry, and closes the connection. Its
  sockets are counted in the io's limits beside the box's lines and the
  opener's.
- **Time and the seed** are skein-shell's, as `temper-box open`'s (`../line.md`, 2).
- **Closing:** `/quit` (or `/exit`, `pages/README.md`, 2, *`/quit`*), Ctrl+C anywhere,
  the input's
  end, SIGINT or SIGTERM; in front of a box, the end of the box's lines
  too, and `/quit` and Ctrl+C ask first while work runs (`box.md`). The TUI
  doesn't sign out. A loopback listener open closes at once, its
  connections unanswered. The terminal
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

- **The TUI's later passes** (`README.md`, 1, and `directory.md`, 1, and `host.md`, 1 and `pages/README.md` and `theme.md` and `README.md`, 4 and `../sign-in.md`, 1.1): beyond the
  launchpad as drawn, its commands, the walk of what needs the person
  and a chat, a design session among them (`pages/README.md`): talking with a role, `/talk`, `/tell` and the launchpad's team
  lines, T3's (`../../draft/surfaces.md`, 11.1, 11.3 and 11.4); a need
  from elsewhere on a chat, the line above its box; the web's hand-off,
  a page's terminal command to copy; and choosing a project from a
  checkout (`../../draft/tui.md`, sections 2 and 3). Default: refine each
  on the recorded frames before designing it.
- **The launchpad's and a chat's later parts** (`pages/README.md`; `pages/launchpad.md`, 8,
  holds the launchpad's own): a run opened to its
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

- **The TUI's design location:** `README.md`, section 1, and `directory.md`, section 1, and `host.md`, section 1 and `pages/README.md` and `theme.md` and `README.md`, section 4 and `../sign-in.md`, section 1.1 stay
  here; moving them under `client/tui/`, as `../../README.md`, section 6
  assigns them, is a later cleanup that must preserve their citations.
- **The TUI after landing** (`README.md`, 1, and `directory.md`, 1, and `host.md`, 1 and `pages/README.md` and `theme.md` and `README.md`, 4): on a fresh box, start bare
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
  its wrapped rows; on a box set up before, `/web` opening the browser
  signed in, then the web's Home, and over SSH its note's link copied
  whole through a tunnel; the look request `tui-box-front` is checked against
  its frames, the light theme among them (`../web/look-requests.md`,
  sections 4 and 5). The user's requests of 2026-11-09 are checked
  there too, in emacs vterm at 150 by 39 on a brand-new box: keys typed
  on the welcome before the sign-in lands drawn at once and kept, Enter
  then saying "Still signing in", the box's words in the terminal's ink
  after an accent "❯", `/theme`'s list with "dark: your terminal doesn't
  say" and the choice starting the next run, `/quit` and `/exit`, and a
  notice standing 6 seconds on the quiet line, in a narrow window too
  (`pages/README.md` and `theme.md`). Left open: the question counts the TUI's project's
  chats alone, not work in other projects; two steps restarting at once
  show the one last said; a box over SSH is the person's tunnel, with
  no front of its own. Default: so, until the user asks otherwise on a
  recorded frame.
- **`/web` and a sign-in it doesn't see** (`box.md`, *`/web`, temper in
  the browser*): the TUI reads a sign-in by the printed link as the
  forges document's `linked` turning true, and `linked` says whether
  the printed link signed anyone in since the engine started
  (`../forge/records.md`), so a sign-in by a fresh link printed while it
  already reads true is not seen: `/web` then opens that spent link,
  whose page says a fresh one is in the terminal, and the fresh `link`
  line makes `/web` open the fresh one. And once the link is spent,
  `/web` opens the web's Home, so a browser that holds no sign-in, a
  sign-in ended or another browser, lands on the sign-in page with no
  link from the TUI. Default: so, the request's own reading; an engine
  that says each `LinkSignedIn` to its clients, `linked` reset as a
  fresh printed link is drawn or counted, or `/web` asking a link of its
  own (jig's `protocol/client/sign-in.md`, the asked links) for a
  browser with no sign-in, waits for the user's word. Owner:
  temper/tui.

- **The TUI's focus in reverse video** (`theme.md`): the mark and the accent
  show the focus. Default: so; reverse video is the fallback if the
  frames show they can't, a look request's to ask.
- **A sign-out made in the TUI** (`pages/README.md`): the first pass has none, so its
  note's `SignedOut` comes from a sign-in ended elsewhere. Default: a
  later sign-out of the TUI's writes the same note.
- **An engine at the same address that never made the TUI's sign-in**
  (`pages/README.md`): read as a sign-in ended, so the TUI waits for a person's act
  there. Default: so, the safe side, until a deployment's own name
  tells engines apart.
- **Expiry and revocation at the same time** (`pages/README.md`): the engine answers
  both as signed out. The note's last minute is treated as expiry, so a
  revocation in that interval can renew automatically; an absent or
  unreadable note also permits a local try. Distinguishing these ends
  reliably needs an engine answer that names the reason. Default: the
  note and its minute's slack, with this limit kept explicit.
- **The TUI while offline** (`pages/README.md`): the launchpad and a chat keep what
  they last read while the state says "Offline". Default: so, its words
  refined on the recording.
