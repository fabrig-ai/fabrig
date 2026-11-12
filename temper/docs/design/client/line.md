# The line client

## 1. `temper-box open DIRECTORY LINK_OR_ADDRESS`

- **DIRECTORY** is the client's private directory. The terminal opens
  its parent as a root (skein-shell's `open_root`) and gives its last
  component to `jig-client-shell-native`, whose files make it `0o700` if
  it is absent and refuse it unless it is a directory of this user's
  with no group or other bits (skein's `runtime/io/files.md`, 3). It holds the
  bearer credential's file and the small store's, both the native
  shell's (`README.md`, section 4).
- **LINK_OR_ADDRESS** is one of:
  - the link `temper-box serve` printed,
    `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`: the client
    signs in by it first, offering the credential DIRECTORY holds, if
    any, beside it, so that a person still signed in keeps their sign-in
    (`README.md`, section 4);
  - the engine's address, its clients' listener's `http://HOST:PORT`,
    with or without a final `/`: the client signs in with the credential
    DIRECTORY holds. Before its loop, `jig-client-shell-native` reads
    `DIRECTORY/token` once, as it loads the small store, and the host
    gives it to `jig-client` as held (`README.md`, section 4).

  HOST is an IP address, and must be on loopback (in `127.0.0.0/8`, or
  `[::1]`).
- **Exit status:** 0 once the client closed and drained (6); 1 when
  it cannot start, with one line on stderr:

  | Cause | stderr |
  |---|---|
  | not two arguments | `temper-box open: usage: temper-box open DIRECTORY LINK_OR_ADDRESS` |
  | not an `http://` link or address, or HOST not an IP address and port | `temper-box open: not a link or an address: ARG` |
  | HOST not on loopback | `temper-box open: the engine's address must be on loopback` |
  | DIRECTORY refused | `temper-box open: DIRECTORY: ` and what skein's io found unsafe or failed |
  | an address given and no token in DIRECTORY | `temper-box open: not signed in: give the link temper-box serve printed` |
  | the link refused by the engine | `temper-box open: the link was refused: use the fresh link temper-box serve printed` |
  | the engine's sign-ins full (7) | `temper-box open: all N sign-ins are in use; the earliest ends at T; this link stays usable until U; after that, temper-box serve prints a fresh one when it is tried` |

  A panic aborts the process (fail-stop); the small store and the keys
  make what was pending safe to send again at the next `temper-box open`.

## 2. The loop

- **One host on skein's loop** (skein-shell's `drive`; skein's
  `runtime/shell.md`, section 11): temper's client root, its view,
  `jig-client-shell-native`'s transport, and skein's text shell's
  terminal (skein's `ui/text.md`, section 7), with SIGINT and SIGTERM
  from skein-shell's `open_termination_signals`, adopted by io.
- **The text shell's settings** (skein's `ui/text.md`, section 3): the
  name `temper`; the longest line, 4096 bytes; a print interval of one
  second; the status region, the link's state (`view.md`, 1); temper's
  lines, 32 (3); and their line bytes, DIRECTORY's bytes and 160:
  6's signed out holds DIRECTORY and 84 bytes, and a service's
  credential's line its credential's base64 and at most 57.
- **Time and the seed** are skein-shell's: its clock once per
  iteration, and a seed from the kernel at each start (`seed`), from
  which the client root's children draw their keys' numbers (`README.md`, 3.1).
  The view builds at the iteration's wall time, and the loop's
  deadline is the earlier of the domain's and the instant the view's
  words next change (`view.md`, 6).
- **Each iteration:** the transport iterates; its loaded records start
  the client (`README.md`, 3.4); signing in, by the link or with the token DIRECTORY
  holds (`README.md`, section 4); the token written once signed in by a link; the
  transport's events into the domain; the domain's `fire`; the
  transport's requests carried; the view built; each whole line made
  into one command and done; a print, when one is due; the output
  written.

## 3. When it prints

As skein's text shell prints (skein's `ui/text.md`, section 4), at the
interval of 6.2. Before the first tree (signing in), only temper's own
lines, each starting `temper: `.

- **temper's lines wait for room.** The host says each of temper's
  lines only within the room skein's terminal gives (`Terminal::room`).
  A line with none waits in the host, in the order they came; a line
  equal to one already waiting adds none (a sign-in tried again, the
  small store refused again). The waiting queue is uncapped; terminal
  credit and the command hold below bound what waits, without an
  output-count reservation.
- **While a line waits, the person's commands wait.** The host takes no
  command from the terminal while one of temper's lines waits in it, so
  no act of the person's makes a new line; SIGINT and SIGTERM still
  close. What waits is then at most one of each line but a service's
  credential, and one credential for each request already sent, at most
  the request table's requests.
- **Notices are taken whatever the room.** The host takes each of its
  client's notices at every iteration, so a held stdout delays temper's
  lines and the person's commands, never the client: the sign-in's
  retries keep their backoff (7). Lines that wait when the terminal
  closes are not printed.

## 4. How it prints

As skein's text shell prints (skein's `ui/text.md`, section 5). The
first line is `== TITLE == STATUS`: TITLE the page's level-1 heading,
STATUS the link's state (`view.md`, 1), with "Behind" after it while the page
is behind. With the repository sheet open, the underlying page is
Repositories and the dialog is named “Add repositories to PROJECT”. Its
source choices, repository checkboxes, fields and foot actions receive the
same numbered terminal controls as the other pages. Private, already in
the project and copying marks remain words; the terminal prints refusals
and request state beside the affected controls. The sheet has no Next
button: list demand and search read subsequent pages (`forge/family.md`, 4).


## 5. Numbers and commands

skein's text shell's (skein's `ui/text.md`, section 6). temper adds no
command.

- **A sign-in's address** is an external link (skein's `ui/README.md`, 6.2),
  which the terminal prints as `TEXT <URL>`, with no number (skein's
  `ui/text.md`, section 5), and never opens: it starts no browser and
  no program. The person opens the address in a browser on the
  engine's machine, where the provider's redirect reaches the engine
  (`../protocol/credentials.md`, section 3), or in one elsewhere, and
  pastes the address it ends at into the page's field, which the
  terminal draws as the browser does (`pages/accounts.md`, 1, `ConnectModel`, The paste);
  the page shows the sign-in's end when the `accounts` watch tells it. A browser opens the address
  in a new tab instead (`pages/accounts.md`, 1, `ConnectModel`).
- **A forge's connect or install link** is printed the same way, as
  `TEXT <URL>` with no number, and never opened; the terminal names no
  `back`, so the setup route ends on its own page. A connect link's
  address is on the clients' listener (`forge/served.md`, 1), for example
  `http://127.0.0.1:PORT/forges/1/github/manifest?state=STATE`; an
  install link's is GitHub's,
  `https://github.com/apps/SLUG/installations/new?state=STATE`, whose
  end GitHub sends back to the clients' listener. The person opens
  either in a browser that reaches the clients' listener, on the
  engine's machine in standalone, and the forges' page shows the
  connect's end when the engine tells it (`forge/deployment.md`).
- **A key's field,** a secret, prints `(hidden)` once written, and the
  command `n KEY` writes it (skein's `ui/text.md`, sections 5 and 6);
  the terminal's line editing echoes the key as it is typed, which
  skein's text shell cannot turn off (skein's `ui/text.md`, section 9).
  `temper ctl account add-key` reads a key from a file instead
  (`ctl/areas.md`, area 5).
- **A forge's pasted value** prints the same way: each of the forges'
  page's secret fields (`README.md`, 3.6) prints `(hidden)` once written, and the
  terminal echoes it as it is typed. A PEM key spans lines, which one
  command cannot write, so from a terminal an App's key is given with
  `temper ctl`, which reads every pasted value from a private file and
  never from an argument (`ctl/areas.md`, area 6).

## 6. The link, signing out and closing

- **The link's state** is the first line's (4). Its change changes
  the tree, so the page is printed again.
- **Signing out** is `Settings`' "Sign out" (`pages/README.md`), or the top bar's on
  a page that is no project's, except the welcome (`view.md`, 1): the domain asks it, and
  the host gives `jig-client` `SignOut`, which sends jig's sign-out.
  Signed out: the host has the native shell keep none, which leaves
  DIRECTORY's token a zero-byte file, read as none; the main says
  "Signed out." (`view.md`, 1), the first line reads `== temper == Signed out`
  (skein's `ui/text.md`, section 5: TITLE is "the shell's name when
  there is none"), and temper prints `temper: signed out; to sign in
  again, run temper-box open DIRECTORY LINK with a link temper-box serve
  printed`. A sign-out that did not reach the engine leaves the person
  signed in (jig's
  `client/domain/client/README.md`, section 9): the rail or the top bar says
  "Sign out did not reach the engine.", and "Sign out" is offered
  again, except on the welcome: its bar has the link region alone, no person,
  sign-out alert or Sign out, at every step and before the deployment's read.
  The Welcome step prints temper box's paragraph after the cards' note, or
  after the cards without a note; Forge, Model and Ready print none
  (`pages/welcome.md`, 1).
- **Another party's kept work:** while the small store holds requests
  of a party other than the one `me` confirms, temper prints `temper:
  work of another party is kept in DIRECTORY, and waits for them` once.
- **Out of date** (`view.md`, 1): the first line reads `== temper == Out of
  date`, and the main says "This temper is out of date: run the temper
  the engine runs."; temper prints no line of its own. What is pending
  stays in DIRECTORY's small store, closed as below.
- **Kept work dropped** (`README.md`, 3.4): when `jig-client` drops kept requests at
  start, temper prints `temper: N requests kept in DIRECTORY were not
  sent: another version of temper made them` once, N the count it told.
- **A service's credential,** when an answer carries one (jig's
  `protocol/client/sign-in.md`), is printed once: `temper:
  credential for service N, shown once: CREDENTIAL`, the credential in
  base64url. The client keeps no copy of it.
- **Closing:** `q`, the end of stdin, SIGINT or SIGTERM. The client does
  not sign out. skein's text shell closes (`temper: closed`); no new
  exchange starts; `jig-client-shell-native` finishes the small store's
  last write, then closes its connections and files; the process exits
  0. What is pending stays in DIRECTORY's small store, and the next
  `temper-box open` on the same DIRECTORY sends it with its keys, within
  the engine's retention.

## 7. When the engine is unreachable

- **While signing in by the link,** an exchange with no answer, or
  answered not ready (the engine stopping), prints `temper: signing in:
  no answer from the engine; trying again`; one answered busy prints
  `temper: signing in: the engine is busy; trying again`. Either is
  made again, the same link with the same held credential, after a
  backoff of 250 ms, doubling to 8 s. `q` gives up and exits 0. A
  link answered refused, used, expired or unknown exits 1 with the same
  refusal words (1). Temper prints a fresh link when none is live or
  reserved (jig's `protocol/client/sign-in.md`).
- **The engine's sign-ins full** exits 1 with `temper-box open: all N
  sign-ins are in use; the earliest ends at T; this link stays usable
  until U; after that, temper-box serve prints a fresh one when it is tried`
  (1), N the deployment's room, T the earliest saved expiry among its
  sign-ins and U the link's own, in universal time to the second,
  `YYYY-MM-DDTHH:MM:SSZ`. Nothing is written to DIRECTORY, and the link
  is not spent: the same `temper-box open DIRECTORY LINK`, once a sign-in
  has ended and before U, signs in. Full waits for a sign-in to end,
  hours away, so the terminal does not wait for it.
- **Once signed in,** the link is `jig-client`'s to judge (jig's
  `client/domain/client/watches.md`, 2). The first line says its state, and
  the page is printed again at each change. Requests made meanwhile are
  kept in the small store and shown as parked; nothing the person did
  is lost, and `q` keeps it in DIRECTORY.
- **A small store that cannot be written** stops that step's requests,
  whose cards say they were not sent (`NotStored`), and prints `temper:
  cannot write the small store in DIRECTORY`.

## 8. Pure where it decides

The renderer and the command reader are skein's text shell's, step code
under the subset (skein's `ui/text.md`, section 3). The host adds the
terminal's operations (`Terminal::operations`) to its ring's, beside
its client's and its own; it copies none of the terminal's count. The
terminal starts from temper's view limits, as `temper-box open` makes it,
in a unit test (`testing.md`). What temper decides is in its domain and
its view, both step code; golden text fixes how temper's pages print
(`testing.md`).

## 9. While a run's text streams

The terminal is chiefly the real loop's test surface (`../testing/real-loop.md`
): people use the browser, or the TUI (`tui/README.md`, 1). While a
run's text streams, the terminal reprints the whole page at each print
interval (3), as skein's text shell does, with the chat's provisional
turn as the page draws it (`pages/README.md`); the print interval is the only
pace it keeps. No rule of skein's text shell is added for streaming.
A command typed while text streams is done as at any time (5).

## 10. A page's address

The terminal opens at the inbox (`README.md`, 3.4) and reaches each page by its
numbered link (5), never by a page's address: `temper-box open` takes the
engine's address or the sign-in link (1). A browser opens a page by
its address: a reload, or `/tasks/N` typed or opened, is a new client
at that address (`web/README.md`, 2.3), and an inbox's address holds its
filter, so that `/?kind=question` opens the inbox filtered. Of that
open, the path and its load are the browser's; the rest is the client
root's, the same under both shells: a root started at a page's address,
its pages opening that page, asking its reads and drawing it once they
answer. A task address shows its reading status before the document, and
the page asks again after a transient end, as in 3.5. The client's joint
world, `tests/client`, starts its clients so, with no shell
(`fixtures::task_page`), and proves the root's part:
in rung B2's story (`testing.md`) a fresh client started at a delegate's address
draws its heading and its "Task brief".

## 11. Open questions

- **The terminal's forge-key command** (`pages/README.md`, `line.md`, 5): its hint says
  `temper ctl forge replace N FILE`, without the client directory or
  `--key KEY` that ctl requires (`ctl/areas.md`, area 6, and 8.1).
  Completing that command and checking its pointer is separate wording
  work. Default: the browser's paste or a complete ctl call from its
  help; the run-wait pointer checks of `pages/accounts.md`, 2 cover waiting lines alone.
- **Sign-in words in the terminal and command help** (1, 6): the browser's
  signed-out paragraph names the link temper printed (`view.md`, 1; `web/README.md`, 2.4),
  while `temper-box open`'s refusals, the terminal's link instructions and the
  command's help still name `temper-box serve`. A person may have run `temper box`.
  Default: those words stay until a wording change covers the terminal and
  command help; the request `ui-qa-small` names the browser paragraph alone.
  The used, expired and unknown answers also retain the terminal's and
  `temper ctl`'s generic refusal words: `ui-small-1` changes the page's
  paragraph, with reason-specific command refusals left for later.
- **Several terminals:** each `temper-box open` with its own DIRECTORY is a
  client of its own. Two on one DIRECTORY share its token and write its
  small store whole over each other. Default: not supported and not
  detected; a lock on DIRECTORY is the candidate, which `temper ctl`
  takes for its own calls (`ctl/README.md`, 6.3).
- **A fresh link from the client:** default: the person runs `temper-box
  open` with the last link, whose refusal makes `temper-box serve` print a
  fresh one.
- **A key typed in the terminal** is echoed by its line editing (5).
  Default: accepted on one person's machine; `temper ctl account
  add-key` reads one from a file.
- **The terminal's reprint:** skein's text shell prints the whole page
  when it changes (skein's `ui/text.md`, section 4), so a page with a
  time under 12 hours old prints again each minute while it is shown.
  Default: as printed; a terminal shell that says every time as its
  moment, with no wake, is the candidate.
- **The terminal's keys and opening** (`pages/chat/conversation.md`, 2; `../draft/chat-reading.md`, 5): its
  input is never raw (skein's `ui/text.md`, section 7), so PgUp and PgDn are lines
  with Enter, and with no view to measure a chat opens with what its opening reads
  hold, as the TUI's does (`tui/pages/README.md`): the history's end or 2,880 bytes, a screen and a
  half of an 80 by 24 terminal, whatever the terminal's size. Default: as it
  stands; keys of their own are a change of skein's shell, and an opening measured
  by the shell's own size a change of the domain's (*Opening history*).
