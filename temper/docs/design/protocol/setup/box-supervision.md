# The box's supervision

This document holds the box's supervision, split from [`box.md`](box.md).

## 1. What it does

- **In order:** each step starts once the one before it has started, when
  that one is long, or has exited 0, when it is once or link; the `source` step
  alone (`box.md`, 1) ends the group's start whichever way it ends, never failing it.
  A `source` step that did not make the source serve runs again while its
  group stands, after a wait doubling from `box.backoff_ms` to
  `box.backoff_max_ms`, until one does: the engine may be starting, or
  away, when Forgejo starts (`../openid-provider.md`, 9). A link
  step starts once the long step before it has written its first line
  to its stdout.
- **A link step** is given that line whole on its stdin, which is then
  closed; it is the link step's alone, and the box's stdout shows the
  long step's lines after it. A long step that exits before writing a
  line hands its restart's first line to the link step. A link step
  runs once per box: the long step's restarts never run it again.
- **Restarts** a long step that exits, after a backoff doubling from
  `box.backoff_ms` to `box.backoff_max_ms` (../configuration/limits.md),
  first running again the steps that are once just before it: a Forgejo is
  restarted after its preparation, so a change of the setup reaches its
  `app.ini`. `temper-box serve` restarts on its own, with a pipe made afresh,
  and its start takes a handover the preparation left and reconciles each
  managed and owned forge. A token revoked under a running engine stops
  that forge alone, named, and not the engine (../forge/README.md, a forge stopped);
  the engine's next start makes it again. A fake forge's restart is such a
  revoke: it serves its seed afresh, without the bot, so that forge stops
  alone until the engine's next start takes the handover its preparation
  drew again (`playground.md`).
- **Gives up,** exit 1, naming the step, when a once or link step fails,
  but a dormant group's before its `started`, which answers `failed`
  (`box.md`, 1), or a long one exits more than `box.restarts` times within
  `box.window_ms`. The backoff is counted from a step's exits within the
  window, so it starts again from `box.backoff_ms` once a step has run a
  window without exiting. Its stop treats a step still starting as the box's
  stop does. With `--front`, it then exits 1 (`box-front.md`, 1).
- **Stops** at a first termination signal, `SIGINT`, `SIGTERM` or a closed
  terminal's `SIGHUP`, or with `--front` its standard input's end (`box-front.md`, 1): the long steps in reverse
  order, each by a signal, `box.grace_ms`, a second signal, which turns
  `temper-box serve`'s drain into an abort (../binaries.md),
  `box.grace_ms` again, then a close. The machine knows a step only as
  once, long or link, so every long step, and a link step still
  running, is sent both; a dormant group's steps run nothing, and are
  sent nothing. A second termination
  signal closes them all. A step the box has not yet asked to start when the
  stop begins is not started. One it has asked for is waited on in its place
  in that order, and a child that starts then is stopped as above, or closed
  at once after a second termination signal; it is never readied, continued,
  recorded or restarted. The box cannot interrupt the system's creation of a
  process: that call blocks the box's one loop for as long as it lasts,
  without bound on a filesystem that stalls (skein's `runtime/kernel.md`, section 6),
  and a signal or a deadline is acted on only once it returns. It exits 0
  after a stop it was asked for, its standard input's end with `--front`
  among them (`box-front.md`, 1).
- **Stops a group** at `stop FORGE` or `remove FORGE` (`box.md`, 1): the group's running steps
  alone, as the box's stop does, in reverse order, each by a signal,
  `box.grace_ms`, a second signal, `box.grace_ms` again, then a close;
  a step that exits during it is not restarted, and a once step still
  running is closed with them; a step still starting is treated as at the
  box's stop. Once they are all closed the group is dormant, the box
  writes `<label>: stopped` to its own log (below), and answers
  `stopped FORGE`, or at a `remove` runs the group's remover (`box.md`, 1), and
  runs on; a termination signal during it
  stops the box as above, which closes them too.
- **Its stdout** carries the first link (below) and then `temper-box
  serve`'s stdout, the sign-in links it draws, but for its first line
  when a link step follows it, which is that step's (`playground.md`). The box reads every step's output a line at a
  time, and writes a line only once it holds it whole, to its line feed
  or its step's end, so that nothing written to its stderr lands inside
  a link when both streams are one terminal. A terminal-bound serve URL has
  at most 4096 payload bytes, excluding its line feed; an EOF fragment within
  that bound receives a line feed. An oversized URL is discarded through its
  line feed or EOF and named once on stderr, without printing any of its
  bytes; the next line can be read normally. This bound covers the configured
  public origin and sign-in path. The legacy first-line handoff to a link
  step keeps its existing bounded-piece semantics and receives no sign-in
  block. Other child streams retain bounded pieces, each completed with a
  line feed at the piece bound or EOF. With `--front`, the box writes
  nothing on its stdout or its stderr: their links go to its parent on
  fd 4, and their other lines to `box.log` (`box-front.md`, 1).
- **Its terminal,** its stderr, is quiet: the sign-in block and the
  blank line before each fresh link (below), and errors. Each error line
  begins with the time of day in UTC, `HH:MM:SSZ `, since the box reads no
  time zone:
  - for an asked group, named by its label (`box.md`, 3), `<label>: failed, <WHY>`
    as the pipe says it;
  - a long step's exit, `<WHY>; restarting in <n> s`, and the box's give
    up, `<WHY>; giving up`, `<WHY>` in `failed`'s words (`box.md`, 1);
  - and the box's own refusals and failures, as this section and `box.md`, 1
    name them: a record, a pipe's line, a log.

  A refusal of the start comes before any block, as any command's does,
  so no link is shown that cannot work (`box.md`, 1). Once the block is printed, a
  failed start's error follows it and the link stays on the terminal: the
  box restarts the step until it gives up, and a link opened once the box
  has given up gets no answer, and the terminal has said why. The person
  mends what it said and runs `temper-box` again, for a new link. With
  `--front`, these lines go to `box.log` and fd 4 (`box-front.md`, 1).

  `--echo steps` (`TEMPER_ECHO`; the default, `none`; skein's
  `runtime/shell-startup.md`, 1.5) also writes every line each step writes, but serve's
  stdout, prefixed `<step>: `, as it reaches the step's log. It adds no
  line of the box's own log.
- **Its own log,** `<data>/box/logs/box.log`, holds what a person need not
  read on their terminal, in the order it happens, each line beginning with
  the time of day as the terminal's lines do, and no link:
  `First run: wrote the configuration, <path>` when `temper-box init` ran
  (`box.md`, 1), and `First run: no sign-in through temper: <why>` when it made no
  provider's key (`init.md`, 5); `Logs: <data>/box/logs/`; `Ctrl-C stops temper.`; `Starting
  <step>` as each step of `always` starts; for an asked group, named by its
  label (`box.md`, 3), `<label>: starting` at its `start`, then `<label>: started`
  and `<label>: stopped` as the pipe says them, and `<label>: no sign-in
  through temper: <why>` when its `source` step did not add the source;
  with `--front`, the block's words and the terminal's lines in its
  stead (`box-front.md`, 1); `Stopping` at a first
  termination signal; and last, `Stopped` once every step is closed. An
  asked group's `step` lines are the client's row's alone. Each start of an
  owned forge, an asked group's `started` or the box's own start of one
  that starts with it, writes its hands (`../../client/ctl/testing.md`, 5) to its
  Forgejo step's log, `forgejo-N.log`, never the terminal: `Forgejo: api
  http://ADDRESS/api/v1` and `Forgejo: person LOGIN, read header PATH,
  write header PATH`, the files' paths and never their contents.
- **The sign-in block:** before any step starts, with its listeners held
  (`box.md`, 1) and the last run's logs moved (below), so that whoever reads the
  link reads this run's logs, the box writes the lines below, each once the
  one before it is written, so that a terminal shows them in order: the link
  alone on stdout, the rest on stderr. No step starts until they are
  written, and nothing precedes them on either stream. What the start says
  of its logs or records while they are moved waits behind them. The link's
  address is what serve's would be: the configured public origin, or else
  the clients' listener, and the link's page, `/sign-in`
  (`../../client/sign-in.md`, section 1); a link at the route, as a build
  before printed it, still signs in.

  ```
  Open this link in a browser on this machine.

  http://127.0.0.1:8367/sign-in?secret=...

  It signs you in as this temper's owner, once, within 2 hours.
  Opened after that, it prints a fresh one here.
  In a terminal: temper http://127.0.0.1:8367
  ```

  The last line names the TUI's command (`../../client/tui/README.md`, 1) at
  the link's own origin, its `http://HOST:PORT`, for a terminal that
  finds no engine's link file (`../../client/tui/directory.md`, 1): the client's
  `temper` command, then the origin.
  It is the first block's alone, on stderr with the lines around it; a
  fresh link is printed as below. With `--front`, the box prints no
  block: the link goes to fd 4 and the other lines to `box.log` (`box-front.md`, 1).

  The box draws the link's secret as jig's sign-in component draws one
  (jig's `protocol/client/sign-in.md`): 32 bytes from the platform's
  randomness, once, before the box starts anything. Serve's first start
  takes it (`box.md`, 1), and its expiry runs from then. A link opened before serve
  is ready waits on its connection, in the backlog of the socket the box
  holds (`box.md`, 1), the browser showing its own loading, and then signs in. That
  socket is the one the box holds today on both systems: it hands on what
  skein's `open_listener` or `temper-box init` made, and on macOS, where no
  socket says it is listening (skein's `runtime/shell-startup.md`, 1.6), serve's check of it
  passes as it does for any listener given.

  A fresh link, after a spent or expired one was opened (jig's
  `protocol/client/sign-in.md`) or a restarted serve's first, is printed
  alone, a blank line on stderr and then the link on stdout, neither with a
  time. A restarted serve draws its own first secret, and the box gives a
  secret to serve's first start alone.

  **On a terminal.** Where the box's stdout is a terminal and `TERM` isn't
  `dumb`, an unset `TERM` being no `dumb`, it wraps each link it prints, the
  first and every fresh one, in OSC 8, so that a terminal that can makes it
  clickable: `ESC ] 8 ; ; URL ESC \ URL ESC ] 8 ; ; ESC \`, then the line
  feed. The visible text stays the whole URL. A link so wrapped stays whole
  across a wrapped line, where a terminal's autodetection stops at the wrap
  and cuts the secret; a terminal that doesn't know the sequence, emacs vterm
  among them, ignores it. The box reads both once, at its start. Piped, sent
  to a file or under `TERM=dumb`, its stdout stays exactly as it is, the bare
  URL alone on its line, since scripts and tests read it. *Its stdout*'s 4096 bytes
  bound the URL there and here alike; the line a terminal takes holds it
  twice and the markers, at most 8,207 bytes with its line feed, which the
  box's output bound holds.

  **Where the box draws none.** It draws none, and gives none, where it
  cannot print a link that works: the document's printed link not enabled
  (jig's `protocol/configuration-engine.md`, section 1, clients), which prints no block at
  all; a clients' listener at port 0, not held (`box.md`, 1), so with no port to
  print, whose block is written at serve's first link, in these same words;
  and a link step among the steps, whose line is given to it (`playground.md`), with no
  block. A link past the 4096 bytes below refuses the start, naming it in
  the words below. With no first link drawn, `--front` refuses the
  start (`box-front.md`, 1).

  One bounded FIFO owns whole terminal items tagged stdout or stderr. It
  admits the first block as three adjacent items (words, URL, closing words),
  or a fresh block as two (the blank line, URL), before consuming that URL;
  what `--fake-backends` adds (`box-backends.md`, 1) is one item after the first block, or at
  the start with none. With N steps, at most max(4, 2N + 1) items are
  outstanding, including the active item. Only one item is in flight across both descriptors: after
  sending it, the host demands the target's entire io output budget, and
  its Room acknowledgement proves the write finished before the next item
  starts. A small write grant alone is not that acknowledgement.

  Full output pauses new child read demands; already admitted reads retain
  their bounded per-pipe chunk. Policy orders and their source wall times
  grow in paired internal FIFOs. Ordinary events keep their relative order
  while terminal acknowledgements may bypass a front blocked on request
  credit. A control command waits until earlier orders and control replies
  have dispatched; a record diagnostic enters the notice FIFO before its
  policy answer, and the record itself settles before started is announced.
  Captured ordinary events precede policy timers. An unsent spawn respects
  a queued withdrawal even when a terminal notice holds it back.
  Runnability and deadlines wait on the real destination's credit, so
  blocked output does not spin the loop. Its internal queues grow as they
  fill; what bounds the box is at its edges: terminal entries, pipe bytes
  and kernel operations, as above.

  The machine emits typed terminal facts, without words or wall time:
  Starting for an always step; a requested group's starting, started, failed
  and stopped; a step's restart or the box's give up; Stopping and Stopped.
  The host places each: a group's failed, a restart and a give up on the
  terminal, every other in its own log. A placement in the log waits on the
  log's room alone, never on the terminal's.
  Group facts identify the group, and failure facts retain the exact step
  identity and structured reason (exit code, signal, not started, or record
  not written). A restart also carries the chosen backoff duration. Labels
  and step names are looked up by identity, never copied into policy facts.
  Numeric pipe replies keep their existing WHY and settlement semantics.

  A requested group's starting is emitted once when an accepted start
  begins its work; repeated asks that only repeat a pipe reply emit no new
  lifecycle notice. Its failed and stopped notices follow actual group
  settlement, as their pipe replies do. A restart notice is emitted when
  the exit selects its backoff, before its timer fires. Giving up is emitted
  once when decided, before failure teardown, and is not repeated when the
  final outcome is collected. An exit during an intentional stop emits
  neither restart nor give up. Retry preparation failures name the actual
  once step that failed. A delay is written in seconds, preserving a
  fractional part when the configured millisecond delay is not whole seconds.

  A pure terminal formatter receives the host's supplied wall time, step
  names and group labels, and owns every lifecycle word. Starting names
  always steps; Stopping is emitted once on the first requested stop, and
  Stopped after actual settlement.
  It times every line but the block's and a fresh link's. A line's
  completion wall time is retained across output backpressure, rather than
  replaced with its later write time. Policy facts retain their decision's
  supplied wall time across order and terminal queues. Their admission and
  item-size bounds include labels and the longest failure/restart line.
- **Its logs,** `<data>/box/logs/`, mode 0700, made as needed: its own,
  `box.log` (above), and one file per step, its name with each space a `-`
  and `.log` after it (`serve.log`, `prepare-3.log`), mode 0600, holding
  every line the step writes to its stdout or its stderr, each after the
  time the box read it, `2026-10-23T14:03:20Z `, but serve's stdout: no
  file holds a sign-in link. At its start the box moves the logs a box before it
  left to `<data>/box/logs.previous/`, replacing that one, so the last
  two runs are kept. A start cut short after the move keeps
  `logs.previous/`; the box removes nothing else and follows no link.
  A file that would pass `box.log_bytes` (../configuration/limits.md
  ) moves to `<name>.log.1`, replacing that one, and starts
  afresh, so a step's logs stay within twice the bound.
  A record is never split: a file moves to `.log.1` before the record
  that would take it past the bound, so the bound holds at least one.
  A log the box cannot write is named once on its terminal, `box: cannot
  write <path>: <error>; the step's lines follow here`, and that step's
  lines then go to the terminal as `--echo steps` writes them; `box.log`'s
  lines then go there as their own words, with their times. Forgejo also
  logs to its own files.
- **A fetch's lines,** those of `owned.md`, 6 on the stdout of a step at the
  stage `fetch`:
  - **Each becomes a `fetched` line on the pipe** (`box.md`, 1), while the box
    is starting that step's group and the step is the one it runs. A
    line the step's pipe still holds once the step has exited stays in
    its log and goes no further, so none follows the next step's `step`
    line.
  - **The latest waits alone:** the box holds one `fetched` line,
    replaced by the next, and writes it once its replies are written,
    so a slow serve loses a figure, never a reply, and never slows the
    fetch. A reply drops the one held.
  - **The terminal shows none of them:** the step's log keeps them, as
    it keeps the step's other lines, and neither `--echo steps` nor a
    failure's tail shows them, so a failed fetch's tail is what it
    said went wrong.
  - **A line it cannot read,** or one from a step of no asked forge, is
    named on its stderr, `box: fetch: unknown line`, and goes no
    further.
- **A closed terminal's hangup** never counts as a second signal: a
  terminal's closing reaches the box from the kernel and again from the
  person's shell, so a `SIGHUP` while the box already stops does nothing,
  and the stop keeps its order. A `SIGHUP` already ignored when the box
  starts stays ignored (skein's `runtime/shell-startup.md`, 1.3), so `nohup temper box`
  outlives its terminal. The box exits 1 if a write finds that terminal
  gone, as for any terminal that fails.
- **A failure's tail:** under each terminal line of a step's failure,
  a long step's exit, a group's `failed` or a give up, the box writes
  the last `box.tail_lines` lines it logged for that step since its last
  start, held in memory, each as `--echo steps` writes it, indented four
  spaces, without its time, then `Log: <path>`, indented the same.
  With `--echo steps` it writes no tail, the lines already shown.
  A failure that is neither a step's exit nor its readiness, a step
  that logged nothing since its start, or a log not written has no tail.
- **Its environment:** each step gets the box's; `temper-box forge run`
  clears it for Forgejo (`owned.md`, 4).
- **Its children** are skein-io's (skein's `runtime/io/processes.md`), each in a
  process group of its own; closing one kills its group. A descendant
  that starts another group or session is outside that kill. Nothing the
  box starts may outlive its stop, nor, on Linux, its death (`box.md`, 1); each
  spawning owner must end descendants outside its group too. serve's
  agent processes and their programs are serve's children, not the box's
  steps: serve's first signal stops them, each agent ending its program,
  before serve exits (../service-process.md, section 1). A serve the box closes, or
  one a second signal aborts, kills its agents' groups, and a program an
  agent started outlives it until agents run in contained trees
  (`README.md`, section 9).

## 2. Open questions

- **The time of day on the terminal's error lines** (`box-supervision.md`, 1;
  `../../draft/box-link-first.md`, 6): the user may prefer none. Default:
  kept.
