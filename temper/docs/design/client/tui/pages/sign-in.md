# Signing in

## 1. Signing in

At an engine on loopback (`../README.md`, 1); at a name, section 3.
Illustrative:

```
╭─ temper ─────────────────────────────────────╮
│ Sign in this terminal to 127.0.0.1:8367      │
│ Paste the link temper printed, then Enter.   │
│ Link: ••••••••••••••••••••••••••             │
│ Signing in…                                  │
╰──────────────────────────────────────────────╯

 ● Signed out                     Ctrl+C to quit
```

- **The bottom row:** the link's state first, at the row's first column,
  where every screen has it (`README.md`, *The link's state*), and
  "Ctrl+C to quit" at its right.

- **The paste:** one text field, "Link", a secret, drawn masked (skein's
  `ui/screen.md`, 5.4), that submits on Enter, its placeholder the
  link's shape at the TUI's address, in the form temper prints it,
  `http://HOST:PORT/sign-in?secret=…` (`../../sign-in.md`, section 1),
  and the focus asked for it as the screen opens. A paste fills it whole (skein's `ui/screen.md`, 6.2). A
  `q` types in it, so the bottom row says "Ctrl+C to quit".
- **Enter** checks the field with jig's parser of the printed link
  (`jig-client-protocol-http`'s `printed`, under the default mapping's
  prefix `/jig`; `../../sign-in.md`, section 1), and that its `HOST:PORT` is the TUI's
  address:
  - not a link: "That is not a sign-in link: paste the whole link
    temper printed.";
  - a link for another address: "That link is for HOST:PORT. To sign
    in there, run temper http://HOST:PORT."; nothing is sent;
  - a link for this address: the host asks the root `SignIn { with:
    Link { secret, held } }`, held the credential the directory holds,
    as `temper-box open` does (`../../sign-in.md`, section 1), and the field is emptied; the
    secret stays in the TUI's state only until then.
- **While it signs in:** "Signing in…". `jig-client` sends the link
  again by itself, after a backoff, when the engine answers not ready
  or busy, or no answer comes (jig's `client/domain/client/README.md`, section
  9), and the TUI says which: "Signing in: the engine is busy; trying
  again.", "Signing in: the engine is starting or stopping; trying
  again.", "Signing in: no answer from the engine; trying again.".
- **Refused:** a link used, expired or not this temper's, or the
  sign-ins full, in the view's words (`../../view.md`, 1); a link refused otherwise,
  "This link was refused. Use the fresh one temper printed.". The field
  waits for another.
- **Room for the browser's approval:** the paste is the last way of
  signing in on the screen, under its own lines, so that the approval
  (`../../../draft/tui.md`, 4.2) is laid above it.
- **Signed out elsewhere:** the link signed out after a sign-in, by any
  means (a sign-out from another client, the sign-in ended or expired,
  a held credential the engine does not know): the token is erased
  (`../host.md`, 1), and the screen shows again under the view's "Signed out.".

## 2. Signed in from the engine's link

(`../directory.md`, 1, *The engine's link, kept at
rest*), with nothing pasted and no key pressed. At a name the TUI reads
no link file (`Start.local` false), so none of this section holds
there: a sign-in that expires opens the screen, which asks again
(`sign-in.md`, section 3).

- **When:** the link is signed out, the TUI has not closed, it has an
  engine's link file to read (`Start.local`), and its note for the
  address doesn't say signed out: at its start with no credential kept,
  and again when a sign-in expires (*Expired is not signed out*, below).
  The host asks the duty `local` and, while it signs in from the file,
  the view draws the launchpad's "Connecting to HOST:PORT", the state
  "Connecting": no sign-in screen is drawn and no approval is asked.
- **The file read:** none, a link that `jig-client-protocol-http`'s
  `printed` doesn't read under the prefix `/jig`, or a link for another
  address than the TUI's, and the TUI falls back. A link for its address:
  the host asks the root `SignIn { with: Link { secret, held: None } }`,
  as for a paste (`../../sign-in.md`, section 1), and keeps nothing of the secret but its
  SHA-256, to tell a fresh link from it.
- **Signed in:** the launchpad, as by a link pasted; the credential
  kept (`../host.md`, 1), and the note `Until` the sign-in's expiry.
- **Answered `link_used` or `link_expired`:** the engine draws the next
  local link when a TUI spends one, and a sign-in by a spent or expired
  one draws a fresh one when none is live, which the engine keeps in the
  file and never prints (jig's `protocol/client/sign-in.md`, *The
  local link*; `../directory.md`, 1). So the host reads the file again a second later,
  at most twice for one need, and signs in by a link whose digest
  differs from the one answered; the same link read again waits another
  second, within those two reads. This is the way when the file's link
  expired, or another TUI spent the link this one read. A browser signed
  in by the printed link, before or after, leaves the file's link
  live.
- **Any other end** (refused, `link_unknown`, not ready, busy, full, no
  answer; but in front, not ready and busy (`../box.md`) wait for jig's resend),
  the reads spent, or the file read finding none: the TUI falls
  back to the sign-in screen as it is with no file, which asks the
  browser's approval (`../../sign-in.md`, 1.1) and takes the paste. It says nothing of the
  file: no refusal line is drawn for the file's link, and a fallback
  after an expiry shows the screen under the view's "Signed out.", as
  any sign-in ended. The approval's take replaces a link's sign-in that
  `jig-client` still sends again by itself (`../../sign-in.md`, 1.1, *A pasted link, sent*).
- **Expired is not signed out.** The engine answers both alike, the
  link signed out, so the note tells them apart, from the expiry the
  sign-in's answer gave (`../host.md`, 1, *The note*), on the engine's own machine's
  wall clock. The link signed out after a sign-in, or a held credential
  refused, at or past the note's `Until` less a minute's slack, is an
  expiry: the TUI signs in from the file on its own, as above. Before it,
  the sign-in ended: a sign-out here or elsewhere, or an engine at that
  address that never made it. The TUI notes `SignedOut` and never signs
  in from the file at that address again until a person's act, the paste
  or the browser's approval, signs it in, whose sign-in notes `Until`
  once more. With no note for the address, the TUI has not signed in
  there, and the file is read.
- **Never shown:** the file's secret is never drawn, logged or kept in
  the TUI's state; a recording keeps it in the person's private file, and
  `temper-client-journeys print` redacts it (`../journeys.md`, section 5).

## 3. At a name

(`../../../draft/tui-remote.md`, 5.1, 5.2 and 5.5.) At an `https://`
origin (`../README.md`, 1), signed out. Illustrative, the one click, a
tab opened, "Type a code instead" the button under the lines:

```
╭─ temper ──────────────────────────────────────────────────────────╮
│ Sign in this terminal to dev.fabrig.ai                            │
│ A tab opened in your browser: press Approve there.                │
│ The code there is WDJB-MJHT.                                      │
│ The code expires in 10 min.                                       │
│ No tab? Press Enter for a code to type instead.                   │
│ Type a code instead                                               │
╰───────────────────────────────────────────────────────────────────╯

 ● Signed out                                       Ctrl+C to quit
```

the one click, the opener failed:

```
│ Sign in this terminal to dev.fabrig.ai                            │
│ Open                                                              │
│ https://dev.fabrig.ai/terminal?code=WDJB-MJHT                     │
│ in a browser on this computer, then press Approve.                │
│ The code expires in 10 min.                                       │
│ Or press Enter for a code to type instead.                        │
│ Type a code instead                                               │
```

a typed code, a tab opened:

```
│ Sign in this terminal to dev.fabrig.ai                            │
│ A tab opened in your browser: type WDJB-MJHT there,               │
│ then press Approve.                                               │
│ No tab? Open                                                      │
│ https://dev.fabrig.ai/terminal                                    │
│ The code expires in 10 min.                                       │
```

and a typed code with no tab:

```
│ Sign in this terminal to dev.fabrig.ai                            │
│ Open                                                              │
│ https://dev.fabrig.ai/terminal                                    │
│ in a browser signed in to temper, type WDJB-MJHT,                 │
│ then press Approve.                                               │
│ The code expires in 10 min.                                       │
```

- **The approval alone,** with no paste: no "Paste the link…" line and
  no field, since a deployment elsewhere prints no link the person
  holds (`../../../protocol/sign-in.md`, 2.3). The first line names the
  place, "Sign in this terminal to PLACE", `place(origin, false)`
  (`chat.md`, 1, *The place*).
- **It asks** as on loopback, once each time the screen opens signed
  out (`../../sign-in.md`, 1.1, *The TUI's flow*): at its start with no
  credential kept, after a sign-out or a kept credential the engine
  refused, and after a sign-in expired, since there is no link file to
  renew from (`sign-in.md`, section 2). The ask is the one click where the TUI can
  open a browser, else a typed code's.
- **The one click** (`../../../draft/tui-remote.md`, 5.1): the host
  first has its loopback listener opened (section 4), the duty
  `listen`, and while it opens the screen is as while asking. The run
  reports where it listens, `Listening { port }`; at PORT, the host asks
  `AskApproval { port: Some(PORT) }`, the approval's loopback return,
  and once it is answered opens the tab, once, at the approval page
  with its code, `https://NAME/terminal?code=CODE` (`approval_address`,
  `../../sign-in.md`, 1.1). That page offers Approve on the read alone,
  and the browser hands the sign-in back through the listener (`sign-in.md`, section 4
  ). The run reports no port where the binary chose no opener (an SSH
  session, no display: `../../sign-in.md`, 1.1, *The opener*) or the
  listener couldn't open: the host then asks a typed code's approval.
- **A typed code** (`../../../draft/tui-remote.md`, 5.2):
  `AskApproval { port: None }`, with no listener, where there is no
  opener or no listener, and when the person presses "Type a code
  instead" (below). The code is typed on the page, which asks for it
  (5.3 there). With an opener, the tab opens once for each such ask
  answered, at the origin's approval page with no code,
  `https://NAME/terminal`, so that no code is carried in an address;
  but not after "Type a code instead", where the person has chosen a
  code over the tab, and the lines are those with no tab.
- **The one click's lines,** under the first: "Asking temper for a
  code…" while the listener opens and until the engine answers; then,
  the tab opening or opened, "A tab opened in your browser: press
  Approve there.", "The code there is CODE.", the expiry's line and "No
  tab? Press Enter for a code to type instead."; with the opener failed
  (`Opened { opened: false }`), "Open", the address with its code, "in
  a browser on this computer, then press Approve.", the expiry's line
  and "Or press Enter for a code to type instead.". The ask stands when
  the opener fails, so that a tab it opened all the same, or the
  address opened by hand on this computer, still signs in by one press.
- **A typed code's lines,** under the first: "Asking temper for a
  code…" until the engine answers; then, with a tab opened, "A tab
  opened in your browser: type CODE there, then press Approve." and "No
  tab? Open" with the address; with none, "Open", the address, then "in
  a browser signed in to temper, type CODE, then press Approve.".
- **Both,** the expiry's line "The code expires in SPAN."
  (`../../view.md`, 6's *span*, with its tick). The address stands on a
  line of its own, so that a long name is cut at its end and the words
  around it stay whole. No tunnel's line: a deployment by name is
  reached from any browser.
- **"Type a code instead",** while a one-click ask waits
  (`may_type_instead`: the link signed out, the root's ask `Asked` with
  a port, and no "Ask again"): a button of its own under the lines,
  labelled "Type a code instead", with the focus while it stands. Enter
  or Space presses it, making `TypeCode` (`../host.md`, 1, the
  events' table): the listener is closed (`unlisten`) and a typed
  code's approval asked, whose take replaces the one click's. The
  one-click approval is left to expire, signing nobody in, one of its
  source's two waiting asks until then
  (`../../../draft/tui-remote.md`, 5.4).
- **"Ask again",** with no field: once the approval has ended or was
  not asked (`may_ask_again`), a button of its own stands under the
  lines, labelled "Ask again", with the focus while it stands; Enter or
  Space presses it, making `SignIn` (`../host.md`, 1, the events'
  table), which asks again, the one click where there is an opener.
- **No other key:** while an ask is in flight, the listener's opening
  among it, or a typed code's ask waits, there is no button and no key
  does anything. Every other key does nothing but Ctrl+C, which closes.
  The first line stays a heading, so that a screen reader doesn't read
  the screen's name as a button, and the lines' "Press Enter to ask
  again." and "Press Enter for a code to type instead." stay true. A
  button, not a region acting on Enter alone as the draft has it:
  skein's UI vocabulary has no region that acts on a key
  (`skein/docs/design/ui/tree.md`, 2.1), and a button presses on
  Space too (`skein/docs/design/ui/screen.md`, the keys' table;
  `tui-remote.status.md`, *Design departs*).
- **The outcomes** are loopback's, each in place of the lines after the
  first (`../../sign-in.md`, 1.1, *Each outcome's line*), but for not
  asked: refused, which at a name says "PLACE doesn't sign terminals in
  from the browser.", since there is no link to paste instead. A grant
  that isn't the approval's (`not_granted`) is drawn as waiting.
- **Signed in:** the credential kept in the origin's directory
  (`../directory.md`, 1), the note `Until` the sign-in's expiry, and the
  launchpad drawn, as on loopback.
- **The bottom row** is section 1's; where the native shell's last
  handshake failed, its state says why (`README.md`, *The link's
  state*).
- **The names the one click crosses,** between the root, the TUI's
  state and view, and the host:

  ```rust
  // temper-client-domain (`../../sign-in.md`, 1.1)
  Event::AskApproval { port: Option<u16> }
  Event::Granted { grant: skein_lib::Secret }
  asked::State::Asked { code: ApprovalCode, expires: Wall, port: Option<u16> }
  // temper-client-tui
  pub fn approval_address(origin: &Origin, code: Option<ApprovalCode>) -> Box<[u8]>;
  pub fn may_type_instead(tui: &Tui, root: &Domain) -> bool; // "Type a code instead" stands
  Event::TypeCode                  // that button pressed
  Told::Listener { opening: bool } // the host's listener opening for an ask: drawn as asking, no button
  ```

  While `Told::Listener { opening: true }` holds, the screen draws
  "Asking temper for a code…", and neither `may_ask_again` nor
  `may_type_instead` holds; the host tells `opening: false` with the
  ask it makes, or when it gives the listener up.
- **Its tests:** the host's: an `https://` start has the listener
  opened, asks with its port once it listens, and opens
  `https://NAME/terminal?code=CODE`; with no port reported, it asks a
  typed code's approval and opens `https://NAME/terminal`, no code in
  it; "Type a code instead" closes the listener, asks with no port and
  opens nothing; "Ask again" once the approval ended asks the one click
  again; no key does anything while the listener opens or a code
  waits; an expiry opens the screen and asks again; refused's line. The
  view's rows for each line, the buttons, each with the focus while it
  stands and none while a code waits or an ask is in flight, a 253-byte
  name with a port of its own among them; golden frames at 80 by 24,
  dark and light, of the one click with a tab opened and with the
  opener failed, and of a typed code with a tab opened and with none.

## 4. The loopback return

(`../../../draft/tui-remote.md`, 5.1, *The terminal listens* and *The
terminal takes it*.) The TUI's half of jig's loopback return (jig's
`protocol/client/sign-in.md`, *A sign-in approved by a browser*): a
listener of the run's, for one sign-in, which takes the grant the
approving browser brings and hands it to the root.

- **One listener, for one ask:** the duty `listen` has the run listen
  on `127.0.0.1` at a port the kernel picks (skein-io's `Listen` at
  port 0), never `localhost`, another address or IPv6, and report
  `Listening { port: Some(PORT) }`. A run with no opener reports
  `Listening { port: None }` at the duty itself, listening nowhere, and
  so does one whose listen failed: the ask is then a typed code's
  (section 3). The host holds at most one listener, asking it for each
  one-click ask alone.
- **Its one path:** a request is the listener's when its method is
  `GET`, its `Host` is exactly `127.0.0.1:PORT`, PORT its own in
  decimal, and its target is exactly `/approved?grant=GRANT`, GRANT 43
  characters of base64url with no padding (RFC 4648, 5) that read as 32
  bytes: the form the approval page sends the browser to
  (`../../pages/shared-behaviour.md`, 1, *The hand-off*). While the root holds
  the one-click ask's `Asked`, the host gives the root the grant,
  `Event::Granted { grant }` (`../../sign-in.md`, 1.1, *The grant*), and
  answers the browser `303 See Other` to `approval_address(origin,
  Some(code))`, `https://NAME/terminal?code=CODE`, at once, without
  waiting for the take: the page there reads the approval approved
  ("Waiting for your terminal to sign in…") until the take lands, then
  taken, "Terminal signed in" (`../../pages/shared-behaviour.md`, 1, *Its
  states*).
- **Every other request** is answered `404 Not Found` and gives the
  root nothing: another method, path or query, a `Host` other than its
  own or none, a grant that doesn't read, or the grant's request while
  the root holds no such ask. A grant that reads but isn't the
  approval's is the take's to refuse, `not_granted`, which leaves the
  approval as it was, at most 3 times (jig's
  `protocol/client/sign-in.md`, *Taken*); the listener stays, and the
  browser is sent back to the page all the same. A second grant's
  request takes again, replacing the take before.
- **Its answers:** no body; `Referrer-Policy: no-referrer`,
  `Cache-Control: no-store`, `Content-Length: 0` and `Connection:
  close`, the redirect's `Location` beside them; each connection
  answered once, then closed.
- **Its connections:** at most 4 at once (`limits::RETURNS`), so that a
  browser's idle preconnection holds no other back; each read through
  skein-http's server, one request head of at most 16 KiB
  (`limits::RETURN_HEAD`). A head that skein-http's server refuses, or
  one past that bound, is answered as that server refuses it, or the
  connection closed, and reported to no one. A connection whose head
  hasn't come within 10 seconds (`limits::RETURN_WAIT`) is closed
  unanswered.
- **Its closing,** the duty `unlisten`, once, at the turn the one click
  ends: the link leaves signed out (the sign-in taken, or made by any
  other means), the ask ends (`may_ask_again`: refused, expired, used,
  unknown or full), it wasn't asked (`NotAsked`), or "Type a code
  instead" replaces it; and the TUI's closing, with the run. A
  `Listening` the host no longer wants, its ask gone meanwhile, is
  closed at once. Closing ends the listener at once, each connection
  once its answer is written, and a connection not yet read
  unanswered.
- **Why it is safe:** the grant reaches only the approving browser's
  own machine, and the take needs the ask's secret beside it, which
  only this TUI holds (`../../../draft/tui-remote.md`, 5.1, *Why a press
  is safe*). A process on this machine that reaches the listener first
  could take the grant, but it can read the TUI's token already
  (`../directory.md`, 1, *The risk, accepted*).
- **Never shown:** the grant is never drawn, logged or kept: the run
  hands it up in the request's target, and the host gives it to the
  root, which gives it to `jig-client` for the take. A recording keeps
  it in the person's private file, and `temper-client-journeys print` redacts it
  (`../journeys.md`, section 5).
- **Its tests:** the host's: the grant's request with its own `Host`
  takes the sign-in by the grant and is answered the redirect; a wrong
  `Host`, no `Host`, another path or method, a grant of 42 or 44
  characters or one that doesn't read, and the grant's request before
  the ask is answered or after it ended, each answered 404 and nothing
  taken; a grant `not_granted` leaves the listener and the screen as
  they were; `unlisten` once at each of the closing's turns; the
  run's: a listener on `127.0.0.1` at a port the kernel picked, one
  request answered and closed, a connection silent past the wait
  closed, and no listener with no opener. A fuzzy test reads random
  requests and mutations of the grant's by the listener's rule against
  a naive reading.
