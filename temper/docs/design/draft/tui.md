# The TUI

Draft, 2026-11-07, from a session with the user that day. temper's TUI:
a full-screen terminal client where a person works with temper's agents,
close to Claude Code's look and feel, in temper's own words. **Not
settled in detail.** Sections 2 and 3 are a direction; section 4 is the
first pass, the only part to build now, with section 5's recording
(`tui.execution.md`). The rest is refined by the user once the first
pass runs, on its recorded frames, as the browser's look is refined on
its journeys. The reasons are `notes.md`, section 9.

## 1. In one page

- **Two surfaces, two jobs.** The web says where things stand: an
  overview, glanced at on a phone or a laptop, of progress and what
  needs the person, and where things are set up. The TUI is where a
  person works with agents: one thing in focus, at length, in words.
  Both show the same objects in the same words, and neither replaces
  the other (section 2).
- **Once work starts, it goes on unattended.** Watching or steering
  running work is not the TUI's: a quick look at whether it is going
  where the person wanted is the web's.
- **Agents never touch the person's machine.** They work on the forge.
  The TUI edits no checkout, unlike Claude Code; run in one, it would
  only choose the project (section 3).
- **Claude Code's look and feel, temper's words:** a framed header, a
  conversation, a composer, a status line, slash commands and prompts
  with numbered choices, in temper's mark and palette. The browser's
  chat already has such a look, its terminal (`../client/README.md`,
  3.5, *A chat's terminal*), and the TUI starts from its glyphs, words,
  commands and themes.
- **Full screen.** The TUI takes the terminal's whole screen and
  scrolls its own content, so that a side pane can fold, take half the
  screen or all of it (section 3), and a recorded frame is the whole
  screen (section 5).
- **Its own view, on the shared client.** It holds the client root the
  browser holds (`../client/README.md`, section 3) and draws its own
  pages as skein-ui's tree, mapped by a full-screen shell of skein's
  (4.4).
- **Signed in from the browser** (4.2): `temper` opens a tab, the
  person checks the code there and approves, then closes the tab. Over
  SSH it prints the address and the code instead.
- **Recorded from the start** (section 5): journeys of its own,
  replayed through its shipped host and printed as frames, which the
  user edits to ask for a look, as `../client/journeys.md` does for the
  browser.
- **The first pass is small** (section 4): a box on the person's
  machine, the browser's approval, a first screen that proves the
  connection end to end, and its journey. It is part of v0.1.0
  (section 6).

## 2. Two surfaces

| | The web | The TUI |
|---|---|---|
| Answers | where do things stand? | let's work on this |
| Shows | everything, briefly | one thing, in depth |
| Where | a phone or a laptop, at a glance | a laptop, in a sitting |
| A person mostly | reads the brief, answers with a choice, sets things up | talks, drafts, starts goals |

- **The same objects.** A chat, a goal and a need are the engine's, so
  what one surface decides the other shows as decided.
- **Setting up stays on the web:** accounts, forges, members, and for
  now configuring by talking (`configuring.md`). The TUI links to the
  web's page rather than drawing a form of its own.
- **The browser's chat terminal is a look, the TUI a client.** The
  terminal is the web's chat page drawn otherwise, inside the web's
  frame (`../client/README.md`, 3.5, *A chat's terminal*). The TUI is a
  program of its own, with pages of its own; where both draw a chat,
  they draw it alike.
- **Hand-offs both ways** (section 7): `/open` opens the object in
  focus on the web; a chat's or a goal's page there gives the terminal
  command to copy; a chat begun in the TUI is an ordinary chat on the
  web.
- **`temper open`, in `temper-box` (`cli.md`, section 4), and `temper ctl` stay what they are:** the line client
  is the tests' surface (`../client/README.md`, 6.9), `temper ctl` the
  agents' (`../client/tools.md`, section 8). The TUI is the person's.

## 3. The direction

What the first pass builds toward. None of it is settled: each part is
drawn, recorded and refined in turn.

- **The first screen, a launchpad.** A framed header with the project
  and the deployment; the status agent's brief in words, before any
  count; where the person left off; what needs them; then the composer.
  Illustrative:

  ```
  ╭─ temper · Hello ─────────────────────────────╮
  │ 3 changes landed overnight. csv-export is    │
  │ two of three tasks in; one waits on you.     │
  │             written by the status agent 07:42│
  ╰──────────────────────────────────────────────╯
   Where you left off
     Away mode, draft · waiting on your words
   Needs you
     ? csv-export: comma or semicolon?
  ❯ Ask temper, or / for commands
    Hello · Claude Opus · ● online · 1 needs you
  ```

- **A chat, read as the browser's chat terminal reads it:** the
  person's words and the agent's answers; a run of calls on one line
  (`chat-reading.md`, section 2); the answer only growing; a
  confirmation as a prompt with numbered choices, the recommended one
  first and "tell it something else" last; the terminal's commands.
- **Drafting:** a chat that writes a draft shows it in a side pane, as
  the web's chat does (`../client/journeys.md`, section 9,
  `chat-draft`): folded, half the screen or all of it. No diffs.
  Leaving it for a goal is "Start a goal from it?", with the project's
  ways of working as the choices.
- **Claude Code's habits, mapped:** a session is a chat, `/resume` the
  chats, `/model` the chat's account and model, the status line the
  project, the account, the link's state and what needs the person.
- **Run in a checkout,** `temper` may choose the project that holds the
  checkout's repository, by its remote.
- **Needs from elsewhere:** one line above the composer, never taking
  the focus; a key walks through them, one prompt after another.

## 4. The first pass

A box on the person's machine, and nothing past it: no deployment at a
name, no Google, no chat, no side pane. Its frames are illustrative:
their words and look are refined on the recording (section 5).

### 4.1 The command

- **`temper`,** alone, with stdin and stdout a terminal, runs the TUI.
  Otherwise it prints the usage it prints today.
- **`temper ADDRESS`** names the engine's clients' listener,
  `http://HOST:PORT`, on loopback, as `temper open` takes it
  (`../client/README.md`, 6.1). With none, the address the TUI last
  signed in to; with none kept, `http://127.0.0.1:8367`, the clients'
  port `temper init` prefers (`../../help/reference/commands.md`). A box
  whose port fell back to another is named by its address.
- **The printed link, pasted:** the sign-in screen also takes the link
  the box printed, pasted into its field, and signs in by it as `temper
  open` does (`../client/README.md`, section 7): the way in when no
  browser can approve, and the only one until the approval is built.
  The link is never an argument, as `temper ctl`'s isn't
  (`../client/tools.md`, 8.1).
- **Its directory,** private as `temper open`'s (`../client/README.md`,
  6.1), under the person's state directory
  (`$XDG_STATE_HOME/temper/tui`, else `~/.local/state/temper/tui`): its
  credential, its small store and the address it last signed in to. It
  is the TUI's alone, never `temper open`'s or `temper ctl`'s.
- **Closing:** `q` or Ctrl+C, as `temper open` closes
  (`../client/README.md`, 6.6): the TUI doesn't sign out, and the
  terminal is left as it was found.

### 4.2 Signing in from the browser

- **A built-in way of jig's,** a third beside the printed link and a
  service's credential (jig's `client/domain/contract/README.md`, section 2):
  **a sign-in approved by a signed-in browser, for that browser's own
  party.** It is enabled where the printed link is, by configuration
  and on a loopback listener alone (jig's `protocol/client/sign-in.md`
  ). Why jig's and not temper's: `notes.md`, section 9.
- **The flow:**
  1. The TUI, signed out, asks the engine for an approval. The answer
     holds a secret, which the TUI keeps and never shows, a short code
     to compare, and an expiry a few minutes away.
  2. The TUI opens the web client's approval page in the browser and
     draws the code and the page's address. It makes that address
     itself, from the engine's address, a fixed path and the code
     checked against the code's alphabet, and gives it to the opener as
     one argument; it opens no address the engine sent. Illustrative:

     ```
     ╭─ temper ─────────────────────────────────────╮
     │ Sign in this terminal                        │
     │ A tab opened in your browser. Check it shows │
     │ WDJB-MJHT, then press Approve.               │
     │ No tab? Open http://127.0.0.1:8367/terminal  │
     ╰──────────────────────────────────────────────╯
     ```

  3. The page, signed in by the browser's cookie, says "Sign in a
     terminal?", the code, when it was asked, "Approve only a code your
     own terminal shows.", then Approve and Refuse. Opening the page
     approves nothing. Opened with no code, it asks for the code to be
     typed, and never lists the approvals that wait.
  4. Approve, sent under the browser's sign-in, marks the approval
     approved for that party. Nothing is made yet.
  5. The TUI, asking with its secret, takes it: only then does the
     engine make the sign-in and draw its credential, handed out once
     in that answer, as any sign-in's is (jig's `protocol/client/sign-in.md`
     ). The TUI keeps it in its directory and draws the first
     screen; the page says "Signed in. You can close this tab." An
     approval never taken expires with nothing made.
- **A browser approves, nothing else:** the approval is sent with the
  browser's cookie, under jig's browser guards (jig's
  `protocol/browser.md`, 12.2), so that no agent driving `temper ctl`
  as the owner (`../client/tools.md`, section 9) can approve a
  terminal.
- **The code in the tab's address,** on a box, so that one press
  approves. Someone else who could ask for an approval and send the
  person its link is on the engine's machine already, as whoever can
  read the printed link is. A deployment at a name types the code
  instead (section 7).
- **Refused, expired or used:** the TUI and the page each say which;
  the TUI offers to ask again.
- **The browser signed out:** the page says to open the link the box
  printed first (`../client/browser.md`, 2.4), then the approval's
  address again.
- **No browser to open** (an SSH session, no display, or the opener
  failing): the TUI prints the address and the code, to open in a
  browser that reaches the engine. The engine is on loopback in this
  pass, so a browser elsewhere needs a tunnel to its listener, on the
  same port, which jig's guards require of the address.
- **Bounds:** a code of a few characters, none that look alike; a few
  approvals waiting at once per deployment; each spent by its first
  approval or refusal. The sign-in it makes counts as any does (jig's
  `protocol/client/sign-in.md`, *Full*).
- **Opening a browser** is the TUI's shell's, `xdg-open` on Linux and
  `open` on macOS: a departure, on purpose, from `temper open`, which
  never opens one (`../client/README.md`, 6.5).

### 4.3 The first screen

It proves the connection, and nothing more. Illustrative:

```
╭─ temper ─────────────────────────────────────╮
│ Connected to 127.0.0.1:8367 as Owner         │
│ Projects: Hello · Demo                       │
╰──────────────────────────────────────────────╯

 q to quit                              ● online
```

- **What it reads:** `me`, for the party's name and projects, and the
  link's state (`../client/README.md`, 3.1), live.
- **The engine stopped and started again:** the status line says so,
  then online again, and nothing is lost.
- **Signed out** elsewhere: the TUI returns to signing in.

### 4.4 Its parts

- **The client root, shared:** `temper-client-domain`, holding
  `jig-client` (`../client/README.md`, sections 2 and 3), as the browser
  and the line client hold it.
- **A view of its own:** the TUI's pages as skein-ui's tree, beside
  `temper-client-view`, with the same words where the same thing is
  said.
- **A full-screen shell of skein's,** beside its text shell (skein's
  `ui.md`, section 16): a renderer that lays the tree out on a grid of
  cells, placing regions by their names (skein's `ui.md`, section 8),
  and a terminal that takes raw keys, the alternate screen, resizes and
  colour. The renderer is step code, the terminal ordinary Rust, as the
  text shell's are (skein's `ui-text.md`).
- **Its look,** a theme the application gives the shell: colours,
  glyphs and borders for its tokens and regions, from the browser's
  chat terminal (`../client/browser.md`, 2.5, *A chat's terminal*), dark
  and light.
- **A host with one seam,** shaped as the browser's
  (`../client/journeys.md`, section 2): reports in (the start, a due, a
  key, a resize, the transport's), what the host asks back (exchanges,
  writes, a browser opened, closing), and the frame drawn. The shell,
  the recorder and the replay all call it (section 5).
- **Names** are the designer's.

## 5. Recorded from the start

The TUI's journeys, as the browser's (`../client/journeys.md`), with the
TUI's host:

- **Recorded at the host's seam,** in a build of its own, on a fresh
  fake box (`../client/journeys.md`, 3.5): the start's facts, the
  screen's size among them, then turn by turn the clocks, each report
  with what the host asked back, each key as the event the view makes
  of it, and a mark for each frame the person saw. A resize is a
  report. A key marks the frame as drawn, another saves; the shell
  reads both before the view does.
- **Replayed through the shipped host,** natively, and stale where it
  asks otherwise (`../client/journeys.md`, section 4).
- **Printed as frames:** a mark's screen as text, a line per row, for a
  diff; and a page that draws it in the theme's colours, linked so that
  the same keys walk the frames; with each journey's README.
- **Kept current by a test,** as the browser's are
  (`../client/journeys.md`, section 6).
- **Redacted and scanned** as the browser's, with the same code
  (`../client/journeys.md`, 3.6 and 3.7), which gains the TUI's
  originals: the approval's secret, the credential it takes, and the
  bearer every exchange carries.
- **Recorded again** by a command, on a fresh fake box, the TUI in a
  pseudo-terminal and the approval given in headless Chromium, as the
  browser's are recorded again (`../client/journeys.md`, 8.1).
- **The look is asked for on the frames:** the person edits a frame's
  text in place and writes the theme's overlay, which stays empty on
  main as the browser's `overlay.css` does; each request lands as the
  browser's do (`look-requests.md`; `../client/journeys.md`, section 7).
- **The first journey:** a fresh fake box, the browser signed in by its
  printed link, `temper` started, the approval's frame, the approval
  given, and the first screen.

## 6. Its place in v0.1.0

- **The first pass is v0.1.0's** (`v0.1.0.md`, section 5), the user's
  word of 2026-11-07: sections 4 and 5 built, and the user has taken
  the first journey's frames.
- **`temper open`'s line client** stays left out as a person's client
  (`v0.1.0.md`, section 6).
- **What follows the first pass** is v0.1.0's as far as `surfaces.md`,
  section 11 takes it (the user, 2026-11-08): the launchpad as drawn,
  design sessions with the draft beside them, and talking with a
  standing role. The rest waits for the user's refinement.

## 7. Later

- **A deployment at a name:** Google's sign-in on the web
  (`../protocol/sign-in.md`), then the same approval over HTTPS, with
  native clients between hosts (`../client/README.md`, section 7). The
  code is typed on the page there, never carried in its address, and
  the page says where the request came from and how long ago.
- **The direction's parts** (section 3) that `surfaces.md`, section 11
  doesn't take into v0.1.0: needs from elsewhere, the hand-offs, the
  project chosen from a checkout.
- **The web's list of a person's sign-ins,** each ended from there.

## 8. Open questions

- **Bare `temper` as a person's command,** where today it prints usage.
  Default: the TUI in a terminal, the usage otherwise. The user's word.
- **The code:** its length and lifetime. Default: eight characters, ten
  minutes. The designer's.
- **The approval's page:** the web client's, or a stock page of the
  server's, as the sign-in's endings are (`../protocol/sign-in.md`,
  4.3). Default: the client's. The designer's.
- **One journey format or two:** the browser's file
  (`../client/journeys.md`, 3.2) with records of the TUI's, or a file of
  the TUI's own. Default: sharing what fits. The designer's.
