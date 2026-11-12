# tui: status

2026-11-11. Where the design lags or departs from [tui.md](tui.md).

- **Design lags:** 3, 6 and 7, from 2026-11-08: the launchpad as drawn, design sessions with the side pane and talking with a standing role are v0.1.0's through `surfaces.md`, section 11, whose status holds their lags; the side pane and drafting are
  `../client/tui/pages/design-session.md`, 1, as `surfaces.md`, 11.2
  revises them. 3 and 7: of the direction, a need from elsewhere on a chat, the line above its box, the
  web's hand-off, a page's terminal command to copy, and the project
  chosen from a checkout; approval between hosts and managing sign-ins;
  all await the user's refinement, and `../client/README.md`, section 5
  defers them. The needs walked from the launchpad, by `n` or `/needs`,
  and `/open`, the hand-off to the web, are `../client/tui/pages/README.md`,
  as `surfaces.md`, 11.1 and 11.4 take them into v0.1.0. `tui.md`, section 3's launchpad, a chat read as the browser's chat
  terminal reads it, and the terminal's commands are
  `../client/tui/README.md`, 3 and `../client/tui/pages/README.md` and `../client/tui/theme.md` and `../client/tui/README.md`, 4, with what they leave out in its
  section 11. Opened call rows, earlier-page loading
  and "Latest ↓", and cards at their calls remain `../client/README.md`,
  section 5's work for temper/tui; the
  read-once requirement remains `chat-reading.status.md`, 3, for temper/ui.
  `tui.md`, sections 4 and 5 have no lag: 4.2's TUI side is `../client/tui/README.md`, 3 and `../client/README.md`, 4.1 and `../client/view.md`
   is `../client/tui/journeys.md`. `tui.md`, section 8's journey-format
  default is settled by that design: the browser's versioned syntax and
  common records, with the TUI's own records and codec, sharing the
  browser's redaction and last scan (its sections 3 and 5).
  `tui.md`, section 8's other defaults stand as the user confirmed them on
  2026-11-08: bare `temper` runs the TUI, with no subcommand
  (`../client/tui/README.md`, 1); the code rides the approval tab's
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
    its way back (`../client/tui/README.md`, 1 and 2 and `../client/tui/pages/README.md`): the user's
    decision of 2026-11-07. The file's link is never the printed link:
    the engine keeps two one-time links live, the printed link and the
    file's, each spent once and expiring as the printed link does, and
    draws the file's next when a TUI spends it, so the TUI never spends
    the link the person opens in a browser (jig's `protocol/client/sign-in.md`
    , *The local link*; `../client/tui/README.md`, 2): the
    user's decision of 2026-11-08, `tui-native-secret`, which set aside
    a reusable link and no link on loopback. One case is left: a TUI's
    try by a local link the engine no longer remembers, read before a
    restart or before 8 other links were spent or expired, is answered
    `link_unknown` as any unknown secret and draws a printed link when
    none is live, so the box prints again, where the request has the
    TUI never make it print. The cut of `tui-native-secret` kept the
    unknown secret's answer, which needs no stored history (jig's
    `protocol/client/sign-in.md` and `protocol/client/README.md`, section 12, *Remembering why a
    printed link failed*).
  - 4.1: the local sign-in's expiry/revocation distinction uses a note,
    because the engine answers both as signed out (`../client/tui/pages/README.md` and `../client/README.md`, 5
    ). The design chosen by `tui-link-colours` notes
    the answer's expiry once the link is live with a party. A sign-in
    ended more than a minute before that expiry notes `SignedOut`, which
    blocks automatic sign-in until a person's paste or approval. A
    revocation in the final minute is treated as expiry and can renew;
    a missing or unreadable note permits a local try too. Thus the
    request's unconditional refusal to renew after an explicit sign-out
    is not established in those cases. A distinct engine answer remains
    later work; the original draft's browser and pasted-link paths need
    no such distinction.
  - 4.4 and 5: the TUI draws on the terminal's own colours. Its base is the terminal's default text and background, the palette kept for the accent, dim, faint and good colours, and the focus shown by the accent and its mark with no background of its own; dark or light comes from `COLORFGBG`, else the terminal's answer to an OSC 11 query at start, else dark (`../client/tui/theme.md`; skein's `ui/screen.md`, sections 3 and 8). Frames and goldens keep the terminal's default as a colour of its own, which a journey's page draws in the theme's stand-ins (`../client/tui/journeys.md`, 6.4): the user's decision of 2026-11-07, designed by the cut of `tui-link-colours`.
  - 4.1: on the sign-in screen `q` is a character of the pasted link's field, which has the focus there, so Ctrl+C closes it (`../client/tui/pages/README.md`): decided by the cut of `tui-connected`, 2026-11-07. Once signed in, `q` types in the box, and Ctrl+C or `/quit` closes (below).
  - 4.1 and 4.3: once signed in, the TUI opens on the launchpad, not on the first screen's two lines, "Connected to HOST:PORT as PARTY" and "Projects: ...", and its "q to quit" goes: the box has the focus, `q` types in it, and Ctrl+C or `/quit` closes. From the launchpad's box a question asks Ask temper and opens its chat (`../client/tui/README.md`, 3 and `../client/tui/pages/README.md` and `../client/tui/theme.md` and `../client/tui/README.md`, 4): the user's request of 2026-11-08, the look request `tui-launchpad` (its frames 01 to 04 and their message).
  - 3: as that request draws them, the commands are the browser's chat terminal's, `/chats` where this section has `/resume`; the quiet line says the model and effort and the link's state, not the account; and the confirmation's list puts the recommended choice first and "Tell it something else" last with no "Later", as this section asks, where the browser's terminal keeps the choices' order (`../client/tui/pages/README.md`): the user's request of 2026-11-08.
  - 3, and the look request `tui-launchpad` of 2026-11-08: with no project the
    box takes commands alone, rather than still asking temper, because
    template 5 needs a project (`../client/tui/pages/README.md`). No engine
    fact or projectless chat was added. The newest live chat has no
    ", draft" suffix because Home reads no document; a design session's
    row says ", a design session" (`../client/tui/pages/README.md`). The shared Home state says "waiting on you", and the
    shared running line says "✦ Working", rather than the frames'
    "waiting on your words" and "✦ working", so both clients use their
    shared words. Active commands carry the focus mark and accent over
    the band where the theme has one, as every options list does. The
    header has no blank row before the body. These are the landing account's
    explained differences; real-terminal refinement remains
    `../client/README.md`, section 5's after-landing check.
  - 1, 4.1 and 4.3, the user's decision of 2026-11-08, the look request
    `tui-box-front`: `temper box` run in a terminal starts the TUI in
    front of its services, where 4.1 has only `temper` start it; there
    the welcome, following the browser's steps read-only, is its first
    screen, where 4.3 has the launchpad's; it signs in by a link the box
    hands it on a descriptor with nothing to read, paste or approve,
    where 1 and 4.2 have it signed in from the browser or by a paste; and
    Ctrl+C or `/quit` there stops the box with it, asking first while
    work runs, where 4.1's closing is the TUI's alone. `temper-box`, or `temper box` with no terminal, runs without a TUI
    (`../client/tui/box.md`; `../protocol/setup/box.md`, 6). The
    landing account's explained differences: the quiet line names a
    failing step as the box names it ("forgejo-3 restarting"), where the
    request's frame 05 has "forge restarting", and `/log` shows the box's
    own lines as it wrote them, where the frame's are hand-made; its journey recording remains `../client/tui/journeys.status.md`, 8.3; and
    the render check and a real terminal's walk, Linux and macOS, follow
    landing (`../client/README.md`, section 5).
  - 3, 4.4 and 5: the user's requests of 2026-11-08, from a walk on
    their own box (emacs vterm, 150 by 39, a solarized-dark ground), and
    the look request `tui-chat-title-below` (plan `tui-launchpad-2`):
    while the launchpad's box is empty, Up and Down move through up to
    three chats under "Where you left off" and Enter opens the one
    selected; `/effort` is offered wherever `/model` is, the launchpad
    included, its choice taken by the chat the box starts; the person's
    words lie on a band one step in lightness from the terminal's own
    answered background, or, with no answer, in the dim colour after an
    accent "❯", the palette keeping no `sel`; a Markdown bullet is "•",
    never "✦"; and a chat's title, cut to 20 characters at a word, with
    its number and the engine's place ("box", "box:PORT" or `HOST:PORT`,
    the box told by the engine's link file), is in the rule above the box
    rather than a top line (`../client/tui/README.md`, 2 to 3 and `../client/tui/pages/README.md` and `../client/tui/theme.md`). The
    landing account's explained differences: the file does not tell a box
    from `temper serve` run alone, which reads as a box too, where the
    request names "box" for a temper box; the box starting the TUI in
    front tells it (`../client/tui/box.md`), and bare `temper`
    keeps the file (`../client/README.md`, section 5, *On a box, told
    by the box*); `launchpad-chat/04` names the place
    `127.0.0.1:PORT`, not the request's "box:40691", because its walk
    keeps the TUI apart from the box's link file to walk the approval
    (`../client/tui/journeys.md`, 8.2 and 11). The render check and the
    walk follow landing (`../client/README.md`, section 5).
  - 4.1, 4.3, 4.4 and 5, the user's requests of 2026-11-09 (plan
    `tui-box-polish`), after a night of `temper box` with the TUI in
    front (emacs vterm, 150 by 39, a brand-new box): what is typed in
    the box shows from the first frame and the sign-in never clears it,
    Enter before it lands saying "Still signing in"; the person's words
    in the box are the terminal's own ink, the accent on "❯" alone;
    `/theme` opens a list of Auto, Light and Dark, Auto being 4.4's
    start, and the choice is kept in the TUI's small store for the next
    run; the command that closes the TUI is `/quit`, `/exit` taking it
    still, unlisted, where 4.1 closes by `q` or Ctrl+C; and a one-line
    notice stands 6 seconds at the right end of the quiet line
    (`../client/tui/README.md`, 3 and `../client/tui/pages/README.md` and `../client/tui/theme.md` and `../client/tui/README.md`, 4 and `../client/tui/box.md`). Where the request's
    "in a narrow window the notice goes before the quiet line's own
    words" reads two ways, the design draws the notice first, at the
    row's start, and cuts the line's own words after it
    (`../client/tui/pages/README.md`, 1, *The notice*).
- **Open:**
  - 3: a need from elsewhere on a chat, the web's hand-off and a checkout's project, refined on recorded frames as the launchpad and a chat were.
