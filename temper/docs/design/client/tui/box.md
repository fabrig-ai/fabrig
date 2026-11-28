# The TUI in front of a box

## 1. Running in front of a box

`temper box`, run in a terminal, runs the TUI at once and the box
behind it, as its child: it finds `temper-box` of its own build,
downloading it once if it must (`../box-binary.md`), starts `temper-box
--front`, signs in by the link the box hands it, and stops the box when
the person quits (`../../draft/cli.md`, section 5). The box's side is
`../../protocol/setup/box-front.md`, 1. The TUI is the one of `README.md`, 1, and `directory.md`, 1, and `host.md`, 1
 and `pages/README.md` and `theme.md` and `README.md`, 4 but where this
section says otherwise. Its pages are drawn from the user's hand-made
frames of the look request `tui-box-front` (2026-11-08), at 80 by 24 in
the dark theme, and are illustrative, as `pages/README.md`'s are; the box's
start and stop are this section's own.

- **When:** `temper box [ARGUMENT...]` with its standard input and output
  each a terminal and no `--echo steps` among its arguments; otherwise
  it runs the box in its own place (`../box-binary.md`, 5).
- **Its terminal:** the TUI runs on descriptors 0 and 1, as bare
  `temper` does (`README.md`, 4), from the first moment: raw, Ctrl+C a
  key, its `LEAVE` given to the fail-stop hook. The box never touches
  the terminal. Its directory, note, scheme and depth are bare
  `temper`'s (`directory.md`, 1, `theme.md`).
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
     handing it five descriptors (`../../protocol/setup/box-front.md`, 1): 0,
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
  `directory.md`, 1 says. In front the host signs in by it at `Start`,
  whatever credential the directory holds and whatever its note says:
  the root starts holding no credential, and the link's sign-in carries
  the one held, `SignIn { with: Link { secret, held } }`, so that a
  sign-in the engine still holds is kept and no new one is made, and a
  credential the engine does not hold signs in afresh (`../sign-in.md`,
  section 1; jig's `protocol/client/sign-in.md`, *A sign-in held is
  kept*). While it signs in, the starting screen is drawn on a box set
  up before, and the welcome on a first run, its state "Connecting"
  (`box-startup.md`, 2). It ends as `pages/sign-in.md`, section 2 says: `link_used` or `link_expired` reads the engine's link file
  a second later. Not ready and busy wait for jig's resend after its
  backoff, still "Connecting", because the box starts serve after it
  hands the link. Any other end falls back to the sign-in screen.
- **A box set up before,** the user's request `tui-11` of 2026-11-15:
  before it spawns the fetch, the run looks once at the box's
  configuration directory, CONFIG resolved from the arguments and the
  environment as the box resolves it (`../box-binary.md`), as it
  resolves DATA and CACHE for a lend (*Stepping aside*): absent, or a
  directory with no entry, is a first run, as the box takes it
  (`../../protocol/setup/box.md`, 1, *An empty configuration
  directory*); a directory with an entry, or one it cannot read, is a
  box set up before. The run decides it, not a line of the box's,
  because the first frame comes before the box runs, a download's time
  before on a new build, and a first run's welcome must stand from it.
  The box's log's path, `DATA/box/logs/box.log`, the one its exit's
  `Log:` line names, is told beside it.
- **The seam** (`host.md`, 1) gains the front's: `Start.front`, true in
  front, `Start.local` true with it; `Start.set_up`, true in front on a
  box set up before, and `Start.box_log`, the box's log's path in front
  and empty out of it:

  ```rust
  // temper_client_shell_tui::host::Start gains
  pub set_up: bool,                               // in front, on a box set up before
  pub box_log: Box<[u8]>,                         // in front, DATA/box/logs/box.log; else empty
  ```

  `Report::Front(BoxLine)`, a line of
  the box's, read; and `Report::Stage(BoxStage)`, the box's stage, which
  the host tells the TUI's state. At `Start` in front the host tells the
  TUI's state it is in front, its stage `Finding`, whether the box was
  set up before and its log's path. The host's duties
  gain the box's stop and its second stop, which the run carries out
  (*Quitting*, below).
- **The box's lines,** one a line on the lines' pipe
  (`../../protocol/setup/box-front.md`, 1): a word, one space, a field that
  runs to the line feed, bytes with spaces and no escapes, never empty;
  at most 8,192 bytes with the line feed, `BoxLine::MOST`. A line past
  it, with an unknown word, or with no field after its word and a space
  is dropped. `temper_client_tui::BoxLine` reads one and prints it back:

  | Line | The TUI |
  |---|---|
  | `link URL` | the printed link, the latest replacing the one before (`box-startup.md`, 3) |
  | `log TEXT` | one of the box's last lines, at most 50 kept, `limits::BOX_LOG`, the oldest dropped (*`/log`*) |
  | `restarting STEP` | the box restarts STEP (*The box's state*) |
  | `running STEP` | STEP runs again |
  | `gave-up STEP` | the box gave up, for good |
  | `starting NAME` | the box's group NAME starts, `Forgejo` or `Playground` (*The box's state*) |
  | `started NAME` | NAME is started: the notice "NAME started" |
  | `dormant NAME` | NAME runs nothing again, started or not |

  The run reads them on its own io, beside its signals, as they come,
  since the box holds at most 256 the TUI has not taken, and reports
  each as `Report::Front` in the order read, once `Start` was given,
  after the credential's write and before the transport's events. The
  TUI's state keeps the printed link, the last lines, the step
  restarting, the step given up and the groups starting, in memory alone; of them the printed
  link alone is a secret, drawn on the welcome and in `/web`'s note and
  given to the opener by `/web`, nowhere else (*`/web`, temper in the
  browser*).
- **The box's end:** the box's exit, read by the run with the lines'
  pipe's end, is told as `Ended`, with its exit code or its signal.
  Asked for (*Quitting*), it closes the TUI; unasked, the TUI stays open
  and says so (*A give up and an end*, below).
- **Its pages:** on a box set up before, the starting screen until the
  TUI has read `me` (`box-startup.md`, 2); the welcome while
  `me` names no project, its link's place holding "The link comes here
  once the box has started." until the box's first `link` line; then the
  launchpad and a chat as `pages/README.md` draws them, with the playground's
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
    (`../../protocol/setup/box-front.md`, 1).

## 6. The playground

```

  ▄█▀▀   temper 0.0.0 (b948f40d6df1)
 ▄█▄█▄█  temper box · a whole deployment on this machine

 This is the playground: a scripted model and in-memory forge. Set
 temper up in your browser when you're ready to leave it.
...
────────────────────────────────────────────────────────────────────────────────
❯ Ask temper
────────────────────────────────────────────────────────────────────────────────
 ● Live · Hello · Scripted model                                     playground
```

- **Where:** wherever the TUI's project, the launchpad's or the open
  chat's, is the running playground's (`../forge/family.md`, 7: its read's `project` while
  it runs), in front of a box or not. The TUI works in it as in any
  project.
- **The headline,** on the launchpad (`pages/launchpad.md`, 3): in the
  place of Home's brief and headline, which are not drawn while the
  playground runs, the TUI's "This is the playground: a scripted model
  and in-memory forge. Set temper up in your browser when you're ready
  to leave it.": the browser's banner (`../pages/welcome.md`, 2), whose
  way out is the browser's. Its project is a demonstration, and the
  banner is what the person needs of it. "Connecting", out of date and
  a sign-in not kept are drawn as on any launchpad, and the sections
  below as on any project. The header names no playground
  (`pages/README.md`, 3): the quiet line does.
- **The quiet line,** on the launchpad and a chat: " · " and
  "playground" (`tui-hint`) after the box's state, if any, and before
  the needs' count (`pages/README.md`, 1, *The quiet line*). A chat's top line is as on
  any chat.
- **Read again:** a playground another client started reaches the TUI
  through the forge's side, which reads the playground again whenever
  the `forges` watch gains or loses the playground's fake row (`../forge/family.md`, 7);
  `me` then names its project, and the host goes to its launchpad
  (`host.md`, 1, *Its pages*). Reset from the browser, `me` names the project no
  more: the host goes back to the start's address, and in front the
  welcome stands again, at its step.

## 7. Quitting

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
  draining (`../../protocol/setup/box-front.md`, 1): the stage is `Stopping`,
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

## 8. The box’s state and log

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
 ● Live · Hello · Claude Opus 5.5 · high                   forgejo-3 restarting
```

- **The box's state,** on the quiet line, first at its right (`pages/README.md`, 1, *The quiet line*): "STEP restarting" (`tui-signal`) from the box's `restarting` line
  until its `running` line for that step, for the step it last said
  restarting; once it gave up, "STEP gave up" (`tui-down`). A notice
  stands in its place for its time (`pages/README.md`, 1, *The notice*). STEP is as
  the box names it, its bytes read as UTF-8 with a replacement for each
  invalid one (`forge::words::lossy`) and cut at 64 bytes,
  `limits::STEP`.
- **A group starting,** the user's request `tui-11` of 2026-11-15:
  "NAME starting…" (`tui-hint`) from the box's `starting NAME` line
  until its `started NAME` or `dormant NAME` line, NAME as the box
  names it, `Forgejo` or `Playground`, read and cut as STEP is. The
  TUI keeps the names starting in the order their lines came, a name
  once, at most 4, `limits::STARTING`, a line past them dropped; the
  state says the first. A step restarting, the box stopping, stopped
  or given up win over it: `Tui::box_state` says given up, then
  stopped, then stopping, then restarting, then starting. On the
  starting screen, the welcome, the launchpad and a chat alike, so that
  a box set up before says its Forgejo starting wherever the TUI is.
- **A group started:** at `started NAME` the host gives the notice
  "NAME started" (`Notice::Started { name }`, `pages/README.md`, 1,
  *The notice*), "Forgejo started"; `dormant NAME` gives none, the
  group's failure being the browser's to say (`box-startup.md`, 3, *The forge
  under way*) and `/log`'s.
- **A give up and an end:** the box stops its steps, says `gave-up`, and
  exits 1 (`../../protocol/setup/box-front.md`, 1). The TUI stays open and says
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
  "Nothing yet." in their place. The lines are a log (skein's `ui/tree.md`,
  1.2) whose end request is the TUI's state's revision, so that it moves
  as the note opens and as lines come while it stands, and the body
  shows its end, the newest lines, as many as fit; PgUp reads back the
  rest until the next change (skein's `ui/screen.md`, 5.2).

## 10. The place

**The place.** In front, the TUI is on a box: it names its place
"box" at the default port, 8367, and "box:PORT" at another,
`temper_client_tui::place(address, true)`, as the look request
`tui-chat-title-below` names a box's, in the rule above a chat's box.
The header says "temper box" in its stead (`pages/README.md`, 3).

## 11. State, events, words and tokens

**Its state, events, words and tokens:**

- **The TUI's state** (`pages/README.md`) holds, beside the rest: whether it is in
  front; whether the box was set up before, and its log's path; whether
  `me` was read once in its run, which ends the starting screen; the
  box's stage; the printed link, and whether it is live; the forges
  document's `linked` as last told; the box's last lines; the
  step restarting; the step given up; the groups starting; whether the
  quit question stands; and whether `/log`'s note and `/web`'s stand.
  `Tui::box_state` says running, starting, restarting, stopping, given
  up or stopped, given up winning, then stopped, then stopping, then
  restarting, then starting.
- **The lines' surface:** `BoxLine` gains `Starting { name }`,
  `Started { name }` and `Dormant { name }`, each `name: Box<[u8]>`,
  read and printed as `starting NAME`, `started NAME` and `dormant
  NAME`; `Notice` gains `Started { name: Box<[u8]> }`, its words "NAME
  started"; `limits::STARTING`, 4.
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
- **Its events,** beside `host.md`, 1's:

  | Event | Does |
  |---|---|
  | `Stop` | "Stop the box" taken: the box stops |
  | `Stay` | "Don't quit" taken, or Esc on the question: the question lowered |
  | `Log` | `/log` pressed or sent: its note stands, the box's words cleared |

  Each marks the frame it acts on in a recording, as any key with an
  event but typing does (`journeys.md`, 3.3).
- **Its words,** every one quoted in this section, are the TUI's own,
  under its owner (skein's `ui/README.md`, section 9), but "temper box", the
  steps' labels and Model's ACCOUNT, the view's.
- **Its tokens,** under the TUI's owner, after `theme.md`'s, in its table's
  form:

  | Token | Paint | Else | Carried by |
  |---|---|---|---|
  | `tui-step-done` | none | the mark `✓`, good | "Signed in as the owner", a step ticked |
  | `tui-step-now` | bold | the mark `▸`, accent | the step under way |
  | `tui-step-ahead` | dim | the mark `·`, faint | a step not reached |

  `tui-playground` and `tui-end-faint`, which carried "playground" and
  its " · " in the framed header's border, went with it (the user's
  request `tui-14`). The rest is `theme.md`'s: `tui-signal` carries a step restarting and
  the box stopping too, `tui-down` a step given up and the box stopped,
  `tui-refused` the give up's and the end's notes and a failed start,
  and `tui-head`, `tui-hint` and `tui-note` what this section says.
- **The recording:** the start facts `front`, `set-up` and `box-log`; the record `box-line`, a
  line of the box's as `BoxLine::print` prints it, the secret of a
  `box-line` record's `link` a *link* original (`journeys.md`, 3.2 and
  5); and the record `box-stage`, the stage as `BoxStage::print` prints
  it. The handed link is never recorded: its sign-in is the host's
  `local` duty answered, as any. `/web`'s duty `open` is recorded as any,
  a `browse` holding the printed link's URL, whose secret is the same
  *link* original, its URL-base64 replaced in every field
  (`journeys.md`, 5), and `/web`'s note wraps the link as the welcome
  does, redacted across its rows (`journeys.md`, 8.3, *The welcome's
  link, wrapped*). The front host and what a journey
  driver still needs are `journeys.md`, 8.3.
- **Its tests,** beside `../testing.md`'s: golden frames at 80 by 24, dark,
  of the welcome before and after the sign-in, the welcome at each stage
  before the box's link (finding, fetching with and without a size,
  starting with a line of the box's, failed), the starting screen with
  the box starting, with temper starting and Forgejo starting on the
  quiet line, and failed, the playground's
  launchpad, the question, the box's state with `/log`'s note, stopping,
  and an end unasked; rows of the state for each line of the box's, each
  stage and the question, and of `BoxStage`'s and `BoxLine`'s print and read; of the view
  for each step's mark and the forge under way, the starting screen's
  steps, the box's start on the quiet line, a group starting and its
  precedence, the playground's headline
  and quiet line, and `/log` offered in front alone; of the host, for the
  handed link with and without a credential held, each line and stage
  told, `started` giving its notice, the starting screen giving way to
  the launchpad or the welcome at `me` and never standing again, a
  first run never drawing it, Ctrl+C and `/quit` with and without work, while stopping, before
  the box runs and after an end, `/log` sent, the host back at the
  start's address, and the key's order against a chat's end: a chat
  read working, then its work ended, then Ctrl+C stopping the box at
  once; a chat parked on a live delegate asking, the head naming it;
  the key read first, the question standing through the chat's end, its
  head then "Stop the box?"; and the facts the working sitting's TUI
  may read, a chat's start and then its wait with nothing read between:
  an Ask temper chat waiting on its person's words, never drawn
  "Working", Ctrl+C then stopping the box at once, and a chat started by
  `/change` waiting on its live delegate, drawn "waiting on 1
  delegate", each Ctrl+C read after it asking (section 7); of the run, with fakes for its children and pipes,
  for the configuration directory absent, empty and with an entry, the
  fetch's lines, `found` and its failure, the box's spawn and its
  descriptors, the link read and refused, the lines read, the life pipe
  closed at the stop, `SIGTERM` at the second stop, and the box's end;
  of the recording's codec and redaction, for `front`, `set-up`, `box-log` and `box-stage`;
  of the domain, for `signed_in` and the playground read again; and the
  joint world's (`../../testing/checks.md`, 3): a TUI in front signed in
  from its handed link with no key, drawing the welcome; the printed
  link signing a browser in, which ticks "Signed in as the owner"; the
  playground started in the browser, in the TUI's header and quiet line;
  and lines of the box's, its state and `/log`. Fuzzy, seeded: a chat's
  facts read in the order of its life (started, working, a tool's call,
  its delegate live, parked waiting on it, the delegate ended, parked
  with none) with Ctrl+C read at a seeded place among them, the
  question standing exactly where the chat worked by section 7's
  `work_runs` at the key's read and the box stopping at once elsewhere;
  and, its delegate never ending, every key read after the chat was
  first read working asking; and that life with each of its passing
  facts (writing, working, a tool's call) left unread by a seeded
  choice, as a watch delivers its latest document: the chat drawn
  "waiting on 1 delegate" once its wait is read, whatever was left
  unread, and every key read after that drawing asking. The process tier, written
  and compiled, run after landing: `temper box` on a pseudo-terminal
  with a `temper-box` beside it, the local link on its pipe signing the
  TUI in, Ctrl+C with "Stop the box" stopping the box and `temper`
  exiting 0, and `temper` killed stopping the box, watched to its end on
  macOS by skein-world's walk, as the box is not in `temper`'s group
  (`skein/docs/design/examples.md`, 6). The working sitting, which asks
  the question, starts its box with every landing held
  (`../../domain/landing-settings.md`, 1.3) and its chat with
  `/change Add CSV export to widget` (`pages/README.md`, 2): the
  project's chat template, whose charter delegates. Bare words on the
  launchpad ask Ask temper instead, whose charter has no delegate tool:
  the scripted model's `delegate` call is refused, no change is made,
  and the chat waits on its person's words within milliseconds of its
  start, so no hold keeps it working. Started by `/change`, the chat's
  delegated change is made and checked and then waits, ready to land,
  and the chat waits on its live delegate until the box stops. Ctrl+C is
  sent once the chat is drawn "waiting on 1 delegate", and meets the
  question on every run: that drawing stays while the delegate lives,
  and `work_runs` reads the chat as working at every read from the one
  that drew it. A story waits on a state that stays drawn, never on one
  that passes: a watch delivers its latest document, one delivery in
  flight, and the terminal draws its newest frame, so a chat's writing,
  milliseconds long under the scripted model, may never be drawn
  "Working", and a story waiting for it waits for ever. Without the
  hold, the change can land, and the chat park with none, before the
  TUI reads the key; the box then stops at once, asking nothing (section
  7), and the story waits for a question that never comes. Turned down:
  taking either end as a pass, which checks less; a slower model, which
  narrows the window without closing it; a held reply, as the
  playground's model is the box's own, with no hold a story reaches; the
  TUI keeping "Working" drawn a while, a change of what a person sees
  for a story's sake. The idle and killed sittings need no hold: no work
  runs in them.
- **`/web`'s tests** (*`/web`, temper in the browser*), focused: rows of
  the state for the printed link's life (a `link` line live; `linked`
  told false then true spending it; true told again spending nothing; a
  `link` line after it live again; a restart's false, a restarted
  serve's line, then true spending that one; none read as false); of
  the view for `/web` offered in front alone, on the launchpad, the
  welcome, a chat, a talk and a place not yet started, and not on the
  starting screen, the sign-in screen, out of front, or once the box
  stops, has stopped or gave up, at its place in the commands' list and
  `/help`'s note and as Start's last row; golden frames at 80 by 24,
  dark, of the launchpad with Start's `/web` row and of `/web`'s note
  with the link wrapped; of the host for `/web` pressed and sent whole
  with the link live, its duty `open` the link's URL, and spent, the
  project's Home and on the welcome `/`, nothing reaching the root, the
  opener's failure for the link raising the note and for a Home
  `/open`'s, a fresh `link` line making `/web` open the fresh one, and
  the forges document's `linked` told as it changes; and of the
  recording, `/web`'s `browse` and its note's wrapped link redacted and
  replayed with the first replay's proof holding.
- **Lending's tests** (*Lending its terminal*): golden frames at 80 by
  24, dark, of the welcome back with each of the three notices, and with
  Model's detail; rows of the host for the offer given once in front and
  never out of front, `lend` from a login named and none while lending
  or for a number taken, `unlend` once, and `Lent` giving `LoginEnded`
  and each notice; of the recording's codec for `lent` and the duties;
  of the run, with fakes for the child and the terminal, for `lend`'s
  order (the terminal lent, the modes set back, the clear and the line
  right after `LEAVE`'s bytes, the spawn's
  program, arguments, environment and inherited descriptors), the end
  `NotStarted` for a NAME refused, `SIGTERM` to the child's group at
  `unlend` and at a termination signal while lent, and the terminal
  taken back at the child's end; and the joint world's: the browser's
  press reaching a TUI in front as `lend`, its `Lent` at exit 0 making
  the source, Cancel reaching it as `unlend`, and the TUI's end turning
  the page to its form with no terminal. The process tier, written and
  compiled: `temper box` on a pseudo-terminal lending it to a fake
  `temper-box program-login` that reads a line and exits 0, and the
  TUI's frame back after it. A journey of it waits with `box-welcome`'s
  (`journeys.md`, 9).

## 12. Open questions

None.
