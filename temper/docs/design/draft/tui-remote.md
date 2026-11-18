# The TUI at a deployment elsewhere

Draft, 2026-11-18, from a session with the user, who asked to "add
support for the TUI to connect to remote temper deployments", as its
first user, "connecting from my laptop to dev.fabrig.ai". The laptop
runs Linux. On versions: "As a first pass, don't worry too much about
this, I'll make sure to keep my client at the same version as dev. Then
when cross-version is there we hopefully get it for free (please
assess), at least to a certain extent" (section 8).

It covers:

- a deployment's address by name, over HTTPS (section 3);
- the connection, over TLS (section 4);
- signing a terminal in from the browser, across the internet (section 5);
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
- **Signed in from the browser,** through jig's approval, as on a box:
  the TUI asks, then opens `https://NAME/terminal` in the person's
  browser, signed in there by Google; the person types the code the
  terminal shows and approves (5.1). The code never travels in an
  address, and the page says where the ask came from and how long ago
  (`tui.md`, 7). Nothing is pasted: a deployment elsewhere prints no
  link the person holds.
- **The engine guards the ask** by the public name: one live approval
  per source address, the source as the deployment's proxy passes it
  (5.2).
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
  link, or the browser's approval (`../client/README.md`, 4.1).
- **jig's approval is designed for loopback:** it is enabled only where
  the clients' listener is on loopback, and approval between hosts, over
  TLS, is later (jig's `protocol/client/sign-in.md`, *A sign-in approved
  by a browser*). `tui.md`, 7 sketched it: Google's sign-in on the web,
  then the same approval over HTTPS, the code typed on the page, which
  says where the ask came from and how long ago.
- **A deployment at a name exists already** (`../protocol/sign-in.md`,
  2): a TLS proxy in front of a listener on loopback, the printed link's
  route denied from outside, Google's sign-in on the clients' listener,
  and jig's mapping admitting the public name's `Host`, with a bearer as
  well as a cookie (jig's `protocol/client/default-http.md`, 3).
  dev.fabrig.ai is one, with Google's sign-in on. Its listener is on
  loopback behind the proxy, so jig's startup check admits approval
  there, and approval's routes are not guarded by `Host`: enabled, it
  would be asked from the internet with none of section 5's guards.
- **jig's native shell speaks plain HTTP** to a socket address (jig's
  `protocol/client/README.md`, 9). skein's TLS client, `skein-tls`, and
  the machine's roots, found by skein-shell's startup, are in the tree
  already, used by the engine's own calls out.
- **A client of another build is refused** (`redeploys.md`, 8;
  `layouts-both-ways.md`, 3), and the browser reloads to the engine's
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
  crosses a network in plaintext (`../client/README.md`, 4, *Loopback
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
  bytes (`../client/tui/README.md`, 2). Bare `temper` keeps its order:
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
  (`../client/tui/README.md`, 3, *The TUI's events*). A page opens on
  the deployment the TUI reads.
- **No link file at a name:** the engine's link file names an engine of
  this machine, so at a name the TUI reads none and is never on a box.
  Its start's `local` and `boxed` are false (`../client/tui/README.md`, 2).

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

### 5.1 The terminal's side

- **The sign-in screen at a name** holds the approval alone, with no
  paste: a deployment elsewhere prints no link the person holds
  (`../protocol/sign-in.md`, 2.3). It asks as on a box, each time it
  opens signed out (`../client/README.md`, 4.1, *The TUI's flow*).
- **The tab** opens at `https://NAME/terminal`, with no code:
  `approval_address(origin, None)`. The person types the code there
  (5.3).
- **Its lines,** illustrative:

  ```
  Sign in this terminal to dev.fabrig.ai
  A tab opened in your browser. Type the code
  WDJB-MJHT there, then press Approve.
  No tab? Open https://dev.fabrig.ai/terminal
  The code expires in 10 min.
  ```

- **No browser to open** (an SSH session, no display, or the opener
  failing): "Open https://dev.fabrig.ai/terminal in a browser signed in
  to temper, type WDJB-MJHT, then press Approve.", then the expiry's
  line. The tunnel's line is a box's alone: a deployment at a name is
  reached from any browser.
- **The outcomes** are a box's (`../client/README.md`, 4.1, *Each
  outcome's line*), but for "not asked: refused", which at a name says
  "dev.fabrig.ai doesn't sign terminals in from the browser.", since
  there is no link to paste instead.
- **Signed in:** the credential is kept in the deployment's own
  directory (section 6), and the launchpad is drawn, as on a box.
- **An expiry at a name** opens the sign-in screen, which asks again:
  there is no link file to renew from (`../client/tui/pages/sign-in.md`,
  2).

### 5.2 The engine's side

- **Approval by the public name is what `approval` enables,** in jig's
  clients' configuration (jig's `protocol/configuration.md`, 3), where
  the application gives a public name. There is no second switch, and
  the startup's check stays the listener on loopback, as a deployment
  behind its proxy has it. The guards below are what this draft adds.
- **The ask's source:** jig's engine end gives the component each ask's
  source address with the exchange, as it gives the credential's carrier
  (jig's `protocol/client/default-http.md`, 3). An exchange that came by
  the public name has the client's address as the proxy saw it, the last
  entry of `X-Forwarded-For`, which the proxy sets. One by the
  listener's own address has its socket's peer. A header missing or
  unreadable by the public name reads as an unknown source. A process
  on the engine's machine could claim any source by the public name;
  it is on that machine already.
- **One live approval per source,** by the public name, within the
  deployment's 4 (jig's `protocol/client/sign-in.md`, *A sign-in
  approved by a browser*). An ask from a source that holds a live one
  replaces it, as a sign-in's asked link replaces its own (*The asked
  link*, there): the one replaced is remembered expired, a terminal
  asking again never waits, and no one source holds the deployment's
  four. Unknown sources count as one source. An ask by the listener's
  own address keeps today's rule.
- **The ask keeps where it came from:** its source beside the wall time
  it was asked, both answered by `ApprovalRead` while it waits, so that
  the page can say them (5.3). It is memory alone, as the approval is,
  and never logged beside its code.

### 5.3 The page's side

- **Off loopback, the page asks for the code** whatever its address
  carries: on an origin other than an `http://` one on a loopback IP
  address, `/terminal?code=…` reads as `/terminal`, so that a link sent
  by someone else can't put their code in front of the person
  (`../client/pages/README.md`, 2, *The approval page*;
  `../client/web/README.md`, 2.3). The page knows its origin from its own
  address. On loopback, as today.
- **"Sign in first", Google's way given:** off loopback, its `then` is
  `/terminal`, with no code.
- **The ask, waiting,** says where it came from: "Asked AGO from
  ADDRESS", such as "Asked 12 seconds ago from 203.0.113.7", or "from an
  unknown address" where the engine had none; then, as today, "Approve
  only a code your own terminal shows."

## 6. One directory per deployment

- **Each origin has its own directory** under the TUI's
  (`../client/tui/README.md`, 2): `at/NAME` for `https://NAME` on port
  443, `at/NAME:PORT` on another port, and `at/HOST:PORT` for an engine
  on loopback. Names and IP addresses never meet, so neither do the two
  schemes (section 3). Each holds that deployment's `token`, `store` and
  `sign-in`, as the TUI's directory holds them today, and is private as
  it is. `address` and the recordings stay in the TUI's directory.
- **No credential is offered to another engine:** the TUI gives the
  native shell its origin's directory alone, so a credential and the
  requests kept meet only the engine that made them. Today's "started at
  another address, the TUI offers it there" (`../client/tui/README.md`,
  2, *One sign-in at a time*) goes: across deployments, it would hand
  one deployment's bearer to whatever listens at another's address.
- **Switching signs nobody out:** a box and dev.fabrig.ai each keep
  their own sign-in, so going from one to the other and back asks
  nothing again while each sign-in lives.
- **The theme stays the person's:** it is a record of the small store
  (`../client/tui/README.md`, 3, `Start.kept`), so a deployment's store
  starts, at its first start, with the theme of the one used last.
- **From today's directory:** a build with this, at its first start,
  finds `token`, `store` and `sign-in` in the TUI's directory and moves
  them, by rename, into the directory of the address `address` names, or
  of the default address where it names none, before anything is read.
  A move that fails leaves them where they are, and the TUI signs in
  afresh: nothing but a sign-in is lost.

## 7. A build that doesn't match

- **The person keeps the laptop's build at the deployment's,** for this
  pass (the user, 2026-11-18).
- **When the engine refuses the TUI's version** (`redeploys.md`, 8; the
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
  `jig-client` reports, and nothing else decides it (section 8).

## 8. Across builds, later

The user asked whether the TUI gets cross-version communication for
free once it exists. Mostly yes, for every change a plan derives:

- **What would come:** the clients' wires bridged as a worker's are
  (`layouts-both-ways.md`, 10), the engine converting both ways for a
  client at another layout, from the texts the client sends at its
  opening (`layouts-both-ways.md`, 5 and 6).
- **Why the TUI gets it for free:** the TUI speaks the clients' wires
  through the crates the browser's shell uses, `jig-client`, its
  protocol layer and wire, and temper's client wire. A bridge in the
  engine and in those crates reaches the TUI with no code of its own,
  and this draft adds none (section 7).
- **How far it goes:**
  - a change no plan derives, a retype or a reshape with no written
    converter, is still refused, and section 7's screen still shows
    (`layouts-both-ways.md`, 7.3);
  - a build older than the floor of what dev.fabrig.ai runs is refused
    (`layouts-both-ways.md`, 7.4);
  - a new variant the TUI's build lacks has no plan down to it, so
    what the engine does with a value it can't send a client is the
    bridge's to design (`layouts-both-ways.md`, 6.2);
  - meaning moves with the code, not the layout: an older TUI shows
    less of a newer engine, and a newer TUI's new requests go unanswered
    by an older engine;
  - the requests kept in the small store at another version are dropped
    with a notice today (`upgrades.md`, 7.4); a bridge could convert
    them as the store converts its own.
- **What the bridge must design for a remote client:** the browser
  always reloads to the engine's own bundle, so native clients are the
  bridge's first users, and a remote one is the first peer to send its
  texts from the internet. A worker's come over an authenticated channel
  (`layouts-both-ways.md`, 5); a client's would come before or around
  its sign-in. Bounding them, taking them only after sign-in or by
  fingerprint from a cache, is the bridge's design.

## 9. Tests

Focused and fuzzy alone.

- **The address:** the parser's cases and each refusal's line, in the
  TUI's start tests (`temper-client-shell-tui`'s `start`), and a fuzz
  target over the argument's bytes.
- **The connection:** jig's native shell's tests over a TLS listener
  with test certificates and no network: a certificate trusted, one
  refused, a name that doesn't match, and the stream through it.
- **The engine's side:** jig's clients' component tests: an ask by the
  public name with and without `X-Forwarded-For`, one per source
  replacing the last, the deployment's four, and the read's source and
  wall time.
- **The page:** temper's client world: `/terminal?code=…` on an origin
  off loopback asks for the code typed, on loopback reads it, and the
  ask waiting says its source.
- **The TUI:** host tests and golden frames at 80 by 24, dark and
  light: the sign-in screen at a name, with a browser and without; the
  build refused; the header and the rule naming the place; a directory
  per deployment, and the move from today's.
- **After landing,** on the user's laptop: `temper
  https://dev.fabrig.ai`, the tab, Google's sign-in when signed out, the
  code typed and approved, the launchpad and a chat, the laptop slept
  and woken, then a box beside it and back.

## 10. Later and open questions

- **`temper ctl` at a name:** the same address, connection and
  directories, ctl signing in as it does (`../client/ctl/README.md`).
  Default: after this pass, when the user asks.
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
  Default: accepted for this pass; asks by the public name capped apart
  from loopback's if it is seen.
- **The web's list of a person's sign-ins,** each ended from there
  (`tui.md`, 7).
- **Bare `temper`'s order** (section 3): a box running on the same
  machine wins over the deployment used last. Default: so; `temper
  https://NAME` names the other.
- **A deployment's own certificate authority:** default, the machine's
  roots, with `SSL_CERT_FILE` for another bundle.
