# The box's front

This document holds the box's front protocol, split from [`box.md`](box.md).

## 1. In front of `temper box`

In a terminal, `temper box` runs `temper`'s TUI with the box as its
child, `temper-box --front`: the TUI is on the person's terminal from
the first moment, signs itself in with nothing to read or paste, and
quitting it stops the box. The TUI's side, finding the box and running
it, is `../../client/tui/box.md` and `../../client/box-binary.md`; this
section is the box's side, what `--front` changes of `box.md`, 1, `box-supervision.md`, 1, `box.md`, 3 to 4, and `box-backends.md`, 1. The box
spawns no TUI, and a `temper-box` without `--front` runs with no front,
in a terminal or not, as `box.md`, 1, `box-supervision.md`, 1, `box.md`, 3 to 4, and `box-backends.md`, 1 say.

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
  | 1, 2 | pipes its parent reads, for what the box writes before `box.log` is open, a fail-stop report among it; one pipe or two, the box writes to each as to its stdout and stderr | out |
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
  `--fake-backends`' lines after the block (`box-backends.md`, 1); a group's failure, a
  restart and a give up, each with its tail and its `Log:` line; a step's
  lines when its log cannot be written; and the box's own refusals and
  failures. A refusal before `box.log` is open goes on its stderr, which
  its parent shows. A `box.log` the box cannot write leaves its lines to
  fd 4's `log` lines alone.
- **The sign-in it hands.** Beside the printed link's secret the box
  draws the first local link's, 32 bytes as jig's sign-in component draws
  one (jig's `protocol/client/sign-in.md`), once, before it starts
  anything, and gives both to serve's first start (`box.md`, 1, *The first link's
  secret*). The local link's URL, made at the clients' listener with no
  public origin, so that its authority is the engine's loopback address
  the TUI connects to, on the link route, at most 4,096 bytes, is written
  whole on fd 3 and fd 3 closed, in the first block's place: once the
  listeners are held and the last run's logs moved, before any step. It
  goes nowhere else: never an argument, the environment, a log or a
  file. The TUI signs in by it, its connection waiting in the listener's
  backlog until serve is up, as a browser's does (`box-supervision.md`, 1), and the printed
  link stays the browser's. Where 2, *Where the box draws none*, holds,
  `--front` refuses the start, exit 1, naming why, before any step.
- **The box's lines,** on fd 4, one each: a word, one space, a field
  that runs to the line feed (bytes, spaces allowed, no escapes, never
  empty), then the line feed. A line is at most 8,192 bytes with its
  line feed; the box cuts a `log` field to fit at a UTF-8 boundary.

  | Line | Field | When |
  |---|---|---|
  | `link URL` | the printed link's URL, bare, never in OSC 8 | first, the first block's link; then each fresh link serve prints (`box-supervision.md`, 1) |
  | `log TEXT` | one line as `box.log` holds it, its time first, without its line feed | each line `box.log` gets, in its order, those it gets in the terminal's stead among them |
  | `restarting STEP` | the step's name, as the terminal's lines name it (`forgejo-3`) | a long step exited and is to be restarted (`box-supervision.md`, 1, *Restarts*) |
  | `running STEP` | the same | that step spawned again |
  | `gave-up STEP` | the same | the box gives up on that step (`box-supervision.md`, 1, *Gives up*) |
  | `starting NAME` | the group's kind's name, `Forgejo` for the owned kind's and `Playground` for the fake kind's, never its label (`box.md`, 3) | a group's start begins, where `box.log` gets `<label>: starting` (`box-supervision.md`, 1, *Its own log*): an asked group's at its `start`, and a started group the box runs at a later start at its first step (`box.md`, 1, *At a later start*) |
  | `started NAME` | the same | that group is started: its long steps started and, for an owned Forgejo, Forgejo answers, where `box.log` gets `<label>: started` |
  | `dormant NAME` | the same | that group runs nothing again: its start failed, answering `failed`, or its steps closed at a stop or a removal, where `box.log` gets `<label>: stopped` (`box-supervision.md`, 1, *Stops a group*) |

  A box runs at most one group of a kind (`box.md`, 1), so NAME names one group;
  the names are `temper_box_machine::front`'s words, the TUI's state
  and notice in front saying them as they come (`../../client/tui/box.md`,
  *The box's state*): the user's request `tui-11` of 2026-11-15, so
  that a box set up before says its Forgejo starting. No line but
  `link` holds a secret, and the box writes no `link` to a
  log. The box holds at most 256 lines its parent has not taken; a line
  that finds 256 held is dropped, `box.log` and the steps' logs keeping
  what it said. Writing them never holds a step, a log or the pipe to
  serve. A write that fails, its parent gone, drops the line.
- **Its stop:** the end of fd 0, a read that answers its end or fails,
  is a first termination signal (`box-supervision.md`, 1, *Stops*): `temper`'s quit, which
  closes its end, and `temper`'s death, whose end the kernel closes, both
  stop the box, on Linux and on macOS alike, with no parent-death signal.
  `box.log` says `Stopping` as at any first termination signal. A
  termination signal is as `box-supervision.md`, 1 says, and one after fd 0's end is a second,
  which closes the steps.
- **A give up:** the box stops its steps as `box-supervision.md`, 1 says, writes `gave-up
  STEP` and the give up's `log` lines on fd 4, and exits 1. Its parent
  keeps what it said (`../../client/tui/box.md`).
- **Its exit:** 0 after a stop, 1 after a give up or a failure (`box-supervision.md`, 1), 2
  when it refused its arguments or its descriptors.
- **Where it lives.** The binary (`temper-box-shell`) checks `--front`'s
  descriptors, draws the local link's secret and URL, and gives
  temper-box-machine's host the URL, its secret and the descriptors in
  its settings. The host writes fd 3 in the first block's place and the
  box's lines on fd 4 with `temper_box_machine::front`, which owns their
  words and bytes, reads fd 0 beside its signals, and routes the
  terminal's lines to `box.log`. No front child, no front's log and no
  foreground taken back: those went with the box's own TUI.

## 2. Open questions

- **A `temper box` started in the background of a terminal** (`box-front.md`,
  1): `temper box &` from a shell with job control has a terminal on its
  standard input and output, so it runs its TUI, which job control stops
  at its first read. `temper-box` runs the box alone. Telling a
  background start from a foreground one needs `temper` to read its
  terminal's foreground group, which skein's `runtime/shell.md`, 6.3 does
  not give yet.

- **The terminal after `temper box` killed outright** (`box-front.md`, 1): its
  TUI's modes and screen stay on the terminal, as any program's killed
  outright do (skein's `runtime/shell.md`, 6.3); the box stops at its
  life pipe's end.
