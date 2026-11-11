# The TUI in front of a box

`temper box`, run in a terminal, runs the TUI at once and the box
behind it, as its child: it finds `temper-box` of its own build,
downloading it once if it must (`../box-binary.md`), starts `temper-box
--front`, signs in by the link the box hands it, and stops the box when
the person quits (`../../draft/cli.md`, section 5). The box's side is
`../../protocol/setup/box.md`, 6. The TUI is the one of `README.md`, 1
to 3 and `pages.md` and `theme.md` and `README.md`, 4 but where this
section says otherwise. Its pages are drawn from the user's hand-made
frames of the look request `tui-box-front` (2026-11-08), at 80 by 24 in
the dark theme, and are illustrative, as `pages.md`'s are; the box's
start and stop are this section's own.

- **When:** `temper box [ARGUMENT...]` with its standard input and output
  each a terminal and no `--echo steps` among its arguments; otherwise
  it runs the box in its own place (`../box-binary.md`, 5).
- **Its terminal:** the TUI runs on descriptors 0 and 1, as bare
  `temper` does (`README.md`, 4), from the first moment: raw, Ctrl+C a
  key, its `LEAVE` given to the fail-stop hook. The box never touches
  the terminal. Its directory, note, scheme and depth are bare
  `temper`'s (`README.md`, 2, `theme.md`).
- **The box's start,** in the run, in this order, each told to the TUI
  as its stage (*The box's stages*, below):
  1. **Finding:** the run spawns `temper internal fetch-box --cache
     CACHE` (`../box-binary.md`, 4), its own executable, CACHE resolved
     from the arguments and the environment as the box resolves it, with
     the captured environment, the null device on its stdin, and pipes
     on its stdout and stderr. Each `fetched` line is told as the
     download's progress; `found PATH` names the box; its stderr, at
     most 8 KiB, is kept for the failure's words.
  2. **Starting:** on `found PATH` and exit 0, the run spawns `PATH
     --front ARGUMENT...`, the arguments unchanged, with the captured
     environment and working directory, in a process group of its own as
     skein's spawn makes every child's (skein-io's `kernel::Spawn`),
     handing it five descriptors (`../../protocol/setup/box.md`, 6): 0,
     the read end of a pipe whose write end the run holds and never
     writes, its life; 1 and 2, the write ends of two pipes the run reads
     into the box's one tail, its last 8 KiB, in the order their reads
     come; 3 and 4, the write ends of the link's pipe and the lines'
     pipe. The run keeps no write end it handed. Since the box is not in
     `temper`'s group, no signal sent to `temper`'s group or from its
     terminal reaches it: it stops by its life pipe's end, or the second
     stop's `SIGTERM`, alone.
  3. **Running:** the run reads the link's pipe to its end, at most
     4,096 bytes: the local link's URL, `http://HOST:PORT/...`, whose
     authority is the engine's address. The run makes no connection to
     the engine before it: it connects then and signs in by it (below).
     A link empty, past the bound, or not `http://` with a host and a
     port is the box's start failed, "temper-box handed no link", and the
     run stops the box as at a quit.
  A fetch that ends without `found`, or a spawn that fails, is the
  start failed, its why the fetch's first stderr line or the spawn's
  error.
- **Signed in by the handed link,** with nothing to read, paste or
  press. The local link's first secret is the box's, given to serve
  before it starts and to the TUI on the link's pipe (jig's
  `protocol/client/sign-in.md`, *The local link*;
  `../../protocol/setup/box.md`, 1). The run answers the host's first
  duty `local` with it, `Report::Local`, once it has read it, and every
  later one from the engine's link file of the box's data directory
  (`--data` among the arguments, else as the directories resolve), as
  `README.md`, 2 says. In front the host signs in by it at `Start`,
  whatever credential the directory holds and whatever its note says:
  the root starts holding no credential, and the link's sign-in carries
  the one held, `SignIn { with: Link { secret, held } }`, so that a
  sign-in the engine still holds is kept and no new one is made, and a
  credential the engine does not hold signs in afresh (`../README.md`,
  section 4; jig's `protocol/client/sign-in.md`, *A sign-in held is
  kept*). While it signs in, the welcome is drawn, its state
  "Connecting". It ends as `pages.md`'s *Signed in from the engine's
  link* says: `link_used` or `link_expired` reads the engine's link file
  a second later. Not ready and busy wait for jig's resend after its
  backoff, still "Connecting", because the box starts serve after it
  hands the link. Any other end falls back to the sign-in screen.
- **The seam** (`README.md`, 3) gains the front's: `Start.front`, true in
  front, `Start.local` true with it; `Report::Front(BoxLine)`, a line of
  the box's, read; and `Report::Stage(BoxStage)`, the box's stage, which
  the host tells the TUI's state. At `Start` in front the host tells the
  TUI's state it is in front, its stage `Finding`. The host's duties
  gain the box's stop and its second stop, which the run carries out
  (*Quitting*, below).
- **The box's lines,** one a line on the lines' pipe
  (`../../protocol/setup/box.md`, 6): a word, one space, a field that
  runs to the line feed, bytes with spaces and no escapes, never empty;
  at most 8,192 bytes with the line feed, `BoxLine::MOST`. A line past
  it, with an unknown word, or with no field after its word and a space
  is dropped. `temper_client_tui::BoxLine` reads one and prints it back:

  | Line | The TUI |
  |---|---|
  | `link URL` | the printed link, the latest replacing the one before (*The welcome*) |
  | `log TEXT` | one of the box's last lines, at most 50 kept, `limits::BOX_LOG`, the oldest dropped (*`/log`*) |
  | `restarting STEP` | the box restarts STEP (*The box's state*) |
  | `running STEP` | STEP runs again |
  | `gave-up STEP` | the box gave up, for good |

  The run reads them on its own io, beside its signals, as they come,
  since the box holds at most 256 the TUI has not taken, and reports
  each as `Report::Front` in the order read, once `Start` was given,
  after the credential's write and before the transport's events. The
  TUI's state keeps the printed link, the last lines, the step
  restarting and the step given up, in memory alone; of them the printed
  link alone is a secret, and it is drawn on the welcome, nowhere else.
- **The box's end:** the box's exit, read by the run with the lines'
  pipe's end, is told as `Ended`, with its exit code or its signal.
  Asked for (*Quitting*), it closes the TUI; unasked, the TUI stays open
  and says so (*A give up and an end*, below).
- **Its pages:** the welcome while `me` names no project, its link's place
  holding the box's start until the box's first `link` line; then the
  launchpad and a chat as `pages.md` draws them, with the playground's
  words where the project is the playground's. The sign-in screen only
  where the handed link's sign-in fell back.
- **Its exit,** once the TUI has left the terminal (`README.md`, 4):
  - 0, the box stopped at the person's quit and exited 0, printing
    nothing;
  - the start failed: `temper: ` and the fetch's stderr as it wrote it,
    the hand-fetch line among them (`../box-binary.md`, 3), or `temper:
    temper-box handed no link` or the spawn's error, then the box's tail
    if it ran, exit 1;
  - the box ended otherwise: `temper: temper-box exited N`, or `killed by
    signal N`, its tail under it, and `Log: DATA/box/logs/box.log`, on
    stderr, exit N for an exit, 1 for a signal;
  - a termination signal of `temper`'s own closes the run as
    `README.md`, 4 says, exit 1: the box's life pipe closes with
    `temper`, and the box stops by itself
    (`../../protocol/setup/box.md`, 6).

**The box's stages**, illustrative (the box's start, then its stop):

```
╭─ temper ─────────────────────────────────────────────────────────────── box ─╮
│ temper box · on this machine, to try it                                      │
╰──────────────────────────────────────────────────────────────────────────────╯

 Welcome. Set temper up in your browser: open this link on this machine.

   Downloading temper-box, once for this build: 12.3 of 24.1 MB

 In the browser
   1 Welcome · 2 Forge · 3 Model · 4 Ready
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
- **On the welcome,** before the box's first `link` line, the link's
  place, two columns in, holds the stage's line in the link's stead, and
  "It signs you in…" is left out: "Finding temper-box…", "Downloading
  temper-box, once for this build: DONE of TOTAL MB", or with no size
  "Downloading temper-box, once for this build: DONE MB", DONE and TOTAL
  in millions of bytes with one decimal, and "Starting the box…", each
  `tui-hint`; and once failed, "temper-box could not start: WHY"
  (`tui-refused`), WHY read as UTF-8 with a replacement for each invalid
  byte, and under it "Ctrl+C closes; temper then says what to do."
  (`tui-hint`). Running with no `link` line yet draws nothing there.
- **The box's steps as they come:** until the TUI is signed in by the
  handed link, the box's last `log` line, if any, stands under the
  welcome's link section, two columns in (`tui-hint`), cut to the
  window's columns, so the person sees the box start its steps; once
  signed in, it is `/log`'s alone.
- **Stopping:** from "Stop the box" taken, or a quit with nothing to
  ask (*Quitting*), until the box has ended: a note of its own, after the
  others, standing for good: the head "Stopping the box…" (`tui-head`),
  then, two columns in, "Ctrl+C again ends its steps at once."
  (`tui-hint`) and the box's last lines as `/log` draws them, its end
  following them. The box's state on the quiet line says "stopping"
  (`tui-signal`). The box's words stay, taking nothing but commands.

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
    the link's place holds the box's stage (*The box's stages*, above),
    and its line is left out. The TUI never opens the link nor
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

- **Ctrl+C or `/quit`, in front, while the box runs:** with the question
  standing, the box stops; else, while work runs (`work_runs`, below),
  the question stands; else the box stops at once. **The box stops**
  when the host asks it, a duty the run carries out by closing the box's
  life pipe, which the box takes as its first termination signal,
  draining (`../../protocol/setup/box.md`, 6): the stage is `Stopping`,
  its note stands (*The box's stages*), and the TUI closes once the box
  has ended. While it stops, Ctrl+C or `/quit` again is the second
  stop: the run sends the box `SIGTERM`, its second termination signal,
  which closes its steps, and the TUI still waits for its end.
- **Before the box runs,** while it is found, fetched or started, and
  once its start failed, Ctrl+C and `/quit` stop what runs, the fetch's
  child by `SIGTERM` or the box as above, and close once it has ended.
- **The question,** a list under the box in the quiet line's place
  (`tui-list`), before any other list: its head (`tui-list-head`)
  "Stop the box? TITLE's running task stops with it." while one chat
  works, TITLE its title, "Stop the box? N running tasks stop with it."
  while more do, and "Stop the box?" once none does; then "1. Stop the
  box" and "2. Don't quit" (`tui-option`), the first active. While it
  stands the box keeps the focus: Up and Down move the active choice,
  Enter takes it, Esc is "Don't quit", and Ctrl+C or `/quit` again
  stops the box. "Stop the box" stops it; "Don't quit" lowers it.
- **`work_runs`,** `temper_client_tui::work_runs`, from what the root
  already reads, no fact of the engine's added: a chat of the TUI's
  project works, of Home's chats (`../pages/home.md`, 1, `Chats::live`) while they
  are read, or the open chat. A chat works while it has not ended and
  its status (`jig_client::facts::chat_status`) is starting, writing, a
  tool's call, working or retrying, or while a delegate of it lives
  (its chip's `live_delegates`). The open chat's title, else the newest
  working chat's, is the head's TITLE.
- **Ended:** once the box has ended unasked, given up or not (below),
  Ctrl+C and `/quit` close at once, asking nothing: nothing runs.
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
- **A give up and an end:** the box stops its steps, says `gave-up`, and
  exits 1 (`../../protocol/setup/box.md`, 6). The TUI stays open and says
  so in a note of its own, after the others and standing for good: "The
  box gave up: STEP. /log shows why; Ctrl+C closes." (`tui-refused`).
  The link goes offline as the engine stops. A box that ends unasked
  otherwise, its stage `Ended` with no `gave-up` line, gets the same
  note, "The box stopped: exited N. /log shows why; Ctrl+C closes." or
  "The box stopped: killed by signal N. /log shows why; Ctrl+C closes.",
  and the quiet line's state "box stopped" (`tui-down`); a give up's
  words win.
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
  front; the box's stage; the printed link; the box's last lines; the
  step restarting; the step given up; whether the quit question stands;
  and whether `/log`'s note stands. `Tui::box_state` says running,
  restarting, stopping, given up or stopped, given up winning, then
  stopped, then stopping.
- **The stage's surface,** what the host gives the state:

  ```rust
  // temper_client_tui
  /// Where `temper box`'s box stands (*The box's stages*).
  pub enum BoxStage {
      Finding,
      Fetching { done: u64, total: Option<u64> },
      Starting,
      Running,
      Stopping,
      Ended { end: BoxEnd },
      Failed { why: Box<[u8]> },
  }
  pub enum BoxEnd { Exited { code: u8 }, Killed { signal: u8 } }
  impl BoxStage {
      /// `finding`, `fetching DONE TOTAL` (`-` for no size), `starting`,
      /// `running`, `stopping`, `ended exited N`, `ended killed N`,
      /// `failed WHY`: the recording's words.
      pub fn print(&self) -> Box<[u8]>;
      /// The words `print` gives back; None for any other.
      pub fn read(words: &[u8]) -> Option<BoxStage>;
  }
  // state::Told gains
  Told::Stage { stage: BoxStage }
  ```

  A stage told after `Ended` or `Failed` is ignored, but `Ended` after
  `Failed`, the box's end once its start failed.
- **Its events,** beside `README.md`, 3's:

  | Event | Does |
  |---|---|
  | `Stop` | "Stop the box" taken: the box stops |
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

  The rest is `theme.md`'s: `tui-signal` carries a step restarting and
  the box stopping too, `tui-down` a step given up and the box stopped,
  `tui-refused` the give up's and the end's notes and a failed start,
  and `tui-head`, `tui-hint` and `tui-note` what this section says.
- **The recording:** the start fact `front`; the record `box-line`, a
  line of the box's as `BoxLine::print` prints it, the secret of a
  `box-line` record's `link` a *link* original (`journeys.md`, 3.2 and
  5); and the record `box-stage`, the stage as `BoxStage::print` prints
  it. The handed link is never recorded: its sign-in is the host's
  `local` duty answered, as any. A journey of a box with the TUI in front
  is `journeys.md`, 8.3.
- **Its tests,** beside `../testing.md`'s: golden frames at 80 by 24, dark,
  of the welcome before and after the sign-in, the welcome at each stage
  before the box's link (finding, fetching with and without a size,
  starting with a line of the box's, failed), the playground's
  launchpad, the question, the box's state with `/log`'s note, stopping,
  and an end unasked; rows of the state for each line of the box's, each
  stage and the question, and of `BoxStage`'s print and read; of the view
  for each step's mark and the forge under way, the playground's header
  and quiet line, and `/log` offered in front alone; of the host, for the
  handed link with and without a credential held, each line and stage
  told, Ctrl+C and `/quit` with and without work, while stopping, before
  the box runs and after an end, `/log` sent, and the host back at the
  start's address; of the run, with fakes for its children and pipes,
  for the fetch's lines, `found` and its failure, the box's spawn and its
  descriptors, the link read and refused, the lines read, the life pipe
  closed at the stop, `SIGTERM` at the second stop, and the box's end;
  of the recording's codec and redaction, for `front` and `box-stage`;
  of the domain, for `signed_in` and the playground read again; and the
  joint world's (`../../testing/checks.md`, 3): a TUI in front signed in
  from its handed link with no key, drawing the welcome; the printed
  link signing a browser in, which ticks "Signed in as the owner"; the
  playground started in the browser, in the TUI's header and quiet line;
  and lines of the box's, its state and `/log`. The process tier, written
  and compiled, run after landing: `temper box` on a pseudo-terminal
  with a `temper-box` beside it, the local link on its pipe signing the
  TUI in, Ctrl+C with "Stop the box" stopping the box and `temper`
  exiting 0, and `temper` killed stopping the box.
