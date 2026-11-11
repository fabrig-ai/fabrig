# The box

## 1. What it runs

`temper-box [DIRECTORIES] [--echo none|steps] [--front]` runs its steps, which
`temper-forge-setup`'s `kinds` and `temper-fake`'s fake slot give from
the document (`init.md`), each a command of the `temper-box`
binary, as a child process given the box's three directories as flags,
`--config C --data D --cache K`, each absolute (../configuration/directories.md
), written `DIRS` below, `N` a forge's number:

```
temper-box
│                                     the fake kind's group, the playground's fakes: asked (below)
├── temper-box fake llm DIRS              long (8.4)
├── temper-box forge prepare DIRS N       once: the fake forge's handover
├── temper-box fake forge DIRS N          long (8.4)
│                                     the owned kind's group, an owned Forgejo's: asked (below)
├── temper-box forge fetch DIRS N         once, when its binaries are fetched (6.6)
├── temper-box forge prepare DIRS N       once
├── temper-box forge run DIRS N forgejo   long: Forgejo's parent on Linux with fd 3 (6.4)
│   └── forgejo web                   via temper-box internal forge-child; execs without fd 3
├── forgejo-runner daemon             long: temper-box forge run DIRS N runner
├── temper-box forge source DIRS N        once, then again until the source serves, when machine.owned.sign_in is set (`../openid-provider.md`, 9)
└── temper-box internal serve DIRS --box  long: its own tree (../binaries.md); its setup
                                      host takes each handover and reconciles (section 5);
                                      its listeners from fd 3, then its pipe to the box (below)
```

- **With `--front`,** the box is the child of `temper box`'s TUI, its
  link and its lines on the descriptors its parent hands it, and nothing
  on its terminal (6). It runs no TUI of its own.
- **A step's start.** On Linux the box spawns each step through `temper-box
  internal forge-child --parent BOX --signal terminate -- temper-box ARGS`, BOX
  its own process id (`owned.md`, 4). That process execs the step, keeping its
  descriptors, its environment and its process group, so nothing else of
  this section sees the difference. A box that ends in any way, killed
  among them, leaves each step a `SIGTERM`, which it takes as on any stop.
  **On macOS** there is no parent-death signal (`owned.md`, 4): the box spawns each
  step directly, so a box killed outright leaves them running, and its
  next start refuses on the ports they hold.
- **An empty configuration directory,** absent or with no entry, is
  written first by `temper-box init --box` (`init.md`, 2), in the box's own process, before
  the document is read; the box says so in its own log, `First run:
  wrote the configuration, <path>` (2), and goes on to its steps.
  Otherwise the box
  never writes the document: a configuration directory with entries but
  no document is refused as every command refuses it.
- **The box knows kinds, not forges.** It has a group for each kind
  the document's machine part gives it (`init.md`): the owned kind's,
  an owned Forgejo's fetch, preparation, Forgejo and runner
  (`temper-forge-setup`'s steps), when the machine names the box's owned
  Forgejo; and the fake kind's, the playground's fake LLM, a fake
  forge's preparation and the fake forge (`temper-fake`'s), when it
  names the fakes and the document has `application.demo`. Each is
  dormant until serve asks for it for a forge, and then runs for that
  forge alone, its steps' forge number that forge's: a box runs at most
  one group of each kind. It reads no store and names no forge of its
  own: it stays wiring only.
- **A forge's group, asked.** Under the box, serve writes the group's
  settings, the forge's kind, its setup entry (`init.md`) and its port,
  to `<data>/box/groups/N`, private, then asks `start N` on the pipe:
  at an owned forge's set up (`owned.md`, 5) and at the playground's start (`playground.md`).
  The box builds the group's steps by the kind from that file, and runs
  them in order; `temper-box forge fetch`, `prepare`, `run` and `remove`
  read forge N's setup from the same file, and with none as 4 says.
  Once the group's long steps have started and, for an owned Forgejo,
  Forgejo answers, the box writes its record, `<data>/box/started/N`, one empty file, and
  answers `started N`. `box`, `box/groups` and `box/started` are made as
  needed, 0o700, their files 0o600. A record that cannot be written is
  named on the box's stderr, and the group answers `failed N record not
  written` and is dormant again.
- **At a later start,** the box lists `box/groups`, at most `box.groups`
  files (../configuration/limits.md), and runs each group whose
  `started` record exists from its file, in its place, as an unasked
  group, before `temper-box serve`, or just after it when the box signs its
  person in through that forge: a later preparation's source commands
  and Forgejo, loading its source, fetch discovery from temper
  (`../openid-provider.md`, 9), and serve takes the handover later for it (`owned.md`, 5). A
  group with a file and no record stays dormant until asked. A file it cannot read, or a record it cannot look
  at, is named on its stderr, and its group stays dormant; of two
  started groups of one kind, the one of the higher number runs, and the
  other is named and stays dormant.
- **A forge's group, removed.** At a forge's removal
  (`../../domain/forge/adoption.md`, 1) serve asks `remove N`, and again at each of
  its starts until it is answered `removed N`. The box stops the group's
  steps as it stops a group (2), then runs its remover, `temper-box forge
  remove DIRS N`, a once step, which removes the forge's directory,
  `<data>/forge-N` or `<data>/playground-N`, and refuses any path
  outside the data directory or within the cache, so the cache, and
  Forgejo's fetched binaries in it, stay (`owned.md`, 6). The box then removes
  `box/groups/N` and `box/started/N`, and answers `removed N`. For a
  forge with no group's file it knows, it removes those files alone. A
  removal that fails answers `failed N WHY`, `WHY` one of `remove:
  exited <code>`, `remove: did not start` and `remove: files not
  removed`, and the group is dormant again, to be asked again.
- **The listeners are the box's,** every step's, from the box's start
  to its end, a dormant group's among them: so that the port a step
  reports is its own from before it starts, and a second first run's
  init finds the first one's ports taken (`init.md`, 2). Each step names its
  listeners, the socket addresses the document gives it (3): `temper-box
  serve` the clients' and the webhooks' listeners, the fake LLM and the
  fake forge their ports, and, on Linux, `temper-box forge run N forgejo` the
  owned Forgejo's port, the machine's (`init.md`; `owned.md`, 4). On a
  first run the box keeps the sockets `temper-box init` chose (`init.md`, 2); on a later
  start it opens each at the document's address before any step, with
  skein's `open_listener`, and a port taken refuses its start, naming
  the port and the document, as any start is refused. A box started with
  listeners of its own (`LISTEN_FDS`, systemd's convention, as `temper-box
  internal serve`, `temper-box fake` and `temper-box forge run` read it: skein's
  `runtime/shell.md`, 6.6) takes each one given for a step's address, matched by
  its address, and opens only the rest, so a person's socket activation
  and a test's held port reach the box's steps as they reach serve; one
  given for no step's address refuses its start, naming it. It holds each
  one, never accepting, and gives it to its step at every spawn, a
  restart's among them, as the descriptors 3, 4, … in the step's order,
  with `LISTEN_FDS` set to their count in the step's environment and
  dropped from its own: systemd's convention, which skein's shell reads
  (skein's `runtime/shell.md`, 6.6) and Forgejo reads as well (`owned.md`, 4). The step's
  own `Listen` for that address then takes the socket it was given
  (skein's `runtime/io/sockets.md`, 2), so nothing above io knows it was given one. A
  step with no listener is given none, and no `LISTEN_FDS`. On Linux,
  `temper-box forge run N forgejo` is given the owned Forgejo's port and
  stays Forgejo's parent (`owned.md`, 4); on macOS it is given none, and a fresh
  box closes the reservation `temper-box init` made for its port, so
  Forgejo binds it itself. A listener at port 0 is not held: its step binds it, as without the box, since a
  port the kernel chooses can't collide. A held one could not be claimed
  either, since the step's `Listen` names port 0, not the port chosen.
  It changes at each restart, as it does without the box. `LISTEN_FDS`
  counts only the sockets given.
- **The pipe,** two of them, made again at each spawn of `temper-box
  internal serve`, at the two descriptors after its listeners: 3 + *n*,
  serve to box, and 4 + *n*, box to serve, *n* its listeners' count, so
  fds 5 and 6 under the default document's two. `--box` tells `temper-box
  internal serve` to adopt them, and a serve given `--box` whose
  descriptor 3 + *n* or 4 + *n* is not an open pipe refuses its start,
  naming the fd, *n* being the `LISTEN_FDS` it was given, 0 without it
  (a check that reads `/proc/self/fd`, so Linux's alone: on macOS serve
  takes them as given);
  without `--box`, as under every other arrangement (4), serve has no
  pipe and leaves those descriptors alone. One line each, ending in a line
  feed, `FORGE` the forge's number:

  | Line | From | Says |
  |---|---|---|
  | `start FORGE` | serve, once it has written the group's file: at an owned forge's set up (`owned.md`, 5) and at the playground's start (`playground.md`) | start the forge's group, by its file |
  | `step FORGE STEP` | the box | the group's step `fetch`, `prepare`, `run` or `source` is starting, for the client's row (`../../client/forge/deployment.md`) |
  | `fetched FORGE BINARY DONE TOTAL` | the box | how far the group's `fetch` step has gone, one for each line it says (`owned.md`, 6): `BINARY` `forgejo` or `runner`, `DONE` its bytes written, `TOTAL` its size or `none`. Between `step FORGE fetch` and the next step alone (2), and at most 64 bytes, `box.line_bytes`'s least, whatever its numbers |
  | `source FORGE added`, `source FORGE failed WHY` | the box | how the group's `source` step ended (`../openid-provider.md`, 9), before its `started`: `WHY` as `failed`'s, the step's own words or `source exited N`, `source did not start`, within the line's bound, holding no secret. A `start` for a group already standing says its last such line again, before its `started`. A retried step's ending is said again, after the `started`, only when it differs from the last one said, so `source FORGE added` follows a `failed` once the source serves |
  | `started FORGE` | the box | the group's long steps are started, an owned Forgejo answers, and its record is written; the engine then takes the handover and reconciles (`owned.md`, 5) |
  | `failed FORGE WHY` | the box | the group stopped: `WHY` names the step by its name, as its log file and the box's terminal name it (2), such as `prepare-3 exited 1`: `<step> exited <code>`, `<step> killed by signal <n>`, `<step> did not start`, `record not written`, or a removal's `remove: exited <code>`, `remove: did not start` or `remove: files not removed`, cut to fit the line's bound, and holds no secret; a step's own lines are in its log, the last of them under the terminal's `failed` line (2) |
  | `stop FORGE` | serve | stop the forge's group, its file and record kept |
  | `stopped FORGE` | the box | the group's steps are closed, as the box's own stop closes them (2), and the group is dormant again; its file and record stay |
  | `remove FORGE` | serve, at the forge's removal (`../../domain/forge/adoption.md`, 1), and at each of its starts until answered | stop the forge's group and remove its directory, its file and its record (above) |
  | `removed FORGE` | the box | the group's steps are closed, and its directory, its file and its record removed; the cache is kept |

  One ask at a time: a `start` for another forge while a group starts,
  or while a failed group's steps close, is answered `failed FORGE busy`;
  one for the forge whose group is starting, or whose failed steps
  close, by that group's own lines, its `failed` once they have closed;
  one for a forge with no group, `failed FORGE not asked`; one for a
  started group, `started FORGE`; one while the box stops,
  `failed FORGE stopping`. A `stop` is an ask as a `start` is, one at a
  time: for a started group, the box closes its steps, then answers
  `stopped FORGE`; for a dormant one, `stopped FORGE` at once; for a
  group starting, or whose failed steps close, `failed FORGE busy`; for
  a forge with no group, `failed FORGE not asked`; and while the box
  stops, `failed FORGE stopping`. A `remove` is an ask as a `stop` is,
  answered `removed FORGE` where a `stop` is answered `stopped FORGE`,
  once its remover has run and its files are removed; for a forge with
  no group's file the box knows, it removes the files and answers at
  once. While a group's steps close at its `stop` or `remove`, or its
  remover runs, a `start`, a `stop` or a `remove` for any forge is
  answered `failed FORGE busy`. An
  answer to a `temper-box serve` that has exited is dropped: its restart
  finds the handover at its start (`owned.md`, 5). Each line is at most
  `box.line_bytes` (../configuration/limits.md), which both sides
  read within; another line, or a longer one, is named on its reader's
  stderr and ignored. serve's side, its asks, its reads and the states
  they tell the engine's root, is the routing's box link (../forge/engine.md,
  8).
- **The first link's secret** (2) reaches `temper-box internal serve`'s
  first start on its standard input, a pipe of its own: the box writes
  the secret's 32 bytes, nothing around them, and closes it; with
  `--front`, the printed link's 32 bytes and then the first local
  link's 32, whose link the box hands its parent (6). `--box` tells
  serve to read its standard input to its end before it starts anything,
  and it gives what it read to jig's sign-in component, the first 32 bytes
  as the printed link's `link_secret` and the next 32 as the local link's
  `local_secret` (jig's `protocol/client/sign-in.md`, *A secret given*). No argument and no
  environment entry carries it, and no log holds it. Every later start
  has no such pipe, its standard input is the null device, and its
  component draws its own first secret, so a restart never replays a
  spent one. The box gives the secret at serve's first spawn whatever
  becomes of that start. Serve refuses any length but none, 32 or 64 bytes,
  `--box: stdin: expected the first link's secret`. Linux and macOS read it
  alike, as the pipe's own descriptors are made alike: skein's `Spawn` makes
  a pipe at a child's descriptor 0 as at any other, and fills what no pipe
  takes from `/dev/null`.
- **A group's failure** before `started` answers `failed`, and the group
  is dormant again, to be asked again; the box runs on. Once started,
  its long steps are restarted, and given up on, as 2 says.
- **A first run's steps** (`init.md`, 3): `temper-box serve` alone runs at the
  start; the fake kind's group is dormant until the person tries the
  playground (`playground.md`), and the owned kind's until the person sets up an
  owned Forgejo (`owned.md`, 5). No step is a link: serve's links are the
  person's.

## 2. What it does

- **In order:** each step starts once the one before it has started, when
  that one is long, or has exited 0, when it is once or link; the `source` step
  alone (1) ends the group's start whichever way it ends, never failing it.
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
  (1), or a long one exits more than `box.restarts` times within
  `box.window_ms`. The backoff is counted from a step's exits within the
  window, so it starts again from `box.backoff_ms` once a step has run a
  window without exiting. Its stop treats a step still starting as the box's
  stop does. With `--front`, it then exits 1 (6).
- **Stops** at a first termination signal, `SIGINT`, `SIGTERM` or a closed
  terminal's `SIGHUP`, or with `--front` its standard input's end (6): the long steps in reverse
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
  among them (6).
- **Stops a group** at `stop FORGE` or `remove FORGE` (1): the group's running steps
  alone, as the box's stop does, in reverse order, each by a signal,
  `box.grace_ms`, a second signal, `box.grace_ms` again, then a close;
  a step that exits during it is not restarted, and a once step still
  running is closed with them; a step still starting is treated as at the
  box's stop. Once they are all closed the group is dormant, the box
  writes `<label>: stopped` to its own log (below), and answers
  `stopped FORGE`, or at a `remove` runs the group's remover (1), and
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
  fd 4, and their other lines to `box.log` (6).
- **Its terminal,** its stderr, is quiet: the sign-in block and the
  blank line before each fresh link (below), and errors. Each error line
  begins with the time of day in UTC, `HH:MM:SSZ `, since the box reads no
  time zone:
  - for an asked group, named by its label (3), `<label>: failed, <WHY>`
    as the pipe says it;
  - a long step's exit, `<WHY>; restarting in <n> s`, and the box's give
    up, `<WHY>; giving up`, `<WHY>` in `failed`'s words (1);
  - and the box's own refusals and failures, as this section and 1
    name them: a record, a pipe's line, a log.

  A refusal of the start comes before any block, as any command's does,
  so no link is shown that cannot work (1). Once the block is printed, a
  failed start's error follows it and the link stays on the terminal: the
  box restarts the step until it gives up, and a link opened once the box
  has given up gets no answer, and the terminal has said why. The person
  mends what it said and runs `temper-box` again, for a new link. With
  `--front`, these lines go to `box.log` and fd 4 (6).

  `--echo steps` (`TEMPER_ECHO`; the default, `none`; skein's
  `runtime/shell.md`, 6.5) also writes every line each step writes, but serve's
  stdout, prefixed `<step>: `, as it reaches the step's log. It adds no
  line of the box's own log.
- **Its own log,** `<data>/box/logs/box.log`, holds what a person need not
  read on their terminal, in the order it happens, each line beginning with
  the time of day as the terminal's lines do, and no link:
  `First run: wrote the configuration, <path>` when `temper-box init` ran
  (1), and `First run: no sign-in through temper: <why>` when it made no
  provider's key (`init.md`, 2); `Logs: <data>/box/logs/`; `Ctrl-C stops temper.`; `Starting
  <step>` as each step of `always` starts; for an asked group, named by its
  label (3), `<label>: starting` at its `start`, then `<label>: started`
  and `<label>: stopped` as the pipe says them, and `<label>: no sign-in
  through temper: <why>` when its `source` step did not add the source;
  with `--front`, the block's words and the terminal's lines in its
  stead (6); `Stopping` at a first
  termination signal; and last, `Stopped` once every step is closed. An
  asked group's `step` lines are the client's row's alone. Each start of an
  owned forge, an asked group's `started` or the box's own start of one
  that starts with it, writes its hands (`../../client/ctl/testing.md`, 5) to its
  Forgejo step's log, `forgejo-N.log`, never the terminal: `Forgejo: api
  http://ADDRESS/api/v1` and `Forgejo: person LOGIN, read header PATH,
  write header PATH`, the files' paths and never their contents.
- **The sign-in block:** before any step starts, with its listeners held
  (1) and the last run's logs moved (below), so that whoever reads the
  link reads this run's logs, the box writes the lines below, each once the
  one before it is written, so that a terminal shows them in order: the link
  alone on stdout, the rest on stderr. No step starts until they are
  written, and nothing precedes them on either stream. What the start says
  of its logs or records while they are moved waits behind them. The link's
  address is what serve's would be: the configured public origin, or else
  the clients' listener, and the link route.

  ```
  Open this link in a browser on this machine.

  http://127.0.0.1:8367/jig/v1/sign-in/link?secret=...

  It signs you in as this temper's owner, once, within 2 hours.
  Opened after that, it prints a fresh one here.
  In a terminal: temper http://127.0.0.1:8367
  ```

  The last line names the TUI's command (`../../client/tui/README.md`, 1) at
  the link's own origin, its `http://HOST:PORT`, for a terminal that
  finds no engine's link file (`../../client/tui/README.md`, 2): the client's
  `temper` command, then the origin.
  It is the first block's alone, on stderr with the lines around it; a
  fresh link is printed as below. With `--front`, the box prints no
  block: the link goes to fd 4 and the other lines to `box.log` (6).

  The box draws the link's secret as jig's sign-in component draws one
  (jig's `protocol/client/sign-in.md`): 32 bytes from the platform's
  randomness, once, before the box starts anything. Serve's first start
  takes it (1), and its expiry runs from then. A link opened before serve
  is ready waits on its connection, in the backlog of the socket the box
  holds (1), the browser showing its own loading, and then signs in. That
  socket is the one the box holds today on both systems: it hands on what
  skein's `open_listener` or `temper-box init` made, and on macOS, where no
  socket says it is listening (skein's `runtime/shell.md`, 6.6), serve's check of it
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
  (jig's `protocol/configuration.md`, section 3, clients), which prints no block at
  all; a clients' listener at port 0, not held (1), so with no port to
  print, whose block is written at serve's first link, in these same words;
  and a link step among the steps, whose line is given to it (`playground.md`), with no
  block. A link past the 4096 bytes below refuses the start, naming it in
  the words below. With no first link drawn, `--front` refuses the
  start (6).

  One bounded FIFO owns whole terminal items tagged stdout or stderr. It
  admits the first block as three adjacent items (words, URL, closing words),
  or a fresh block as two (the blank line, URL), before consuming that URL;
  what `--fake-backends` adds (5) is one item after the first block, or at
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
  - **Each becomes a `fetched` line on the pipe** (1), while the box
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
  starts stays ignored (skein's `runtime/shell.md`, 6.3), so `nohup temper box`
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
  box starts may outlive its stop, nor, on Linux, its death (1); each
  spawning owner must end descendants outside its group too. serve's
  agent processes and their programs are serve's children, not the box's
  steps: serve's first signal stops them, each agent ending its program,
  before serve exits (../README.md, section 6). A serve the box closes, or
  one a second signal aborts, kills its agents' groups, and a program an
  agent started outlives it until agents run in contained trees
  (`README.md`, section 9).

## 3. What it names

- **Steps only:** each step's name, its arguments and whether it is
  once, long or link; and which of them are a kind's dormant group,
  bound to a forge by its number alone at its ask. Its policy is a small
  machine of its own (steps, exits, timers and the pipe's lines in;
  spawns, signals, closes, the pipe's lines and the groups' files and
  records out), hosted on skein-io. With `--front` (6) the host writes
  the link and the lines on its parent's pipes beside the machine, and
  reads its standard input's end.
- **The steps in groups:** `Steps { always, asked }`, `always` the steps
  run in order, and `asked` a list of `Group { kind, forge, before,
  steps, remover, recorded, label }`, each a kind's group, dormant until
  asked for a forge (1). Its steps name the forge by a placeholder,
  `FORGE_ARGUMENT`, which the box replaces with the forge's number when
  it binds the group to it, its steps' names taking the number too;
  `remover` is the once step its removal runs (1); and its `started`
  writes the box's record. `label: Box<[u8]>` is the words the terminal
  names it by (2), owned opaque bytes passed from the setup/fake
  constructor into the box's layout and written verbatim but for its
  `FORGE_ARGUMENT`, as a step's name is: `Playground` for the fake kind's,
  `Forge N (Forgejo)` for the owned kind's, N the forge's number. The box never
  infers a label from arguments; it adds no label field to a wire reply
  or stored record. `before` is its place in the order:
  the index in `always` it runs just before (its length puts it last).
  Each step carries its `Stage`, whose word its `step` lines carry, and
  its listeners, the socket addresses the box holds and gives it (1). The
  machine's step tests drive a group from dormant to `started`, and to
  `failed`, a started one to `stopped`, and one to `removed`, by lines
  and exits alone (`testing.md`, 1).
- **The binary hands it the steps.** `temper-box` resolves its three
  directories, runs `temper-box init --box` when the configuration directory is
  empty (1), checks the document as `init.md` says, asks
  `temper-forge-setup` for its kinds' steps, `temper-box serve` and the
  owned kind's group, and `temper-fake` for the fake kind's group (`playground.md`),
  and gives them to `temper-box-machine`, each asked group apart.
- **Welcome peers first:** the fake-backend configuration supplies an
  always-running prefix before the ordinary steps. Every asked group's
  insertion point, the playground's included, follows that prefix.
  An empty prefix leaves the ordinary order unchanged. Stopping or
  removing an asked group never stops these peers.
- **Checked:** a boundary test fails the gate when `temper-box-machine` depends
  on a crate named `temper-*` but `temper-forge-setup`, or names an item
  of it but `steps`, `Steps`, `Group`, `Stage`, `Step` and its `Kind`,
  `FORGE_ARGUMENT`, and of its kinds and group's files `BoxKind`,
  `group_path`, `group_file` and `parse_group_file`.

## 4. Other arrangements of the same commands

None has the box's pipe: `temper-box serve` runs without `--box`, and runs
no owned forge. An owned forge's group is the box's: `SetUp` of an
owned forge is refused *not under the box* (`owned.md`, 5). GitHub and a Forgejo
elsewhere connect as under the box.

`temper-box forge fetch`, `prepare` and `run` are commands of their own
too, and a forge the box did not run has no group's file (1). A forge
with none takes the machine's `owned` part, in `<data>/forge-N`, so that
a host's own units run them with no file written by hand, and a data
directory started afresh keeps them working. A document with no `owned`
part refuses, naming the file and the part; a file there is read as
ever, its kind and its refusals its own. `temper-box forge remove` reads the
file alone, and refuses a forge with none: it removes a directory by
its kind, which it does not guess.

The welcome follows the document under every arrangement, its member
`application.welcome` (../configuration/engine.md), and never `--box`,
which only the box gives `temper-box internal serve`, for its pipe. A unit
running `temper box` shows the welcome as any box does; one running
`temper-box serve` on a hand-written document shows it only if that document
asks.

- **systemd,** a unit for `temper box`, whose own stop stops its tree
  (2), or one for `temper-box serve` alone; `temper-box serve`'s
  unit keeps systemd's default kill of its control group, which ends a
  crashed service's whole tree before its restart (skein's `runtime/shell.md`,
  12);
- **A host's own units,** `temper help server`'s walk: a unit running
  `temper-box forge fetch`, `prepare` and `run`, beside one for `temper-box
  serve`; a deployment's engine host runs the same units on
  `temper-engine`, its document an engine alone's (../binaries.md, 2). No owned forge's record is made there, so nothing takes the
  handover (`owned.md`, 5), and a start on a new store lists no forge. A person
  connects that Forgejo as its admin (`reconcile.md`, 5), the handover file
  `admin_token` the token, the connect's `--api` and `--git` the
  machine's `listen` and `remote`, and `--web` its `public`
  (`../openid-provider.md`, 7). It is a managed forge from then on: `TakeOver`
  never runs, so the file stays in the forge's directory for the same
  connect on a store started afresh, where the forge is numbered 1
  again;
- **the process tier** (`testing.md`, section 4), which runs the commands itself,
  with no group's file, as a host's own units do;
- **a container image** whose entry point is `temper box`, every process
  of its tree ending with the container.

## 5. The welcome's fake backends

temper-box [DIRECTORIES] --fake-backends [--bundle BUNDLE] prepares the retained isolated
sandbox of ../configuration/directories.md, 1. Ordinary children read the effective
temper.json and agent.json through their effective directory flags; peer
commands read absolute effective document paths. Changing only the parent's
parsed bytes is insufficient.

Build the loopback bundle once, then start even with empty configuration:

    scripts/bundle.sh --loopback DIR && temper-box --fake-backends --bundle DIR

The flag overrides an init-shaped source's application.clients.bundle;
without it, that configured bundle is used. Missing both refuses with the
bundle command and --bundle. Empty C stays untouched: init writes directly
into the effective sandbox. Box startup performs no build or fetch.

Without the test override below, a sandbox written by init takes the
kernel's ports, a bind of port 0 for each of its five, never the 8367 to
8371 a person's box prefers (`init.md`, 2). So a
fake box leaves those to the box a person starts after it. A kernel-chosen
port lost between bind and listen is closed and chosen again, within 32
attempts; the fake peers' reservations likewise retry address-in-use,
counting excluded ports against the same bound. Other errors refuse at
once. Only a listening socket's port is reported. The ports are written
fixed and held as init's are, and
the printed link names them. A stopped box holds none, so a port the kernel
hands on meanwhile refuses its restart, naming the port. An init-shaped
source's listeners stay its own.

`TEMPER_INTERNAL_FAKE_BOX_PORTS`, the process tier's and never a person's
(`../../testing/process-tier.md`), names a new sandbox's nine ports, separated
by commas: init's five, clients, owned, forge, llm and webhooks, then the
peers' github, oauth, llm and forgejo. A new sandbox holds each of them or
refuses to start, naming the port and the variable, as it does a value that
isn't nine nonzero ports. A retained sandbox keeps the ports it has,
whatever the variable says.

After the sign-in block (2), before any supervised peer starts, the box
prints the effective directories, the fakes' origins and the files a walk
pastes (../configuration/directories.md, 1).
GitHub has one fixed person and repository. Its App 1 is seeded and
installed, with the setup URL a person gives an App of their own (`github.md`, 3),
`http://<listener>/forges/1/github/setup`, the number GitHub's forge
takes when it is the first forge a walk makes, so that the own-App path's
Install comes back to temper; App 1's private key is D'/github-app.pem,
mode 0600. The dummy model key is the key a walk pastes. The elsewhere
Forgejo's admin token is kept in its private file: instructions name only
the path, and no log, report or record contains its contents. The fixed signed-in model bearer uses the first added
credential stream with the same seed as the issuer, while the primary
model key stays the API-key choice. Neither value is a real credential.

The standalone issuer command is:

    temper-box fake oauth --document FILE [--ready PREFIX] [--seed N]

Its bounded document has a loopback listen socket with a nonzero port,
redirect_port (the shipped 1455 by default) and redirect_path
(auth/callback by default). The listen socket determines its HTTP origin
and shipped authorization, token, revocation and keys paths. Unknown,
duplicate, ill-typed and oversized members are refused before any output
or listener: at most 16 KiB for the document and 1024 bytes per URL/path.

temper-fake decodes the document and configures skein's OAuth peer;
temper owns the files, signal handling, readiness and the real loop.
Authorization grants immediately with the shipped client/scopes/resource,
the fixed fake person, OpenID tokens/keys and the fixed bearer accepted
by the model. No login or consent page is added.

The wrapper refills its bounded token-plan queue each pass. Each issuer
launch admits at most 64 sign-ins, since the primitive retains used codes.
Exhaustion, or refresh after revoking the fixed refresh token, requires an
issuer restart. Generated configuration keeps callback port 1455; concurrent
human sign-ins from several local boxes are not promised. Tests name
explicit callback ports. Long-running peers retain no observation stream
unless one is explicitly requested.

The GitHub and model peers are long steps named `fake github` and
`fake model`, before ordinary steps and outside the playground's asked
group. Their arguments are `fake forge --document C'/github.json --seed 7`
and `fake llm --document C'/llm.json --seed 7`, respectively, with no
directory flags. The issuer uses the same seed, so its bearer matches the
model's first added key. A supervised issuer is `fake issuer`, using
`fake oauth --document C'/oauth.json --seed 7`; it requires the primitive's
inherited-listener support. It never substitutes a self-bound startup race
for that contract.

The box owns the peers' held listeners before spawning them. First
preparation transfers its reservations exactly once; on reuse the ordinary
box listener mechanism binds the retained documents' addresses. Each is
loopback with a nonzero port, distinct from the other peer ports and the
ordinary box listeners. Output paths stay within D'. The retained documents
are revalidated on both paths; edits are not silently replaced.

Held listeners provide the ordinary fake peers' readiness and accept
backlog. Restart, backoff, give-up and reverse-order shutdown use the
ordinary supervisor. The playground's reset ends no peer of these, so a
person's newly connected model or issuer runs on. Peer memory lasts one launch: a restarted peer
uses the same seed and output files, but does not recover in-memory Apps or
other changes. Effective configuration and temper's data remain retained.

**Sources to copy** (`../../draft/repository-copies.md`, 6): the `fake forgejo`
peer's document names two, and `box: copy source ADDRESS` prints each
(../configuration/directories.md, 1). They serve a Forgejo elsewhere, the peer the
walk connects as its admin: an owned Forgejo is real, and reaches no
`.invalid` host.

Forgejo here remains owned and real. The machine's part points GitHub
at its fake (`init.md`), and names no forge: each is made by the walk's
own `SetUp` or `Connect`. Elsewhere is a Forgejo the walk connects as its
admin (`reconcile.md`, 5), at the address preparation prints.
Its separate `fake forgejo` long step uses
`fake forge --document C'/forgejo.json --seed 7`, with its held loopback
listener before ordinary steps and outside the playground's asked group.
The fixed peer prefix order is fake github, fake issuer, fake model,
then fake forgejo. Its retained token output is D'/forgejo-admin.token,
subject to the same private-file validation and seed-output reuse as the
other peers. Admission and reconciliation use the
ordinary configuration checks, without raised budgets or policy limits.
The program way uses a separately provisioned fake adapter
(../configuration/directories.md, 1) and the ordinary program-login mechanism. Only
after validating the adapter does the box print the exact program-login
command for its effective directories; a walker starts the box before using
it. No product path branches on the flag. A conflicting issuer listener is a
refusal, not a reason to bind another port or regenerate the retained
document.

## 6. In front of `temper box`

In a terminal, `temper box` runs `temper`'s TUI with the box as its
child, `temper-box --front`: the TUI is on the person's terminal from
the first moment, signs itself in with nothing to read or paste, and
quitting it stops the box. The TUI's side, finding the box and running
it, is `../../client/tui/box.md` and `../../client/box-binary.md`; this
section is the box's side, what `--front` changes of 1 to 5. The box
spawns no TUI, and a `temper-box` without `--front` runs with no front,
in a terminal or not, as 1 to 5 say.

- **Its forms,** internal, for `temper box` alone (`../../cli/README.md`,
  6): `temper-box [DIRECTORIES] [--echo none] --front` and `temper-box
  [DIRECTORIES] --fake-backends [--bundle BUNDLE] --front`. `--front`
  with `--echo steps` refuses its arguments, exit 2: its lines are fd
  4's, and `temper box` runs a box that echoes in its own place
  (`../../client/box-binary.md`, 5).
- **Its descriptors,** which its parent hands it:

  | fd | What | Way |
  |---|---|---|
  | 0 | a pipe its parent holds and never writes: its end is the parent's quit or death | in |
  | 1, 2 | one pipe its parent reads, for what the box writes before `box.log` is open, a fail-stop report among it | out |
  | 3 | the local link's URL, written whole by the box, then closed | out |
  | 4 | the box's lines (below), until the box exits | out |

  Fd 3 or fd 4 not open for writing at its start refuses, exit 2, before
  anything is written: `temper-box --front: fd 3 and fd 4 are its
  parent's pipes`. Its steps are given none of them.
- **The terminal is never the box's:** it opens no controlling terminal,
  takes no foreground and sets no mode, and writes nothing on its stdout
  or its stderr once `box.log` is open. Every line 2 places on the
  terminal goes to `box.log` in its stead, timed as `box.log`'s lines
  are: the first block's lines but its link, its blank ones left out;
  `--fake-backends`' lines after the block (5); a group's failure, a
  restart and a give up, each with its tail and its `Log:` line; a step's
  lines when its log cannot be written; and the box's own refusals and
  failures. A refusal before `box.log` is open goes on its stderr, which
  its parent shows. A `box.log` the box cannot write leaves its lines to
  fd 4's `log` lines alone.
- **The sign-in it hands.** Beside the printed link's secret the box
  draws the first local link's, 32 bytes as jig's sign-in component draws
  one (jig's `protocol/client/sign-in.md`), once, before it starts
  anything, and gives both to serve's first start (1, *The first link's
  secret*). The local link's URL, made at the clients' listener with no
  public origin, so that its authority is the engine's loopback address
  the TUI connects to, on the link route, at most 4,096 bytes, is written
  whole on fd 3 and fd 3 closed, in the first block's place: once the
  listeners are held and the last run's logs moved, before any step. It
  goes nowhere else: never an argument, the environment, a log or a
  file. The TUI signs in by it, its connection waiting in the listener's
  backlog until serve is up, as a browser's does (2), and the printed
  link stays the browser's. Where 2, *Where the box draws none*, holds,
  `--front` refuses the start, exit 1, naming why, before any step.
- **The box's lines,** on fd 4, one each: a word, one space, a field
  that runs to the line feed (bytes, spaces allowed, no escapes, never
  empty), then the line feed. A line is at most 8,192 bytes with its
  line feed; the box cuts a `log` field to fit at a UTF-8 boundary.

  | Line | Field | When |
  |---|---|---|
  | `link URL` | the printed link's URL, bare, never in OSC 8 | first, the first block's link; then each fresh link serve prints (2) |
  | `log TEXT` | one line as `box.log` holds it, its time first, without its line feed | each line `box.log` gets, in its order, those it gets in the terminal's stead among them |
  | `restarting STEP` | the step's name, as the terminal's lines name it (`forgejo-3`) | a long step exited and is to be restarted (2, *Restarts*) |
  | `running STEP` | the same | that step spawned again |
  | `gave-up STEP` | the same | the box gives up on that step (2, *Gives up*) |

  No line but `link` holds a secret, and the box writes no `link` to a
  log. The box holds at most 256 lines its parent has not taken; a line
  that finds 256 held is dropped, `box.log` and the steps' logs keeping
  what it said. Writing them never holds a step, a log or the pipe to
  serve. A write that fails, its parent gone, drops the line.
- **Its stop:** the end of fd 0, a read that answers its end or fails,
  is a first termination signal (2, *Stops*): `temper`'s quit, which
  closes its end, and `temper`'s death, whose end the kernel closes, both
  stop the box, on Linux and on macOS alike, with no parent-death signal.
  `box.log` says `Stopping` as at any first termination signal. A
  termination signal is as 2 says, and one after fd 0's end is a second,
  which closes the steps.
- **A give up:** the box stops its steps as 2 says, writes `gave-up
  STEP` and the give up's `log` lines on fd 4, and exits 1. Its parent
  keeps what it said (`../../client/tui/box.md`).
- **Its exit:** 0 after a stop, 1 after a give up or a failure (2), 2
  when it refused its arguments or its descriptors.
- **Where it lives.** The binary (`temper-box-shell`) checks `--front`'s
  descriptors, draws the local link's secret and URL, and gives
  temper-box-machine's host the URL, its secret and the descriptors in
  its settings. The host writes fd 3 in the first block's place and the
  box's lines on fd 4 with `temper_box_machine::front`, which owns their
  words and bytes, reads fd 0 beside its signals, and routes the
  terminal's lines to `box.log`. No front child, no front's log and no
  foreground taken back: those went with the box's own TUI.
