# The TUI in front of a box

`temper-box`, run in a terminal, runs its services and the TUI in front
of them: a small front end for the box, started by it, which quitting
stops (`../../protocol/setup/box.md`, 6). The TUI is the one of `README.md`, 1 to 3 and `pages.md` and `theme.md` and `README.md`, 4
 but where this section says otherwise. Its pages are drawn from
the user's hand-made frames of the look request `tui-box-front`
(2026-11-08), at 80 by 24 in the dark theme, and are illustrative, as
`pages.md`'s are.

- **Its start:** the box spawns `temper-box internal tui --box
  http://HOST:PORT`, HOST:PORT the clients' listener, on loopback, never
  the configured public origin, which is the TUI's address
  (`../../protocol/setup/box.md`, 6); the handed link is made there too. Its stdin is
  the null device, and its stdout and stderr pipes the box reads into
  the front's log, `tui.log`, beside `box.log`. It
  reads two descriptors the box gives it: descriptor 3, the local link's
  URL, written whole by the box and closed, read to its end at start, at
  most 4,096 bytes; and descriptor 4, the box's lines (below), until the
  box exits. Its directory, note, scheme and depth are bare `temper`'s
  (`README.md`, 2, `theme.md`), and so are its exit and its exit status (`README.md`, 1, 4). No
  argument, variable or file holds its link.
- **Its terminal:** it opens the process's controlling terminal and takes
  its foreground before it reads the terminal or sets its modes
  (skein's `runtime/shell.md`, 6.3, `open_terminal` and `take_foreground`), then
  runs on that descriptor, its input and its output, as bare `temper`
  runs on 0 and 1 (`README.md`, 4): raw, with Ctrl+C a key, and its `LEAVE` given
  to the fail-stop hook. The box gives it no descriptor of its terminal
  and writes nothing there while it runs.
- **Signed in by the handed link,** with nothing to read, paste or
  press. The local link's first secret is the box's, given to serve
  before it starts and to the TUI on descriptor 3 (jig's
  `protocol/client/sign-in.md`, *The local link*;
  `../../protocol/setup/box.md`, 1). The run answers the host's first duty
  `local` with it, `Report::Local`, and every later one from the
  engine's link file, as `README.md`, 2 says. In front the host signs in by it at
  `Start`, whatever credential the directory holds and whatever its note
  says: the root starts holding no credential, and the link's sign-in
  carries the one held, `SignIn { with: Link { secret, held } }`, so
  that a sign-in the engine still holds is kept and no new one is made,
  and a credential the engine does not hold signs in afresh (`../README.md`, section 4;
  jig's `protocol/client/sign-in.md`, *A sign-in held is kept*).
  While it signs in, the welcome is drawn, its state "Connecting". It
  ends as `pages.md`'s *Signed in from the engine's link* says: `link_used` or
  `link_expired` reads the engine's link file a second later. Not ready
  and busy wait for jig's resend after its backoff, still "Connecting",
  because the box starts serve beside the front. Any other end falls
  back to the sign-in screen.
- **The seam** (`README.md`, 3) gains the front's two: `Start.front`, true in
  front, `Start.local` true with it; and `Report::Front(BoxLine)`, a line
  of the box's, read. At `Start` in front the host tells the TUI's state
  it is in front.
- **The box's lines,** one a line on descriptor 4 (`../../protocol/setup/box.md`,
  6): a word, one space, a field that runs to the line feed, bytes with
  spaces and no escapes, never empty; at most 8,192 bytes with the line
  feed, `BoxLine::MOST`. A line past it, with an unknown word, or with no
  field after its word and a space is dropped. `temper_client_tui::BoxLine` reads one and
  prints it back:

  | Line | The TUI |
  |---|---|
  | `link URL` | the printed link, the latest replacing the one before (*The welcome*) |
  | `log TEXT` | one of the box's last lines, at most 50 kept, `limits::BOX_LOG`, the oldest dropped (*`/log`*) |
  | `restarting STEP` | the box restarts STEP (*The box's state*) |
  | `running STEP` | STEP runs again |
  | `gave-up STEP` | the box gave up, for good |

  The run reads them on its own io, beside its signals, as they come,
  since the box holds at most 256 the front has not taken, and reports
  each as `Report::Front` in the order read, once `Start` was given, after the
  credential's write and before the transport's events. The end of
  descriptor 4, the box gone, closes the run as SIGTERM does (`README.md`, 4). The
  TUI's state keeps the printed link, the last lines, the step
  restarting and the step given up, in memory alone; of them the printed
  link alone is a secret, and it is drawn on the welcome, nowhere else.
- **Its pages:** the welcome while `me` names no project, then the
  launchpad and a chat as `pages.md` draws them, with the playground's words
  where the project is the playground's. The sign-in screen only where
  the handed link's sign-in fell back.

**The welcome**, illustrative (the request's 01, before the person
signed in, and 02, after):

```
╭─ temper ─────────────────────────────────────────────────────────────── box ─╮
│ temper box · on this machine, to try it                                      │
╰──────────────────────────────────────────────────────────────────────────────╯

 Welcome. Set temper up in your browser: open this link on this machine.

   http://127.0.0.1:8367/jig/v1/sign-in/link?secret=…

   It signs you in as this temper's owner, once, within 2 hours.

 In the browser
   1 Welcome · 2 Forge · 3 Model · 4 Ready
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

- **When:** in front, from the first frame, while the sign-in screen is
  not drawn and `me`, read or not, names no project: in the no-project
  launchpad's place (`pages.md`, *No project yet*), "Connecting" included.
  Once `me` names a project the host goes to its launchpad (`README.md`, 3, *Its
  pages*), and once it names none again, the welcome stands again.
- **Its header,** the region `tui-top`, as the launchpad's: "temper" in
  its border and, at the border's end, the place (*The place*, below),
  with no project; inside it, temper box's line, "temper box"
  (`tui-name`; the view's word, `../pages/welcome.md`, 1, *Its frame*), " · "
  (`tui-faint`) and "on this machine, to try it" (`tui-hint`), the
  browser's line with its links left out. Out of date, the view's line
  stands in its place, as on the launchpad.
- **Before the person signed in** by the printed link
  (`welcome::signed_in`, below), its sections (`tui-sections`,
  `tui-spaced`):
  - "Welcome. Set temper up in your browser: open this link on this
    machine."; under it, two columns in, the printed link, the URL of
    the box's latest `link` line, drawn whole and wrapped at the
    window's columns; and "It signs you in as this temper's owner, once,
    within 2 hours." (`tui-hint`), the box's own words for it
    (`../../protocol/setup/box.md`, 2). Before the box's first `link` line,
    the link and its line are left out. The TUI never opens the link nor
    gives it to its opener: the person opens it, in a browser on this
    machine.
  - "In the browser" (`tui-head`) and, two columns in, the welcome's four
    steps on one line, numbered, none started, "1 Welcome · 2 Forge · 3
    Model · 4 Ready" (`tui-hint`), their labels the view's (`../pages/welcome.md`, 1, *Its
    steps*).
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

**The playground**, illustrative (the request's 03):

```
╭─ temper · Demo ─────────────────────────────────────────── playground · box ─╮
│ This is the playground: a scripted model and in-memory forge. Set            │
│ temper up in your browser when you're ready to leave it.                     │
╰──────────────────────────────────────────────────────────────────────────────╯
...
────────────────────────────────────────────────────────────────────────────────
❯ Ask temper
────────────────────────────────────────────────────────────────────────────────
 / for commands · playground                            Scripted model · ● Live
```

- **Where:** wherever the TUI's project, the launchpad's or the open
  chat's, is the running playground's (`../forge/family.md`, 7: its read's `project` while
  it runs), in front of a box or not. The TUI works in it as in any
  project.
- **The header,** on the launchpad: at the border's end, before the
  place, "playground" (`tui-playground`) and " · " (`tui-end-faint`),
  the three left out whole together where the border lacks the room
  (skein's `ui/screen.md`, 5.2). Inside it, in the place of Home's brief
  and headline, which are not drawn while the playground runs, the TUI's
  "This is the playground: a scripted model and in-memory forge. Set
  temper up in your browser when you're ready to leave it.": the
  browser's banner (`../pages/welcome.md`, 2), whose way out is the browser's. Its project is
  a demonstration, and the banner is what the person needs of it.
  "Connecting", out of date and a sign-in not kept are drawn as on any
  launchpad, and the sections below as on any project.
- **The quiet line,** on the launchpad and a chat: " · " and
  "playground" (`tui-hint`) after the box's state, if any, and before
  the needs' count (`pages.md`, *The quiet line*). A chat's top line is as on
  any chat.
- **Read again:** a playground another client started reaches the TUI
  through the forge's side, which reads the playground again whenever
  the `forges` watch gains or loses the playground's fake row (`../forge/family.md`, 7);
  `me` then names its project, and the host goes to its launchpad
  (`README.md`, 3, *Its pages*). Reset from the browser, `me` names the project no
  more: the host goes back to the start's address, and in front the
  welcome stands again, at its step.

**Quitting**, illustrative (the request's 04):

```
────────────────────────────────────────────────────────────────────────────────
❯
────────────────────────────────────────────────────────────────────────────────
Stop the box? csv-export's running task stops with it.
❯ 1. Stop the box
  2. Don't quit
```

- **Ctrl+C or `/quit`, in front:** with the question standing, the TUI
  closes; else, while work runs (`work_runs`, below), the question
  stands; else the TUI closes at once. The box stops when its front
  exits, draining as its first termination signal does
  (`../../protocol/setup/box.md`, 6).
- **The question,** a list under the box in the quiet line's place
  (`tui-list`), before any other list: its head (`tui-list-head`)
  "Stop the box? TITLE's running task stops with it." while one chat
  works, TITLE its title, "Stop the box? N running tasks stop with it."
  while more do, and "Stop the box?" once none does; then "1. Stop the
  box" and "2. Don't quit" (`tui-option`), the first active. While it
  stands the box keeps the focus: Up and Down move the active choice,
  Enter takes it, Esc is "Don't quit", and Ctrl+C or `/quit` again
  closes. "Stop the box" closes; "Don't quit" lowers it.
- **`work_runs`,** `temper_client_tui::work_runs`, from what the root
  already reads, no fact of the engine's added: a chat of the TUI's
  project works, of Home's chats (`../pages/home.md`, 1, `Chats::live`) while they
  are read, or the open chat. A chat works while it has not ended and
  its status (`jig_client::facts::chat_status`) is starting, writing, a
  tool's call, working or retrying, or while a delegate of it lives
  (its chip's `live_delegates`). The open chat's title, else the newest
  working chat's, is the head's TITLE.
- **Given up:** once the box gave up (below), Ctrl+C and `/quit` close at
  once, asking nothing: nothing runs.
- **Out of front,** Ctrl+C and `/quit` close as `README.md`, 4 says, and a box run
  on its own goes on running.
- **Their words in front:** `/quit`'s, "Close temper and stop the box";
  `/help`'s key, "Ctrl+C closes temper and stops the box".

**The box's state and `/log`**, illustrative (the request's 05):

```
 Where you left off
   csv-export, draft · working

 /log
   10:41:02 forgejo exited 1: database is locked
   10:41:03 forgejo restarting, 2 of 5 within 10 min
   10:41:05 forgejo listening on 127.0.0.1:8368

────────────────────────────────────────────────────────────────────────────────
❯ Ask temper
────────────────────────────────────────────────────────────────────────────────
 / for commands · forgejo-3 restarting          Claude Opus 5.5 · high · ● Live
```

- **The box's state,** on the quiet line, first at its right (`pages.md`,
  *The quiet line*): "STEP restarting" (`tui-signal`) from the box's `restarting` line
  until its `running` line for that step, for the step it last said
  restarting; once it gave up, "STEP gave up" (`tui-down`). A notice
  stands in its place for its time (`pages.md`, *The notice*). STEP is as
  the box names it, its bytes read as UTF-8 with a replacement for each
  invalid one (`forge::words::lossy`) and cut at 64 bytes,
  `limits::STEP`.
- **A give up:** the box stops its steps, says `gave-up`, and waits for
  its front (`../../protocol/setup/box.md`, 6). The TUI stays open and says
  so in a note of its own, after the others and standing for good: "The
  box gave up: STEP. /log shows why; Ctrl+C closes." (`tui-refused`).
  The link goes offline as the engine stops.
- **`/log`,** a command of the TUI's own, never the root's: offered in
  front alone, after `/help` and before `/quit`, its words "The box's
  recent log"; in the commands' list and in `/help`'s note there.
  Pressed, or sent whole or begun, it never reaches the root: the box's
  words are cleared (`terminal::Act::Dismiss`) and its note stands until
  the next send or screen.
  The note, after the others: the head "/log" (`tui-head`) and, two
  columns in, the box's last lines, oldest first, each as the box wrote
  it, read as UTF-8 with a replacement for each invalid byte and cut at
  512 bytes, `limits::LOG_LINE` (`tui-note`); with none kept yet,
  "Nothing yet." in their place. The lines are a log (skein's `ui/README.md`,
  3.2) whose end request is the TUI's state's revision, so that it moves
  as the note opens and as lines come while it stands, and the body
  shows its end, the newest lines, as many as fit; PgUp reads back the
  rest until the next change (skein's `ui/screen.md`, 5.2).

**The place.** In front, the TUI is on a box: it names its place
"box" at the default port, 8367, and "box:PORT" at another,
`temper_client_tui::place(address, true)`, as the look request
`tui-chat-title-below` names a box's; the welcome draws it at its
header's border end with no project, as the frames show.

**Its state, events, words and tokens:**

- **The TUI's state** (`pages.md`) holds, beside the rest: whether it is in
  front; the printed link; the box's last lines; the step restarting;
  the step given up; whether the quit question stands; and whether
  `/log`'s note stands. `Tui::box_state` says running, restarting or
  given up, given up winning.
- **Its events,** beside `README.md`, 3's:

  | Event | Does |
  |---|---|
  | `Stop` | "Stop the box" taken: closes |
  | `Stay` | "Don't quit" taken, or Esc on the question: the question lowered |
  | `Log` | `/log` pressed or sent: its note stands, the box's words cleared |

  Each marks the frame it acts on in a recording, as any key with an
  event but typing does (`journeys.md`, 3.3).
- **Its words,** every one quoted in this section, are the TUI's own,
  under its owner (skein's `ui/README.md`, section 9), but "temper box" and the
  steps' labels, the view's.
- **Its tokens,** under the TUI's owner, after `theme.md`'s, in its table's
  form:

  | Token | Paint | Else | Carried by |
  |---|---|---|---|
  | `tui-step-done` | none | the mark `✓`, good | "Signed in as the owner", a step ticked |
  | `tui-step-now` | bold | the mark `▸`, accent | the step under way |
  | `tui-step-ahead` | dim | the mark `·`, faint | a step not reached |
  | `tui-playground` | accent | aligned to the end | "playground" in a title |
  | `tui-end-faint` | faint | aligned to the end | " · " between it and the place |

  The rest is `theme.md`'s: `tui-signal` carries a step restarting too,
  `tui-down` a step given up, `tui-refused` the give up's note, and
  `tui-head`, `tui-hint` and `tui-note` what this section says.
- **The recording:** the start fact `front` and the record `box-line`,
  a line of the box's as `BoxLine::print` prints it; the secret of a
  `box-line` record's `link` is a *link* original (`journeys.md`, 3.2
  and 5). A journey of a box with the TUI in front is
  `journeys.md`, 8.3.
- **Its tests,** beside `../testing.md`'s: golden frames at 80 by 24, dark,
  of the welcome before and after the sign-in, the playground's
  launchpad, the question, and the box's state with `/log`'s note; rows
  of the state for each line of the box's and the question; of the view
  for each step's mark and the forge under way, the playground's
  header and quiet line, and `/log` offered in front alone; of the
  host, for the handed link with and without a credential held, each
  line told, Ctrl+C and `/quit` with and without work and after a give
  up, `/log` sent, and the host back at the start's address; of the run,
  for the lines read and their end closing it; of the recording's codec
  and redaction, for `front`; of the domain, for `signed_in` and the
  playground read again; and the joint world's (`../../testing/checks.md`,
  3): a TUI in front signed in from its handed link with no key,
  drawing the welcome; the printed link signing a browser in, which
  ticks "Signed in as the owner"; the playground started in the browser,
  in the TUI's header and quiet line; and lines of the box's, its state
  and `/log`.
