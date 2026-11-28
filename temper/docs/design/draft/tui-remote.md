# The TUI at a deployment elsewhere

Draft, 2026-11-18, from a session with the user, who asked to "add
support for the TUI to connect to remote temper deployments", as its
first user, "connecting from my laptop to dev.fabrig.ai". The laptop
runs Linux. On signing in: "can we avoid typing codes? and make things
"just work" by clicking some "accept" button? [...] in general, I'd like
the UX to be as easy as possible, without compromising on security"
(section 5). On versions: "As a first pass, don't worry too much about
this, I'll make sure to keep my client at the same version as dev. Then
when cross-version is there we hopefully get it for free (please
assess), at least to a certain extent" (section 8).

It covers:

- a deployment's address by name, over HTTPS (section 3);
- the connection, over TLS (section 4);
- signing a terminal in from the browser across the internet, with one
  click where the browser is on the terminal's machine (section 5);
- one directory per deployment, so that a box and a deployment
  elsewhere keep their sign-ins apart (section 6);
- a build that doesn't match the deployment's (section 7), and what
  bridging the clients' wires later would give the TUI (section 8).

## 1. In one page

- **`temper https://NAME`** runs the TUI at the deployment at NAME, its
  public origin, the one the person's browser opens (section 3).
  `http://` stays for an engine on loopback, so a bearer never crosses
  a network in plaintext.
- **Over TLS:** jig's native shell carries each connection through
  skein's TLS client, trusting the machine's roots. The binary resolves
  the name once, at start (section 4).
- **One click to sign in:** the TUI asks, opens the person's browser at
  the deployment, signed in there by Google, and the person presses
  Approve. The browser hands the sign-in back to the TUI through a
  listener on the machine's loopback, so an Approve pressed on someone
  else's link signs nobody in elsewhere: the sign-in only ever reaches
  a terminal on the machine of the browser that approved it (5.1). It is
  the way native programs sign in with OAuth (RFC 8252).
- **A typed code where the browser is elsewhere** (an SSH session, no
  display, or a browser on another machine): the code the terminal
  shows, typed on the page, as `tui.md`, 7 sketched (5.2).
- **The engine guards the asks** made by the public name: two waiting
  per source address at most, the page says where each came from and
  how long ago, and one with no loopback return is approved only by its
  code typed, on every origin (5.3 and 5.4).
- **One directory per deployment:** each deployment's token, small store
  and note kept apart, a box's among them, so that no credential is
  offered to another engine and switching signs nobody out (section 6).
- **The place is named:** the header and the rule above the box say
  NAME, where a box says "box" (section 3).
- **A build that doesn't match is said,** in place of a TUI that waits
  for nothing. The person keeps the laptop's build at the deployment's,
  for now (section 7).
- **Bridging the clients' wires later** reaches the TUI with no code of
  its own, for every change a plan derives (section 8).
- **Not in this pass:** `temper ctl` at a name, names resolved while
  running, the deployment's build named and fetched, the web's list of
  sign-ins (section 10).

## 2. Today

- **The TUI reaches an engine on its own machine alone**
  (`../client/tui/README.md`, 1): `http://HOST:PORT`, HOST an IP address
  on loopback. It signs in from the engine's link file, a pasted printed
  link, or the browser's approval (`../client/sign-in.md`, 1.1).
- **jig's approval is designed for loopback:** it is enabled only where
  the clients' listener is on loopback, and approval between hosts, over
  TLS, is later (jig's `protocol/client/sign-in.md`, *A sign-in approved
  by a browser*; jig's `protocol/configuration-engine.md`, 1, and `protocol/configuration.md`, 8). `tui.md`, 7
  sketched it: Google's sign-in on the web, then the same approval over
  HTTPS, the code typed on the page, which says where the ask came from
  and how long ago. On loopback, a code in the page's address approves
  with one press: whoever can ask is on the engine's machine already.
- **A deployment at a name exists already** (`../protocol/sign-in.md`,
  2): a TLS proxy in front of a listener on loopback, the printed link's
  route denied from outside, Google's sign-in on the clients' listener,
  and jig's mapping admitting the public name's `Host`, with a bearer as
  well as a cookie (jig's `protocol/client/default-http.md`, 3).
  dev.fabrig.ai is one, with Google's sign-in on. Its listener is on
  loopback behind the proxy, so jig's startup check admits approval
  there, and the `Host` guard in front of approval's routes admits the
  public name as it admits the listener's own address. Enabled, it would
  be asked from the internet with none of section 5's guards.
- **jig's native shell speaks plain HTTP** to a socket address (jig's
  `protocol/client/shells.md`, 1). skein's TLS client, `skein-tls`, and
  the machine's roots, found by skein-shell's startup, are in the tree
  already, used by the engine's own calls out.
- **A client of another build is refused** (`archive/redeploys.md`, 8;
  `archive/layouts-both-ways.md`, 3), and the browser reloads to the engine's
  bundle. The TUI has no words for it.

## 3. The address

- **`temper https://NAME[:PORT]`,** with or without a final `/`. NAME is
  a host name: labels of letters, digits and hyphens, joined by dots,
  compared in lower case, at most 253 bytes. PORT is 443 when none is
  given. The address has no path, query, user or fragment. It is the
  deployment's public origin (`../protocol/sign-in.md`, 2.2).
- **A name for HTTPS, an IP address for HTTP.** HTTPS takes a name,
  never an IP address, since a deployment's certificate is for its name.
  HTTP takes an IP address on loopback, as today, so a bearer never
  crosses a network in plaintext (`../client/sign-in.md`, 1, *Loopback
  only*). `localhost` is a name, and stays refused for HTTP.
- **Its refusals,** beside the table's (`../client/tui/README.md`, 1),
  exit 2 but the last:

  | Cause | stderr |
  |---|---|
  | `http://` with a name | `temper: http:// reaches an engine on this machine alone: use https://NAME` |
  | `https://` with an IP address | `temper: https:// needs the deployment's name` |
  | a name that isn't one, a path, a query, a user or a fragment | `temper: not an address: ARG` |
  | the name not resolved at start (section 4) | `temper: can't find NAME: WHY`, exit 1 |

  `temper: the engine's address must be on loopback` stays, for an
  `http://` address off loopback.
- **Kept, and come back to:** the TUI's `address` file holds the origin
  last signed in to, `https://NAME` or `http://HOST:PORT`, at most 300
  bytes (`../client/tui/directory.md`, 1). Bare `temper` keeps its order:
  the engine's link file's address, else the address kept, else the
  default. So on a machine with no box running, bare `temper` goes back
  to the deployment last used.
- **The place named:** where a box names its place "box", a deployment
  by name names it by NAME, `dev.fabrig.ai`, with `:PORT` where the port
  isn't 443. That is the header's second line
  (`../client/tui/pages/README.md`, 3) and the rule above the box
  (`../client/tui/pages/chat.md`, 1). An engine on loopback that isn't a
  box keeps `HOST:PORT`.
- **Every address the TUI opens is the origin's:** the approval's tab
  (5.1), `/open`, and each "Open it on the web"
  (`../client/tui/host.md`, 1, *The TUI's events*). A page opens on
  the deployment the TUI reads.
- **No link file at a name:** the engine's link file names an engine of
  this machine, so at a name the TUI reads none and is never on a box.
  Its start's `local` and `boxed` are false (`../client/tui/directory.md`, 1).

## 4. The connection

- **jig's native shell gains HTTPS** for an origin by name. Each
  connection is skein-io's TCP connection, carried by skein's TLS client:
  the server's name NAME, `http/1.1` offered by ALPN, the certificate
  checked against the machine's roots as skein-shell's startup finds
  them, `SSL_CERT_FILE` or else the distribution's bundle. What runs
  above it is as on loopback: the exchanges, the stream and their
  backoff.
- **The name is resolved once, at start,** by the binary, through the
  platform's resolver, as skein's shells resolve their configured peers
  before the loop (skein's `runtime/io/README.md`, 3). The addresses it
  finds, IPv6 and IPv4 as the resolver orders them, go to the native
  shell, which tries them in order at each connection. Resolving while
  running is later (section 10).
- **Through the proxy, as a browser goes:** `Host` the public name, so
  the allow-list admits it (jig's `protocol/client/default-http.md`, 3);
  the credential as `Authorization: Bearer`; no browser header. The
  stream's heartbeat already stays within the proxy's read timeout
  (`../protocol/sign-in.md`, 2.1).
- **A connection that fails** is the link's, as on loopback: "Offline",
  and tried again after jig's backoff. A certificate refused, or a
  handshake that failed, says why on the link's state line, such as
  "Offline: dev.fabrig.ai's certificate isn't trusted", and is tried
  again the same way, so that a renewal under way heals by itself.
- **A laptop that sleeps** wakes to a stream ended. The link reconnects
  by itself, and meanwhile the TUI says "Offline" over the launchpad and
  the chat as they were last read (`../client/tui/README.md`, 5, *The TUI
  while offline*).

## 5. Signing in from the browser

The person's check, today, is that the code their terminal shows is the
code their browser shows (jig's `protocol/client/sign-in.md`, *A sign-in
approved by a browser*, *Its risk*). By the public name, anyone can ask
an approval and send the person a link to it; a code carried in that
link, approved with one press, would sign the asker in as the person.
Two things make a press safe instead: the sign-in handed back through
the approving browser's own machine (5.1), or, where that can't be, the
code typed from the person's own terminal (5.2).

### 5.1 One click: the loopback return

- **The terminal listens:** when the TUI can open a browser on its own
  machine (`../client/sign-in.md`, 1.1, *The opener*), it opens, for the
  sign-in alone, a listener on `127.0.0.1` at a port the kernel picks,
  and asks an approval that names that port, its loopback return.
- **The tab** opens at `https://NAME/terminal?code=CODE`. Signed in, the
  page says "Sign in a terminal on this computer?", CODE beside the ask's
  source and age (5.4), and "Approve" and "Refuse". Signed out, "Continue
  with Google" first, which comes back to the same address.
- **Approve, once:** the engine marks the approval approved and answers
  the page with the approval's grant, 32 random bytes, in that answer
  alone. The page sends the browser to
  `http://127.0.0.1:PORT/approved?grant=GRANT`, PORT the ask's as the
  engine answered it, never one from the page's address, and the host
  always `127.0.0.1`, never one the ask or the address named.
- **The terminal takes it:** its listener answers that one path, checks
  `Host` is its own `127.0.0.1:PORT`, and takes the sign-in with the
  ask's secret and the grant together. It answers the browser with a
  redirect back to `https://NAME/terminal?code=CODE`, which reads the
  approval taken: "Terminal signed in". Every other request is answered
  404. The listener closes once the sign-in is taken, the approval
  ends, or the TUI closes.
- **Why a press is safe:** an approval with a loopback return is taken
  only with its grant, and the grant goes only to the approving browser's
  own machine. A stranger who asks, then sends the person the link,
  gets nothing from the person's press: the browser takes the grant to
  the person's loopback, where the stranger's terminal isn't, and the
  stranger's take, which has the secret but no grant, is refused. A
  process on the person's machine could catch the grant, but it can
  read the TUI's token already (`../client/tui/directory.md`, 1, *The risk,
  accepted*). `Referrer-Policy: no-referrer` and the grant's single use
  keep it out of other sites and replays.
- **Its lines,** illustrative:

  ```
  Sign in this terminal to dev.fabrig.ai
  A tab opened in your browser: press Approve there.
  The code there is WDJB-MJHT. It expires in 10 min.
  No tab? Press Enter for a code to type instead.
  ```

### 5.2 A code typed, where the browser is elsewhere

- **When:** the TUI has no opener (an SSH session, no display), or the
  person presses Enter while the one-click ask waits (5.1): the TUI asks
  an approval with no loopback return, and no listener.
- **The page** is opened by the person at `https://NAME/terminal`, on
  any machine, and asks for the code typed (5.3).
- **Its lines,** illustrative: "Open https://dev.fabrig.ai/terminal in
  a browser signed in to temper, type WDJB-MJHT, then press Approve.",
  then the expiry's line. The tunnel's line is a box's alone: a
  deployment at a name is reached from any browser.
- **Taken as today:** the TUI sends the take each second while it waits
  (`../client/sign-in.md`, 1.1, *The take's outcomes are the link's*).

### 5.3 The page

- **One click only for a loopback return:** the page offers Approve on
  the read alone for an ask that has one. An ask with none, made by the
  public name, is approved only by its code typed, on every origin, the
  listener's own address through a tunnel among them; a code in the
  address is dropped for it and the field shown. An ask made by the
  listener's own address keeps today's page, its code read from the
  address (`../client/pages/shared-behaviour.md`, 1, *The approval page*;
  `../client/web/addresses.md`, 1). The read answers which of the three
  the ask is (5.4).
- **"Sign in first", Google's way given,** keeps the code in its `then`
  for a loopback return, and drops it for an ask approved by typing.
- **The ask, waiting,** says where it came from: "Asked AGO from
  ADDRESS", such as "Asked 12 seconds ago from 203.0.113.7", or "from an
  unknown address" where the engine had none; then, for a typed code,
  "Approve only a code your own terminal shows." Where a typed code's
  ask came from another address than the deciding browser's, as the
  engine saw both, the page adds "It was asked from another network
  than this browser's.", which an SSH session's ask is too.
- **Approved with a loopback return:** "Handing the sign-in to your
  terminal…" while the browser goes to the loopback, then the approval
  read again each second, as today, until it is taken. A browser that
  never reaches the terminal (the TUI closed, the port refused) leaves
  the approval approved and untaken, and it ends expired, signing
  nobody in.

### 5.4 The engine

- **Approval by the public name is what `approval` enables,** in jig's
  clients' configuration (jig's `protocol/configuration-engine.md`, 1), where
  the application gives a public name. There is no second switch, and
  the startup's check stays the listener on loopback, as a deployment
  behind its proxy has it. That section's and jig's
  `protocol/client/README.md`, 12's deferral of approval between hosts
  are what this draft takes up.
- **The ask's kind:** each ask is kept as made by the listener's own
  address, by the public name with a loopback return, or by the public
  name with none, from the exchange's `Host` as the engine end decides it
  (jig's `protocol/client/default-http.md`, 3). The read answers it, so
  that the page applies its rule (5.3). That rule is the guard: a
  decision comes only from the deployment's own page, since it needs the
  browser's cookie, which is strict, beside the browser header.
- **A loopback return:** an ask by the public name may name a port,
  1024 to 65535; one by the listener's own address naming one is
  refused, so the box's asks stay as they are. Approving
  it draws its grant, 32 bytes from the component's generator, answered
  to the deciding browser alone and kept as its digest. Its take must
  carry the ask's secret and the grant, each compared in constant time;
  a take with the secret alone is answered `waiting` while the approval
  waits or is approved, and a grant that doesn't match is refused
  without spending the approval, at most 3 times, then the approval
  ends refused. What a take already admitted to the core holds is as
  today.
- **The ask's source:** jig's engine end gives the component each ask's
  source address with the exchange, as it gives the credential's
  carrier. An exchange that came by the public name has the client's
  address as the proxy saw it, the last entry of `X-Forwarded-For`,
  which the proxy sets, overwriting what the client sent. One by the
  listener's own address has its socket's peer. A header missing or
  unreadable by the public name reads as an unknown source. A process
  on the engine's machine could claim any source by the public name; it
  is on that machine already.
- **Two waiting per source,** by the public name, within the
  deployment's 4: an ask from a source that holds two approvals still
  waiting is answered busy, and replaces nothing. An approval approved,
  or reserved for a take, is not counted, and nothing ends it but its
  own take or expiry. Unknown sources count as one source. Two
  terminals behind one address, a home or an office, each ask; a
  stranger behind that same address can hold its two for their ten
  minutes, and a terminal there meanwhile says "Too many terminals are
  waiting to sign in". An ask by the listener's own address keeps
  today's rule.
- **The ask keeps where it came from:** its source and kind beside the
  wall time it was asked, all three answered by `ApprovalRead` while it
  waits, with the reading browser's own source beside them. It is memory
  alone, as the approval is, and never logged beside its code.

### 5.5 The terminal's screen

- **At a name, the screen holds the approval alone,** with no paste: a
  deployment elsewhere prints no link the person holds
  (`../protocol/sign-in.md`, 2.3). It asks as on a box, each time it
  opens signed out (`../client/sign-in.md`, 1.1, *The TUI's flow*).
- **Enter is the screen's one key,** with no field: while a one-click
  ask waits, it asks one for a typed code (5.2); once an approval has
  ended, or wasn't asked, it asks again, one-click where there is an
  opener; while a typed code's ask waits, or an ask is in flight,
  nothing.
- **The outcomes** are a box's (`../client/sign-in.md`, 1.1, *Each
  outcome's line*), but for "not asked: refused", which at a name says
  "dev.fabrig.ai doesn't sign terminals in from the browser.", since
  there is no link to paste instead.
- **Signed in:** the credential is kept in the deployment's own
  directory (section 6), and the launchpad is drawn, as on a box.
- **An expiry at a name** opens the screen, which asks again: there is
  no link file to renew from (`../client/tui/pages/sign-in.md`, 2).

## 6. One directory per deployment

- **Each origin has its own directory** under the TUI's
  (`../client/tui/directory.md`, 1): `at/HOST/PORT`, HOST the origin's name
  or IP address as printed, PORT its port, 443 included. Each component
  stays within a file name's 255 bytes, and names and IP addresses never
  meet, so neither do the two schemes (`tui-remote.md`, section 3). Each holds that
  deployment's `token`, `store` and `sign-in`, as the TUI's directory
  holds them today, and is private as it is. `address` and the
  recordings stay in the TUI's directory.
- **No credential is offered to another engine:** the TUI gives the
  native shell its origin's directory alone, so a credential and the
  requests kept meet only the engine that made them. Today's "started at
  another address, the TUI offers it there" (`../client/tui/directory.md`,
  1, *One sign-in at a time*) goes: across deployments, it would hand
  one deployment's bearer to whatever listens at another's address.
- **Switching signs nobody out:** a box and dev.fabrig.ai each keep
  their own sign-in, so going from one to the other and back asks
  nothing again while each sign-in lives.
- **The theme stays the person's:** it is a record of the small store
  (`../client/tui/host.md`, 1, `Start.kept`), so a deployment's store
  starts, at its first start, with the theme of the one used last.
- **Nothing new in a versioned format:** `address` and `sign-in` are
  lines of text, `token` is the credential's bytes, and `store` stays
  `jig-client`'s, as today, so an upgrade of the laptop's build reads
  them as it did.
- **From today's directory:** today's TUI only ever reached an engine on
  loopback, and writes the token during its run but `address` only once
  it closes, so neither file alone says whose the token is. At its first
  start, a build with this moves `token`, `store` and `sign-in` into the
  directory of the address `sign-in` names, only where `address` names
  that same address; otherwise it removes them, and the TUI signs in
  afresh, which on a box takes nothing (`../client/tui/pages/sign-in.md`,
  2). The move goes through `at/HOST/PORT.new`, renamed whole into place
  once the three are in it; a start that finds `.new` finishes the move,
  and one whose destination exists already removes today's files.

## 7. A build that doesn't match

- **The person keeps the laptop's build at the deployment's,** for this
  pass (the user, 2026-11-18).
- **When the engine refuses the TUI's version** (`archive/redeploys.md`, 8; the
  owners `jig-client`'s link view names as not served), the TUI draws
  this in place of the sign-in screen or the launchpad, illustrative:

  ```
  dev.fabrig.ai runs another build of temper.
  This one is temper 0.0.0 (b948f40d6df1).
  Start the build dev.fabrig.ai runs, then try again.
  ```

  The link's state reads "Refused", and the TUI sends nothing more until
  it closes. An engine on loopback is named by its place, as in
  section 3.
- **No version logic of the TUI's own:** the screen draws what
  `jig-client` reports, and the hello's fingerprints stay
  `jig-client`'s, the same code for every client (section 8).

## 8. Across builds, later

The user asked whether the TUI gets cross-version communication for
free once it exists. Yes, for every change a plan derives, and with no
code of the TUI's own:

- **What would come:** the clients' wires bridged as a worker's are, the
  engine converting both ways for a client at another layout
  (`archive/layouts-both-ways.md`, 6.1 and 10). It is planned there as later,
  and is in neither of that draft's milestones: their client part
  freezes the clients' versions and refuses a client by its layouts'
  fingerprints at the hello, by name, which is section 7's screen.
- **Why the TUI gets it for free:** the TUI speaks the clients' wires
  through the crates the browser's shell uses, `jig-client`, its
  protocol layer and wire, and temper's client wire. The bridge would
  sit in the engine and in those crates, for the browser and the native
  clients alike, and this draft adds no version logic of the TUI's own
  (section 7) and nothing on disk in a versioned format (section 6).
- **How far it goes,** by layout and by value:
  - a change no plan derives, a retype or a reshape with no written
    converter, is still refused at the hello, and section 7's screen
    still shows (`archive/layouts-both-ways.md`, 7.3);
  - where the floor has removed a conversion a build needs, that build
    is refused (`archive/layouts-both-ways.md`, 7.4);
  - a variant added in the engine's build still derives a plan down to
    an older TUI, and only a value that selects it is refused
    (`archive/layouts-both-ways.md`, 4); what the engine then does with that
    value, and how a newer TUI's request an older engine can't hold is
    answered, typed rather than left waiting, are the bridge's to design;
  - meaning moves with the code, not the layout: an older TUI shows less
    of a newer engine;
  - the requests kept in the small store at another version are dropped
    with a notice today (`upgrades.md`, 7.4).
- **Its bounds are the bridge's:** the browser always reloads to the
  engine's own bundle, so native clients are the bridge's first users,
  and a remote one is the first peer to send its layouts' texts from the
  internet. The client wire's bridge and its bounds are
  `archive/layouts-both-ways.md`'s (10): the hello and the sign-in stay at
  layouts that don't move, so a client of any build is refused by name
  or signs in, and texts are taken after sign-in alone.

## 9. Tests

Focused and fuzzy alone.

- **The address:** the parser's cases and each refusal's line, in the
  TUI's start tests (`temper-client-shell-tui`'s `start`), and a fuzz
  target over the argument's bytes.
- **The connection:** jig's native shell's tests over a TLS listener
  with test certificates and no network: a certificate trusted, one
  refused, a name that doesn't match, and the stream through it.
- **The engine:** jig's clients' component tests: an ask of each kind;
  a loopback return's take with the grant, with the secret alone, and
  with a wrong grant three times; the decision's grant answered once;
  an ask by the public name with and without `X-Forwarded-For`; two
  waiting per source, a third busy, an approved one not counted, and a
  take reserved meanwhile untouched; two terminals behind one source;
  the read's source, kind and wall time.
- **The page:** temper's client world: a loopback return's ask approved
  by one press, and the browser sent to `127.0.0.1` at the engine's
  port whatever the address says; a typed-code ask by the public name
  asking for the code typed on a public origin and on the listener's own
  address, its address's code dropped; an ask by the listener's address
  read from the address as today; the waiting state's source.
- **The TUI:** host tests and golden frames at 80 by 24, dark and light:
  the one-click screen, Enter to a typed code, Enter to ask again, and no
  opener; the loopback listener's one path, a wrong `Host`, a wrong
  grant, and its closing; the build refused; the header and the rule
  naming the place; a directory per deployment, and the move from
  today's with `address` agreeing, disagreeing, and a `.new` left
  behind; a 253-byte name with a port of its own.
- **After landing,** on the user's laptop: `temper
  https://dev.fabrig.ai`, the tab, Google's sign-in when signed out, one
  press, the launchpad and a chat, the laptop slept and woken, then a
  box beside it and back; and over SSH, the typed code.

## 10. Later and open questions

- **`temper ctl` at a name:** the same address, connection and
  directories, ctl signing in as it does (`../client/ctl/README.md`).
  Default: after this pass, when the user asks.
- **The box's one press by the loopback return too:** a box's TUI asks
  from the engine's machine, so today's press stays safe there. Default:
  unchanged; the loopback return if the box ever takes asks from
  elsewhere.
- **No press at all:** with a loopback return the press guards consent,
  not the sign-in's reach, so a page could approve on opening. Default:
  the press stays, so that no tab signs a terminal in unseen.
- **A browser that won't go to loopback:** browsers send a page's own
  navigation to `127.0.0.1` for native programs' sign-ins. If one asks
  the person first, or refuses, the TUI's Enter for a typed code is the
  way. Default: so, its words refined on the frames.
- **Names resolved while running:** a deployment whose addresses change
  while the TUI runs needs the TUI started again, until skein's DNS
  client (skein's `runtime/io/README.md`, 3).
- **The deployment's build named and fetched:** section 7's screen could
  name the build the deployment runs, once the refusal carries it, and
  point at its download (`deployment-binaries.md`, 2). A TUI that
  updates itself would let a deployment run code on the person's
  machine: the user's to decide.
- **Many sources asking:** a stranger with many addresses can still
  hold the deployment's four approvals for their ten minutes, and a
  terminal meanwhile says "Too many terminals are waiting to sign in".
  Default: accepted for this pass; asks by the public name given room
  apart from loopback's if it is seen.
- **The web's list of a person's sign-ins,** each ended from there
  (`tui.md`, 7).
- **Bare `temper`'s order** (section 3): a box running on the same
  machine wins over the deployment used last. Default: so; `temper
  https://NAME` names the other.
- **A deployment's own certificate authority:** default, the machine's
  roots, with `SSL_CERT_FILE` for another bundle.
