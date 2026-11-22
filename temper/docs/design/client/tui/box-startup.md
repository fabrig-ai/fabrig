# The box front's startup screens

The box front's stages and startup screens are split from [the TUI in front of a box](box.md).

## 1. The box’s stages

**The box's stages**, illustrative (a first run's start, then a stop):

```

  ▄█▀▀   temper 0.0.0 (b948f40d6df1)
 ▄█▄█▄█  temper box · a whole deployment on this machine

 Welcome. Set temper up in your browser: open this link on this machine.

 The link comes here once the box has started.

 It signs you in as this temper's owner, once, within 2 hours.

 In the browser  1 Welcome · 2 Forge · 3 Model · 4 Ready
...
────────────────────────────────────────────────────────────────────────────────
❯ / for commands
────────────────────────────────────────────────────────────────────────────────
 ● Connecting      Downloading temper-box, once for this build: 12.3 of 24.1 MB
```

```
 Stopping the box…
   Ctrl+C again ends its steps at once.
   10:52:01 Stopping
   10:52:02 serve exited 0
```

- **The stages,** `temper_client_tui::BoxStage`, the host's to tell, in
  this order, `Failed` and `Ended` ending it wherever they come:
  `Finding`, the fetch spawned; `Fetching { done, total }`, each
  `fetched` line; `Starting`, the box spawned; `Running`, its link read;
  `Stopping`, the person's stop asked; `Ended { end }`, the box's exit,
  `BoxEnd::Exited { code }` or `BoxEnd::Killed { signal }`; and `Failed
  { why }`, its start failed, `why` its first line, cut at 512 bytes,
  `limits::LOG_LINE`.
- **The stage's line:** "Finding temper-box…", "Downloading
  temper-box, once for this build: DONE of TOTAL MB", or with no size
  "Downloading temper-box, once for this build: DONE MB", DONE and TOTAL
  in millions of bytes with one decimal, and "Starting the box…"; once
  failed, "temper-box could not start: WHY" (`tui-refused`), WHY read as
  UTF-8 with a replacement for each invalid byte, and under it "Ctrl+C
  closes; temper then says what to do." (`tui-hint`). Running, the
  stage has no line.
- **On the welcome,** the user's request `tui-13` of 2026-11-15, so
  that its body keeps its size while the box starts: before the box's
  first `link` line, the link's place, two columns in, holds "The link
  comes here once the box has started." (`tui-hint`), with the link's
  blank rows and "It signs you in…" under it as with the link; the
  stage's line stands on the quiet line's right (*The box's start on
  the quiet line*, below). A failed start's two lines stand in the
  link's place, "It signs you in…" left out, and on the quiet line
  nothing of the stage: the failure is the body's, for good.
- **The box's start on the quiet line,** on the welcome alone, until
  the TUI is signed in by the handed link, first at the quiet line's
  right (`pages/README.md`, 1, *The quiet line*), in `tui-hint`: the
  stage's line, before the box runs; then the box's last `log` line,
  once it runs and has one, its time and the space after it left out
  ("23:32:06Z Starting serve" stands as "Starting serve"), a line with
  no such time standing whole; so the person sees the box start its
  steps where the notice stands, and the body does not move. Read as
  UTF-8 with a replacement for each invalid byte, and cut by skein
  where the row lacks the room (`pages/README.md`, 1, *A narrow
  terminal*). A notice stands in its place for its time. Once signed
  in, the log is `/log`'s alone; the starting screen draws the stage
  in its own steps and never here (*The starting screen*, below).
- **Stopping:** from "Stop the box" taken, or a quit with nothing to
  ask (*Quitting*), until the box has ended: a note of its own, after the
  others, standing for good: the head "Stopping the box…" (`tui-head`),
  then, two columns in, "Ctrl+C again ends its steps at once."
  (`tui-hint`) and the box's last lines as `/log` draws them, its end
  following them. The box's state on the quiet line says "stopping"
  (`tui-signal`). The box's words stay, taking nothing but commands.

## 2. The starting screen

**The starting screen**, illustrative (the user's request `tui-11` of
2026-11-15, a box set up before, its Forgejo starting):

```

  ▄█▀▀   temper 0.0.0 (b948f40d6df1)
 ▄█▄█▄█  temper box · a whole deployment on this machine

 Starting temper
   ✓ The box started · its log in /srv/temper/box/logs/box.log
   ▸ temper starting…
   · Signing in
   · Your projects

 This screen follows the box as it starts. /log shows its log.
...
────────────────────────────────────────────────────────────────────────────────
❯ / for commands
────────────────────────────────────────────────────────────────────────────────
 ● Connecting                                                  Forgejo starting…
```

The welcome with its sign-in link is for a first run alone: on a box
set up before, the TUI says what it waits for, rather than drawing the
welcome for the seconds it takes to sign in and read `me`.

- **When:** in front, on a box set up before (`Start.set_up`), from the
  first frame until the TUI has signed in and read `me` for the first
  time in its run, while the sign-in screen is not drawn; then at once,
  by `me`, the launchpad of its project, where the host goes at any
  `me` naming one (`host.md`, 1, *Its pages*), or, with none, the
  welcome at its step. It never stands again in that run: a link lost
  or an engine restarting later shows the launchpad's or the welcome's
  "Connecting", as before. On a first run it is never drawn.
- **Its header** is the TUI's, in front (`pages/README.md`, 3).
- **Its section** (`tui-sections`): "Starting temper" (`tui-head`) and,
  two columns in, four steps, each marked as the welcome's steps are,
  "✓" done (`tui-step-done`), "▸" under way (`tui-step-now`), "·" ahead
  (`tui-step-ahead`), the first not done under way:

  | Step | Ahead | Under way | Done |
  |---|---|---|---|
  | the box | | the stage's line (*The box's stages*) | "The box started", then " · its log in PATH" (`tui-hint`) |
  | temper | "temper starting" | "temper starting…" | "temper started" |
  | the sign-in | "Signing in" | "Signing in…" | "Signed in" |
  | the projects | "Your projects" | "Your projects…" | |

  The box is under way until its link is read (the stage `Running`);
  temper from then until the engine answers the handed link's sign-in
  otherwise than not ready or busy, since the box starts serve after it
  hands the link and the sign-in waits for it (*Signed in by the handed
  link*); the sign-in from then until the link is live with a party;
  and the projects from then until `me` is read, when the screen goes.
  The marks follow those facts as they stand. PATH is `Start.box_log`,
  read as UTF-8 with a replacement for each invalid byte, drawn whole
  and wrapped at the window's columns.
- **Under them,** "This screen follows the box as it starts. /log
  shows its log." (`tui-hint`).
- **A failed start:** in the box's step's place, with no mark, "temper-box
  could not start: WHY" (`tui-refused`) and under it "Ctrl+C closes;
  temper then says what to do." (`tui-hint`), as on the welcome (*The
  box's stages*); the other steps ahead, and the line under them left
  out. A box that gives up or ends while it stands gets its note, as on
  any screen (*A give up and an end*).
- **Its foot** is the welcome's: the box takes commands alone, its
  placeholder "/ for commands", and words typed are kept for the
  launchpad (`pages/README.md`, 1, *Typed before the sign-in lands*);
  the quiet line's left "● Connecting", its right the box's state, a
  group starting among it (*The box's state*), and a notice; the box's
  start is never on it, its steps holding the stage. `/log` and the quit
  are as on the welcome.
- **Ended otherwise:** out of date, it gives way to the welcome, whose
  first section says so (*The welcome*, *Its header*); a sign-in fallen
  back, to the sign-in screen, as the welcome does (*Signed in by the
  handed link*).

## 3. The welcome

**The welcome**, illustrative (the request's 01, before the person
signed in, drawn as the user's request `tui-16` of 2026-11-15 gives its
first rows at 150 by 39, dark; and 02, after):

```

  ▄█▀▀   temper 0.0.0 (b948f40d6df1)
 ▄█▄█▄█  temper box · a whole deployment on this machine

 Welcome. Set temper up in your browser: open this link on this machine.

 http://127.0.0.1:8367/sign-in?secret=…

 It signs you in as this temper's owner, once, within 2 hours.

 In the browser  1 Welcome · 2 Forge · 3 Model · 4 Ready
```

```
 Setting temper up in your browser
   ✓ Signed in as the owner
   ✓ 1 Welcome
   ▸ 2 Forge · connecting GitHub…
   · 3 Model
   · 4 Ready

 The steps are the browser's; this screen follows them.
```

- **When:** in front, while the sign-in screen and the starting screen
  are not drawn and `me`, read or not, names no project: in the
  no-project launchpad's place (`pages/launchpad.md`, 7), "Connecting"
  included; on a first run, from the first frame, since the person needs
  its link (*The starting screen*, below). Once `me` names a project the
  host goes to its launchpad (`host.md`, 1, *Its pages*), and once it
  names none again, the welcome stands again.
- **Its header** is the TUI's (`pages/README.md`, 3), in front: "temper
  box · a whole deployment on this machine" under the mark's first line.
  Out of date, the view's line is the welcome's first section, before
  its others, as on the launchpad.
- **Before the person signed in** by the printed link
  (`welcome::signed_in`, below), its sections (`tui-sections`,
  `tui-spaced`):
  - "Welcome. Set temper up in your browser: open this link on this
    machine."; under it, one blank row, then the printed link, the URL
    of the box's latest `link` line, drawn whole and wrapped at the
    window's columns, then one blank row (the user's request `tui-14`);
    and "It signs you in as this temper's owner, once, within 2 hours."
    (`tui-hint`), the box's own words for it
    (`../../protocol/setup/box-supervision.md`, 1). Before the box's first `link`
    line, the link's place holds "The link comes here once the box has
    started.", or a failed start's lines (*The box's stages*, above).
    Each of them, the link, its pending and failed lines and the line
    under them, is a paragraph of its own at the column "Welcome."
    starts at, never indented under it, and the link wraps to that
    column too (the user's request `tui-16`, 2026-11-15). The welcome
    opens nothing by itself: the person opens the link, in a browser on
    this machine, or has `/web` give it to the opener (*`/web`, temper
    in the browser*).
  - "In the browser" (`tui-head`), two spaces, then on the same line the
    welcome's four steps, numbered, none started, joined by " · ", "1
    Welcome · 2 Forge · 3 Model · 4 Ready" (`tui-hint`), their labels
    the view's (`../pages/welcome.md`, 1, *Its steps*): one line, which
    fits 80 columns (`tui-16`). A window too narrow for it wraps it as
    any paragraph.
- **After,** "Setting temper up in your browser" (`tui-head`) and, two
  columns in, a list: "Signed in as the owner", ticked; then the four
  steps, numbered, each marked as `welcome::step_marks` marks it at the
  step `welcome::decision` reaches (`../pages/welcome.md`, 1, *The store's state*): Welcome
  while the store is empty, the step left undone while it is unfinished,
  Ready once whole. A step ticked is "✓" (`tui-step-done`), the current
  one "▸" (`tui-step-now`), any other "·" (`tui-step-ahead`); before the
  decision's facts are in, every step is "·". Then "The steps are the
  browser's; this screen follows them." (`tui-hint`).
  - *Read-only:* the TUI follows the steps and takes none. Setup runs in
    the browser alone: the forge is chosen and set up there, a Forgejo
    fetched, GitHub connected or the playground. The step under way is
    the decision's, the first not done, never the step the browser's tab
    shows, which lives in its address, so a person who goes back to an
    earlier step there is not followed.
  - *The forge under way:* on Forge, after its label, " · " and what the
    `forges` watch says of the first real forge in number order, the
    playground's left out (`../forge/records.md`, `../forge/family.md`, 7), in `tui-hint`: while it is under
    way (`connect::underway`), "fetching Forgejo…" while it fetches,
    "setting up Forgejo…" while it prepares or runs, "connecting
    Forgejo…" or "connecting GitHub…" while it connects or while
    GitHub's installation waits to be accepted, and "making its first
    repository…" while that is made; its set up or its first repository
    failed, "stopped: the browser says why"; else nothing.
- **`welcome::signed_in`** (`temper-client-domain`): the person signed in
  by the printed link: the forges document says so (`linked`, `../forge/records.md`;
  `Side::linked`), or the welcome's decision is past its first step,
  unfinished or whole, or `me` names a project. The two last hold after
  the engine restarts, when `linked` reads false again: a store set up
  so far says someone signed in.
- **Its foot** is the no-project launchpad's: the box takes commands
  alone, its placeholder "/ for commands", then the quiet line. Its notes
  follow its sections, as the launchpad's do.
- **Model's detail:** once a model is usable, "3 Model" ticked reads,
  after its label, " · " and ACCOUNT (`tui-hint`), the first usable
  account as the browser's Ready names it (`../pages/welcome.md`, 1,
  *Ready*: "Claude subscription", "ChatGPT plan", "OpenAI API key"),
  and for a program's source ", through Claude Code" after it: "✓ 3
  Model · Claude subscription, through Claude Code"
  (`../../draft/claude-sign-in.md`, 3, t3).

## 4. Open questions

None.
