# Signing in

## 1. Signing in

Illustrative:

```
╭─ temper ─────────────────────────────────────╮
│ Sign in this terminal to 127.0.0.1:8367      │
│ Paste the link temper printed, then Enter.   │
│ Link: ••••••••••••••••••••••••••             │
│ Signing in…                                  │
╰──────────────────────────────────────────────╯

 Ctrl+C to quit                     ● Signed out
```

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
rest*), with nothing pasted and no key pressed:

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
