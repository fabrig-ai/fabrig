# tui: status

2026-11-08. Where the design lags or departs from [tui.md](tui.md).

- **Design lags:** 3 and 7: of the direction, the side pane and
  drafting, needs from elsewhere, the hand-offs and the project chosen
  from a checkout; approval between hosts and managing sign-ins; all
  await the user's refinement, and `../client/README.md`, section 11
  defers them. Section 3's launchpad, a chat read as the browser's chat
  terminal reads it, and the terminal's commands are
  `../client/README.md`, 6.13 to 6.16, with what they leave out in its
  section 11. The newest goal, opened call rows, earlier-page loading
  and "Latest ↓", and cards at their calls remain `../client/README.md`,
  section 11's work for temper/tui; the
  read-once requirement remains `chat-reading.status.md`, 3, for temper/ui.
  Sections 4 and 5 have no lag: 4.2's TUI side is `../client/README.md`, 6.13 and 7.1,
  and 5 is `../client/tui/journeys.md`. Section 8's journey-format
  default is settled by that design: the browser's versioned syntax and
  common records, with the TUI's own records and codec, sharing the
  browser's redaction and last scan (its sections 3 and 5).
  Section 8's other defaults stand as the user confirmed them on
  2026-11-08: bare `temper` runs the TUI, with no subcommand
  (`../client/README.md`, 6.11); the code rides the approval tab's
  address on a box (4.2); the code is eight characters for ten minutes,
  and the approval's page is the web client's, at `/terminal`.
- **Design departs:**
  - 4.1: a local box signs the TUI in with nothing to paste. The running
    engine keeps a one-time link of its own at rest in a private file,
    under an absolute `$XDG_RUNTIME_DIR` on Linux and the data directory
    on macOS, keyed by the deployment's data directory; with no absolute
    runtime directory on Linux there is no file. A fresh `temper`, or one
    whose sign-in expired, signs in by it with no key pressed, and bare
    `temper`'s address is that file's before the one its directory keeps.
    The risk of a live owner link at rest is recorded as accepted, with
    its way back (`../client/README.md`, 6.11, 6.12 and 6.14): the user's
    decision of 2026-11-07. The file's link is never the printed link:
    the engine keeps two one-time links live, the printed link and the
    file's, each spent once and expiring as the printed link does, and
    draws the file's next when a TUI spends it, so the TUI never spends
    the link the person opens in a browser (jig's `protocol/client.md`,
    section 8, *The local link*; `../client/README.md`, 6.12): the
    user's decision of 2026-11-08, `tui-native-secret`, which set aside
    a reusable link and no link on loopback. One case is left: a TUI's
    try by a local link the engine no longer remembers, read before a
    restart or before 8 other links were spent or expired, is answered
    `link_unknown` as any unknown secret and draws a printed link when
    none is live, so the box prints again, where the request has the
    TUI never make it print. The cut of `tui-native-secret` kept the
    unknown secret's answer, which needs no stored history (jig's
    `protocol/client.md`, section 8 and section 15, *Remembering why a
    printed link failed*).
  - 4.1: the local sign-in's expiry/revocation distinction uses a note,
    because the engine answers both as signed out (`../client/README.md`,
    6.14 and section 11). The design chosen by `tui-link-colours` notes
    the answer's expiry once the link is live with a party. A sign-in
    ended more than a minute before that expiry notes `SignedOut`, which
    blocks automatic sign-in until a person's paste or approval. A
    revocation in the final minute is treated as expiry and can renew;
    a missing or unreadable note permits a local try too. Thus the
    request's unconditional refusal to renew after an explicit sign-out
    is not established in those cases. A distinct engine answer remains
    later work; the original draft's browser and pasted-link paths need
    no such distinction.
  - 4.4 and 5: the TUI draws on the terminal's own colours. Its base is the terminal's default text and background, the palette kept for the accent, dim, faint and good colours, and the focus shown by the accent and its mark with no background of its own; dark or light comes from `COLORFGBG`, else the terminal's answer to an OSC 11 query at start, else dark (`../client/README.md`, 6.15; skein's `ui-screen.md`, sections 3 and 8). Frames and goldens keep the terminal's default as a colour of its own, which a journey's page draws in the theme's stand-ins (`../client/tui/journeys.md`, 6.4): the user's decision of 2026-11-07, designed by the cut of `tui-link-colours`.
  - 4.1: on the sign-in screen `q` is a character of the pasted link's field, which has the focus there, so Ctrl+C closes it (`../client/README.md`, 6.14): decided by the cut of `tui-connected`, 2026-11-07. Once signed in, `q` types in the box, and Ctrl+C or `/exit` closes (below).
  - 4.1 and 4.3: once signed in, the TUI opens on the launchpad, not on the first screen's two lines, "Connected to HOST:PORT as PARTY" and "Projects: ...", and its "q to quit" goes: the box has the focus, `q` types in it, and Ctrl+C or `/exit` closes. From the launchpad's box a question asks Ask temper and opens its chat (`../client/README.md`, 6.13 to 6.16): the user's request of 2026-11-08, the look request `tui-launchpad` (its frames 01 to 04 and their message).
  - 3: as that request draws them, the commands are the browser's chat terminal's, `/chats` where this section has `/resume`; the quiet line says the model and effort and the link's state, not the account; the launchpad's box says "Ask temper" and the quiet line "/ for commands", where this section's sketch has them on one line; and the confirmation's list puts the recommended choice first and "Tell it something else" last with no "Later", as this section asks, where the browser's terminal keeps the choices' order (`../client/README.md`, 6.14): the user's request of 2026-11-08.
  - 3, and the look request `tui-launchpad` of 2026-11-08: with no project the
    box takes commands alone, rather than still asking temper, because
    template 5 needs a project (`../client/README.md`, 6.14). No engine
    fact or projectless chat was added. The newest live chat has no
    ", draft" suffix because Home reads no document and TUI drafting is
    deferred. The shared Home state says "waiting on you", and the
    shared running line says "✦ Working", rather than the frames'
    "waiting on your words" and "✦ working", so both clients use their
    shared words. Active commands carry the focus mark and accent over
    their band, as every options list does. The header and chat top line
    have no blank row before the body, and the chat top line has one
    column of padding at either end. These are the landing account's
    explained differences; real-terminal refinement remains
    `../client/README.md`, section 11's after-landing check.
- **Open:**
  - 3: the side pane, needs from elsewhere, the hand-offs and a checkout's project, refined on recorded frames as the launchpad and a chat were.
