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
  link's shape at the TUI's address, and the focus asked for it as the
  screen opens. A paste fills it whole (skein's `ui/screen.md`, 6.2). A
  `q` types in it, so the bottom row says "Ctrl+C to quit".
- **Enter** checks the field with jig's parser of the printed link
  (`jig-client-protocol-http`'s `printed`, under the default mapping's
  prefix `/jig`; `../../README.md`, section 4), and that its `HOST:PORT` is the TUI's
  address:
  - not a link: "That is not a sign-in link: paste the whole link
    temper printed.";
  - a link for another address: "That link is for HOST:PORT. To sign
    in there, run temper http://HOST:PORT."; nothing is sent;
  - a link for this address: the host asks the root `SignIn { with:
    Link { secret, held } }`, held the credential the directory holds,
    as `temper-box open` does (`../../README.md`, section 4), and the field is emptied; the
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
  (`../README.md`, 3), and the screen shows again under the view's "Signed out.".

## 2. Signed in from the engine's link

(`../README.md`, 2, *The engine's link, kept at
rest*), with nothing pasted and no key pressed. At a name the TUI reads
no link file (`Start.local` false), so none of this section holds
there: a sign-in that expires opens the screen, which asks again
(section 3).

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
  as for a paste (`../../README.md`, section 4), and keeps nothing of the secret but its
  SHA-256, to tell a fresh link from it.
- **Signed in:** the launchpad, as by a link pasted; the credential
  kept (`../README.md`, 3), and the note `Until` the sign-in's expiry.
- **Answered `link_used` or `link_expired`:** the engine draws the next
  local link when a TUI spends one, and a sign-in by a spent or expired
  one draws a fresh one when none is live, which the engine keeps in the
  file and never prints (jig's `protocol/client/sign-in.md`, *The
  local link*; `../README.md`, 2). So the host reads the file again a second later,
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
  browser's approval (`../../README.md`, 4.1) and takes the paste. It says nothing of the
  file: no refusal line is drawn for the file's link, and a fallback
  after an expiry shows the screen under the view's "Signed out.", as
  any sign-in ended. The approval's take replaces a link's sign-in that
  `jig-client` still sends again by itself (`../../README.md`, 4.1, *A pasted link, sent*).
- **Expired is not signed out.** The engine answers both alike, the
  link signed out, so the note tells them apart, from the expiry the
  sign-in's answer gave (`../README.md`, 3, *The note*), on the engine's own machine's
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
  `add` redacts it (`../journeys.md`, section 5).

## 3. At a name

(`../../../draft/tui-remote.md`, 5.2 and 5.5.) At an `https://` origin
(`../README.md`, 1), signed out. Illustrative, a tab opened:

```
╭─ temper ──────────────────────────────────────────────────────────╮
│ Sign in this terminal to dev.fabrig.ai                            │
│ A tab opened in your browser: type WDJB-MJHT there,               │
│ then press Approve.                                               │
│ No tab? Open                                                      │
│ https://dev.fabrig.ai/terminal                                    │
│ The code expires in 10 min.                                       │
╰───────────────────────────────────────────────────────────────────╯

 ● Signed out                                       Ctrl+C to quit
```

and with no browser to open:

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
  out (`../../README.md`, 4.1, *The TUI's flow*): at its start with no
  credential kept, after a sign-out or a kept credential the engine
  refused, and after a sign-in expired, since there is no link file to
  renew from (section 2).
- **A typed code:** the ask has no loopback return, so the code is
  typed on the page, which asks for it (`../../../draft/tui-remote.md`,
  5.3); the one click, with a listener on loopback, comes later (5.1
  there). The ask is jig-client's `AskApproval`, as on loopback.
- **The tab:** with an opener (`../../README.md`, 4.1, *The opener*),
  once for each ask answered, the origin's approval page with no code,
  `https://NAME/terminal` (`approval_address`, `../../README.md`, 4.1),
  so that no code is carried in an address; without one, or with one
  that failed, nothing is opened and the lines say where to go.
- **Its lines,** under the first: "Asking temper for a code…" until
  the engine answers; then, with a tab opened, "A tab opened in your
  browser: type CODE there, then press Approve." and "No tab? Open"
  with the address; with none, "Open", the address, then "in a browser
  signed in to temper, type CODE, then press Approve."; then "The code
  expires in SPAN." (`../../view.md`, 6's *span*, with its tick). The
  address stands on a line of its own, so that a long name is cut at
  its end and the words around it stay whole. No tunnel's line: a
  deployment by name is reached from any browser.
- **Enter is the screen's one key,** with no field: the screen's one
  node, the lines' region, asks for the focus as the screen opens and
  makes `SignIn` on Enter (`../README.md`, 3, the events' table). Once
  the approval has ended or was not asked (`may_ask_again`), it asks
  again; while an ask is in flight or a code waits, it does nothing.
  Every other key does nothing but Ctrl+C, which closes.
- **The outcomes** are loopback's, each in place of the lines after the
  first (`../../README.md`, 4.1, *Each outcome's line*), but for not
  asked: refused, which at a name says "PLACE doesn't sign terminals in
  from the browser.", since there is no link to paste instead.
- **Signed in:** the credential kept in the origin's directory
  (`../README.md`, 2), the note `Until` the sign-in's expiry, and the
  launchpad drawn, as on loopback.
- **The bottom row** is section 1's; where the native shell's last
  handshake failed, its state says why (`README.md`, *The link's
  state*).
- **Its tests:** the host's: an `https://` start asks at once and opens
  `https://NAME/terminal`, no code in it; no opener opens nothing;
  Enter asks again once the approval ended, and does nothing while it
  waits; an expiry opens the screen and asks again; refused's line. The
  view's rows for each line, a 253-byte name with a port of its own
  among them; golden frames at 80 by 24, dark and light, of the screen
  with a tab opened and with none.
