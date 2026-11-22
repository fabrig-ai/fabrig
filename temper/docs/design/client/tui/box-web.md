# Opening the browser from the box front

Opening the browser from the box front is split from [the TUI in front of a box](box.md).

## 1. Opening the browser

**`/web`, temper in the browser**, illustrative (the user's request of
2026-11-15, `temper box` started again on a box set up before: "I have
no signin link I can use to open the web UI"; the launchpad with Start
and `/web`'s note, with no opener):

```
 Start
   /change   one change, run as Hello's way says
   /ask      Ask temper
   /web      Open temper in your browser

 Open this link in a browser that reaches temper:

 http://127.0.0.1:8367/sign-in?secret=…

 It signs you in as this temper's owner, once, within 2 hours.
 Opened after that, temper makes a fresh one: /web again opens the fresh one.
 From another machine, tunnel port 8367 to this one first.

────────────────────────────────────────────────────────────────────────────────
❯ Ask for a change, or ask temper.  / for commands
────────────────────────────────────────────────────────────────────────────────
 ● Live · Hello · Claude Opus 5.5 · high
```

- **Why:** in front the box prints no block: its printed link reaches
  the TUI alone, as its `link` line (`../../protocol/setup/box-supervision.md`, 1, and `../../protocol/setup/box-front.md`, 1
  ), and the welcome, the one page that draws it, stands only
  while `me` names no project. On a box set up before the launchpad
  stands from the start, so the link a browser needs to sign in was in
  the TUI's memory alone. `/web` hands it to the browser.
- **Its name:** `/web`, a command of its own, rather than `/open` taking
  it on (the request left it to the designer). `/open` lands on the
  object in focus, out of front too, and carries a page's address alone,
  never a link, a code or a secret (`pages/README.md`, 2, *`/open`*);
  the link route lands where jig's sign-in sends the browser and carries
  no page to land on after it (jig's `protocol/client/sign-in.md`, *The
  printed link*), so `/open` through it would lose its object.
- **Offered** in front alone, while the box's stage is `Running` and it
  has not given up: on the launchpad, the welcome among it (its place
  in front while `me` names no project), a chat and a talk, and on a
  place not yet started as on the screen under it (`pages/README.md`,
  2, *Not yet started*); never on the starting screen or the sign-in
  screen. It is the TUI's own, `Own::Web`, never the root's; its words
  "Open temper in your browser". In the commands' list and in `/help`'s
  note it comes right after `/open`, and first on the welcome, which
  offers no `/open` (`pages/README.md`, 2, *The TUI's own commands*);
  on the launchpad with a project it is Start's last row too
  (`pages/launchpad.md`, 7, *Start*).
- **The printed link's life:** the TUI's state holds the printed link,
  the URL of the box's latest `link` line (*The box's lines*), and
  whether it is **live**, no sign-in by it seen since its line. A `link`
  line makes its URL the printed link, live. A sign-in by it is jig's
  `LinkSignedIn` (jig's `protocol/client/sign-in.md`, *Signed in by it,
  said*), which the TUI reads as the forges document's `linked` turning
  true (`../forge/records.md`): after each report the host tells the
  state the root's `linked` (`root.forge().linked()`, `Side::linked`)
  where it differs from the last it told, none, before the forges watch's first document, told as
  false; `linked` told true where the last told was false spends the
  printed link standing. So the first sign-in by the printed link in an
  engine's run is seen, and the first in a restarted serve's run, whose
  `linked` reads false again; one by a fresh link printed while `linked`
  already reads true is not (`README.md`, 5, *`/web` and a sign-in it
  doesn't see*).
- **Pressed,** or sent whole or begun, it never reaches the root: the
  box's words are cleared (`terminal::Act::Dismiss`), as `/open`'s are,
  and
  - *the printed link live:* the host's duty `open` carries its URL
    whole, as its `link` line gave it, which the run gives the opener as
    it gives any address (`../sign-in.md`, 1.1, *Opening the tab*). The
    browser opens the link route, which signs it in as the deployment's
    owner and spends the link (jig's `protocol/client/sign-in.md`, *The
    printed link*);
  - *spent, or none printed yet:* the duty `open` carries the web's
    Home, as `/open` opens it with no row selected: `http://HOST:PORT`
    and `address::print`'s `/projects/P`, P the screen's project, the
    launchpad's or the chat's or talk's; on the welcome, with no project,
    the web's root, `/`. A browser has signed in by the link, so the
    page opens signed in.
- **The opener's end,** `Opened { opened }` (`host.md`, 1): a link or
  a Home opened draws nothing more, as `/open` draws nothing for a page
  opened. A link not opened, there being no opener (an SSH session, no
  display, none on `PATH`) or the opener failing (`../sign-in.md`, 1.1,
  *The opener*), raises `/web`'s note; a Home not opened, `/open`'s
  "Open ADDRESS in your browser." (`pages/README.md`, 2, *`/open`*).
- **`/web`'s note,** after the others, until the next send or screen, as
  `/log`'s stands: "Open this link in a browser that reaches temper:"
  (`tui-note`); one blank row; the printed link, the latest the box
  printed, drawn whole and wrapped at the window's columns as the
  welcome draws it (*The welcome*), so that a fresh link printed while
  the note stands takes its place there; one blank row; then, under it
  as on the welcome, "It signs you in as this temper's owner, once,
  within 2 hours.", then "Opened after that, temper makes a fresh one:
  /web again opens the fresh one." and "From another machine, tunnel
  port PORT to this one first." (`tui-hint`), PORT the link's port, the
  approval's words for a browser elsewhere (`../sign-in.md`, 1.1, *No
  browser to open*). Each is a paragraph of its own at the note's
  column, the link wrapping to it.
- **A spent link isn't a dead end** (the request's 2): a link opened
  once spent or past its expiry, copied from the note or opened by
  `/web` where the TUI did not see it spent, is answered why, and serve
  prints a fresh one (jig's `protocol/client/sign-in.md`, *The printed
  link*), which the box gives as a `link` line
  (`../../protocol/setup/box-front.md`, 1). The browser's page says "This link
  was already used. A fresh one is in your terminal." or "This link has
  expired. A fresh one is in your terminal." (`../view.md`, 1): in front
  that terminal is the TUI, whose printed link the fresh line makes
  live, so that `/web` again opens it, as the note says.
- **A secret:** the printed link is drawn on the welcome and in `/web`'s
  note, and given to the opener by `/web`, nowhere else: never a notice,
  a log, a file or the environment. The opener takes it as its one
  argument, as it takes any address, since an address is how a browser
  is given a link; the opener's argument, and a browser's it starts, is
  in the machine's process list while they run, and the link it holds is
  spent by the sign-in it makes at once, and expires within 2 hours.
- **Its surface,** beside *Its state, events, words and tokens*':

  ```rust
  // temper_client_tui
  Own::Web                      // "/web"; Own::ALL holds it after Open
  // state::Told gains
  Told::Linked { linked: bool } // the forges document's `linked`, told where it changed
  Told::LinkNotOpened           // the opener ended without opening the link: /web's note
  impl Tui {
      /// The printed link while live: no sign-in by it seen since its `link` line.
      pub fn live_link(&self) -> Option<&Secret>;
  }
  ```

  The host's own record of what it opened, beside a page and the
  approval, gains the link, so that the opener's end raises the note
  that fits. Nothing new crosses the seam: the duty `open` and the
  report `Opened` carry it (`host.md`, 1).

## 2. Open questions

None.
